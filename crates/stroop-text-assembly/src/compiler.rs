//! Compiler that transforms AST to register-based bytecode.

use indexmap::IndexMap;

use crate::ast::{ConstValue, Expr, Module};
use crate::opcode::Opcode;
use stroop_bytecode::{Addr32, CompiledModule, ConstPoolId, ConstPoolValue, Instruction};

/// Compile-time label for control flow resolution.
#[derive(Debug, Clone, Copy)]
struct CompileLabel {
    /// Target address (for loops: start, for blocks: to be patched)
    target: Addr32,
    /// Whether this is a loop (determines where br jumps to)
    is_loop: bool,
}

/// Compiler state with register allocation.
struct Compiler {
    code: Vec<Instruction>,
    /// Next available register
    next_reg: u8,
    /// First temp register (after locals and cached constants)
    first_temp: u8,
    /// Cache of constants to their dedicated registers (key is f64 bits for reliable equality)
    const_cache: IndexMap<u64, u8>,
    /// Constant pool map: ConstPoolValue -> pool index (IndexMap preserves insertion order)
    pool_index_map: IndexMap<ConstPoolValue, ConstPoolId>,
    /// Compile-time label stack for resolving branch targets
    label_stack: Vec<CompileLabel>,
    /// Pending patches: (instruction_index, label_stack_depth) for br/br_if to blocks
    pending_patches: Vec<(usize, usize)>,
}

impl Compiler {
    fn new(num_locals: u8) -> Self {
        Self {
            code: Vec::new(),
            next_reg: num_locals,
            first_temp: num_locals,
            const_cache: IndexMap::new(),
            pool_index_map: IndexMap::new(),
            label_stack: Vec::with_capacity(32),
            pending_patches: Vec::new(),
        }
    }

    /// Add a constant to the pool, returning its index.
    fn add_const_to_pool(&mut self, key: ConstPoolValue) -> ConstPoolId {
        if let Some(&index) = self.pool_index_map.get(&key) {
            return index;
        }
        let index = self.pool_index_map.len() as ConstPoolId;
        self.pool_index_map.insert(key, index);
        index
    }

    /// Preload frequently used constants into dedicated registers.
    fn preload_constants(&mut self, module: &Module) {
        let mut constants = IndexMap::new();
        for expr in &module.body {
            collect_constants(expr, &mut constants);
        }

        // Load each unique constant into a dedicated register
        for (bits, count) in constants {
            // Only cache constants used more than once
            if count > 1 {
                let reg = self.alloc_temp();
                let index = self.add_const_to_pool(ConstPoolValue::F64(f64::from_bits(bits)));
                self.emit(Instruction::LoadConstF64 { dst: reg, index });
                self.const_cache.insert(bits, reg);
            }
        }

        // Update first_temp to preserve cached constant registers
        self.first_temp = self.next_reg;
    }

    /// Get cached register for a constant, if available.
    fn get_const_reg(&self, value: &ConstValue) -> Option<u8> {
        match value {
            ConstValue::F64(v) => self.const_cache.get(&v.to_bits()).copied(),
            ConstValue::F32(v) => self.const_cache.get(&(*v as f64).to_bits()).copied(),
            _ => None,
        }
    }

