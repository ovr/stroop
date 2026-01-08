use stroop_vm_bytecode::ValueType;

/// Single-byte opcodes for the bytecode instruction set.
/// Follows WebAssembly encoding conventions where applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    // === Control Flow (0x02-0x10) ===
    Block = 0x02,
    Loop = 0x03,
    If = 0x04,
    Br = 0x0C,
    BrIf = 0x0D,
    Call = 0x10,

    // === Local Variable Operations (0x20-0x22) ===
    LocalGet = 0x20,
    LocalSet = 0x21,
    LocalTee = 0x22,

    // === Register Operations (0x23-0x25) ===
    RegGet = 0x23,
    RegSet = 0x24,
    RegTee = 0x25,

    // === Constants (0x41-0x44) ===
    I32Const = 0x41,
    I64Const = 0x42,
    F32Const = 0x43,
    F64Const = 0x44,

    // === i32 Comparison (0x46-0x4F) ===
    I32Eq = 0x46,
    I32Ne = 0x47,
    I32LtS = 0x48,
    I32LtU = 0x49,
    I32GtS = 0x4A,
    I32GtU = 0x4B,
    I32LeS = 0x4C,
    I32LeU = 0x4D,
    I32GeS = 0x4E,
    I32GeU = 0x4F,

    // === i64 Comparison (0x51-0x5A) ===
    I64Eq = 0x51,
    I64Ne = 0x52,
    I64LtS = 0x53,
    I64LtU = 0x54,
    I64GtS = 0x55,
    I64GtU = 0x56,
    I64LeS = 0x57,
    I64LeU = 0x58,
    I64GeS = 0x59,
    I64GeU = 0x5A,

    // === f32 Comparison (0x5B-0x60) ===
    F32Eq = 0x5B,
    F32Ne = 0x5C,
    F32Lt = 0x5D,
    F32Gt = 0x5E,
    F32Le = 0x5F,
    F32Ge = 0x60,

    // === f64 Comparison (0x61-0x66) ===
    F64Eq = 0x61,
    F64Ne = 0x62,
    F64Lt = 0x63,
    F64Gt = 0x64,
    F64Le = 0x65,
    F64Ge = 0x66,

    // === i32 Arithmetic (0x6A-0x78) ===
    I32Add = 0x6A,
    I32Sub = 0x6B,
    I32Mul = 0x6C,
    I32DivS = 0x6D,
    I32DivU = 0x6E,
    I32RemS = 0x6F,
    I32RemU = 0x70,
    I32And = 0x71,
    I32Or = 0x72,
    I32Xor = 0x73,
    I32Shl = 0x74,
    I32ShrS = 0x75,
    I32ShrU = 0x76,

    // === i64 Arithmetic (0x7C-0x8A) ===
    I64Add = 0x7C,
    I64Sub = 0x7D,
    I64Mul = 0x7E,
    I64DivS = 0x7F,
    I64DivU = 0x80,
    I64RemS = 0x81,
    I64RemU = 0x82,
    I64And = 0x83,
    I64Or = 0x84,
    I64Xor = 0x85,
    I64Shl = 0x86,
    I64ShrS = 0x87,
    I64ShrU = 0x88,

    // === f32 Arithmetic (0x8B-0x98) ===
    F32Abs = 0x8B,
    F32Neg = 0x8C,
    F32Ceil = 0x8D,
    F32Floor = 0x8E,
    F32Trunc = 0x8F,
    F32Nearest = 0x90,
    F32Sqrt = 0x91,
    F32Add = 0x92,
    F32Sub = 0x93,
    F32Mul = 0x94,
    F32Div = 0x95,
    F32Min = 0x96,
    F32Max = 0x97,

    // === f64 Arithmetic (0x99-0xA6) ===
    F64Abs = 0x99,
    F64Neg = 0x9A,
    F64Ceil = 0x9B,
    F64Floor = 0x9C,
    F64Trunc = 0x9D,
    F64Nearest = 0x9E,
    F64Sqrt = 0x9F,
    F64Add = 0xA0,
    F64Sub = 0xA1,
    F64Mul = 0xA2,
    F64Div = 0xA3,
    F64Min = 0xA4,
    F64Max = 0xA5,

    // === Type Conversions (0xA7-0xBF) ===
    I32WrapI64 = 0xA7,
    I64ExtendI32S = 0xAC,
    I64ExtendI32U = 0xAD,
    I32TruncF32S = 0xA8,
    I32TruncF32U = 0xA9,
    I32TruncF64S = 0xAA,
    I32TruncF64U = 0xAB,
    I64TruncF32S = 0xAE,
    I64TruncF32U = 0xAF,
    I64TruncF64S = 0xB0,
    I64TruncF64U = 0xB1,
    F32ConvertI32S = 0xB2,
    F32ConvertI32U = 0xB3,
    F32ConvertI64S = 0xB4,
    F32ConvertI64U = 0xB5,
    F32DemoteF64 = 0xB6,
    F64ConvertI32S = 0xB7,
    F64ConvertI32U = 0xB8,
    F64ConvertI64S = 0xB9,
    F64ConvertI64U = 0xBA,
    F64PromoteF32 = 0xBB,
    I32ReinterpretF32 = 0xBC,
    I64ReinterpretF64 = 0xBD,
    F32ReinterpretI32 = 0xBE,
    F64ReinterpretI64 = 0xBF,
}

