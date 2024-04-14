use core::num;
use std::collections::HashMap;

use bril_rs::{self, Argument, ConstOps, Function, Instruction, Literal, Program, Type};
use rand::{random, seq::SliceRandom, thread_rng};

// LocationGen is not a typing Context, it is a collection of names such that
// new or old names can be used when generating new destinations
// TODO: Do they need to be mapped based on types? or just a
// set?(Non-deterministically overwrite something of a different type?)
struct LocationGen {
    map: HashMap<Type, Vec<String>>,
    counter: usize,
}

impl LocationGen {
    pub fn get_location(&mut self, ty: Type) -> String {
        let loc_vec = self.map.entry(ty).or_default();
        if !loc_vec.is_empty() && random() {
            loc_vec.choose(&mut thread_rng()).unwrap().clone()
        } else {
            let loc = format!("var{}", self.counter);
            self.counter += 1;
            loc_vec.push(loc.clone());
            loc
        }
    }
}

// Generates a random integer... you could imagine coming up with a non-uniform distribution
fn gen_int() -> i64 {
    random()
}

fn gen_bool() -> bool {
    random()
}

fn gen_const_op(location_gen: &mut LocationGen) -> Instruction {
    let binding = [
        (Literal::Int(gen_int()), Type::Int),
        (Literal::Bool(gen_bool()), Type::Bool),
    ];

    let (value, const_type) = binding.choose(&mut thread_rng()).unwrap();
    let dest = location_gen.get_location(const_type.clone());
    Instruction::Constant {
        dest: dest,
        op: ConstOps::Const,
        const_type: const_type.clone(),
        value: value.clone(),
    }
}

fn gen_value_op(location_gen: &mut LocationGen) -> Instruction {
    todo!();
    Instruction::Value {
        args: unimplemented!(),
        dest: unimplemented!(),
        funcs: unimplemented!(),
        labels: unimplemented!(),
        op: unimplemented!(),
        op_type: unimplemented!(),
    }
}

fn gen_function(function_context: (), args: Vec<Argument>, num_blocks: u32) -> Function {
    todo!();
    let mut location_gen = LocationGen {
        map: args.iter().fold(HashMap::new(), |mut acc, arg| {
            acc.insert(arg.arg_type.clone(), vec![arg.name.clone()]);
            acc
        }),
        counter: 0,
    };
    Function {
        args,
        instrs: unimplemented!(),
        name: unimplemented!(),
        return_type: unimplemented!(),
    }
}

pub fn gen_program(num_functions:u32) -> Program {
    assert!(num_functions > 1, "Must have at least one function");
    // Iteratively build up functions one after the other
    Program {
        functions: unimplemented!(),
    }
}
