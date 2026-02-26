#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
"""
Benchmark comparison script for brilirs performance improvements.

Uses git worktree to create a proper A/B comparison between the current
version and a baseline commit.

Usage:
    # Compare against parent commit
    uv run benchmark_compare.py

    # Compare against specific commit/branch
    uv run benchmark_compare.py --baseline main

    # Run all benchmarks
    uv run benchmark_compare.py --all

    # Run specific categories
    uv run benchmark_compare.py --category core float

    # Or run directly (shebang uses uv)
    ./benchmark_compare.py --baseline main

Requirements:
    - hyperfine
    - bril2json (in PATH)
    - git
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

BENCHMARK_DIRS = ["core", "float", "mem", "mixed", "long"]
REQUIRED_TOOLS = ["hyperfine", "bril2json", "git", "cargo"]


def check_requirements() -> bool:
    """Check that all required tools are available in PATH."""
    missing = []
    for tool in REQUIRED_TOOLS:
        if shutil.which(tool) is None:
            missing.append(tool)

    if missing:
        print("Missing required tools:", file=sys.stderr)
        for tool in missing:
            print(f"  - {tool}", file=sys.stderr)
        print(
            "\nPlease install the missing tools and ensure they are in PATH.",
            file=sys.stderr,
        )
        return False
    return True


def is_significant(
    baseline_mean: float,
    baseline_stddev: float,
    current_mean: float,
    current_stddev: float,
) -> bool:
    """
    Check if the difference between two measurements is statistically significant.

    Uses a simple heuristic: if the absolute difference is smaller than the
    measurement noise (the larger of the two standard deviations), the result
    is not significant.
    """
    diff = abs(baseline_mean - current_mean)
    noise = max(baseline_stddev, current_stddev)

    return diff > noise


@dataclass
class BenchmarkInfo:
    name: str
    path: Path
    args: str
    category: str


def run_cmd(
    cmd: list[str], cwd: Optional[Path] = None, check: bool = True
) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=check)


def get_current_commit() -> str:
    """Get the current commit hash."""
    result = run_cmd(["git", "rev-parse", "HEAD"])
    return result.stdout.strip()[:8]


def get_commit_message(ref: str) -> str:
    """Get the commit message for a ref."""
    result = run_cmd(["git", "log", "-1", "--format=%s", ref], check=False)
    return result.stdout.strip()[:50] if result.returncode == 0 else ref


def has_uncommitted_changes(path: Path) -> bool:
    """Check if the given path has uncommitted changes."""
    result = run_cmd(["git", "status", "--porcelain", str(path)], check=False)
    return bool(result.stdout.strip())


def setup_worktree(baseline_ref: str, worktree_path: Path) -> bool:
    """Create a git worktree for the baseline."""
    # Clean up if exists
    if worktree_path.exists():
        run_cmd(
            ["git", "worktree", "remove", "--force", str(worktree_path)], check=False
        )

    # Create worktree
    result = run_cmd(
        ["git", "worktree", "add", str(worktree_path), baseline_ref], check=False
    )
    if result.returncode != 0:
        print(f"Error creating worktree: {result.stderr}", file=sys.stderr)
        return False
    return True


def cleanup_worktree(worktree_path: Path):
    """Remove a git worktree."""
    run_cmd(["git", "worktree", "remove", "--force", str(worktree_path)], check=False)


def build_brilirs(brilirs_dir: Path) -> Optional[Path]:
    """Build brilirs in release mode and return path to binary."""
    print(f"  Building in {brilirs_dir}...", end=" ", flush=True)

    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=brilirs_dir,
        capture_output=True,
        text=True,
        env={**os.environ, "RUSTFLAGS": "-C target-cpu=native"},
    )

    if result.returncode != 0:
        print("FAILED")
        print(result.stderr, file=sys.stderr)
        return None

    binary_path = brilirs_dir / "target" / "release" / "brilirs"
    if not binary_path.exists():
        print("FAILED (binary not found)")
        return None

    print("OK")
    return binary_path


def build_pgo(brilirs_dir: Path, benchmarks_dir: Path) -> Optional[Path]:
    """Build PGO-optimized brilirs and return path to binary."""
    # Check for cargo-pgo
    if shutil.which("cargo-pgo") is None:
        print(
            "cargo-pgo not found. Install with: cargo install cargo-pgo",
            file=sys.stderr,
        )
        return None

    print("  Building instrumented binary...", end=" ", flush=True)
    result = subprocess.run(
        ["cargo", "pgo", "build"],
        cwd=brilirs_dir,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("FAILED")
        print(result.stderr, file=sys.stderr)
        return None
    print("OK")

    # Find instrumented binary (cargo-pgo puts it in target/<triple>/release/)
    try:
        find_result = subprocess.run(
            [
                "find",
                str(brilirs_dir / "target"),
                "-name",
                "brilirs",
                "-type",
                "f",
                "-path",
                "*/release/*",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        # Filter to get the one that's NOT in target/release (the instrumented one has a triple)
        candidates = [p for p in find_result.stdout.strip().split("\n") if p]
        instrumented_bin = next(
            (p for p in candidates if "/release/brilirs" in p),
            candidates[0] if candidates else None,
        )
        if not instrumented_bin:
            raise ValueError("No binary found")
    except (subprocess.CalledProcessError, IndexError, ValueError, StopIteration):
        print("Could not find instrumented binary", file=sys.stderr)
        return None

    # Profile directory used by cargo-pgo
    profile_dir = brilirs_dir / "target" / "pgo-profiles"
    profile_env = {
        **os.environ,
        "LLVM_PROFILE_FILE": str(profile_dir / "brilirs_%m_%p.profraw"),
    }

    # Run profiling benchmarks (multiple times for better profile data)
    print("  Collecting profile data...", end=" ", flush=True)
    profile_benchmarks = [
        ("core", "ackermann", "3 6"),
        ("core", "collatz", "100"),
        ("core", "primes-between", "1 1000"),
        ("mem", "quicksort", "100"),
        ("mem", "sieve", "100"),
        ("long", "function_call", "10"),
    ]
    for _ in range(3):  # Run multiple iterations for better coverage
        for category, name, bench_args in profile_benchmarks:
            bril_file = benchmarks_dir / category / f"{name}.bril"
            if bril_file.exists():
                with open(bril_file) as f:
                    bril2json = subprocess.run(
                        ["bril2json"], stdin=f, capture_output=True
                    )
                subprocess.run(
                    [instrumented_bin] + bench_args.split(),
                    input=bril2json.stdout,
                    capture_output=True,
                    env=profile_env,
                )
    print("OK")

    # Build optimized binary
    print("  Building PGO-optimized binary...", end=" ", flush=True)
    result = subprocess.run(
        ["cargo", "pgo", "optimize"],
        cwd=brilirs_dir,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print("FAILED")
        print(result.stderr, file=sys.stderr)
        return None

    binary_path = brilirs_dir / "target" / "release" / "brilirs"
    if not binary_path.exists():
        print("FAILED (binary not found)")
        return None

    print("OK")
    return binary_path


def parse_args_from_bril(bril_path: Path) -> str:
    """Extract ARGS from bril file comments."""
    try:
        with open(bril_path) as f:
            for line in f:
                match = re.match(r"#\s*ARGS:\s*(.*)", line)
                if match:
                    return match.group(1).strip()
    except Exception:
        pass
    return ""


def discover_benchmarks(
    benchmarks_dir: Path, categories: Optional[list] = None
) -> list[BenchmarkInfo]:
    """Discover all benchmark files."""
    benchmarks = []
    dirs_to_scan = categories if categories else BENCHMARK_DIRS

    for category in dirs_to_scan:
        category_dir = benchmarks_dir / category
        if not category_dir.exists():
            continue

        for bril_file in sorted(category_dir.glob("*.bril")):
            name = bril_file.stem
            args = parse_args_from_bril(bril_file)
            benchmarks.append(
                BenchmarkInfo(
                    name=name,
                    path=bril_file,
                    args=args,
                    category=category,
                )
            )

    return benchmarks


def convert_to_json(bril_path: Path, json_path: Path) -> bool:
    """Convert .bril file to JSON."""
    with open(bril_path) as bril_file:
        result = subprocess.run(
            ["bril2json"],
            stdin=bril_file,
            capture_output=True,
            text=True,
        )
    if result.returncode != 0:
        return False

    with open(json_path, "w") as json_file:
        json_file.write(result.stdout)
    return True


def run_comparison(
    baseline_bin: Path,
    current_bin: Path,
    json_path: Path,
    args: str,
    min_runs: int,
    warmup: int,
) -> Optional[dict]:
    """Run hyperfine comparison between two binaries."""
    baseline_cmd = f"{baseline_bin} -f {json_path}"
    current_cmd = f"{current_bin} -f {json_path}"
    if args:
        baseline_cmd += f" {args}"
        current_cmd += f" {args}"

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as f:
        result_path = f.name

    try:
        subprocess.run(
            [
                "hyperfine",
                "--warmup",
                str(warmup),
                "--min-runs",
                str(min_runs),
                "--export-json",
                result_path,
                "--shell=none",
                "-n",
                "baseline",
                baseline_cmd,
                "-n",
                "current",
                current_cmd,
            ],
            capture_output=True,
            check=True,
        )

        with open(result_path) as f:
            data = json.load(f)

        results = {}
        for r in data["results"]:
            results[r["command"]] = {
                "mean": r["mean"],
                "stddev": r["stddev"],
                "min": r["min"],
                "max": r["max"],
                "runs": len(r["times"]),  # Actual number of runs hyperfine performed
            }
        return results
    except subprocess.CalledProcessError:
        return None
    finally:
        if os.path.exists(result_path):
            os.unlink(result_path)


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark brilirs against a baseline",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                          # Compare against HEAD~1
  %(prog)s --baseline main          # Compare against main branch
  %(prog)s --baseline abc123        # Compare against specific commit
  %(prog)s --pgo                    # Compare release vs PGO-optimized
  %(prog)s --all                    # Run all benchmarks
  %(prog)s --category core float    # Run specific categories
        """,
    )
    parser.add_argument(
        "--baseline",
        type=str,
        default="HEAD~1",
        help="Baseline commit/branch to compare against (default: HEAD~1)",
    )
    parser.add_argument(
        "--pgo",
        action="store_true",
        help="Compare regular release vs PGO-optimized build",
    )
    parser.add_argument(
        "--min-runs",
        type=int,
        default=10,
        dest="min_runs",
        help="Minimum benchmark runs; hyperfine may run more (default: 10)",
    )
    parser.add_argument(
        "--warmup", type=int, default=3, help="Number of warmup runs (default: 3)"
    )
    parser.add_argument("--all", action="store_true", help="Run all benchmarks")
    parser.add_argument(
        "--category",
        nargs="*",
        choices=BENCHMARK_DIRS,
        help="Benchmark categories to run (default: core)",
    )
    parser.add_argument("--output", type=str, help="Output results to JSON file")
    args = parser.parse_args()

    # Check requirements before doing anything
    if not check_requirements():
        sys.exit(1)

    if args.pgo and shutil.which("cargo-pgo") is None:
        print(
            "--pgo requires cargo-pgo. Install with: cargo install cargo-pgo",
            file=sys.stderr,
        )
        sys.exit(1)

    script_dir = Path(__file__).parent.resolve()
    repo_root = script_dir.parent
    benchmarks_dir = repo_root / "benchmarks"

    # Determine mode: PGO or git worktree
    use_pgo_mode = args.pgo

    if use_pgo_mode:
        baseline_label = "release"
        current_label = "PGO-optimized"
    else:
        baseline_label = f"{args.baseline} ({get_commit_message(args.baseline)})"
        uncommitted = has_uncommitted_changes(script_dir)
        current_label = (
            "working directory"
            if uncommitted
            else f"{get_current_commit()} ({get_commit_message('HEAD')})"
        )

    print("=" * 70)
    print("BRILIRS BENCHMARK COMPARISON")
    print("=" * 70)
    print(f"Baseline: {baseline_label}")
    print(f"Current:  {current_label}")
    print(f"Min runs: {args.min_runs}, Warmup: {args.warmup}")
    print()

    # Discover benchmarks
    categories = (
        args.category if args.category else (BENCHMARK_DIRS if args.all else ["core"])
    )
    benchmarks = discover_benchmarks(benchmarks_dir, categories)

    if not benchmarks:
        print("No benchmarks found.", file=sys.stderr)
        sys.exit(1)

    print(f"Benchmarks to run: {len(benchmarks)}")
    print()

    if use_pgo_mode:
        # PGO mode: build release first, then PGO
        print("Building binaries...")
        baseline_bin = build_brilirs(script_dir)
        if not baseline_bin:
            print("Build failed!", file=sys.stderr)
            sys.exit(1)

        # Copy baseline before PGO overwrites it
        baseline_copy = Path(tempfile.mkdtemp()) / "brilirs-baseline"
        shutil.copy2(baseline_bin, baseline_copy)
        baseline_bin = baseline_copy

        current_bin = build_pgo(script_dir, benchmarks_dir)
        if not current_bin:
            print("PGO build failed!", file=sys.stderr)
            sys.exit(1)

    else:
        # In-place checkout mode: build both binaries from the same directory
        # to guarantee identical compilation (same paths, same metadata hashes).
        # 1. Build current version first, copy binary aside
        # 2. Stash changes, checkout baseline, build, copy binary
        # 3. Restore original state

        print("Building current version...")
        current_bin = build_brilirs(script_dir)
        if not current_bin:
            print("Build failed!", file=sys.stderr)
            sys.exit(1)

        # Copy current binary to temp location
        current_copy = Path(tempfile.mkdtemp()) / "brilirs-current"
        shutil.copy2(current_bin, current_copy)
        current_bin = current_copy

        # Save current state
        original_ref = run_cmd(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=repo_root
        ).stdout.strip()
        if original_ref == "HEAD":
            # Detached HEAD, save the hash instead
            original_ref = run_cmd(
                ["git", "rev-parse", "HEAD"], cwd=repo_root
            ).stdout.strip()

        # Stash any uncommitted changes (including untracked files in brilirs/src)
        stashed = False
        if has_uncommitted_changes(repo_root):
            print("Stashing uncommitted changes...")
            result = run_cmd(
                [
                    "git",
                    "stash",
                    "push",
                    "--include-untracked",
                    "-m",
                    "benchmark_compare auto-stash",
                ],
                cwd=repo_root,
                check=False,
            )
            stashed = result.returncode == 0

        try:
            # Checkout baseline
            print(f"Checking out baseline ({args.baseline})...")
            result = run_cmd(
                ["git", "checkout", args.baseline], cwd=repo_root, check=False
            )
            if result.returncode != 0:
                print(
                    f"Failed to checkout {args.baseline}: {result.stderr}",
                    file=sys.stderr,
                )
                sys.exit(1)

            print("Building baseline version...")
            baseline_bin = build_brilirs(script_dir)
            if not baseline_bin:
                print("Baseline build failed!", file=sys.stderr)
                sys.exit(1)

            # Copy baseline binary
            baseline_copy = Path(tempfile.mkdtemp()) / "brilirs-baseline"
            shutil.copy2(baseline_bin, baseline_copy)
            baseline_bin = baseline_copy

        finally:
            # Always restore original state
            print(f"Restoring {original_ref}...")
            run_cmd(["git", "checkout", original_ref], cwd=repo_root, check=False)
            if stashed:
                print("Restoring stashed changes...")
                run_cmd(["git", "stash", "pop"], cwd=repo_root, check=False)

    try:
        # Run benchmarks
        print("\nRunning benchmarks...")
        results = {}

        with tempfile.TemporaryDirectory() as tmpdir:
            for i, bench in enumerate(benchmarks, 1):
                print(
                    f"[{i}/{len(benchmarks)}] {bench.category}/{bench.name}...",
                    end=" ",
                    flush=True,
                )

                json_path = Path(tmpdir) / f"{bench.name}.json"

                if not convert_to_json(bench.path, json_path):
                    print("failed (conversion)")
                    continue

                result = run_comparison(
                    baseline_bin,
                    current_bin,
                    json_path,
                    bench.args,
                    args.min_runs,
                    args.warmup,
                )

                if result:
                    baseline_time = result["baseline"]["mean"] * 1000
                    current_time = result["current"]["mean"] * 1000
                    change = ((baseline_time - current_time) / baseline_time) * 100

                    significant = is_significant(
                        result["baseline"]["mean"],
                        result["baseline"]["stddev"],
                        result["current"]["mean"],
                        result["current"]["stddev"],
                    )

                    results[bench.name] = {
                        "category": bench.category,
                        "args": bench.args,
                        "baseline": result["baseline"],
                        "current": result["current"],
                        "change_pct": change,
                        "significant": significant,
                    }

                    if not significant:
                        print(
                            f"{baseline_time:.1f}ms -> {current_time:.1f}ms (not significant)"
                        )
                    else:
                        direction = "faster" if change > 0 else "slower"
                        print(
                            f"{baseline_time:.1f}ms -> {current_time:.1f}ms ({abs(change):.1f}% {direction})"
                        )
                else:
                    print("failed")

    finally:
        pass

    # Output JSON if requested
    if args.output:
        output_data = {
            "baseline_ref": baseline_label,
            "current_ref": current_label,
            "runs": args.min_runs,
            "results": results,
        }
        with open(args.output, "w") as f:
            json.dump(output_data, f, indent=2)
        print(f"\nResults saved to {args.output}")

    # Print comparison table
    print()
    print("=" * 90)
    print("RESULTS")
    print("=" * 90)

    # Group by category
    by_category = {}
    for name, data in results.items():
        cat = data["category"]
        if cat not in by_category:
            by_category[cat] = []
        by_category[cat].append((name, data))

    for category in BENCHMARK_DIRS:
        if category not in by_category:
            continue

        print(f"\n## {category.upper()}")
        print("-" * 90)
        print(
            f"{'Benchmark':<25} {'Baseline':>12} {'Current':>12} {'Change':>15} {'StdDev':>12}"
        )
        print("-" * 90)

        for name, data in sorted(by_category[category]):
            baseline_ms = data["baseline"]["mean"] * 1000
            current_ms = data["current"]["mean"] * 1000
            stddev_ms = data["current"]["stddev"] * 1000
            change = data["change_pct"]

            if not data["significant"]:
                change_str = "not significant"
            else:
                direction = "faster" if change > 0 else "slower"
                change_str = f"{abs(change):.1f}% {direction}"

            print(
                f"{name:<25} {baseline_ms:>10.2f}ms {current_ms:>10.2f}ms {change_str:>15} ±{stddev_ms:>8.2f}ms"
            )

    print("-" * 90)

    # Summary statistics
    if results:
        significant_results = {k: v for k, v in results.items() if v["significant"]}
        not_significant = len(results) - len(significant_results)

        sig_changes = [d["change_pct"] for d in significant_results.values()]
        faster = [c for c in sig_changes if c > 0]
        slower = [c for c in sig_changes if c < 0]

        print("\n## SUMMARY")
        print("-" * 50)
        print(f"Total benchmarks: {len(results)}")
        print(
            f"Faster: {len(faster)}, Slower: {len(slower)}, Not significant: {not_significant}"
        )
        if sig_changes:
            print(
                f"Average change (significant only): {sum(sig_changes) / len(sig_changes):+.1f}%"
            )
        if faster:
            print(f"Average speedup (where faster): {sum(faster) / len(faster):+.1f}%")
        print("-" * 50)

        # Top improvements (only significant)
        sorted_results = sorted(
            significant_results.items(), key=lambda x: x[1]["change_pct"], reverse=True
        )
        if sorted_results:
            print("\nTop 5 improvements:")
            for name, data in sorted_results[:5]:
                print(f"  {name}: {data['change_pct']:+.1f}%")

        if slower:
            print("\nRegressions:")
            for name, data in sorted_results[-len(slower) :]:
                if data["change_pct"] < 0:
                    print(f"  {name}: {data['change_pct']:+.1f}%")


if __name__ == "__main__":
    main()
