pub mod input_stream;
pub mod lexer;
pub mod errors;

use input_stream::InputStream;
use lexer::TokenStream;
use std::env;
use std::fs;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

fn main() {
    let args: Vec<String> = env::args().collect();
    print_splash_screen();

    match args.get(1) {
        Some(filename) => start_lexer_from_file(filename).expect("Something went wrong while reading the file"),
        None => start_prompt().expect("Something went wrong"),
    };
}

fn print_splash_screen() {
    println!("\x1b[1mWolff interpreter v0.1.0\x1b[0m");
}

fn start_prompt() -> Result<()> {

    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("\x1b[1mλ \x1b[0m");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
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

    start_lexer(&contents);
    return Result::Ok(())
}

fn print_error_message(message: &String) {
    println!("[\x1b[91mERR\x1b[0m] {}", message);
}

fn start_lexer(contents: &String) {
    let mut input_stream = InputStream::new(&contents);
    let mut lexer = TokenStream::new(&mut input_stream);

    while !lexer.eof() {
        match lexer.next() {
            Ok(new_token) => println!("{}: {}", new_token.token_type, new_token.value),
            Err(e) => { 
                print_error_message(&e.message);
                break;
            }
        };
    }
}