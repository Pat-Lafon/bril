#![crate_type = "lib"]
#![feature(option_unwrap_none)]
#![feature(vec_remove_item)]
#![feature(or_patterns)]
#![feature(map_into_keys_values)]
#![feature(iterator_fold_self)]
#![feature(drain_filter)]
pub mod cfg;
pub mod dce;
pub mod lvn;
pub mod program;
pub mod ssa;
pub mod worklist;
