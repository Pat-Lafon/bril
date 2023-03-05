use bril_rs::{
    AbstractArgument, AbstractCode, AbstractFunction, AbstractInstruction, AbstractProgram,
    AbstractType, ConstOps, Import, ImportedFunction, Literal,
};

use crate::{
    bril_grammar::{
        Alias, Args, Func, Ident, Label, ParserArgumentList, ParserCode, ParserConstOps,
        ParserFunction, ParserImport, ParserImportedFunction, ParserInstruction, ParserLiteral,
        ParserOutputType, ParserProgram, ParserType,
    },
    Lines,
};

impl ParserProgram {
    pub(crate) fn into(self, pos: &Lines) -> AbstractProgram {
        AbstractProgram {
            functions: self.functions.into_iter().map(|f| f.into(pos)).collect(),
            imports: self.imports.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl ParserImport {
    fn into(self) -> Import {
        Import {
            functions: self.functions.into_iter().map(|f| f.into()).collect(),
            path: self.path,
        }
    }
}

impl ParserImportedFunction {
    fn into(self) -> ImportedFunction {
        ImportedFunction {
            alias: self.alias.map(|a| a.into()),
            name: self.name.into(),
        }
    }
}

impl ParserFunction {
    fn into(self, pos: &Lines) -> AbstractFunction {
        AbstractFunction {
            args: self.args.map_or_else(|| Vec::new(), |a| a.into()),
            instrs: self.code.into_iter().map(|c| c.into(pos)).collect(),
            name: self.name.value.into(),
            pos: pos.get_position(self.name.span.0, self.ty.span.1),
            return_type: self.ty.value.map(Into::into),
        }
    }
}

impl ParserArgumentList {
    fn into(self) -> Vec<AbstractArgument> {
        self.args
            .into_iter()
            .map(|a| AbstractArgument {
                name: a.name.into(),
                arg_type: a.arg_type.into(),
            })
            .collect()
    }
}

impl ParserCode {
    fn into(self, pos: &Lines) -> AbstractCode {
        match self {
            ParserCode::Label(l, c) => AbstractCode::Label {
                label: l.value.into(),
                pos: pos.get_position(l.span.0, c.span.1),
            },
            ParserCode::Instruction(i) => AbstractCode::Instruction(i.into(pos)),
        }
    }
}

impl ParserInstruction {
    fn into(self, pos: &Lines) -> AbstractInstruction {
        match self {
            ParserInstruction::Constant(d, ty, _, c, l, e) => AbstractInstruction::Constant {
                dest: d.value.into(),
                op: c.into(),
                pos: pos.get_position(d.span.0, e.span.1),
                const_type: ty.map(Into::into),
                value: l.into(),
            },
            ParserInstruction::Value(d, ty, _, op, a, e) => {
                let mut args = Vec::new();
                let mut funcs = Vec::new();
                let mut labels = Vec::new();
                for x in a {
                    match x {
                        Args::Func(f) => funcs.push(f.into()),
                        Args::Label(l) => labels.push(l.into()),
                        Args::Ident(a) => args.push(a.into()),
                    }
                }
                AbstractInstruction::Value {
                    args,
                    dest: d.value.into(),
                    funcs,
                    labels,
                    op: op.into(),
                    pos: pos.get_position(d.span.0, e.span.1),
                    op_type: ty.map(Into::into),
                }
            }
            ParserInstruction::Effect(o, a, e) => {
                let mut args = Vec::new();
                let mut funcs = Vec::new();
                let mut labels = Vec::new();
                for x in a {
                    match x {
                        Args::Func(f) => funcs.push(f.into()),
                        Args::Label(l) => labels.push(l.into()),
                        Args::Ident(a) => args.push(a.into()),
                    }
                }
                AbstractInstruction::Effect {
                    args,
                    funcs,
                    labels,
                    op: o.value.into(),
                    pos: pos.get_position(o.span.0, e.span.1),
                }
            }
        }
    }
}

impl From<ParserConstOps> for ConstOps {
    fn from(value: ParserConstOps) -> Self {
        match value {
            ParserConstOps::Const(_) => Self::Const,
        }
    }
}

impl From<Alias> for String {
    fn from(Alias { alias, .. }: Alias) -> Self {
        alias.into()
    }
}

impl From<Func> for String {
    fn from(Func { name }: Func) -> Self {
        name
    }
}

impl From<Ident> for String {
    fn from(Ident { name }: Ident) -> Self {
        name
    }
}

impl From<Label> for String {
    fn from(Label { name }: Label) -> Self {
        name
    }
}

impl From<ParserOutputType> for AbstractType {
    fn from(ParserOutputType { arg_type, .. }: ParserOutputType) -> Self {
        arg_type.value.into()
    }
}

impl From<ParserType> for AbstractType {
    fn from(value: ParserType) -> Self {
        match value {
            ParserType::Primitive(i) => Self::Primitive(i.into()),
            ParserType::Parameterized(i, _, ty, _) => {
                Self::Parameterized(i.into(), Box::new(AbstractType::from(*ty)))
            }
        }
    }
}

impl From<ParserLiteral> for Literal {
    fn from(value: ParserLiteral) -> Self {
        match value {
            ParserLiteral::Int(i) => Self::Int(i),
            ParserLiteral::Bool(b) => Self::Bool(b),
            ParserLiteral::Float(f) => Self::Float(f),
        }
    }
}
