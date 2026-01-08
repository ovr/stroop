//! stroop-assembly-text: SAT (Stroop Assembly Text) parser and compiler.
//!
//! This crate provides a WAT-like S-expression parser for the Stroop bytecode
//! text format (.sat files), plus a compiler to transform AST to bytecode.
//!
//! # Example
//!
//! ```rust
//! use stroop_assembly_text::{parse, Expr, ConstValue};
//!
//! let expr = parse("(i64.add (i64.const 1) (i64.const 2))").unwrap();
//!
//! match expr {
//!     Expr::BinaryOp { opcode, lhs, rhs, .. } => {
//!         println!("Binary op: {}", opcode);
//!     }
//!     _ => {}
//! }
//! ```

pub mod ast;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;

pub use ast::{BlockType, ConstValue, Expr, Module};
pub use compiler::compile_module;
pub use error::{CompileError, Error, LexError, ParseError, SourceLocation};
pub use lexer::{Token, TokenKind};
pub use parser::{ModuleParser, Parser};
pub use stroop_assembly::Opcode;

/// Convenience function to parse a string into an expression AST.
///
/// # Example
///
/// ```rust
/// use stroop_assembly_text::parse;
///
/// let expr = parse("(i32.const 42)").unwrap();
/// ```
pub fn parse(input: &str) -> Result<Expr, Error> {
    let mut parser = Parser::new(input).map_err(Error::Parse)?;
    parser.parse_expr().map_err(Error::Parse)
}

/// Parse multiple expressions from input.
///
/// # Example
///
/// ```rust
/// use stroop_assembly_text::parse_all;
///
/// let exprs = parse_all("(i32.const 1) (i32.const 2)").unwrap();
/// assert_eq!(exprs.len(), 2);
/// ```
pub fn parse_all(input: &str) -> Result<Vec<Expr>, Error> {
    let mut parser = Parser::new(input).map_err(Error::Parse)?;
    let mut exprs = Vec::new();
    while !parser.is_at_end() {
        exprs.push(parser.parse_expr().map_err(Error::Parse)?);
    }
    Ok(exprs)
}

/// Parse a module with imports from input.
///
/// # Example
///
/// ```rust
/// use stroop_assembly_text::parse_module;
///
/// let module = parse_module(r#"
///     (module
///         (import "console" "log" (func $log (param i32)))
///         (call $log (i32.const 42))
///     )
/// "#).unwrap();
///
/// assert_eq!(module.imports.len(), 1);
/// assert_eq!(module.body.len(), 1);
/// ```
pub fn parse_module(input: &str) -> Result<Module, Error> {
    let mut parser = ModuleParser::new(input).map_err(Error::Parse)?;
    parser.parse_module().map_err(Error::Parse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let expr = parse("(i32.const 42)").unwrap();
        assert!(matches!(
            expr,
            Expr::Const {
                value: ConstValue::I32(42),
                ..
            }
        ));
    }

    #[test]
    fn test_parse_all_multiple() {
        let exprs = parse_all("(i32.const 1) (i32.const 2) (i32.const 3)").unwrap();
        assert_eq!(exprs.len(), 3);
    }

    #[test]
    fn test_complex_expression() {
        let input = "(i64.add (i64.mul (i64.const 2) (i64.const 3)) (i64.const 4))";
        let expr = parse(input).unwrap();
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                opcode: Opcode::I64Add,
                ..
            }
        ));
    }
}

/// Integration tests for parse → compile → execute pipeline
#[cfg(test)]
mod integration_tests {
    use super::*;
    use stroop_vm::{BytecodeVm, Value};
    use stroop_vm_bytecode::{FuncType, ValueType};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// Helper to execute a module and get the last register value
    fn run_module(code: &str) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let module = parse_module(code)?;
        let compiled = compile_module(&module)?;
        let mut vm = BytecodeVm::new();
        Ok(vm.execute(&compiled)?)
    }

    #[test]
    fn test_i32_add() -> TestResult {
        let result = run_module("(module (i32.add (i32.const 1) (i32.const 2)))")?;
        assert_eq!(result, Some(Value::I32(3)));
        Ok(())
    }

    #[test]
    fn test_i64_add() -> TestResult {
        let result = run_module("(module (i64.add (i64.const 1000000000000) (i64.const 1)))")?;
        assert_eq!(result, Some(Value::I64(1000000000001)));
        Ok(())
    }

    #[test]
    fn test_f64_add() -> TestResult {
        let result = run_module("(module (f64.add (f64.const 1.5) (f64.const 2.5)))")?;
        match result {
            Some(Value::F64(v)) => assert!((v - 4.0).abs() < 0.0001),
            _ => panic!("expected f64"),
        }
        Ok(())
    }

    #[test]
    fn test_nested_expr() -> TestResult {
        // (1 + 2) * 3 = 9
        let result =
            run_module("(module (i32.mul (i32.add (i32.const 1) (i32.const 2)) (i32.const 3)))")?;
        assert_eq!(result, Some(Value::I32(9)));
        Ok(())
    }

    #[test]
    fn test_i32_comparison() -> TestResult {
        let result = run_module("(module (i32.lt_s (i32.const 1) (i32.const 2)))")?;
        assert_eq!(result, Some(Value::I32(1)));
        Ok(())
    }

    #[test]
    fn test_module_with_import() -> TestResult {
        let mut vm = BytecodeVm::new();

        vm.register_host_fn(
            "console",
            "log",
            FuncType::with_params_results(vec![ValueType::I32], vec![ValueType::I32]),
            |args| Ok(Some(args[0])),
        );

        let module = parse_module(
            r#"
            (module
                (import "console" "log" (func $log (param i32) (result i32)))
                (call $log (i32.add (i32.const 1) (i32.const 1)))
            )
            "#,
        )?;

        let compiled = compile_module(&module)?;
        let result = vm.execute(&compiled)?;
        assert_eq!(result, Some(Value::I32(2)));
        Ok(())
    }

    #[test]
    fn test_return_i32() -> TestResult {
        let result = run_module("(module (return (i32.const 42)))")?;
        assert_eq!(result, Some(Value::I32(42)));
        Ok(())
    }

    #[test]
    fn test_return_expression() -> TestResult {
        let result = run_module("(module (return (i32.add (i32.const 10) (i32.const 32))))")?;
        assert_eq!(result, Some(Value::I32(42)));
        Ok(())
    }

    #[test]
    fn test_return_f64() -> TestResult {
        let result = run_module("(module (return (f64.const 3.14)))")?;
        match result {
            Some(Value::F64(v)) => assert!((v - 3.14).abs() < 0.0001),
            _ => panic!("expected f64"),
        }
        Ok(())
    }

    #[test]
    fn test_return_local() -> TestResult {
        let result = run_module(
            r#"(module
                (local.set 0 (i32.const 100))
                (return (local.get 0))
            )"#,
        )?;
        assert_eq!(result, Some(Value::I32(100)));
        Ok(())
    }
}
