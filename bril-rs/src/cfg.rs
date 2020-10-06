use crate::program::{Argument, Code, EffectOps, Function, Instruction, Program, Type};
use std::collections::HashMap;
use std::convert::From;

macro_rules! final_label {
    () => {
        "%%%%%THIS_IS_THE_END%%%%%".to_string()
    };
}

#[derive(Debug)]
pub struct Cfg {
    pub function_graphs: Vec<FunctionGraph>,
}

impl Program {
    pub fn to_cfg(self) -> Cfg {
        Cfg {
            function_graphs: self
                .functions
                .into_iter()
                .map(|x| FunctionGraph {
                    name: x.name.clone(),
                    args: x.args,
                    return_type: x.return_type,
                    graph: create_graph(x.instrs, x.name).do_prune(),
                })
                .collect(),
        }
    }
}

impl Cfg {
    pub fn to_program(self) -> Program {
        Program {
            functions: self
                .function_graphs
                .into_iter()
                .map(|x| Function {
                    name: x.name,
                    args: x.args,
                    return_type: x.return_type,
                    instrs: x.graph.create_trace(),
                })
                .collect(),
        }
    }
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
    pub starting_vertex: u32,
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
                // todo somehow clean this up with to_vec() and a fold/map
                match ends.successor {
                    Successor::End => "".to_string(),
                    Successor::Jump(i) => format!("\t{} -> {};\n", x, i),
                    Successor::Conditional {
                        true_branch,
                        false_branch,
                    } => format!(
                        "\t{} -> {};\n\t{} -> {};\n",
                        x, true_branch, x, false_branch
                    ),
                }
            })
            .collect::<String>();
        format!("digraph {} {{\n{}{}}}", self.name, nodes, edges)
    }

    // This is the trivial implementation of tracing
    // I am going to assume that I added jump instructions to basic blocks that didn't have them but fall through
    pub fn create_trace(mut self) -> Vec<Code> {
        let mut code = Vec::new();
        let mut verts_done = Vec::new();
        let mut verts_to_do = vec![self.starting_vertex];
        while let Some(block_idx) = verts_to_do.pop() {
            let block = self.vertices.remove(&(block_idx)).unwrap();
            if let Some(label) = block.label {
                // We are going to take the last instruction off of the list, look at it, and if it is a jump to the label we are about to add, keep it off the list. Otherwise we will add it back on.
                if let Some(instr) = code.pop() {
                    match instr {
                        Code::Instruction(Instruction::Effect {
                            op: EffectOps::Jump,
                            labels,
                            ..
                        }) if labels.clone().unwrap()[0] == label => (),
                        _ => code.push(instr),
                    }
                }
                code.push(Code::Label { label })
            }
            code.append(
                &mut block
                    .code
                    .into_iter()
                    .map(|x| Code::Instruction(x))
                    .collect(),
            );
            verts_done.push(block_idx);
            let mut try_add = |x| {
                if !verts_done.contains(&x) && !verts_to_do.contains(&x) {
                    if let Successor::Jump(i) = self.vertices.get(&x).unwrap().successor {
                        if self.label_map.get(&final_label!()).unwrap() == &i {
                            verts_to_do.insert(0, x)
                        } else {
                            verts_to_do.push(x)
                        }
                    } else {
                        verts_to_do.push(x)
                    }
                }
            };
            block
                .successor
                .to_vec()
                .into_iter()
                .rev()
                .for_each(|x| try_add(x));
        }
        match code.pop() {
            None => {}
            Some(Code::Label { label }) if label == "%%%%%THIS_IS_THE_END%%%%%".to_string() => {}
            Some(x) => code.push(x),
        }
        code
    }

    // I'm basically going to start at the starting block and find all the blocks I can reach from there. Then delete all the blocks that get missed
    pub fn do_prune(mut self) -> Self {
        let mut verts: Vec<u32> = self.vertices.keys().copied().collect();
        let mut worklist = vec![verts.remove_item(&self.starting_vertex).unwrap()];
        while let Some(idx) = worklist.pop() {
            self.vertices
                .get(&idx)
                .unwrap()
                .successor
                .to_vec()
                .into_iter()
                .for_each(|x| match verts.remove_item(&x) {
                    Some(i) => worklist.push(i),
                    None => (),
                })
        }
        verts
            .into_iter()
            .for_each(|x| match self.vertices.remove(&x) {
                None => panic!("whoops, Was there an invalid block or something?"),
                Some(b) => {
                    b.successor
                        .to_vec()
                        .into_iter()
                        .for_each(|y| match self.vertices.get_mut(&y) {
                            Some(s) => {
                                s.predecessor.remove_item(&x);
                            }
                            None => (),
                        })
                }
            });

        self
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: Option<String>,
    pub index: u32,
    pub code: Vec<Instruction>,
    pub predecessor: Vec<u32>,
    pub successor: Successor,
}

