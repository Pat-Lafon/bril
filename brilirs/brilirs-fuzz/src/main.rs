use bril_rs::Program;
use brilirs::{basic_block, check, interp};
use afl::fuzz;

fn main() {
  fuzz!(|prog: Program| {
    if let Ok(bbprog) = basic_block::BBProgram::new(prog) {
      if let Ok(()) = check::type_check(&bbprog) {
        if let Ok(()) = interp::execute_main(&bbprog, std::io::stdout(), &vec![String::new()], true) {}
      }
    }
  });
}
