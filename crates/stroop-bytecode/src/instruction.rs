//! Register-based bytecode instruction format for the Stroop VM.

/// Register index (0-255).
pub type Reg8 = u8;

/// Index into the constant pool.
pub type ConstPoolId = u16;

/// Address (instruction index) for control flow.
pub type Addr32 = u32;

/// A single bytecode instruction with register operands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // Constants - load into register
    LoadConstI32 {
        dst: Reg8,
        value: i32,
    },
    /// Load i64 constant from pool: dst = constant_pool[index]
    LoadConstI64 {
        dst: Reg8,
        index: ConstPoolId,
    },
    LoadConstF32 {
        dst: Reg8,
        value: f32,
    },
    /// Load f64 constant from pool: dst = constant_pool[index]
    LoadConstF64 {
        dst: Reg8,
        index: ConstPoolId,
    },

    // Register move
    Mov {
        dst: Reg8,
        src: Reg8,
    },

    // i32 arithmetic: dst = lhs op rhs
    I32Add {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32Sub {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32Mul {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32DivS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32DivU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32RemS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32RemU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32And {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32Or {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32Xor {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32Shl {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32ShrS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32ShrU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // i32 comparison: dst = (lhs cmp rhs) ? 1 : 0
    I32Eq {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32Ne {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32LtS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32LtU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32GtS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32GtU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32LeS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32LeU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32GeS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I32GeU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // i64 arithmetic
    I64Add {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64Sub {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64Mul {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64DivS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64DivU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64RemS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64RemU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64And {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64Or {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64Xor {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64Shl {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64ShrS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64ShrU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // i64 comparison
    I64Eq {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64Ne {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64LtS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64LtU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64GtS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64GtU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64LeS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64LeU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64GeS {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    I64GeU {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // f32 arithmetic
    F32Add {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Sub {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Mul {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Div {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Min {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Max {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // f32 unary: dst = op(src)
    F32Abs {
        dst: Reg8,
        src: Reg8,
    },
    F32Neg {
        dst: Reg8,
        src: Reg8,
    },
    F32Ceil {
        dst: Reg8,
        src: Reg8,
    },
    F32Floor {
        dst: Reg8,
        src: Reg8,
    },
    F32Trunc {
        dst: Reg8,
        src: Reg8,
    },
    F32Nearest {
        dst: Reg8,
        src: Reg8,
    },
    F32Sqrt {
        dst: Reg8,
        src: Reg8,
    },

    // f32 comparison
    F32Eq {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Ne {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Lt {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Gt {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Le {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F32Ge {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // f64 arithmetic
    F64Add {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Sub {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Mul {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Div {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Min {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Max {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // f64 unary
    F64Abs {
        dst: Reg8,
        src: Reg8,
    },
    F64Neg {
        dst: Reg8,
        src: Reg8,
    },
    F64Ceil {
        dst: Reg8,
        src: Reg8,
    },
    F64Floor {
        dst: Reg8,
        src: Reg8,
    },
    F64Trunc {
        dst: Reg8,
        src: Reg8,
    },
    F64Nearest {
        dst: Reg8,
        src: Reg8,
    },
    F64Sqrt {
        dst: Reg8,
        src: Reg8,
    },

    // f64 comparison
    F64Eq {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Ne {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Lt {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Gt {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Le {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },
    F64Ge {
        dst: Reg8,
        lhs: Reg8,
        rhs: Reg8,
    },

    // Type conversions: integer wrap/extend
    I32WrapI64 {
        dst: Reg8,
        src: Reg8,
    },
    I64ExtendI32S {
        dst: Reg8,
        src: Reg8,
    },
    I64ExtendI32U {
        dst: Reg8,
        src: Reg8,
    },

    // Type conversions: float demote/promote
    F32DemoteF64 {
        dst: Reg8,
        src: Reg8,
    },
    F64PromoteF32 {
        dst: Reg8,
        src: Reg8,
    },

    // Type conversions: float to integer (truncate)
    I32TruncF32S {
        dst: Reg8,
        src: Reg8,
    },
    I32TruncF32U {
        dst: Reg8,
        src: Reg8,
    },
    I32TruncF64S {
        dst: Reg8,
        src: Reg8,
    },
    I32TruncF64U {
        dst: Reg8,
        src: Reg8,
    },
    I64TruncF32S {
        dst: Reg8,
        src: Reg8,
    },
    I64TruncF32U {
        dst: Reg8,
        src: Reg8,
    },
    I64TruncF64S {
        dst: Reg8,
        src: Reg8,
    },
    I64TruncF64U {
        dst: Reg8,
        src: Reg8,
    },

    // Type conversions: integer to float (convert)
    F32ConvertI32S {
        dst: Reg8,
        src: Reg8,
    },
    F32ConvertI32U {
        dst: Reg8,
        src: Reg8,
    },
    F32ConvertI64S {
        dst: Reg8,
        src: Reg8,
    },
    F32ConvertI64U {
        dst: Reg8,
        src: Reg8,
    },
    F64ConvertI32S {
        dst: Reg8,
        src: Reg8,
    },
    F64ConvertI32U {
        dst: Reg8,
        src: Reg8,
    },
    F64ConvertI64S {
        dst: Reg8,
        src: Reg8,
    },
    F64ConvertI64U {
        dst: Reg8,
        src: Reg8,
    },

    // Type conversions: reinterpret (bitcast)
    I32ReinterpretF32 {
        dst: Reg8,
        src: Reg8,
    },
    I64ReinterpretF64 {
        dst: Reg8,
        src: Reg8,
    },
    F32ReinterpretI32 {
        dst: Reg8,
        src: Reg8,
    },
    F64ReinterpretI64 {
        dst: Reg8,
        src: Reg8,
    },

    // Control flow
    /// Unconditional jump to target address
    Jump {
        target: Addr32,
    },
    /// Conditional jump: if reg[cond] != 0, jump to target
    JumpIf {
        cond: Reg8,
        target: Addr32,
    },

    // Function calls
    // Args are in registers base..base+argc, result goes to dst
    Call {
        func: u32,
        base: Reg8,
        argc: u8,
        dst: Reg8,
    },

    Halt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_size() {
        assert_eq!(std::mem::size_of::<Instruction>(), 8);
    }
}
