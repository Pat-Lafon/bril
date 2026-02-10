use crate::basic_block::{BBFunction, BBProgram};
use crate::error::{InterpError, PositionalInterpError};
use crate::ir;
use crate::ir::{FlatIR, LabelIndex, VarIndex};
use bril2json::escape_control_chars;

use fxhash::FxHashMap;

use std::cmp::max;
use std::collections::HashMap;
use std::fmt;

// The Environment is the data structure used to represent the stack of the program.
// The values of all variables are store here. Each variable is represented as a number so
// each value can be store at the index of that number.
// Each function call gets allocated a "frame" which is just the offset that each variable
// should be index from for the duration of that call.
//  Call "main" pointer(frame size 3)
//  |
//  |        Call "foo" pointer(frame size 2)
//  |        |
// [a, b, c, a, b]
struct Environment {
  // Pointer into `env` for the start of the current frame
  current_pointer: usize,
  // Size of the current frame
  current_frame_size: usize,
  // Maximum frame size across all functions - ensures tail calls always fit
  max_frame_size: usize,
  // A list of all stack pointers and frame sizes for previous frames
  stack_pointers: Vec<(usize, usize)>,
  // `env` is used like a stack. Assume it only grows
  env: Vec<Value>,
}

impl Environment {
  pub fn new(initial_frame_size: usize, max_frame_size: usize) -> Self {
    Self {
      current_pointer: 0,
      current_frame_size: initial_frame_size,
      max_frame_size,
      stack_pointers: Vec::with_capacity(50),
      // Allocate enough for max_frame_size so tail calls always fit
      env: vec![Value::default(); max(max_frame_size, 50)],
    }
  }

  pub fn get(&self, ident: VarIndex) -> &Value {
    // A Bril program is well formed when, dynamically, every variable is defined before its use.
    // If this is violated, this will return Value::Uninitialized and the whole interpreter will come crashing down.
    self.env.get(self.current_pointer + ident).unwrap()
  }

  // Used for getting arguments that should be passed to the current frame from the previous one
  pub fn get_from_last_frame(&self, ident: VarIndex) -> &Value {
    let past_pointer = self.stack_pointers.last().unwrap().0;
    self.env.get(past_pointer + ident).unwrap()
  }

  pub fn set(&mut self, ident: VarIndex, val: Value) {
    self.env[self.current_pointer + ident] = val;
  }

  // Push a new frame onto the stack
  pub fn push_frame(&mut self, size: usize) {
    self
      .stack_pointers
      .push((self.current_pointer, self.current_frame_size));
    self.current_pointer += self.current_frame_size;
    self.current_frame_size = size;

    // Ensure at least max_frame_size is available (so tail calls to any function fit)
    if self.current_pointer + self.max_frame_size > self.env.len() {
      self.env.resize(self.env.len() * 4, Value::default());
    }
  }

  // Remove a frame from the stack
  pub fn pop_frame(&mut self) {
    (self.current_pointer, self.current_frame_size) = self.stack_pointers.pop().unwrap();
  }

  // Update frame size for tail calls (callee may have different size than caller)
  pub const fn set_frame_size(&mut self, size: usize) {
    self.current_frame_size = size;
  }
}

// todo: This is basically a copy of the heap implement in `brili` and we could probably do something smarter. This currently isn't that worth it to optimize because most benchmarks do not use the memory extension nor do they run for very long. You (the reader in the future) may be working with Bril programs that you would like to speed up that extensively use the Bril memory extension. In that case, it would be worth seeing how to implement Heap without a map based memory. Maybe try to re-implement malloc for a large Vec<Value>?
struct Heap {
  memory: FxHashMap<usize, Vec<Value>>,
  base_num_counter: usize,
}

impl Default for Heap {
  fn default() -> Self {
    Self {
      memory: FxHashMap::with_capacity_and_hasher(20, fxhash::FxBuildHasher::default()),
      base_num_counter: 0,
    }
  }
}

