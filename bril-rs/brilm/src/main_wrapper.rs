//! Main wrapper generation for handling argc/argv parsing

use bril_rs::{Argument, Type as BrilType};
use melior::{
    Context,
    dialect::{
        cf,
        llvm::{self, AllocaOptions, LoadStoreOptions, r#type as llvm_type},
    },
    ir::{
        Identifier,
        attribute::{
            BoolAttribute, DenseI32ArrayAttribute, FlatSymbolRefAttribute,
            StringAttribute, TypeAttribute,
        },
        operation::{OperationBuilder, OperationLike},
        r#type::IntegerType,
        *,
    },
};

use crate::translator::{emit_cmpi_eq, make_const, translate_bril_type};

/// Declare an extern function with the given name, return type, and argument types
fn declare_extern_func<'c>(
    context: &'c Context,
    name: &str,
    return_type: Type<'c>,
    arg_types: &[Type<'c>],
) -> Operation<'c> {
    let location = Location::unknown(context);
    let func_type = llvm_type::function(return_type, arg_types, false);
    llvm::func(
        context,
        StringAttribute::new(context, name),
        TypeAttribute::new(func_type),
        Region::new(),
        &[],
        location,
    )
}

/// Declare extern strtoll function for int parsing
pub fn declare_strtoll<'c>(context: &'c Context) -> Operation<'c> {
    let i32_type: Type = IntegerType::new(context, 32).into();
    let i64_type: Type = IntegerType::new(context, 64).into();
    let ptr_type: Type = llvm_type::pointer(context, 0);
    declare_extern_func(context, "strtoll", i64_type, &[ptr_type, ptr_type, i32_type])
}

/// Declare extern strcmp function for bool parsing
pub fn declare_strcmp<'c>(context: &'c Context) -> Operation<'c> {
    let i32_type: Type = IntegerType::new(context, 32).into();
    let ptr_type: Type = llvm_type::pointer(context, 0);
    declare_extern_func(context, "strcmp", i32_type, &[ptr_type, ptr_type])
}

/// Declare a global constant string for bool comparison
pub fn declare_bool_string<'c>(context: &'c Context, name: &str, value: &str) -> Operation<'c> {
    let location = Location::unknown(context);
    let i8_type: Type = IntegerType::new(context, 8).into();
    let array_type = llvm_type::array(i8_type, value.len() as u32);

    // Create initializer region with return
    let region = Region::new();
    let block = Block::new(&[]);

    let const_op = OperationBuilder::new("llvm.mlir.constant", location)
        .add_attributes(&[(
            Identifier::new(context, "value"),
            StringAttribute::new(context, value).into(),
        )])
        .add_results(&[array_type])
        .build()
        .unwrap();
    let str_val = const_op.result(0).unwrap().into();
    block.append_operation(const_op);

    block.append_operation(llvm::r#return(Some(str_val), location));
    region.append_block(block);

    OperationBuilder::new("llvm.mlir.global", location)
        .add_attributes(&[
            (
                Identifier::new(context, "sym_name"),
                StringAttribute::new(context, name).into(),
            ),
            (
                Identifier::new(context, "global_type"),
                TypeAttribute::new(array_type).into(),
            ),
            (
                Identifier::new(context, "linkage"),
                llvm::attributes::linkage(context, llvm::attributes::Linkage::Internal),
            ),
            (
                Identifier::new(context, "constant"),
                BoolAttribute::new(context, true).into(),
            ),
        ])
        .add_regions([region])
        .build()
        .unwrap()
}

/// Helper to emit an llvm.load
fn emit_load<'c>(
    context: &'c Context,
    block: &Block<'c>,
    ptr: Value<'c, 'c>,
    ty: Type<'c>,
    location: Location<'c>,
) -> Value<'c, 'c> {
    let op = llvm::load(context, ptr, ty, location, LoadStoreOptions::default());
    let val = op.result(0).unwrap().into();
    block.append_operation(op);
    val
}

/// Helper to emit an llvm.call
fn emit_llvm_call<'c>(
    context: &'c Context,
    block: &Block<'c>,
    callee: &str,
    args: &[Value<'c, 'c>],
    result_type: Type<'c>,
    location: Location<'c>,
) -> Value<'c, 'c> {
    let op = OperationBuilder::new("llvm.call", location)
        .add_attributes(&[
            (Identifier::new(context, "callee"), FlatSymbolRefAttribute::new(context, callee).into()),
            (Identifier::new(context, "operandSegmentSizes"), DenseI32ArrayAttribute::new(context, &[args.len() as i32, 0]).into()),
            (Identifier::new(context, "op_bundle_sizes"), DenseI32ArrayAttribute::new(context, &[]).into()),
        ])
        .add_operands(args)
        .add_results(&[result_type])
        .build()
        .unwrap();
    let val = op.result(0).unwrap().into();
    block.append_operation(op);
    val
}

