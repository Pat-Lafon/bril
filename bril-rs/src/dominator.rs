use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::{Argument, Instruction, Type, ValueOps};
use crate::worklist::Constraints;

use std::collections::HashSet;
use std::{collections::HashMap, mem::transmute_copy};

fn transfer(
    mut in_constraint: HashSet<u32>,
    block: &BasicBlock,
    starting_index: u32,
) -> HashSet<u32> {
    if block.index == starting_index {
        let mut x = HashSet::new();
        x.insert(block.index);
        x
    } else {
        in_constraint.insert(block.index);
        in_constraint
    }
}

fn meet(vec_of_sets: Vec<HashSet<u32>>) -> HashSet<u32> {
    match vec_of_sets.into_iter().fold_first(|a, b| {
        if a.is_empty() {
            b
        } else if b.is_empty() {
            a
        } else {
            a.intersection(&b).copied().collect()
        }
    }) {
        Some(s) => s,
        None => HashSet::new(),
    }
}

pub fn dominators(graph: &mut Graph) -> Constraints<HashSet<u32>> {
    graph.worklist_algo(
        &|b| {
            if b.index == graph.starting_vertex {
                HashSet::new()
            } else {
                graph.vertices.keys().cloned().collect()
            }
        },
        &|a, b| transfer(a, b, graph.starting_vertex),
        &meet,
        true,
    )
}
