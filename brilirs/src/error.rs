use bril_rs::{conversion::ConversionError, positional::PositionalErrorTrait};
use brilwf::error::CheckError;
use thiserror::Error;

// Having the #[error(...)] for all variants derives the Display trait as well
#[derive(Error, Debug)]
pub enum InterpError {
  #[error("Attempt to divide by 0")]
  DivisionByZero,
  #[error("Some memory locations have not been freed by the end of execution")]
  MemLeak,
  #[error("Trying to load from uninitialized memory")]
  UsingUninitializedMemory,
  #[error("phi node executed with no last label")]
  NoLastLabel,
  #[error("Could not find label: {0}")]
  MissingLabel(String),
  #[error("no main function defined, doing nothing")]
  NoMainFunction,
  #[error("multiple functions of the same name found")]
  DuplicateFunction,
  #[error("Expected empty return for `{0}`, found value")]
  NonEmptyRetForFunc(String),
  #[error("cannot allocate `{0}` entries")]
  CannotAllocSize(i64),
  #[error("Tried to free illegal memory location base: `{0}`, offset: `{1}`. Offset must be 0.")]
  IllegalFree(usize, i64), // (base, offset)
  #[error("Uninitialized heap location `{0}` and/or illegal offset `{1}`")]
  InvalidMemoryAccess(usize, i64), // (base, offset)
  #[error("Expected `{0}` function arguments, found `{1}`")]
  BadNumFuncArgs(usize, usize), // (expected, actual)
  #[error("no function of name `{0}` found")]
  FuncNotFound(String),
  #[error("undefined variable `{0}`")]
  VarUndefined(String),
  #[error("Label `{0}` for phi node not found")]
  PhiMissingLabel(String),
  #[error("Expected type `{0:?}` for function argument, found `{1:?}`")]
  BadFuncArgType(bril_rs::Type, String), // (expected, actual)
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  ConversionError(#[from] ConversionError), // Parsing error
  #[error(transparent)]
  CheckError(#[from] CheckError), // Typechecking error
}

impl PositionalErrorTrait<Self> for InterpError {}
