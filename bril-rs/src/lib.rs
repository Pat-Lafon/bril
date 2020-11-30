#![crate_type = "lib"]
#![feature(option_unwrap_none)]
#![feature(or_patterns)]
#![feature(vec_remove_item)]
#![feature(iterator_fold_self)]
#![feature(drain_filter)]
pub mod cfg;
pub mod dce;
pub mod dominator;
pub mod licm;
pub mod lvn;
pub mod program;
pub mod reaching_defs;
pub mod ssa;
pub mod worklist;
