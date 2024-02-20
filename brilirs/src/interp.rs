use crate::basic_block::{BBFunction, BBProgram, BasicBlock};
use crate::error::{InterpError, PositionalInterpError};
use bril2json::escape_control_chars;
use bril_rs::Instruction;

use crate::allocator::{optimized_val_output, BrilAllocator, BrilPointer, Value};
use crate::basic_heap::{BasicHeap};

use std::cmp::max;

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
struct Environment<P: BrilPointer> {
  // Pointer into env for the start of the current frame
  current_pointer: usize,
  // Size of the current frame
  current_frame_size: usize,
  // A list of all stack pointers for valid frames on the stack
  stack_pointers: Vec<(usize, usize)>,
  // env is used like a stack. Assume it only grows
  env: Vec<Value<P>>,
}

impl<P: BrilPointer> Environment<P> {
  pub fn new(size: usize) -> Self {
    Self {
      current_pointer: 0,
      current_frame_size: size,
      stack_pointers: Vec::new(),
      // Allocate a larger stack size so the interpreter needs to allocate less often
      env: vec![Value::default(); max(size, 50)],
    }
  }

  pub fn get(&self, ident: usize) -> &Value<P> {
    // A bril program is well formed when, dynamically, every variable is defined before its use.
    // If this is violated, this will return Value::Uninitialized and the whole interpreter will come crashing down.
    self.env.get(self.current_pointer + ident).unwrap()
  }

  // Used for getting arguments that should be passed to the current frame from the previous one
  pub fn get_from_last_frame(&self, ident: usize) -> &Value<P> {
    let past_pointer = self.stack_pointers.last().unwrap().0;
    self.env.get(past_pointer + ident).unwrap()
  }

  pub fn set(&mut self, ident: usize, val: Value<P>) {
    self.env[self.current_pointer + ident] = val;
  }
  // Push a new frame onto the stack
  pub fn push_frame(&mut self, size: usize) {
    self
      .stack_pointers
      .push((self.current_pointer, self.current_frame_size));
    self.current_pointer += self.current_frame_size;
    self.current_frame_size = size;

    // Check that the stack is large enough
    if self.current_pointer + self.current_frame_size > self.env.len() {
      // We need to allocate more stack
      self.env.resize(
        max(
          self.env.len() * 4,
          self.current_pointer + self.current_frame_size,
        ),
        Value::default(),
      );
    }
  }

  // Remove a frame from the stack
  pub fn pop_frame(&mut self) {
    (self.current_pointer, self.current_frame_size) = self.stack_pointers.pop().unwrap();
  }
}

// A getter function for when you know what constructor of the Value enum you have and
// you just want the underlying value(like a f64).
// Or can just be used to get a owned version of the Value
fn get_arg<'a, P: BrilPointer, T: From<&'a Value<P>>>(
  vars: &'a Environment<P>,
  index: usize,
  args: &[usize],
) -> T {
  T::from(vars.get(args[index]))
}

// Sets up the Environment for the next function call with the supplied arguments
fn make_func_args<P: BrilPointer>(
  callee_func: &BBFunction,
  args: &[usize],
  vars: &mut Environment<P>,
) {
  vars.push_frame(callee_func.num_of_vars);

  args
    .iter()
    .zip(callee_func.args_as_nums.iter())
    .for_each(|(arg_name, expected_arg)| {
      let arg = vars.get_from_last_frame(*arg_name);
      vars.set(*expected_arg, arg.clone());
    });
}

