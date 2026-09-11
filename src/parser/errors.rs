use crate::lexer::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
    token: Option<Token>,
    line: Option<u32>,
}

impl ParseError {
    pub fn new(message: String, line: Option<u32>) -> Self {
        ParseError {
            message,
            token: None,
            line,
        }
    }
    pub fn new_with_token(message: String, token: Token) -> Self {
        ParseError {
            message,
            line: Some(token.line),
            token: Some(token),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            if let Some(tok) = &self.token {
                write!(
                    f,
                    "Parse Error at [line: {} near '{}']: {}",
                    line, tok.lexeme, self.message
                )
            } else {
                write!(f, "Parse Error at [line: '{}']: {}", line, self.message)
            }
        } else {
            write!(f, "Parse Error: {}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
pub struct SemanticError {
    message: String,
    token: Option<Token>,
    line: u32,
}

impl SemanticError {
    pub fn new(message: String, line: u32) -> Self {
        SemanticError {
            message,
            token: None,
            line,
        }
    }
    pub fn new_with_token(message: String, token: Token) -> Self {
        SemanticError {
            message,
            line: token.line,
            token: Some(token),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(tok) = &self.token {
            write!(
                f,
                "Semantic Error at [line: {} near '{}']: {}",
                self.line, tok.lexeme, self.message
            )
        } else {
            write!(
                f,
                "Semantic Error at [line: {}]: {}",
                self.line, self.message
            )
        }
    }
}

impl std::error::Error for SemanticError {}
