#!/bin/bash
# Profile-Guided Optimization build using cargo-pgo
#
# Prerequisites:
#   cargo install cargo-pgo
#   bril2json in PATH
#
# Usage:
#   ./pgo.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARKS="$SCRIPT_DIR/../benchmarks"
cd "$SCRIPT_DIR"

# Check for cargo-pgo
if ! command -v cargo-pgo &> /dev/null; then
    echo "cargo-pgo not found. Install with: cargo install cargo-pgo" >&2
    exit 1
fi

# Check for bril2json
if ! command -v bril2json &> /dev/null; then
    echo "bril2json not found. Install from bril-txt/" >&2
    exit 1
fi

# Run a benchmark
run_bench() {
    local category="$1"
    local name="$2"
    shift 2
    local args="$@"

    local bril_file="$BENCHMARKS/$category/$name.bril"
    if [ -f "$bril_file" ]; then
        echo "  $category/$name $args"
        bril2json < "$bril_file" | "$BRILIRS" $args > /dev/null
    fi
}

echo "=== Step 1: Build instrumented binary ==="
cargo pgo build

# Find the instrumented binary
BRILIRS=$(find "$SCRIPT_DIR/target" -name brilirs -type f -path "*/release/*" 2>/dev/null | head -1)
if [ -z "$BRILIRS" ] || [ ! -f "$BRILIRS" ]; then
    echo "Could not find instrumented brilirs binary" >&2
    exit 1
fi

echo ""
echo "=== Step 2: Collect profile data ==="
echo "Binary: $BRILIRS"

# Core benchmarks
run_bench core ackermann 3 6
run_bench core collatz 100
run_bench core primes-between 1 1000
run_bench core sum-sq-diff 100

# Float benchmarks
run_bench float leibniz 1000
run_bench float euler 100

# Memory benchmarks
run_bench mem bubblesort
run_bench mem mat-mul 10
run_bench mem quicksort 100
run_bench mem sieve 100

# Mixed benchmarks
run_bench mixed eight-queens

# Long benchmarks
run_bench long function_call 10

echo ""
echo "=== Step 3: Build optimized binary ==="
cargo pgo optimize

echo ""
echo "=== Done ==="
echo "Optimized binary: target/release/brilirs"
