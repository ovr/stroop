//! Tests for type conversion instructions.

use stroop_bytecode::{CompiledModule, ConstPoolValue, Instruction, ValueType};
use stroop_vm::{BytecodeVm, RuntimeError, Value};

/// Helper to run a sequence of instructions and get the result from register 0.
fn run_instructions(instructions: Vec<Instruction>) -> Result<Value, RuntimeError> {
    run_instructions_with_type(instructions, ValueType::I32)
}

/// Helper to run instructions with a specified return type.
fn run_instructions_with_type(
    instructions: Vec<Instruction>,
    result_type: ValueType,
) -> Result<Value, RuntimeError> {
    let module = CompiledModule {
        instructions,
        constant_pool: vec![],
        imports: vec![],
        result_type: Some(result_type),
    };
    let mut vm = BytecodeVm::new();
    vm.execute(&module)
        .map(|v| v.expect("expected return value"))
}

/// Helper to run instructions with a constant pool.
fn run_with_pool(
    instructions: Vec<Instruction>,
    constant_pool: Vec<ConstPoolValue>,
) -> Result<Value, RuntimeError> {
    run_with_pool_and_type(instructions, constant_pool, ValueType::I32)
}

/// Helper to run instructions with a constant pool and return type.
fn run_with_pool_and_type(
    instructions: Vec<Instruction>,
    constant_pool: Vec<ConstPoolValue>,
    result_type: ValueType,
) -> Result<Value, RuntimeError> {
    let module = CompiledModule {
        instructions,
        constant_pool,
        imports: vec![],
        result_type: Some(result_type),
    };
    let mut vm = BytecodeVm::new();
    vm.execute(&module)
        .map(|v| v.expect("expected return value"))
}

// =============================================================================
// Integer Wrap/Extend Tests
// =============================================================================

#[test]
fn test_i32_wrap_i64() -> Result<(), RuntimeError> {
    // Wrap i64 to i32 (keeps lower 32 bits)
    let result = run_with_pool(
        vec![
            Instruction::LoadConstI64 { dst: 1, index: 0 },
            Instruction::I32WrapI64 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::I64(0x1_0000_0042)], // 4294967362
    )?;
    assert_eq!(result.as_i32(), 0x42); // lower 32 bits = 66
    Ok(())
}

#[test]
fn test_i32_wrap_i64_negative() -> Result<(), RuntimeError> {
    let result = run_with_pool(
        vec![
            Instruction::LoadConstI64 { dst: 1, index: 0 },
            Instruction::I32WrapI64 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::I64(-1i64)],
    )?;
    assert_eq!(result.as_i32(), -1); // -1 as i64 wraps to -1 as i32
    Ok(())
}

#[test]
fn test_i64_extend_i32_s() -> Result<(), RuntimeError> {
    // Sign-extend i32 to i64
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 { dst: 1, value: -42 },
            Instruction::I64ExtendI32S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::I64,
    )?;
    assert_eq!(result.as_i64(), -42i64);
    Ok(())
}

#[test]
fn test_i64_extend_i32_u() -> Result<(), RuntimeError> {
    // Zero-extend i32 to i64
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 { dst: 1, value: -1 }, // 0xFFFFFFFF as u32
            Instruction::I64ExtendI32U { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::I64,
    )?;
    assert_eq!(result.as_i64(), 0xFFFF_FFFF_i64); // 4294967295
    Ok(())
}

#[test]
fn test_i64_extend_i32_s_positive() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 { dst: 1, value: 42 },
            Instruction::I64ExtendI32S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::I64,
    )?;
    assert_eq!(result.as_i64(), 42i64);
    Ok(())
}

// =============================================================================
// Float Demote/Promote Tests
// =============================================================================

#[test]
fn test_f32_demote_f64() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::F32DemoteF64 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(3.141592653589793)],
        ValueType::F32,
    )?;
    // f32 has less precision
    assert!((result.as_f32() - 3.1415927).abs() < 1e-6);
    Ok(())
}

#[test]
fn test_f64_promote_f32() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstF32 {
                dst: 1,
                value: 3.14f32,
            },
            Instruction::F64PromoteF32 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::F64,
    )?;
    assert!((result.as_f64() - 3.14).abs() < 1e-6);
    Ok(())
}

#[test]
fn test_f32_demote_f64_infinity() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::F32DemoteF64 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(f64::INFINITY)],
        ValueType::F32,
    )?;
    assert!(result.as_f32().is_infinite());
    Ok(())
}

