use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::{ConstOps, Instruction, Literal, ValueOps};

use std::collections::HashMap;

#[derive(Debug, Clone)]
enum LvnValue {
    Const(Literal),
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

fn const_args(
    args: Vec<String>,
    var2num: &HashMap<String, u32>,
    table: &Vec<(LvnValue, String)>,
) -> Option<Vec<Literal>> {
    let mut result = Vec::new();
    for i in args.into_iter() {
        if let (LvnValue::Const(l), _) = &table[*var2num.get(&i).unwrap() as usize] {
            result.push(l.clone())
        } else {
            return None;
        }
    }
    Some(result)
}

fn update_args(
    args: Vec<String>,
    var2num: &HashMap<String, u32>,
    table: &Vec<(LvnValue, String)>,
) -> Option<Vec<String>> {
    let mut should_update = false;
    let mut result = Vec::new();
    for i in args.into_iter() {
        let (_, v) = &table[*var2num.get(&i).unwrap() as usize];
        result.push(v.to_string());
        should_update = should_update || i != v.to_string();
    }
    if should_update {
        Some(result)
    } else {
        None
    }
}

// I really want the let chains here in guard statements
fn convert_args(op: ValueOps, args: Vec<Literal>) -> Option<Literal> {
    match op {
        ValueOps::Add => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Int(i + i2));
                }
            }
        }
        ValueOps::Sub => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Int(i - i2));
                }
            }
        }
        ValueOps::Mul => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Int(i * i2));
                }
            }
        }
        ValueOps::Div => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    if i2 != 0 {
                        return Some(Literal::Int(i / i2));
                    }
                }
            }
        }
        ValueOps::Eq => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Bool(i == i2));
                }
            }
        }
        ValueOps::Lt => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Bool(i < i2));
                }
            }
        }
        ValueOps::Gt => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Bool(i > i2));
                }
            }
        }
        ValueOps::Le => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Bool(i <= i2));
                }
            }
        }
        ValueOps::Ge => {
            if let Literal::Int(i) = args[0] {
                if let Literal::Int(i2) = args[1] {
                    return Some(Literal::Bool(i >= i2));
                }
            }
        }
        ValueOps::Not => {
            if let Literal::Bool(i) = args[0] {
                return Some(Literal::Bool(!i));
            }
        }
        ValueOps::And => {
            if let Literal::Bool(i) = args[0] {
                if let Literal::Bool(i2) = args[1] {
                    return Some(Literal::Bool(i && i2));
                }
            }
        }
        ValueOps::Or => {
            if let Literal::Bool(i) = args[0] {
                if let Literal::Bool(i2) = args[1] {
                    return Some(Literal::Bool(i || i2));
                }
            }
        }
        ValueOps::Id => {
            if args.len() == 1 {
                return Some(args[0].clone());
            }
        }
        ValueOps::Call | ValueOps::Phi => return None,
    }
    None
}

fn update_instruction(
    instr: &Instruction,
    var2num: &HashMap<String, u32>,
    table: &Vec<(LvnValue, String)>,
) -> Option<Instruction> {
    match instr {
        Instruction::Constant { .. } => None,
        Instruction::Effect { args: None, .. } => None,
        Instruction::Effect {
            op,
            args: Some(args),
            funcs,
            labels,
        } => {
            if let Some(new_args) = update_args(args.clone(), var2num, table) {
                Some(Instruction::Effect {
                    op: op.clone(),
                    args: Some(new_args),
                    funcs: funcs.clone(),
                    labels: labels.clone(),
                })
            } else {
                None
            }
        }
        Instruction::Value {
            op: op @ ValueOps::Call,
            dest,
            op_type,
            args: Some(args),
            funcs,
            labels,
        } => {
            if let Some(new_args) = update_args(args.clone(), var2num, table) {
                Some(Instruction::Value {
                    op: op.clone(),
                    dest: dest.clone(),
                    op_type: op_type.clone(),
                    args: Some(new_args),
                    funcs: funcs.clone(),
                    labels: labels.clone(),
                })
            } else {
                None
            }
        }
        // These cases could be cleaned up when if_let_guard langs
        Instruction::Value {
            op,
            dest,
            op_type,
            args: Some(args),
            funcs,
            labels,
        } => {
            if let Some(consts) = const_args(args.clone(), var2num, table) {
                if let Some(i) = convert_args(op.clone(), consts) {
                    Some(Instruction::Constant {
                        op: ConstOps::Const,
                        dest: dest.clone(),
                        const_type: op_type.clone(),
                        value: i,
                    })
                } else {
                    Some(Instruction::Value {
                        op: op.clone(),
                        dest: dest.clone(),
                        op_type: op_type.clone(),
                        args: Some(args.clone()),
                        funcs: funcs.clone(),
                        labels: labels.clone(),
                    })
                }
            } else if let Some(new_args) = update_args(args.clone(), var2num, table) {
                Some(Instruction::Value {
                    op: op.clone(),
                    dest: dest.clone(),
                    op_type: op_type.clone(),
                    args: Some(new_args),
                    funcs: funcs.clone(),
                    labels: labels.clone(),
                })
            } else {
                None
            }
        }
        Instruction::Value { args: None, .. } => None,
    }
}

