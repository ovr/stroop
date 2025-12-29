//! Register-based bytecode virtual machine.

use crate::error::RuntimeError;
use crate::value::Value;
use stroop_bytecode::{Addr32, CompiledModule, FuncType, Instruction};

/// Host function that can be called from the VM.
pub type HostFn = Box<dyn Fn(&[Value]) -> Result<Option<Value>, RuntimeError>>;

/// Imported function with its type and implementation.
pub struct ImportedFunc {
    pub module: String,
    pub name: String,
    pub func_type: FuncType,
    pub func: HostFn,
}

/// Label for control flow.
#[derive(Debug, Clone, Copy)]
struct Label {
    target: Addr32,
    is_loop: bool,
}

/// Register-based bytecode virtual machine.
pub struct BytecodeVm {
    /// Fixed-size register file (256 registers).
    regs: [Value; 256],
    /// Label stack for control flow.
    label_stack: Vec<Label>,
    /// Imported functions.
    imports: Vec<ImportedFunc>,
}

impl Default for BytecodeVm {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeVm {
    /// Create a new register-based VM.
    pub fn new() -> Self {
        Self {
            regs: [Value::I32(0); 256],
            label_stack: Vec::with_capacity(32),
            imports: Vec::new(),
        }
    }

    /// Create a new VM with specified number of locals (for compatibility).
    pub fn with_locals(_num_locals: usize) -> Self {
        Self::new()
    }

    /// Register a host function.
    pub fn register_host_fn<F>(&mut self, module: &str, name: &str, func_type: FuncType, func: F)
    where
        F: Fn(&[Value]) -> Result<Option<Value>, RuntimeError> + 'static,
    {
        self.imports.push(ImportedFunc {
            module: module.to_string(),
            name: name.to_string(),
            func_type,
            func: Box::new(func),
        });
    }

