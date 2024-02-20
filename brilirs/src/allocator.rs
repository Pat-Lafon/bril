use std::fmt;
use std::fmt::Debug;

use crate::error::InterpError;

pub trait BrilPointer: Debug + Clone + for<'a> From<&'a Value<Self>>
{
  fn add(&self, offset: i64) -> Self;
}

pub trait BrilAllocator<P: BrilPointer>: Default {
  fn is_empty(&self) -> bool;
  fn alloc(&mut self, amount: i64) -> Result<Value<P>, InterpError>;
  fn free(&mut self, key: &P) -> Result<(), InterpError>;
  fn write(&mut self, key: &P, val: Value<P>) -> Result<(), InterpError>;
  fn read(&self, key: &P) -> Result<&Value<P>, InterpError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Value<P: BrilPointer> {
  Int(i64),
  Bool(bool),
  Float(f64),
  Char(char),
  Pointer(P),
  #[default]
  Uninitialized,
}

impl<P: BrilPointer> fmt::Display for Value<P> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Int(i) => write!(f, "{i}"),
      Self::Bool(b) => write!(f, "{b}"),
      Self::Float(v) if v.is_infinite() && v.is_sign_positive() => write!(f, "Infinity"),
      Self::Float(v) if v.is_infinite() && v.is_sign_negative() => write!(f, "-Infinity"),
      Self::Float(v) => write!(f, "{v:.17}"),
      Self::Char(c) => write!(f, "{c}"),
      Self::Pointer(p) => write!(f, "{p:?}"),
      Self::Uninitialized => unreachable!(),
    }
  }
}

pub fn optimized_val_output<T: std::io::Write, P: BrilPointer>(
  out: &mut T,
  val: &Value<P>,
) -> Result<(), std::io::Error> {
  match val {
    Value::Int(i) => out.write_all(itoa::Buffer::new().format(*i).as_bytes()),
    Value::Bool(b) => out.write_all(if *b { b"true" } else { b"false" }),
    Value::Float(f) if f.is_infinite() && f.is_sign_positive() => out.write_all(b"Infinity"),
    Value::Float(f) if f.is_infinite() && f.is_sign_negative() => out.write_all(b"-Infinity"),
    Value::Float(f) if f.is_nan() => out.write_all(b"NaN"),
    Value::Float(f) => out.write_all(format!("{f:.17}").as_bytes()),
    Value::Char(c) => {
      let buf = &mut [0_u8; 2];
      out.write_all(c.encode_utf8(buf).as_bytes())
    }
    Value::Pointer(p) => out.write_all(format!("{p:?}").as_bytes()),
    Value::Uninitialized => unreachable!(),
  }
}

impl<P: BrilPointer> From<&bril_rs::Literal> for Value<P> {
  fn from(l: &bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(*i),
      bril_rs::Literal::Bool(b) => Self::Bool(*b),
      bril_rs::Literal::Float(f) => Self::Float(*f),
      bril_rs::Literal::Char(c) => Self::Char(*c),
    }
  }
}

impl<P: BrilPointer> From<bril_rs::Literal> for Value<P> {
  fn from(l: bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(i),
      bril_rs::Literal::Bool(b) => Self::Bool(b),
      bril_rs::Literal::Float(f) => Self::Float(f),
      bril_rs::Literal::Char(c) => Self::Char(c),
    }
  }
}

impl<P: BrilPointer> From<&Value<P>> for i64 {
  fn from(value: &Value<P>) -> Self {
    if let Value::Int(i) = value {
      *i
    } else {
      unreachable!()
    }
  }
}

impl<P: BrilPointer> From<&Value<P>> for bool {
  fn from(value: &Value<P>) -> Self {
    if let Value::Bool(b) = value {
      *b
    } else {
      unreachable!()
    }
  }
}

impl<P: BrilPointer> From<&Value<P>> for f64 {
  fn from(value: &Value<P>) -> Self {
    if let Value::Float(f) = value {
      *f
    } else {
      unreachable!()
    }
  }
}

impl<P: BrilPointer> From<&Value<P>> for char {
  fn from(value: &Value<P>) -> Self {
    if let Value::Char(c) = value {
      *c
    } else {
      unreachable!()
    }
  }
}

impl<P: BrilPointer> From<&Self> for Value<P> {
  fn from(value: &Self) -> Self {
    value.clone()
  }
}
