use bril_rs::{
    Argument, Code, ConstOps, EffectOps, Function, Instruction, Literal, Program, Type, ValueOps,
};
use fxhash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Pointer {
    base: i32,
    offset: i64,
    ptr_type: Type,
}

impl Pointer {
    fn add(&self, offset: i64) -> Pointer {
        Pointer {
            base: self.base,
            offset: self.offset + offset,
            ptr_type: self.ptr_type.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Lit(Literal),
    Ptr(Pointer),
}

impl Value {
    fn to_string(&self) -> String {
        match self {
            Value::Lit(Literal::Int(i)) => i.to_string(),
            Value::Lit(Literal::Bool(b)) => b.to_string(),
            Value::Lit(Literal::Float(f)) => f.to_string(),
            Value::Ptr(Pointer {
                base: _,
                offset,
                ptr_type,
            }) => format!("{:?}{}", ptr_type, offset),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Lit(Literal::Int(-1))
    }
}

fn type_check_lit(lit: &Literal, typ: &Type) -> bool {
    match lit {
        Literal::Bool(_) => &Type::Bool == typ,
        Literal::Int(_) => &Type::Int == typ,
        Literal::Float(_) => &Type::Float == typ,
    }
}

fn type_check_val(val: &Value, typ: &Type) -> bool {
    match val {
        Value::Lit(Literal::Bool(_)) => &Type::Bool == typ,
        Value::Lit(Literal::Int(_)) => &Type::Int == typ,
        Value::Lit(Literal::Float(_)) => &Type::Float == typ,
        Value::Ptr(Pointer { ptr_type, .. }) => ptr_type == typ,
    }
}

fn expect_int(val: &Value) -> Result<i64, String> {
    match val {
        Value::Lit(Literal::Int(i)) => Ok(*i),
        Value::Lit(Literal::Float(_)) => Err("Expected int found float".to_string()),
        Value::Lit(Literal::Bool(_)) => Err("Expected int found bool".to_string()),
        Value::Ptr(Pointer { ptr_type, .. }) => Err(format!("Expected int found {:?}", ptr_type)),
    }
}

fn expect_float(val: &Value) -> Result<f64, String> {
    match val {
        Value::Lit(Literal::Int(_)) => Err("Expected float found int".to_string()),
        Value::Lit(Literal::Float(f)) => Ok(*f),
        Value::Lit(Literal::Bool(_)) => Err("Expected int found bool".to_string()),
        Value::Ptr(Pointer { ptr_type, .. }) => Err(format!("Expected float found {:?}", ptr_type)),
    }
}

fn expect_bool(val: &Value) -> Result<bool, String> {
    match val {
        Value::Lit(Literal::Int(_)) => Err("Expected bool found int".to_string()),
        Value::Lit(Literal::Float(_)) => Err("Expected bool found float".to_string()),
        Value::Lit(Literal::Bool(b)) => Ok(*b),
        Value::Ptr(Pointer { ptr_type, .. }) => Err(format!("Expected bool found {:?}", ptr_type)),
    }
}

fn expect_ptr(val: &Value) -> Result<Pointer, String> {
    match val {
        Value::Lit(Literal::Int(_)) => Err("Expected ptr found int".to_string()),
        Value::Lit(Literal::Float(_)) => Err("Expected ptr found float".to_string()),
        Value::Lit(Literal::Bool(_)) => Err("Expected ptr found bool".to_string()),
        Value::Ptr(p @ Pointer { .. }) => Ok(p.clone()),
    }
}

fn convert_op_bool(op: &ValueOps) -> fn(bool, bool) -> Value {
    match op {
        ValueOps::And => (|x, y| Value::Lit(Literal::Bool(x && y))),
        ValueOps::Or => (|x, y| Value::Lit(Literal::Bool(x || y))),
        _ => unreachable!(),
    }
}

fn convert_op_int(op: &ValueOps) -> fn(i64, i64) -> Value {
    match op {
        ValueOps::Add => (|x, y| Value::Lit(Literal::Int(x + y))),
        ValueOps::Sub => (|x, y| Value::Lit(Literal::Int(x - y))),
        ValueOps::Mul => (|x, y| Value::Lit(Literal::Int(x * y))),
        ValueOps::Div => (|x, y| Value::Lit(Literal::Int(x / y))),
        _ => unreachable!(),
    }
}

fn convert_op_eqv(op: &ValueOps) -> fn(i64, i64) -> Value {
    match op {
        ValueOps::Eq => (|x, y| Value::Lit(Literal::Bool(x == y))),
        ValueOps::Lt => (|x, y| Value::Lit(Literal::Bool(x < y))),
        ValueOps::Gt => (|x, y| Value::Lit(Literal::Bool(x > y))),
        ValueOps::Le => (|x, y| Value::Lit(Literal::Bool(x <= y))),
        ValueOps::Ge => (|x, y| Value::Lit(Literal::Bool(x >= y))),
        _ => unreachable!(),
    }
}

fn convert_op_float(op: &ValueOps) -> fn(f64, f64) -> Value {
    match op {
        ValueOps::Fadd => (|x, y| Value::Lit(Literal::Float(x + y))),
        ValueOps::Fsub => (|x, y| Value::Lit(Literal::Float(x - y))),
        ValueOps::Fmul => (|x, y| Value::Lit(Literal::Float(x * y))),
        ValueOps::Fdiv => (|x, y| Value::Lit(Literal::Float(x / y))),
        _ => unreachable!(),
    }
}

fn convert_op_feqv(op: &ValueOps) -> fn(f64, f64) -> Value {
    match op {
        ValueOps::Feq => (|x, y| Value::Lit(Literal::Bool(x == y))),
        ValueOps::Flt => (|x, y| Value::Lit(Literal::Bool(x < y))),
        ValueOps::Fgt => (|x, y| Value::Lit(Literal::Bool(x > y))),
        ValueOps::Fle => (|x, y| Value::Lit(Literal::Bool(x <= y))),
        ValueOps::Fge => (|x, y| Value::Lit(Literal::Bool(x >= y))),
        _ => unreachable!(),
    }
}

trait BrilType {
    fn is_ptr(&self) -> bool;
}

impl BrilType for Type {
    fn is_ptr(&self) -> bool {
        match self {
            Type::Pointer(_) => true,
            Type::Bool | Type::Int | Type::Float => false,
        }
    }
}

pub struct Heap {
    memory: FxHashMap<i32, Vec<Value>>,
    base_num_counter: i32,
}

impl Default for Heap {
    fn default() -> Self {
        Heap {
            memory: FxHashMap::default(),
            base_num_counter: 0,
        }
    }
}

impl Heap {
    fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }
    fn alloc(&mut self, amount: i64, ptr_type: Type) -> Result<Value, String> {
        if !ptr_type.is_ptr() {
            return Err(format!("unspecified pointer type {:?}", ptr_type));
        }
        if amount < 0 {
            return Err(format!("cannot allocate {} entries", amount));
        }
        let base = self.base_num_counter;
        self.base_num_counter += 1;
        self.memory
            .insert(base, vec![Value::default(); amount as usize]);
        Ok(Value::Ptr(Pointer {
            base,
            offset: 0,
            ptr_type,
        }))
    }

    fn free(&mut self, key: Pointer) -> Result<(), String> {
        if self.memory.remove(&key.base).is_some() && key.offset == 0 {
            Ok(())
        } else {
            Err(format!(
                "Tried to free illegal memory location base: {}, offset: {}. Offset must be 0.",
                key.base, key.offset
            ))
        }
    }

    fn write(&mut self, key: &Pointer, val: Value) -> Result<(), String> {
        match self.memory.get_mut(&key.base) {
            Some(vec) if vec.len() > (key.offset as usize) => {
                vec[key.offset as usize] = val;
                Ok(())
            }
            Some(_) | None => Err(format!(
                "Uninitialized heap location {} and/or illegal offset {}",
                key.base, key.offset
            )),
        }
    }

    fn read(&self, key: &Pointer) -> Result<&Value, String> {
        self.memory
            .get(&key.base)
            .and_then(|vec| vec.get(key.offset as usize))
            .ok_or(format!(
                "Uninitialized heap location {} and/or illegal offset {}",
                key.base, key.offset
            ))
    }
}

pub struct Environment<'a> {
    env: FxHashMap<&'a str, Value>,
}

impl Default for Environment<'_> {
    fn default() -> Self {
        Environment {
            env: FxHashMap::default(),
        }
    }
}

