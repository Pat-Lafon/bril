use crate::ir::{BinaryOp, CmpBranch, FlatIR, FuncIndex, LabelIndex, VarIndex, get_num_from_map};
use bril_rs::{Function, Position, Program};
use fxhash::FxHashMap;

use crate::error::{InterpError, PositionalInterpError};

/// A program represented as basic blocks. This is the IR of `brilirs`
#[derive(Debug)]
pub struct BBProgram {
  #[doc(hidden)]
  pub index_of_main: Option<FuncIndex>,
  #[doc(hidden)]
  pub func_index: Vec<BBFunction>,
  /// Maximum frame size across all functions - used for pre-allocation
  #[doc(hidden)]
  pub max_frame_size: usize,
}

impl TryFrom<Program> for BBProgram {
  type Error = InterpError;

  fn try_from(prog: Program) -> Result<Self, Self::Error> {
    Self::new(prog)
  }
}

impl BBProgram {
  /// Converts a [`Program`] into a [`BBProgram`]
  /// # Errors
  /// Will return an error if the program is invalid in some way.
  /// Reasons include the `Program` have multiple functions with the same name, a function name is not found, or a label is expected by an instruction but missing.
  /// # Panics
  /// Panics if there are more than 2^16 functions or 2^16 labels in a function
  /// or 2^16 variables in a function.
  pub fn new(prog: Program) -> Result<Self, InterpError> {
    let num_funcs = prog.functions.len();

    let func_map: FxHashMap<String, FuncIndex> = prog
      .functions
      .iter()
      .enumerate()
      .map(|(idx, func)| (func.name.clone(), FuncIndex::try_from(idx).unwrap()))
      .collect();

    let func_index = prog
      .functions
      .into_iter()
      .map(|func| BBFunction::new(func, &func_map))
      .collect::<Result<Vec<BBFunction>, InterpError>>()?;

    // Compute max frame size across all functions for pre-allocation
    let max_frame_size = func_index.iter().map(|f| f.num_of_vars).max().unwrap_or(0);

    let bb = Self {
      index_of_main: func_map.get("main").copied(),
      func_index,
      max_frame_size,
    };
    if func_map.len() == num_funcs {
      Ok(bb)
    } else {
      Err(InterpError::DuplicateFunction)
    }
  }

  #[doc(hidden)]
  #[must_use]
  pub fn get(&self, func_name: FuncIndex) -> Option<&BBFunction> {
    self.func_index.get(func_name.0 as usize)
  }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BasicBlock {
  pub label: Option<String>,
  pub flat_instrs: Vec<FlatIR>,
  pub positions: Vec<Option<Position>>,
  pub exit: Vec<LabelIndex>,
  /// Precomputed instruction count for this block (tail calls count as 2)
  pub instruction_count: usize,
}

impl BasicBlock {
  const fn new() -> Self {
    Self {
      label: None,
      flat_instrs: Vec::new(),
      positions: Vec::new(),
      exit: Vec::new(),
      instruction_count: 0,
    }
  }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct BBFunction {
  pub name: String,
  pub args: Vec<bril_rs::Argument>,
  pub return_type: Option<bril_rs::Type>,
  pub blocks: Vec<BasicBlock>,
  // The following is an optimization by replacing the string representation of variables with a number
  // Variable names are ordered from 0 to num_of_vars.
  // These replacements are found for function args and for code in the `BasicBlock`
  pub num_of_vars: usize,
  pub args_as_nums: Vec<VarIndex>,
  pub pos: Option<Position>,
}

impl BBFunction {
  fn new(f: Function, func_map: &FxHashMap<String, FuncIndex>) -> Result<Self, InterpError> {
    let mut func = Self::find_basic_blocks(f, func_map)?;
    func.build_cfg();
    Ok(func)
  }