impl Heap {
  fn is_empty(&self) -> bool {
    self.memory.is_empty()
  }

  fn alloc(&mut self, amount: i64) -> Result<Value, InterpError> {
    let amount: usize = amount
      .try_into()
      .map_err(|_| InterpError::CannotAllocSize(amount))?;
    let base = self.base_num_counter;
    self.base_num_counter += 1;
    self.memory.insert(base, vec![Value::default(); amount]);
    Ok(Value::Pointer(Pointer { base, offset: 0 }))
  }

  fn free(&mut self, key: &Pointer) -> Result<(), InterpError> {
    if self.memory.remove(&key.base).is_some() && key.offset == 0 {
      Ok(())
    } else {
      Err(InterpError::IllegalFree(key.base, key.offset))
    }
  }

  fn write(&mut self, key: &Pointer, val: Value) -> Result<(), InterpError> {
    // Will check that key.offset is >=0
    let offset: usize = key
      .offset
      .try_into()
      .map_err(|_| InterpError::InvalidMemoryAccess(key.base, key.offset))?;
    match self.memory.get_mut(&key.base) {
      Some(vec) if vec.len() > offset => {
        vec[offset] = val;
        Ok(())
      }
      Some(_) | None => Err(InterpError::InvalidMemoryAccess(key.base, key.offset)),
    }
  }

  fn read(&self, key: &Pointer) -> Result<&Value, InterpError> {
    // Will check that key.offset is >=0
    let offset: usize = key
      .offset
      .try_into()
      .map_err(|_| InterpError::InvalidMemoryAccess(key.base, key.offset))?;
    self
      .memory
      .get(&key.base)
      .and_then(|vec| vec.get(offset))
      .ok_or(InterpError::InvalidMemoryAccess(key.base, key.offset))
      .and_then(|val| match val {
        Value::Uninitialized => Err(InterpError::UsingUninitializedMemory),
        _ => Ok(val),
      })
  }
}

// A getter function for when you know what constructor of the Value enum you have and
// you just want the underlying value(like a f64).
// Or can just be used to get an owned version of the Value
fn get_arg<'a, T: From<&'a Value>>(vars: &'a Environment, index: VarIndex) -> T {
  T::from(vars.get(index))
}

#[derive(Debug, Default, Clone, Copy)]
enum Value {
  Int(i64),
  Bool(bool),
  Float(f64),
  Char(char),
  Pointer(Pointer),
  #[default]
  Uninitialized,
}

#[derive(Debug, Clone, PartialEq, Copy)]
struct Pointer {
  base: usize,
  offset: i64,
}

impl Pointer {
  const fn add(&self, offset: i64) -> Self {
    Self {
      base: self.base,
      offset: self.offset + offset,
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Int(i) => write!(f, "{i}"),
      Self::Bool(b) => write!(f, "{b}"),
      Self::Float(v) if v.is_infinite() && v.is_sign_positive() => write!(f, "Infinity"),
      Self::Float(v) if v.is_infinite() && v.is_sign_negative() => write!(f, "-Infinity"),
      Self::Float(v) if v != &0.0 && v.abs().log10() >= 10.0 => {
        f.write_str(format!("{v:.17e}").replace('e', "e+").as_str())
      }
      Self::Float(v) if v != &0.0 && v.abs().log10() <= -10.0 => write!(f, "{v:.17e}"),
      Self::Float(v) => write!(f, "{v:.17}"),
      Self::Char(c) => write!(f, "{c}"),
      Self::Pointer(p) => write!(f, "{p:?}"),
      Self::Uninitialized => unreachable!(),
    }
  }
}