/// Helper to emit strcmp(str, global_name) == 0
fn emit_strcmp_eq<'c>(
    context: &'c Context,
    block: &Block<'c>,
    str_val: Value<'c, 'c>,
    global_name: &str,
    location: Location<'c>,
) -> Value<'c, 'c> {
    let i32_type: Type = IntegerType::new(context, 32).into();
    let ptr_type: Type = llvm_type::pointer(context, 0);

    // Get address of global string
    let addr_op = OperationBuilder::new("llvm.mlir.addressof", location)
        .add_attributes(&[(
            Identifier::new(context, "global_name"),
            FlatSymbolRefAttribute::new(context, global_name).into(),
        )])
        .add_results(&[ptr_type])
        .build()
        .unwrap();
    let global_ptr = addr_op.result(0).unwrap().into();
    block.append_operation(addr_op);

    // strcmp(str_val, global_ptr)
    let cmp_result = emit_llvm_call(context, block, "strcmp", &[str_val, global_ptr], i32_type, location);

    // result == 0
    let zero = make_const(context, block, i32_type, 0, location);
    emit_cmpi_eq(context, block, cmp_result, zero, location)
}

/// Parse and validate a command-line argument from argv, then branch.
/// Branches to success_block with [success_args_prefix..., parsed_value] on valid input,
/// or to error_block on invalid input. Returns the parsed value.
#[allow(clippy::too_many_arguments)]
fn parse_validate_and_branch<'c>(
    context: &'c Context,
    block: &Block<'c>,
    argv_val: Value<'c, 'c>,
    arg_idx: usize,
    arg_type: &BrilType,
    success_args_prefix: &[Value<'c, 'c>],
    success_block: &Block<'c>,
    error_block: &Block<'c>,
    location: Location<'c>,
) -> Value<'c, 'c> {
    let i32_type: Type = IntegerType::new(context, 32).into();
    let i64_type: Type = IntegerType::new(context, 64).into();
    let ptr_type: Type = llvm_type::pointer(context, 0);

    // Load argv[arg_idx + 1]
    let idx = make_const(context, block, i64_type, (arg_idx + 1) as i64, location);
    let gep_op = llvm::get_element_ptr_dynamic(context, argv_val, &[idx], ptr_type, ptr_type, location);
    let arg_ptr = gep_op.result(0).unwrap().into();
    block.append_operation(gep_op);

    let arg_str = emit_load(context, block, arg_ptr, ptr_type, location);

    let (parsed, valid) = match arg_type {
        BrilType::Int => {
            // Allocate space for endptr on stack
            let one_val = make_const(context, block, i64_type, 1, location);
            let alloca_op = llvm::alloca(
                context,
                one_val,
                ptr_type,
                location,
                AllocaOptions::new().elem_type(Some(TypeAttribute::new(ptr_type))),
            );
            let endptr_ptr = alloca_op.result(0).unwrap().into();
            block.append_operation(alloca_op);

            // strtoll(str, &endptr, 10)
            let base = make_const(context, block, i32_type, 10, location);
            let val = emit_llvm_call(context, block, "strtoll", &[arg_str, endptr_ptr, base], i64_type, location);

            // Load endptr, then load *endptr (the character at endptr)
            let endptr = emit_load(context, block, endptr_ptr, ptr_type, location);
            let i8_type: Type = IntegerType::new(context, 8).into();
            let end_char = emit_load(context, block, endptr, i8_type, location);

            // Valid if *endptr == '\0'
            let zero_i8 = make_const(context, block, i8_type, 0, location);
            let valid = emit_cmpi_eq(context, block, end_char, zero_i8, location);

            (val, valid)
        }
        BrilType::Bool => {
            let is_true = emit_strcmp_eq(context, block, arg_str, "str_true", location);
            let is_false = emit_strcmp_eq(context, block, arg_str, "str_false", location);

            // valid = is_true OR is_false
            let i1_type: Type = IntegerType::new(context, 1).into();
            let valid_op = OperationBuilder::new("arith.ori", location)
                .add_operands(&[is_true, is_false])
                .add_results(&[i1_type])
                .build()
                .unwrap();
            let valid = valid_op.result(0).unwrap().into();
            block.append_operation(valid_op);

            (is_true, valid)
        }
        _ => unimplemented!("Argument type not supported for main"),
    };

    // Branch to success or error based on validity
    let mut success_args: Vec<Value> = success_args_prefix.to_vec();
    success_args.push(parsed);
    block.append_operation(cf::cond_br(
        context,
        valid,
        success_block,
        error_block,
        &success_args,
        &[],
        location,
    ));

    parsed
}

