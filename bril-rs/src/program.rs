use serde::{Deserialize, Serialize};
use std::io::{self, Read};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Option<Vec<Argument>>,
    #[serde(rename = "type")]
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

impl From<Code> for String {
    fn from(item: Code) -> Self {
        match item {
            Code::Label { label } => format!("Label {}", label),
            Code::Instruction(i) => String::from(i),
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
        args: Option<Vec<String>>,
        funcs: Option<Vec<String>>,
        labels: Option<Vec<String>>,
    },
    Effect {
        op: EffectOps,
        args: Option<Vec<String>>,
        funcs: Option<Vec<String>>,
        labels: Option<Vec<String>>,
    },
}

// I am going to assume that each of these has been checked on creation so I'm leaving out a bunch of asserts. These can be added if stuff looks weird
impl From<Instruction> for String {
    fn from(item: Instruction) -> Self {
        match item {
            Instruction::Constant {
                op: ConstOps::Const,
                dest,
                const_type,
                value,
            } => format!("{} : {:?} = const {:?}", dest, const_type, value),
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
                op: EffectOps::Jump,

                labels,
                ..
            } => format!("jmp {}", labels.unwrap()[0]),
            Instruction::Effect {
                op: EffectOps::Branch,
                args,
                labels,
                ..
            } => {
                let l = labels.unwrap();
                format!("br {} {} {}", args.unwrap()[0], l[0], l[1])
            }
            Instruction::Effect {
                op: EffectOps::Call,
                args,
                funcs,
                ..
            } => match args {
                Some(a) => format!("call {} {}", funcs.unwrap()[0], a.join(" ")),
                None => "print".to_string(),
            }
            Instruction::Effect {
                op: EffectOps::Return,
                args,
                ..
            } => match args {
                Some(a) => format!("ret {}", a[0]),
                None => "ret".to_string(),
            },
            Instruction::Effect {
                op: EffectOps::Print,
                args,
                ..
            } => match args {
                Some(a) => format!("print {}", a.join(" ")),
                None => "print".to_string(),
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ValueOps {
    //TODo
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Type {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "bool")]
    Bool,
    //TODO There is also some parameterized pointer type
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Literal {
    Int(u32),
    Bool(bool),
}

pub fn load_program() -> Program {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    serde_json::from_str(&buffer).unwrap()
}
