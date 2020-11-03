use bril_rs::helper::write_graphs_to_file;
use bril_rs::program::load_program;
use clap::{App, Arg};
use std::fs;
use std::io::{self, Write};

fn main() {
    let args = App::new("bril-rs")
        .author("Patrick LaFontaine")
        .about("Does things with bril programs.")
        .arg(Arg::with_name("graph").long("graph").takes_value(false))
        .arg(Arg::with_name("dce").long("dce").takes_value(false))
        .arg(Arg::with_name("lvn").long("lvn").takes_value(false))
        .arg(
            Arg::with_name("fix_names")
                .long("fix_names")
                .takes_value(false),
        )
        .get_matches();

    let program = load_program();
    //println!("{:?}", program);
    let mut graphs = program.to_cfg();
    //println!("{:?}", graphs);
    /* io::stdout()
                           .write_all(graphs.function_graphs[0].graph.to_dot().as_bytes())
       .unwrap();

       return;
    */

    if args.is_present("graph") {
        let _ = fs::create_dir("graph");
        write_graphs_to_file(&graphs, "graph/cfg_init.pdf");
    }
    graphs.to_ssa();
    if args.is_present("graph") {
        write_graphs_to_file(&graphs, "graph/cfg_ssa.pdf");
    }
    /* io::stdout()
    .write_all(graphs.function_graphs[0].graph.to_dot().as_bytes())
    .unwrap(); */

    if args.is_present("lvn") {
        graphs.do_lvn();
    }

    if args.is_present("dce") {
        graphs.do_dce();
    }

    if args.is_present("graph") {
        write_graphs_to_file(&graphs, "graph/cfg_opt.pdf");
    }

    //graphs.from_ssa();

    if args.is_present("fix_names") {
        graphs.fix_variable_names();
    }

    let result_program = graphs.to_program();

    io::stdout()
        .write_all(serde_json::to_string(&result_program).unwrap().as_bytes())
        .unwrap();
}
