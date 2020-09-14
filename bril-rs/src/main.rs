extern crate bril_rs;
use bril_rs::cfg::convert_to_cfg;
use bril_rs::program::load_program;
use std::io::{self, Write};

fn main() {
    let program = load_program();
    //println!("{:?}", program);
    let graphs = convert_to_cfg(program);
    //println!("{:?}", graphs);
    io::stdout().write_all(graphs.function_graphs[0].graph.to_dot().as_bytes()).unwrap();
}
