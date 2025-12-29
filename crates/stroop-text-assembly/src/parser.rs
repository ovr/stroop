use crate::ast::{BlockType, ConstValue, Expr, Module};
use crate::error::{ParseError, SourceLocation};
use crate::lexer::{Lexer, Span, Token, TokenKind};
use crate::opcode::Opcode;
use std::collections::HashMap;
use stroop_bytecode::{FuncType, Import, ValueType};

/// Recursive descent parser for S-expression syntax.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given input.
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self { lexer, current })
    }

    /// Check if we're at the end of input.
    pub fn is_at_end(&self) -> bool {
        matches!(self.current.kind, TokenKind::Eof)
    }

    /// Get the current source location.
    fn loc(&self) -> SourceLocation {
        SourceLocation::new(self.current.span.start, 0, 0)
    }

    /// Advance to the next token and return the previous one.
    fn advance(&mut self) -> Result<Token, ParseError> {
        let prev = std::mem::replace(&mut self.current, self.lexer.next_token()?);
        Ok(prev)
    }

    /// Expect the current token to be of a specific kind and advance.
    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if std::mem::discriminant(&self.current.kind) == std::mem::discriminant(&expected) {
            self.advance()
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", expected),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            })
        }
    }

    /// Parse a single expression.
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current.span;

        // Expect opening paren
        self.expect(TokenKind::LParen)?;

        // Get instruction mnemonic
        let mnemonic = match &self.current.kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Eof => {
                return Err(ParseError::UnexpectedEof {
                    expected: "instruction".to_string(),
                    loc: self.loc(),
                });
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "instruction".to_string(),
                    found: format!("{}", self.current.kind),
                    loc: self.loc(),
                });
            }
        };
        self.advance()?;

        // Dispatch based on mnemonic
        let expr = self.parse_instruction(&mnemonic, start_span)?;

        // Expect closing paren
        let end_token = self.expect(TokenKind::RParen)?;
        let span = Span::new(start_span.start, end_token.span.end);

        // Update span in the expression
        Ok(self.with_span(expr, span))
    }

    /// Update the span of an expression.
    fn with_span(&self, expr: Expr, span: Span) -> Expr {
        match expr {
            Expr::Const { value, .. } => Expr::Const { value, span },
            Expr::BinaryOp {
                opcode, lhs, rhs, ..
            } => Expr::BinaryOp {
                opcode,
                lhs,
                rhs,
                span,
            },
            Expr::UnaryOp {
                opcode, operand, ..
            } => Expr::UnaryOp {
                opcode,
                operand,
                span,
            },
            Expr::LocalGet { index, .. } => Expr::LocalGet { index, span },
            Expr::LocalSet { index, value, .. } => Expr::LocalSet { index, value, span },
            Expr::LocalTee { index, value, .. } => Expr::LocalTee { index, value, span },
            Expr::RegGet { index, .. } => Expr::RegGet { index, span },
            Expr::RegSet { index, value, .. } => Expr::RegSet { index, value, span },
            Expr::RegTee { index, value, .. } => Expr::RegTee { index, value, span },
            Expr::Call {
                func_idx,
                func_name,
                args,
                ..
            } => Expr::Call {
                func_idx,
                func_name,
                args,
                span,
            },
            Expr::Block {
                block_type, body, ..
            } => Expr::Block {
                block_type,
                body,
                span,
            },
            Expr::Loop {
                block_type, body, ..
            } => Expr::Loop {
                block_type,
                body,
                span,
            },
            Expr::Br { label_depth, .. } => Expr::Br { label_depth, span },
            Expr::BrIf {
                label_depth,
                condition,
                ..
            } => Expr::BrIf {
                label_depth,
                condition,
                span,
            },
            Expr::If {
                block_type,
                condition,
                then_body,
                else_body,
                ..
            } => Expr::If {
                block_type,
                condition,
                then_body,
                else_body,
                span,
            },
        }
    }

    /// Parse instruction based on mnemonic.
    fn parse_instruction(&mut self, mnemonic: &str, span: Span) -> Result<Expr, ParseError> {
        // Check for constant operations
        match mnemonic {
            "i32.const" => return self.parse_const(ValueType::I32, span),
            "i64.const" => return self.parse_const(ValueType::I64, span),
            "f32.const" => return self.parse_const(ValueType::F32, span),
            "f64.const" => return self.parse_const(ValueType::F64, span),
            "local.get" => return self.parse_local_get(span),
            "local.set" => return self.parse_local_set(span),
            "local.tee" => return self.parse_local_tee(span),
            "reg.get" => return self.parse_reg_get(span),
            "reg.set" => return self.parse_reg_set(span),
            "reg.tee" => return self.parse_reg_tee(span),
            _ => {}
        }

        // Try to parse as opcode
        let Some(opcode) = Opcode::from_mnemonic(mnemonic) else {
            return Err(ParseError::UnknownInstruction {
                mnemonic: mnemonic.to_string(),
                loc: SourceLocation::new(span.start, 0, 0),
            });
        };

        if opcode.is_unary() {
            self.parse_unary_op(opcode, span)
        } else if opcode.is_binary() {
            self.parse_binary_op(opcode, span)
        } else {
            Err(ParseError::UnknownInstruction {
                mnemonic: mnemonic.to_string(),
                loc: SourceLocation::new(span.start, 0, 0),
            })
        }
    }

    /// Parse a constant expression like (i32.const 42).
    fn parse_const(&mut self, value_type: ValueType, span: Span) -> Result<Expr, ParseError> {
        let value = match (&self.current.kind, value_type) {
            (TokenKind::Integer(n), ValueType::I32) => {
                let v = *n as i32;
                self.advance()?;
                ConstValue::I32(v)
            }
            (TokenKind::Integer(n), ValueType::I64) => {
                let v = *n;
                self.advance()?;
                ConstValue::I64(v)
            }
            (TokenKind::Integer(n), ValueType::F32) => {
                let v = *n as f32;
                self.advance()?;
                ConstValue::F32(v)
            }
            (TokenKind::Integer(n), ValueType::F64) => {
                let v = *n as f64;
                self.advance()?;
                ConstValue::F64(v)
            }
            (TokenKind::Float(n), ValueType::F32) => {
                let v = *n as f32;
                self.advance()?;
                ConstValue::F32(v)
            }
            (TokenKind::Float(n), ValueType::F64) => {
                let v = *n;
                self.advance()?;
                ConstValue::F64(v)
            }
            (TokenKind::Float(_), ValueType::I32 | ValueType::I64) => {
                return Err(ParseError::InvalidOperand {
                    message: format!("expected integer for {}.const", value_type),
                    loc: self.loc(),
                });
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "number".to_string(),
                    found: format!("{}", self.current.kind),
                    loc: self.loc(),
                });
            }
        };

        Ok(Expr::Const { value, span })
    }

    /// Parse a binary operation like (i64.add <expr> <expr>).
    fn parse_binary_op(&mut self, opcode: Opcode, span: Span) -> Result<Expr, ParseError> {
        let lhs = self.parse_expr()?;
        let rhs = self.parse_expr()?;

        Ok(Expr::BinaryOp {
            opcode,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        })
    }

    /// Parse a unary operation like (f32.neg <expr>).
    fn parse_unary_op(&mut self, opcode: Opcode, span: Span) -> Result<Expr, ParseError> {
        let operand = self.parse_expr()?;

        Ok(Expr::UnaryOp {
            opcode,
            operand: Box::new(operand),
            span,
        })
    }

    /// Parse local index from current token.
    fn parse_local_index(&mut self) -> Result<u32, ParseError> {
        match &self.current.kind {
            TokenKind::Integer(n) => {
                let index = *n as u32;
                self.advance()?;
                Ok(index)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "local index".to_string(),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            }),
        }
    }

    /// Parse (local.get <index>).
    fn parse_local_get(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_local_index()?;
        Ok(Expr::LocalGet { index, span })
    }

    /// Parse (local.set <index> <expr>).
    fn parse_local_set(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_local_index()?;
        let value = self.parse_expr()?;
        Ok(Expr::LocalSet {
            index,
            value: Box::new(value),
            span,
        })
    }

    /// Parse (local.tee <index> <expr>).
    fn parse_local_tee(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_local_index()?;
        let value = self.parse_expr()?;
        Ok(Expr::LocalTee {
            index,
            value: Box::new(value),
            span,
        })
    }

    /// Parse register index from current token.
    fn parse_reg_index(&mut self) -> Result<u32, ParseError> {
        match &self.current.kind {
            TokenKind::Integer(n) => {
                let index = *n as u32;
                self.advance()?;
                Ok(index)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "register index".to_string(),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            }),
        }
    }

    /// Parse (reg.get <index>).
    fn parse_reg_get(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_reg_index()?;
        Ok(Expr::RegGet { index, span })
    }

    /// Parse (reg.set <index> <expr>).
    fn parse_reg_set(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_reg_index()?;
        let value = self.parse_expr()?;
        Ok(Expr::RegSet {
            index,
            value: Box::new(value),
            span,
        })
    }

    /// Parse (reg.tee <index> <expr>).
    fn parse_reg_tee(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_reg_index()?;
        let value = self.parse_expr()?;
        Ok(Expr::RegTee {
            index,
            value: Box::new(value),
            span,
        })
    }
}