    /// Allocate a temporary register.
    fn alloc_temp(&mut self) -> u8 {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    /// Free temporary registers (reset to after locals and cached constants).
    fn free_temps(&mut self) {
        self.next_reg = self.first_temp;
    }

    /// Emit an instruction.
    fn emit(&mut self, instr: Instruction) {
        self.code.push(instr);
    }

    /// Get register for a simple expression (LocalGet or cached constant) without emitting code.
    /// Returns None if expression is not a simple register reference.
    fn get_simple_reg(&self, expr: &Expr) -> Option<u8> {
        match expr {
            Expr::LocalGet { index, .. } => Some(*index as u8),
            Expr::RegGet { index, .. } => Some((*index as u8).wrapping_add(100)),
            Expr::Const { value, .. } => self.get_const_reg(value),
            _ => None,
        }
    }

    /// Compile an expression, placing result in dst register.
    fn compile_expr(&mut self, expr: &Expr, dst: u8) {
        match expr {
            Expr::Const { value, .. } => {
                // Check if constant is cached
                if let Some(cached_reg) = self.get_const_reg(value) {
                    if cached_reg != dst {
                        self.emit(Instruction::Mov {
                            dst,
                            src: cached_reg,
                        });
                    }
                } else {
                    match value {
                        ConstValue::I32(v) => {
                            // Keep i32 inline (only 4 bytes)
                            self.emit(Instruction::LoadConstI32 { dst, value: *v })
                        }
                        ConstValue::I64(v) => {
                            // Use constant pool for i64
                            let index = self.add_const_to_pool(ConstPoolValue::I64(*v));
                            self.emit(Instruction::LoadConstI64 { dst, index })
                        }
                        ConstValue::F32(v) => {
                            // Keep f32 inline (only 4 bytes)
                            self.emit(Instruction::LoadConstF32 { dst, value: *v })
                        }
                        ConstValue::F64(v) => {
                            // Use constant pool for f64
                            let index = self.add_const_to_pool(ConstPoolValue::F64(*v));
                            self.emit(Instruction::LoadConstF64 { dst, index })
                        }
                    }
                }
            }

            Expr::LocalGet { index, .. } => {
                let src = *index as u8;
                if src != dst {
                    self.emit(Instruction::Mov { dst, src });
                }
            }

            Expr::LocalSet { index, value, .. } => {
                let local_reg = *index as u8;
                self.compile_expr(value, local_reg);
                // Don't emit Mov for the return value - caller decides if needed
                if local_reg != dst {
                    self.emit(Instruction::Mov {
                        dst,
                        src: local_reg,
                    });
                }
            }

            Expr::LocalTee { index, value, .. } => {
                let local_reg = *index as u8;
                self.compile_expr(value, local_reg);
                if local_reg != dst {
                    self.emit(Instruction::Mov {
                        dst,
                        src: local_reg,
                    });
                }
            }

            Expr::RegGet { index, .. } => {
                // Treat reg as high-numbered locals
                let src = (*index as u8).wrapping_add(100);
                if src != dst {
                    self.emit(Instruction::Mov { dst, src });
                }
            }

            Expr::RegSet { index, value, .. } => {
                let reg = (*index as u8).wrapping_add(100);
                self.compile_expr(value, reg);
                if reg != dst {
                    self.emit(Instruction::Mov { dst, src: reg });
                }
            }

            Expr::RegTee { index, value, .. } => {
                let reg = (*index as u8).wrapping_add(100);
                self.compile_expr(value, reg);
                if reg != dst {
                    self.emit(Instruction::Mov { dst, src: reg });
                }
            }

            Expr::BinaryOp {
                opcode, lhs, rhs, ..
            } => {
                // Optimize: use local registers directly if possible
                let lhs_reg = if let Some(r) = self.get_simple_reg(lhs) {
                    r
                } else {
                    let r = self.alloc_temp();
                    self.compile_expr(lhs, r);
                    r
                };
                let rhs_reg = if let Some(r) = self.get_simple_reg(rhs) {
                    r
                } else {
                    let r = self.alloc_temp();
                    self.compile_expr(rhs, r);
                    r
                };
                self.emit(binary_op_instruction(*opcode, dst, lhs_reg, rhs_reg));
            }

            Expr::UnaryOp {
                opcode, operand, ..
            } => {
                // Optimize: use local register directly if possible
                let src_reg = if let Some(r) = self.get_simple_reg(operand) {
                    r
                } else {
                    let r = self.alloc_temp();
                    self.compile_expr(operand, r);
                    r
                };
                self.emit(unary_op_instruction(*opcode, dst, src_reg));
            }

            Expr::Call { func_idx, args, .. } => {
                // Put args in consecutive registers starting at base
                let base = self.alloc_temp();
                for (i, arg) in args.iter().enumerate() {
                    let arg_reg = base + i as u8;
                    self.next_reg = arg_reg + 1;
                    self.compile_expr(arg, arg_reg);
                }
                self.emit(Instruction::Call {
                    func: *func_idx,
                    base,
                    argc: args.len() as u8,
                    dst,
                });
            }

            Expr::Block { body, .. } => {
                // Push block label (target will be patched when block ends)
                let label_idx = self.label_stack.len();
                self.label_stack.push(CompileLabel {
                    target: 0, // placeholder, will be patched
                    is_loop: false,
                });

                for (i, e) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    if is_last {
                        self.compile_expr(e, dst);
                    } else {
                        let tmp = self.alloc_temp();
                        self.compile_expr(e, tmp);
                        self.free_temps();
                    }
                }

                // Block ends here - patch all pending jumps to this label
                let end_pos = self.code.len() as Addr32;
                self.label_stack[label_idx].target = end_pos;

                // Patch pending Jump/JumpIf instructions that target this label
                self.pending_patches
                    .retain(|(instr_idx, target_label_idx)| {
                        if *target_label_idx == label_idx {
                            match &mut self.code[*instr_idx] {
                                Instruction::Jump { target } => *target = end_pos,
                                Instruction::JumpIf { target, .. } => *target = end_pos,
                                _ => {}
                            }
                            false // remove from pending
                        } else {
                            true // keep in pending
                        }
                    });

                self.label_stack.pop();
            }

            Expr::Loop { body, .. } => {
                // Loop target is the start (for continue)
                let loop_start = self.code.len() as Addr32;
                self.label_stack.push(CompileLabel {
                    target: loop_start,
                    is_loop: true,
                });

                for (i, e) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    if is_last {
                        self.compile_expr(e, dst);
                    } else {
                        let tmp = self.alloc_temp();
                        self.compile_expr(e, tmp);
                        self.free_temps();
                    }
                }

                self.label_stack.pop();
            }

            Expr::Br { label_depth, .. } => {
                let label_idx = self.label_stack.len() - 1 - *label_depth as usize;
                let label = self.label_stack[label_idx];

                if label.is_loop {
                    // Loop: target is known (start of loop)
                    self.emit(Instruction::Jump {
                        target: label.target,
                    });
                } else {
                    // Block: target not yet known, emit placeholder and record for patching
                    let instr_idx = self.code.len();
                    self.emit(Instruction::Jump { target: 0 });
                    self.pending_patches.push((instr_idx, label_idx));
                }
            }

            Expr::BrIf {
                label_depth,
                condition,
                ..
            } => {
                let cond_reg = self.alloc_temp();
                self.compile_expr(condition, cond_reg);

                let label_idx = self.label_stack.len() - 1 - *label_depth as usize;
                let label = self.label_stack[label_idx];

                if label.is_loop {
                    // Loop: target is known (start of loop)
                    self.emit(Instruction::JumpIf {
                        cond: cond_reg,
                        target: label.target,
                    });
                } else {
                    // Block: target not yet known, emit placeholder and record for patching
                    let instr_idx = self.code.len();
                    self.emit(Instruction::JumpIf {
                        cond: cond_reg,
                        target: 0,
                    });
                    self.pending_patches.push((instr_idx, label_idx));
                }
            }

            Expr::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                // Compile condition
                let cond_reg = self.alloc_temp();
                self.compile_expr(condition, cond_reg);

                if else_body.is_none() {
                    // Simple if without else:
                    // JumpIf (inverted) to end if condition is false
                    // <then body>
                    // end:

                    // Invert condition: emit cond == 0
                    let zero_reg = self.alloc_temp();
                    self.emit(Instruction::LoadConstI32 {
                        dst: zero_reg,
                        value: 0,
                    });
                    let inv_cond_reg = self.alloc_temp();
                    self.emit(Instruction::I32Eq {
                        dst: inv_cond_reg,
                        lhs: cond_reg,
                        rhs: zero_reg,
                    });

                    // Jump to end if inverted condition is true (i.e., original was false)
                    let jump_idx = self.code.len();
                    self.emit(Instruction::JumpIf {
                        cond: inv_cond_reg,
                        target: 0, // placeholder
                    });

                    for (i, e) in then_body.iter().enumerate() {
                        let is_last = i == then_body.len() - 1;
                        if is_last {
                            self.compile_expr(e, dst);
                        } else {
                            let tmp = self.alloc_temp();
                            self.compile_expr(e, tmp);
                        }
                    }

                    // Patch the jump to point here
                    let end_pos = self.code.len() as Addr32;
                    if let Instruction::JumpIf { target, .. } = &mut self.code[jump_idx] {
                        *target = end_pos;
                    }
                } else {
                    // If with else:
                    // JumpIf cond to then_body
                    // <else body>
                    // Jump to end
                    // then_start:
                    // <then body>
                    // end:

                    let jump_to_then_idx = self.code.len();
                    self.emit(Instruction::JumpIf {
                        cond: cond_reg,
                        target: 0, // placeholder
                    });

                    // Else body
                    if let Some(else_b) = else_body {
                        for (i, e) in else_b.iter().enumerate() {
                            let is_last = i == else_b.len() - 1;
                            if is_last {
                                self.compile_expr(e, dst);
                            } else {
                                let tmp = self.alloc_temp();
                                self.compile_expr(e, tmp);
                            }
                        }
                    }

                    // Jump over then body
                    let jump_to_end_idx = self.code.len();
                    self.emit(Instruction::Jump { target: 0 }); // placeholder

                    // Patch jump_to_then to point here
                    let then_start = self.code.len() as Addr32;
                    if let Instruction::JumpIf { target, .. } = &mut self.code[jump_to_then_idx] {
                        *target = then_start;
                    }

                    // Then body
                    for (i, e) in then_body.iter().enumerate() {
                        let is_last = i == then_body.len() - 1;
                        if is_last {
                            self.compile_expr(e, dst);
                        } else {
                            let tmp = self.alloc_temp();
                            self.compile_expr(e, tmp);
                        }
                    }

                    // Patch jump_to_end to point here
                    let end_pos = self.code.len() as Addr32;
                    if let Instruction::Jump { target } = &mut self.code[jump_to_end_idx] {
                        *target = end_pos;
                    }
                }
            }
        }
    }
}

