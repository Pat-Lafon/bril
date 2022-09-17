use std::collections::HashSet;

use bril_rs::{Code, ConstOps, EffectOps, Function, Instruction, Program, Type, ValueOps};

use crate::error::CheckError;
use cfg_if::cfg_if;
use fxhash::FxHashMap;

cfg_if! {
    if #[cfg(feature = "position")] {
        use bril_rs::positional::{PositionalError,PositionalErrorTrait};
        type Error = PositionalError<CheckError>;
    } else {
        type Error = CheckError;
    }
}

const fn check_num_args(expected: usize, args: &[String]) -> Result<(), CheckError> {
    if expected == args.len() {
        Ok(())
    } else {
        Err(CheckError::BadNumArgs(expected, args.len()))
    }
}

const fn check_num_funcs(expected: usize, funcs: &[String]) -> Result<(), CheckError> {
    if expected == funcs.len() {
        Ok(())
    } else {
        Err(CheckError::BadNumFuncs(expected, funcs.len()))
    }
}

const fn check_num_labels(expected: usize, labels: &[String]) -> Result<(), CheckError> {
    if expected == labels.len() {
        Ok(())
    } else {
        Err(CheckError::BadNumLabels(expected, labels.len()))
    }
}

fn check_asmt_type(expected: &bril_rs::Type, actual: &bril_rs::Type) -> Result<(), CheckError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CheckError::BadAsmtType(expected.clone(), actual.clone()))
    }
}

fn update_env<'a>(
    env: &mut FxHashMap<&'a str, &'a Type>,
    dest: &'a str,
    typ: &'a Type,
) -> Result<(), CheckError> {
    match env.get(dest) {
        Some(current_typ) => check_asmt_type(current_typ, typ),
        None => {
            env.insert(dest, typ);
            Ok(())
        }
    }
}

fn get_type<'a>(
    env: &'a FxHashMap<&'a str, &'a Type>,
    index: usize,
    args: &[String],
) -> Result<&'a &'a Type, CheckError> {
    if index >= args.len() {
        return Err(CheckError::BadNumArgs(index, args.len()));
    }

    env.get(&args[index] as &str)
        .ok_or_else(|| CheckError::VarUndefined(args[index].to_string()))
}

#[cfg(feature = "memory")]
fn get_ptr_type(typ: &bril_rs::Type) -> Result<&bril_rs::Type, CheckError> {
    match typ {
        bril_rs::Type::Pointer(ptr_type) => Ok(ptr_type),
        _ => Err(CheckError::ExpectedPointerType(typ.clone())),
    }
}

fn checklabels(labels: &[String], label_set: &HashSet<&String>) -> Result<(), CheckError> {
    labels.iter().try_for_each(|l| match label_set.get(l) {
        Some(_) => Ok(()),
        None => Err(CheckError::MissingLabel(l.to_string())),
    })
}

