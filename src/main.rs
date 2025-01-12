extern crate num_derive;
extern crate num_traits as num_derived_traits;

pub mod input_stream;
pub mod lexer;
pub mod errors;
pub mod parser;

use colored::*;
use input_stream::InputStream;
use lexer::Token;
use lexer::TokenStream;
use parser::{Parser, AstInterpreter, AstPrinter};
use rustyline::history::FileHistory;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use std::env;
use std::fs;


fn main() {
    let args: Vec<String> = env::args().collect();
    print_splash_screen();

    return match args.get(1) {
        Some(filename) => interpret_file(filename),
        None => start_prompt().expect("Something went wrong"),
    };
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
                interpret_string(&line);
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

fn interpret_file(filename: &String) {
    let contents = fs::read_to_string(filename.as_str()).expect("Error when opening file");
    interpret_string(&contents);
}

fn interpret_string(source_code: &String) {
    let tokens = tokenize(&source_code);

    if tokens.len() == 0 {
        println!("The lexer finished with errors. Aborting.");
        return;
    }

    let mut parser = Parser::new(&tokens);
    let result = parser.parse();
    println!("{}", result.len());

    for stmt in result.iter() {
        match &stmt {
            Ok(a) => {
                let mut printer = AstPrinter;
                let result = a.accept(&mut printer);
                print_text_with_blue(&"Abstract syntax tree".to_string());
                println!("{}", result);

                let mut interpreter = AstInterpreter;
                let evaluation_result = a.accept(&mut interpreter);

                match evaluation_result {
                    Ok(evaled) => {
                        print_text_with_blue(&"Evaluation result".to_string());
                        println!("{:?}", evaled);
                    },
                    Err(e) => {
                        println!("{e}");
                    }
                }
            },
            Err(e) => {
                println!("{}:{} {}", e.line, e.col, e.message);
            }
        };
    }
}

fn print_splash_screen() {
    println!("\x1b[1mWolff interpreter {}\x1b[0m", env!("CARGO_PKG_VERSION"));
}

fn print_error_message(error: &errors::InvalidTokenError) {
    println!("[\x1b[91mERR\x1b[0m] {}", error.message);
    println!("  \x1b[96m|\x1b[0m {}", error.line_as_string);
    println!("  \x1b[96m|\x1b[0m \x1b[93m{:>width$}\x1b[0m", "^", width = (error.col+1) as usize);
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

#[allow(dead_code)]
fn print_text_with_red(message: &String) {
    let colored_message = message.red().bold();
    println!("{colored_message}");
}

fn print_text_with_blue(message: &String) {
    let colored_message = message.blue().bold();
    println!("{colored_message}");
}

fn print_text_with_green(message: &String) {
    let colored_message = message.green().bold();
    println!("{colored_message}");
}