/// Compile a module AST to register-based bytecode.
pub fn compile_module(module: &Module) -> Result<CompiledModule, crate::error::CompileError> {
    // Count locals used in the module
    let num_locals = count_locals(module);

    let mut compiler = Compiler::new(num_locals);

    // Preload frequently used constants into dedicated registers
    compiler.preload_constants(module);

    // Compile each expression, keeping track of the last result register
    let mut last_dst = 0u8;
    for expr in &module.body {
        let dst = compiler.alloc_temp();
        compiler.compile_expr(expr, dst);
        last_dst = dst;
        compiler.free_temps();
    }

    // Move the last result to r0 so the VM can return it
    if !module.body.is_empty() && last_dst != 0 {
        compiler.emit(Instruction::Mov {
            dst: 0,
            src: last_dst,
        });
    }

    compiler.emit(Instruction::Halt);

    if compiler.pool_index_map.len() > ConstPoolId::MAX as usize {
        return Err(crate::error::CompileError::ConstantPoolOverflow {
            count: compiler.pool_index_map.len(),
        });
    }

    Ok(CompiledModule {
        types: module.types.clone(),
        functions: module.functions.clone(),
        instructions: compiler.code,
        imports: module.imports.clone(),
        constant_pool: compiler.pool_index_map.keys().copied().collect(),
    })
}

