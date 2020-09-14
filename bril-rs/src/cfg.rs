use crate::program::{Argument, Code, EffectOps, Instruction, Program, Type};
use std::collections::HashMap;
use std::convert::From;

#[derive(Debug)]
pub struct Cfg {
    pub function_graphs: Vec<FunctionGraph>,
}

#[derive(Debug)]
pub struct FunctionGraph {
    pub name: String,
    pub args: Option<Vec<Argument>>,
    pub return_type: Option<Type>,
    pub graph: Graph,
}

#[derive(Debug)]
pub struct Graph {
    pub name: String,
    pub vertices: HashMap<u32, BasicBlock>,
    pub label_map: HashMap<String, u32>, // I'm not sure if I need this but I'll hold on to it for ease of use
}

impl Graph {
    pub fn to_dot(&self) -> String {
        let nodes = self
            .vertices
            .keys()
            .map(|x| {
                format!(
                    "\t{} [label=\"{}\"];\n",
                    x,
                    String::from(self.vertices.get(x).unwrap())
                )
            })
            .collect::<String>();
        let edges = self
            .vertices
            .keys()
            .map(|x| {
                let ends = self.vertices.get(x).unwrap();
                match ends.successor {
                    Successor::End => "".to_string(),
                    Successor::Jump(i) => format!("\t{} -> {};\n", x, i),
                    Successor::Conditional {
                        true_branch,
                        false_branch,
                    } => format!("\t{} -> {};\n\t{} -> {};\n", x, true_branch, x, false_branch),
                }
            })
            .collect::<String>();
        format!("digraph {} {{\n{}{}}}", self.name, nodes, edges)
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub label: Option<String>,
    pub code: Vec<Instruction>,
    //pub predecessor: Option<u32>,
    pub successor: Successor,
}

impl Default for BasicBlock {
    fn default() -> BasicBlock {
        BasicBlock {
            label: None,
            code: Vec::new(),
            //predecessor: None,
            successor: Successor::End,
        }
    }
}

impl From<&BasicBlock> for String {
    fn from(item: &BasicBlock) -> Self {
        let mut block: Vec<Code> = item
            .code
            .iter()
            .map(|x| Code::Instruction(x.clone()))
            .collect();
        match &item.label {
            Some(l) => block.insert(
                0,
                Code::Label {
                    label: l.to_string(),
                },
            ),
            None => (),
        }
        block
            .into_iter()
            .map(|x| String::from(x))
            .collect::<Vec<String>>()
            .join(";\n")
    }
}

#[derive(Debug)]
pub enum Successor {
    End,
    Jump(u32),
    Conditional { true_branch: u32, false_branch: u32 },
}

pub fn convert_to_cfg(p: Program) -> Cfg {
    Cfg {
        function_graphs: p
            .functions
            .into_iter()
            .map(|x| FunctionGraph {
                name: x.name.clone(),
                args: x.args,
                return_type: x.return_type,
                graph: create_graph(x.instrs, x.name),
            })
            .collect(),
    }
}

// todo originally I was going to do graph making and blocking together but the mental overhead was high. For now, I'm going make blocks of code first and then make the graph. This can be combined later
fn make_blocks(code: Vec<Code>) -> (Vec<(Vec<Code>, Successor)>, HashMap<String, u32>, u32) {
    let mut result: Vec<(Vec<Code>, Successor)> = Vec::new();

    let mut label_map = HashMap::new();
    let mut index_acc = 0;

    let mut current_code: Vec<Code> = Vec::new();

    let mut get_number = |label: String| -> u32 {
        match label_map.get(&label) {
            Some(i) => *i,
            None => {
                let x = index_acc;
                label_map.insert(label.clone(), x);
                index_acc += 1;
                x
            }
        }
    };

    for i in code.into_iter() {
        match i {
            instr @ Code::Label { .. } if current_code.len() == 0 => current_code.push(instr),
            Code::Label { label } => {
                result.push((current_code, Successor::Jump(get_number(label.clone()))));
                current_code = vec![Code::Label { label }]
            }
            instr @ Code::Instruction(Instruction::Constant { .. }) => current_code.push(instr),
            instr @ Code::Instruction(Instruction::Value { .. }) => current_code.push(instr),
            instr
            @
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Call,
                ..
            }) => {
                current_code.push(instr);
            }
            instr
            @
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Print,
                ..
            }) => {
                current_code.push(instr);
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Return,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.is_none());
                debug_assert!(args.is_none() || args.as_ref().unwrap().len() == 1);
                debug_assert!(funcs.is_none());
                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Return,
                    labels,
                    args,
                    funcs,
                }));
                result.push((current_code, Successor::End));
                current_code = Vec::new();
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Jump,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.as_ref().unwrap().len() == 1);
                debug_assert!(args.is_none());
                debug_assert!(funcs.is_none());

                let target = labels.as_ref().unwrap()[0].clone();
                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Jump,
                    labels,
                    args,
                    funcs,
                }));
                result.push((current_code, Successor::Jump(get_number(target))));
                current_code = Vec::new();
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Branch,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.as_ref().unwrap().len() == 2);
                debug_assert!(args.as_ref().unwrap().len() == 1);
                debug_assert!(funcs.is_none());

                let t_branch = get_number(labels.as_ref().unwrap()[0].clone());
                let f_branch = get_number(labels.as_ref().unwrap()[1].clone());

                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Branch,
                    labels,
                    args,
                    funcs,
                }));
                result.push((
                    current_code,
                    Successor::Conditional {
                        true_branch: t_branch,
                        false_branch: f_branch,
                    },
                ));
                current_code = Vec::new();
            }
        }
    }

    if current_code.len() != 0 {
        result.push((current_code, Successor::End));
    }
    (result, label_map, index_acc)
}

fn create_graph(code: Vec<Code>, name: String) -> Graph {
    let mut vertices: HashMap<u32, BasicBlock> = HashMap::new();

    let (blocks_n_parts, label_map, mut index_acc) = make_blocks(code);

    for (mut b, s) in blocks_n_parts.into_iter() {
        let mut block = BasicBlock::default();
        let vert = match b[0].clone() {
            Code::Label { label } => {
                b.remove(0);
                block.label = Some(label.to_string());
                *label_map.get(&label).unwrap()
            }
            _ => {
                let x = index_acc;
                index_acc += 1;
                x
            }
        };
        block.code = b
            .into_iter()
            .map(|x| match x {
                Code::Label { .. } => panic!("I wasn't expecting there to be a label in a block"),
                Code::Instruction(x) => x,
            })
            .collect();
        block.successor = s;
        vertices.insert(vert, block).unwrap_none();
    }

    Graph {
        name,
        vertices,
        label_map,
    }
}
