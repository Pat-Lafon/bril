use bril_rs::{Argument, Code, Function, Instruction, Program, Type};

fn remove_regions_type(typ: &Type) -> Type {
    match typ {
        Type::Int | Type::Float | Type::Bool | Type::Pointer(..) => typ.clone(),
        Type::PointerRegions { pointer_type, .. } => Type::Pointer(pointer_type.clone()),
    }
}

fn remove_regions_args(arg: &Argument) -> Argument {
    Argument {
        name: arg.name.clone(),
        arg_type: remove_regions_type(&arg.arg_type),
    }
}

fn remove_regions_code(code: &Code) -> Code {
    match code {
        Code::Label{..} |
        // We can assume that the type of const will not have regions
        Code::Instruction(Instruction::Constant { .. })
        | Code::Instruction(Instruction::Effect { .. }) => code.clone(),
        Code::Instruction(Instruction::Value {
            op,
            dest,
            op_type,
            args,
            funcs,
            labels,
        }) => Code::Instruction(Instruction::Value {
            op : op.clone(),
            dest: dest.clone(),
            op_type: remove_regions_type(op_type),
            args: args.clone(),
            funcs: funcs.clone(),
            labels: labels.clone(),
        }),
    }
}

pub fn remove_regions(program: &Program) -> Program {
    Program {
        functions: program
            .functions
            .iter()
            .map(|f| Function {
                name: f.name.clone(),
                args: f.args.iter().map(remove_regions_args).collect(),
                // return types don't need regions
                return_type: f.return_type.clone(),
                instrs: f.instrs.iter().map(remove_regions_code).collect(),
                region: None,
            })
            .collect(),
    }
}