    /// Execute a compiled module.
    pub fn execute(&mut self, module: &CompiledModule) -> Result<Option<Value>, RuntimeError> {
        let code = &module.instructions;
        let mut pc = 0usize;

        loop {
            match code[pc] {
                // Constants
                Instruction::LoadConstI32 { dst, value } => {
                    self.regs[dst as usize] = Value::I32(value);
                    pc += 1;
                }
                Instruction::LoadConstI64 { dst, index } => {
                    let value = module.constant_pool[index as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(value);
                    pc += 1;
                }
                Instruction::LoadConstF32 { dst, value } => {
                    self.regs[dst as usize] = Value::F32(value);
                    pc += 1;
                }
                Instruction::LoadConstF64 { dst, index } => {
                    let value = module.constant_pool[index as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(value);
                    pc += 1;
                }

                // Register move
                Instruction::Mov { dst, src } => {
                    self.regs[dst as usize] = self.regs[src as usize];
                    pc += 1;
                }

                // i32 arithmetic
                Instruction::I32Add { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a.wrapping_add(b));
                    pc += 1;
                }
                Instruction::I32Sub { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a.wrapping_sub(b));
                    pc += 1;
                }
                Instruction::I32Mul { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a.wrapping_mul(b));
                    pc += 1;
                }
                Instruction::I32DivS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    if a == i32::MIN && b == -1 {
                        return Err(RuntimeError::IntegerOverflow);
                    }
                    self.regs[dst as usize] = Value::I32(a / b);
                    pc += 1;
                }
                Instruction::I32DivU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32() as u32;
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.regs[dst as usize] = Value::I32((a / b) as i32);
                    pc += 1;
                }
                Instruction::I32RemS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.regs[dst as usize] = Value::I32(a % b);
                    pc += 1;
                }
                Instruction::I32RemU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32() as u32;
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.regs[dst as usize] = Value::I32((a % b) as i32);
                    pc += 1;
                }
                Instruction::I32And { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a & b);
                    pc += 1;
                }
                Instruction::I32Or { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a | b);
                    pc += 1;
                }
                Instruction::I32Xor { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a ^ b);
                    pc += 1;
                }
                Instruction::I32Shl { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a << (b & 31));
                    pc += 1;
                }
                Instruction::I32ShrS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(a >> (b & 31));
                    pc += 1;
                }
                Instruction::I32ShrU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32((a >> (b & 31)) as i32);
                    pc += 1;
                }

                // i32 comparison
                Instruction::I32Eq { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(if a == b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32Ne { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(if a != b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32LtS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(if a < b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32LtU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32() as u32;
                    self.regs[dst as usize] = Value::I32(if a < b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32GtS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(if a > b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32GtU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32() as u32;
                    self.regs[dst as usize] = Value::I32(if a > b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32LeS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(if a <= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32LeU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32() as u32;
                    self.regs[dst as usize] = Value::I32(if a <= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32GeS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32();
                    let b = self.regs[rhs as usize].as_i32();
                    self.regs[dst as usize] = Value::I32(if a >= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I32GeU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i32() as u32;
                    let b = self.regs[rhs as usize].as_i32() as u32;
                    self.regs[dst as usize] = Value::I32(if a >= b { 1 } else { 0 });
                    pc += 1;
                }

                // i64 arithmetic
                Instruction::I64Add { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a.wrapping_add(b));
                    pc += 1;
                }
                Instruction::I64Sub { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a.wrapping_sub(b));
                    pc += 1;
                }
                Instruction::I64Mul { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a.wrapping_mul(b));
                    pc += 1;
                }
                Instruction::I64DivS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    if a == i64::MIN && b == -1 {
                        return Err(RuntimeError::IntegerOverflow);
                    }
                    self.regs[dst as usize] = Value::I64(a / b);
                    pc += 1;
                }
                Instruction::I64DivU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64() as u64;
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.regs[dst as usize] = Value::I64((a / b) as i64);
                    pc += 1;
                }
                Instruction::I64RemS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.regs[dst as usize] = Value::I64(a % b);
                    pc += 1;
                }
                Instruction::I64RemU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64() as u64;
                    if b == 0 {
                        return Err(RuntimeError::DivisionByZero);
                    }
                    self.regs[dst as usize] = Value::I64((a % b) as i64);
                    pc += 1;
                }
                Instruction::I64And { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a & b);
                    pc += 1;
                }
                Instruction::I64Or { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a | b);
                    pc += 1;
                }
                Instruction::I64Xor { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a ^ b);
                    pc += 1;
                }
                Instruction::I64Shl { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a << (b & 63));
                    pc += 1;
                }
                Instruction::I64ShrS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64(a >> (b & 63));
                    pc += 1;
                }
                Instruction::I64ShrU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I64((a >> (b & 63)) as i64);
                    pc += 1;
                }

                // i64 comparison
                Instruction::I64Eq { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I32(if a == b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64Ne { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I32(if a != b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64LtS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I32(if a < b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64LtU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64() as u64;
                    self.regs[dst as usize] = Value::I32(if a < b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64GtS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I32(if a > b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64GtU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64() as u64;
                    self.regs[dst as usize] = Value::I32(if a > b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64LeS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I32(if a <= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64LeU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64() as u64;
                    self.regs[dst as usize] = Value::I32(if a <= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64GeS { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64();
                    let b = self.regs[rhs as usize].as_i64();
                    self.regs[dst as usize] = Value::I32(if a >= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::I64GeU { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_i64() as u64;
                    let b = self.regs[rhs as usize].as_i64() as u64;
                    self.regs[dst as usize] = Value::I32(if a >= b { 1 } else { 0 });
                    pc += 1;
                }

                // f32 arithmetic
                Instruction::F32Add { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::F32(a + b);
                    pc += 1;
                }
                Instruction::F32Sub { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::F32(a - b);
                    pc += 1;
                }
                Instruction::F32Mul { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::F32(a * b);
                    pc += 1;
                }
                Instruction::F32Div { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::F32(a / b);
                    pc += 1;
                }
                Instruction::F32Min { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::F32(a.min(b));
                    pc += 1;
                }
                Instruction::F32Max { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::F32(a.max(b));
                    pc += 1;
                }

                // f32 unary
                Instruction::F32Abs { dst, src } => {
                    self.regs[dst as usize] = Value::F32(self.regs[src as usize].as_f32().abs());
                    pc += 1;
                }
                Instruction::F32Neg { dst, src } => {
                    self.regs[dst as usize] = Value::F32(-self.regs[src as usize].as_f32());
                    pc += 1;
                }
                Instruction::F32Ceil { dst, src } => {
                    self.regs[dst as usize] = Value::F32(self.regs[src as usize].as_f32().ceil());
                    pc += 1;
                }
                Instruction::F32Floor { dst, src } => {
                    self.regs[dst as usize] = Value::F32(self.regs[src as usize].as_f32().floor());
                    pc += 1;
                }
                Instruction::F32Trunc { dst, src } => {
                    self.regs[dst as usize] = Value::F32(self.regs[src as usize].as_f32().trunc());
                    pc += 1;
                }
                Instruction::F32Nearest { dst, src } => {
                    self.regs[dst as usize] = Value::F32(self.regs[src as usize].as_f32().round());
                    pc += 1;
                }
                Instruction::F32Sqrt { dst, src } => {
                    self.regs[dst as usize] = Value::F32(self.regs[src as usize].as_f32().sqrt());
                    pc += 1;
                }

                // f32 comparison
                Instruction::F32Eq { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::I32(if a == b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F32Ne { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::I32(if a != b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F32Lt { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::I32(if a < b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F32Gt { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::I32(if a > b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F32Le { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::I32(if a <= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F32Ge { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f32();
                    let b = self.regs[rhs as usize].as_f32();
                    self.regs[dst as usize] = Value::I32(if a >= b { 1 } else { 0 });
                    pc += 1;
                }

                // f64 arithmetic
                Instruction::F64Add { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(a + b);
                    pc += 1;
                }
                Instruction::F64Sub { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(a - b);
                    pc += 1;
                }
                Instruction::F64Mul { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(a * b);
                    pc += 1;
                }
                Instruction::F64Div { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(a / b);
                    pc += 1;
                }
                Instruction::F64Min { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(a.min(b));
                    pc += 1;
                }
                Instruction::F64Max { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::F64(a.max(b));
                    pc += 1;
                }

                // f64 unary
                Instruction::F64Abs { dst, src } => {
                    self.regs[dst as usize] = Value::F64(self.regs[src as usize].as_f64().abs());
                    pc += 1;
                }
                Instruction::F64Neg { dst, src } => {
                    self.regs[dst as usize] = Value::F64(-self.regs[src as usize].as_f64());
                    pc += 1;
                }
                Instruction::F64Ceil { dst, src } => {
                    self.regs[dst as usize] = Value::F64(self.regs[src as usize].as_f64().ceil());
                    pc += 1;
                }
                Instruction::F64Floor { dst, src } => {
                    self.regs[dst as usize] = Value::F64(self.regs[src as usize].as_f64().floor());
                    pc += 1;
                }
                Instruction::F64Trunc { dst, src } => {
                    self.regs[dst as usize] = Value::F64(self.regs[src as usize].as_f64().trunc());
                    pc += 1;
                }
                Instruction::F64Nearest { dst, src } => {
                    self.regs[dst as usize] = Value::F64(self.regs[src as usize].as_f64().round());
                    pc += 1;
                }
                Instruction::F64Sqrt { dst, src } => {
                    self.regs[dst as usize] = Value::F64(self.regs[src as usize].as_f64().sqrt());
                    pc += 1;
                }

                // f64 comparison
                Instruction::F64Eq { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::I32(if a == b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F64Ne { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::I32(if a != b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F64Lt { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::I32(if a < b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F64Gt { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::I32(if a > b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F64Le { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::I32(if a <= b { 1 } else { 0 });
                    pc += 1;
                }
                Instruction::F64Ge { dst, lhs, rhs } => {
                    let a = self.regs[lhs as usize].as_f64();
                    let b = self.regs[rhs as usize].as_f64();
                    self.regs[dst as usize] = Value::I32(if a >= b { 1 } else { 0 });
                    pc += 1;
                }

                // Control flow
                Instruction::Block { end } => {
                    self.label_stack.push(Label {
                        target: end,
                        is_loop: false,
                    });
                    pc += 1;
                }
                Instruction::Loop { start } => {
                    self.label_stack.push(Label {
                        target: start,
                        is_loop: true,
                    });
                    pc += 1;
                }
                Instruction::Br { depth } => {
                    let idx = self.label_stack.len() - 1 - depth as usize;
                    let label = self.label_stack[idx];
                    pc = label.target as usize;
                    if label.is_loop {
                        self.label_stack.truncate(idx + 1);
                    } else {
                        self.label_stack.truncate(idx);
                    }
                }
                Instruction::BrIf { cond, depth } => {
                    if self.regs[cond as usize].as_i32() != 0 {
                        let idx = self.label_stack.len() - 1 - depth as usize;
                        let label = self.label_stack[idx];
                        pc = label.target as usize;
                        if label.is_loop {
                            self.label_stack.truncate(idx + 1);
                        } else {
                            self.label_stack.truncate(idx);
                        }
                    } else {
                        pc += 1;
                    }
                }
                Instruction::End => {
                    self.label_stack.pop();
                    pc += 1;
                }

                // Function calls
                Instruction::Call {
                    func,
                    base,
                    argc,
                    dst,
                } => {
                    let import = &module.imports[func as usize];
                    let host_fn = self
                        .imports
                        .iter()
                        .find(|f| f.module == import.module && f.name == import.name)
                        .ok_or_else(|| RuntimeError::FunctionNotFound {
                            name: format!("{}.{}", import.module, import.name),
                        })?;

                    let args: Vec<Value> =
                        (0..argc).map(|i| self.regs[(base + i) as usize]).collect();

                    let result = (host_fn.func)(&args)?;
                    if let Some(v) = result {
                        self.regs[dst as usize] = v;
                    }
                    pc += 1;
                }

                Instruction::Halt => break,
            }
        }

        Ok(Some(self.regs[0]))
    }
}
