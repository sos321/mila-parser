use std::iter::Peekable;
use std::str::Chars;

use super::tokens::{KeyWords, Token, TokenKind};

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_line: u32,
    ended: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            current_line: 1,
            ended: false,
        }
    }

    fn advance(&mut self) -> Option<char> {
        // count lines
        if self.input.peek().unwrap_or(&'e') == &'\n' {
            self.current_line += 1;
        }

        self.input.next()
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                // whitespace
                Some(&c) if c.is_whitespace() => {
                    self.advance();
                }

                // comment
                Some(&'{') => {
                    self.advance();

                    // ignore everything to '}'
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            self.current_line += 1;
                        }
                        if c == '}' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn read_identifier(&mut self, first: char) -> String {
        let mut ident = String::new();
        ident.push(first);

        // read until whitespace or unrecognized char
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        ident
    }

    fn is_digit(c: char, base: u32) -> bool {
        match base {
            16 => c.is_ascii_hexdigit(),
            10 => c.is_ascii_digit(),
            8 => c >= '0' && c <= '7',
            _ => false, // unknow base
        }
    }

    fn read_number(&mut self, first: char, base: u32) -> String {
        let mut number = String::new();
        number.push(first);

        // read digits
        while let Some(&c) = self.peek() {
            if Self::is_digit(c, base) {
                number.push(self.advance().unwrap());
            } else if base == 10 && c == '.' {
                // floats are not implemented
                // should probably throw error
                break;
            } else {
                // end string
                break;
            }
        }

        let value_str = match base {
            16 => number.strip_prefix("$").unwrap_or(&number).to_string(),
            8 => number.strip_prefix("&").unwrap_or(&number).to_string(),
            _ => number.clone(), // Base 10
        };

        value_str
    }

    fn read_string_literal(&mut self) -> Result<String, String> {
        let mut value = String::new();

        loop {
            match self.advance() {
                Some('\'') => break Ok(value), // end of string
                Some('\\') => {
                    // character escapes
                    match self.advance() {
                        Some('n') => value.push('\n'),
                        Some('t') => value.push('\t'),
                        Some('r') => value.push('\r'),
                        Some('\\') => value.push('\\'),
                        Some('\'') => value.push('\''),
                        Some(other) => {
                            value.push('\\');
                            value.push(other);
                        }

                        // EOF in escpae sequence
                        None => break Err("Unterminated escape sequence".to_string()),
                    }
                }
                Some('\n') => break Err("Unterminated string literal".to_string()),
                Some(c) => value.push(c),

                // EOF before closing quote
                None => break Err("Unterminated string literal".to_string()),
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        match self.advance() {
            Some(c) => {
                let mut lex_tok = c.to_string();
                let kind = match c {
                    '+' => TokenKind::Plus,
                    '-' => TokenKind::Minus,
                    '*' => TokenKind::Multiply,
                    '(' => TokenKind::LeftParen,
                    ')' => TokenKind::RightParen,
                    '[' => TokenKind::LeftBracket,
                    ']' => TokenKind::RightBracket,
                    ';' => TokenKind::Semicolon,
                    ',' => TokenKind::Comma,
                    '=' => TokenKind::Equal,

                    ':' => {
                        if let Some(&'=') = self.peek() {
                            self.advance();
                            lex_tok.push('=');
                            TokenKind::Assign
                        } else {
                            TokenKind::Colon
                        }
                    }
                    '<' => {
                        if let Some(&'=') = self.peek() {
                            self.advance();
                            lex_tok.push('=');
                            TokenKind::LessEqual
                        } else if let Some('>') = self.peek() {
                            self.advance();
                            lex_tok.push('>');
                            TokenKind::NotEqual
                        } else {
                            TokenKind::Less
                        }
                    }
                    '>' => {
                        if let Some(&'=') = self.peek() {
                            self.advance();
                            lex_tok.push('=');
                            TokenKind::GreaterEqual
                        } else {
                            TokenKind::Greater
                        }
                    }
                    '.' => {
                        if let Some(&'.') = self.peek() {
                            self.advance();
                            lex_tok.push('.');
                            TokenKind::DoubleDot
                        } else {
                            TokenKind::Dot
                        }
                    }

                    // string literal
                    '\'' => match self.read_string_literal() {
                        Ok(value) => {
                            lex_tok = format!("\"{}\"", value);
                            TokenKind::StringLiteral(value)
                        }

                        Err(msg) => {
                            TokenKind::Error(format!("{} on line {}", msg, self.current_line))
                        }
                    },

                    // hexadecimal number
                    '$' => {
                        let num_value = self.read_number(c, 16);
                        lex_tok = num_value.clone();
                        TokenKind::Number {
                            value: num_value,
                            base: 16,
                        }
                    }

                    // octal number
                    '&' => {
                        let num_value = self.read_number(c, 8);
                        lex_tok = num_value.clone();
                        TokenKind::Number {
                            value: num_value,
                            base: 8,
                        }
                    }

                    // decimal number
                    c if c.is_ascii_digit() => {
                        let num_value = self.read_number(c, 10);
                        lex_tok = num_value.clone();
                        TokenKind::Number {
                            value: num_value,
                            base: 10,
                        }
                    }

                    // string (ident or keyword)
                    c if c.is_alphabetic() || c == '_' => {
                        let ident = self.read_identifier(c);
                        lex_tok = ident.clone();

                        // try to match keyword
                        match KeyWords.get(&ident.as_str()) {
                            Some(keyword) => keyword.clone(),
                            _ => TokenKind::Identifier(ident),
                        }
                    }

                    // unmatched character
                    _ => TokenKind::Error(format!(
                        "Unexpected character: {} on line {}",
                        c, self.current_line
                    )),
                };

                Token::new(kind, lex_tok, self.current_line)
            }

            // eof
            None => Token::new(TokenKind::EndOfFile, "".to_string(), self.current_line),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        let token = self.next_token();
        if let TokenKind::EndOfFile = token.kind {
            self.ended = true;
        }

        Some(token)
    }
}
