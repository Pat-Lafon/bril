use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::Instruction;
use crate::worklist::Constraints;

use std::collections::HashSet;

fn transfer(mut in_constraint: HashSet<String>, block: &BasicBlock) -> HashSet<String> {
    if block.code.len() == 0 {
        return in_constraint;
    }

    block.code.iter().rev().for_each(|x| match x {
        Instruction::Constant { dest, .. } => {
            in_constraint.remove(&dest.to_string());
        }
        Instruction::Effect { args: Some(a), .. } => {
            a.into_iter().for_each(|x| {
                in_constraint.insert(x.to_string());
            });
        }
        Instruction::Effect { args: None, .. } => (),
        Instruction::Value {
            dest,
            args: Some(a),
            ..
        } => {
            in_constraint.remove(&dest.to_string());
            a.into_iter().for_each(|x| {
                in_constraint.insert(x.to_string());
            });
        }
        Instruction::Value {
            dest, args: None, ..
        } => {
            in_constraint.remove(&dest.to_string());
        }
    });
    in_constraint
}

fn meet(vec_of_sets: Vec<HashSet<String>>) -> HashSet<String> {
    match vec_of_sets
        .into_iter()
        .fold_first(|a, b| (a.union(&b).into_iter().map(|x| x.to_string())).collect())
    {
        Some(s) => s,
        None => HashSet::new(),
    }
}

fn liveness(graph: &Graph) -> Constraints<HashSet<String>> {
    graph.worklist_algo(|_| HashSet::new(), transfer, meet, false)
}

fn remove_dead(block: &mut BasicBlock, mut live_out: HashSet<String>) {
    if block.code.len() == 0 {
        return;
    }

    block.code = block
        .code
        .clone()
        .into_iter()
        .rev()
        .filter(|x| match x {
            Instruction::Constant { dest, .. } if !live_out.contains(dest) => false,
            Instruction::Constant { dest, .. } => {
                live_out.remove(dest);
                true
            }
            Instruction::Effect { args: Some(a), .. } => {
                a.into_iter().for_each(|x| {
                    live_out.insert(x.to_string());
                });
                true
            }
            Instruction::Effect { args: None, .. } => true,
            Instruction::Value { dest, .. } if !live_out.contains(dest) => false,
            Instruction::Value {
                dest,
                args: Some(a),
                ..
            } => {
                live_out.remove(dest);
                a.into_iter().for_each(|x| {
                    live_out.insert(x.to_string());
                });
                true
            }
            Instruction::Value {
                dest, args: None, ..
            } => {
                live_out.remove(dest);
                true
            }
        })
        .collect::<Vec<Instruction>>()
        .into_iter()
        .rev()
        .collect();
}

fn dce_graph_worklist(graph: &mut Graph) {
    let live = liveness(graph);
    graph
        .vertices
        .iter_mut()
        .for_each(|(k, b)| remove_dead(b, live.get_in_const(k).clone()));
}

impl Cfg {
    pub fn do_dce(&mut self) {
        self.function_graphs
            .iter_mut()
            .for_each(|x| dce_graph_worklist(&mut x.graph));
    }
}
