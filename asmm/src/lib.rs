#![feature(or_patterns)]
#![feature(option_unwrap_none)]
#![feature(vec_remove_item)]
#![allow(deprecated)]
#![feature(iterator_fold_self)]
pub mod removal;
pub mod infer;
pub mod cfg;
pub mod worklist;
pub mod free;
pub mod dominator;