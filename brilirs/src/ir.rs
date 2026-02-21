use std::{fmt::Debug, num::TryFromIntError, ops::Add};

use bril_rs::{ConstOps, EffectOps, Instruction, Literal, ValueOps};
use fxhash::FxHashMap;

use crate::error::InterpError;

#[derive(Debug)]
pub struct UnaryOp {
  pub dest: VarIndex,
  pub arg: VarIndex,
}

#[derive(Debug)]
pub struct BinaryOp {
  pub dest: VarIndex,
  pub arg0: VarIndex,
  pub arg1: VarIndex,
}

#[derive(Debug)]
pub struct CmpBranch {
  pub dest: VarIndex,
  pub arg0: VarIndex,
  pub arg1: VarIndex,
  pub true_dest: LabelIndex,
  pub false_dest: LabelIndex,
}

pub type IndexType = u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FuncIndex(pub IndexType);

impl TryFrom<usize> for FuncIndex {
  type Error = TryFromIntError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    Ok(Self(IndexType::try_from(value)?))
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LabelIndex(pub IndexType);

impl TryFrom<usize> for LabelIndex {
  type Error = TryFromIntError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    Ok(Self(IndexType::try_from(value)?))
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VarIndex(pub IndexType);

impl Add<VarIndex> for usize {
  type Output = Self;

  fn add(self, rhs: VarIndex) -> Self::Output {
    self + (rhs.0 as Self)
  }
}

impl TryFrom<usize> for VarIndex {
  type Error = TryFromIntError;

