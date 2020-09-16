use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{self, Read};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Argument>>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<Type>,
    pub instrs: Vec<Code>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Argument {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: Type,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Code {
    Label { label: String },
    Instruction(Instruction),
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Code::Label { label } => write!(f, "Label {}", label),
            Code::Instruction(i) => write!(f, "{}", i),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        funcs: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        labels: Option<Vec<String>>,
    },
    Effect {
        op: EffectOps,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        funcs: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        labels: Option<Vec<String>>,
    },
}

impl Instruction {
    pub fn get_dest(&self) -> Option<String> {
        match self {
            Instruction::Constant { dest, .. } => Some(dest.clone()),
            Instruction::Value { dest, .. } => Some(dest.clone()),
            Instruction::Effect { .. } => None,
        }
    }
}

// I am going to assume that each of these has been checked on creation so I'm leaving out a bunch of asserts. These can be added if stuff looks weird
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Constant {
                op: ConstOps::Const,
                dest,
                const_type,
                value,
            } => write!(f, "{} : {:?} = const {:?}", dest, const_type, value),
            Instruction::Value {
                ..
                /* op,
                dest,
                op_type,
                args,
                funcs,
                labels, */
            } => unimplemented!(),
            Instruction::Effect {
                op: EffectOps::Branch,
                args,
                labels,
                ..
            } => {
                let l = labels.as_ref().unwrap();
                write!(f, "br {} {} {}", args.as_ref().unwrap()[0], l[0], l[1])
            }
            Instruction::Effect {
                op: EffectOps::Call,
                args,
                funcs,
                ..
            } => match args {
                Some(a) => write!(f, "call {} {}", funcs.as_ref().unwrap()[0], a.join(" ")),
                None => write!(f, "call"),
            }
            Instruction::Effect {
                op: op @ (EffectOps::Jump | EffectOps::Nop | EffectOps::Return | EffectOps::Print | EffectOps::Store | EffectOps::Free),
                args,
                ..
            } => match args {
                Some(a) => write!(f, "{} {}", op, a.join(" ")),
                None => write!(f, "{}", op),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConstOps {
    #[serde(rename = "const")]
    Const,
}

// Todo Can I handle ops in a better way because call overlaps?
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EffectOps {
    #[serde(rename = "jmp")]
    Jump,
    #[serde(rename = "br")]
    Branch,
    #[serde(rename = "call")]
    Call,
    #[serde(rename = "ret")]
    Return,
    #[serde(rename = "print")]
    Print,
    #[serde(rename = "nop")]
    Nop,
    #[serde(rename = "store")]
    Store,
    #[serde(rename = "free")]
    Free,
}

impl fmt::Display for EffectOps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EffectOps::Jump => write!(f, "jump"),
            EffectOps::Branch => write!(f, "br"),
            EffectOps::Call => write!(f, "call"),
            EffectOps::Return => write!(f, "ret"),
            EffectOps::Print => write!(f, "print"),
            EffectOps::Nop => write!(f, "nop"),
            EffectOps::Store => write!(f, "store"),
            EffectOps::Free => write!(f, "free"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ValueOps {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "sub")]
    Sub,
    #[serde(rename = "mul")]
    Mul,
    #[serde(rename = "div")]
    Div,
    #[serde(rename = "eq")]
    Eq,
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "gt")]
    Gt,
    #[serde(rename = "le")]
    Le,
    #[serde(rename = "ge")]
    Ge,
    #[serde(rename = "not")]
    Not,
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
    #[serde(rename = "call")]
    Call,
    #[serde(rename = "alloc")]
    Alloc,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "bool")]
    Bool,
    /*     PointerType, */
    //TODO There is also some parameterized pointer type
}
/*
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PointerType {
    Pointer { ptr: PrimitiveType },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PrimitiveType {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "bool")]
    Bool,
} */

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Literal {
    Int(i32),
    Bool(bool),
}

pub fn load_program() -> Program {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    serde_json::from_str(&buffer).unwrap()
}