impl<'a> Environment<'a> {
    fn get(&self, ident: &str) -> Result<&Value, String> {
        self.env
            .get(ident)
            .ok_or(format!("undefined variable {}", ident))
    }
    fn set(&mut self, ident: &'a str, val: Value) {
        self.env.insert(ident, val);
    }
}

pub struct State<'a> {
    heap: Rc<RefCell<Heap>>,
    instruction_count: i32,
    current_label: Option<&'a str>,
    last_label: Option<&'a str>,
}

impl Default for State<'_> {
    fn default() -> Self {
        State {
            heap: Rc::new(RefCell::new(Heap::default())),
            instruction_count: 0,
            current_label: None,
            last_label: None,
        }
    }
}

impl State<'_> {
    fn new_state(&self) -> Self {
        State {
            heap: self.heap.clone(),
            instruction_count: self.instruction_count,
            current_label: None,
            last_label: None,
        }
    }
}

#[inline(always)]
pub fn meta_err_info<T>(func: &str, line: &usize, result: Result<T, String>) -> Result<T, String> {
    result.map_err(|msg| format!("Function {}, Line {} : {}", func, line, msg))
}

macro_rules! err {
    ($func:tt, $index:tt, $($x:tt)*) => {
        meta_err_info(&$func.name, &$index, $($x)*)?
    };
}