  fn try_from(value: usize) -> Result<Self, Self::Error> {
    Ok(Self(IndexType::try_from(value)?))
  }
}

#[derive(Debug)]
pub enum FlatIR {
  Const {
    dest: VarIndex,
    value: Literal,
  },
  Undef {
    dest: VarIndex,
  },
  Get {
    dest: VarIndex,
  },
  // Flattened unary operations (8 variants)
  Id(UnaryOp),
  Not(UnaryOp),
  Char2int(UnaryOp),
  Int2char(UnaryOp),
  Alloc(UnaryOp),
  Load(UnaryOp),
  Float2Bits(UnaryOp),
  Bits2Float(UnaryOp),
  // Flattened binary operations (26 variants)
  Add(BinaryOp),
  Sub(BinaryOp),
  Mul(BinaryOp),
  Div(BinaryOp),
  Eq(BinaryOp),
  Lt(BinaryOp),
  Gt(BinaryOp),
  Le(BinaryOp),
  Ge(BinaryOp),
  And(BinaryOp),
  Or(BinaryOp),
  Fadd(BinaryOp),
  Fsub(BinaryOp),
  Fmul(BinaryOp),
  Fdiv(BinaryOp),
  Feq(BinaryOp),
  Flt(BinaryOp),
  Fgt(BinaryOp),
  Fle(BinaryOp),
  Fge(BinaryOp),
  Ceq(BinaryOp),
  Clt(BinaryOp),
  Cgt(BinaryOp),
  Cle(BinaryOp),
  Cge(BinaryOp),
  PtrAdd(BinaryOp),
  // Fused compare-and-branch operations (integer)
  EqBranch(CmpBranch),
  LtBranch(CmpBranch),
  GtBranch(CmpBranch),
  LeBranch(CmpBranch),
  GeBranch(CmpBranch),
  // Fused compare-and-branch operations (float)
  FeqBranch(CmpBranch),
  FltBranch(CmpBranch),
  FgtBranch(CmpBranch),
  FleBranch(CmpBranch),
  FgeBranch(CmpBranch),
  MultiArityCall {
    func: FuncIndex,
    dest: VarIndex,
    args: Vec<VarIndex>,
  },
  /// Tail call with return value - can reuse env frame
  TailCall {
    func: FuncIndex,
    args: Vec<VarIndex>,
  },
  Nop,
  Jump {
    dest: LabelIndex,
  },
  Branch {
    arg: VarIndex,
    true_dest: LabelIndex,
    false_dest: LabelIndex,
  },
  ReturnValue {
    arg: VarIndex,
  },
  ReturnVoid,
  EffectfulCall {
    func: FuncIndex,
    args: Vec<VarIndex>,
  },
  /// Tail call without return value - can reuse env frame
  TailCallVoid {
    func: FuncIndex,
    args: Vec<VarIndex>,
  },
  PrintOne {
    arg: VarIndex,
  },
  PrintMultiple {
    args: Vec<VarIndex>,
  },
  Store {
    arg0: VarIndex,
    arg1: VarIndex,
  },
  Set {
    arg0: VarIndex,
    arg1: VarIndex,
  },
  Free {
    arg: VarIndex,
  },
}

const _: () = {
  assert!(
    32 == std::mem::size_of::<FlatIR>(),
    "There is a performance improvement in shrinking the size down to 32 bytes."
  );
};

impl FlatIR {
  pub fn new(
    i: Instruction,
    func_map: &FxHashMap<String, FuncIndex>,
    num_var_map: &mut FxHashMap<String, VarIndex>,
    num_label_map: &FxHashMap<String, LabelIndex>,
  ) -> Result<Self, InterpError> {
    match i {
      Instruction::Constant {
        dest,
        op: ConstOps::Const,
        pos: _,
        const_type,
        value,
      } => Ok(Self::Const {
        dest: get_num_from_map(dest, num_var_map),
        value: if const_type == bril_rs::Type::Float {
          match value {
            #[expect(clippy::cast_precision_loss)]
            Literal::Int(i) => Literal::Float(i as f64),
            Literal::Float(_) => value,
            _ => unreachable!(),
          }
        } else {
          value
        },
      }),
      Instruction::Value {
        op: ValueOps::Undef,
        dest,
        args: _,
        funcs: _,
        labels: _,
        pos: _,
        op_type: _,
      } => Ok(Self::Undef {
        dest: get_num_from_map(dest, num_var_map),
      }),
      Instruction::Value {
        op: ValueOps::Get,
        dest,
        args: _,
        funcs: _,
        labels: _,
        pos: _,
        op_type: _,
      } => Ok(Self::Get {
        dest: get_num_from_map(dest, num_var_map),
      }),

      Instruction::Value {
        op:
          op @ (ValueOps::Id
          | ValueOps::Not
          | ValueOps::Char2int
          | ValueOps::Int2char
          | ValueOps::Alloc
          | ValueOps::Load
          | ValueOps::Float2Bits
          | ValueOps::Bits2Float),
        args,
        dest,
        funcs: _,
        labels: _,
        pos: _,
        op_type: _,
      } => {
        let dest = get_num_from_map(dest, num_var_map);

        let mut iter = args.into_iter().map(|v| get_num_from_map(v, num_var_map));
        let arg = iter.next().unwrap();
        let u = UnaryOp { dest, arg };

        Ok(match op {
          ValueOps::Id => Self::Id(u),
          ValueOps::Not => Self::Not(u),
          ValueOps::Char2int => Self::Char2int(u),
          ValueOps::Int2char => Self::Int2char(u),
          ValueOps::Alloc => Self::Alloc(u),
          ValueOps::Load => Self::Load(u),
          ValueOps::Float2Bits => Self::Float2Bits(u),
          ValueOps::Bits2Float => Self::Bits2Float(u),
          _ => unreachable!(),
        })
      }
      Instruction::Value {
        op:
          op @ (ValueOps::Add
          | ValueOps::Sub
          | ValueOps::Mul
          | ValueOps::Div
          | ValueOps::And
          | ValueOps::Or
          | ValueOps::Le
          | ValueOps::Eq
          | ValueOps::Gt
          | ValueOps::Ge
          | ValueOps::Lt
          | ValueOps::Fadd
          | ValueOps::Fsub
          | ValueOps::Fmul
          | ValueOps::Fdiv
          | ValueOps::Fle
          | ValueOps::Feq
          | ValueOps::Fgt
          | ValueOps::Fge
          | ValueOps::Flt
          | ValueOps::Ceq
          | ValueOps::Clt
          | ValueOps::Cgt
          | ValueOps::Cge
          | ValueOps::Cle
          | ValueOps::PtrAdd),
        args,
        dest,
        funcs: _,
        labels: _,
        pos: _,
        op_type: _,
      } => {
        let dest = get_num_from_map(dest, num_var_map);

        let mut iter = args.into_iter().map(|v| get_num_from_map(v, num_var_map));
        let arg0 = iter.next().unwrap();
        let arg1 = iter.next().unwrap();
        let b = BinaryOp { dest, arg0, arg1 };

        Ok(match op {
          ValueOps::Add => Self::Add(b),
          ValueOps::Sub => Self::Sub(b),
          ValueOps::Mul => Self::Mul(b),
          ValueOps::Div => Self::Div(b),
          ValueOps::Eq => Self::Eq(b),
          ValueOps::Lt => Self::Lt(b),
          ValueOps::Gt => Self::Gt(b),
          ValueOps::Le => Self::Le(b),
          ValueOps::Ge => Self::Ge(b),
          ValueOps::And => Self::And(b),
          ValueOps::Or => Self::Or(b),
          ValueOps::Fadd => Self::Fadd(b),
          ValueOps::Fsub => Self::Fsub(b),
          ValueOps::Fmul => Self::Fmul(b),
          ValueOps::Fdiv => Self::Fdiv(b),
          ValueOps::Feq => Self::Feq(b),
          ValueOps::Flt => Self::Flt(b),
          ValueOps::Fgt => Self::Fgt(b),
          ValueOps::Fle => Self::Fle(b),
          ValueOps::Fge => Self::Fge(b),
          ValueOps::Ceq => Self::Ceq(b),
          ValueOps::Clt => Self::Clt(b),
          ValueOps::Cgt => Self::Cgt(b),
          ValueOps::Cle => Self::Cle(b),
          ValueOps::Cge => Self::Cge(b),
          ValueOps::PtrAdd => Self::PtrAdd(b),
          _ => unreachable!(),
        })
      }
      Instruction::Value {
        op: ValueOps::Call,
        args,
        dest,
        funcs,
        labels: _,
        pos: _,
        op_type: _,
      } => {
        let dest = get_num_from_map(dest, num_var_map);
        let args = args
          .into_iter()
          .map(|v| get_num_from_map(v, num_var_map))
          .collect();
        let func = func_map.get(&funcs[0]).copied().unwrap();
        Ok(Self::MultiArityCall { func, dest, args })
      }
      Instruction::Effect {
        op: EffectOps::Nop,
        args: _,
        funcs: _,
        labels: _,
        pos: _,
      } => Ok(Self::Nop),
      Instruction::Effect {
        op: EffectOps::Jump,
        args: _,
        funcs: _,
        labels,
        pos: _,
      } => {
        let dest = labels
          .into_iter()
          .map(|v| {
            num_label_map
              .get(&v)
              .copied()
              .ok_or_else(|| InterpError::MissingLabel(v.clone()))
          })
          .next()
          .unwrap()?;
        Ok(Self::Jump { dest })
      }
      Instruction::Effect {
        op: EffectOps::Branch,
        args,
        funcs: _,
        labels,
        pos: _,
      } => {
        let arg = args
          .into_iter()
          .map(|v| get_num_from_map(v, num_var_map))
          .next()
          .unwrap();
        let mut iter = labels.into_iter().map(|v| {
          num_label_map
            .get(&v)
            .copied()
            .ok_or_else(|| InterpError::MissingLabel(v.clone()))
        });
        let true_dest = iter.next().unwrap()?;
        let false_dest = iter.next().unwrap()?;
        Ok(Self::Branch {
          arg,
          true_dest,
          false_dest,
        })
      }
      Instruction::Effect {
        op: EffectOps::Return,
        args,
        funcs: _,
        labels: _,
        pos: _,
      } => {
        if args.is_empty() {
          Ok(Self::ReturnVoid)
        } else {
          let arg = args
            .into_iter()
            .map(|v| get_num_from_map(v, num_var_map))
            .next()
            .unwrap();
          Ok(Self::ReturnValue { arg })
        }
      }
      Instruction::Effect {
        op: EffectOps::Call,
        args,
        funcs,
        labels: _,
        pos: _,
      } => {
        let args = args
          .into_iter()
          .map(|v| get_num_from_map(v, num_var_map))
          .collect();
        let func = func_map.get(&funcs[0]).copied().unwrap();
        Ok(Self::EffectfulCall { func, args })
      }
      Instruction::Effect {
        op: EffectOps::Print,
        args,
        funcs: _,
        labels: _,
        pos: _,
      } => {
        if args.len() == 1 {
          let arg = args
            .into_iter()
            .map(|v| get_num_from_map(v, num_var_map))
            .next()
            .unwrap();
          Ok(Self::PrintOne { arg })
        } else {
          let args = args
            .into_iter()
            .map(|v| get_num_from_map(v, num_var_map))
            .collect();
          Ok(Self::PrintMultiple { args })
        }
      }
      Instruction::Effect {
        op: EffectOps::Store,
        args,
        funcs: _,
        labels: _,
        pos: _,
      } => {
        let mut iter = args.into_iter().map(|v| get_num_from_map(v, num_var_map));
        let arg0 = iter.next().unwrap();
        let arg1 = iter.next().unwrap();
        Ok(Self::Store { arg0, arg1 })
      }
      Instruction::Effect {
        op: EffectOps::Set,
        args,
        funcs: _,
        labels: _,
        pos: _,
      } => {
        let mut iter = args.into_iter().map(|v| get_num_from_map(v, num_var_map));
        let arg0 = iter.next().unwrap();
        let arg1 = iter.next().unwrap();
        Ok(Self::Set { arg0, arg1 })
      }
      Instruction::Effect {
        op: EffectOps::Free,
        args,
        funcs: _,
        labels: _,
        pos: _,
      } => {
        let arg = args
          .into_iter()
          .map(|v| get_num_from_map(v, num_var_map))
          .next()
          .unwrap();
        Ok(Self::Free { arg })
      }
      Instruction::Effect {
        op: EffectOps::Speculate | EffectOps::Guard | EffectOps::Commit,
        ..
      } => unimplemented!("brilirs does not currently support the speculative execution extension"),
    }
  }
}

pub fn get_num_from_map<T: Copy + TryFrom<usize, Error = impl Debug>>(
  variable_name: String,
  num_var_map: &mut FxHashMap<String, T>,
) -> T {
  num_var_map.get(&variable_name).copied().unwrap_or_else(|| {
    let x = num_var_map.len().try_into().unwrap();
    num_var_map.insert(variable_name, x);
    x
  })
}
