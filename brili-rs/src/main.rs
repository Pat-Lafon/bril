use bril_rs::load_program;
use brili_rs::interp::eval_program;
use clap::{App, Arg};

fn main() {
    let args = App::new("brili-rs")
        .author("Patrick LaFontaine")
        .about("Interprets bril programs")
        .arg(Arg::with_name("profiling").short("p").takes_value(false))
        .arg(Arg::with_name("tracing").long("tracing").takes_value(false))
        .arg(Arg::with_name("arguments").multiple(true).allow_hyphen_values(true))
        .get_matches();

    let other_args = if args.is_present("arguments") {
        args.values_of("arguments").unwrap().map(|s| s.replace(",", "")).collect()
    } else {
        vec![]
    };

    // todo Point 3 of a well formed bril program is that the runtime type of a variable does not change within a function. I could probably go implement that

    let program = load_program();
    match eval_program(program, args.is_present("profiling"), args.is_present("tracing"), other_args) {
        Ok(()) => std::process::exit(0),
        Err(s) => {
            eprintln!("{}", s);
            std::process::exit(2)
        }
    }
}