impl Opcode {
    /// Parse a mnemonic string into an opcode.
    pub fn from_mnemonic(s: &str) -> Option<Self> {
        match s {
            // Control flow
            "block" => Some(Opcode::Block),
            "loop" => Some(Opcode::Loop),
            "if" => Some(Opcode::If),
            "br" => Some(Opcode::Br),
            "br_if" => Some(Opcode::BrIf),
            "call" => Some(Opcode::Call),

            // Local variables
            "local.get" => Some(Opcode::LocalGet),
            "local.set" => Some(Opcode::LocalSet),
            "local.tee" => Some(Opcode::LocalTee),

            // Registers
            "reg.get" => Some(Opcode::RegGet),
            "reg.set" => Some(Opcode::RegSet),
            "reg.tee" => Some(Opcode::RegTee),

            // Constants
            "i32.const" => Some(Opcode::I32Const),
            "i64.const" => Some(Opcode::I64Const),
            "f32.const" => Some(Opcode::F32Const),
            "f64.const" => Some(Opcode::F64Const),

            // i32 comparison
            "i32.eq" => Some(Opcode::I32Eq),
            "i32.ne" => Some(Opcode::I32Ne),
            "i32.lt_s" => Some(Opcode::I32LtS),
            "i32.lt_u" => Some(Opcode::I32LtU),
            "i32.gt_s" => Some(Opcode::I32GtS),
            "i32.gt_u" => Some(Opcode::I32GtU),
            "i32.le_s" => Some(Opcode::I32LeS),
            "i32.le_u" => Some(Opcode::I32LeU),
            "i32.ge_s" => Some(Opcode::I32GeS),
            "i32.ge_u" => Some(Opcode::I32GeU),

            // i64 comparison
            "i64.eq" => Some(Opcode::I64Eq),
            "i64.ne" => Some(Opcode::I64Ne),
            "i64.lt_s" => Some(Opcode::I64LtS),
            "i64.lt_u" => Some(Opcode::I64LtU),
            "i64.gt_s" => Some(Opcode::I64GtS),
            "i64.gt_u" => Some(Opcode::I64GtU),
            "i64.le_s" => Some(Opcode::I64LeS),
            "i64.le_u" => Some(Opcode::I64LeU),
            "i64.ge_s" => Some(Opcode::I64GeS),
            "i64.ge_u" => Some(Opcode::I64GeU),

            // f32 comparison
            "f32.eq" => Some(Opcode::F32Eq),
            "f32.ne" => Some(Opcode::F32Ne),
            "f32.lt" => Some(Opcode::F32Lt),
            "f32.gt" => Some(Opcode::F32Gt),
            "f32.le" => Some(Opcode::F32Le),
            "f32.ge" => Some(Opcode::F32Ge),

            // f64 comparison
            "f64.eq" => Some(Opcode::F64Eq),
            "f64.ne" => Some(Opcode::F64Ne),
            "f64.lt" => Some(Opcode::F64Lt),
            "f64.gt" => Some(Opcode::F64Gt),
            "f64.le" => Some(Opcode::F64Le),
            "f64.ge" => Some(Opcode::F64Ge),

            // i32 arithmetic
            "i32.add" => Some(Opcode::I32Add),
            "i32.sub" => Some(Opcode::I32Sub),
            "i32.mul" => Some(Opcode::I32Mul),
            "i32.div_s" => Some(Opcode::I32DivS),
            "i32.div_u" => Some(Opcode::I32DivU),
            "i32.rem_s" => Some(Opcode::I32RemS),
            "i32.rem_u" => Some(Opcode::I32RemU),
            "i32.and" => Some(Opcode::I32And),
            "i32.or" => Some(Opcode::I32Or),
            "i32.xor" => Some(Opcode::I32Xor),
            "i32.shl" => Some(Opcode::I32Shl),
            "i32.shr_s" => Some(Opcode::I32ShrS),
            "i32.shr_u" => Some(Opcode::I32ShrU),

            // i64 arithmetic
            "i64.add" => Some(Opcode::I64Add),
            "i64.sub" => Some(Opcode::I64Sub),
            "i64.mul" => Some(Opcode::I64Mul),
            "i64.div_s" => Some(Opcode::I64DivS),
            "i64.div_u" => Some(Opcode::I64DivU),
            "i64.rem_s" => Some(Opcode::I64RemS),
            "i64.rem_u" => Some(Opcode::I64RemU),
            "i64.and" => Some(Opcode::I64And),
            "i64.or" => Some(Opcode::I64Or),
            "i64.xor" => Some(Opcode::I64Xor),
            "i64.shl" => Some(Opcode::I64Shl),
            "i64.shr_s" => Some(Opcode::I64ShrS),
            "i64.shr_u" => Some(Opcode::I64ShrU),

            // f32 arithmetic
            "f32.abs" => Some(Opcode::F32Abs),
            "f32.neg" => Some(Opcode::F32Neg),
            "f32.ceil" => Some(Opcode::F32Ceil),
            "f32.floor" => Some(Opcode::F32Floor),
            "f32.trunc" => Some(Opcode::F32Trunc),
            "f32.nearest" => Some(Opcode::F32Nearest),
            "f32.sqrt" => Some(Opcode::F32Sqrt),
            "f32.add" => Some(Opcode::F32Add),
            "f32.sub" => Some(Opcode::F32Sub),
            "f32.mul" => Some(Opcode::F32Mul),
            "f32.div" => Some(Opcode::F32Div),
            "f32.min" => Some(Opcode::F32Min),
            "f32.max" => Some(Opcode::F32Max),

            // f64 arithmetic
            "f64.abs" => Some(Opcode::F64Abs),
            "f64.neg" => Some(Opcode::F64Neg),
            "f64.ceil" => Some(Opcode::F64Ceil),
            "f64.floor" => Some(Opcode::F64Floor),
            "f64.trunc" => Some(Opcode::F64Trunc),
            "f64.nearest" => Some(Opcode::F64Nearest),
            "f64.sqrt" => Some(Opcode::F64Sqrt),
            "f64.add" => Some(Opcode::F64Add),
            "f64.sub" => Some(Opcode::F64Sub),
            "f64.mul" => Some(Opcode::F64Mul),
            "f64.div" => Some(Opcode::F64Div),
            "f64.min" => Some(Opcode::F64Min),
            "f64.max" => Some(Opcode::F64Max),

            // Type conversions
            "i32.wrap_i64" => Some(Opcode::I32WrapI64),
            "i64.extend_i32_s" => Some(Opcode::I64ExtendI32S),
            "i64.extend_i32_u" => Some(Opcode::I64ExtendI32U),
            "i32.trunc_f32_s" => Some(Opcode::I32TruncF32S),
            "i32.trunc_f32_u" => Some(Opcode::I32TruncF32U),
            "i32.trunc_f64_s" => Some(Opcode::I32TruncF64S),
            "i32.trunc_f64_u" => Some(Opcode::I32TruncF64U),
            "i64.trunc_f32_s" => Some(Opcode::I64TruncF32S),
            "i64.trunc_f32_u" => Some(Opcode::I64TruncF32U),
            "i64.trunc_f64_s" => Some(Opcode::I64TruncF64S),
            "i64.trunc_f64_u" => Some(Opcode::I64TruncF64U),
            "f32.convert_i32_s" => Some(Opcode::F32ConvertI32S),
            "f32.convert_i32_u" => Some(Opcode::F32ConvertI32U),
            "f32.convert_i64_s" => Some(Opcode::F32ConvertI64S),
            "f32.convert_i64_u" => Some(Opcode::F32ConvertI64U),
            "f32.demote_f64" => Some(Opcode::F32DemoteF64),
            "f64.convert_i32_s" => Some(Opcode::F64ConvertI32S),
            "f64.convert_i32_u" => Some(Opcode::F64ConvertI32U),
            "f64.convert_i64_s" => Some(Opcode::F64ConvertI64S),
            "f64.convert_i64_u" => Some(Opcode::F64ConvertI64U),
            "f64.promote_f32" => Some(Opcode::F64PromoteF32),
            "i32.reinterpret_f32" => Some(Opcode::I32ReinterpretF32),
            "i64.reinterpret_f64" => Some(Opcode::I64ReinterpretF64),
            "f32.reinterpret_i32" => Some(Opcode::F32ReinterpretI32),
            "f64.reinterpret_i64" => Some(Opcode::F64ReinterpretI64),

            _ => None,
        }
    }

