use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::Instruction;

use std::collections::HashSet;

fn dce_basic_block(block: &mut BasicBlock) {
    let mut used_vars = HashSet::new();

    if block.code.len() == 0 {
        return;
    }

    for instr_num in (0..block.code.len() - 1).rev() {
        match block.code[instr_num].clone() {
            Instruction::Constant { dest, .. } if !used_vars.contains(&dest) => {
                block.code.remove(instr_num);
            }
            Instruction::Constant { dest, .. } => {
                used_vars.remove(&dest);
            }
            Instruction::Effect { args: Some(a), .. } => {
                a.into_iter().for_each(|x| {
                    used_vars.insert(x);
                });
            }
            Instruction::Effect { args: None, .. } => (),
            Instruction::Value { dest, .. } if !used_vars.contains(&dest) => {
                block.code.remove(instr_num);
            }
            Instruction::Value {
                dest,
                args: Some(a),
                ..
            } => {
                used_vars.remove(&dest);
                a.into_iter().for_each(|x| {
                    used_vars.insert(x);
                });
            }
            Instruction::Value {
                dest, args: None, ..
            } => {
                used_vars.remove(&dest);
            }
        }
    }
}

fn dce_graph(graph: &mut Graph) {
    graph.vertices.values_mut().for_each(dce_basic_block);
}

impl Cfg {
    pub fn do_dce(&mut self) {
        self.function_graphs
            .iter_mut()
            .for_each(|x| dce_graph(&mut x.graph));
    }
}
