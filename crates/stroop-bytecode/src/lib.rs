//! stroop-bytecode: Core bytecode definitions for the Stroop VM.
//!
//! This crate provides the fundamental types for the Stroop bytecode format:
//! - `Instruction`: Register-based bytecode instructions
//! - `CompiledModule`: Compiled module with instructions and imports
//! - `ValueType` and `FuncType`: Type system definitions
//!
//! For text parsing and compilation (SAT format), see the `stroop-text-assembly` crate.

pub mod instruction;
pub mod module;
pub mod types;

pub use instruction::{ConstPoolId, Instruction};
pub use module::{CompiledModule, Import, Span};
pub use types::{ConstPoolValue, FuncType, ValueType};
