use bril_rs::Program;
use brilirs::{basic_block, check, interp};

use honggfuzz::fuzz;

fn main() {
  loop {
    fuzz!(|prog: Program| {
      if let Ok(bbprog) = basic_block::BBProgram::new(prog) {
        if let Ok(()) = check::type_check(&bbprog) {
          if let Ok(()) = interp::execute_main(&bbprog, std::io::stdout(), &vec![], true) {}
        }
      }
    });
  }
}
