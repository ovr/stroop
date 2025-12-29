use crate::opcode::Opcode;
use stroop_bytecode::{Import, Span, ValueType};

/// Constant value for const expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl ConstValue {
    /// Returns the type of this constant value.
    pub fn value_type(&self) -> ValueType {
        match self {
            ConstValue::I32(_) => ValueType::I32,
            ConstValue::I64(_) => ValueType::I64,
            ConstValue::F32(_) => ValueType::F32,
            ConstValue::F64(_) => ValueType::F64,
        }
    }
}

impl std::fmt::Display for ConstValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstValue::I32(v) => write!(f, "{}", v),
            ConstValue::I64(v) => write!(f, "{}", v),
            ConstValue::F32(v) => write!(f, "{}", v),
            ConstValue::F64(v) => write!(f, "{}", v),
        }
    }
}

/// Block type annotation for structured control flow.
/// Specifies the result type of a block, loop, or if expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// No result value (void)
    Empty,
    /// Returns a single value of the specified type
    Value(ValueType),
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::Empty
    }
}

/// A complete module with imports and code.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// Import declarations
    pub imports: Vec<Import>,
    /// Module body expressions
    pub body: Vec<Expr>,
    /// Source span
    pub span: Span,
}

impl Module {
    /// Create a new empty module.
    pub fn new(span: Span) -> Self {
        Self {
            imports: Vec::new(),
            body: Vec::new(),
            span,
        }
    }
}

/// AST node representing an expression in the text format.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Constant value: (i32.const 42)
    Const { value: ConstValue, span: Span },

    /// Binary operation: (i64.add <expr> <expr>)
    BinaryOp {
        opcode: Opcode,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },

    /// Unary operation: (f32.neg <expr>)
    UnaryOp {
        opcode: Opcode,
        operand: Box<Expr>,
        span: Span,
    },

    /// Get local variable: (local.get 0)
    LocalGet { index: u32, span: Span },

    /// Set local variable: (local.set 0 <expr>)
    LocalSet {
        index: u32,
        value: Box<Expr>,
        span: Span,
    },

    /// Tee local variable: (local.tee 0 <expr>)
    LocalTee {
        index: u32,
        value: Box<Expr>,
        span: Span,
    },

    /// Get register value: (reg.get 0)
    RegGet { index: u32, span: Span },

    /// Set register value: (reg.set 0 <expr>)
    RegSet {
        index: u32,
        value: Box<Expr>,
        span: Span,
    },

    /// Tee register value: (reg.tee 0 <expr>)
    RegTee {
        index: u32,
        value: Box<Expr>,
        span: Span,
    },

    /// Function call: (call $log <args...>) or (call 0 <args...>)
    Call {
        /// Resolved function index
        func_idx: u32,
        /// Original function name if used (e.g., $log)
        func_name: Option<String>,
        /// Call arguments
        args: Vec<Expr>,
        /// Source span
        span: Span,
    },

    /// Block construct: (block (result i32)? <body...>)
    /// Creates a scope that can be branched out of.
    Block {
        /// Optional result type
        block_type: BlockType,
        /// Block body expressions
        body: Vec<Expr>,
        /// Source span
        span: Span,
    },

    /// Loop construct: (loop (result i32)? <body...>)
    /// Creates a scope that can be branched back to (re-executed).
    Loop {
        /// Optional result type
        block_type: BlockType,
        /// Loop body expressions
        body: Vec<Expr>,
        /// Source span
        span: Span,
    },

    /// Unconditional branch: (br <label_depth>)
    /// Branches to label at the specified relative depth.
    Br {
        /// Relative depth (0 = innermost)
        label_depth: u32,
        /// Source span
        span: Span,
    },

    /// Conditional branch: (br_if <label_depth> <condition>)
    /// Branches if condition is non-zero.
    BrIf {
        /// Relative depth (0 = innermost)
        label_depth: u32,
        /// Condition expression
        condition: Box<Expr>,
        /// Source span
        span: Span,
    },

    /// If-else construct: (if <condition> (then <body...>) (else <body...>)?)
    /// Conditional execution with optional else branch.
    If {
        /// Optional result type
        block_type: BlockType,
        /// Condition expression
        condition: Box<Expr>,
        /// Then branch body
        then_body: Vec<Expr>,
        /// Optional else branch body
        else_body: Option<Vec<Expr>>,
        /// Source span
        span: Span,
    },
}

