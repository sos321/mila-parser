use std::env;
use std::io::{self, Read};

mod lexer;
use lexer::{Lexer, print};

mod parser;
use parser::Parser;

mod code_gen;

fn read_stdin_to_string() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn lex(code: String) {
    // create lexer
    let lexer = Lexer::new(&code);

    // lex code
    println!("Lexing code ...\n");
    print(lexer);
}

fn tree(code: String) {
    // create lexer
    let lexer = Lexer::new(&code);
    let mut parser = Parser::new(lexer);

    // parse code to AST
    let ast = parser.parse_program();
    match ast {
        Ok(program) => {
            program.print_ast();
        }
        Err(err) => {
            println!("{}", err);
            return;
        }
    }

    return;
}

fn compile(code: String) {
    println!("Not yet supoorted for {}", code);
    return;
}

fn main() {
    // get args
    let args: Vec<String> = env::args().collect();

    // checks args
    if args.len() > 2 {
        eprintln!("Too many options");
        std::process::exit(1);
    }

    let cmd = if args.iter().len() == 2 {
        args[1].clone()
    } else {
        "".to_string()
    };

    // read code from stdin
    let code = if let Ok(s) = read_stdin_to_string() {
        s
    } else {
        eprintln!("Failed to read file contents.");
        std::process::exit(1);
    };

    match cmd.as_str() {
        "--lex" => {
            lex(code);
        }
        "--tree" => {
            tree(code);
        }
        "" => {
            compile(code);
        }
        _ => {
            eprintln!("Invalid argument!");
            std::process::exit(1);
        }
    }
}
