//! Register-based bytecode instruction format for the Stroop VM.

/// Index into the constant pool.
pub type ConstPoolId = u16;

/// Address (instruction index) for control flow.
pub type Addr32 = u32;

/// A single bytecode instruction with register operands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // Constants - load into register
    LoadConstI32 {
        dst: u8,
        value: i32,
    },
    /// Load i64 constant from pool: dst = constant_pool[index]
    LoadConstI64 {
        dst: u8,
        index: ConstPoolId,
    },
    LoadConstF32 {
        dst: u8,
        value: f32,
    },
    /// Load f64 constant from pool: dst = constant_pool[index]
    LoadConstF64 {
        dst: u8,
        index: ConstPoolId,
    },

    // Register move
    Mov {
        dst: u8,
        src: u8,
    },

    // i32 arithmetic: dst = lhs op rhs
    I32Add {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32Sub {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32Mul {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32DivS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32DivU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32RemS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32RemU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32And {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32Or {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32Xor {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32Shl {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32ShrS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32ShrU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // i32 comparison: dst = (lhs cmp rhs) ? 1 : 0
    I32Eq {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32Ne {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32LtS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32LtU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32GtS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32GtU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32LeS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32LeU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32GeS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I32GeU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // i64 arithmetic
    I64Add {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64Sub {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64Mul {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64DivS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64DivU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64RemS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64RemU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64And {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64Or {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64Xor {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64Shl {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64ShrS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64ShrU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // i64 comparison
    I64Eq {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64Ne {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64LtS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64LtU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64GtS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64GtU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64LeS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64LeU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64GeS {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    I64GeU {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // f32 arithmetic
    F32Add {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Sub {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Mul {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Div {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Min {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Max {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // f32 unary: dst = op(src)
    F32Abs {
        dst: u8,
        src: u8,
    },
    F32Neg {
        dst: u8,
        src: u8,
    },
    F32Ceil {
        dst: u8,
        src: u8,
    },
    F32Floor {
        dst: u8,
        src: u8,
    },
    F32Trunc {
        dst: u8,
        src: u8,
    },
    F32Nearest {
        dst: u8,
        src: u8,
    },
    F32Sqrt {
        dst: u8,
        src: u8,
    },

    // f32 comparison
    F32Eq {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Ne {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Lt {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Gt {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Le {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F32Ge {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // f64 arithmetic
    F64Add {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Sub {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Mul {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Div {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Min {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Max {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // f64 unary
    F64Abs {
        dst: u8,
        src: u8,
    },
    F64Neg {
        dst: u8,
        src: u8,
    },
    F64Ceil {
        dst: u8,
        src: u8,
    },
    F64Floor {
        dst: u8,
        src: u8,
    },
    F64Trunc {
        dst: u8,
        src: u8,
    },
    F64Nearest {
        dst: u8,
        src: u8,
    },
    F64Sqrt {
        dst: u8,
        src: u8,
    },

    // f64 comparison
    F64Eq {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Ne {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Lt {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Gt {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Le {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },
    F64Ge {
        dst: u8,
        lhs: u8,
        rhs: u8,
    },

    // Control flow
    Block {
        end: Addr32,
    },
    Loop {
        start: Addr32,
    },
    Br {
        depth: u32,
    },
    BrIf {
        cond: u8,
        depth: u32,
    },
    End,

    // Function calls
    // Args are in registers base..base+argc, result goes to dst
    Call {
        func: u32,
        base: u8,
        argc: u8,
        dst: u8,
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