/// Count the maximum local index used in a module.
fn count_locals(module: &Module) -> u8 {
    let mut max_local = 0u8;
    for expr in &module.body {
        max_local = max_local.max(count_locals_expr(expr));
    }
    max_local.saturating_add(1) // +1 because indices are 0-based
}

fn count_locals_expr(expr: &Expr) -> u8 {
    match expr {
        Expr::LocalGet { index, .. }
        | Expr::LocalSet { index, .. }
        | Expr::LocalTee { index, .. } => *index as u8,
        Expr::BinaryOp { lhs, rhs, .. } => count_locals_expr(lhs).max(count_locals_expr(rhs)),
        Expr::UnaryOp { operand, .. } => count_locals_expr(operand),
        Expr::Block { body, .. } | Expr::Loop { body, .. } => {
            body.iter().map(count_locals_expr).max().unwrap_or(0)
        }
        Expr::BrIf { condition, .. } => count_locals_expr(condition),
        Expr::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            let mut m = count_locals_expr(condition);
            m = m.max(then_body.iter().map(count_locals_expr).max().unwrap_or(0));
            if let Some(eb) = else_body {
                m = m.max(eb.iter().map(count_locals_expr).max().unwrap_or(0));
            }
            m
        }
        Expr::Call { args, .. } => args.iter().map(count_locals_expr).max().unwrap_or(0),
        _ => 0,
    }
}