fn execute<'a>(
    func: &'a Function,
    label_map: &'a FxHashMap<String, usize>,
    mut state: State<'a>,
    mut env: Environment<'a>,
    funcs: &'a FxHashMap<String, (Function, FxHashMap<String, usize>)>,
) -> Result<(State<'a>, Option<Value>), String> {
    let mut index = 0;

    while let Some(code) = func.instrs.get(index) {
        state.instruction_count += 1;

        match code {
            Code::Label { label } => {
                state.last_label = state.current_label;
                state.current_label = Some(label);
                state.instruction_count -= 1;
                index += 1
            }
            Code::Instruction(Instruction::Constant {
                op: ConstOps::Const,
                dest,
                const_type,
                value,
            }) => {
                let value = if const_type == &Type::Float {
                    match value {
                        Literal::Int(i) => Literal::Float(*i as f64),
                        _ => value.clone(),
                    }
                } else {
                    value.clone()
                };
                if !type_check_lit(&value, const_type) {
                    err!(func, index, Err("Type error".to_string()));
                }
                env.set(&dest, Value::Lit(value));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: op @ (ValueOps::Add | ValueOps::Sub | ValueOps::Mul | ValueOps::Div),
                dest,
                op_type: Type::Int,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_int)
                );
                let arg2 = err!(
                    func,
                    index,
                    env.get(vec.get(1).unwrap()).and_then(expect_int)
                );

                env.set(&dest, convert_op_int(op)(arg1, arg2));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: op @ (ValueOps::Eq | ValueOps::Lt | ValueOps::Gt | ValueOps::Le | ValueOps::Ge),
                dest,
                op_type: Type::Bool,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_int)
                );
                let arg2 = err!(
                    func,
                    index,
                    env.get(vec.get(1).unwrap()).and_then(expect_int)
                );

                env.set(&dest, convert_op_eqv(op)(arg1, arg2));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::Not,
                dest,
                op_type: Type::Bool,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_bool)
                );

                env.set(&dest, Value::Lit(Literal::Bool(!arg1)));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: op @ (ValueOps::And | ValueOps::Or),
                dest,
                op_type: Type::Bool,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_bool)
                );
                let arg2 = err!(
                    func,
                    index,
                    env.get(vec.get(1).unwrap()).and_then(expect_bool)
                );

                env.set(&dest, convert_op_bool(op)(arg1, arg2));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: op @ (ValueOps::Fadd | ValueOps::Fsub | ValueOps::Fmul | ValueOps::Fdiv),
                dest,
                op_type: Type::Float,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_float)
                );
                let arg2 = err!(
                    func,
                    index,
                    env.get(vec.get(1).unwrap()).and_then(expect_float)
                );

                env.set(&dest, convert_op_float(op)(arg1, arg2));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op:
                    op @ (ValueOps::Feq | ValueOps::Flt | ValueOps::Fgt | ValueOps::Fle | ValueOps::Fge),
                dest,
                op_type: Type::Bool,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_float)
                );
                let arg2 = err!(
                    func,
                    index,
                    env.get(vec.get(1).unwrap()).and_then(expect_float)
                );

                env.set(&dest, convert_op_feqv(op)(arg1, arg2));
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::Id,
                dest,
                op_type,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(func, index, env.get(vec.get(0).unwrap())).clone();

                if !type_check_val(&arg1, op_type) {
                    err!(func, index, Err("Type error".to_string()));
                }

                env.set(&dest, arg1);
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::Call,
                dest,
                op_type,
                args,
                funcs: Some(fvec),
                labels: _,
            }) => {
                if fvec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of functions".to_string())
                    )
                }
                let (callee_f, callee_map) = err!(
                    func,
                    index,
                    funcs
                        .get(fvec.get(0).unwrap())
                        .ok_or(format!("no function of name {} found", func.name))
                );

                let next_state = State::new_state(&state);
                let mut next_env = Environment::default();

                if args.is_none() && callee_f.args.is_none() {
                    // do nothing because we have not args to add to the environment
                } else if args.is_some() && callee_f.args.is_some() {
                    let args_vec = args.as_ref().unwrap();
                    let callee_vec = callee_f.args.as_ref().unwrap();
                    if args_vec.len() != callee_vec.len() {
                        err!(
                            func,
                            index,
                            Err("Unexpected number of arguments".to_string())
                        )
                    }
                    err!(
                        func,
                        index,
                        args_vec
                            .iter()
                            .zip(callee_vec.iter())
                            .map(|(arg_name, expected_arg)| {
                                let arg = env.get(arg_name)?;
                                if !type_check_val(&arg, &expected_arg.arg_type) {
                                    return Err("Type error".to_string());
                                }
                                Ok((expected_arg.name.as_ref(), arg.clone()))
                            })
                            .try_for_each(|res| res.map(|(name, val)| next_env.set(name, val)))
                    );
                } else {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }

                let ret_type = err!(
                    func,
                    index,
                    callee_f
                        .return_type
                        .as_ref()
                        .ok_or_else(|| "Expected a return type and found none".to_string())
                );

                if ret_type != op_type {
                    err!(
                        func,
                        index,
                        Err(format!("Expected {:?} found {:?}", op_type, ret_type))
                    )
                }

                let (next_state, ret_opt) =
                    execute(&callee_f, callee_map, next_state, next_env, funcs)?;
                let ret = ret_opt.unwrap();

                state.instruction_count = next_state.instruction_count;

                if !type_check_val(&ret, op_type) {
                    err!(func, index, Err("Type error".to_string()));
                }

                env.set(&dest, ret);
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::Phi,
                dest,
                op_type,
                args: Some(vec),
                funcs: _,
                labels: Some(label_vec),
            }) => {
                if vec.len() != label_vec.len() {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }

                if state.last_label.is_none() {
                    err!(
                        func,
                        index,
                        Err("No previous label for phi node".to_string())
                    )
                }

                let val = err!(
                    func,
                    index,
                    label_vec
                        .iter()
                        .position(|l| l == state.last_label.as_ref().unwrap())
                        .and_then(|i| vec.get(i))
                        .ok_or_else(|| "Last label was not found in phi node".to_string())
                        .and_then(|var| env.get(&var))
                )
                .clone();

                if !type_check_val(&val, op_type) {
                    err!(func, index, Err("Type error".to_string()));
                }

                env.set(&dest, val);
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::PtrAdd,
                dest,
                op_type,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_ptr)
                );
                let arg2 = err!(
                    func,
                    index,
                    env.get(vec.get(1).unwrap()).and_then(expect_int)
                );

                let val = Value::Ptr(arg1.add(arg2));

                if !type_check_val(&val, op_type) {
                    err!(func, index, Err("Type error".to_string()));
                }

                env.set(&dest, val);
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::Alloc,
                dest,
                op_type,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_int)
                );

                let val = err!(
                    func,
                    index,
                    state.heap.borrow_mut().alloc(arg1, op_type.clone())
                );

                env.set(&dest, val);
                index += 1
            }
            Code::Instruction(Instruction::Value {
                op: ValueOps::Load,
                dest,
                op_type,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_ptr)
                );

                let val = err!(func, index, state.heap.borrow_mut().read(&arg1)).clone();

                if !type_check_val(&val, op_type) {
                    err!(func, index, Err("Type error".to_string()));
                }

                env.set(&dest, val);
                index += 1
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Jump,
                args: _,
                funcs: _,
                labels: Some(labels),
            }) => {
                if labels.len() != 1 {
                    err!(func, index, Err("Unexpected number of labels".to_string()))
                }
                index = *err!(
                    func,
                    index,
                    label_map
                        .get(labels.get(0).unwrap())
                        .ok_or_else(|| "Unknown label".to_string())
                )
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Branch,
                args: Some(args),
                funcs: _,
                labels: Some(labels),
            }) => {
                if args.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }

                let arg1 = err!(
                    func,
                    index,
                    env.get(args.get(0).unwrap()).and_then(expect_bool)
                );

                if labels.len() != 2 {
                    err!(func, index, Err("Unexpected number of labels".to_string()))
                }

                index = *err!(
                    func,
                    index,
                    label_map
                        .get(if arg1 {
                            labels.get(0).unwrap()
                        } else {
                            labels.get(1).unwrap()
                        })
                        .ok_or_else(|| "Unknown label".to_string())
                )
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Return,
                args,
                funcs: _,
                labels: _,
            }) => match args {
                None => {
                    if func.return_type.is_none() {
                        return Ok((state, None));
                    } else {
                        err!(
                            func,
                            index,
                            Err("Expected return argument and found none".to_string())
                        )
                    }
                }
                Some(args) => {
                    if args.len() != 1 {
                        err!(
                            func,
                            index,
                            Err("Unexpected number of arguments".to_string())
                        )
                    } else if func.return_type.is_none() {
                        err!(
                            func,
                            index,
                            Err("Found return argument and expected none".to_string())
                        )
                    } else {
                        let ret = err!(func, index, env.get(args.get(0).unwrap()));
                        if type_check_val(ret, func.return_type.as_ref().unwrap()) {
                            return Ok((state, Some(ret.clone())));
                        } else {
                            err!(func, index, Err("Type Error".to_string()))
                        }
                    }
                }
            },
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Nop,
                args: _,
                funcs: _,
                labels: _,
            }) => index += 1,
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Print,
                args,
                funcs: _,
                labels: _,
            }) => {
                let vals: String = err!(
                    func,
                    index,
                    args.as_ref()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|n| env.get(n).map(|v| v.to_string()))
                        .collect::<Result<String, String>>()
                );
                println!("{}", vals);
                index += 1
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Store,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 2 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_ptr)
                );

                let arg2 = err!(func, index, env.get(vec.get(1).unwrap()));

                let typ = match arg1.ptr_type {
                    Type::Bool | Type::Float | Type::Int => unreachable!(),
                    Type::Pointer(ref ptr) => ptr.clone(),
                };

                if !type_check_val(arg2, &typ) {
                    err!(func, index, Err("Type Error".to_string()))
                }

                err!(
                    func,
                    index,
                    state.heap.borrow_mut().write(&arg1, arg2.clone())
                );

                index += 1
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Free,
                args: Some(vec),
                funcs: _,
                labels: _,
            }) => {
                if vec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }
                let arg1 = err!(
                    func,
                    index,
                    env.get(vec.get(0).unwrap()).and_then(expect_ptr)
                );

                err!(func, index, state.heap.borrow_mut().free(arg1));
                index += 1
            }
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Call,
                args,
                funcs: Some(fvec),
                labels: _,
            }) => {
                if fvec.len() != 1 {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of functions".to_string())
                    )
                }
                let (callee_f, callee_map) = err!(
                    func,
                    index,
                    funcs
                        .get(fvec.get(0).unwrap())
                        .ok_or(format!("no function of name {} found", func.name))
                );

                let next_state = State::new_state(&state);
                let mut next_env = Environment::default();

                if args.is_none() && callee_f.args.is_none() {
                    // do nothing because we have not args to add to the environment
                } else if args.is_some() && callee_f.args.is_some() {
                    let args_vec = args.as_ref().unwrap();
                    let callee_vec = callee_f.args.as_ref().unwrap();
                    if args_vec.len() != callee_vec.len() {
                        err!(
                            func,
                            index,
                            Err("Unexpected number of arguments".to_string())
                        )
                    }
                    err!(
                        func,
                        index,
                        args_vec
                            .iter()
                            .zip(callee_vec.iter())
                            .map(|(arg_name, expected_arg)| {
                                let arg = env.get(arg_name)?;
                                if !type_check_val(&arg, &expected_arg.arg_type) {
                                    return Err("Type error".to_string());
                                }
                                Ok((expected_arg.name.as_ref(), arg.clone()))
                            })
                            .try_for_each(|res| res.map(|(name, val)| next_env.set(name, val)))
                    );
                } else {
                    err!(
                        func,
                        index,
                        Err("Unexpected number of arguments".to_string())
                    )
                }

                if callee_f.return_type.is_some() {
                    err!(
                        func,
                        index,
                        Err("Found a return type and expected none".to_string())
                    )
                }

                let (next_state, _) = execute(&callee_f, callee_map, next_state, next_env, funcs)?;

                state.instruction_count = next_state.instruction_count;

                index += 1
            }
            Code::Instruction(Instruction::Value {
                op:
                    ValueOps::Add
                    | ValueOps::Sub
                    | ValueOps::Mul
                    | ValueOps::Div
                    | ValueOps::Eq
                    | ValueOps::Lt
                    | ValueOps::Gt
                    | ValueOps::Le
                    | ValueOps::Ge
                    | ValueOps::Fadd
                    | ValueOps::Fsub
                    | ValueOps::Fmul
                    | ValueOps::Fdiv
                    | ValueOps::Feq
                    | ValueOps::Flt
                    | ValueOps::Fgt
                    | ValueOps::Fle
                    | ValueOps::Fge
                    | ValueOps::Not
                    | ValueOps::And
                    | ValueOps::Or
                    | ValueOps::Call
                    | ValueOps::Alloc
                    | ValueOps::Load
                    | ValueOps::Id
                    | ValueOps::Phi
                    | ValueOps::PtrAdd,
                dest: _,
                op_type: _,
                args: _,
                funcs: _,
                labels: _,
            }) => {
                err!(func, index, Err("Invalid instruction".to_string()))
            }
            Code::Instruction(Instruction::Effect {
                op:
                    EffectOps::Jump
                    | EffectOps::Branch
                    | EffectOps::Call
                    | EffectOps::Store
                    | EffectOps::Free,
                args: _,
                funcs: _,
                labels: _,
            }) => {
                err!(func, index, Err("Invalid instruction".to_string()))
            }
        }
    }

    if func.return_type != None {
        return Err("Function has a return type but no value returned".to_string());
    }

    Ok((state, None))
}

