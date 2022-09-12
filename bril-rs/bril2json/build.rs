use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::io::Seek;

fn main() -> Result<(), Box<dyn Error>> {
    // All of this file manipulation is a big hack
    // Lalrpop does not allow #[cfg(..)] information because it has to build a full parser lookup table
    // It also does not allow compile time manipulation of the return type
    // So we need to compile two different versions of the parser to support with and without importing
    // We need two different files because features are additive so cargo will sometimes unexplainably use a previous
    // build it has lying around even if the feature set does not match. There also isn't a great way to force a
    // rebuild of bril2json each time.
    // Thus 2 parser modules, which have two different names in lib.rs which need to be feature gated.

    let mut out_file1 = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./src/bril_grammar.lalrpop")
        .unwrap();

    let mut first_file = OpenOptions::new()
        .read(true)
        .open("./src/grammar.lalrpop")
        .unwrap();

    let mut out_file2 = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("./src/bril_grammar_import.lalrpop")
        .unwrap();

    let mut second_file1 = OpenOptions::new()
        .read(true)
        .open("./src/without_imports.lalrpop")
        .unwrap();

    let mut second_file2 = OpenOptions::new()
        .read(true)
        .open("./src/with_imports.lalrpop")
        .unwrap();


    io::copy(&mut first_file, &mut out_file1).unwrap();
    first_file.rewind().unwrap();
    io::copy(&mut first_file, &mut out_file2).unwrap();

    io::copy(&mut second_file1, &mut out_file1).unwrap();
    io::copy(&mut second_file2, &mut out_file2).unwrap();

    let mut config = lalrpop::Configuration::new();
    config.generate_in_source_tree();
    config.process_file("./src/bril_grammar.lalrpop").unwrap();

    config.process_file("./src/bril_grammar_import.lalrpop")
}
