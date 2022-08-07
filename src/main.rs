pub mod input_stream;
pub mod lexer;

use input_stream::{InputStream};
use lexer::{TokenStream};
use std::fs;

#[test]
fn test_input_stream() {
    let contents = fs::read_to_string("/home/flu/Documents/main.lsp")
        .expect("Something went wrong reading the file");

    let mut input_stream = InputStream::new(&contents);

    while !input_stream.eof() {
        let c = input_stream.peek();
        input_stream.next();
        print!("{}", c);
    }
}

#[test]
fn test_lexer() {
    let contents = fs::read_to_string("/home/flu/Documents/main.lsp")
        .expect("Something went wrong reading the file");

    let mut input_stream = InputStream::new(&contents);

    let lexer = TokenStream::new(&mut input_stream);
}

fn main() {

    let contents = fs::read_to_string("/home/flu/Documents/main.lsp")
        .expect("Something went wrong reading the file");

    let mut input_stream = InputStream::new(&contents);

    let lexer = TokenStream::new(&mut input_stream);

}