fn parse_args<'a>(
    mut env: Environment<'a>,
    args: Option<&'a Vec<Argument>>,
    inputs: Vec<&str>,
) -> Result<Environment<'a>, String> {
    match args {
        None => {
            if inputs.is_empty() {
                Ok(env)
            } else {
                Err("Received arguments but expected none".to_string())
            }
        }
        Some(a) => {
            if inputs.len() != a.len() {
                return Err("Incorrect number of inputs".to_string());
            }

            a.iter()
                .enumerate()
                .try_for_each(|(index, arg)| match arg.arg_type {
                    Type::Bool => {
                        match inputs.get(index).unwrap().parse::<bool>() {
                            Err(_) => return Err(format!("Type error on argument {}", index)),
                            Ok(b) => env.set(&arg.name, Value::Lit(Literal::Bool(b))),
                        };
                        Ok(())
                    }
                    Type::Int => {
                        match inputs.get(index).unwrap().parse::<i64>() {
                            Err(_) => return Err(format!("Type error on argument {}", index)),
                            Ok(i) => env.set(&arg.name, Value::Lit(Literal::Int(i))),
                        };
                        Ok(())
                    }
                    Type::Float => {
                        match inputs.get(index).unwrap().parse::<f64>() {
                            Err(_) => return Err(format!("Type error on argument {}", index)),
                            Ok(f) => env.set(&arg.name, Value::Lit(Literal::Float(f))),
                        };
                        Ok(())
                    }
                    Type::Pointer(_) => Err("Can't pass a pointer as input".to_string()),
                })?;
            Ok(env)
        }
    }
}

