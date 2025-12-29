//! stroop-vm: Virtual machine for executing Stroop bytecode.
//!
//! This crate provides register-based bytecode VMs for executing compiled bytecode.
//! It only depends on `stroop-bytecode` for the bytecode types.
//!
//! For parsing and compiling text assembly, use `stroop-text-assembly`.

pub mod bytecode_vm;
pub mod error;
pub mod value;

/// Alias for BytecodeVm
pub use bytecode_vm::BytecodeVm as Vm;
pub use bytecode_vm::{BytecodeVm, HostFn, ImportedFunc};
pub use error::RuntimeError;
pub use value::Value;
