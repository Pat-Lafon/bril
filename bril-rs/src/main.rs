use bril_rs::program::load_program;
use clap::{App, Arg};
use std::io::{self, Write};

fn main() {
    let args = App::new("bril-rs")
        .author("Patrick LaFontaine")
        .about("Does things with bril programs.")
        //.arg(Arg::with_name("json").index(1).required(true))
        .arg(Arg::with_name("dce").long("dce").takes_value(false))
        .arg(Arg::with_name("lvn").long("lvn").takes_value(false))
        .get_matches();

    let program = load_program();
    //println!("{:?}", program);
    let mut graphs = program.to_cfg();
    //println!("{:?}", graphs);
    /* io::stdout()
                        .write_all(graphs.function_graphs[0].graph.to_dot().as_bytes())
    .unwrap(); */
    if args.is_present("lvn") {
        graphs.do_lvn();
    }

    if args.is_present("dce") {
        graphs.do_dce_worklist();
    }

    /* io::stdout()
                            .write_all(graphs.function_graphs[0].graph.to_dot().as_bytes())
    .unwrap(); */
    let result_program = graphs.to_program();

    io::stdout()
        .write_all(serde_json::to_string(&result_program).unwrap().as_bytes())
        .unwrap();
}
