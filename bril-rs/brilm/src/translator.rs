use bril_rs::{
    Code, EffectOps, Function, Instruction, Literal, Program, Type as BrilType, ValueOps,
};
use melior::{
    Context,
    ir::{
        Identifier,
        attribute::{
            BoolAttribute, DenseI32ArrayAttribute, FlatSymbolRefAttribute, IntegerAttribute,
            StringAttribute, TypeAttribute,
        },
        operation::{OperationBuilder, OperationLike},
        r#type::{FunctionType, IntegerType},
        *,
    },
};
use std::collections::HashMap;

use crate::main_wrapper::{
    create_main_wrapper, declare_bool_string, declare_strcmp, declare_strtoll,
};

/// Check if an instruction is a block terminator (control flow transfer)
fn is_terminator(instr: &Instruction) -> bool {
    matches!(
        instr,
        Instruction::Effect {
            op: EffectOps::Return | EffectOps::Jump | EffectOps::Branch,
            ..
        }
    )
}

/// Map a Bril value operation to its MLIR op name
fn value_op_name(op: &ValueOps) -> &'static str {
    match op {
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
        _ => unimplemented!("ValueOp {:?} not implemented", op),
    }
}

/// Collect argument values from variable map
fn collect_args<'c>(
    args: &[String],
    variable_map: &HashMap<String, Value<'c, 'c>>,
) -> Vec<Value<'c, 'c>> {
    args.iter()
        .map(|arg| {
            *variable_map
                .get(arg)
                .unwrap_or_else(|| panic!("Undefined variable: {}", arg))
        })
        .collect()
}

/// Translates a bril-rs Program into melior IR
pub fn translate_program<'c>(context: &'c Context, program: &Program) -> Module<'c> {
    let module = Module::new(Location::unknown(context));

    // Find main function and verify it exists
    let main_func = program
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("Program must have a main function");

    // Add declarations for argument parsing functions
    let needs_strtoll = main_func.args.iter().any(|a| matches!(a.arg_type, BrilType::Int));
    let needs_strcmp = main_func.args.iter().any(|a| matches!(a.arg_type, BrilType::Bool));
    if needs_strtoll {
        module.body().append_operation(declare_strtoll(context));
    }
    if needs_strcmp {
        module.body().append_operation(declare_strcmp(context));
        module.body().append_operation(declare_bool_string(context, "str_true", "true\0"));
        module.body().append_operation(declare_bool_string(context, "str_false", "false\0"));
    }

    program.functions.iter().for_each(|func| {
        module
            .body()
            .append_operation(translate_function(context, func));
    });

    // Add main wrapper for proper exit code (and arg parsing if needed)
    module
        .body()
        .append_operation(create_main_wrapper(context, &main_func.args));

    module
}

/// A basic block with an optional label and a sequence of instructions
struct BasicBlock<'a> {
    label: Option<String>,
    instructions: Vec<&'a Instruction>,
}

impl<'a> BasicBlock<'a> {
    fn has_terminator(&self) -> bool {
        self.instructions.last().is_some_and(|i| is_terminator(i))
    }
}

/// Split function instructions into basic blocks
fn split_into_blocks(instrs: &[Code]) -> Vec<BasicBlock<'_>> {
    let mut blocks: Vec<BasicBlock<'_>> = Vec::new();
    let mut current_label: Option<String> = None;
    let mut current_instrs: Vec<&Instruction> = Vec::new();
    let mut after_terminator = false; // Track if we're in unreachable code

    for code in instrs {
        match code {
            Code::Label { label, .. } => {
                // Save current block if it has instructions
                if !current_instrs.is_empty() || current_label.is_some() {
                    blocks.push(BasicBlock {
                        label: current_label.take(),
                        instructions: std::mem::take(&mut current_instrs),
                    });
                }
                current_label = Some(label.clone());
                after_terminator = false; // Label makes code reachable again
            }
            Code::Instruction(instr) => {
                // Skip unreachable instructions (after terminator, before next label)
                if after_terminator {
                    continue;
                }
                current_instrs.push(instr);
                // Check if this is a terminator - end block after terminator
                if is_terminator(instr) {
                    blocks.push(BasicBlock {
                        label: current_label.take(),
                        instructions: std::mem::take(&mut current_instrs),
                    });
                    after_terminator = true;
                }
            }
        }
    }

    // Don't forget the last block - but only if it has instructions
    // (a trailing label-only block is dead code and should be skipped)
    if !current_instrs.is_empty() {
        blocks.push(BasicBlock {
            label: current_label,
            instructions: current_instrs,
        });
    }

    blocks
}

/// Get the nth block from a region by traversing the linked list
fn get_block_by_index<'c, 'a>(region: &'a Region<'c>, index: usize) -> Option<BlockRef<'c, 'a>> {
    let mut current = region.first_block()?;
    for _ in 0..index {
        current = current.next_in_region()?;
    }
    Some(current)
}

/// Resolves labels to blocks within a region
struct BlockResolver<'a, 'c> {
    label_to_block_idx: &'a HashMap<String, usize>,
    region: &'a Region<'c>,
}

