use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::{ConstOps, Instruction, Literal, ValueOps};

use std::collections::HashMap;

#[derive(Debug, Clone)]
enum LvnValue {
    Const(Literal),
    // TODO what about unary not
    Op(ValueOps, Vec<u32>),
}

impl PartialEq for LvnValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LvnValue::Const(i), LvnValue::Const(k)) => i == k,
            (LvnValue::Op(val, args1), LvnValue::Op(val2, args2)) => {
                val == val2 && args1.clone().sort() == args2.clone().sort()
            }
            (LvnValue::Const(..), LvnValue::Op(..)) | (LvnValue::Op(..), LvnValue::Const(..)) => {
                false
            }
        }
    }
}

impl LvnValue {
    fn from(item: Instruction, var2num: &HashMap<String, u32>) -> Option<Self> {
        match item {
            Instruction::Constant {
                op: ConstOps::Const,
                value,
                ..
            } => Some(LvnValue::Const(value)),
            Instruction::Value { op, args, .. } => Some(LvnValue::Op(
                op,
                args.unwrap_or(Vec::new())
                    .into_iter()
                    .map(|x| *var2num.get(&x).unwrap())
                    .collect(),
            )),
            Instruction::Effect { .. } => None,
        }
    }
}

fn find_value(val: &LvnValue, table: &Vec<(LvnValue, String)>) -> Option<(LvnValue, String)> {
    table
        .iter()
        .find(|(x, _)| val == x)
        .map(|(y, z)| ((*y).clone(), z.clone()))
}

fn lvn_basic_block(block: &mut BasicBlock) {
    let mut index_num = 0;
    // Variable name to index
    let mut var2num: HashMap<String, u32> = HashMap::new();
    // Canonical value to destination variable
    // Index to this pair
    let mut table: Vec<(LvnValue, String)> = Vec::new();
    for i in block.code.iter() {
        match LvnValue::from(i.clone(), &var2num) {
            Some(v) => match find_value(&v, &table) {
                Some((_, s)) => {
                    var2num.insert(i.get_dest().unwrap(), *var2num.get(&s).unwrap());
                } // More to do
                None => {
                    var2num.insert(i.get_dest().unwrap(), index_num);
                    table.insert(index_num as usize, (v, i.get_dest().unwrap()));
                    index_num += 1;
                }
            },
            None => (),
        }
    }
    println!("{:?}", table);
}

fn lvn_graph(graph: &mut Graph) {
    graph.vertices.values_mut().for_each(lvn_basic_block);
}

impl Cfg {
    pub fn do_lvn(&mut self) {
        self.function_graphs
            .iter_mut()
            .for_each(|x| lvn_graph(&mut x.graph));
    }
}
