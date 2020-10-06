use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::worklist::Constraints;

use std::collections::HashMap;
use std::collections::HashSet;

fn transfer(mut in_constraint: HashSet<u32>, block: &mut BasicBlock) -> HashSet<u32> {
    in_constraint.insert(block.index);
    in_constraint
}

fn meet(vec_of_sets: Vec<HashSet<u32>>) -> HashSet<u32> {
    match vec_of_sets.into_iter().fold_first(|a, b| {
        if a.is_empty() {
            b
        } else {
            if b.is_empty() {
                a
            } else {
                a.intersection(&b).copied().collect()
            }
        }
    }) {
        Some(s) => s,
        None => HashSet::new(),
    }
}

fn dominators(graph: &mut Graph) -> Constraints<HashSet<u32>> {
    graph.worklist_algo(|_| HashSet::new(), transfer, meet, true)
}

fn dominator_tree(graph: &mut Graph) -> HashMap<u32, Vec<u32>> {
    let mut idom = HashMap::new();
    let constraints = dominators(graph).in_constraints;

    for (idx, doms) in &constraints {
        if idx == &graph.starting_vertex {
            // The starting vertex is undominated
        } else {
            idom.entry(
                *doms
                    .iter()
                    .max_by(|x, y| {
                        constraints
                            .get(x)
                            .unwrap()
                            .len()
                            .cmp(&constraints.get(y).unwrap().len())
                    })
                    .unwrap(),
            )
            .or_insert(Vec::new())
            .push(*idx);
        }
    }

    /* println!();
    println!("{:?}", graph);
    println!();
    println!("{:?}", idom);
    println!(); */

    idom
}

fn dominance_frontier(graph: &mut Graph) -> HashMap<u32, Vec<u32>> {
    let idom = dominator_tree(graph);
    unimplemented!();
}

impl Cfg {
    pub fn do_dominator_tree(&mut self) {
        self.function_graphs.iter_mut().for_each(|x| {
            dominator_tree(&mut x.graph);
        });
    }
}
