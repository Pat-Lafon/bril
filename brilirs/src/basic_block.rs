use crate::ir::{FlatIR, FuncIndex, LabelIndex, VarIndex, get_num_from_map};
use bril_rs::{Function, Position, Program};
use fxhash::FxHashMap;

use crate::error::{InterpError, PositionalInterpError};

/// A program represented as basic blocks. This is the IR of brilirs
#[derive(Debug)]
pub struct BBProgram {
  #[doc(hidden)]
  pub index_of_main: Option<FuncIndex>,
  #[doc(hidden)]
  pub func_index: Vec<BBFunction>,
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
  pub fn new(prog: Program) -> Result<Self, InterpError> {
    let num_funcs = prog.functions.len();

    let func_map: FxHashMap<String, FuncIndex> = prog
      .functions
      .iter()
      .enumerate()
      .map(|(idx, func)| (func.name.clone(), FuncIndex::new(idx)))
      .collect();

    let func_index = prog
      .functions
      .into_iter()
      .map(|func| BBFunction::new(func, &func_map))
      .collect::<Result<Vec<BBFunction>, InterpError>>()?;

    let bb = Self {
      index_of_main: func_map.get("main").copied(),
      func_index,
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
  // These two vecs work in parallel
  // One is the normal instruction
  // The other contains the numified version of the destination and arguments
  pub flat_instrs: Vec<FlatIR>,
  pub positions: Vec<Option<Position>>,
  pub exit: Vec<LabelIndex>,
}

impl BasicBlock {
  const fn new() -> Self {
    Self {
      label: None,
      flat_instrs: Vec::new(),
      positions: Vec::new(),
      exit: Vec::new(),
    }
  }
}
/*
#[doc(hidden)]
#[derive(Debug)]
pub struct NumifiedInstruction {
  pub dest: Option<VarIndex>,
  pub args: Vec<VarIndex>,
  pub funcs: Vec<FuncIndex>,
}

impl NumifiedInstruction {
  fn new(
    instr: &Instruction,
    // The total number of variables so far. Only grows
    num_of_vars: &mut usize,
    // A map from variables to numbers
    num_var_map: &mut FxHashMap<String, VarIndex>,
    // A map from function names to numbers
    func_map: &FxHashMap<String, FuncIndex>,
  ) -> Result<Self, PositionalInterpError> {
    Ok(match instr {
      Instruction::Constant { dest, .. } => Self {
        dest: Some(get_num_from_map(dest, num_var_map)),
        args: Vec::new(),
        funcs: Vec::new(),
      },
      Instruction::Value {
        dest,
        args,
        funcs,
        pos,
        ..
      } => Self {
        dest: Some(get_num_from_map(dest, num_of_vars, num_var_map)),
        args: args
          .iter()
          .map(|v| get_num_from_map(v, num_of_vars, num_var_map))
          .collect(),
        funcs: funcs
          .iter()
          .map(|f| {
            func_map
              .get(f)
              .copied()
              .ok_or_else(|| InterpError::FuncNotFound(f.to_string()).add_pos(pos.clone()))
          })
          .collect::<Result<Vec<_>, PositionalInterpError>>()?,
      },
      Instruction::Effect {
        args, funcs, pos, ..
      } => Self {
        dest: None,
        args: args
          .iter()
          .map(|v| get_num_from_map(v, num_of_vars, num_var_map))
          .collect(),
        funcs: funcs
          .iter()
          .map(|f| {
            func_map
              .get(f)
              .copied()
              .ok_or_else(|| InterpError::FuncNotFound(f.to_string()).add_pos(pos.clone()))
          })
          .collect::<Result<Vec<_>, PositionalInterpError>>()?,
      },
    })
  }
}
 */
#[doc(hidden)]
#[derive(Debug)]
pub struct BBFunction {
  pub name: String,
  pub args: Vec<bril_rs::Argument>,
  pub return_type: Option<bril_rs::Type>,
  pub blocks: Vec<BasicBlock>,
  // the following is an optimization by replacing the string representation of variables with a number
  // Variable names are ordered from 0 to num_of_vars.
  // These replacements are found for function args and for code in the BasicBlocks
  pub num_of_vars: usize,
  pub args_as_nums: Vec<VarIndex>,
  pub pos: Option<Position>,
}

impl BBFunction {
  fn new(f: Function, func_map: &FxHashMap<String, FuncIndex>) -> Result<Self, InterpError> {
    let mut func = Self::find_basic_blocks(f, func_map)?;
    func.build_cfg()?;
    Ok(func)
  }

  fn find_basic_blocks(
    func: bril_rs::Function,
    func_map: &FxHashMap<String, FuncIndex>,
  ) -> Result<Self, PositionalInterpError> {
    let mut blocks = Vec::new();
    let mut label_map = FxHashMap::default();

    let offset = func.instrs.iter().next().map_or(0, |code| {
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
        label_map.insert(label.clone(), LabelIndex::new(label_map.len() + offset));
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
            blocks.push(curr_block);
          }
          curr_block = BasicBlock::new();
          curr_block.label = Some(label);
        }
        bril_rs::Code::Instruction(i @ bril_rs::Instruction::Effect { op, .. })
          if op == bril_rs::EffectOps::Jump
            || op == bril_rs::EffectOps::Branch
            || op == bril_rs::EffectOps::Return =>
        {
          if curr_block.label.is_some() || blocks.is_empty() {
            curr_block
              .flat_instrs
              .push(FlatIR::new(i, func_map, &mut num_var_map, &label_map)?);
            blocks.push(curr_block);
          }
          curr_block = BasicBlock::new();
        }
        bril_rs::Code::Instruction(code) => {
          curr_block
            .flat_instrs
            .push(FlatIR::new(code, func_map, &mut num_var_map, &label_map)?)
        }
      }
    }

    if !curr_block.flat_instrs.is_empty() || curr_block.label.is_some() {
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

  fn build_cfg(&mut self) -> Result<(), InterpError> {
    if self.blocks.is_empty() {
      return Ok(());
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
          block.exit.push(*false_dest)
        }
        Some(FlatIR::ReturnValue { .. } | FlatIR::ReturnVoid) => { // We are done, there is no exit from this block}
        }
        _ => {
          // If we're before the last block
          if i < last_idx {
            block.exit.push(LabelIndex::new(i + 1));
          }
        }
      }
    }
    Ok(())
  }
}