/// Create a wrapper `@main` that calls `@bril_main` and returns 0.
/// If bril_main has arguments, parses them from argc/argv and validates argc.
pub fn create_main_wrapper<'c>(context: &'c Context, main_args: &[Argument]) -> Operation<'c> {
    let location = Location::unknown(context);
    let i32_type: Type = IntegerType::new(context, 32).into();
    let ptr_type: Type = llvm_type::pointer(context, 0);
    let func_type = llvm_type::function(i32_type, &[i32_type, ptr_type], false);

    let region = Region::new();

    // Entry block: validate argc
    let entry_block = Block::new(&[(i32_type, location), (ptr_type, location)]);
    let argc = entry_block.argument(0).unwrap();
    let argv = entry_block.argument(1).unwrap();

    // Error block: return 1
    let error_block = Block::new(&[]);
    let one = make_const(context, &error_block, i32_type, 1, location);
    error_block.append_operation(llvm::r#return(Some(one), location));

    // Compute arg types for block arguments
    let arg_types: Vec<Type> = main_args
        .iter()
        .map(|a| translate_bril_type(context, &a.arg_type))
        .collect();

    // Create all blocks upfront:
    // - body_block: receives argv, parses first arg
    // - continuation blocks: one after each arg that has more args after it
    // - call_block: receives all parsed args, calls bril_main
    let body_block = Block::new(&[(ptr_type, location)]);

    // Each continuation block receives (argv, args_parsed_so_far...)
    // We need one for each arg except the last (which branches to call_block)
    let mut continuation_blocks: Vec<Block> = Vec::new();
    for i in 1..main_args.len() {
        let mut block_arg_types: Vec<(Type, Location)> = vec![(ptr_type, location)];
        for &arg_type in arg_types.iter().take(i) {
            block_arg_types.push((arg_type, location));
        }
        continuation_blocks.push(Block::new(&block_arg_types));
    }

    // Call block receives all parsed args
    let call_block_arg_types: Vec<(Type, Location)> =
        arg_types.iter().map(|&t| (t, location)).collect();
    let call_block = Block::new(&call_block_arg_types);

    // Entry: check argc == expected
    let expected_argc = (main_args.len() + 1) as i64;
    let expected = make_const(context, &entry_block, i32_type, expected_argc, location);
    let cmp = emit_cmpi_eq(context, &entry_block, argc.into(), expected, location);
    entry_block.append_operation(cf::cond_br(
        context,
        cmp,
        &body_block,
        &error_block,
        &[argv.into()],
        &[],
        location,
    ));

    // Fill in the blocks
    let mut current_block = &body_block;
    let mut argv_val: Value = body_block.argument(0).unwrap().into();
    let mut parsed_args: Vec<Value> = Vec::new();

    if let Some((last_arg, init)) = main_args.split_last() {
        // Process all args except last - they branch to continuation blocks
        for (i, arg) in init.iter().enumerate() {
            let next_block = &continuation_blocks[i];
            let mut success_prefix: Vec<Value> = vec![argv_val];
            success_prefix.extend(parsed_args.iter().copied());

            let parsed = parse_validate_and_branch(
                context, current_block, argv_val, i, &arg.arg_type,
                &success_prefix, next_block, &error_block, location,
            );
            parsed_args.push(parsed);

            current_block = next_block;
            argv_val = current_block.argument(0).unwrap().into();
            parsed_args = (1..=parsed_args.len())
                .map(|j| current_block.argument(j).unwrap().into())
                .collect();
        }

        // Process last arg - branches to call_block
        parse_validate_and_branch(
            context, current_block, argv_val, init.len(), &last_arg.arg_type,
            &parsed_args, &call_block, &error_block, location,
        );
    } else {
        // No args - branch directly to call_block
        current_block.append_operation(cf::br(&call_block, &[], location));
    }

    // Call block: call bril_main with parsed arguments
    let final_args: Vec<Value> = (0..main_args.len())
        .map(|j| call_block.argument(j).unwrap().into())
        .collect();
    let call_op = OperationBuilder::new("bril.call", location)
        .add_attributes(&[(
            Identifier::new(context, "callee"),
            FlatSymbolRefAttribute::new(context, "bril_main").into(),
        )])
        .add_operands(&final_args)
        .build()
        .unwrap();
    call_block.append_operation(call_op);

    let zero = make_const(context, &call_block, i32_type, 0, location);
    call_block.append_operation(llvm::r#return(Some(zero), location));

    // Add all blocks to region
    region.append_block(entry_block);
    region.append_block(body_block);
    continuation_blocks
        .into_iter()
        .for_each(|block| { region.append_block(block); });
    region.append_block(call_block);
    region.append_block(error_block);

    llvm::func(
        context,
        StringAttribute::new(context, "main"),
        TypeAttribute::new(func_type),
        region,
        &[],
        location,
    )
}