/// Create a binary operation instruction.
fn binary_op_instruction(opcode: Opcode, dst: u8, lhs: u8, rhs: u8) -> Instruction {
    match opcode {
        // i32
        Opcode::I32Add => Instruction::I32Add { dst, lhs, rhs },
        Opcode::I32Sub => Instruction::I32Sub { dst, lhs, rhs },
        Opcode::I32Mul => Instruction::I32Mul { dst, lhs, rhs },
        Opcode::I32DivS => Instruction::I32DivS { dst, lhs, rhs },
        Opcode::I32DivU => Instruction::I32DivU { dst, lhs, rhs },
        Opcode::I32RemS => Instruction::I32RemS { dst, lhs, rhs },
        Opcode::I32RemU => Instruction::I32RemU { dst, lhs, rhs },
        Opcode::I32And => Instruction::I32And { dst, lhs, rhs },
        Opcode::I32Or => Instruction::I32Or { dst, lhs, rhs },
        Opcode::I32Xor => Instruction::I32Xor { dst, lhs, rhs },
        Opcode::I32Shl => Instruction::I32Shl { dst, lhs, rhs },
        Opcode::I32ShrS => Instruction::I32ShrS { dst, lhs, rhs },
        Opcode::I32ShrU => Instruction::I32ShrU { dst, lhs, rhs },
        Opcode::I32Eq => Instruction::I32Eq { dst, lhs, rhs },
        Opcode::I32Ne => Instruction::I32Ne { dst, lhs, rhs },
        Opcode::I32LtS => Instruction::I32LtS { dst, lhs, rhs },
        Opcode::I32LtU => Instruction::I32LtU { dst, lhs, rhs },
        Opcode::I32GtS => Instruction::I32GtS { dst, lhs, rhs },
        Opcode::I32GtU => Instruction::I32GtU { dst, lhs, rhs },
        Opcode::I32LeS => Instruction::I32LeS { dst, lhs, rhs },
        Opcode::I32LeU => Instruction::I32LeU { dst, lhs, rhs },
        Opcode::I32GeS => Instruction::I32GeS { dst, lhs, rhs },
        Opcode::I32GeU => Instruction::I32GeU { dst, lhs, rhs },

        // i64
        Opcode::I64Add => Instruction::I64Add { dst, lhs, rhs },
        Opcode::I64Sub => Instruction::I64Sub { dst, lhs, rhs },
        Opcode::I64Mul => Instruction::I64Mul { dst, lhs, rhs },
        Opcode::I64DivS => Instruction::I64DivS { dst, lhs, rhs },
        Opcode::I64DivU => Instruction::I64DivU { dst, lhs, rhs },
        Opcode::I64RemS => Instruction::I64RemS { dst, lhs, rhs },
        Opcode::I64RemU => Instruction::I64RemU { dst, lhs, rhs },
        Opcode::I64And => Instruction::I64And { dst, lhs, rhs },
        Opcode::I64Or => Instruction::I64Or { dst, lhs, rhs },
        Opcode::I64Xor => Instruction::I64Xor { dst, lhs, rhs },
        Opcode::I64Shl => Instruction::I64Shl { dst, lhs, rhs },
        Opcode::I64ShrS => Instruction::I64ShrS { dst, lhs, rhs },
        Opcode::I64ShrU => Instruction::I64ShrU { dst, lhs, rhs },
        Opcode::I64Eq => Instruction::I64Eq { dst, lhs, rhs },
        Opcode::I64Ne => Instruction::I64Ne { dst, lhs, rhs },
        Opcode::I64LtS => Instruction::I64LtS { dst, lhs, rhs },
        Opcode::I64LtU => Instruction::I64LtU { dst, lhs, rhs },
        Opcode::I64GtS => Instruction::I64GtS { dst, lhs, rhs },
        Opcode::I64GtU => Instruction::I64GtU { dst, lhs, rhs },
        Opcode::I64LeS => Instruction::I64LeS { dst, lhs, rhs },
        Opcode::I64LeU => Instruction::I64LeU { dst, lhs, rhs },
        Opcode::I64GeS => Instruction::I64GeS { dst, lhs, rhs },
        Opcode::I64GeU => Instruction::I64GeU { dst, lhs, rhs },

        // f32
        Opcode::F32Add => Instruction::F32Add { dst, lhs, rhs },
        Opcode::F32Sub => Instruction::F32Sub { dst, lhs, rhs },
        Opcode::F32Mul => Instruction::F32Mul { dst, lhs, rhs },
        Opcode::F32Div => Instruction::F32Div { dst, lhs, rhs },
        Opcode::F32Min => Instruction::F32Min { dst, lhs, rhs },
        Opcode::F32Max => Instruction::F32Max { dst, lhs, rhs },
        Opcode::F32Eq => Instruction::F32Eq { dst, lhs, rhs },
        Opcode::F32Ne => Instruction::F32Ne { dst, lhs, rhs },
        Opcode::F32Lt => Instruction::F32Lt { dst, lhs, rhs },
        Opcode::F32Gt => Instruction::F32Gt { dst, lhs, rhs },
        Opcode::F32Le => Instruction::F32Le { dst, lhs, rhs },
        Opcode::F32Ge => Instruction::F32Ge { dst, lhs, rhs },

        // f64
        Opcode::F64Add => Instruction::F64Add { dst, lhs, rhs },
        Opcode::F64Sub => Instruction::F64Sub { dst, lhs, rhs },
        Opcode::F64Mul => Instruction::F64Mul { dst, lhs, rhs },
        Opcode::F64Div => Instruction::F64Div { dst, lhs, rhs },
        Opcode::F64Min => Instruction::F64Min { dst, lhs, rhs },
        Opcode::F64Max => Instruction::F64Max { dst, lhs, rhs },
        Opcode::F64Eq => Instruction::F64Eq { dst, lhs, rhs },
        Opcode::F64Ne => Instruction::F64Ne { dst, lhs, rhs },
        Opcode::F64Lt => Instruction::F64Lt { dst, lhs, rhs },
        Opcode::F64Gt => Instruction::F64Gt { dst, lhs, rhs },
        Opcode::F64Le => Instruction::F64Le { dst, lhs, rhs },
        Opcode::F64Ge => Instruction::F64Ge { dst, lhs, rhs },

        _ => panic!("Unsupported binary opcode: {:?}", opcode),
    }
}

