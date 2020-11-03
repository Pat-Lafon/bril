use crate::cfg::{Cfg, Graph};
use std::io::{Write};
use std::process::{Command, Stdio};

pub fn write_graph_to_file(graph: &Graph, location: &str) {
    let process = match Command::new("dot")
        .args(&["-Tpdf", "-o", location])
        .stdin(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn wc: {}", why),
        Ok(process) => process,
    };
     match process
        .stdin
        .unwrap()
        .write_all(graph.to_dot().as_bytes())
    {
        Err(why) => panic!("couldn't write to wc stdin: {}", why),
        Ok(_) => (),
    }
}

pub fn write_graphs_to_file(graphs: &Cfg, location: &str) {
    let process = match Command::new("dot")
        .args(&["-Tpdf", "-o", location])
        .stdin(Stdio::piped())
        .spawn()
    {
        Err(why) => panic!("couldn't spawn wc: {}", why),
        Ok(process) => process,
    };
    match process
        .stdin
        .unwrap()
        .write_all(graphs.function_graphs[0].graph.to_dot().as_bytes())
    {
        Err(why) => panic!("couldn't write to wc stdin: {}", why),
        Ok(_) => (),
    }
}