// =============================================================================
// Float to Integer Truncation Tests
// =============================================================================

#[test]
fn test_i32_trunc_f32_s() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 1,
            value: 42.9f32,
        },
        Instruction::I32TruncF32S { dst: 0, src: 1 },
        Instruction::Halt,
    ])?;
    assert_eq!(result.as_i32(), 42);
    Ok(())
}

#[test]
fn test_i32_trunc_f32_s_negative() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 1,
            value: -42.9f32,
        },
        Instruction::I32TruncF32S { dst: 0, src: 1 },
        Instruction::Halt,
    ])?;
    assert_eq!(result.as_i32(), -42);
    Ok(())
}

#[test]
fn test_i32_trunc_f32_u() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 1,
            value: 42.9f32,
        },
        Instruction::I32TruncF32U { dst: 0, src: 1 },
        Instruction::Halt,
    ])?;
    assert_eq!(result.as_i32() as u32, 42u32);
    Ok(())
}

#[test]
fn test_i32_trunc_f64_s() -> Result<(), RuntimeError> {
    let result = run_with_pool(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I32TruncF64S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(99.9)],
    )?;
    assert_eq!(result.as_i32(), 99);
    Ok(())
}

#[test]
fn test_i32_trunc_f32_s_nan_traps() {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 1,
            value: f32::NAN,
        },
        Instruction::I32TruncF32S { dst: 0, src: 1 },
        Instruction::Halt,
    ]);
    assert!(matches!(result, Err(RuntimeError::IntegerOverflow)));
}

#[test]
fn test_i32_trunc_f32_u_negative_traps() {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 1,
            value: -1.0f32,
        },
        Instruction::I32TruncF32U { dst: 0, src: 1 },
        Instruction::Halt,
    ]);
    assert!(matches!(result, Err(RuntimeError::IntegerOverflow)));
}

#[test]
fn test_i32_trunc_f64_s_overflow_traps() {
    let result = run_with_pool(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I32TruncF64S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(3e10)], // > i32::MAX
    );
    assert!(matches!(result, Err(RuntimeError::IntegerOverflow)));
}

#[test]
fn test_i64_trunc_f64_s() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I64TruncF64S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(-999.99)],
        ValueType::I64,
    )?;
    assert_eq!(result.as_i64(), -999);
    Ok(())
}

#[test]
fn test_i64_trunc_f64_u() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I64TruncF64U { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(1e18)],
        ValueType::I64,
    )?;
    assert_eq!(result.as_i64() as u64, 1_000_000_000_000_000_000u64);
    Ok(())
}

// =============================================================================
// Integer to Float Conversion Tests
// =============================================================================

#[test]
fn test_f32_convert_i32_s() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 { dst: 1, value: -42 },
            Instruction::F32ConvertI32S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::F32,
    )?;
    assert_eq!(result.as_f32(), -42.0f32);
    Ok(())
}

#[test]
fn test_f32_convert_i32_u() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 { dst: 1, value: -1 }, // 0xFFFFFFFF as u32
            Instruction::F32ConvertI32U { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::F32,
    )?;
    // -1 as i32 = 4294967295 as u32
    assert!((result.as_f32() - 4294967295.0f32).abs() < 1000.0);
    Ok(())
}

#[test]
fn test_f64_convert_i32_s() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 {
                dst: 1,
                value: -100,
            },
            Instruction::F64ConvertI32S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::F64,
    )?;
    assert_eq!(result.as_f64(), -100.0);
    Ok(())
}

#[test]
fn test_f64_convert_i32_u() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 { dst: 1, value: -1 }, // 4294967295 as u32
            Instruction::F64ConvertI32U { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::F64,
    )?;
    assert_eq!(result.as_f64(), 4294967295.0);
    Ok(())
}

#[test]
fn test_f64_convert_i64_s() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstI64 { dst: 1, index: 0 },
            Instruction::F64ConvertI64S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::I64(-1_000_000_000_000i64)],
        ValueType::F64,
    )?;
    assert_eq!(result.as_f64(), -1_000_000_000_000.0);
    Ok(())
}

#[test]
fn test_f64_convert_i64_u() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstI64 { dst: 1, index: 0 },
            Instruction::F64ConvertI64U { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::I64(-1i64)], // u64::MAX
        ValueType::F64,
    )?;
    // -1 as i64 = 18446744073709551615 as u64
    assert!(result.as_f64() > 1.8e19);
    Ok(())
}

