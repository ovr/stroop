//! Module types for bytecode representation.

use crate::instruction::Instruction;
use crate::types::{ConstPoolValue, FuncType};

/// A span in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Function import declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// Module name (e.g., "console")
    pub module: String,
    /// Function name (e.g., "log")
    pub name: String,
    /// Optional alias (e.g., $log)
    pub alias: Option<String>,
    /// Function signature
    pub func_type: FuncType,
    /// Source span
    pub span: Span,
}

/// A compiled module ready for execution.
#[derive(Debug, Clone)]
pub struct CompiledModule {
    /// The bytecode instructions.
    pub instructions: Vec<Instruction>,
    /// Import declarations (for resolving calls).
    pub imports: Vec<Import>,
    /// Constant pool for i64/f64 values.
    pub constant_pool: Vec<ConstPoolValue>,
    /// Result type of the main function (for return value conversion).
    pub result_type: Option<crate::types::ValueType>,
}
