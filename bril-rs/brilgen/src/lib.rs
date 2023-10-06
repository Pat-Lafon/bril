use std::collections::HashMap;

use bril_rs::{self, ConstOps, Function, Instruction, Literal, Type, ValueOps};
use rand::{random, seq::SliceRandom, thread_rng};

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

fn gen_const_op(location_gen: &mut LocationGen) -> Instruction {
    let binding = [
        (Literal::Int(random()), Type::Int),
        (Literal::Bool(random()), Type::Bool),
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
        args: (),
        dest: (),
        funcs: (),
        labels: (),
        op: (),
        op_type: (),
    }
}

fn gen_function(args: ()) -> Function {
    todo!()
}