impl Expr {
    /// Returns the span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Expr::Const { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::UnaryOp { span, .. }
            | Expr::LocalGet { span, .. }
            | Expr::LocalSet { span, .. }
            | Expr::LocalTee { span, .. }
            | Expr::RegGet { span, .. }
            | Expr::RegSet { span, .. }
            | Expr::RegTee { span, .. }
            | Expr::Call { span, .. }
            | Expr::Block { span, .. }
            | Expr::Loop { span, .. }
            | Expr::Br { span, .. }
            | Expr::BrIf { span, .. }
            | Expr::If { span, .. } => *span,
        }
    }

    /// Create a new Const expression.
    pub fn i32_const(value: i32, span: Span) -> Self {
        Expr::Const {
            value: ConstValue::I32(value),
            span,
        }
    }

    /// Create a new Const expression.
    pub fn i64_const(value: i64, span: Span) -> Self {
        Expr::Const {
            value: ConstValue::I64(value),
            span,
        }
    }

    /// Create a new Const expression.
    pub fn f32_const(value: f32, span: Span) -> Self {
        Expr::Const {
            value: ConstValue::F32(value),
            span,
        }
    }

    /// Create a new Const expression.
    pub fn f64_const(value: f64, span: Span) -> Self {
        Expr::Const {
            value: ConstValue::F64(value),
            span,
        }
    }

    /// Create a new binary operation.
    pub fn binary(opcode: Opcode, lhs: Expr, rhs: Expr, span: Span) -> Self {
        Expr::BinaryOp {
            opcode,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        }
    }

    /// Create a new unary operation.
    pub fn unary(opcode: Opcode, operand: Expr, span: Span) -> Self {
        Expr::UnaryOp {
            opcode,
            operand: Box::new(operand),
            span,
        }
    }

    /// Create a local.get expression.
    pub fn local_get(index: u32, span: Span) -> Self {
        Expr::LocalGet { index, span }
    }

    /// Create a local.set expression.
    pub fn local_set(index: u32, value: Expr, span: Span) -> Self {
        Expr::LocalSet {
            index,
            value: Box::new(value),
            span,
        }
    }

    /// Create a local.tee expression.
    pub fn local_tee(index: u32, value: Expr, span: Span) -> Self {
        Expr::LocalTee {
            index,
            value: Box::new(value),
            span,
        }
    }

    /// Create a reg.get expression.
    pub fn reg_get(index: u32, span: Span) -> Self {
        Expr::RegGet { index, span }
    }

    /// Create a reg.set expression.
    pub fn reg_set(index: u32, value: Expr, span: Span) -> Self {
        Expr::RegSet {
            index,
            value: Box::new(value),
            span,
        }
    }

    /// Create a reg.tee expression.
    pub fn reg_tee(index: u32, value: Expr, span: Span) -> Self {
        Expr::RegTee {
            index,
            value: Box::new(value),
            span,
        }
    }

    /// Create a call expression.
    pub fn call(func_idx: u32, func_name: Option<String>, args: Vec<Expr>, span: Span) -> Self {
        Expr::Call {
            func_idx,
            func_name,
            args,
            span,
        }
    }

    /// Create a block expression.
    pub fn block(block_type: BlockType, body: Vec<Expr>, span: Span) -> Self {
        Expr::Block {
            block_type,
            body,
            span,
        }
    }

    /// Create a loop expression.
    pub fn loop_expr(block_type: BlockType, body: Vec<Expr>, span: Span) -> Self {
        Expr::Loop {
            block_type,
            body,
            span,
        }
    }

    /// Create an unconditional branch.
    pub fn br(label_depth: u32, span: Span) -> Self {
        Expr::Br { label_depth, span }
    }

    /// Create a conditional branch.
    pub fn br_if(label_depth: u32, condition: Expr, span: Span) -> Self {
        Expr::BrIf {
            label_depth,
            condition: Box::new(condition),
            span,
        }
    }

    /// Create an if-else expression.
    pub fn if_else(
        block_type: BlockType,
        condition: Expr,
        then_body: Vec<Expr>,
        else_body: Option<Vec<Expr>>,
        span: Span,
    ) -> Self {
        Expr::If {
            block_type,
            condition: Box::new(condition),
            then_body,
            else_body,
            span,
        }
    }
}
