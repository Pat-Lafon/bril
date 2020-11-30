use crate::cfg::{BasicBlock, Cfg, FunctionGraph, Graph};
use crate::dominator::dominators;
use crate::program::Code;
use crate::program::{Argument, EffectOps, Instruction, Type, ValueOps};
use crate::reaching_defs::reaching_defs;
use crate::worklist::Constraints;

use std::collections::HashSet;
use std::{collections::HashMap, fmt::format};

fn loop_invariant(
    var: &String,
    block: u32,
    reaching_defs: &HashMap<u32, HashMap<String, HashSet<u32>>>,
    current_loop: &Vec<u32>,
) -> bool {
    let default = HashSet::new();
    let defs = reaching_defs.get(&block).unwrap().get(var).unwrap_or(&default);
    if defs
        .intersection(&current_loop.iter().cloned().collect())
        .max()
        .is_none()
    {
        return true;
    } else {
        return false;
    }
    // There's another condition that a variable is loop invariant if there is only one definition of it and that definition instruction is loop invariant. For the purposes of code hoisting, I think this can be found recursively
}

fn loop_invariant_instr(
    code: &Instruction,
    block: u32,
    reaching_defs: &HashMap<u32, HashMap<String, HashSet<u32>>>,
    current_loop: &Vec<u32>,
) -> bool {
    match code.get_args().flatten() {
        Some(args) => args.iter().fold(true, |acc, var| {
            acc && loop_invariant(var, block, reaching_defs, current_loop)
        }),
        None => true,
    }
}

fn not_live_out_preheader(
    graph: &Graph,
    var: &String,
    pre_header_block: u32,
    reaching_defs: &HashMap<u32, HashMap<String, HashSet<u32>>>,
) -> bool {
    let default = HashSet::new();
    graph
        .vertices
        .get(&pre_header_block)
        .unwrap()
        .predecessor
        .iter()
        .position(|pred_idx| {
            !reaching_defs
                .get(pred_idx)
                .unwrap()
                .get(var)
                .unwrap_or(&default)
                .is_empty()
        })
        .is_none()
}

fn dominates_all_exits(
    graph: &Graph,
    current_block: u32,
    current_loop: &Vec<u32>,
    dominators: &HashMap<u32, HashSet<u32>>,
) -> bool {
    current_loop
        .iter()
        .position(|block_idx| {
            graph
                .vertices
                .get(block_idx)
                .unwrap()
                .successor
                .to_vec()
                .into_iter()
                .position(|suc_idx| !current_loop.contains(&suc_idx))
                .is_some()
                && !dominators.get(&block_idx).unwrap().contains(&current_block)
        })
        .is_none()
}

fn add_instruction(b: &mut Vec<Instruction>, c: Instruction) {
    let jump_instr = b.pop().unwrap();
    b.push(c);
    b.push(jump_instr);
}

// https://www.cs.cornell.edu/courses/cs6120/2019fa/blog/loop-reduction/#strength-reduction
fn licm(graph: &mut Graph) {
    let mut back_edges = Vec::new();
    let dominators = dominators(graph);
    for (idx, doms) in dominators.out_constraints.clone() {
        for dominator in doms.iter() {
            if graph
                .vertices
                .get(&idx)
                .unwrap()
                .successor
                .to_vec()
                .contains(dominator)
            {
                back_edges.push((idx, dominator.clone()));
            }
        }
    }

    let mut all_loops = Vec::new();
    for (a, b) in back_edges.into_iter() {
        let mut nat_loop = vec![b];
        let mut seen = HashSet::new();
        seen.insert(b);

        let mut queue = vec![a];

        while let Some(x) = queue.pop() {
            if !seen.contains(&x) {
                seen.insert(x);
                nat_loop.push(x);

                let mut preds = graph.vertices.get(&x).unwrap().predecessor.clone();
                queue.append(&mut preds);
            }
        }

        all_loops.push(nat_loop);
    }

    let reaching_defs = reaching_defs(graph).out_constraints;

    all_loops.into_iter().for_each(|loop_vec| {
        let header = loop_vec.get(0).unwrap();
        let mut preds = graph.vertices.get(header).unwrap().predecessor.clone();
        let preds = preds
            .iter()
            .filter(|x| !loop_vec.contains(&x))
            .copied()
            .collect();
        let new_block_label = format!("loop_preheader_{}", header);
        let new_block_num = graph.insert_block_between(new_block_label, *header, preds);
        let mut header_block_code = graph.vertices.get(&new_block_num).unwrap().code.clone();

        let def_counts = loop_vec.iter().fold(HashMap::new(), |mut acc, idx| {
            graph
                .vertices
                .get(idx)
                .unwrap()
                .code
                .iter()
                .for_each(|i| match i.get_dest() {
                    Some(dest) => {
                        let num = acc.entry(dest).or_insert(0);
                        *num += 1
                    }
                    None => {}
                });
            acc
        });

        // check is loop invariant
        // check only has one definition
        // check that the predecessor's to the loop header don't have var as a reaching def
        // check that all exit blocks in the loop are dominated by the current block
        loop_vec.iter().for_each(|working_block_idx| {
            let mut working_block = graph.vertices.get(working_block_idx).unwrap().clone();
            working_block
                .code
                .drain_filter(|code| match code.get_dest() {
                    Some(dest) => {
                        if loop_invariant_instr(
                            &code,
                            *working_block_idx,
                            &reaching_defs,
                            &loop_vec,
                        ) && *def_counts.get(&dest).unwrap() == 1
                            && not_live_out_preheader(graph, &dest, new_block_num, &reaching_defs)
                            && dominates_all_exits(
                                graph,
                                *working_block_idx,
                                &loop_vec,
                                &dominators.out_constraints,
                            )
                        {

                                add_instruction(&mut header_block_code, code.clone());
                                true

                        } else {
                            false
                        }
                    }
                    None => false,
                });
            let new_working_bloc = graph.vertices.get_mut(working_block_idx).unwrap();
            new_working_bloc.code = working_block.code;
        });
        let new_header_block = graph.vertices.get_mut(&new_block_num).unwrap();
        new_header_block.code = header_block_code
    });
    //println!("{:?}",graph);
}

impl Cfg {
    pub fn do_licm(&mut self) {
        self.function_graphs
            .iter_mut()
            .for_each(|x| licm(&mut x.graph));
    }
}
