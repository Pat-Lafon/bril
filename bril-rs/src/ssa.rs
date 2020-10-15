use crate::cfg::{BasicBlock, Cfg, Graph};
use crate::program::{Argument, Instruction, Type, ValueOps};
use crate::worklist::Constraints;
use cached::proc_macro::cached;
use cached::UnboundCache;

use std::collections::HashMap;
use std::collections::HashSet;

fn transfer(mut in_constraint: HashSet<u32>, block: &BasicBlock) -> HashSet<u32> {
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
    graph
        .vertices
        .keys()
        .for_each(|x| idom.insert(*x, Vec::new()).unwrap_none());
    let constraints = dominators(graph).in_constraints;

    for (idx, doms) in &constraints {
        // we need to check that there are actually dominators
        if doms.len() > 0 {
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
            .and_modify(|x| x.push(*idx));
        }
    }

    /* println!();
    println!("{:?}", graph);
    println!();
    println!("{:?}", idom);
    println!(); */

    idom
}

// This basically inverts the tree
// Whereas in a dominator tree a node pointed to a later node that it dominated,
// this tells you, given a child node, what was the parent node that immediately dominated it
fn immediate_dominators(dom_tree: &HashMap<u32, Vec<u32>>) -> HashMap<u32, u32> {
    let mut idom = HashMap::new();
    //println!("{:?}", dom_tree);
    for (idx, dom) in dom_tree {
        for node in dom {
            idom.insert(*node, *idx);
        }
    }

    idom
}

// With a map of nodes to their immediate dominators, we can construct a dominance frontier to see the extent of the dominance of a given node.
// todo pull out the dom_tree and idom parts of this and pass idom as an arg
fn dominance_frontier(
    graph: &mut Graph,
    dom_tree: &HashMap<u32, Vec<u32>>,
) -> HashMap<u32, HashSet<u32>> {
    let idom = immediate_dominators(&dom_tree);
    let mut frontier = HashMap::new();
    graph
        .vertices
        .keys()
        .for_each(|x| frontier.insert(*x, HashSet::new()).unwrap_none());

    // https://en.wikipedia.org/wiki/Static_single_assignment_form#Converting_to_SSA
    for (idx, immediate_dom) in idom.clone() {
        if graph.vertices.get(&idx).unwrap().predecessor.len() >= 2 {
            for p in graph.vertices.get(&idx).unwrap().predecessor.iter() {
                let mut runner = p;
                while runner != &immediate_dom {
                    frontier.entry(*runner).and_modify(|e| {
                        e.insert(idx);
                    });
                    runner = idom.get(runner).unwrap();
                }
            }
        }
    }

    frontier
}

// iterative dominance frontier
// The bible of SSA: http://ssabook.gforge.inria.fr/latest/book-full.pdf
// Basically the dominance frontier of a node unioned with the dominance frontier of all of those nodes... recursing till completion
// This finds all of the nodes that need to have a phi node for any given definition in the original node
// todo I should check at some point to see if this caching actually helps
#[cached(
    type = "UnboundCache<u32, HashSet<u32>>",
    create = "{ UnboundCache::new() }",
    convert = r#"{node}"#
)]
fn idf(dom_front: &HashMap<u32, HashSet<u32>>, node: u32) -> HashSet<u32> {
    let mut res = dom_front.get(&node).unwrap().clone();
    let mut worklist: Vec<u32> = res.clone().into_iter().collect();
    while let Some(s) = worklist.pop() {
        let temp = idf(dom_front, s);
        for i in temp.into_iter() {
            // if res did not contain i before
            if res.insert(i) {
                worklist.push(i);
            }
        }
    }
    res.clone()
}

