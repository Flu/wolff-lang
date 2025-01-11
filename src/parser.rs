use crate::lexer::*;

struct Parser<'a> {
    current: usize,
    token_vector: &'a Vec<Token>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {

    #[allow(dead_code)]
    pub fn new(token_vector: &'a Vec<Token>) -> Self {
        Parser {
            current: 0,
            token_vector,
            had_error: false,
            panic_mode: false
        }
    }

    #[allow(dead_code)]
    pub fn compile(&self) -> bool {

        !self.had_error
    }

    #[allow(dead_code)]
    pub fn error_at(&mut self, token: &Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        println!("Error at {}:{}", token.line, token.col);
        println!("{}", message);
        self.had_error = true;
    }

    #[allow(dead_code)]
    pub fn advance(&mut self) {
        self.current += 1;
    }

    #[allow(dead_code)]
    pub fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.token_vector[self.current].token_type == token_type {
            self.advance();
            return;
        }
        
        self.error_at(&self.token_vector[self.current], message);
    }
}