fn optimized_val_output<T: std::io::Write>(out: &mut T, val: &Value) -> Result<(), std::io::Error> {
  match val {
    Value::Int(i) => out.write_all(itoa::Buffer::new().format(*i).as_bytes()),
    Value::Bool(b) => out.write_all(if *b { b"true" } else { b"false" }),
    Value::Float(f) if f.is_infinite() && f.is_sign_positive() => out.write_all(b"Infinity"),
    Value::Float(f) if f.is_infinite() && f.is_sign_negative() => out.write_all(b"-Infinity"),
    Value::Float(f) if f.is_nan() => out.write_all(b"NaN"),
    Value::Float(f) if f != &0.0 && f.abs().log10() >= 10.0 => {
      out.write_all(format!("{f:.17e}").replace('e', "e+").as_bytes())
    }
    Value::Float(f) if f != &0.0 && f.abs().log10() <= -10.0 => {
      out.write_all(format!("{f:.17e}").as_bytes())
    }
    Value::Float(f) => out.write_all(format!("{f:.17}").as_bytes()),
    Value::Char(c) => {
      let buf = &mut [0_u8; 2];
      out.write_all(c.encode_utf8(buf).as_bytes())
    }
    Value::Pointer(p) => out.write_all(format!("{p:?}").as_bytes()),
    Value::Uninitialized => unreachable!(),
  }
}

impl From<&bril_rs::Literal> for Value {
  fn from(l: &bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(*i),
      bril_rs::Literal::Bool(b) => Self::Bool(*b),
      bril_rs::Literal::Float(f) => Self::Float(*f),
      bril_rs::Literal::Char(c) => Self::Char(*c),
    }
  }
}

impl From<bril_rs::Literal> for Value {
  fn from(l: bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(i),
      bril_rs::Literal::Bool(b) => Self::Bool(b),
      bril_rs::Literal::Float(f) => Self::Float(f),
      bril_rs::Literal::Char(c) => Self::Char(c),
    }
  }
}

impl From<&Value> for i64 {
  fn from(value: &Value) -> Self {
    if let Value::Int(i) = value {
      *i
    } else {
      unreachable!()
    }
  }
}

impl From<&Value> for bool {
  fn from(value: &Value) -> Self {
    if let Value::Bool(b) = value {
      *b
    } else {
      unreachable!()
    }
  }
}

impl From<&Value> for f64 {
  fn from(value: &Value) -> Self {
    if let Value::Float(f) = value {
      *f
    } else {
      unreachable!()
    }
  }
}

impl From<&Value> for char {
  fn from(value: &Value) -> Self {
    if let Value::Char(c) = value {
      *c
    } else {
      unreachable!()
    }
  }
}

impl<'a> From<&'a Value> for &'a Pointer {
  fn from(value: &'a Value) -> Self {
    if let Value::Pointer(p) = value {
      p
    } else {
      unreachable!()
    }
  }
}

impl From<&Self> for Value {
  fn from(value: &Self) -> Self {
    *value
  }
}

// Sets up the Environment for the next function call with the supplied arguments
fn make_func_args(callee_func: &BBFunction, args: &[VarIndex], vars: &mut Environment) {
  vars.push_frame(callee_func.num_of_vars);

  args
    .iter()
    .zip(callee_func.args_as_nums.iter())
    .for_each(|(arg_name, expected_arg)| {
      let arg = vars.get_from_last_frame(*arg_name);
      vars.set(*expected_arg, *arg);
    });
}

// Sets up arguments for a tail call by reusing the current frame
fn make_tail_call_args(
  callee_func: &BBFunction,
  args: &[VarIndex],
  vars: &mut Environment,
  scratch: &mut Vec<Value>,
) {
  scratch.clear();
  scratch.extend(args.iter().map(|arg_name| *vars.get(*arg_name)));
  vars.set_frame_size(callee_func.num_of_vars);
  for (val, dest) in scratch.iter().zip(callee_func.args_as_nums.iter()) {
    vars.set(*dest, *val);
  }
}

/// Store a comparison result and return the branch target.
#[inline(always)]
fn cmp_branch(env: &mut Environment, cb: &ir::CmpBranch, cond: bool) -> LabelIndex {
  env.set(cb.dest, Value::Bool(cond));
  if cond { cb.true_dest } else { cb.false_dest }
}

