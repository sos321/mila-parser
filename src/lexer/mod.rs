pub(crate) mod lexer;
pub(crate) mod tokens;

pub use lexer::Lexer;
pub use tokens::{Token, TokenKind};

pub fn print(lexer: Lexer) {
    for token in lexer {
        println!("{:?}", token);
        if let TokenKind::Error(msg) = token.kind {
            eprintln!("Lexer Error: {}", msg);
            return;
        }
    }
}