/// Create a unary operation instruction.
fn unary_op_instruction(opcode: Opcode, dst: u8, src: u8) -> Instruction {
    match opcode {
        Opcode::F32Abs => Instruction::F32Abs { dst, src },
        Opcode::F32Neg => Instruction::F32Neg { dst, src },
        Opcode::F32Ceil => Instruction::F32Ceil { dst, src },
        Opcode::F32Floor => Instruction::F32Floor { dst, src },
        Opcode::F32Trunc => Instruction::F32Trunc { dst, src },
        Opcode::F32Nearest => Instruction::F32Nearest { dst, src },
        Opcode::F32Sqrt => Instruction::F32Sqrt { dst, src },
        Opcode::F64Abs => Instruction::F64Abs { dst, src },
        Opcode::F64Neg => Instruction::F64Neg { dst, src },
        Opcode::F64Ceil => Instruction::F64Ceil { dst, src },
        Opcode::F64Floor => Instruction::F64Floor { dst, src },
        Opcode::F64Trunc => Instruction::F64Trunc { dst, src },
        Opcode::F64Nearest => Instruction::F64Nearest { dst, src },
        Opcode::F64Sqrt => Instruction::F64Sqrt { dst, src },
        // Type conversions
        Opcode::I32WrapI64 => Instruction::I32WrapI64 { dst, src },
        Opcode::I64ExtendI32S => Instruction::I64ExtendI32S { dst, src },
        Opcode::I64ExtendI32U => Instruction::I64ExtendI32U { dst, src },
        Opcode::I32TruncF32S => Instruction::I32TruncF32S { dst, src },
        Opcode::I32TruncF32U => Instruction::I32TruncF32U { dst, src },
        Opcode::I32TruncF64S => Instruction::I32TruncF64S { dst, src },
        Opcode::I32TruncF64U => Instruction::I32TruncF64U { dst, src },
        Opcode::I64TruncF32S => Instruction::I64TruncF32S { dst, src },
        Opcode::I64TruncF32U => Instruction::I64TruncF32U { dst, src },
        Opcode::I64TruncF64S => Instruction::I64TruncF64S { dst, src },
        Opcode::I64TruncF64U => Instruction::I64TruncF64U { dst, src },
        Opcode::F32ConvertI32S => Instruction::F32ConvertI32S { dst, src },
        Opcode::F32ConvertI32U => Instruction::F32ConvertI32U { dst, src },
        Opcode::F32ConvertI64S => Instruction::F32ConvertI64S { dst, src },
        Opcode::F32ConvertI64U => Instruction::F32ConvertI64U { dst, src },
        Opcode::F32DemoteF64 => Instruction::F32DemoteF64 { dst, src },
        Opcode::F64ConvertI32S => Instruction::F64ConvertI32S { dst, src },
        Opcode::F64ConvertI32U => Instruction::F64ConvertI32U { dst, src },
        Opcode::F64ConvertI64S => Instruction::F64ConvertI64S { dst, src },
        Opcode::F64ConvertI64U => Instruction::F64ConvertI64U { dst, src },
        Opcode::F64PromoteF32 => Instruction::F64PromoteF32 { dst, src },
        Opcode::I32ReinterpretF32 => Instruction::I32ReinterpretF32 { dst, src },
        Opcode::I64ReinterpretF64 => Instruction::I64ReinterpretF64 { dst, src },
        Opcode::F32ReinterpretI32 => Instruction::F32ReinterpretI32 { dst, src },
        Opcode::F64ReinterpretI64 => Instruction::F64ReinterpretI64 { dst, src },
        _ => panic!("Unsupported unary opcode: {:?}", opcode),
    }
}

