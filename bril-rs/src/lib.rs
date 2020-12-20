use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<Argument>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<Type>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instrs: Vec<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Code {
    Label { label: String },
    Instruction(Instruction),
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Code::Label { label } => write!(f, "Label {}", label),
            Code::Instruction(i) => write!(f, "{:?}", i),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Instruction {
    Constant {
        op: ConstOps,
        dest: String,
        #[serde(rename = "type")]
        const_type: Type,
        value: Literal,
    },
    Value {
        op: ValueOps,
        dest: String,
        #[serde(rename = "type")]
        op_type: Type,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
    },
    Effect {
        op: EffectOps,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
    },
}

impl Instruction {
    // for inserting blocks
    pub fn get_labels(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Constant { .. } => None,
            Instruction::Value { labels, .. } => Some(labels.clone()),
            Instruction::Effect { labels, .. } => Some(labels.clone()),
        }
    }
    pub fn set_labels(&mut self, new_labels: Vec<String>) {
        match self.clone() {
            Instruction::Constant { .. } => panic!("There is no labels to set"),
            Instruction::Value {
                op,
                dest,
                op_type,
                args,
                funcs,
                labels: _,
            } => {
                *self = Instruction::Value {
                    op,
                    dest,
                    op_type,
                    args,
                    funcs,
                    labels: new_labels,
                }
            }
            Instruction::Effect {
                op,
                args,
                funcs,
                labels: _,
            } => {
                *self = Instruction::Effect {
                    op,
                    args,
                    funcs,
                    labels: new_labels,
                }
            }
        }
    }
    // for ssa
    pub fn get_args(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Constant { .. } => None,
            Instruction::Value { args, .. } => Some(args.clone()),
            Instruction::Effect { args, .. } => Some(args.clone()),
        }
    }
    pub fn set_args(&mut self, new_args: Vec<String>) {
        match self.clone() {
            Instruction::Constant { .. } => panic!("There is no args to set"),
            Instruction::Value {
                op,
                dest,
                op_type,
                args: _,
                funcs,
                labels,
            } => {
                *self = Instruction::Value {
                    op,
                    dest,
                    op_type,
                    args: new_args,
                    funcs,
                    labels,
                }
            }
            Instruction::Effect {
                op,
                args: _,
                funcs,
                labels,
            } => {
                *self = Instruction::Effect {
                    op,
                    args: new_args,
                    funcs,
                    labels,
                }
            }
        }
    }

    pub fn get_dest(&self) -> Option<String> {
        match self {
            Instruction::Constant { dest, .. } => Some(dest.clone()),
            Instruction::Value { dest, .. } => Some(dest.clone()),
            Instruction::Effect { .. } => None,
        }
    }
    pub fn set_dest(&mut self, new_dest: String) {
        match self.clone() {
            Instruction::Constant {
                op,
                dest: _,
                const_type,
                value,
            } => {
                *self = Instruction::Constant {
                    op,
                    dest: new_dest,
                    const_type,
                    value,
                }
            }
            Instruction::Value {
                op,
                dest: _,
                op_type,
                args,
                funcs,
                labels,
            } => {
                *self = Instruction::Value {
                    op,
                    dest: new_dest,
                    op_type,
                    args,
                    funcs,
                    labels,
                }
            }
            Instruction::Effect { .. } => panic!("There is no dest to be set"),
        }
    }

    pub fn get_type(&self) -> Option<Type> {
        match self {
            Instruction::Constant { const_type, .. } => Some(const_type.clone()),
            Instruction::Value { op_type, .. } => Some(op_type.clone()),
            Instruction::Effect { .. } => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ConstOps {
    #[serde(rename = "const")]
    Const,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EffectOps {
    #[serde(rename = "jmp")]
    Jump,
    #[serde(rename = "br")]
    Branch,
    Call,
    #[serde(rename = "ret")]
    Return,
    Print,
    Nop,
    #[cfg(feature = "memory")]
    Store,
    #[cfg(feature = "memory")]
    Free,
    #[cfg(feature = "speculate")]
    Speculate,
    #[cfg(feature = "speculate")]
    Commit,
    #[cfg(feature = "speculate")]
    Guard,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ValueOps {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
    Not,
    And,
    Or,
    Call,
    Id,
    #[cfg(feature = "ssa")]
    Phi,
    #[cfg(feature = "float")]
    Fadd,
    #[cfg(feature = "float")]
    Fsub,
    #[cfg(feature = "float")]
    Fmul,
    #[cfg(feature = "float")]
    Fdiv,
    #[cfg(feature = "float")]
    Feq,
    #[cfg(feature = "float")]
    Flt,
    #[cfg(feature = "float")]
    Fgt,
    #[cfg(feature = "float")]
    Fle,
    #[cfg(feature = "float")]
    Fge,
    #[cfg(feature = "memory")]
    Alloc,
    #[cfg(feature = "memory")]
    Load,
    #[cfg(feature = "memory")]
    PtrAdd,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Int,
    Bool,
    #[cfg(feature = "float")]
    Float,
    #[cfg(feature = "memory")]
    #[serde(rename = "ptr")]
    Pointer(Box<Type>),
    PointerRegions {
        #[serde(rename = "type")]
        pointer_type : Box<Type>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ownership : Option<Ownership>,
        region: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owner,
    Borrower,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    #[cfg(feature = "float")]
    Float(f64),
}

pub fn load_program() -> Program {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    serde_json::from_str(&buffer).unwrap()
}

pub fn output_program(p: &Program) {
    io::stdout()
        .write_all(serde_json::to_string(p).unwrap().as_bytes())
        .unwrap();
}