fn execute_value_op<T: std::io::Write, P: BrilPointer, H: BrilAllocator<P>>(
  state: &mut State<T, P, H>,
  op: bril_rs::ValueOps,
  dest: usize,
  args: &[usize],
  labels: &[String],
  funcs: &[usize],
  last_label: Option<&String>,
) -> Result<(), InterpError> {
  use bril_rs::ValueOps::{
    Add, Alloc, And, Call, Ceq, Cge, Cgt, Char2int, Cle, Clt, Div, Eq, Fadd, Fdiv, Feq, Fge, Fgt,
    Fle, Flt, Fmul, Fsub, Ge, Gt, Id, Int2char, Le, Load, Lt, Mul, Not, Or, Phi, PtrAdd, Sub,
  };
  match op {
    Add => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Int(arg0.wrapping_add(arg1)));
    }
    Mul => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Int(arg0.wrapping_mul(arg1)));
    }
    Sub => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Int(arg0.wrapping_sub(arg1)));
    }
    Div => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      if arg1 == 0 {
        return Err(InterpError::DivisionByZero);
      }
      state.env.set(dest, Value::Int(arg0.wrapping_div(arg1)));
    }
    Eq => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 == arg1));
    }
    Lt => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 < arg1));
    }
    Gt => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 > arg1));
    }
    Le => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 <= arg1));
    }
    Ge => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 >= arg1));
    }
    Not => {
      let arg0 = get_arg::<P, bool>(&state.env, 0, args);
      state.env.set(dest, Value::Bool(!arg0));
    }
    And => {
      let arg0 = get_arg::<P, bool>(&state.env, 0, args);
      let arg1 = get_arg::<P, bool>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 && arg1));
    }
    Or => {
      let arg0 = get_arg::<P, bool>(&state.env, 0, args);
      let arg1 = get_arg::<P, bool>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 || arg1));
    }
    Id => {
      let src = get_arg::<P, Value<P>>(&state.env, 0, args);
      state.env.set(dest, src);
    }
    Fadd => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 + arg1));
    }
    Fmul => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 * arg1));
    }
    Fsub => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 - arg1));
    }
    Fdiv => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 / arg1));
    }
    Feq => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 == arg1));
    }
    Flt => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 < arg1));
    }
    Fgt => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 > arg1));
    }
    Fle => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 <= arg1));
    }
    Fge => {
      let arg0 = get_arg::<P, f64>(&state.env, 0, args);
      let arg1 = get_arg::<P, f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 >= arg1));
    }
    Ceq => {
      let arg0 = get_arg::<P, char>(&state.env, 0, args);
      let arg1 = get_arg::<P, char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 == arg1));
    }
    Clt => {
      let arg0 = get_arg::<P, char>(&state.env, 0, args);
      let arg1 = get_arg::<P, char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 < arg1));
    }
    Cgt => {
      let arg0 = get_arg::<P, char>(&state.env, 0, args);
      let arg1 = get_arg::<P, char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 > arg1));
    }
    Cle => {
      let arg0 = get_arg::<P, char>(&state.env, 0, args);
      let arg1 = get_arg::<P, char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 <= arg1));
    }
    Cge => {
      let arg0 = get_arg::<P, char>(&state.env, 0, args);
      let arg1 = get_arg::<P, char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 >= arg1));
    }
    Char2int => {
      let arg0 = get_arg::<P, char>(&state.env, 0, args);
      state.env.set(dest, Value::Int(u32::from(arg0).into()));
    }
    Int2char => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);

      let arg0_char = u32::try_from(arg0)
        .ok()
        .and_then(char::from_u32)
        .ok_or(InterpError::ToCharError(arg0))?;

      state.env.set(dest, Value::Char(arg0_char));
    }
    Call => {
      let callee_func = state.prog.get(funcs[0]).unwrap();

      make_func_args(callee_func, args, &mut state.env);

      let result = execute(state, callee_func)?.unwrap();

      state.env.pop_frame();

      state.env.set(dest, result);
    }
    Phi => match last_label {
      None => return Err(InterpError::NoLastLabel),
      Some(last_label) => {
        let arg = labels
          .iter()
          .position(|l| l == last_label)
          .ok_or_else(|| InterpError::PhiMissingLabel(last_label.to_string()))
          .map(|i| get_arg::<P, Value<P>>(&state.env, i, args))?;
        state.env.set(dest, arg);
      }
    },
    Alloc => {
      let arg0 = get_arg::<P, i64>(&state.env, 0, args);
      let res = state.heap.alloc(arg0)?;
      state.env.set(dest, res);
    }
    Load => {
      let arg0 = get_arg::<P, P>(&state.env, 0, args);
      let res = state.heap.read(&arg0)?;
      state.env.set(dest, res.clone());
    }
    PtrAdd => {
      let arg0 = get_arg::<P, P>(&state.env, 0, args);
      let arg1 = get_arg::<P, i64>(&state.env, 1, args);
      let res = Value::Pointer(arg0.add(arg1));
      state.env.set(dest, res);
    }
  }
  Ok(())
}

