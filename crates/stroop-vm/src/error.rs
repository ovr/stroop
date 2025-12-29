//! Runtime errors for the Stroop VM.

use stroop_bytecode::ValueType;

/// Runtime error during VM execution.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// Type mismatch during operation.
    TypeMismatch {
        expected: ValueType,
        found: ValueType,
    },

    /// Division by zero.
    DivisionByZero,

    /// Integer overflow.
    IntegerOverflow,

    /// Invalid local variable index.
    InvalidLocalIndex { index: u32, max: u32 },

    /// Invalid register index.
    InvalidRegisterIndex { index: u32, max: u32 },

    /// Invalid function index.
    InvalidFunctionIndex { index: u32, max: u32 },

    /// Function not found by name.
    FunctionNotFound { name: String },

    /// Argument count mismatch.
    ArgumentCountMismatch { expected: usize, found: usize },

    /// Stack underflow.
    StackUnderflow,

    /// No result from expression.
    NoResult,

    /// Invalid branch depth (branch target out of scope).
    InvalidBranchDepth { depth: u32, max: u32 },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {}, found {}", expected, found)
            }
            RuntimeError::DivisionByZero => write!(f, "division by zero"),
            RuntimeError::IntegerOverflow => write!(f, "integer overflow"),
            RuntimeError::InvalidLocalIndex { index, max } => {
                write!(f, "invalid local index {} (max {})", index, max)
            }
            RuntimeError::InvalidRegisterIndex { index, max } => {
                write!(f, "invalid register index {} (max {})", index, max)
            }
            RuntimeError::InvalidFunctionIndex { index, max } => {
                write!(f, "invalid function index {} (max {})", index, max)
            }
            RuntimeError::FunctionNotFound { name } => {
                write!(f, "function not found: {}", name)
            }
            RuntimeError::ArgumentCountMismatch { expected, found } => {
                write!(
                    f,
                    "argument count mismatch: expected {}, found {}",
                    expected, found
                )
            }
            RuntimeError::StackUnderflow => write!(f, "stack underflow"),
            RuntimeError::NoResult => write!(f, "no result from expression"),
            RuntimeError::InvalidBranchDepth { depth, max } => {
                write!(f, "invalid branch depth {} (max {})", depth, max)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
