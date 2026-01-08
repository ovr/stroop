use crate::error::{LexError, SourceLocation};

// Re-export Span from stroop-vm-bytecode
pub use stroop_vm_bytecode::Span;

/// Token types for the S-expression lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// Opening parenthesis '('
    LParen,
    /// Closing parenthesis ')'
    RParen,
    /// Instruction mnemonic like "i64.add", "local.get"
    Ident(String),
    /// String literal "hello"
    String(String),
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// End of input
    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its kind and source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Lexer for tokenizing S-expression input.
pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, ch)| ch)
    }

    /// Advance to the next character and return it.
    fn next_char(&mut self) -> Option<char> {
        if let Some((pos, ch)) = self.chars.next() {
            self.pos = pos + ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Skip whitespace and comments.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            } else if ch == ';' {
                // Skip line comment starting with ;;
                self.next_char();
                if self.peek_char() == Some(';') {
                    // Skip until end of line
                    while let Some(ch) = self.peek_char() {
                        if ch == '\n' {
                            break;
                        }
                        self.next_char();
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Read an identifier (letters, digits, dots, underscores, and WAT-specific chars).
    fn read_identifier(&mut self, start: usize) -> String {
        let mut end = start;
        while let Some(ch) = self.peek_char() {
            // Include '=' for WAT immediates like offset=12, align=4
            if ch.is_alphanumeric() || ch == '.' || ch == '_' || ch == '=' {
                self.next_char();
                end = self.pos;
            } else {
                break;
            }
        }
        self.input[start..end].to_string()
    }

    /// Read a number (integer or float, with optional hex prefix).
    fn read_number(&mut self, start: usize, first_char: char) -> Result<TokenKind, LexError> {
        let loc = SourceLocation::new(start, self.line, self.column - 1);
        let is_negative = first_char == '-';

        // Check for hex prefix
        if first_char == '0' || (is_negative && self.peek_char() == Some('0')) {
            if is_negative {
                self.next_char(); // consume '0'
            }
            if self.peek_char() == Some('x') || self.peek_char() == Some('X') {
                self.next_char(); // consume 'x'
                return self.read_hex_number(start, is_negative, loc);
            }
        }

        // Read decimal digits
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.next_char();
            } else if ch == '.' && !has_dot && !has_exp {
                has_dot = true;
                self.next_char();
            } else if (ch == 'e' || ch == 'E') && !has_exp {
                has_exp = true;
                has_dot = true; // treat as float
                self.next_char();
                // Optional sign after exponent
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.next_char();
                }
            } else {
                break;
            }
        }

        let text = &self.input[start..self.pos];

        if has_dot {
            text.parse::<f64>()
                .map(TokenKind::Float)
                .map_err(|_| LexError::InvalidNumber {
                    text: text.to_string(),
                    loc,
                })
        } else {
            text.parse::<i64>()
                .map(TokenKind::Integer)
                .map_err(|_| LexError::InvalidNumber {
                    text: text.to_string(),
                    loc,
                })
        }
    }

    /// Read a hexadecimal number.
    fn read_hex_number(
        &mut self,
        start: usize,
        is_negative: bool,
        loc: SourceLocation,
    ) -> Result<TokenKind, LexError> {
        let hex_start = self.pos;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_hexdigit() {
                self.next_char();
            } else {
                break;
            }
        }

        let hex_text = &self.input[hex_start..self.pos];
        if hex_text.is_empty() {
            return Err(LexError::InvalidNumber {
                text: self.input[start..self.pos].to_string(),
                loc,
            });
        }

        i64::from_str_radix(hex_text, 16)
            .map(|n| TokenKind::Integer(if is_negative { -n } else { n }))
            .map_err(|_| LexError::InvalidNumber {
                text: self.input[start..self.pos].to_string(),
                loc,
            })
    }

    /// Read a string literal.
    fn read_string(&mut self, start: usize) -> Result<TokenKind, LexError> {
        let mut value = String::new();
        let loc = SourceLocation::new(start, self.line, self.column - 1);

        loop {
            match self.next_char() {
                Some('"') => break,
                Some('\\') => {
                    // Handle escape sequences
                    match self.next_char() {
                        Some('n') => value.push('\n'),
                        Some('t') => value.push('\t'),
                        Some('r') => value.push('\r'),
                        Some('\\') => value.push('\\'),
                        Some('"') => value.push('"'),
                        Some(ch) => {
                            value.push('\\');
                            value.push(ch);
                        }
                        None => {
                            return Err(LexError::UnexpectedEof { loc });
                        }
                    }
                }
                Some(ch) => value.push(ch),
                None => {
                    return Err(LexError::UnexpectedEof { loc });
                }
            }
        }

        Ok(TokenKind::String(value))
    }

    /// Skip a block comment (;...;), handling nested comments.
    /// Called after '(' has been consumed and ';' is peeked.
    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let start = self.pos;
        self.next_char(); // consume ';'
        let mut depth = 1;

        while depth > 0 {
            match self.next_char() {
                Some(';') if self.peek_char() == Some(')') => {
                    self.next_char(); // consume ')'
                    depth -= 1;
                }
                Some('(') if self.peek_char() == Some(';') => {
                    self.next_char(); // consume ';'
                    depth += 1;
                }
                Some(_) => {}
                None => {
                    return Err(LexError::UnexpectedEof {
                        loc: SourceLocation::new(start, self.line, self.column),
                    });
                }
            }
        }
        Ok(())
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let start = self.pos;

        let Some(ch) = self.next_char() else {
            return Ok(Token::new(TokenKind::Eof, Span::new(start, start)));
        };

        let kind = match ch {
            '(' => {
                // Check for block comment (;...;)
                if self.peek_char() == Some(';') {
                    self.skip_block_comment()?;
                    return self.next_token();
                }
                TokenKind::LParen
            }
            ')' => TokenKind::RParen,
            '"' => self.read_string(start)?,
            '-' => {
                // Could be negative number or identifier starting with -
                if let Some(next) = self.peek_char() {
                    if next.is_ascii_digit() {
                        self.read_number(start, '-')?
                    } else {
                        TokenKind::Ident(self.read_identifier(start))
                    }
                } else {
                    TokenKind::Ident("-".to_string())
                }
            }
            '0'..='9' => self.read_number(start, ch)?,
            _ if ch.is_alphabetic() || ch == '_' || ch == '$' => {
                TokenKind::Ident(self.read_identifier(start))
            }
            _ => {
                return Err(LexError::UnexpectedCharacter {
                    ch,
                    loc: SourceLocation::new(start, self.line, self.column - 1),
                });
            }
        };

        Ok(Token::new(kind, Span::new(start, self.pos)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("()");
        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::LParen
        ));
        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::RParen
        ));
        assert!(matches!(lexer.next_token().unwrap().kind, TokenKind::Eof));
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("i64.add");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Ident("i64.add".to_string()));
    }

    #[test]
    fn test_integer() {
        let mut lexer = Lexer::new("42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(42));
    }

    #[test]
    fn test_negative_integer() {
        let mut lexer = Lexer::new("-42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(-42));
    }

    #[test]
    fn test_hex_integer() {
        let mut lexer = Lexer::new("0xFF");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(255));
    }

    #[test]
    fn test_float() {
        let mut lexer = Lexer::new("3.14");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token.kind, TokenKind::Float(f) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_float_exp() {
        let mut lexer = Lexer::new("1.5e10");
        let token = lexer.next_token().unwrap();
        assert!(matches!(token.kind, TokenKind::Float(f) if (f - 1.5e10).abs() < 1.0));
    }

    #[test]
    fn test_expression() {
        let mut lexer = Lexer::new("(i64.add (i64.const 1) (i64.const 2))");
        let tokens: Vec<_> = std::iter::from_fn(|| {
            let t = lexer.next_token().ok()?;
            if matches!(t.kind, TokenKind::Eof) {
                None
            } else {
                Some(t.kind)
            }
        })
        .collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::LParen,
                TokenKind::Ident("i64.add".to_string()),
                TokenKind::LParen,
                TokenKind::Ident("i64.const".to_string()),
                TokenKind::Integer(1),
                TokenKind::RParen,
                TokenKind::LParen,
                TokenKind::Ident("i64.const".to_string()),
                TokenKind::Integer(2),
                TokenKind::RParen,
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new(";; comment\n42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(42));
    }

    #[test]
    fn test_string() {
        let mut lexer = Lexer::new("\"hello\"");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String("hello".to_string()));
    }

    #[test]
    fn test_string_with_escapes() {
        let mut lexer = Lexer::new("\"hello\\nworld\"");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_import_tokens() {
        let mut lexer = Lexer::new("(import \"console\" \"log\")");
        let tokens: Vec<_> = std::iter::from_fn(|| {
            let t = lexer.next_token().ok()?;
            if matches!(t.kind, TokenKind::Eof) {
                None
            } else {
                Some(t.kind)
            }
        })
        .collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::LParen,
                TokenKind::Ident("import".to_string()),
                TokenKind::String("console".to_string()),
                TokenKind::String("log".to_string()),
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn test_block_comment() {
        let mut lexer = Lexer::new("(;comment;) 42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(42));
    }

    #[test]
    fn test_block_comment_with_index() {
        // This is how type indices appear in WAT: (type (;0;) ...)
        let mut lexer = Lexer::new("(type (;0;) (func))");
        let tokens: Vec<_> = std::iter::from_fn(|| {
            let t = lexer.next_token().ok()?;
            if matches!(t.kind, TokenKind::Eof) {
                None
            } else {
                Some(t.kind)
            }
        })
        .collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::LParen,
                TokenKind::Ident("type".to_string()),
                TokenKind::LParen,
                TokenKind::Ident("func".to_string()),
                TokenKind::RParen,
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn test_nested_block_comment() {
        let mut lexer = Lexer::new("(; outer (; inner ;) outer ;) 42");
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenKind::Integer(42));
    }
}