fn execute<'a, T: std::io::Write>(
  state: &mut State<'a, T>,
  func: &'a BBFunction,
) -> Result<Option<Value>, PositionalInterpError> {
  let mut shadow_env = HashMap::new();
  let mut curr_block_idx = LabelIndex(0);

  loop {
    let curr_block = &func.blocks[curr_block_idx.0 as usize];
    let curr_instrs = &curr_block.flat_instrs;
    let mut jumped = if curr_block.exit.len() == 1 {
      curr_block_idx = curr_block.exit[0];
      true
    } else {
      false
    };

    state.instruction_count += curr_block.instruction_count;

    for (idx, code) in curr_instrs.iter().enumerate() {
      match code {
        FlatIR::Const { dest, value } => {
          state.env.set(*dest, Value::from(value));
        }
        FlatIR::Undef { dest } => state.env.set(*dest, Value::Uninitialized),
        FlatIR::Get { dest } => match shadow_env.get(dest) {
          Some(v) => state.env.set(*dest, *v),
          None => {
            return Err(InterpError::GetWithoutSet).map_err(|e| {
              Into::<InterpError>::into(e)
                .add_pos(curr_block.positions.get(idx).cloned().unwrap_or_default())
            });
          }
        },
        // Flattened unary operations
        FlatIR::Id(op) => {
          let src = get_arg::<Value>(&state.env, op.arg);
          state.env.set(op.dest, src);
        }
        FlatIR::Not(op) => {
          let a = get_arg::<bool>(&state.env, op.arg);
          state.env.set(op.dest, Value::Bool(!a));
        }
        FlatIR::Char2int(op) => {
          let a = get_arg::<char>(&state.env, op.arg);
          state.env.set(op.dest, Value::Int(u32::from(a).into()));
        }
        FlatIR::Int2char(op) => {
          let a = get_arg::<i64>(&state.env, op.arg);
          let c = u32::try_from(a)
            .ok()
            .and_then(char::from_u32)
            .ok_or(InterpError::ToCharError(a))
            .map_err(|e| e.add_pos(curr_block.positions.get(idx).cloned().unwrap_or_default()))?;
          state.env.set(op.dest, Value::Char(c));
        }
        FlatIR::Alloc(op) => {
          let a = get_arg::<i64>(&state.env, op.arg);
          let res = state.heap.alloc(a).map_err(|e| {
            e.add_pos(curr_block.positions.get(idx).cloned().unwrap_or_default())
          })?;
          state.env.set(op.dest, res);
        }
        FlatIR::Load(op) => {
          let a = get_arg::<&Pointer>(&state.env, op.arg);
          let res = state.heap.read(a).map_err(|e| {
            e.add_pos(curr_block.positions.get(idx).cloned().unwrap_or_default())
          })?;
          state.env.set(op.dest, *res);
        }
        FlatIR::Float2Bits(op) => {
          let float = get_arg::<f64>(&state.env, op.arg);
          let int = i64::from_ne_bytes(float.to_ne_bytes());
          state.env.set(op.dest, Value::Int(int));
        }
        FlatIR::Bits2Float(op) => {
          let int = get_arg::<i64>(&state.env, op.arg);
          let float = f64::from_ne_bytes(int.to_ne_bytes());
          state.env.set(op.dest, Value::Float(float));
        }
        // Flattened binary operations
        FlatIR::Add(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Int(a0.wrapping_add(a1)));
        }
        FlatIR::Sub(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Int(a0.wrapping_sub(a1)));
        }
        FlatIR::Mul(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Int(a0.wrapping_mul(a1)));
        }
        FlatIR::Div(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          if a1 == 0 {
            return Err(InterpError::DivisionByZero.add_pos(
              curr_block.positions.get(idx).cloned().unwrap_or_default(),
            ));
          }
          state.env.set(op.dest, Value::Int(a0.wrapping_div(a1)));
        }
        FlatIR::Eq(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 == a1));
        }
        FlatIR::Lt(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 < a1));
        }
        FlatIR::Gt(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 > a1));
        }
        FlatIR::Le(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 <= a1));
        }
        FlatIR::Ge(op) => {
          let a0 = get_arg::<i64>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 >= a1));
        }
        FlatIR::And(op) => {
          let a0 = get_arg::<bool>(&state.env, op.arg0);
          let a1 = get_arg::<bool>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 && a1));
        }
        FlatIR::Or(op) => {
          let a0 = get_arg::<bool>(&state.env, op.arg0);
          let a1 = get_arg::<bool>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 || a1));
        }
        FlatIR::Fadd(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Float(a0 + a1));
        }
        FlatIR::Fsub(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Float(a0 - a1));
        }
        FlatIR::Fmul(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Float(a0 * a1));
        }
        FlatIR::Fdiv(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Float(a0 / a1));
        }
        FlatIR::Feq(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 == a1));
        }
        FlatIR::Flt(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 < a1));
        }
        FlatIR::Fgt(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 > a1));
        }
        FlatIR::Fle(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 <= a1));
        }
        FlatIR::Fge(op) => {
          let a0 = get_arg::<f64>(&state.env, op.arg0);
          let a1 = get_arg::<f64>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 >= a1));
        }
        FlatIR::Ceq(op) => {
          let a0 = get_arg::<char>(&state.env, op.arg0);
          let a1 = get_arg::<char>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 == a1));
        }
        FlatIR::Clt(op) => {
          let a0 = get_arg::<char>(&state.env, op.arg0);
          let a1 = get_arg::<char>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 < a1));
        }
        FlatIR::Cgt(op) => {
          let a0 = get_arg::<char>(&state.env, op.arg0);
          let a1 = get_arg::<char>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 > a1));
        }
        FlatIR::Cle(op) => {
          let a0 = get_arg::<char>(&state.env, op.arg0);
          let a1 = get_arg::<char>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 <= a1));
        }
        FlatIR::Cge(op) => {
          let a0 = get_arg::<char>(&state.env, op.arg0);
          let a1 = get_arg::<char>(&state.env, op.arg1);
          state.env.set(op.dest, Value::Bool(a0 >= a1));
        }
        FlatIR::PtrAdd(op) => {
          let a0 = get_arg::<&Pointer>(&state.env, op.arg0);
          let a1 = get_arg::<i64>(&state.env, op.arg1);
          let res = Value::Pointer(a0.add(a1));
          state.env.set(op.dest, res);
        }
        FlatIR::MultiArityCall { func, dest, args } => {
          let callee_func = state.prog.get(*func).unwrap();

          make_func_args(callee_func, args, &mut state.env);

          let result = execute(state, callee_func)?.unwrap();

          state.env.pop_frame();

          state.env.set(*dest, result);
        }
        FlatIR::TailCall { func, args } => {
          let callee_func = state.prog.get(*func).unwrap();
          make_tail_call_args(callee_func, args, &mut state.env, &mut state.arg_scratch);
          return execute(state, callee_func);
        }
        FlatIR::Nop => {}
        FlatIR::Jump { dest } => {
          curr_block_idx = *dest;
          jumped = true;
        }
        FlatIR::Branch {
          arg,
          true_dest,
          false_dest,
        } => {
          let cond = get_arg::<bool>(&state.env, *arg);
          curr_block_idx = if cond { *true_dest } else { *false_dest };
          jumped = true;
        }
        // Fused compare-and-branch: integer
        FlatIR::EqBranch(cb) => {
          let cond = get_arg::<i64>(&state.env, cb.arg0) == get_arg::<i64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::LtBranch(cb) => {
          let cond = get_arg::<i64>(&state.env, cb.arg0) < get_arg::<i64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::GtBranch(cb) => {
          let cond = get_arg::<i64>(&state.env, cb.arg0) > get_arg::<i64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::LeBranch(cb) => {
          let cond = get_arg::<i64>(&state.env, cb.arg0) <= get_arg::<i64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::GeBranch(cb) => {
          let cond = get_arg::<i64>(&state.env, cb.arg0) >= get_arg::<i64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        // Fused compare-and-branch: float
        FlatIR::FeqBranch(cb) => {
          let cond = get_arg::<f64>(&state.env, cb.arg0) == get_arg::<f64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::FltBranch(cb) => {
          let cond = get_arg::<f64>(&state.env, cb.arg0) < get_arg::<f64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::FgtBranch(cb) => {
          let cond = get_arg::<f64>(&state.env, cb.arg0) > get_arg::<f64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::FleBranch(cb) => {
          let cond = get_arg::<f64>(&state.env, cb.arg0) <= get_arg::<f64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::FgeBranch(cb) => {
          let cond = get_arg::<f64>(&state.env, cb.arg0) >= get_arg::<f64>(&state.env, cb.arg1);
          curr_block_idx = cmp_branch(&mut state.env, cb, cond);
          jumped = true;
        }
        FlatIR::ReturnValue { arg } => {
          let res = get_arg::<Value>(&state.env, *arg);
          return Ok(Some(res));
        }
        FlatIR::ReturnVoid => {
          return Ok(None);
        }
        FlatIR::EffectfulCall { func, args } => {
          let callee_func = state.prog.get(*func).unwrap();

          make_func_args(callee_func, args, &mut state.env);

          execute(state, callee_func)?;
          state.env.pop_frame();
        }
        FlatIR::TailCallVoid { func, args } => {
          let callee_func = state.prog.get(*func).unwrap();
          make_tail_call_args(callee_func, args, &mut state.env, &mut state.arg_scratch);
          return execute(state, callee_func);
        }
        FlatIR::PrintOne { arg } => {
          optimized_val_output(&mut state.out, state.env.get(*arg))
            .and_then(|()| // Add new line
            state.out.write_all(b"\n"))
            .map_err(|e| {
              Into::<InterpError>::into(e)
                .add_pos(curr_block.positions.get(idx).cloned().unwrap_or_default())
            })?;
        }
        FlatIR::PrintMultiple { args } => {
          writeln!(
            state.out,
            "{}",
            args
              .iter()
              .map(|a| state.env.get(*a).to_string())
              .collect::<Vec<String>>()
              .join(" ")
          )
          .map_err(|e| {
            Into::<InterpError>::into(e)
              .add_pos(curr_block.positions.get(idx).cloned().unwrap_or_default())
          })?;
        }
        FlatIR::Store { arg0, arg1 } => {
          let key = get_arg::<&Pointer>(&state.env, *arg0);
          let val = get_arg::<Value>(&state.env, *arg1);
          state.heap.write(key, val)?;
        }
        FlatIR::Set { arg0, arg1 } => {
          let val = get_arg::<Value>(&state.env, *arg1);
          shadow_env.insert(*arg0, val);
        }
        FlatIR::Free { arg } => {
          let ptr = get_arg::<&Pointer>(&state.env, *arg);
          state.heap.free(ptr)?;
        }
      }
    }

    if !jumped {
      if let Some(ty) = &func.return_type {
        return Err(
          InterpError::NonVoidFuncNoRet(ty.clone()).add_pos(if curr_instrs.is_empty() {
            // Ideally we use the last instruction in the block before the fatal
            // (implicit) return... but if that block is empty, we should just
            // point to the function itself
            func.pos.clone()
          } else {
            curr_block
              .positions
              .get(curr_instrs.len() - 1)
              .cloned()
              .unwrap_or_default()
          }),
        );
      }
      return Ok(None);
    }
  }
}

fn parse_args(
  mut env: Environment,
  args: &[bril_rs::Argument],
  args_as_nums: &[VarIndex],
  inputs: &[String],
) -> Result<Environment, InterpError> {
  if args.is_empty() && inputs.is_empty() {
    Ok(env)
  } else if inputs.len() != args.len() {
    Err(InterpError::BadNumFuncArgs(args.len(), inputs.len()))
  } else {
    args
      .iter()
      .zip(args_as_nums.iter())
      .enumerate()
      .try_for_each(|(index, (arg, arg_as_num))| match arg.arg_type {
        bril_rs::Type::Bool => {
          match inputs.get(index).unwrap().parse::<bool>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Bool,
                inputs.get(index).unwrap().clone(),
              ));
            }
            Ok(b) => env.set(*arg_as_num, Value::Bool(b)),
          }
          Ok(())
        }
        bril_rs::Type::Int => {
          match inputs.get(index).unwrap().parse::<i64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Int,
                inputs.get(index).unwrap().clone(),
              ));
            }
            Ok(i) => env.set(*arg_as_num, Value::Int(i)),
          }
          Ok(())
        }
        bril_rs::Type::Float => {
          match inputs.get(index).unwrap().parse::<f64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Float,
                inputs.get(index).unwrap().clone(),
              ));
            }
            Ok(f) => env.set(*arg_as_num, Value::Float(f)),
          }
          Ok(())
        }
        bril_rs::Type::Char => escape_control_chars(inputs.get(index).unwrap().as_ref())
          .map_or_else(
            || Err(InterpError::NotOneChar),
            |c| {
              env.set(*arg_as_num, Value::Char(c));
              Ok(())
            },
          ),
        bril_rs::Type::Pointer(..) | bril_rs::Type::Any => unreachable!(),
      })?;
    Ok(env)
  }
}

