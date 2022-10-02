use crate::{
    EffectOps,
    Type::{self, *},
    ValueOps::{self, *},
};

pub struct TypeSignature {
    pub inputs: Vec<Type>,
    pub outputs: Vec<Type>,
}

impl From<ValueOps> for TypeSignature {
    fn from(v: ValueOps) -> Self {
        match v {
            Add | Sub | Mul | Div => Self {
                inputs: vec![Int, Int],
                outputs: vec![Int],
            },
            Eq | Lt | Gt | Le | Ge => Self {
                inputs: vec![Int, Int],
                outputs: vec![Bool],
            },
            Not => Self {
                inputs: vec![Bool],
                outputs: vec![Bool],
            },
            And | Or => Self {
                inputs: vec![Bool, Bool],
                outputs: vec![Bool],
            },
            Call => todo!(),
            Id => todo!(),
            Fadd | Fsub | Fmul | Fdiv => Self {
                inputs: vec![Float, Float],
                outputs: vec![Float],
            },
            Feq | Flt | Fgt | Fle | Fge => Self {
                inputs: vec![Float, Float],
                outputs: vec![Bool],
            },
            Alloc => todo!(),
            Load => todo!(),
            PtrAdd => todo!(),
            Phi => todo!(),
        }
    }
}
