use std::{fs::File, io::Write, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src");
    rust_sitter_tool::build_parsers(&PathBuf::from("src/grammar.rs"));
    let mut g = File::create("grammar.json").unwrap();
    g.write(rust_sitter_tool::generate_grammars(&PathBuf::from("src/grammar.rs"))[0].as_bytes())
        .unwrap();
}
