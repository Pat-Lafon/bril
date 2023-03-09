use std::collections::HashMap;

use bril_rs::{
    AbstractArgument, AbstractCode, AbstractFunction, AbstractInstruction, AbstractProgram,
    AbstractType, Program, Type,
};
use rusttyc::{Arity, Partial, TcVar, TypeChecker, Variant};

#[derive(PartialEq, Eq, Clone, Debug)]
enum BrilTypes {
    Top,
    Bool,
    Int,
    Float,
    Num,
    Ptr(Box<BrilTypes>),
    Bot,
}

impl From<Type> for BrilTypes {
    fn from(value: Type) -> Self {
        match value {
            Type::Int => Self::Int,
            Type::Bool => Self::Bool,
            Type::Float => Self::Float,
            Type::Pointer(x) => Self::Ptr(Box::new(From::from(*x))),
        }
    }
}

impl From<AbstractType> for BrilTypes {
    fn from(value: AbstractType) -> Self {
        TryInto::<Type>::try_into(value).unwrap().into()
    }
}

fn convert_up_type(ty: &Option<AbstractType>) -> BrilTypes {
    ty.clone().map_or_else(|| BrilTypes::Top, Into::into)
}

fn convert_down_type(ty: &Option<AbstractType>) -> BrilTypes {
    ty.clone().map_or_else(|| BrilTypes::Bot, Into::into)
}

impl Variant for BrilTypes {
    type Err = String;

    fn top() -> Self {
        Self::Top
    }

    fn meet(lhs: Partial<Self>, rhs: Partial<Self>) -> Result<Partial<Self>, Self::Err> {
        match (&lhs.variant, &rhs.variant) {
            // Top is the supertype of everything
            (BrilTypes::Top, _) => Ok(rhs),
            (_, BrilTypes::Top) => Ok(lhs),

            // Bot constrains everything
            (BrilTypes::Bot, _) | (_, BrilTypes::Bot) => Ok(Partial {
                variant: BrilTypes::Bot,
                least_arity: 0,
            }),

            // Bool can only be met with other bools
            (BrilTypes::Bool, BrilTypes::Bool) => Ok(lhs),
            (BrilTypes::Bool, _) | (_, BrilTypes::Bool) => Err("Error: Bool and other".to_string()),

            // Pointers can only be met with other pointers
            (BrilTypes::Ptr(_), BrilTypes::Ptr(_)) => todo!(),
            (_, BrilTypes::Ptr(_)) | (BrilTypes::Ptr(_), _) => {
                Err("Error: Ptr and other".to_string())
            }

            (BrilTypes::Num, BrilTypes::Num) => Ok(lhs),
            (BrilTypes::Num, _) => Ok(rhs),
            (_, BrilTypes::Num) => Ok(lhs),

            (BrilTypes::Int, BrilTypes::Int) => Ok(lhs),
            (BrilTypes::Int, BrilTypes::Float) | (BrilTypes::Float, BrilTypes::Int) => {
                Err("Error: Trying to mismatch int's and floats".to_string())
            }
            (BrilTypes::Float, BrilTypes::Float) => Ok(lhs),
        }
    }

    fn arity(&self) -> Arity {
        match self {
            BrilTypes::Top
            | BrilTypes::Bool
            | BrilTypes::Int
            | BrilTypes::Float
            | BrilTypes::Num
            | BrilTypes::Bot => Arity::Fixed(0),
            BrilTypes::Ptr(_) => Arity::Fixed(1),
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct VarName(String);
impl TcVar for VarName {}

fn create_fun_arg_name(func_name: &str, idx: usize) -> String {
    format!("{func_name}___{idx}")
}

fn create_fun_ret_name(func_name: &str) -> String {
    format!("{func_name}___ret")
}

fn my_get<'a>(map: &'a mut HashMap<&'a String, String>, name: &'a String) -> String {
    map.entry(name).or_insert(name.clone()).clone()
}

fn inference_function(
    tc: &mut TypeChecker<BrilTypes, VarName>,
    AbstractFunction {
        args,
        instrs,
        name,
        pos: _,
        return_type,
    }: &AbstractFunction,
) {
    let func_name = name;
    let mut map = HashMap::new();
    args.iter()
        .enumerate()
        .for_each(|(idx, AbstractArgument { name, arg_type })| {
            let constraint_name = create_fun_arg_name(func_name, idx);
            map.insert(name, constraint_name.clone());
            let key = tc.get_var_key(&VarName(constraint_name));
            tc.impose(key.concretizes_explicit(arg_type.clone().into()))
                .unwrap();
        });
    let return_constraint = create_fun_ret_name(func_name);
    let key = tc.get_var_key(&VarName(return_constraint));
    tc.impose(key.concretizes_explicit(convert_down_type(return_type)))
        .unwrap();

    instrs.iter().for_each(|i| match i {
        AbstractCode::Label { .. } => (),
        AbstractCode::Instruction(AbstractInstruction::Constant {
            dest,
            op,
            pos,
            const_type,
            value,
        }) => {}
        AbstractCode::Instruction(AbstractInstruction::Value {
            args,
            dest,
            funcs,
            labels,
            op,
            pos,
            op_type,
        }) => todo!(),
        AbstractCode::Instruction(AbstractInstruction::Effect {
            args,
            funcs,
            labels,
            op,
            pos,
        }) => todo!(),
    });
}

fn inference(
    AbstractProgram {
        functions,
        imports: _,
    }: &AbstractProgram,
) -> Program {
    let tc = &mut TypeChecker::new();
    functions.iter().map(|func| inference_function(tc, func));
    todo!()
}

fn main() {
    println!("Hello, world!");
}