fn type_check_instruction<'a>(
    instr: &'a Instruction,
    func: &Function,
    prog: &Program,
    env: &mut FxHashMap<&'a str, &'a Type>,
    label_set: &HashSet<&String>,
) -> Result<(), CheckError> {
    match instr {
        Instruction::Constant {
            op: ConstOps::Const,
            dest,
            const_type,
            value,
            ..
        } => {
            cfg_if! {
                if #[cfg(feature = "float")] {
                    // For floats, Integer literals can be implicitly coerced into floating point
                    if !(const_type == &Type::Float && value.get_type() == Type::Int) {
                        check_asmt_type(const_type, &value.get_type())?;
                    }
                } else {
                    check_asmt_type(const_type, &value.get_type())?;
                }
            }

            update_env(env, dest, const_type)
        }
        Instruction::Value {
            op: ValueOps::Add | ValueOps::Sub | ValueOps::Mul | ValueOps::Div,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Int, get_type(env, 0, args)?)?;
            check_asmt_type(&Type::Int, get_type(env, 1, args)?)?;
            check_asmt_type(&Type::Int, op_type)?;
            update_env(env, dest, op_type)
        }
        Instruction::Value {
            op: ValueOps::Eq | ValueOps::Lt | ValueOps::Gt | ValueOps::Le | ValueOps::Ge,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Int, get_type(env, 0, args)?)?;
            check_asmt_type(&Type::Int, get_type(env, 1, args)?)?;
            check_asmt_type(&Type::Bool, op_type)?;
            update_env(env, dest, op_type)
        }
        Instruction::Value {
            op: ValueOps::Not,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(1, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Bool, get_type(env, 0, args)?)?;
            check_asmt_type(&Type::Bool, op_type)?;
            update_env(env, dest, op_type)
        }
        Instruction::Value {
            op: ValueOps::And | ValueOps::Or,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Bool, get_type(env, 0, args)?)?;
            check_asmt_type(&Type::Bool, get_type(env, 1, args)?)?;
            check_asmt_type(&Type::Bool, op_type)?;
            update_env(env, dest, op_type)
        }
        Instruction::Value {
            op: ValueOps::Id,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(1, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(op_type, get_type(env, 0, args)?)?;
            update_env(env, dest, op_type)
        }
        #[cfg(feature = "float")]
        Instruction::Value {
            op: ValueOps::Fadd | ValueOps::Fsub | ValueOps::Fmul | ValueOps::Fdiv,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Float, get_type(env, 0, args)?)?;
            check_asmt_type(&Type::Float, get_type(env, 1, args)?)?;
            check_asmt_type(&Type::Float, op_type)?;
            update_env(env, dest, op_type)
        }
        #[cfg(feature = "float")]
        Instruction::Value {
            op: ValueOps::Feq | ValueOps::Flt | ValueOps::Fgt | ValueOps::Fle | ValueOps::Fge,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Float, get_type(env, 0, args)?)?;
            check_asmt_type(&Type::Float, get_type(env, 1, args)?)?;
            check_asmt_type(&Type::Bool, op_type)?;
            update_env(env, dest, op_type)
        }
        Instruction::Value {
            op: ValueOps::Call,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_funcs(1, funcs)?;
            check_num_labels(0, labels)?;
            let callee_func = prog
                .functions
                .iter()
                .find(|f| f.name == funcs[0])
                .ok_or_else(|| CheckError::FuncNotFound(funcs[0].to_string()))?;

            if args.len() != callee_func.args.len() {
                return Err(CheckError::BadNumArgs(callee_func.args.len(), args.len()));
            }
            args.iter()
                .zip(callee_func.args.iter())
                .try_for_each(|(arg_name, expected_arg)| {
                    let ty = env
                        .get(arg_name as &str)
                        .ok_or_else(|| CheckError::VarUndefined(arg_name.to_string()))?;

                    check_asmt_type(ty, &expected_arg.arg_type)
                })?;

            match &callee_func.return_type {
                None => Err(CheckError::NonEmptyRetForFunc(callee_func.name.clone())),
                Some(t) => check_asmt_type(op_type, t),
            }?;
            update_env(env, dest, op_type)
        }
        #[cfg(feature = "ssa")]
        Instruction::Value {
            op: ValueOps::Phi,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            if args.len() != labels.len() {
                return Err(CheckError::UnequalPhiNode);
            }
            checklabels(labels, label_set)?;
            check_num_funcs(0, funcs)?;
            // Phi nodes are a little weird with their args and there has been some discussion on an _undefined var name in #108
            // Instead, we are going to assign the type we expect to all of the args and this will trigger an error if any of these args ends up being a different type.
            args.iter().try_for_each(|a| update_env(env, a, op_type))?;

            update_env(env, dest, op_type)
        }
        #[cfg(feature = "memory")]
        Instruction::Value {
            op: ValueOps::Alloc,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(1, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            check_asmt_type(&Type::Int, get_type(env, 0, args)?)?;
            get_ptr_type(op_type)?;
            update_env(env, dest, op_type)
        }
        #[cfg(feature = "memory")]
        Instruction::Value {
            op: ValueOps::Load,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(1, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            let ptr_type = get_ptr_type(get_type(env, 0, args)?)?;
            check_asmt_type(ptr_type, op_type)?;
            update_env(env, dest, op_type)
        }
        #[cfg(feature = "memory")]
        Instruction::Value {
            op: ValueOps::PtrAdd,
            dest,
            op_type,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            let ty0 = get_type(env, 0, args)?;
            get_ptr_type(ty0)?;
            check_asmt_type(&Type::Int, get_type(env, 1, args)?)?;
            check_asmt_type(ty0, op_type)?;
            update_env(env, dest, op_type)
        }
        Instruction::Effect {
            op: EffectOps::Jump,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(0, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(1, labels)?;
            checklabels(labels, label_set)?;
            Ok(())
        }
        Instruction::Effect {
            op: EffectOps::Branch,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(1, args)?;
            check_asmt_type(&Type::Bool, get_type(env, 0, args)?)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(2, labels)?;
            checklabels(labels, label_set)?;
            Ok(())
        }
        Instruction::Effect {
            op: EffectOps::Return,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            match &func.return_type {
                Some(t) => {
                    check_num_args(1, args)?;
                    let ty0 = get_type(env, 0, args)?;
                    check_asmt_type(t, ty0)
                }
                None => {
                    if args.is_empty() {
                        Ok(())
                    } else {
                        Err(CheckError::NonEmptyRetForFunc(func.name.clone()))
                    }
                }
            }
        }
        Instruction::Effect {
            op: EffectOps::Print,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            args.iter().enumerate().try_for_each(|(i, _)| {
                get_type(env, i, args)?;
                Ok(())
            })
        }
        Instruction::Effect {
            op: EffectOps::Nop,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(0, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            Ok(())
        }
        Instruction::Effect {
            op: EffectOps::Call,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_funcs(1, funcs)?;
            check_num_labels(0, labels)?;
            let callee_func = prog
                .functions
                .iter()
                .find(|f| f.name == funcs[0])
                .ok_or_else(|| CheckError::FuncNotFound(funcs[0].to_string()))?;

            if args.len() != callee_func.args.len() {
                return Err(CheckError::BadNumArgs(callee_func.args.len(), args.len()));
            }
            args.iter()
                .zip(callee_func.args.iter())
                .try_for_each(|(arg_name, expected_arg)| {
                    let ty = env
                        .get(arg_name as &str)
                        .ok_or_else(|| CheckError::VarUndefined(arg_name.to_string()))?;

                    check_asmt_type(ty, &expected_arg.arg_type)
                })?;

            if callee_func.return_type.is_some() {
                Err(CheckError::NonEmptyRetForFunc(callee_func.name.clone()))
            } else {
                Ok(())
            }
        }
        #[cfg(feature = "memory")]
        Instruction::Effect {
            op: EffectOps::Store,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(2, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            let ty0 = get_type(env, 0, args)?;
            let ty1 = get_type(env, 1, args)?;
            check_asmt_type(get_ptr_type(ty0)?, ty1)
        }
        #[cfg(feature = "memory")]
        Instruction::Effect {
            op: EffectOps::Free,
            args,
            funcs,
            labels,
            ..
        } => {
            check_num_args(1, args)?;
            check_num_funcs(0, funcs)?;
            check_num_labels(0, labels)?;
            get_ptr_type(get_type(env, 0, args)?)?;
            Ok(())
        }
        #[cfg(feature = "speculate")]
        Instruction::Effect {
            op: EffectOps::Speculate | EffectOps::Guard | EffectOps::Commit,
            args: _,
            funcs: _,
            labels: _,
            ..
        } => {
            unimplemented!()
        }
    }
}

fn type_check_func(func: &Function, prog: &Program) -> Result<(), Error> {
    let mut env: FxHashMap<&str, &Type> =
        FxHashMap::with_capacity_and_hasher(20, fxhash::FxBuildHasher::default());
    func.args.iter().for_each(|a| {
        env.insert(&a.name, &a.arg_type);
    });

    let mut label_set = HashSet::new();
    func.instrs.iter().try_for_each(|c| match c {
        Code::Label { label, .. } => match label_set.replace(label) {
            Some(l) => {
                Err(CheckError::DuplicateLabel(l.to_string(), func.name.to_string()).no_pos())
            }
            None => Ok(()),
        },
        _ => Ok(()),
    })?;

    func.instrs.iter().try_for_each(|c| match c {
        Code::Label { .. } => Ok(()),
        Code::Instruction(i) => {
            cfg_if! {
              if #[cfg(feature = "position")] {
                let pos = i.get_pos();
              } else {
                let pos = None;
              }
            }

            type_check_instruction(i, func, prog, &mut env, &label_set).map_err(|e| e.add_pos(pos))
        }
    })
}

/// Provides validation of Bril programs. This involves
/// statically checking the types and number of arguments to Bril
/// instructions.
/// # Errors
/// Will return an error if typechecking fails or if the input program is not well-formed.
pub fn type_check(prog: &Program) -> Result<(), Error> {
    prog.functions
        .iter()
        .try_for_each(|func| type_check_func(func, prog))
}