fn do_var_rename(
    graph: &mut Graph,
    reaching_defs: &mut HashMap<String, Vec<String>>,
    dom_tree: &HashMap<u32, Vec<u32>>,
    current_node: u32,
    variable_count: &mut u32,
) {
    // updated_definitions
    let mut new_defs = Vec::new();
    // operate on current_node
    graph
        .vertices
        .get_mut(&current_node)
        .unwrap()
        .code
        .iter_mut()
        .for_each(|i| {
            if i.not_phi() {
                if let Some(Some(args)) = i.get_args() {
                    i.set_args(Some(
                        args.into_iter()
                            .map(|x| reaching_defs.get(&x).unwrap().last().unwrap().to_string())
                            .collect(),
                    ));
                }
            }
            if let Some(dest) = i.get_dest() {
                let renamed_var = format!("{}_{}", dest, variable_count);
                i.set_dest(renamed_var.clone());
                reaching_defs.get_mut(&dest).unwrap().push(renamed_var);
                *variable_count = *variable_count + 1;
                new_defs.push(dest);
            }
        });

    let succs = graph
        .vertices
        .get(&current_node)
        .unwrap()
        .successor
        .to_vec();

    let current_label = graph
        .vertices
        .get_mut(&current_node)
        .unwrap()
        .label
        .to_string();

    succs.clone().into_iter().for_each(|x| {
        graph
            .vertices
            .get_mut(&x)
            .unwrap()
            .code
            .iter_mut()
            .for_each(|i| match i.clone() {
                Instruction::Value {
                    op: ValueOps::Phi,
                    dest,
                    op_type,
                    args: Some(mut args),
                    funcs,
                    labels: Some(labels),
                } => {
                    let idx = labels.iter().position(|l| l == &current_label).unwrap();
                    args[idx] = reaching_defs
                        .get(&args[idx])
                        .unwrap()
                        .last()
                        .unwrap()
                        .to_string();
                    *i = Instruction::Value {
                        op: ValueOps::Phi,
                        dest,
                        op_type,
                        args: Some(args),
                        funcs,
                        labels: Some(labels),
                    };
                }
                _ => (),
            })
    });

    // Call on children of node
    dom_tree
        .get(&current_node)
        .unwrap()
        .clone()
        .into_iter()
        .for_each(|x| do_var_rename(graph, reaching_defs, dom_tree, x, variable_count));

    // do any finishing stuff
    new_defs.into_iter().for_each(|x| {
        reaching_defs.get_mut(&x).unwrap().pop().unwrap();
    });
}

// rename_vars
/*
foreach v : Variable do
    v.reachingDef←⊥
foreach BB: basic Block in depth-first search preorder traversal of the dom. tree do
    foreach i : instruction in linear code sequence of BB do
        foreach v : variable used by non-φ-function i do
            updateReachingDef(v,i)
            replace this use of v by v.reachingDef in i
        foreach v : variable defined by i (may be aφ-function)do
            updateReachingDef(v,i)
            create fresh variable v′
            replace this definition of v by v′ in i
            v′.reachingDef←v.reachingDef
            v.reachingDef←v′
    foreach φ:φ-function in a successor of BB do
        foreach v : variable used by φ do
            updateReachingDef(v,φ)
            replace this use of v by v.reachingDef in phi
*/

