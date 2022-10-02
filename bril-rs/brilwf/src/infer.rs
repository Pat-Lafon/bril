use std::sync::atomic::Ordering::Relaxed;
use std::{collections::HashMap, sync::atomic::AtomicU64};

use bril_rs::{Code, ConstOps, Function, Instruction, Literal, Program, Type, ValueOps};

// I would prefer to use static mut here since I'm only considering the single threaded case but even that would still require unsafe code
// So I'm using Atomics instead and I believe this compiles to a mov instruction in my case.
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Eq, Clone)]
enum InferenceTypes {
    Bounded(Vec<Type>), // When a value can only be one of a few concrete types, probably Int/Float
    Poly(u64),          // Standing in for a polymorphic type
    Mono(Type),
    Unit,
}

struct TypeSignature {
    inputs: Vec<InferenceTypes>,
    output: InferenceTypes,
}

impl From<Option<Type>> for InferenceTypes {
    fn from(o: Option<Type>) -> Self {
        match o {
            Some(Type::Unknown) => Self::Poly(COUNTER.fetch_add(1, Relaxed)),
            Some(t) => Self::Mono(t),
            None => Self::Unit,
        }
    }
}

impl From<Type> for InferenceTypes {
    fn from(t: Type) -> Self {
        match t {
            Type::Unknown => Self::Poly(COUNTER.fetch_add(1, Relaxed)),
            _ => Self::Mono(t),
        }
    }
}

impl From<&Type> for InferenceTypes {
    fn from(t: &Type) -> Self {
        match t {
            Type::Unknown => Self::Poly(COUNTER.fetch_add(1, Relaxed)),
            _ => Self::Mono(t.clone()),
        }
    }
}

impl From<&Literal> for InferenceTypes {
    fn from(l: &Literal) -> Self {
        match l {
            Literal::Int(_) => InferenceTypes::Bounded(vec![Type::Int, Type::Float]),
            Literal::Bool(_) => InferenceTypes::Mono(Type::Bool),
            Literal::Float(_) => InferenceTypes::Mono(Type::Float),
        }
    }
}

/// Helper function of type_resolution
fn type_map_replace(
    type_map: &mut HashMap<&str, InferenceTypes>,
    poly_idx: u64,
    expected_type: InferenceTypes,
) {
    // we want to resolve all poly types of value u to this non-poly type
    type_map.iter_mut().for_each(|(_, t)| match t {
        InferenceTypes::Poly(i_1) if *i_1 == poly_idx => *t = expected_type.clone(),
        _ => (),
    })
}

fn type_resolution<'a>(
    type_map: &mut HashMap<&'a str, InferenceTypes>,
    var_name: &'a str,
    expected_type: InferenceTypes,
) {
    let current_type = type_map.get(var_name).cloned();
    match current_type {
        // No type recorded for this variable yet
        None => {
            type_map.insert(var_name, expected_type);
            ()
        }
        // The types are the same so we can move on
        Some(t) if t == expected_type => (),
        // var_name is a unit type but that's not what we got
        Some(InferenceTypes::Unit) => {
            panic!("current type for {var_name} is unit but expected {expected_type:?}")
        }
        // var_name is a bril_rs type
        Some(InferenceTypes::Mono(t)) => match expected_type {
            InferenceTypes::Poly(_) => todo!(),
            InferenceTypes::Bounded(_) => todo!(),
            // But it's not the same type
            InferenceTypes::Mono(t2) => {
                panic!("current type for {var_name} is {t:?} but expected {t2:?}")
            }
            InferenceTypes::Unit => panic!(),
        },
        Some(InferenceTypes::Bounded(_)) => todo!(),
        Some(InferenceTypes::Poly(i)) => match expected_type {
            InferenceTypes::Poly(u) => {
                // Poly types are always decreasing in index
                if i <= u {
                    type_map_replace(type_map, u, InferenceTypes::Poly(i))
                } else {
                    type_map_replace(type_map, i, InferenceTypes::Poly(u))
                }
            }
            _ => type_map_replace(type_map, i, expected_type),
        },
    }
}

fn collect_types<'a>(
    f: &'a Function,
    func_sigs: Vec<(&str, TypeSignature)>,
) -> HashMap<&'a str, InferenceTypes> {
    let mut type_map = f
        .args
        .iter()
        .map(|a| (a.name.as_ref(), a.arg_type.clone().into()))
        .collect();

    f.instrs.iter().for_each(|c| match c {
        Code::Label { .. } => (),
        Code::Instruction(Instruction::Constant {
            dest,
            op: ConstOps::Const,
            pos: _,
            const_type,
            value,
        }) => {
            type_resolution(&mut type_map, dest, const_type.into());
            type_resolution(&mut type_map, dest, value.into())
        }
        Code::Instruction(Instruction::Value {
            args,
            dest,
            funcs,
            labels,
            op : ValueOps::Add | ValueOps::Sub | ValueOps::Mul | ValueOps::Div,
            pos,
            op_type,
        }) => todo!(),
        Code::Instruction(Instruction::Effect {
            args,
            funcs,
            labels,
            op,
            pos,
        }) => todo!(),
    });

    type_map
}

pub fn infer_prog(Program { functions }: Program) -> Program {
    // Collect a list of function signatures
    let func_sigs: Vec<(&str, TypeSignature)> = functions
        .iter()
        .map(|f| {
            (
                f.name.as_ref(),
                TypeSignature {
                    inputs: f.args.iter().map(|a| a.arg_type.clone().into()).collect(),
                    output: f.return_type.clone().into(),
                },
            )
        })
        .collect();

    // Create a starting map of variables to types

    // Iterate through each function to find all of the missing types and add them to the map

    // Resolve any missing types in function signatures and resolve function maps.

    // Make inferred types concrete.

    // monomorphize functions?

    Program { functions }
}
