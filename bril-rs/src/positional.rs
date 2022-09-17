use std::error::Error;
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// <https://capra.cs.cornell.edu/bril/lang/syntax.html#source-positions>
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// Column
    pub col: u64,
    /// Row
    pub row: u64,
}

/// A wrapper around an error type containing a optional Position
#[derive(Error, Debug)]
pub struct PositionalError<E: Error> {
    ///
    pub e: E,
    ///
    pub pos: Option<Position>,
}

impl<E: Error + From<std::io::Error>> From<std::io::Error> for PositionalError<E> {
    fn from(e: std::io::Error) -> Self {
        PositionalError {
            e: e.into(),
            pos: None,
        }
    }
}

impl<E: Error> PositionalError<E> {
    /// My fake From/Into Trait
    /// # Errors
    /// If previous `Result` was an error then the output will be aswell
    pub fn convert<T, E2: Error + From<E>>(r: Result<T, Self>) -> Result<T, PositionalError<E2>> {
        r.map_err(|PositionalError { e, pos }| PositionalError { e: e.into(), pos })
    }

    /// Add position information is None is currently available
    #[must_use]
    // https://github.com/rust-lang/rust-clippy/issues/8874
    // https://github.com/rust-lang/rust/issues/73255
    #[allow(clippy::missing_const_for_fn)]
    pub fn add_pos(self, pos: Option<Position>) -> Self {
        match self {
            PositionalError { e, pos: None } => PositionalError { e, pos },
            _ => self,
        }
    }
}

/// A helper error trait to create `PositionalError` from arbitrary errors
pub trait PositionalErrorTrait<E: Error>: Error + Sized {
    /// Optionally adds a position to an Error
    fn add_pos(self, pos: Option<Position>) -> PositionalError<Self> {
        PositionalError { e: self, pos }
    }

    /// Gives the wrapper without any position
    fn no_pos(self) -> PositionalError<Self> {
        PositionalError { e: self, pos: None }
    }
}

impl<E: Error> Display for PositionalError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionalError { e, pos: Some(pos) } => {
                write!(f, "Line {}, Column {}: {e}", pos.row, pos.col)
            }
            PositionalError { e, pos: None } => write!(f, "{e}"),
        }
    }
}