    /// Get the mnemonic string for this opcode.
    pub const fn mnemonic(self) -> &'static str {
        match self {
            // Control flow
            Opcode::Block => "block",
            Opcode::Loop => "loop",
            Opcode::If => "if",
            Opcode::Br => "br",
            Opcode::BrIf => "br_if",
            Opcode::Call => "call",

            // Local variables
            Opcode::LocalGet => "local.get",
            Opcode::LocalSet => "local.set",
            Opcode::LocalTee => "local.tee",

            // Registers
            Opcode::RegGet => "reg.get",
            Opcode::RegSet => "reg.set",
            Opcode::RegTee => "reg.tee",

            // Constants
            Opcode::I32Const => "i32.const",
            Opcode::I64Const => "i64.const",
            Opcode::F32Const => "f32.const",
            Opcode::F64Const => "f64.const",

            // i32 comparison
            Opcode::I32Eq => "i32.eq",
            Opcode::I32Ne => "i32.ne",
            Opcode::I32LtS => "i32.lt_s",
            Opcode::I32LtU => "i32.lt_u",
            Opcode::I32GtS => "i32.gt_s",
            Opcode::I32GtU => "i32.gt_u",
            Opcode::I32LeS => "i32.le_s",
            Opcode::I32LeU => "i32.le_u",
            Opcode::I32GeS => "i32.ge_s",
            Opcode::I32GeU => "i32.ge_u",

            // i64 comparison
            Opcode::I64Eq => "i64.eq",
            Opcode::I64Ne => "i64.ne",
            Opcode::I64LtS => "i64.lt_s",
            Opcode::I64LtU => "i64.lt_u",
            Opcode::I64GtS => "i64.gt_s",
            Opcode::I64GtU => "i64.gt_u",
            Opcode::I64LeS => "i64.le_s",
            Opcode::I64LeU => "i64.le_u",
            Opcode::I64GeS => "i64.ge_s",
            Opcode::I64GeU => "i64.ge_u",

            // f32 comparison
            Opcode::F32Eq => "f32.eq",
            Opcode::F32Ne => "f32.ne",
            Opcode::F32Lt => "f32.lt",
            Opcode::F32Gt => "f32.gt",
            Opcode::F32Le => "f32.le",
            Opcode::F32Ge => "f32.ge",

            // f64 comparison
            Opcode::F64Eq => "f64.eq",
            Opcode::F64Ne => "f64.ne",
            Opcode::F64Lt => "f64.lt",
            Opcode::F64Gt => "f64.gt",
            Opcode::F64Le => "f64.le",
            Opcode::F64Ge => "f64.ge",

            // i32 arithmetic
            Opcode::I32Add => "i32.add",
            Opcode::I32Sub => "i32.sub",
            Opcode::I32Mul => "i32.mul",
            Opcode::I32DivS => "i32.div_s",
            Opcode::I32DivU => "i32.div_u",
            Opcode::I32RemS => "i32.rem_s",
            Opcode::I32RemU => "i32.rem_u",
            Opcode::I32And => "i32.and",
            Opcode::I32Or => "i32.or",
            Opcode::I32Xor => "i32.xor",
            Opcode::I32Shl => "i32.shl",
            Opcode::I32ShrS => "i32.shr_s",
            Opcode::I32ShrU => "i32.shr_u",

            // i64 arithmetic
            Opcode::I64Add => "i64.add",
            Opcode::I64Sub => "i64.sub",
            Opcode::I64Mul => "i64.mul",
            Opcode::I64DivS => "i64.div_s",
            Opcode::I64DivU => "i64.div_u",
            Opcode::I64RemS => "i64.rem_s",
            Opcode::I64RemU => "i64.rem_u",
            Opcode::I64And => "i64.and",
            Opcode::I64Or => "i64.or",
            Opcode::I64Xor => "i64.xor",
            Opcode::I64Shl => "i64.shl",
            Opcode::I64ShrS => "i64.shr_s",
            Opcode::I64ShrU => "i64.shr_u",

            // f32 arithmetic
            Opcode::F32Abs => "f32.abs",
            Opcode::F32Neg => "f32.neg",
            Opcode::F32Ceil => "f32.ceil",
            Opcode::F32Floor => "f32.floor",
            Opcode::F32Trunc => "f32.trunc",
            Opcode::F32Nearest => "f32.nearest",
            Opcode::F32Sqrt => "f32.sqrt",
            Opcode::F32Add => "f32.add",
            Opcode::F32Sub => "f32.sub",
            Opcode::F32Mul => "f32.mul",
            Opcode::F32Div => "f32.div",
            Opcode::F32Min => "f32.min",
            Opcode::F32Max => "f32.max",

            // f64 arithmetic
            Opcode::F64Abs => "f64.abs",
            Opcode::F64Neg => "f64.neg",
            Opcode::F64Ceil => "f64.ceil",
            Opcode::F64Floor => "f64.floor",
            Opcode::F64Trunc => "f64.trunc",
            Opcode::F64Nearest => "f64.nearest",
            Opcode::F64Sqrt => "f64.sqrt",
            Opcode::F64Add => "f64.add",
            Opcode::F64Sub => "f64.sub",
            Opcode::F64Mul => "f64.mul",
            Opcode::F64Div => "f64.div",
            Opcode::F64Min => "f64.min",
            Opcode::F64Max => "f64.max",

            // Type conversions
            Opcode::I32WrapI64 => "i32.wrap_i64",
            Opcode::I64ExtendI32S => "i64.extend_i32_s",
            Opcode::I64ExtendI32U => "i64.extend_i32_u",
            Opcode::I32TruncF32S => "i32.trunc_f32_s",
            Opcode::I32TruncF32U => "i32.trunc_f32_u",
            Opcode::I32TruncF64S => "i32.trunc_f64_s",
            Opcode::I32TruncF64U => "i32.trunc_f64_u",
            Opcode::I64TruncF32S => "i64.trunc_f32_s",
            Opcode::I64TruncF32U => "i64.trunc_f32_u",
            Opcode::I64TruncF64S => "i64.trunc_f64_s",
            Opcode::I64TruncF64U => "i64.trunc_f64_u",
            Opcode::F32ConvertI32S => "f32.convert_i32_s",
            Opcode::F32ConvertI32U => "f32.convert_i32_u",
            Opcode::F32ConvertI64S => "f32.convert_i64_s",
            Opcode::F32ConvertI64U => "f32.convert_i64_u",
            Opcode::F32DemoteF64 => "f32.demote_f64",
            Opcode::F64ConvertI32S => "f64.convert_i32_s",
            Opcode::F64ConvertI32U => "f64.convert_i32_u",
            Opcode::F64ConvertI64S => "f64.convert_i64_s",
            Opcode::F64ConvertI64U => "f64.convert_i64_u",
            Opcode::F64PromoteF32 => "f64.promote_f32",
            Opcode::I32ReinterpretF32 => "i32.reinterpret_f32",
            Opcode::I64ReinterpretF64 => "i64.reinterpret_f64",
            Opcode::F32ReinterpretI32 => "f32.reinterpret_i32",
            Opcode::F64ReinterpretI64 => "f64.reinterpret_i64",
        }
    }

    /// Returns the number of operands this instruction consumes from the stack.
    /// Note: Call returns 0 here as its arity depends on the function signature.
    pub const fn stack_operands(self) -> usize {
        match self {
            // Constants push, don't consume
            Opcode::I32Const | Opcode::I64Const | Opcode::F32Const | Opcode::F64Const => 0,

            // LocalGet/RegGet push without consuming
            Opcode::LocalGet | Opcode::RegGet => 0,

            // Control flow - operand count depends on structure
            Opcode::Block | Opcode::Loop | Opcode::Call => 0,
            Opcode::Br => 0,
            Opcode::BrIf | Opcode::If => 1, // condition

            // Unary operations consume 1
            Opcode::F32Abs
            | Opcode::F32Neg
            | Opcode::F32Ceil
            | Opcode::F32Floor
            | Opcode::F32Trunc
            | Opcode::F32Nearest
            | Opcode::F32Sqrt
            | Opcode::F64Abs
            | Opcode::F64Neg
            | Opcode::F64Ceil
            | Opcode::F64Floor
            | Opcode::F64Trunc
            | Opcode::F64Nearest
            | Opcode::F64Sqrt
            // Type conversions
            | Opcode::I32WrapI64
            | Opcode::I64ExtendI32S
            | Opcode::I64ExtendI32U
            | Opcode::I32TruncF32S
            | Opcode::I32TruncF32U
            | Opcode::I32TruncF64S
            | Opcode::I32TruncF64U
            | Opcode::I64TruncF32S
            | Opcode::I64TruncF32U
            | Opcode::I64TruncF64S
            | Opcode::I64TruncF64U
            | Opcode::F32ConvertI32S
            | Opcode::F32ConvertI32U
            | Opcode::F32ConvertI64S
            | Opcode::F32ConvertI64U
            | Opcode::F32DemoteF64
            | Opcode::F64ConvertI32S
            | Opcode::F64ConvertI32U
            | Opcode::F64ConvertI64S
            | Opcode::F64ConvertI64U
            | Opcode::F64PromoteF32
            | Opcode::I32ReinterpretF32
            | Opcode::I64ReinterpretF64
            | Opcode::F32ReinterpretI32
            | Opcode::F64ReinterpretI64 => 1,

            // LocalSet/LocalTee/RegSet/RegTee consume one value
            Opcode::LocalSet | Opcode::LocalTee | Opcode::RegSet | Opcode::RegTee => 1,

            // Binary operations consume 2
            _ => 2,
        }
    }

    /// Returns the result type of this opcode, if any.
    pub const fn result_type(self) -> Option<ValueType> {
        match self {
            // i32 operations
            Opcode::I32Const
            | Opcode::I32Add
            | Opcode::I32Sub
            | Opcode::I32Mul
            | Opcode::I32DivS
            | Opcode::I32DivU
            | Opcode::I32RemS
            | Opcode::I32RemU
            | Opcode::I32And
            | Opcode::I32Or
            | Opcode::I32Xor
            | Opcode::I32Shl
            | Opcode::I32ShrS
            | Opcode::I32ShrU
            | Opcode::I32WrapI64
            | Opcode::I32TruncF32S
            | Opcode::I32TruncF32U
            | Opcode::I32TruncF64S
            | Opcode::I32TruncF64U
            | Opcode::I32ReinterpretF32 => Some(ValueType::I32),

            // i64 operations
            Opcode::I64Const
            | Opcode::I64Add
            | Opcode::I64Sub
            | Opcode::I64Mul
            | Opcode::I64DivS
            | Opcode::I64DivU
            | Opcode::I64RemS
            | Opcode::I64RemU
            | Opcode::I64And
            | Opcode::I64Or
            | Opcode::I64Xor
            | Opcode::I64Shl
            | Opcode::I64ShrS
            | Opcode::I64ShrU
            | Opcode::I64ExtendI32S
            | Opcode::I64ExtendI32U
            | Opcode::I64TruncF32S
            | Opcode::I64TruncF32U
            | Opcode::I64TruncF64S
            | Opcode::I64TruncF64U
            | Opcode::I64ReinterpretF64 => Some(ValueType::I64),

            // f32 operations
            Opcode::F32Const
            | Opcode::F32Abs
            | Opcode::F32Neg
            | Opcode::F32Ceil
            | Opcode::F32Floor
            | Opcode::F32Trunc
            | Opcode::F32Nearest
            | Opcode::F32Sqrt
            | Opcode::F32Add
            | Opcode::F32Sub
            | Opcode::F32Mul
            | Opcode::F32Div
            | Opcode::F32Min
            | Opcode::F32Max
            | Opcode::F32ConvertI32S
            | Opcode::F32ConvertI32U
            | Opcode::F32ConvertI64S
            | Opcode::F32ConvertI64U
            | Opcode::F32DemoteF64
            | Opcode::F32ReinterpretI32 => Some(ValueType::F32),

            // f64 operations
            Opcode::F64Const
            | Opcode::F64Abs
            | Opcode::F64Neg
            | Opcode::F64Ceil
            | Opcode::F64Floor
            | Opcode::F64Trunc
            | Opcode::F64Nearest
            | Opcode::F64Sqrt
            | Opcode::F64Add
            | Opcode::F64Sub
            | Opcode::F64Mul
            | Opcode::F64Div
            | Opcode::F64Min
            | Opcode::F64Max
            | Opcode::F64ConvertI32S
            | Opcode::F64ConvertI32U
            | Opcode::F64ConvertI64S
            | Opcode::F64ConvertI64U
            | Opcode::F64PromoteF32
            | Opcode::F64ReinterpretI64 => Some(ValueType::F64),

            // Comparison operations return i32 (boolean)
            Opcode::I32Eq
            | Opcode::I32Ne
            | Opcode::I32LtS
            | Opcode::I32LtU
            | Opcode::I32GtS
            | Opcode::I32GtU
            | Opcode::I32LeS
            | Opcode::I32LeU
            | Opcode::I32GeS
            | Opcode::I32GeU
            | Opcode::I64Eq
            | Opcode::I64Ne
            | Opcode::I64LtS
            | Opcode::I64LtU
            | Opcode::I64GtS
            | Opcode::I64GtU
            | Opcode::I64LeS
            | Opcode::I64LeU
            | Opcode::I64GeS
            | Opcode::I64GeU
            | Opcode::F32Eq
            | Opcode::F32Ne
            | Opcode::F32Lt
            | Opcode::F32Gt
            | Opcode::F32Le
            | Opcode::F32Ge
            | Opcode::F64Eq
            | Opcode::F64Ne
            | Opcode::F64Lt
            | Opcode::F64Gt
            | Opcode::F64Le
            | Opcode::F64Ge => Some(ValueType::I32),

            // LocalGet/RegGet result depends on the variable's type (unknown here)
            Opcode::LocalGet | Opcode::RegGet => None,
            // LocalTee/RegTee returns the value it sets
            Opcode::LocalTee | Opcode::RegTee => None,
            // LocalSet/RegSet has no result
            Opcode::LocalSet | Opcode::RegSet => None,
            // Control flow - result depends on block type annotation
            Opcode::Block | Opcode::Loop | Opcode::If | Opcode::Call => None,
            // Branches have no result (they transfer control)
            Opcode::Br | Opcode::BrIf => None,
        }
    }

    /// Returns true if this is a binary operation.
    pub const fn is_binary(self) -> bool {
        self.stack_operands() == 2
    }

    /// Returns true if this is a unary operation.
    pub const fn is_unary(self) -> bool {
        matches!(
            self,
            Opcode::F32Abs
                | Opcode::F32Neg
                | Opcode::F32Ceil
                | Opcode::F32Floor
                | Opcode::F32Trunc
                | Opcode::F32Nearest
                | Opcode::F32Sqrt
                | Opcode::F64Abs
                | Opcode::F64Neg
                | Opcode::F64Ceil
                | Opcode::F64Floor
                | Opcode::F64Trunc
                | Opcode::F64Nearest
                | Opcode::F64Sqrt
                // Type conversions
                | Opcode::I32WrapI64
                | Opcode::I64ExtendI32S
                | Opcode::I64ExtendI32U
                | Opcode::I32TruncF32S
                | Opcode::I32TruncF32U
                | Opcode::I32TruncF64S
                | Opcode::I32TruncF64U
                | Opcode::I64TruncF32S
                | Opcode::I64TruncF32U
                | Opcode::I64TruncF64S
                | Opcode::I64TruncF64U
                | Opcode::F32ConvertI32S
                | Opcode::F32ConvertI32U
                | Opcode::F32ConvertI64S
                | Opcode::F32ConvertI64U
                | Opcode::F32DemoteF64
                | Opcode::F64ConvertI32S
                | Opcode::F64ConvertI32U
                | Opcode::F64ConvertI64S
                | Opcode::F64ConvertI64U
                | Opcode::F64PromoteF32
                | Opcode::I32ReinterpretF32
                | Opcode::I64ReinterpretF64
                | Opcode::F32ReinterpretI32
                | Opcode::F64ReinterpretI64
        )
    }

    /// Returns true if this is a constant operation.
    pub const fn is_const(self) -> bool {
        matches!(
            self,
            Opcode::I32Const | Opcode::I64Const | Opcode::F32Const | Opcode::F64Const
        )
    }

    /// Returns true if this is a local variable operation.
    pub const fn is_local(self) -> bool {
        matches!(self, Opcode::LocalGet | Opcode::LocalSet | Opcode::LocalTee)
    }

    /// Returns true if this is a register operation.
    pub const fn is_register(self) -> bool {
        matches!(self, Opcode::RegGet | Opcode::RegSet | Opcode::RegTee)
    }

    /// Returns true if this is a call operation.
    pub const fn is_call(self) -> bool {
        matches!(self, Opcode::Call)
    }

    /// Returns true if this is a control flow operation.
    pub const fn is_control_flow(self) -> bool {
        matches!(
            self,
            Opcode::Block | Opcode::Loop | Opcode::If | Opcode::Br | Opcode::BrIf | Opcode::Call
        )
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mnemonic())
    }
}
