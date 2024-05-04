use core::num;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::ops::Range;

use bril_rs::{self, Argument, Code, ConstOps, Function, Instruction, Literal, Program, Type};
use rand::prelude::*;

// LocationGen is not a typing Context, it is a collection of names such that
// new or old names can be used when generating new destinations
// TODO: Do they need to be mapped based on types? or just a
// set?(Non-deterministically overwrite something of a different type?)
struct LocationGen {
    map: HashMap<Type, HashSet<String>>,
    counter: usize,
}

impl LocationGen {
    pub fn get_location(&mut self, ty: Type) -> String {
        let loc_vec = self.map.entry(ty).or_default();
        if !loc_vec.is_empty() && random() {
            loc_vec.iter().choose(&mut thread_rng()).unwrap().clone()
        } else {
            let loc = format!("var{}", self.counter);
            self.counter += 1;
            loc_vec.insert(loc.clone());
            loc
        }
    }
}

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

// TODO: Would be interesting to use rand's distributions instead of the
// standard range
pub struct Config {
    pub total_num_functions: NonZeroU32,
    pub num_blocks_range: Range<u32>,
    pub num_args_range: Range<u32>,
    pub rng: StdRng,
}

struct FunctionSig {
    pub name: String,
    pub args: Vec<Argument>,
    pub return_type: Option<Type>,
}

impl Config {
    fn gen_type(&mut self) -> Type {
        let binding = [Type::Int, Type::Bool];
        binding.choose(&mut self.rng).unwrap().clone()
    }

    fn gen_argument(&mut self, arg_name: String) -> Argument {
        Argument {
            name: arg_name,
            arg_type: self.gen_type(),
        }
    }

    fn gen_function(&mut self, sig: &FunctionSig, function_context: Vec<FunctionSig>) -> Function {
        let typing_context = sig.args.iter().fold(HashMap::new(), |mut acc, arg| {
            let m: &mut HashSet<_> = acc.entry(arg.arg_type.clone()).or_default();
            m.insert(arg.name.clone());
            acc
        });

        let mut location_gen = LocationGen {
            map: typing_context.clone(),
            counter: 0,
        };

        let blocks = (1..self.rng.gen_range(self.num_args_range.clone())).map(|_| todo!());

        Function {
            args: sig.args,
            instrs,
            name: sig.name,
            return_type: sig.return_type,
        }
    }

    pub fn gen_program(&mut self) -> Program {
        let mut function_signatures = vec![FunctionSig {
            name: "main".to_string(),
            args: vec![],
            return_type: None,
        }];
        let funcs = (1..self.total_num_functions.into()).map(|i| {
            let args = (0..self.rng.gen_range(self.num_args_range.clone()))
                .map(|j| self.gen_argument(format!("arg{j}")))
                .collect();
            FunctionSig {
                name: format!("func{i}"),
                args,
                return_type: Some(self.gen_type()),
            }
        });

        function_signatures.extend(funcs);

        let functions: Vec<Function> = unimplemented!();
        functions.shuffle(&mut self.rng);
        Program { functions }
    }
}