/// Module parser with function name resolution.
pub struct ModuleParser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    /// Map from function name ($name) to index
    func_names: HashMap<String, u32>,
}

impl<'a> ModuleParser<'a> {
    /// Create a new module parser.
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self {
            lexer,
            current,
            func_names: HashMap::new(),
        })
    }

    fn loc(&self) -> SourceLocation {
        SourceLocation::new(self.current.span.start, 0, 0)
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        let prev = std::mem::replace(&mut self.current, self.lexer.next_token()?);
        Ok(prev)
    }

    fn expect_lparen(&mut self) -> Result<Token, ParseError> {
        if matches!(self.current.kind, TokenKind::LParen) {
            self.advance()
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "(".to_string(),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            })
        }
    }

    fn expect_rparen(&mut self) -> Result<Token, ParseError> {
        if matches!(self.current.kind, TokenKind::RParen) {
            self.advance()
        } else {
            Err(ParseError::UnexpectedToken {
                expected: ")".to_string(),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            })
        }
    }

    fn expect_ident(&mut self, expected: &str) -> Result<Token, ParseError> {
        if let TokenKind::Ident(s) = &self.current.kind {
            if s == expected {
                return self.advance();
            }
        }
        Err(ParseError::UnexpectedToken {
            expected: expected.to_string(),
            found: format!("{}", self.current.kind),
            loc: self.loc(),
        })
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        if let TokenKind::String(s) = &self.current.kind {
            let s = s.clone();
            self.advance()?;
            Ok(s)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "string".to_string(),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            })
        }
    }

    /// Parse a complete module.
    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let start_span = self.current.span;

        // (module ...)
        self.expect_lparen()?;
        self.expect_ident("module")?;

        let mut module = Module::new(start_span);

        // Parse imports and body
        while matches!(self.current.kind, TokenKind::LParen) {
            // Peek at next token to see if it's import or expression
            let checkpoint_span = self.current.span;
            self.advance()?; // consume (

            match &self.current.kind {
                TokenKind::Ident(s) if s == "import" => {
                    self.advance()?; // consume "import"
                    let import = self.parse_import(checkpoint_span)?;

                    // Register function name if present
                    if let Some(ref alias) = import.alias {
                        let idx = module.imports.len() as u32;
                        self.func_names.insert(alias.clone(), idx);
                    }

                    module.imports.push(import);
                }
                _ => {
                    // It's an expression - parse it
                    let expr = self.parse_expr_after_lparen(checkpoint_span)?;
                    module.body.push(expr);
                }
            }
        }

        let end_token = self.expect_rparen()?;
        module.span = Span::new(start_span.start, end_token.span.end);

        Ok(module)
    }

    /// Parse import after "import" keyword has been consumed.
    fn parse_import(&mut self, start_span: Span) -> Result<Import, ParseError> {
        // (import "module" "name" (func $alias? (param ...)? (result ...)?))
        let module_name = self.expect_string()?;
        let func_name = self.expect_string()?;

        // Parse (func ...)
        self.expect_lparen()?;
        self.expect_ident("func")?;

        // Optional $alias
        let alias = if let TokenKind::Ident(s) = &self.current.kind {
            if s.starts_with('$') {
                let alias = s.clone();
                self.advance()?;
                Some(alias)
            } else {
                None
            }
        } else {
            None
        };

        // Parse function type (params and results)
        let func_type = self.parse_func_type()?;

        self.expect_rparen()?; // close (func ...)
        let end_token = self.expect_rparen()?; // close (import ...)

        Ok(Import {
            module: module_name,
            name: func_name,
            alias,
            func_type,
            span: Span::new(start_span.start, end_token.span.end),
        })
    }

    /// Parse function type (param and result clauses).
    fn parse_func_type(&mut self) -> Result<FuncType, ParseError> {
        let mut params = Vec::new();
        let mut results = Vec::new();

        while matches!(self.current.kind, TokenKind::LParen) {
            self.advance()?; // consume (

            match &self.current.kind {
                TokenKind::Ident(s) if s == "param" => {
                    self.advance()?; // consume "param"
                    // Parse value types until )
                    while let Some(vt) = self.try_parse_value_type() {
                        params.push(vt);
                    }
                    self.expect_rparen()?;
                }
                TokenKind::Ident(s) if s == "result" => {
                    self.advance()?; // consume "result"
                    // Parse value types until )
                    while let Some(vt) = self.try_parse_value_type() {
                        results.push(vt);
                    }
                    self.expect_rparen()?;
                }
                _ => break,
            }
        }

        Ok(FuncType { params, results })
    }

    /// Try to parse a value type, returning None if not a value type.
    fn try_parse_value_type(&mut self) -> Option<ValueType> {
        if let TokenKind::Ident(s) = &self.current.kind {
            let vt = match s.as_str() {
                "i32" => Some(ValueType::I32),
                "i64" => Some(ValueType::I64),
                "f32" => Some(ValueType::F32),
                "f64" => Some(ValueType::F64),
                _ => None,
            };
            if vt.is_some() {
                self.advance().ok()?;
            }
            vt
        } else {
            None
        }
    }

    /// Parse expression after ( has been consumed.
    fn parse_expr_after_lparen(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        // Get instruction mnemonic
        let mnemonic = match &self.current.kind {
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Eof => {
                return Err(ParseError::UnexpectedEof {
                    expected: "instruction".to_string(),
                    loc: self.loc(),
                });
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "instruction".to_string(),
                    found: format!("{}", self.current.kind),
                    loc: self.loc(),
                });
            }
        };
        self.advance()?;

        let expr = self.parse_instruction(&mnemonic, start_span)?;

        let end_token = self.expect_rparen()?;
        let span = Span::new(start_span.start, end_token.span.end);

        Ok(self.with_span(expr, span))
    }

    /// Parse a single expression.
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current.span;
        self.expect_lparen()?;
        self.parse_expr_after_lparen(start_span)
    }

    fn with_span(&self, expr: Expr, span: Span) -> Expr {
        match expr {
            Expr::Const { value, .. } => Expr::Const { value, span },
            Expr::BinaryOp {
                opcode, lhs, rhs, ..
            } => Expr::BinaryOp {
                opcode,
                lhs,
                rhs,
                span,
            },
            Expr::UnaryOp {
                opcode, operand, ..
            } => Expr::UnaryOp {
                opcode,
                operand,
                span,
            },
            Expr::LocalGet { index, .. } => Expr::LocalGet { index, span },
            Expr::LocalSet { index, value, .. } => Expr::LocalSet { index, value, span },
            Expr::LocalTee { index, value, .. } => Expr::LocalTee { index, value, span },
            Expr::RegGet { index, .. } => Expr::RegGet { index, span },
            Expr::RegSet { index, value, .. } => Expr::RegSet { index, value, span },
            Expr::RegTee { index, value, .. } => Expr::RegTee { index, value, span },
            Expr::Call {
                func_idx,
                func_name,
                args,
                ..
            } => Expr::Call {
                func_idx,
                func_name,
                args,
                span,
            },
            Expr::Block {
                block_type, body, ..
            } => Expr::Block {
                block_type,
                body,
                span,
            },
            Expr::Loop {
                block_type, body, ..
            } => Expr::Loop {
                block_type,
                body,
                span,
            },
            Expr::Br { label_depth, .. } => Expr::Br { label_depth, span },
            Expr::BrIf {
                label_depth,
                condition,
                ..
            } => Expr::BrIf {
                label_depth,
                condition,
                span,
            },
            Expr::If {
                block_type,
                condition,
                then_body,
                else_body,
                ..
            } => Expr::If {
                block_type,
                condition,
                then_body,
                else_body,
                span,
            },
        }
    }

    /// Parse instruction based on mnemonic.
    fn parse_instruction(&mut self, mnemonic: &str, span: Span) -> Result<Expr, ParseError> {
        match mnemonic {
            "i32.const" => return self.parse_const(ValueType::I32, span),
            "i64.const" => return self.parse_const(ValueType::I64, span),
            "f32.const" => return self.parse_const(ValueType::F32, span),
            "f64.const" => return self.parse_const(ValueType::F64, span),
            "local.get" => return self.parse_local_get(span),
            "local.set" => return self.parse_local_set(span),
            "local.tee" => return self.parse_local_tee(span),
            "reg.get" => return self.parse_reg_get(span),
            "reg.set" => return self.parse_reg_set(span),
            "reg.tee" => return self.parse_reg_tee(span),
            "call" => return self.parse_call(span),
            "block" => return self.parse_block(span),
            "loop" => return self.parse_loop(span),
            "br" => return self.parse_br(span),
            "br_if" => return self.parse_br_if(span),
            "if" => return self.parse_if(span),
            _ => {}
        }

        let Some(opcode) = Opcode::from_mnemonic(mnemonic) else {
            return Err(ParseError::UnknownInstruction {
                mnemonic: mnemonic.to_string(),
                loc: SourceLocation::new(span.start, 0, 0),
            });
        };

        if opcode.is_unary() {
            self.parse_unary_op(opcode, span)
        } else if opcode.is_binary() {
            self.parse_binary_op(opcode, span)
        } else {
            Err(ParseError::UnknownInstruction {
                mnemonic: mnemonic.to_string(),
                loc: SourceLocation::new(span.start, 0, 0),
            })
        }
    }

    fn parse_const(&mut self, value_type: ValueType, span: Span) -> Result<Expr, ParseError> {
        let value = match (&self.current.kind, value_type) {
            (TokenKind::Integer(n), ValueType::I32) => ConstValue::I32(*n as i32),
            (TokenKind::Integer(n), ValueType::I64) => ConstValue::I64(*n),
            (TokenKind::Integer(n), ValueType::F32) => ConstValue::F32(*n as f32),
            (TokenKind::Integer(n), ValueType::F64) => ConstValue::F64(*n as f64),
            (TokenKind::Float(n), ValueType::F32) => ConstValue::F32(*n as f32),
            (TokenKind::Float(n), ValueType::F64) => ConstValue::F64(*n),
            (TokenKind::Float(_), ValueType::I32 | ValueType::I64) => {
                return Err(ParseError::InvalidOperand {
                    message: format!("expected integer for {}.const", value_type),
                    loc: self.loc(),
                });
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "number".to_string(),
                    found: format!("{}", self.current.kind),
                    loc: self.loc(),
                });
            }
        };
        self.advance()?;
        Ok(Expr::Const { value, span })
    }

    fn parse_binary_op(&mut self, opcode: Opcode, span: Span) -> Result<Expr, ParseError> {
        let lhs = self.parse_expr()?;
        let rhs = self.parse_expr()?;
        Ok(Expr::BinaryOp {
            opcode,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        })
    }

    fn parse_unary_op(&mut self, opcode: Opcode, span: Span) -> Result<Expr, ParseError> {
        let operand = self.parse_expr()?;
        Ok(Expr::UnaryOp {
            opcode,
            operand: Box::new(operand),
            span,
        })
    }

    fn parse_index(&mut self, kind: &str) -> Result<u32, ParseError> {
        match &self.current.kind {
            TokenKind::Integer(n) => {
                let index = *n as u32;
                self.advance()?;
                Ok(index)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: format!("{} index", kind),
                found: format!("{}", self.current.kind),
                loc: self.loc(),
            }),
        }
    }

    fn parse_local_get(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_index("local")?;
        Ok(Expr::LocalGet { index, span })
    }

    fn parse_local_set(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_index("local")?;
        let value = self.parse_expr()?;
        Ok(Expr::LocalSet {
            index,
            value: Box::new(value),
            span,
        })
    }

    fn parse_local_tee(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_index("local")?;
        let value = self.parse_expr()?;
        Ok(Expr::LocalTee {
            index,
            value: Box::new(value),
            span,
        })
    }

    fn parse_reg_get(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_index("register")?;
        Ok(Expr::RegGet { index, span })
    }

    fn parse_reg_set(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_index("register")?;
        let value = self.parse_expr()?;
        Ok(Expr::RegSet {
            index,
            value: Box::new(value),
            span,
        })
    }

    fn parse_reg_tee(&mut self, span: Span) -> Result<Expr, ParseError> {
        let index = self.parse_index("register")?;
        let value = self.parse_expr()?;
        Ok(Expr::RegTee {
            index,
            value: Box::new(value),
            span,
        })
    }

    /// Parse (call <$name|index> <args...>)
    fn parse_call(&mut self, span: Span) -> Result<Expr, ParseError> {
        // Parse function reference: either $name or integer index
        let (func_idx, func_name) = match &self.current.kind {
            TokenKind::Ident(s) if s.starts_with('$') => {
                let name = s.clone();
                self.advance()?;

                // Look up function index
                let idx = self.func_names.get(&name).copied().ok_or_else(|| {
                    ParseError::InvalidOperand {
                        message: format!("unknown function '{}'", name),
                        loc: self.loc(),
                    }
                })?;

                (idx, Some(name))
            }
            TokenKind::Integer(n) => {
                let idx = *n as u32;
                self.advance()?;
                (idx, None)
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "function name or index".to_string(),
                    found: format!("{}", self.current.kind),
                    loc: self.loc(),
                });
            }
        };

        // Parse arguments
        let mut args = Vec::new();
        while matches!(self.current.kind, TokenKind::LParen) {
            args.push(self.parse_expr()?);
        }

        Ok(Expr::Call {
            func_idx,
            func_name,
            args,
            span,
        })
    }

    /// Parse (block (result type)? <body...>)
    fn parse_block(&mut self, span: Span) -> Result<Expr, ParseError> {
        let block_type = BlockType::Empty; // Simplified: no result type parsing for now

        // Parse body expressions until we hit )
        let mut body = Vec::new();
        while matches!(self.current.kind, TokenKind::LParen) {
            body.push(self.parse_expr()?);
        }

        Ok(Expr::Block {
            block_type,
            body,
            span,
        })
    }

    /// Parse (loop (result type)? <body...>)
    fn parse_loop(&mut self, span: Span) -> Result<Expr, ParseError> {
        let block_type = BlockType::Empty; // Simplified: no result type parsing for now

        // Parse body expressions until we hit )
        let mut body = Vec::new();
        while matches!(self.current.kind, TokenKind::LParen) {
            body.push(self.parse_expr()?);
        }

        Ok(Expr::Loop {
            block_type,
            body,
            span,
        })
    }

    /// Parse (br <label_depth>)
    fn parse_br(&mut self, span: Span) -> Result<Expr, ParseError> {
        let label_depth = self.parse_index("label")?;
        Ok(Expr::Br { label_depth, span })
    }

    /// Parse (br_if <label_depth> <condition>)
    fn parse_br_if(&mut self, span: Span) -> Result<Expr, ParseError> {
        let label_depth = self.parse_index("label")?;
        let condition = self.parse_expr()?;
        Ok(Expr::BrIf {
            label_depth,
            condition: Box::new(condition),
            span,
        })
    }

    /// Parse (if <condition> (then <body...>) (else <body...>)?)
    fn parse_if(&mut self, span: Span) -> Result<Expr, ParseError> {
        let block_type = BlockType::Empty; // Simplified: no result type parsing for now

        // Parse condition expression
        let condition = self.parse_expr()?;

        // Parse (then <body...>)
        self.expect_lparen()?;
        self.expect_ident("then")?;

        let mut then_body = Vec::new();
        while matches!(self.current.kind, TokenKind::LParen) {
            then_body.push(self.parse_expr()?);
        }
        self.expect_rparen()?;

        // Parse optional (else <body...>)
        let else_body = if matches!(self.current.kind, TokenKind::LParen) {
            // Peek to see if it's "else"
            let checkpoint = self.current.span;
            self.advance()?; // consume (

            if let TokenKind::Ident(s) = &self.current.kind {
                if s == "else" {
                    self.advance()?; // consume "else"

                    let mut body = Vec::new();
                    while matches!(self.current.kind, TokenKind::LParen) {
                        body.push(self.parse_expr()?);
                    }
                    self.expect_rparen()?;
                    Some(body)
                } else {
                    // Not an else clause - this shouldn't happen in well-formed input
                    return Err(ParseError::UnexpectedToken {
                        expected: "else or )".to_string(),
                        found: format!("{}", self.current.kind),
                        loc: SourceLocation::new(checkpoint.start, 0, 0),
                    });
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "else or )".to_string(),
                    found: format!("{}", self.current.kind),
                    loc: self.loc(),
                });
            }
        } else {
            None
        };

        Ok(Expr::If {
            block_type,
            condition: Box::new(condition),
            then_body,
            else_body,
            span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::Opcode;

    fn parse(input: &str) -> Result<Expr, ParseError> {
        Parser::new(input)?.parse_expr()
    }

    #[test]
    fn test_parse_i32_const() {
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
    fn test_parse_i64_const() {
        let expr = parse("(i64.const 123456789)").unwrap();
        assert!(matches!(
            expr,
            Expr::Const {
                value: ConstValue::I64(123456789),
                ..
            }
        ));
    }

    #[test]
    fn test_parse_f64_const() {
        let expr = parse("(f64.const 3.14)").unwrap();
        match expr {
            Expr::Const {
                value: ConstValue::F64(v),
                ..
            } => assert!((v - 3.14).abs() < 0.001),
            _ => panic!("expected f64 const"),
        }
    }

    #[test]
    fn test_parse_binary_add() {
        let expr = parse("(i64.add (i64.const 1) (i64.const 2))").unwrap();
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                opcode: Opcode::I64Add,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_nested_expr() {
        let expr = parse("(i32.mul (i32.add (i32.const 1) (i32.const 2)) (i32.const 3))").unwrap();
        match expr {
            Expr::BinaryOp {
                opcode: Opcode::I32Mul,
                lhs,
                rhs,
                ..
            } => {
                assert!(matches!(
                    *lhs,
                    Expr::BinaryOp {
                        opcode: Opcode::I32Add,
                        ..
                    }
                ));
                assert!(matches!(
                    *rhs,
                    Expr::Const {
                        value: ConstValue::I32(3),
                        ..
                    }
                ));
            }
            _ => panic!("expected binary op"),
        }
    }

    #[test]
    fn test_parse_unary_neg() {
        let expr = parse("(f64.neg (f64.const 1.0))").unwrap();
        assert!(matches!(
            expr,
            Expr::UnaryOp {
                opcode: Opcode::F64Neg,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_local_get() {
        let expr = parse("(local.get 0)").unwrap();
        assert!(matches!(expr, Expr::LocalGet { index: 0, .. }));
    }

    #[test]
    fn test_parse_local_set() {
        let expr = parse("(local.set 1 (i32.const 42))").unwrap();
        match expr {
            Expr::LocalSet { index, value, .. } => {
                assert_eq!(index, 1);
                assert!(matches!(
                    *value,
                    Expr::Const {
                        value: ConstValue::I32(42),
                        ..
                    }
                ));
            }
            _ => panic!("expected local.set"),
        }
    }

    #[test]
    fn test_parse_local_tee() {
        let expr = parse("(local.tee 2 (i64.const 100))").unwrap();
        match expr {
            Expr::LocalTee { index, value, .. } => {
                assert_eq!(index, 2);
                assert!(matches!(
                    *value,
                    Expr::Const {
                        value: ConstValue::I64(100),
                        ..
                    }
                ));
            }
            _ => panic!("expected local.tee"),
        }
    }

    #[test]
    fn test_parse_hex_const() {
        let expr = parse("(i32.const 0xFF)").unwrap();
        assert!(matches!(
            expr,
            Expr::Const {
                value: ConstValue::I32(255),
                ..
            }
        ));
    }

    #[test]
    fn test_parse_negative_const() {
        let expr = parse("(i32.const -42)").unwrap();
        assert!(matches!(
            expr,
            Expr::Const {
                value: ConstValue::I32(-42),
                ..
            }
        ));
    }

    #[test]
    fn test_error_unknown_instruction() {
        let result = parse("(unknown.op)");
        assert!(matches!(result, Err(ParseError::UnknownInstruction { .. })));
    }

    #[test]
    fn test_error_missing_paren() {
        let result = parse("(i32.const 42");
        assert!(result.is_err());
    }

    #[test]
    fn test_comparison_ops() {
        let expr = parse("(i32.lt_s (i32.const 1) (i32.const 2))").unwrap();
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                opcode: Opcode::I32LtS,
                ..
            }
        ));
    }

    #[test]
    fn test_bitwise_ops() {
        let expr = parse("(i64.and (i64.const 0xFF) (i64.const 0x0F))").unwrap();
        assert!(matches!(
            expr,
            Expr::BinaryOp {
                opcode: Opcode::I64And,
                ..
            }
        ));
    }

    #[test]
    fn test_parse_reg_get() {
        let expr = parse("(reg.get 0)").unwrap();
        assert!(matches!(expr, Expr::RegGet { index: 0, .. }));
    }

    #[test]
    fn test_parse_reg_set() {
        let expr = parse("(reg.set 1 (i32.const 42))").unwrap();
        match expr {
            Expr::RegSet { index, value, .. } => {
                assert_eq!(index, 1);
                assert!(matches!(
                    *value,
                    Expr::Const {
                        value: ConstValue::I32(42),
                        ..
                    }
                ));
            }
            _ => panic!("expected reg.set"),
        }
    }

    #[test]
    fn test_parse_reg_tee() {
        let expr = parse("(reg.tee 2 (i64.const 100))").unwrap();
        match expr {
            Expr::RegTee { index, value, .. } => {
                assert_eq!(index, 2);
                assert!(matches!(
                    *value,
                    Expr::Const {
                        value: ConstValue::I64(100),
                        ..
                    }
                ));
            }
            _ => panic!("expected reg.tee"),
        }
    }

    #[test]
    fn test_reg_in_expression() {
        let expr = parse("(i32.add (reg.get 0) (reg.get 1))").unwrap();
        match expr {
            Expr::BinaryOp {
                opcode: Opcode::I32Add,
                lhs,
                rhs,
                ..
            } => {
                assert!(matches!(*lhs, Expr::RegGet { index: 0, .. }));
                assert!(matches!(*rhs, Expr::RegGet { index: 1, .. }));
            }
            _ => panic!("expected binary op with reg.get"),
        }
    }
}
