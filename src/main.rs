pub mod input_stream;
pub mod lexer;

use input_stream::InputStream;
use lexer::TokenStream;
use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(filename) => start_lexer_from_file(filename),
        None => start_prompt(),
    };
}

fn start_prompt() {
    let mut buffer = String::new();

    loop {
        print!(">>");
        stdout().flush().unwrap();
        let bytes = stdin()
            .read_line(&mut buffer)
            .ok()
            .expect("Failed to read line");
        if bytes == 0 {
            println!("\nEOF reached. Exiting.");
            break;
        }
        start_lexer(&buffer);
        buffer.clear();
    }
}

fn start_lexer_from_file(filename: &String) {
    let contents = fs::read_to_string(filename.as_str()).expect("Error when opening file");

    start_lexer(&contents);
}

fn start_lexer(contents: &String) {
    let mut input_stream = InputStream::new(&contents);
    let mut lexer = TokenStream::new(&mut input_stream);

    while !lexer.eof() {
        let new_token = lexer.next().unwrap();
        println!("{}: {}", new_token.token_type, new_token.value)
    }
}
