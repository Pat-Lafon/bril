use cfg_if::cfg_if;
use thiserror::Error;

// Having the #[error(...)] for all variants derives the Display trait as well
#[derive(Error, Debug)]
pub enum CheckError {
    #[error("Could not find label: {0}")]
    MissingLabel(String),
    #[error("phi node has unequal numbers of labels and args")]
    UnequalPhiNode,
    #[error("multiple functions name `{0}` found")]
    DuplicateFunction(String),
    #[error("multiple labels named `{0}` found in `{1}`")]
    DuplicateLabel(String, String), //Label, Function
    #[error("Expected empty return for `{0}`, found value")]
    NonEmptyRetForFunc(String),
    #[error("Expected `{0}` function arguments, found `{1}`")]
    BadNumFuncArgs(usize, usize), // (expected, actual)
    #[error("Expected `{0}` instruction arguments, found `{1}`")]
    BadNumArgs(usize, usize), // (expected, actual)
    #[error("Expected `{0}` labels, found `{1}`")]
    BadNumLabels(usize, usize), // (expected, actual)
    #[error("Expected `{0}` functions, found `{1}`")]
    BadNumFuncs(usize, usize), // (expected, actual)
    #[error("no function of name `{0}` found")]
    FuncNotFound(String),
    #[error("undefined variable `{0}`")]
    VarUndefined(String),
    #[error("Label `{0}` for phi node not found")]
    PhiMissingLabel(String),
    #[error("unspecified pointer type `{0:?}`")]
    ExpectedPointerType(bril_rs::Type), // found type
    #[error("Expected type `{0:?}` for function argument, found `{1:?}`")]
    BadFuncArgType(bril_rs::Type, String), // (expected, actual)
    #[error("Expected type `{0:?}` for assignment, found `{1:?}`")]
    BadAsmtType(bril_rs::Type, bril_rs::Type), // (expected, actual). For when the LHS type of an instruction is bad
}

cfg_if! {
    if #[cfg(feature = "position")] {
        use bril_rs::positional::PositionalErrorTrait;
        impl PositionalErrorTrait<CheckError> for CheckError {}
    } else {
        impl CheckError {
            /// This gets compiled away as a nop place holder for the `PostionalError` version when not using the "position" features
            #[must_use]
            pub const fn add_pos(self, _: Option<()>) -> CheckError {
                self
            }
            /// This gets compiled away as a nop place holder for the `PostionalError` version when not using the "position" features
            #[must_use]
            pub const fn no_pos(self) -> CheckError {
                self
            }
        }
    }
}
