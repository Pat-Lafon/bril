use bril_rs::program::load_program;
use std::io::{self, Write};

fn main() {
    let program = load_program();
    //println!("{:?}", program);
    let graphs = program.to_cfg();
    //println!("{:?}", graphs);
    /* io::stdout()
        .write_all(graphs.function_graphs[0].graph.to_dot().as_bytes())
.unwrap(); */

    let result_program = graphs.to_program();
    io::stdout()
        .write_all(serde_json::to_string(&result_program).unwrap()
        .as_bytes()).unwrap();
}
