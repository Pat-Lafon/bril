# brilm – Bril to MLIR Translator

Converts Bril programs to MLIR IR using the melior library.

## Quick Start

Have a recent install of LLVM with MLIR tools on your path.

Build with `cargo build`

Pass a Bril JSON program via stdin:

```bash
cargo run < program.json
```

## Output & Execution

The translator emits MLIR module text to stdout. To execute:

```bash
cargo run < program.json | mlir-opt -convert-arith-to-llvm -finalize-memref-to-llvm -convert-func-to-llvm -reconcile-unrealized-casts | mlir-translate -mlir-to-llvmir | lli
```

(Note: This doesn't fully work yet. Working with the different dialect
translations and needing special print/main handling from a runtime like in
brillvm)
