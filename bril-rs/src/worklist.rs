use crate::cfg::{BasicBlock, Graph};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Constraints<B: Clone + PartialEq + std::fmt::Debug> {
    in_constraints: HashMap<u32, B>,
    out_constraints: HashMap<u32, B>,
}

impl<B: Clone + PartialEq + std::fmt::Debug> Constraints<B> {
    pub fn get_in_const(&self, node: &u32) -> &B {
        self.in_constraints.get(node).unwrap()
    }
    pub fn set_in_const(&mut self, node: u32, constraint: B) {
        self.in_constraints.insert(node, constraint);
    }
    pub fn get_out_const(&self, node: &u32) -> &B {
        self.out_constraints.get(node).unwrap()
    }
    pub fn set_out_const(&mut self, node: u32, constraint: B) {
        self.out_constraints.insert(node, constraint);
    }
}

fn new_constraints<B: Clone + PartialEq + std::fmt::Debug>(
    graph: &mut Graph,
    worklist: &Vec<u32>,
    init: fn(&BasicBlock) -> B,
    transfer: fn(B, &mut BasicBlock) -> B,
) -> Constraints<B> {
    let mut in_constraints = HashMap::new();
    let mut out_constraints = HashMap::new();
    for i in worklist {
        let block = graph.vertices.get_mut(i).unwrap();
        let in_b = init(block);
        in_constraints.insert(*i, in_b.clone());
        out_constraints.insert(*i, transfer(in_b, block));
    }
    Constraints {
        in_constraints,
        out_constraints,
    }
}

fn worklist_algo_helper<B: Clone + PartialEq + std::fmt::Debug>(
    graph: &mut Graph,
    transfer: fn(B, &mut BasicBlock) -> B,
    meet: fn(Vec<B>) -> B,
    forward: bool,
    mut worklist: Vec<u32>,
    mut constraints: Constraints<B>,
) -> Constraints<B> {
    while let Some(node) = worklist.pop() {
        let mut block = graph.vertices.get_mut(&node).unwrap();
        let old_in_constraints = constraints.get_in_const(&node);
        let in_constraints = if forward {
            meet(
                block
                    .predecessor
                    .iter()
                    .map(|x| constraints.get_out_const(x).clone())
                    .collect(),
            )
        } else {
            meet(
                Into::<Vec<u32>>::into(&block.successor)
                    .iter()
                    .map(|x| constraints.get_out_const(x).clone())
                    .collect(),
            )
        };
        if *old_in_constraints != in_constraints {
            constraints.set_in_const(node, in_constraints.clone());
            let new_out_constraints = transfer(in_constraints, &mut block);
            constraints.set_out_const(node, new_out_constraints);

            for i in if forward {
                Into::<Vec<u32>>::into(&block.successor).into_iter()
            } else {
                block.predecessor.clone().into_iter()
            } {
                worklist.push(i)
            }
        }
    }
    constraints
}

impl Graph {
    pub fn worklist_algo<B: Clone + PartialEq + std::fmt::Debug>(
        &mut self,
        init: fn(&BasicBlock) -> B,
        transfer: fn(B, &mut BasicBlock) -> B,
        meet: fn(Vec<B>) -> B,
        forward: bool,
    ) -> Constraints<B> {
        //graph.vertices.values_mut().for_each(dce_basic_block);
        let worklist = self
            .vertices
            .clone()
            .into_keys()
            .into_iter()
            .collect::<Vec<u32>>();
        let constraints = new_constraints(self, &worklist, init, transfer);
        worklist_algo_helper(self, transfer, meet, forward, worklist, constraints)
    }
}