  fn find_basic_blocks(
    func: bril_rs::Function,
    func_map: &FxHashMap<String, FuncIndex>,
  ) -> Result<Self, PositionalInterpError> {
    let mut blocks = Vec::new();
    let mut label_map = FxHashMap::default();

    let offset = func.instrs.first().map_or(0, |code| {
      // Build in an offset if the first basic block does not have a label
      if let bril_rs::Code::Label { .. } = code {
        0
      } else {
        1
      }
    });

    func.instrs.iter().try_for_each(|code| {
      if let bril_rs::Code::Label { label, pos } = code {
        if label_map.contains_key(label) {
          return Err(InterpError::DuplicateLabel(label.clone()).add_pos(pos.clone()));
        }
        label_map.insert(
          label.clone(),
          LabelIndex::try_from(label_map.len() + offset).unwrap(),
        );
      }
      Ok(())
    })?;

    let label_map = label_map;

    let mut num_var_map = FxHashMap::default();

    let args_as_nums = func
      .args
      .iter()
      .map(|a| get_num_from_map(a.name.clone(), &mut num_var_map))
      .collect();

    let mut curr_block = BasicBlock::new();
    for instr in func.instrs {
      match instr {
        bril_rs::Code::Label { label, .. } => {
          if (!curr_block.flat_instrs.is_empty() && blocks.is_empty()) || curr_block.label.is_some()
          {
            curr_block.instruction_count = curr_block.flat_instrs.len();
            blocks.push(curr_block);
          }
          curr_block = BasicBlock::new();
          curr_block.label = Some(label);
        }
        bril_rs::Code::Instruction(
          i @ bril_rs::Instruction::Effect {
            op: bril_rs::EffectOps::Jump,
            ..
          },
        ) => {
          if curr_block.label.is_some() || blocks.is_empty() {
            curr_block.positions.push(i.get_pos());
            curr_block
              .flat_instrs
              .push(FlatIR::new(i, func_map, &mut num_var_map, &label_map)?);
            curr_block.instruction_count = curr_block.flat_instrs.len();
            blocks.push(curr_block);
          }
          curr_block = BasicBlock::new();
        }
        // Handle Branch separately to detect compare-branch fusion inline
        bril_rs::Code::Instruction(
          i @ bril_rs::Instruction::Effect {
            op: bril_rs::EffectOps::Branch,
            ..
          },
        ) => {
          if curr_block.label.is_some() || blocks.is_empty() {
            let pos = i.get_pos();
            let branch_ir = FlatIR::new(i, func_map, &mut num_var_map, &label_map)?;

            // Check for compare+branch fusion: compare immediately followed by branch
            let fused = if let FlatIR::Branch {
              arg,
              true_dest,
              false_dest,
            } = &branch_ir
            {
              curr_block.flat_instrs.last().and_then(|prev| {
                let (op, ctor): (&BinaryOp, fn(CmpBranch) -> FlatIR) = match prev {
                  FlatIR::Eq(op) => (op, FlatIR::EqBranch),
                  FlatIR::Lt(op) => (op, FlatIR::LtBranch),
                  FlatIR::Gt(op) => (op, FlatIR::GtBranch),
                  FlatIR::Le(op) => (op, FlatIR::LeBranch),
                  FlatIR::Ge(op) => (op, FlatIR::GeBranch),
                  FlatIR::Feq(op) => (op, FlatIR::FeqBranch),
                  FlatIR::Flt(op) => (op, FlatIR::FltBranch),
                  FlatIR::Fgt(op) => (op, FlatIR::FgtBranch),
                  FlatIR::Fle(op) => (op, FlatIR::FleBranch),
                  FlatIR::Fge(op) => (op, FlatIR::FgeBranch),
                  _ => return None,
                };
                (op.dest == *arg).then(|| {
                  ctor(CmpBranch {
                    dest: op.dest,
                    arg0: op.arg0,
                    arg1: op.arg1,
                    true_dest: *true_dest,
                    false_dest: *false_dest,
                  })
                })
              })
            } else {
              None
            };

            if let Some(fused) = fused {
              // Replace the compare with the fused instruction, don't add the branch
              *curr_block.flat_instrs.last_mut().unwrap() = fused;
              curr_block.instruction_count = curr_block.flat_instrs.len() + 1;
            } else {
              curr_block.positions.push(pos);
              curr_block.flat_instrs.push(branch_ir);
              curr_block.instruction_count = curr_block.flat_instrs.len();
            }
            blocks.push(curr_block);
          }
          curr_block = BasicBlock::new();
        }
        // Handle Return separately to detect tail calls inline
        bril_rs::Code::Instruction(
          i @ bril_rs::Instruction::Effect {
            op: bril_rs::EffectOps::Return,
            ..
          },
        ) => {
          if curr_block.label.is_some() || blocks.is_empty() {
            let ret_ir = FlatIR::new(i.clone(), func_map, &mut num_var_map, &label_map)?;

            // Check for tail call pattern: Call followed by Return
            let is_tail_call = if let Some(prev) = curr_block.flat_instrs.last() {
              match (prev, &ret_ir) {
                (FlatIR::MultiArityCall { func, dest, args }, FlatIR::ReturnValue { arg })
                  if dest == arg =>
                {
                  Some(FlatIR::TailCall {
                    func: *func,
                    args: args.clone(),
                  })
                }
                (FlatIR::EffectfulCall { func, args }, FlatIR::ReturnVoid) => {
                  Some(FlatIR::TailCallVoid {
                    func: *func,
                    args: args.clone(),
                  })
                }
                _ => None,
              }
            } else {
              None
            };

            if let Some(tail_call) = is_tail_call {
              // Replace the call with tail call, don't add the return
              *curr_block.flat_instrs.last_mut().unwrap() = tail_call;
              // Tail call replaces call+return (2 instrs) with 1, so add 1 back
              curr_block.instruction_count = curr_block.flat_instrs.len() + 1;
            } else {
              // Normal return
              curr_block.positions.push(i.get_pos());
              curr_block.flat_instrs.push(ret_ir);
              curr_block.instruction_count = curr_block.flat_instrs.len();
            }
            blocks.push(curr_block);
          }
          curr_block = BasicBlock::new();
        }
        bril_rs::Code::Instruction(code) => {
          curr_block.positions.push(code.get_pos());
          curr_block
            .flat_instrs
            .push(FlatIR::new(code, func_map, &mut num_var_map, &label_map)?);
        }
      }
    }

    if !curr_block.flat_instrs.is_empty() || curr_block.label.is_some() {
      curr_block.instruction_count = curr_block.flat_instrs.len();
      blocks.push(curr_block);
    }

    Ok(Self {
      name: func.name,
      args: func.args,
      return_type: func.return_type,
      blocks,
      args_as_nums,
      num_of_vars: num_var_map.len(),
      pos: func.pos,
    })
  }

