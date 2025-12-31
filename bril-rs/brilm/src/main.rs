use melior::{
    Context,
    dialect::{DialectRegistry, arith, func},
    ir::{
        attribute::{StringAttribute, TypeAttribute},
        operation::OperationLike,
        r#type::FunctionType,
        *,
    },
    utility::register_all_dialects,
};

melior::dialect! {
    name: "bril",
    files: ["../../brilir/include/bril/BrilDialect.td", "../../brilir/include/bril/BrilOps.td" , "../../brilir/include/bril/BrilPasses.td", "../../brilir/include/bril/BrilTypes.td"],
    include_directories: ["../../brilir/include"]
}

pub fn main() {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let location = Location::unknown(&context);
    let module = Module::new(location);

    let index_type = Type::index(&context);

    module.body().append_operation(func::func(
        &context,
        StringAttribute::new(&context, "add"),
        TypeAttribute::new(
            FunctionType::new(&context, &[index_type, index_type], &[index_type]).into(),
        ),
        {
            let block = Block::new(&[(index_type, location), (index_type, location)]);

            let sum = block
                .append_operation(arith::addi(
                    block.argument(0).unwrap().into(),
                    block.argument(1).unwrap().into(),
                    location,
                ))
                .result(0)
                .unwrap();

            block.append_operation(func::r#return(&[sum.into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        &[],
        location,
    ));

    assert!(module.as_operation().verify());
    println!("blah")
}
