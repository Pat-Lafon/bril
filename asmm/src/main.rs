use asmm::infer::infer;
use asmm::cfg::to_cfg;
use asmm::removal::remove_regions;
use asmm::free::add_free_calls;
use bril_rs::{load_program, output_program};

// Assumptions:
// Don't allow alloc in non-dominating exit blocks
// Don't overwrite a variable holding an alloc
// If you are returning a pointer from a function, it owns it's memory

fn main() {
    // get the program
    let program = load_program();
    // Convert to cfg
    let cfg = to_cfg(program);
    // infer any regions
    let inferred = infer(&cfg);
    // insert free calls
    let freed = add_free_calls(inferred);
    // toss the regions
    // output
    output_program(&remove_regions(&freed.to_program()));

    //output_program(&freed.to_program());
}
