use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::Instruction;
use std::collections::HashMap;

fn new_constraints() {
    unimplemented!()
}

fn worklist_algo_helper () {
unimplemented!()
}

impl Graph {
    fn worklist_algo<B>(graph: &mut Graph, init : fn(BasicBlock) -> B, transfer : fn(B, BasicBlock) -> B, meet : fn(Vec<B>) -> B, forward : bool) -> (HashMap<u32, B>, HashMap<u32, B>) {
        //graph.vertices.values_mut().for_each(dce_basic_block);
        //let mut worklist = graph.vertices.keys().into_iter().collect();
        unimplemented!()
    }
}
