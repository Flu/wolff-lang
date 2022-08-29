#[macro_use]
extern crate num_derive;
extern crate num_traits as num_derived_traits;

pub mod input_stream;
pub mod lexer;
pub mod errors;
pub mod parser;
pub mod vm;

use input_stream::InputStream;
use lexer::TokenStream;
use vm::{VM, OpCode, Constant};
use std::env;
use std::fs;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

fn main() {
    let args: Vec<String> = env::args().collect();
    print_splash_screen();

    // Chunk testing area
    let mut vm = VM::new(true, true);

    let mut offset = vm.chunk.add_constant(Constant::Integer(45688874));
    vm.chunk.write_chunk(OpCode::Constant as u8, 0);
    vm.chunk.write_chunk(offset, 0);

    offset = vm.chunk.add_constant(Constant::Float(1.2356));
    vm.chunk.write_chunk(OpCode::Constant as u8, 1);
    vm.chunk.write_chunk(offset, 1);
    offset = vm.chunk.add_constant(Constant::Float(256.235444));
    vm.chunk.write_chunk(OpCode::Constant as u8, 2);
    vm.chunk.write_chunk(offset, 2);
    offset = vm.chunk.add_constant(Constant::Float(4589845542425.2));
    vm.chunk.write_chunk(OpCode::Constant as u8, 3);
    vm.chunk.write_chunk(offset, 3);
    offset = vm.chunk.add_constant(Constant::Integer(10));
    vm.chunk.write_chunk(OpCode::Constant as u8, 3);
    vm.chunk.write_chunk(offset, 3);
    offset = vm.chunk.add_constant(Constant::Integer(-5));
    vm.chunk.write_chunk(OpCode::Constant as u8, 3);
    vm.chunk.write_chunk(offset, 3);
    offset = vm.chunk.add_constant(Constant::Integer(800));
    vm.chunk.write_chunk(OpCode::Constant as u8, 3);
    vm.chunk.write_chunk(offset, 3);

    vm.chunk.write_chunk(OpCode::Negate as u8, 4);
    vm.chunk.write_chunk(OpCode::Negate as u8, 4);

    vm.chunk.write_chunk(OpCode::Addition as u8, 4);
    vm.chunk.write_chunk(OpCode::Subtraction as u8, 4);
    
    vm.chunk.write_chunk(OpCode::Return as u8, 4);

    let result_code = vm.interpret();
    println!("VM returned status code {}", result_code);

    // End chunk testing area

    return match args.get(1) {
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
            Ok(new_token) => println!("{}: {}", new_token.token_type, new_token.value),
            Err(e) => {
                print_error_message(&e);
            }
        };
    }

    println!("There was an error in the tokenizer: {}", lexer.has_error);
}