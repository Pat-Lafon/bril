use crate::cfg::{BasicBlock, Cfg, FunctionGraph, Graph, Successor};
use crate::dominator::dominators;
use bril_rs::{EffectOps, Instruction, Ownership, Type, ValueOps};

fn get_freeable_pointers(code: &Vec<Instruction>) -> Vec<String> {
    code.iter()
        .filter_map(|c| match c {
            Instruction::Value {
                op: ValueOps::Alloc,
                op_type:
                    Type::PointerRegions {
                        ownership: Some(Ownership::Owner),
                        ..
                    },
                dest,
                ..
            } => Some(dest.to_string()),
            Instruction::Value {
                op: ValueOps::Call,
                op_type:
                    Type::PointerRegions {
                        ownership: Some(Ownership::Owner),
                        ..
                    },
                dest,
                ..
            } => Some(dest.to_string()),
            Instruction::Value { .. }
            | Instruction::Constant { .. }
            | Instruction::Effect { .. } => None,
        })
        .collect()
}

fn free_graph(mut graph: Graph) -> Graph {
    let dominators = dominators(&mut graph).out_constraints;

    let mut end_blocks = Vec::new();

    // Find all of the exits for the function. The region dominates all of these so this is where we need to add the free calls.
    graph.vertices.values().for_each(|block| match block {
        BasicBlock {
            label: _,
            index,
            code: _,
            predecessor: _,
            successor: Successor::End,
        } => end_blocks.push(index),
        _ => (),
    });

    // For each end block, find all of the pointers
    let mut frees_to_add: Vec<(u32, Vec<Instruction>)> = end_blocks
        .into_iter()
        .map(|end| {
            (
                *end,
                dominators
                    .get(end)
                    .unwrap()
                    .iter()
                    .fold(Vec::new(), |mut acc, x| {
                        acc.append(
                            &mut get_freeable_pointers(&graph.vertices.get(x).unwrap().code)
                                .into_iter()
                                .map(|var| Instruction::Effect {
                                    op: EffectOps::Free,
                                    args: vec![var],
                                    funcs: Vec::new(),
                                    labels: Vec::new(),
                                })
                                .collect(),
                        );
                        acc
                    }),
            )
        })
        .collect();

    frees_to_add.iter_mut().for_each(|(end, free_instrs)| {
        let mut end_block = graph.vertices.get_mut(end).unwrap();
        let mut current_code = end_block.code.clone();
        if current_code.len() == 0 {
            // Empty block, just add our free calls and leave
            current_code.append(free_instrs);
        } else {
            let final_instr = current_code.pop().unwrap();
            match final_instr.clone() {
                // If there is a return statement, put the free calls before that
                Instruction::Effect {
                    op: EffectOps::Return,
                    args,
                    ..
                } => {
                    if args.len() > 0 {
                        // If we are returning the variable, don't free it!
                        // I'm guessing it is in the list but if it's not then nothing is removed
                        free_instrs.remove_item(&Instruction::Effect {
                            op: EffectOps::Free,
                            args: vec![args.get(0).unwrap().to_string()],
                            funcs: Vec::new(),
                            labels: Vec::new(),
                        });
                    }
                    current_code.append(free_instrs);
                    current_code.push(final_instr.clone());
                }
                _ => {
                    current_code.push(final_instr);
                    current_code.append(free_instrs);
                }
            }
        }
        end_block.code = current_code.to_vec();
    });

    Graph {
        name: graph.name,
        starting_vertex: graph.starting_vertex,
        vertices: graph.vertices,
        label_map: graph.label_map,
        num_blocks: graph.num_blocks,
    }
}

fn free_func(func: FunctionGraph) -> FunctionGraph {
    FunctionGraph {
        name: func.name,
        args: func.args,
        return_type: func.return_type,
        region: func.region,
        graph: free_graph(func.graph),
    }
}

pub fn add_free_calls(cfg: Cfg) -> Cfg {
    Cfg {
        function_graphs: cfg.function_graphs.into_iter().map(free_func).collect(),
    }
}
