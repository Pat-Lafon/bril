//! Build script for brilm - generates Bril dialect registration code from TableGen files.

use melior_build::DialectBuilder;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bril_root = manifest_dir.parent().unwrap().parent().unwrap();

    // brilir TableGen files and include directory
    let brilir_include = bril_root.join("brilir/include");
    let brilir_td = brilir_include.join("bril");

    // Local C++ files (headers and verifier implementations)
    let dialect_dir = manifest_dir.join("dialect");

    DialectBuilder::new("bril")
        .td_file(brilir_td.join("BrilDialect.td"))
        .td_file(brilir_td.join("BrilTypes.td"))
        .td_file(brilir_td.join("BrilOps.td"))
        .cpp_file(dialect_dir.join("BrilOpsImpl.cpp"))
        .include_dir(&brilir_include)
        .include_dir(&dialect_dir)
        .cpp_namespace("mlir::bril")
        .build()
        .expect("Failed to build bril dialect");
}