// =============================================================================
// Reinterpret (Bitcast) Tests
// =============================================================================

#[test]
fn test_i32_reinterpret_f32() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 1,
            value: 1.0f32,
        },
        Instruction::I32ReinterpretF32 { dst: 0, src: 1 },
        Instruction::Halt,
    ])?;
    // 1.0f32 bit pattern = 0x3f800000 = 1065353216
    assert_eq!(result.as_i32(), 0x3f800000u32 as i32);
    Ok(())
}

#[test]
fn test_f32_reinterpret_i32() -> Result<(), RuntimeError> {
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstI32 {
                dst: 1,
                value: 0x3f800000u32 as i32,
            },
            Instruction::F32ReinterpretI32 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        ValueType::F32,
    )?;
    assert_eq!(result.as_f32(), 1.0f32);
    Ok(())
}

#[test]
fn test_i64_reinterpret_f64() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I64ReinterpretF64 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(1.0)],
        ValueType::I64,
    )?;
    // 1.0f64 bit pattern = 0x3ff0000000000000
    assert_eq!(result.as_i64(), 0x3ff0000000000000u64 as i64);
    Ok(())
}

#[test]
fn test_f64_reinterpret_i64() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstI64 { dst: 1, index: 0 },
            Instruction::F64ReinterpretI64 { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        // pi bit pattern
        vec![ConstPoolValue::I64(0x400921fb54442d18u64 as i64)],
        ValueType::F64,
    )?;
    assert!((result.as_f64() - std::f64::consts::PI).abs() < 1e-15);
    Ok(())
}

#[test]
fn test_reinterpret_roundtrip_f32() -> Result<(), RuntimeError> {
    // f32 -> i32 -> f32 should preserve bit pattern
    let result = run_instructions_with_type(
        vec![
            Instruction::LoadConstF32 {
                dst: 1,
                value: -0.0f32,
            },
            Instruction::I32ReinterpretF32 { dst: 2, src: 1 },
            Instruction::F32ReinterpretI32 { dst: 0, src: 2 },
            Instruction::Halt,
        ],
        ValueType::F32,
    )?;
    // -0.0 has a different bit pattern than 0.0
    assert!(result.as_f32().is_sign_negative());
    assert_eq!(result.as_f32(), 0.0);
    Ok(())
}

#[test]
fn test_reinterpret_roundtrip_f64() -> Result<(), RuntimeError> {
    let result = run_with_pool_and_type(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I64ReinterpretF64 { dst: 2, src: 1 },
            Instruction::F64ReinterpretI64 { dst: 0, src: 2 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(f64::NEG_INFINITY)],
        ValueType::F64,
    )?;
    assert_eq!(result.as_f64(), f64::NEG_INFINITY);
    Ok(())
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_i32_trunc_f64_s_boundary() -> Result<(), RuntimeError> {
    // Test near i32::MAX
    let result = run_with_pool(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I32TruncF64S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(2147483647.0)], // i32::MAX
    )?;
    assert_eq!(result.as_i32(), i32::MAX);
    Ok(())
}

#[test]
fn test_i32_trunc_f64_s_min_boundary() -> Result<(), RuntimeError> {
    // Test i32::MIN
    let result = run_with_pool(
        vec![
            Instruction::LoadConstF64 { dst: 1, index: 0 },
            Instruction::I32TruncF64S { dst: 0, src: 1 },
            Instruction::Halt,
        ],
        vec![ConstPoolValue::F64(-2147483648.0)], // i32::MIN
    )?;
    assert_eq!(result.as_i32(), i32::MIN);
    Ok(())
}

#[test]
fn test_conversion_chain() -> Result<(), RuntimeError> {
    // i32 -> i64 -> f64 -> f32 -> i32
    let result = run_instructions(vec![
        Instruction::LoadConstI32 { dst: 1, value: 42 },
        Instruction::I64ExtendI32S { dst: 2, src: 1 },
        Instruction::F64ConvertI64S { dst: 3, src: 2 },
        Instruction::F32DemoteF64 { dst: 4, src: 3 },
        Instruction::I32TruncF32S { dst: 0, src: 4 },
        Instruction::Halt,
    ])?;
    assert_eq!(result.as_i32(), 42);
    Ok(())
}
