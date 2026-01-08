//! stroop-vm-bytecode: Low-level bytecode for Stroop VM.
//!
//! This crate provides the fundamental types for the Stroop bytecode format:
//! - `Instruction`: Register-based bytecode instructions
//! - `CompiledModule`: Compiled module with instructions and imports
//! - `ValueType` and `FuncType`: Type system definitions
//!
//! For text parsing and compilation (SAT format), see the `stroop-assembly-text` crate.

pub mod instruction;
pub mod module;
pub mod types;

pub use instruction::{Addr32, ConstPoolId, Instruction, Reg8};
pub use module::{CompiledModule, Function, Import, Span};
pub use types::{ConstPoolValue, FuncType, ValueType};