// todo I think this is only needed in local value numbering because we haven't handled redefinition
fn clear_dest(dest: &String, table: &mut Vec<(LvnValue, String)>) {
    match table.iter().position(|(_, x)| dest == x) {
        Some(i) => table[i] = (LvnValue::Op(ValueOps::Not, Vec::new()), "%%%".to_string()),
        None => (),
    }
}

fn lvn_basic_block(block: &mut BasicBlock) {
    if block.code.len() == 0 {
        return;
    }
    let mut index_num = 0;
    // Variable name to index
    let mut var2num: HashMap<String, u32> = HashMap::new();
    // Canonical value to destination variable
    // Index to this pair
    let mut table: Vec<(LvnValue, String)> = Vec::new();
    for instr_num in 0..(block.code.len() - 1) {
        let mut i = block.code[instr_num].clone();
        if let Some(new_i) = update_instruction(&i, &var2num, &table) {
            block.code[instr_num] = new_i.clone();
            i = new_i
        }
        match LvnValue::from(i.clone(), &var2num) {
            Some(v @ LvnValue::Op(_, _)) => match find_value(&v, &table) {
                Some((_, s)) => {
                    var2num.insert(i.get_dest().unwrap(), *var2num.get(&s).unwrap());
                    block.code[instr_num] = Instruction::Value {
                        op_type: i.get_type().unwrap(),
                        op: ValueOps::Id,
                        dest: i.get_dest().unwrap(),
                        args: Some(vec![s]),
                        funcs: None,
                        labels: None,
                    }
                }
                None => {
                    var2num.insert(i.get_dest().unwrap(), index_num);
                    clear_dest(&i.get_dest().unwrap(), &mut table);
                    table.insert(index_num as usize, (v, i.get_dest().unwrap()));
                    index_num += 1;
                }
            },
            Some(v @ LvnValue::Const(_)) => match find_value(&v, &table) {
                Some((_, s)) => {
                    var2num.insert(i.get_dest().unwrap(), *var2num.get(&s).unwrap());
                }
                None => {
                    var2num.insert(i.get_dest().unwrap(), index_num);
                    clear_dest(&i.get_dest().unwrap(), &mut table);
                    table.insert(index_num as usize, (v, i.get_dest().unwrap()));
                    index_num += 1;
                }
            },
            // Could do a replacement here for effect instructions
            None => {
                if let Instruction::Effect {
                    op,
                    args: Some(args),
                    funcs,
                    labels,
                } = i
                {
                    match update_args(args.clone(), &var2num, &table) {
                        Some(new_args) => {
                            block.code[instr_num] = Instruction::Effect {
                                op: op.clone(),
                                args: Some(new_args),
                                funcs: funcs.clone(),
                                labels: labels.clone(),
                            }
                        }
                        None => (),
                    }
                } else {
                    panic!("Why have we ended up here? I thought if we couldn't convert to a lvn that it had to be an effect")
                }
            }
        }
    }
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