fn do_ssa(graph: &mut Graph, args: Vec<Argument>) {
    let dom_tree = dominator_tree(graph);
    let dom_front = dominance_frontier(graph, &dom_tree);
    // Get a Map of nodes to a set of variable names
    let mut need_phi: HashMap<u32, HashSet<String>> = HashMap::new();
    let mut all_vars = HashSet::new();
    // Get all blocks, for all blocks get all instructions and filter_map to dest, convert to set, union set with the set of each idf for that node
    for block in graph.vertices.values() {
        let reaching_defs: HashSet<String> = block
            .code
            .clone()
            .into_iter()
            .filter_map(|i| i.get_dest())
            .collect();
        all_vars = all_vars
            .union(&reaching_defs)
            .map(|x| x.to_string())
            .collect();
        for i in idf(&dom_front, block.index) {
            let phi_defs = need_phi.entry(i).or_insert(HashSet::new());
            *phi_defs = phi_defs
                .union(&reaching_defs)
                .map(|x| x.to_string())
                .collect();
        }
    }

    let idx_to_label: HashMap<u32, String> = graph
        .vertices
        .iter()
        .map(|(i, b)| (*i, b.label.to_string()))
        .collect();

    // For each node, add the phi instructions to the beginning
    for (idx, phi_defs) in need_phi {
        let block = graph.vertices.get_mut(&idx).unwrap();
        for i in phi_defs {
            block.code.insert(
                0,
                Instruction::Value {
                    op: ValueOps::Phi,
                    dest: i.clone(),
                    // todo figure out how to get the actual type
                    op_type: Type::Int,
                    args: Some(vec![i; block.predecessor.len()]),
                    funcs: None,
                    labels: Some(
                        block
                            .predecessor
                            .clone()
                            .into_iter()
                            .map(|x| idx_to_label.get(&x).unwrap().to_string())
                            .collect(),
                    ),
                },
            );
        }
    }

    // I am passing renaming off onto a function to do a recursive depth-first search on the dom-tree

    let mut reaching_defs = HashMap::new();
    all_vars.iter().for_each(|x| {
        reaching_defs
            .insert(x.to_string(), Vec::new())
            .unwrap_none()
    });
    let starting_node = graph.starting_vertex;
    // Args passed into a function are already defined and I'm not going to bother renaming them
    args.into_iter().for_each(|x| {
        reaching_defs.insert(x.name.clone(), vec![x.name]);
    });

    let mut variable_count = 1;
    do_var_rename(
        graph,
        &mut reaching_defs,
        &dom_tree,
        starting_node,
        &mut variable_count,
    )
}

fn undo_ssa(graph: &mut Graph) {
    // Will not contain all blocks
    let mut new_defs: HashMap<String, Vec<Instruction>> = HashMap::new();

    // remove phi nodes
    graph.vertices.iter_mut().for_each(|(_, b)| {
        {
            b.code.drain_filter(|i| match i {
                Instruction::Value {
                    op: ValueOps::Phi,
                    dest,
                    op_type,
                    args: Some(args),
                    labels: Some(labels),
                    ..
                } => {
                    args.into_iter()
                        .zip(labels.into_iter())
                        .for_each(|(arg, label)| {
                            new_defs.entry(label.clone()).or_insert(Vec::new()).push(
                                Instruction::Value {
                                    op: ValueOps::Id,
                                    dest: dest.clone(),
                                    op_type: op_type.clone(),
                                    args: Some(vec![arg.to_string()]),
                                    labels: None,
                                    funcs: None,
                                },
                            );
                        });
                    true
                }
                _ => false,
            })
        };
    });

    // Add in assignments
    graph.vertices.iter_mut().for_each(|(_, b)| {
        if let Some(def_code) = new_defs.get_mut(&b.label) {
            let jump = b.code.pop().unwrap();
            b.code.append(def_code);
            b.code.push(jump)
        }
    })
}

fn fix(s: String) -> String {
    (&s).strip_suffix(|x: char| x.is_numeric())
        .unwrap_or(&s)
        .strip_suffix("_")
        .unwrap_or(&s)
        .to_string()
}

fn fix_names(graph: &mut Graph) {
    graph.vertices.iter_mut().for_each(|(_, b)| {
        b.code.iter_mut().for_each(|i| {
            match i.get_dest() {
                Some(d) => i.set_dest(fix(d)),
                None => (),
            };
            match i.get_args() {
                Some(Some(args)) => i.set_args(Some(args.into_iter().map(fix).collect())),
                Some(None) | None => (),
            }
        })
    })
}

impl Cfg {
    pub fn to_ssa(&mut self) {
        self.function_graphs.iter_mut().for_each(|x| {
            let args = x.args.clone().unwrap_or(Vec::new());
            do_ssa(&mut x.graph, args);
        });
    }
    pub fn from_ssa(&mut self) {
        self.function_graphs
            .iter_mut()
            .for_each(|x| undo_ssa(&mut x.graph));
    }
    pub fn fix_variable_names(&mut self) {
        self.function_graphs
            .iter_mut()
            .for_each(|x| fix_names(&mut x.graph));
    }
}