impl<'a, 'c> BlockResolver<'a, 'c> {
    fn new(label_to_block_idx: &'a HashMap<String, usize>, region: &'a Region<'c>) -> Self {
        Self { label_to_block_idx, region }
    }

    fn get(&self, label: &str) -> BlockRef<'c, 'a> {
        let &idx = self.label_to_block_idx
            .get(label)
            .unwrap_or_else(|| panic!("Undefined label: {}", label));
        get_block_by_index(self.region, idx).unwrap()
    }
}

/// Create a void return operation
fn make_void_return<'c>(context: &'c Context) -> Operation<'c> {
    OperationBuilder::new("bril.ret", Location::unknown(context))
        .build()
        .unwrap()
}

/// Helper to create an arith.constant and append it to a block
pub(crate) fn make_const<'c>(
    context: &'c Context,
    block: &Block<'c>,
    ty: Type<'c>,
    value: i64,
    location: Location<'c>,
) -> Value<'c, 'c> {
    let op = OperationBuilder::new("arith.constant", location)
        .add_attributes(&[(
            Identifier::new(context, "value"),
            IntegerAttribute::new(ty, value).into(),
        )])
        .add_results(&[ty])
        .build()
        .unwrap();
    let val = op.result(0).unwrap().into();
    block.append_operation(op);
    val
}

/// Helper to emit an arith.cmpi equality comparison
pub(crate) fn emit_cmpi_eq<'c>(
    context: &'c Context,
    block: &Block<'c>,
    lhs: Value<'c, 'c>,
    rhs: Value<'c, 'c>,
    location: Location<'c>,
) -> Value<'c, 'c> {
    let i1_type: Type = IntegerType::new(context, 1).into();
    let op = OperationBuilder::new("arith.cmpi", location)
        .add_attributes(&[(
            Identifier::new(context, "predicate"),
            IntegerAttribute::new(IntegerType::new(context, 64).into(), 0).into(), // 0 = eq
        )])
        .add_operands(&[lhs, rhs])
        .add_results(&[i1_type])
        .build()
        .unwrap();
    let val = op.result(0).unwrap().into();
    block.append_operation(op);
    val
}

/// Emit a bril.jmp to a target block
fn emit_jmp<'c>(block: &BlockRef<'c, '_>, target: &BlockRef<'c, '_>, location: Location<'c>) {
    let jmp_op = OperationBuilder::new("bril.jmp", location)
        .add_successors(&[target])
        .build()
        .unwrap();
    block.append_operation(jmp_op);
}

/// Emit a bril value operation: build op, store result in variable_map, append to block
fn emit_bril_value_op<'c>(
    block: &BlockRef<'c, '_>,
    op_name: &str,
    args: &[Value<'c, 'c>],
    result_type: Type<'c>,
    dest: &str,
    variable_map: &mut HashMap<String, Value<'c, 'c>>,
    location: Location<'c>,
) {
    let op = OperationBuilder::new(op_name, location)
        .add_operands(args)
        .add_results(&[result_type])
        .build()
        .unwrap();
    let result = op.result(0).unwrap().into();
    variable_map.insert(dest.to_string(), result);
    block.append_operation(op);
}

fn translate_function<'c>(context: &'c Context, func: &Function) -> Operation<'c> {
    let location = Location::unknown(context);
    // Rename main to bril_main so our wrapper can use the main name
    let func_name = if func.name == "main" {
        "bril_main"
    } else {
        &func.name
    };

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

    // Split instructions into basic blocks
    let basic_blocks = split_into_blocks(&func.instrs);
    assert!(
        !basic_blocks.is_empty(),
        "Function {} has no instructions",
        func_name
    );

    // Create the region and entry block with function arguments
    let region = Region::new();
    let entry_block = Block::new(
        &arg_types
            .iter()
            .map(|ty| (*ty, location))
            .collect::<Vec<_>>(),
    );

    // Initialize variable map with function arguments
    let mut variable_map: HashMap<String, Value<'c, 'c>> = HashMap::new();
    func.args.iter().enumerate().for_each(|(i, arg)| {
        let block_arg = entry_block
            .argument(i)
            .unwrap_or_else(|_| panic!("Failed to get argument {} from entry block", i));
        variable_map.insert(arg.name.clone(), block_arg.into());
    });
    region.append_block(entry_block);

    // Create remaining blocks and build label -> block index mapping
    let mut label_to_block_idx: HashMap<String, usize> = HashMap::new();
    for (i, bb) in basic_blocks.iter().enumerate() {
        if let Some(label_name) = &bb.label {
            label_to_block_idx.insert(label_name.clone(), i);
        }
        if i > 0 {
            region.append_block(Block::new(&[]));
        }
    }

    // Emit instructions into each block
    let resolver = BlockResolver::new(&label_to_block_idx, &region);
    for (i, bb) in basic_blocks.iter().enumerate() {
        let block = get_block_by_index(&region, i).unwrap();

        bb.instructions
            .iter()
            .for_each(|instr| translate_instruction(context, instr, &block, &mut variable_map, &resolver));

        // Add implicit terminator if needed
        if !bb.has_terminator() {
            let next_label = basic_blocks.get(i + 1).and_then(|bb| bb.label.as_ref());

            if let Some(label) = next_label {
                emit_jmp(&block, &resolver.get(label), location);
            } else {
                block.append_operation(make_void_return(context));
            }
        }
    }

    OperationBuilder::new("bril.func", location)
        .add_attributes(&[
            (
                Identifier::new(context, "sym_name"),
                StringAttribute::new(context, func_name).into(),
            ),
            (
                Identifier::new(context, "function_type"),
                TypeAttribute::new(func_type.into()).into(),
            ),
        ])
        .add_regions([region])
        .build()
        .unwrap()
}

