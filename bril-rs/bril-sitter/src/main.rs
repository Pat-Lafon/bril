/* use bril2json::bril_grammar;
fn main() {
    dbg!(bril_grammar::parse("")).unwrap();
    dbg!(bril_grammar::parse("from \"test\" import;")).unwrap();
    dbg!(bril_grammar::parse("from \"test\" import @Foo;")).unwrap();
    dbg!(bril_grammar::parse("from \"test\" import @Foo as @MyFoo;")).unwrap();
    dbg!(bril_grammar::parse(
        "from \"test\" import @Foo as @MyFoo, @Bar;"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "from \"test\" import @Foo as @MyFoo, @Bar as @MyBar;"
    ))
    .unwrap();
    dbg!(bril_grammar::parse("@main {}")).unwrap();
    dbg!(bril_grammar::parse("@main () {}")).unwrap();
    dbg!(bril_grammar::parse("@main (i : int) {}")).unwrap();
    dbg!(bril_grammar::parse("@main (i : int, i2 : int) {}")).unwrap();
    dbg!(bril_grammar::parse("@main : int {}")).unwrap();
    dbg!(bril_grammar::parse("@main (i : int, i2 : int) : int {}")).unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        .label1:
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        .label1:\n
        .label2:
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        x = const 0;
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        x : int = const 0;
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        x = const true;
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        x : bool = const false;
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        x = const 0.0;
    \n}"
    ))
    .unwrap();
    dbg!(bril_grammar::parse(
        "@main {\n
        x = const -0.0;
    \n}"
    ))
    .unwrap();
} */

use bril2json::cli::Cli;
use bril2json::parse_abstract_program;
use bril_rs::output_abstract_program;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    output_abstract_program(&parse_abstract_program(
        args.position >= 1,
        args.position >= 2,
        args.file,
    ))
}