/// Collect all f64 constants from an expression and count their occurrences.
fn collect_constants(expr: &Expr, constants: &mut IndexMap<u64, usize>) {
    match expr {
        Expr::Const { value, .. } => {
            if let ConstValue::F64(v) = value {
                *constants.entry(v.to_bits()).or_insert(0) += 1;
            } else if let ConstValue::F32(v) = value {
                *constants.entry((*v as f64).to_bits()).or_insert(0) += 1;
            }
        }
        Expr::BinaryOp { lhs, rhs, .. } => {
            collect_constants(lhs, constants);
            collect_constants(rhs, constants);
        }
        Expr::UnaryOp { operand, .. } => {
            collect_constants(operand, constants);
        }
        Expr::LocalSet { value, .. } | Expr::LocalTee { value, .. } => {
            collect_constants(value, constants);
        }
        Expr::RegSet { value, .. } | Expr::RegTee { value, .. } => {
            collect_constants(value, constants);
        }
        Expr::Block { body, .. } | Expr::Loop { body, .. } => {
            for e in body {
                collect_constants(e, constants);
            }
        }
        Expr::BrIf { condition, .. } => {
            collect_constants(condition, constants);
        }
        Expr::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_constants(condition, constants);
            for e in then_body {
                collect_constants(e, constants);
            }
            if let Some(eb) = else_body {
                for e in eb {
                    collect_constants(e, constants);
                }
            }
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_constants(arg, constants);
            }
        }
        _ => {}
    }
}