/// Translate a single instruction with access to block resolver for control flow
fn translate_instruction<'c>(
    context: &'c Context,
    instr: &Instruction,
    block: &BlockRef<'c, '_>,
    variable_map: &mut HashMap<String, Value<'c, 'c>>,
    resolver: &BlockResolver<'_, 'c>,
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
                _ => panic!("Unsupported constant type"),
            };
            let const_op = OperationBuilder::new("bril.const", location)
                .add_attributes(&[(Identifier::new(context, "value"), attr)])
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
            funcs,
            ..
        } => {
            let result_type = translate_bril_type(context, op_type);

            match op {
                ValueOps::Call => {
                    // Value-returning call
                    let func_name = funcs.first().expect("Call must have function name");
                    let call_args = collect_args(args, variable_map);
                    let call_op = OperationBuilder::new("bril.call", location)
                        .add_attributes(&[(
                            Identifier::new(context, "callee"),
                            FlatSymbolRefAttribute::new(context, func_name).into(),
                        )])
                        .add_operands(&call_args)
                        .add_results(&[result_type])
                        .build()
                        .unwrap();
                    let result = call_op.result(0).unwrap();
                    variable_map.insert(dest.clone(), result.into());
                    block.append_operation(call_op);
                }
                _ => {
                    let op_args = collect_args(args, variable_map);
                    emit_bril_value_op(block, value_op_name(op), &op_args, result_type, dest, variable_map, location);
                }
            }
        }

        Instruction::Effect {
            op,
            args,
            labels,
            funcs,
            ..
        } => match op {
            EffectOps::Print => {
                let print_args = collect_args(args, variable_map);
                let print_op = OperationBuilder::new("bril.print", location)
                    .add_operands(&print_args)
                    .build()
                    .unwrap();
                block.append_operation(print_op);
            }
            EffectOps::Return => {
                match args.first() {
                    Some(arg) => {
                        let value = *variable_map.get(arg).unwrap();
                        let ret_op = OperationBuilder::new("bril.ret", location)
                            .add_operands(&[value])
                            .build()
                            .unwrap();
                        block.append_operation(ret_op);
                    }
                    None => {
                        block.append_operation(make_void_return(context));
                    }
                };
            }
            EffectOps::Jump => {
                let target_label = labels.first().expect("Jump must have target label");
                emit_jmp(block, &resolver.get(target_label), location);
            }
            EffectOps::Branch => {
                let cond = *variable_map
                    .get(&args[0])
                    .expect("Branch condition not found");
                let true_block = resolver.get(&labels[0]);
                let false_block = resolver.get(&labels[1]);
                // operandSegmentSizes: [condition_count, true_args_count, false_args_count]
                let operand_segment_sizes = DenseI32ArrayAttribute::new(context, &[1, 0, 0]);
                let br_op = OperationBuilder::new("bril.br", location)
                    .add_operands(&[cond])
                    .add_successors(&[&true_block, &false_block])
                    .add_attributes(&[(
                        Identifier::new(context, "operandSegmentSizes"),
                        operand_segment_sizes.into(),
                    )])
                    .build()
                    .unwrap();
                block.append_operation(br_op);
            }
            EffectOps::Call => {
                // Effect call (no return value)
                let func_name = funcs.first().expect("Call must have function name");
                let call_args = collect_args(args, variable_map);
                let call_op = OperationBuilder::new("bril.call", location)
                    .add_attributes(&[(
                        Identifier::new(context, "callee"),
                        FlatSymbolRefAttribute::new(context, func_name).into(),
                    )])
                    .add_operands(&call_args)
                    .build()
                    .unwrap();
                block.append_operation(call_op);
            }
            EffectOps::Nop => {
                let nop_op = OperationBuilder::new("bril.nop", location).build().unwrap();
                block.append_operation(nop_op);
            }
            _ => {
                unimplemented!("EffectOp {:?} not implemented", op)
            }
        },
    }
}

pub(crate) fn translate_bril_type<'c>(context: &'c Context, bril_ty: &BrilType) -> Type<'c> {
    match bril_ty {
        BrilType::Int => IntegerType::new(context, 64).into(),
        BrilType::Bool => IntegerType::new(context, 1).into(),
        BrilType::Pointer(_) => unimplemented!("Pointer types not yet supported"),
    }
}
