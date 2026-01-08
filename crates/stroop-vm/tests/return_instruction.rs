//! Tests for the Return instruction.

use stroop_vm::{BytecodeVm, RuntimeError, Value};
use stroop_vm_bytecode::{CompiledModule, ConstPoolValue, Instruction};

fn run_instructions(instructions: Vec<Instruction>) -> Result<Value, RuntimeError> {
    let module = CompiledModule {
        types: vec![],
        functions: vec![],
        instructions,
        constant_pool: vec![],
        imports: vec![],
    };
    let mut vm = BytecodeVm::new();
    vm.execute(&module)
        .map(|v| v.expect("expected return value"))
}

fn run_with_pool(
    instructions: Vec<Instruction>,
    constant_pool: Vec<ConstPoolValue>,
) -> Result<Value, RuntimeError> {
    let module = CompiledModule {
        types: vec![],
        functions: vec![],
        instructions,
        constant_pool,
        imports: vec![],
    };
    let mut vm = BytecodeVm::new();
    vm.execute(&module)
        .map(|v| v.expect("expected return value"))
}

#[test]
fn test_return_i32() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstI32 { dst: 5, value: 42 },
        Instruction::Return { src: 5 },
    ])?;
    assert_eq!(result.as_i32(), 42);
    Ok(())
}

#[test]
fn test_return_i32_negative() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstI32 {
            dst: 0,
            value: -123,
        },
        Instruction::Return { src: 0 },
    ])?;
    assert_eq!(result.as_i32(), -123);
    Ok(())
}

#[test]
fn test_return_i64() -> Result<(), RuntimeError> {
    let result = run_with_pool(
        vec![
            Instruction::LoadConstI64 { dst: 10, index: 0 },
            Instruction::Return { src: 10 },
        ],
        vec![ConstPoolValue::I64(0x1_0000_0042)],
    )?;
    assert_eq!(result.as_i64(), 0x1_0000_0042);
    Ok(())
}

#[test]
fn test_return_f32() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstF32 {
            dst: 3,
            value: 3.14,
        },
        Instruction::Return { src: 3 },
    ])?;
    assert!((result.as_f32() - 3.14).abs() < 0.001);
    Ok(())
}

#[test]
fn test_return_f64() -> Result<(), RuntimeError> {
    let result = run_with_pool(
        vec![
            Instruction::LoadConstF64 { dst: 7, index: 0 },
            Instruction::Return { src: 7 },
        ],
        vec![ConstPoolValue::F64(2.718281828)],
    )?;
    assert!((result.as_f64() - 2.718281828).abs() < 0.0000001);
    Ok(())
}

#[test]
fn test_return_from_different_registers() -> Result<(), RuntimeError> {
    // Return from register 0
    let result = run_instructions(vec![
        Instruction::LoadConstI32 { dst: 0, value: 100 },
        Instruction::Return { src: 0 },
    ])?;
    assert_eq!(result.as_i32(), 100);

    // Return from register 255
    let result = run_instructions(vec![
        Instruction::LoadConstI32 {
            dst: 255,
            value: 200,
        },
        Instruction::Return { src: 255 },
    ])?;
    assert_eq!(result.as_i32(), 200);

    Ok(())
}

#[test]
fn test_return_arithmetic_result() -> Result<(), RuntimeError> {
    let result = run_instructions(vec![
        Instruction::LoadConstI32 { dst: 1, value: 10 },
        Instruction::LoadConstI32 { dst: 2, value: 32 },
        Instruction::I32Add {
            dst: 3,
            lhs: 1,
            rhs: 2,
        },
        Instruction::Return { src: 3 },
    ])?;
    assert_eq!(result.as_i32(), 42);
    Ok(())
}

#[test]
fn test_return_early_exits() -> Result<(), RuntimeError> {
    // Return should stop execution before reaching Halt
    let result = run_instructions(vec![
        Instruction::LoadConstI32 { dst: 1, value: 1 },
        Instruction::Return { src: 1 },
        // These instructions should never execute
        Instruction::LoadConstI32 { dst: 1, value: 999 },
        Instruction::Halt,
    ])?;
    assert_eq!(result.as_i32(), 1);
    Ok(())
}
