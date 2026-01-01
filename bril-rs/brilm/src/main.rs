mod translator;

use bril_rs::Program;
use melior::{
    Context, dialect::DialectRegistry, ir::operation::OperationLike, utility::register_all_dialects,
};
use std::io::Read;
use translator::translate_program;

melior::dialect! {
    name: "bril",
    files: ["../../brilir/include/bril/BrilDialect.td", "../../brilir/include/bril/BrilOps.td" , "../../brilir/include/bril/BrilPasses.td", "../../brilir/include/bril/BrilTypes.td"],
    include_directories: ["../../brilir/include"]
}

fn main() {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    // TODO: Proper dialect registration is gated on https://github.com/mlir-rs/melior/issues/718
    // Once that issue is resolved, we can properly register the Bril dialect to avoid unregistered dialect warnings
    context.set_allow_unregistered_dialects(true);

    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read input");

    let program: Program = serde_json::from_str(&buffer).expect("Failed to parse Bril program");
    let module = translate_program(&context, &program);

    let module_op = module.as_operation();
    if module_op.verify() {
        println!("{}", module_op);
    } else {
        eprintln!("Warning: Module verification failed");
        eprintln!("{}", module_op);
    }
}
