use crate::cfg::{BasicBlock, Cfg, FunctionGraph, Graph};
use bril_rs::{Argument, Instruction, Ownership, Type, ValueOps};
use rand::random;
use std::collections::HashMap;

fn create_region() -> String {
    format!("{}_{}", random::<char>(), random::<u8>())
}

fn infer_type(ty: &Type, region: String) -> Type {
    match ty {
        Type::Int | Type::Bool | Type::Float => ty.clone(),
        Type::Pointer(ptr_type) => Type::PointerRegions {
            pointer_type: ptr_type.clone(),
            ownership: Some(Ownership::Borrower),
            region,
        },
        Type::PointerRegions {
            pointer_type,
            region,
            ownership,
        } => Type::PointerRegions {
            pointer_type: pointer_type.clone(),
            ownership: ownership.clone().or(Some(Ownership::Borrower)),
            region: region.clone(),
        },
    }
}

fn set_owner(ty: Type) -> Type {
    match ty {
        Type::PointerRegions {
            pointer_type,
            region,
            ownership: _,
        } => Type::PointerRegions {
            pointer_type: pointer_type,
            ownership: Some(Ownership::Owner),
            region,
        },
        _ => ty,
    }
}

fn get_region(ty: &Type) -> Option<String> {
    match ty {
        Type::PointerRegions {
            pointer_type,
            region,
            ownership: _,
        } => Some(region.to_string()),
        _ => None,
    }
}

fn infer_code(
    c: &Instruction,
    region: &String,
    ptr_map: &mut HashMap<String, String>,
) -> Instruction {
    use ValueOps::*;
    match c {
        Instruction::Constant { .. }
        | Instruction::Effect { .. }
        | Instruction::Value {
            op:
                Add | Sub | Mul | Div | Eq | Lt | Gt | Le | Ge | Not | And | Or | Fadd | Fsub | Fmul
                | Fdiv | Feq | Flt | Fgt | Fle | Fge,
            ..
        } => c.clone(),
        Instruction::Value {
            op: op @ (Id | Load | PtrAdd),
            dest,
            op_type,
            args,
            funcs,
            labels,
        } => Instruction::Value {
            op: op.clone(),
            dest: dest.clone(),
            op_type: match op_type {
                Type::Int | Type::Float | Type::Bool | Type::PointerRegions { .. } => {
                    op_type.clone()
                }
                Type::Pointer(ty) => {
                    let region = ptr_map.get(args.get(0).unwrap()).unwrap().to_string();
                    ptr_map.insert(dest.to_string(), region.clone());
                    Type::PointerRegions {
                        pointer_type: ty.clone(),
                        ownership: Some(Ownership::Borrower),
                        region,
                    }
                }
            },
            //infer_type(op_type, Some(region.to_string())),
            args: args.clone(),
            funcs: funcs.clone(),
            labels: labels.clone(),
        },

        Instruction::Value {
            op: Alloc,
            dest,
            op_type,
            args,
            funcs,
            labels,
        } => {
            let mut owned_type = infer_type(op_type, region.clone());
            owned_type = set_owner(owned_type);
            ptr_map.insert(dest.to_string(), region.clone());
            Instruction::Value {
                op: Alloc,
                dest: dest.clone(),
                op_type: owned_type,
                args: args.clone(),
                funcs: funcs.clone(),
                labels: labels.clone(),
            }
        }

        Instruction::Value {
            op: Call,
            dest,
            op_type,
            args,
            funcs,
            labels,
        } => {
            let mut owned_type = infer_type(op_type, region.clone());
            owned_type = set_owner(owned_type);
            ptr_map.insert(dest.to_string(), region.clone());
            Instruction::Value {
                op: Call,
                dest: dest.clone(),
                op_type: owned_type,
                args: args.clone(),
                funcs: funcs.clone(),
                labels: labels.clone(),
            }
        }
    }
}

fn infer_graph(
    mut graph: Graph,
    region: &String,
    mut ptr_map: HashMap<String, String>, // Name, Region
) -> Graph {
    let mut verts_done = Vec::new();
    let mut verts_to_do = vec![graph.starting_vertex];
    while let Some(block_idx) = verts_to_do.pop() {
        let block = graph.vertices.remove(&block_idx).unwrap();

        // do some work

        let inferred_block = BasicBlock {
            index: block.index,
            label: block.label,
            predecessor: block.predecessor.clone(),
            successor: block.successor.clone(),
            code: block
                .code
                .into_iter()
                .map(|c| infer_code(&c, region, &mut ptr_map))
                .collect(),
        };

        graph.vertices.insert(block_idx, inferred_block);

        verts_done.push(block_idx);
        let mut try_add = |x| {
            if !verts_done.contains(&x) && !verts_to_do.contains(&x) {
                verts_to_do.insert(0, x);
            }
        };
        block
            .successor
            .to_vec()
            .into_iter()
            .rev()
            .for_each(|x| try_add(x));
    }
    graph
}

fn infer_func(function: &FunctionGraph) -> FunctionGraph {
    let region = function.region.clone().unwrap_or_else(create_region);
    // Name, Region
    let mut ptr_map: HashMap<String, String> = HashMap::new();
    let args: Vec<Argument> = function
        .args
        .iter()
        .map(|a| {
            let temp_region = create_region();
            match &a.arg_type {
                Type::Int | Type::Bool | Type::Float => (),
                Type::Pointer(_) => {
                    ptr_map.insert(a.name.clone(), temp_region.clone());
                }
                Type::PointerRegions { region, .. } => {
                    ptr_map.insert(a.name.clone(), region.clone());
                }
            }

            Argument {
                name: a.name.clone(),
                arg_type: infer_type(&a.arg_type, temp_region),
            }
        })
        .collect();

    let graph = infer_graph(function.graph.clone(), &region, ptr_map);

    FunctionGraph {
        name: function.name.clone(),
        args,
        graph,
        return_type: function.return_type.clone(),
        region: Some(region),
    }
}

pub fn infer(program: &Cfg) -> Cfg {
    Cfg {
        function_graphs: program.function_graphs.iter().map(infer_func).collect(),
    }
}
