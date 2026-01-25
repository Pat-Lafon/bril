mod translator;

use bril_rs::Program;
use melior::{
    Context, dialect::DialectRegistry, ir::operation::OperationLike, utility::register_all_dialects,
};
use std::io::Read;
use translator::translate_program;

// Include the generated dialect registration code from melior-build
include!(concat!(env!("OUT_DIR"), "/bril_register.rs"));

fn main() {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    // Load the Bril dialect using the generated registration function
    load(&context);

    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .expect("Failed to read input");

    let program: Program = serde_json::from_str(&buffer).expect("Failed to parse Bril program");
    let module = translate_program(&context, &program);

    // Verify the module
    if !module.as_operation().verify() {
        eprintln!("Warning: Module verification failed");
        eprintln!("{}", module.as_operation());
        return;
    }

    println!("{}", module.as_operation());
}