impl Default for BasicBlock {
    fn default() -> BasicBlock {
        BasicBlock {
            label: None,
            index: 0,
            code: Vec::new(),
            predecessor: Vec::new(),
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
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(";\n")
    }
}

#[derive(Debug, Clone)]
pub enum Successor {
    End,
    Jump(u32),
    Conditional { true_branch: u32, false_branch: u32 },
}

impl Successor {
    pub fn to_vec(&self) -> Vec<u32> {
        self.into()
    }
}

impl From<&Successor> for Vec<u32> {
    fn from(item: &Successor) -> Self {
        match item {
            Successor::End => Vec::new(),
            Successor::Jump(i) => vec![*i],
            Successor::Conditional {
                true_branch,
                false_branch,
            } => vec![*true_branch, *false_branch],
        }
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
                current_code.push({
                    Code::Instruction(Instruction::Effect {
                        op: EffectOps::Jump,
                        labels: Some(vec![label.clone()]),
                        args: None,
                        funcs: None,
                    })
                });
                result.push((current_code, Successor::Jump(get_number(label.clone()))));
                current_code = vec![Code::Label { label }]
            }
            instr @ Code::Instruction(Instruction::Constant { .. }) => current_code.push(instr),
            instr @ Code::Instruction(Instruction::Value { .. }) => current_code.push(instr),

            Code::Instruction(Instruction::Effect {
                op: EffectOps::Call,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.is_none());
                debug_assert!(funcs.as_ref().unwrap().len() == 1);
                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Call,
                    labels,
                    args,
                    funcs,
                }));
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Store,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.is_none());
                debug_assert!(args.as_ref().unwrap().len() == 2);
                debug_assert!(funcs.is_none());
                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Store,
                    labels,
                    args,
                    funcs,
                }));
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Free,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.is_none());
                debug_assert!(args.as_ref().unwrap().len() == 1);
                debug_assert!(funcs.is_none());
                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Free,
                    labels,
                    args,
                    funcs,
                }));
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Print,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.is_none());
                debug_assert!(funcs.is_none());
                current_code.push(Code::Instruction(Instruction::Effect {
                    op: EffectOps::Print,
                    labels,
                    args,
                    funcs,
                }));
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
            // I'm just going to ignore nop's
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Nop,
                labels,
                args,
                funcs,
            }) => {
                debug_assert!(labels.is_none());
                debug_assert!(args.is_none());
                debug_assert!(funcs.is_none());
            }
        }
    }

    if current_code.len() != 0 {
        let final_label = final_label!();
        current_code.push({
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Jump,
                labels: Some(vec![final_label.clone()]),
                args: None,
                funcs: None,
            })
        });
        result.push((
            current_code,
            Successor::Jump(get_number(final_label.clone())),
        ));
        result.push((vec![Code::Label { label: final_label }], Successor::End));
    }
    (result, label_map, index_acc)
}

fn add_back_edges(graph: &mut HashMap<u32, BasicBlock>) {
    let indices: Vec<u32> = graph.keys().map(|x| *x).collect();
    for i in indices.into_iter() {
        graph
            .get(&i)
            .unwrap()
            .successor
            .to_vec()
            .into_iter()
            .for_each(|x| graph.get_mut(&x).unwrap().predecessor.push(i))
    }
}

fn create_graph(code: Vec<Code>, name: String) -> Graph {
    let mut vertices: HashMap<u32, BasicBlock> = HashMap::new();

    let (blocks_n_parts, label_map, mut index_acc) = make_blocks(code);

    let mut starting_vertex = None;

    for (mut b, s) in blocks_n_parts.into_iter() {
        let mut block = BasicBlock::default();
        let vert = match b[0].clone() {
            Code::Label { label } => {
                b.remove(0);
                block.label = Some(label.to_string());
                match label_map.get(&label) {
                    Some(l) => *l,
                    None => {
                        let x = index_acc;
                        index_acc += 1;
                        x
                    }
                }
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
        block.index = vert;
        vertices.insert(vert, block).unwrap_none();
        // TODO this is a hack-y way to do this but we will leave it for now
        if starting_vertex.is_none() {
            starting_vertex = Some(vert);
        }
    }

    add_back_edges(&mut vertices);

    Graph {
        name,
        starting_vertex: starting_vertex.unwrap(),
        vertices,
        label_map,
    }
}
