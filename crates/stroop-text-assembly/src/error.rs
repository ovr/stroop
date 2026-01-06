use std::fmt;

/// Source location for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceLocation {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Lexer errors.
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { ch: char, loc: SourceLocation },
    InvalidNumber { text: String, loc: SourceLocation },
    UnexpectedEof { loc: SourceLocation },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedCharacter { ch, loc } => {
                write!(f, "unexpected character '{}' at {}", ch, loc)
            }
            LexError::InvalidNumber { text, loc } => {
                write!(f, "invalid number '{}' at {}", text, loc)
            }
            LexError::UnexpectedEof { loc } => {
                write!(f, "unexpected end of input at {}", loc)
            }
        }
    }
}

impl std::error::Error for LexError {}

/// Parser errors.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        loc: SourceLocation,
    },
    UnexpectedEof {
        expected: String,
        loc: SourceLocation,
    },
    UnknownInstruction {
        mnemonic: String,
        loc: SourceLocation,
    },
    InvalidOperand {
        message: String,
        loc: SourceLocation,
    },
    LexError(LexError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                loc,
            } => {
                write!(f, "expected {}, found '{}' at {}", expected, found, loc)
            }
            ParseError::UnexpectedEof { expected, loc } => {
                write!(f, "expected {}, found end of input at {}", expected, loc)
            }
            ParseError::UnknownInstruction { mnemonic, loc } => {
                write!(f, "unknown instruction '{}' at {}", mnemonic, loc)
            }
            ParseError::InvalidOperand { message, loc } => {
                write!(f, "{} at {}", message, loc)
            }
            ParseError::LexError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::LexError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError::LexError(e)
    }
}

/// Compiler errors.
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    ConstantPoolOverflow { count: usize },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::ConstantPoolOverflow { count } => {
                write!(f, "constant pool overflow: {} constants (max 65535)", count)
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Combined error type for the crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Lex(LexError),
    Parse(ParseError),
    Compile(CompileError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex(e) => write!(f, "{}", e),
            Error::Parse(e) => write!(f, "{}", e),
            Error::Compile(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Lex(e) => Some(e),
            Error::Parse(e) => Some(e),
            Error::Compile(e) => Some(e),
        }
    }
}

impl From<LexError> for Error {
    fn from(e: LexError) -> Self {
        Error::Lex(e)
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<CompileError> for Error {
    fn from(e: CompileError) -> Self {
        Error::Compile(e)
    }
}
