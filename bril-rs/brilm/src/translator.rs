use bril_rs::{
    Code, EffectOps, Function, Instruction, Literal, Program, Type as BrilType, ValueOps,
};
use melior::{
    Context,
    ir::{
        Identifier,
        attribute::{BoolAttribute, IntegerAttribute, StringAttribute, TypeAttribute},
        operation::{OperationBuilder, OperationLike},
        r#type::{FunctionType, IntegerType},
        *,
    },
};
use std::collections::HashMap;

/// Translates a bril-rs Program into melior IR
pub fn translate_program<'c>(context: &'c Context, program: &Program) -> Module<'c> {
    let location = Location::unknown(context);
    let module = Module::new(location);

    program.functions.iter().for_each(|func| {
        module
            .body()
            .append_operation(translate_function(context, func));
    });

    module
}

fn translate_function<'c>(context: &'c Context, func: &Function) -> Operation<'c> {
    // todo Add positions
    let location = Location::unknown(context);

    let arg_types: Vec<_> = func
        .args
        .iter()
        .map(|arg| translate_bril_type(context, &arg.arg_type))
        .collect();

    let result_types: Vec<_> = func
        .return_type
        .as_ref()
        .map(|t| vec![translate_bril_type(context, t)])
        .unwrap_or_default();

    let func_type = FunctionType::new(context, &arg_types, &result_types);

    let block = Block::new(
        &func
            .args
            .iter()
            .map(|arg| (translate_bril_type(context, &arg.arg_type), location))
            .collect::<Vec<_>>(),
    );

    let mut variable_map = HashMap::new();
    for (i, arg) in func.args.iter().enumerate() {
        if let Ok(block_arg) = block.argument(i) {
            variable_map.insert(arg.name.clone(), block_arg.into());
        }
    }

    for code in &func.instrs {
        match code {
            Code::Label { .. } => {
                unimplemented!()
            }
            Code::Instruction(instr) => {
                translate_instruction(context, instr, &block, &mut variable_map);
            }
        }
    }

    let region = Region::new();
    region.append_block(block);

    OperationBuilder::new("bril.func", location)
        .add_attributes(&[(
            Identifier::new(context, "sym_name"),
            StringAttribute::new(context, &func.name).into(),
        )])
        .add_attributes(&[(
            Identifier::new(context, "type"),
            TypeAttribute::new(func_type.into()).into(),
        )])
        .add_regions([region])
        .build()
        .unwrap()
}

/// Translate a single instruction to a melior operation
fn translate_instruction<'c>(
    context: &'c Context,
    instr: &Instruction,
    block: &Block<'c>,
    variable_map: &mut HashMap<String, Value<'c, 'c>>,
) {
    let location = Location::unknown(context);

    match instr {
        Instruction::Constant {
            dest,
            const_type,
            value,
            ..
        } => {
            let (ty, attr): (Type, Attribute) = match (value, const_type) {
                (Literal::Int(i), BrilType::Int) => {
                    let ty = IntegerType::new(context, 64).into();
                    (ty, IntegerAttribute::new(ty, *i).into())
                }
                (Literal::Bool(b), BrilType::Bool) => {
                    let ty = IntegerType::new(context, 1).into();
                    (ty, BoolAttribute::new(context, *b).into())
                }
                _ => panic!("I'll add a better error message later"),
            };
            let const_op = OperationBuilder::new("bril.const", location)
                .add_attributes(&[(Identifier::new(context, "value"), attr.into())])
                .add_results(&[ty])
                .build()
                .unwrap();
            let result = const_op.result(0).unwrap();
            variable_map.insert(dest.clone(), result.into());
            block.append_operation(const_op);
        }

        Instruction::Value {
            op,
            dest,
            args,
            op_type,
            ..
        } => {
            let op_args: Vec<_> = args
                .iter()
                .map(|arg| variable_map.get(arg).unwrap().clone())
                .collect();

            let result_type = translate_bril_type(context, op_type);

            let op_name = match op {
                ValueOps::Add => "bril.add",
                ValueOps::Sub => "bril.sub",
                ValueOps::Mul => "bril.mul",
                ValueOps::Div => "bril.div",
                ValueOps::Eq => "bril.eq",
                ValueOps::Lt => "bril.lt",
                ValueOps::Gt => "bril.gt",
                ValueOps::Le => "bril.le",
                ValueOps::Ge => "bril.ge",
                ValueOps::Not => "bril.not",
                ValueOps::And => "bril.and",
                ValueOps::Or => "bril.or",
                ValueOps::Id => "bril.id",
                _ => unimplemented!(),
            };

            let operation = OperationBuilder::new(op_name, location)
                .add_operands(&op_args)
                .add_results(&[result_type])
                .build()
                .unwrap();

            if let Ok(result) = operation.result(0) {
                let result_value = result.into();
                variable_map.insert(dest.clone(), result_value);
                block.append_operation(operation);
            }
        }

        Instruction::Effect { op, args, .. } => match op {
            EffectOps::Print => {
                for arg in args {
                    let value = variable_map.get(arg).unwrap().clone();
                    emit_print(context, value, block, location);
                }
            }
            EffectOps::Return => {
                let ret_op = if let Some(arg) = args.first() {
                    let value = variable_map.get(arg).unwrap().clone();
                    OperationBuilder::new("bril.ret", location)
                        .add_operand(value)
                        .build()
                        .unwrap()
                } else {
                    OperationBuilder::new("bril.ret", location)
                        .build()
                        .unwrap()
                };
                block.append_operation(ret_op);
            }
            EffectOps::Jump | EffectOps::Branch => {
                unimplemented!()
            }
            EffectOps::Nop => {
                unimplemented!()
            }
            _ => {
                unimplemented!()
            }
        },
    }
}

fn translate_bril_type<'c>(context: &'c Context, bril_ty: &BrilType) -> Type<'c> {
    match bril_ty {
        BrilType::Int => IntegerType::new(context, 64).into(),
        BrilType::Bool => IntegerType::new(context, 1).into(),
        BrilType::Pointer(_) => unimplemented!("Is there a briltype somewhere?"),
    }
}

/// Emit a print operation that outputs a single value to stdout
/// Uses a simple approach: emit LLVM calls to printf via melior's custom operation API
fn emit_print<'c>(
    _context: &'c Context,
    value: Value<'c, 'c>,
    _block: &Block<'c>,
    _location: Location<'c>,
) {
    let _format_str = "%ld\n";
    unimplemented!()
    // Note: A complete implementation would need:
    // 1. Global string constant creation in the module
    // 2. LLVM function reference for printf
    // 3. LLVM call operations with proper type conversions
    //
    // For now, this is a placeholder that captures the intent.
    // The actual MLIR lowering pass (from bril dialect to LLVM)
    // should handle printf codegen similar to brilir's BrilPasses.cpp

    // This can be expanded once melior provides better support for:
    // - Global constants
    // - External function declarations
    // - LLVM type system integration
}
