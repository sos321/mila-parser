use ::std::collections::HashMap;
use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: u32,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: u32) -> Self {
        Token { kind, lexeme, line }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // literals
    Number { value: String, base: u32 },
    StringLiteral(String),

    // identifiers
    Identifier(String),

    // keywords
    Begin,
    End,
    Const,
    Procedure,
    Forward,
    Function,
    If,
    Then,
    Else,
    Program,
    While,
    Exit,
    Var,
    Integer,
    For,
    Do,
    To,
    Downto,
    Array,
    Of,

    // predefined functions
    WriteLn,
    ReadLn,
    Write,

    // keyword operators
    Mod,
    Div,
    Not,
    And,
    Xor,
    Or,

    // symbol operators
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Assign,       // :=
    Equal,        // =
    NotEqual,     // <>
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=

    // punctuation
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    Semicolon,    // ;
    Colon,        // :
    Comma,        // ,
    DoubleDot,    // ..
    Dot,          // .

    // control
    EndOfFile,

    // error
    Error(String),
}

lazy_static! {
    pub static ref KeyWords: HashMap<&'static str, TokenKind> = {
        use TokenKind::*;

        let keywords = vec![
            ("begin", Begin),
            ("end", End),
            ("const", Const),
            ("procedure", Procedure),
            ("forward", Forward),
            ("function", Function),
            ("if", If),
            ("then", Then),
            ("else", Else),
            ("program", Program),
            ("while", While),
            ("exit", Exit),
            ("var", Var),
            ("integer", Integer),
            ("for", For),
            ("do", Do),
            ("to", To),
            ("downto", Downto),
            ("array", Array),
            ("of", Of),
            ("writeln", WriteLn),
            ("write", Write),
            ("readln", ReadLn),
            ("mod", Mod),
            ("div", Div),
            ("not", Not),
            ("and", And),
            ("xor", Xor),
            ("or", Or),
        ];

        let map: HashMap<&'static str, TokenKind> = keywords.iter().cloned().collect();

        map
    };
}
