extern crate num_derive;
extern crate num_traits as num_derived_traits;

pub mod input_stream;
pub mod lexer;
pub mod errors;
pub mod parser;

use errors::InvalidTokenError;
use input_stream::InputStream;
use lexer::Token;
use lexer::TokenStream;
use lexer::TokenType;
use parser::{Parser};
use rustyline::history::FileHistory;
use std::env;
use std::fs;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use parser::{AstPrinter, Expr};

fn main() {
    let args: Vec<String> = env::args().collect();
    print_splash_screen();

    return match args.get(1) {
        Some(filename) => compile(filename),
        None => start_prompt().expect("Something went wrong"),
    };
}

fn compile(filename: &String) {
    let contents = fs::read_to_string(filename.as_str()).expect("Error when opening file");

    let tokens = tokenize(&contents);

    if tokens.len() == 0 {
        println!("The lexer finished with errors. Aborting.");
        return;
    }
    let mut parser = Parser::new(&tokens);
    let expr = parser.parse();
    println!("{}", expr.len());

    let ast = match &expr[0] {
        Ok(a) => a,
        Err(e) => panic!("This is not an AST")
    };

    let mut printer = AstPrinter;
    let result = ast.accept(&mut printer);
    println!("{}", result);
}

fn print_splash_screen() {
    println!("\x1b[1mWolff interpreter {}\x1b[0m", env!("CARGO_PKG_VERSION"));
}

fn start_prompt() -> Result<()> {

    let mut rl = Editor::<(), FileHistory>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("\x1b[1mλ\x1b[0m ");

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                start_lexer(&line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("Interruption detected. Halting.");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("EOF reached. Halting.");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt")
}

fn start_lexer_from_file(filename: &String) -> Result<()> {
    let contents = fs::read_to_string(filename.as_str()).expect("Error when opening file");

    let return_value = start_lexer(&contents);
    return Result::Ok(return_value)
}

fn print_error_message(error: &errors::InvalidTokenError) {
    println!("[\x1b[91mERR\x1b[0m] {}", error.message);
    println!("  \x1b[96m|\x1b[0m {}", error.line_as_string);
    println!("  \x1b[96m|\x1b[0m \x1b[93m{:>width$}\x1b[0m", "^", width = (error.col+1) as usize);
}

fn start_lexer(contents: &String) {
    let mut input_stream = InputStream::new(&contents);
    let mut lexer = TokenStream::new(&mut input_stream);

    while !lexer.eof() {
        match lexer.next() {
            Ok(Token { token_type: lexer::TokenType::EOF, line: _, col: _, lexeme: _}) => println!("EOF reached"),
            Ok(new_token) => println!("{}: {}", new_token.token_type, new_token.lexeme),
            Err(e) => {
                print_error_message(&e);
            }
        };
    }
}

fn tokenize(contents: &String) -> Vec<Token> {
    let mut input_stream = InputStream::new(&contents);
    let mut lexer = TokenStream::new(&mut input_stream);

    let mut token_vector: Vec<Token> = Vec::new();

    while !lexer.eof() {
        match lexer.next() {
            Ok(new_token) => token_vector.push(new_token),
            Err(e) => {
                print_error_message(&e);
            }
        };
    }

    if !lexer.has_error {
        return token_vector;
    } else {
        return Vec::new();
    }
}