  fn build_cfg(&mut self) {
    if self.blocks.is_empty() {
      return;
    }
    let last_idx = self.blocks.len() - 1;
    for (i, block) in self.blocks.iter_mut().enumerate() {
      // Get the last instruction
      let last_instr = block.flat_instrs.last();

      match last_instr {
        Some(FlatIR::Jump { dest }) => block.exit.push(*dest),
        Some(FlatIR::Branch {
          true_dest,
          false_dest,
          ..
        }) => {
          block.exit.push(*true_dest);
          block.exit.push(*false_dest);
        }
        Some(
          FlatIR::EqBranch(cb)
          | FlatIR::LtBranch(cb)
          | FlatIR::GtBranch(cb)
          | FlatIR::LeBranch(cb)
          | FlatIR::GeBranch(cb)
          | FlatIR::FeqBranch(cb)
          | FlatIR::FltBranch(cb)
          | FlatIR::FgtBranch(cb)
          | FlatIR::FleBranch(cb)
          | FlatIR::FgeBranch(cb),
        ) => {
          block.exit.push(cb.true_dest);
          block.exit.push(cb.false_dest);
        }
        // Terminal instructions - no exit from this block
        Some(
          FlatIR::ReturnValue { .. }
          | FlatIR::ReturnVoid
          | FlatIR::TailCall { .. }
          | FlatIR::TailCallVoid { .. },
        ) => {}
        _ => {
          // If we're before the last block
          if i < last_idx {
            block.exit.push(LabelIndex::try_from(i + 1).unwrap());
          }
        }
      }
    }
  }
}