fn execute_effect_op<T: std::io::Write, P: BrilPointer, H: BrilAllocator<P>>(
  state: &mut State<T, P, H>,
  op: bril_rs::EffectOps,
  args: &[usize],
  funcs: &[usize],
  curr_block: &BasicBlock,
  // There are two output variables where values are stored to effect the loop execution.
  next_block_idx: &mut Option<usize>,
  result: &mut Option<Value<P>>,
) -> Result<(), InterpError> {
  use bril_rs::EffectOps::{
    Branch, Call, Commit, Free, Guard, Jump, Nop, Print, Return, Speculate, Store,
  };
  match op {
    Jump => {
      *next_block_idx = Some(curr_block.exit[0]);
    }
    Branch => {
      let bool_arg0 = get_arg::<P, bool>(&state.env, 0, args);
      let exit_idx = usize::from(!bool_arg0);
      *next_block_idx = Some(curr_block.exit[exit_idx]);
    }
    Return => {
      if !args.is_empty() {
        *result = Some(get_arg::<P, Value<P>>(&state.env, 0, args));
      }
    }
    Print => {
      // In the typical case, users only print out one value at a time
      // So we can usually avoid extra allocations by providing that string directly
      if args.len() == 1 {
        optimized_val_output(&mut state.out, state.env.get(*args.first().unwrap()))?;
        // Add new line
        state.out.write_all(&[b'\n'])?;
      } else {
        writeln!(
          state.out,
          "{}",
          args
            .iter()
            .map(|a| state.env.get(*a).to_string())
            .collect::<Vec<String>>()
            .join(" ")
        )?;
      }
    }
    Nop => {}
    Call => {
      let callee_func = state.prog.get(funcs[0]).unwrap();

      make_func_args(callee_func, args, &mut state.env);

      execute(state, callee_func)?;
      state.env.pop_frame();
    }
    Store => {
      let arg0 = get_arg::<P, P>(&state.env, 0, args);
      let arg1 = get_arg::<P, Value<P>>(&state.env, 1, args);
      state.heap.write(&arg0, arg1)?;
    }
    Free => {
      let arg0 = get_arg::<P, P>(&state.env, 0, args);
      state.heap.free(&arg0)?;
    }
    Speculate | Commit | Guard => unimplemented!(),
  }
  Ok(())
}

fn execute<'a, T: std::io::Write, P: BrilPointer, H: BrilAllocator<P>>(
  state: &mut State<'a, T, P, H>,
  func: &'a BBFunction,
) -> Result<Option<Value<P>>, PositionalInterpError> {
  let mut last_label;
  let mut current_label = None;
  let mut curr_block_idx = 0;
  // A possible return value
  let mut result = None;

  loop {
    let curr_block = &func.blocks[curr_block_idx];
    let curr_instrs = &curr_block.instrs;
    let curr_numified_instrs = &curr_block.numified_instrs;
    // WARNING!!! We can add the # of instructions at once because you can only jump to a new block at the end. This may need to be changed if speculation is implemented
    state.instruction_count += curr_instrs.len();
    last_label = current_label;
    current_label = curr_block.label.as_ref();

    // A place to store the next block that will be jumped to if specified by an instruction
    let mut next_block_idx = None;

    for (code, numified_code) in curr_instrs.iter().zip(curr_numified_instrs.iter()) {
      match code {
        Instruction::Constant {
          op: bril_rs::ConstOps::Const,
          dest: _,
          const_type,
          value,
          pos: _,
        } => {
          // Integer literals can be promoted to Floating point
          if const_type == &bril_rs::Type::Float {
            match value {
              // So yes, as clippy points out, you technically lose precision here on the `*i as f64` cast. On the other hand, you already give up precision when you start using floats and I haven't been able to find a case where you are giving up precision in the cast that you don't already lose by using floating points.
              // So it's probably fine unless proven otherwise.
              #[allow(clippy::cast_precision_loss)]
              bril_rs::Literal::Int(i) => state
                .env
                .set(numified_code.dest.unwrap(), Value::Float(*i as f64)),
              bril_rs::Literal::Float(f) => {
                state.env.set(numified_code.dest.unwrap(), Value::Float(*f));
              }
              bril_rs::Literal::Char(_) | bril_rs::Literal::Bool(_) => unreachable!(),
            }
          } else {
            state
              .env
              .set(numified_code.dest.unwrap(), Value::from(value));
          };
        }
        Instruction::Value {
          op,
          dest: _,
          op_type: _,
          args: _,
          labels,
          funcs: _,
          pos,
        } => {
          execute_value_op(
            state,
            *op,
            numified_code.dest.unwrap(),
            &numified_code.args,
            labels,
            &numified_code.funcs,
            last_label,
          )
          .map_err(|e| e.add_pos(pos.clone()))?;
        }
        Instruction::Effect {
          op,
          args: _,
          labels: _,
          funcs: _,
          pos,
        } => {
          execute_effect_op(
            state,
            *op,
            &numified_code.args,
            &numified_code.funcs,
            curr_block,
            &mut next_block_idx,
            &mut result,
          )
          .map_err(|e| e.add_pos(pos.clone()))?;
        }
      }
    }

    // Are we jumping to a new block or are we done?
    if let Some(idx) = next_block_idx {
      curr_block_idx = idx;
    } else if curr_block.exit.len() == 1 {
      curr_block_idx = curr_block.exit[0];
    } else {
      return Ok(result);
    }
  }
}