pub fn eval_program(prog: Program, prof: bool, other_args: Vec<&str>) -> Result<(), String> {
    let funcs = {
        let len = prog.functions.len();
        let f: FxHashMap<String, (Function, FxHashMap<String, usize>)> = prog
            .functions
            .into_iter()
            .map(|x| {
                let label_map: FxHashMap<String, usize> = x
                    .instrs
                    .iter()
                    .enumerate()
                    .filter_map(|(index, code)| match code {
                        Code::Label { label } => Some((label.to_string(), index as usize)),
                        _ => None,
                    })
                    .collect();

                (x.name.clone(), (x, label_map))
            })
            .collect();
        if f.len() != len {
            return Err("multiple definitions of functions found".to_string());
        }
        f
    };

    let state = State::default();
    // find main function
    if !funcs.contains_key("main") {
        return Err("No main function to run".to_string());
    }

    let env = Environment::default();

    let (f, label_map) = funcs
        .get("main")
        .ok_or(format!("no function of name {} found", "main"))?;

    let args = f.args.as_ref();

    let env = parse_args(env, args, other_args)?;

    // execute
    let (state, _) = execute(f, label_map, state, env, &funcs)?;

    // Check that the heap is empty
    if !state.heap.borrow().is_empty() {
        return Err("Some memory locations have not been freed by end of execution.".to_string());
    }

    if prof {
        eprintln!("total_dyn_inst: {}", state.instruction_count);
    }

    Ok(())
}