// State captures the parts of the interpreter that are used across function boundaries
struct State<'a, T: std::io::Write> {
  prog: &'a BBProgram,
  env: Environment,
  heap: Heap,
  out: T,
  instruction_count: usize,
  arg_scratch: Vec<Value>,
}

impl<'a, T: std::io::Write> State<'a, T> {
  fn new(prog: &'a BBProgram, env: Environment, heap: Heap, out: T) -> Self {
    Self {
      prog,
      env,
      heap,
      out,
      instruction_count: 0,
      arg_scratch: Vec::with_capacity(prog.max_frame_size),
    }
  }
}

/// The entrance point to the interpreter.
///
/// It runs over a ```prog```:[`BBProgram`] starting at the "main" function with ```input_args``` as input. Print statements output to ```out``` which implements [`std::io::Write`]. You also need to include whether you want the interpreter to count the number of instructions run with ```profiling```. This information is outputted to [`std::io::stderr`].
/// # Panics
/// This should not panic with normal use except if there is a bug or if you are using an unimplemented feature
/// # Errors
/// Will error on malformed `BBProgram`, like if the original Bril program was not well-formed
pub fn execute_main<T: std::io::Write, U: std::io::Write>(
  prog: &BBProgram,
  out: T,
  input_args: &[String],
  profiling: bool,
  mut profiling_out: U,
) -> Result<(), PositionalInterpError> {
  let main_func = prog
    .index_of_main
    .map(|i| prog.get(i).unwrap())
    .ok_or(InterpError::NoMainFunction)?;

  let mut env = Environment::new(main_func.num_of_vars, prog.max_frame_size);
  let heap = Heap::default();

  env = parse_args(env, &main_func.args, &main_func.args_as_nums, input_args)
    .map_err(|e| e.add_pos(main_func.pos.clone()))?;

  let mut state = State::new(prog, env, heap, out);

  execute(&mut state, main_func)?;

  if !state.heap.is_empty() {
    return Err(InterpError::MemLeak).map_err(|e| e.add_pos(main_func.pos.clone()));
  }

  state.out.flush().map_err(InterpError::IoError)?;

  if profiling {
    writeln!(profiling_out, "total_dyn_inst: {}", state.instruction_count)
      // We call flush here in case `profiling_out` is a <https://doc.rust-lang.org/std/io/struct.BufWriter.html>
      // Otherwise we would expect this flush to be a no-op.
      .and_then(|()| profiling_out.flush())
      .map_err(InterpError::IoError)?;
  }

  Ok(())
}