fn parse_args<P: BrilPointer>(
  mut env: Environment<P>,
  args: &[bril_rs::Argument],
  args_as_nums: &[usize],
  inputs: &[String],
) -> Result<Environment<P>, InterpError> {
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
                (*inputs.get(index).unwrap()).to_string(),
              ))
            }
            Ok(b) => env.set(*arg_as_num, Value::Bool(b)),
          };
          Ok(())
        }
        bril_rs::Type::Int => {
          match inputs.get(index).unwrap().parse::<i64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Int,
                (*inputs.get(index).unwrap()).to_string(),
              ))
            }
            Ok(i) => env.set(*arg_as_num, Value::Int(i)),
          };
          Ok(())
        }
        bril_rs::Type::Float => {
          match inputs.get(index).unwrap().parse::<f64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Float,
                (*inputs.get(index).unwrap()).to_string(),
              ))
            }
            Ok(f) => env.set(*arg_as_num, Value::Float(f)),
          };
          Ok(())
        }
        bril_rs::Type::Pointer(..) => unreachable!(),
        bril_rs::Type::Char => escape_control_chars(inputs.get(index).unwrap().as_ref())
          .map_or_else(
            || Err(InterpError::NotOneChar),
            |c| {
              env.set(*arg_as_num, Value::Char(c));
              Ok(())
            },
          ),
      })?;
    Ok(env)
  }
}

// State captures the parts of the interpreter that are used across function boundaries
struct State<'a, T: std::io::Write, P: BrilPointer, H: BrilAllocator<P>> {
  prog: &'a BBProgram,
  env: Environment<P>,
  heap: H,
  out: T,
  instruction_count: usize,
}

impl<'a, T: std::io::Write, P: BrilPointer, H: BrilAllocator<P>> State<'a, T, P, H> {
  const fn new(prog: &'a BBProgram, env: Environment<P>, heap: H, out: T) -> Self {
    Self {
      prog,
      env,
      heap,
      out,
      instruction_count: 0,
    }
  }
}

/// The entrance point to the interpreter. It runs over a ```prog```:[`BBProgram`] starting at the "main" function with ```input_args``` as input. Print statements output to ```out``` which implements [`std::io::Write`]. You also need to include whether you want the interpreter to count the number of instructions run with ```profiling```. This information is outputted to [`std::io::stderr`]
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

  let mut env = Environment::new(main_func.num_of_vars);
  let heap = BasicHeap::default();

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
      // We call flush here in case `profiling_out` is a https://doc.rust-lang.org/std/io/struct.BufWriter.html
      // Otherwise we would expect this flush to be a nop.
      .and_then(|()| profiling_out.flush())
      .map_err(InterpError::IoError)?;
  }

  Ok(())
}
