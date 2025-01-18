use crate::errors::InvalidTokenError;
use crate::input_stream::InputStream;

use regex::Regex;
use std::fmt;

// VERY IMPORTANT that this list stays ordered lexicographically, otherwise the lexer breaks
const KEYWORDS: &'static [&'static str] = &[
    "and", "class", "else", "false", "for", "fun", "if", "lambda", "nil", "or", "print", "return", "super", "this", "true", "var", "while",
    "λ"
];
const PUNCTS: &'static [char] = &['!', '%', '&', '(', ')', '*', '+', '+', ',', '-', '-', '.', '/', ';', '<', '=', '>', '^', '{', '|', '}'];

pub struct TokenStream {
    input: InputStream,
    current: Token,
    has_started: bool,
    pub has_error: bool,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier(String),
    String(String),
    Number(f64),
    // Keywords
    Keyword(String),
    // EOF token
    EOF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_token = match self {
            // Single character tokens
            TokenType::LeftParen => "LeftParen",
            TokenType::RightParen => "RightParen",
            TokenType::LeftBrace => "LeftBrace",
            TokenType::RightBrace => "RightBrace",
            TokenType::Comma => "Comma",
            TokenType::Dot => "Dot",
            TokenType::Minus => "Minus",
            TokenType::Plus => "Plus",
            TokenType::Semicolon => "Semicolon",
            TokenType::Slash => "Slash",
            TokenType::Star => "Star",
            // One or two character tokens
            TokenType::BangEqual => "BangEqual",
            TokenType::Bang => "Bang",
            TokenType::Equal => "Equal",
            TokenType::EqualEqual => "EqualEqual",
            TokenType::Greater => "Greater",
            TokenType::GreaterEqual => "GreaterEqual",
            TokenType::Less => "Less",
            TokenType::LessEqual => "LessEqual",
            // Literals
            TokenType::Identifier(_) => "Identifier",
            TokenType::String(_) => "String",
            TokenType::Number(_) => "Number",
            // Keywords
            TokenType::Keyword(_) => "Keyword",
            // EOF token
            TokenType::EOF => "EOF",
        };
        write!(f, "{}", string_token.to_owned())
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::EOF
    }
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &String, line: usize, col: usize) -> Self {
        Token {
            token_type,
            lexeme: lexeme.to_owned(),
            line,
            col
        }
    }
}

impl TokenStream {
    pub fn new(input: &mut InputStream) -> Self {
        TokenStream {
            input: input.clone(),
            current: Token::new(TokenType::default(), &String::default(), 0, 0),
            has_started: false,
            has_error: false,
        }
    }

    fn read_next(&mut self) -> Result<Token, InvalidTokenError> {
        // If the input char is whitespace, continue reading until it isn't
        self.read_while(&mut is_whitespace);

        // If input is EOF, return EOF token
        if self.input.eof() {
            return Ok(Token::new(TokenType::EOF, &String::default(), self.input.line, self.input.col));
        }

        // Peek at the next character in the input stream to figure out what we need to do
        let ch = self.input.peek();

        // The next line is a comment, so ignore it and try again after newline
        if ch == '#' {
            self.skip_comment();
            return self.read_next();
        }

        if ch == '"' {
            let string_token = self.read_string();
            if string_token.is_none() {
                self.has_error = true;
                return Err(InvalidTokenError {
                    message: format!("Invalid string termination at {}:{}", self.input.line, self.input.col),
                    line_as_string: self.input.get_current_line().to_string(),
                    line: self.input.line,
                    col: self.input.col,
                });
            } else { return Ok(string_token.unwrap()); }
        }

        if ch.is_digit(10) {
            return Ok(self.read_number());
        }

        if is_id_start(ch) {
            return Ok(self.read_ident());
        }

        if is_punctuation(ch) {
            let punctuation_token = self.read_punctuation();
            if punctuation_token.is_none() {
                self.has_error = true;
                return Err(InvalidTokenError {
                    message: format!("Invalid operator at {}:{}", self.input.line, self.input.col),
                    line_as_string: self.input.get_current_line().to_string(),
                    line: self.input.line,
                    col: self.input.col,
                });
            } else {return Ok(punctuation_token.unwrap());}
        }

        // Illegal character detected here, skip this one and return an error

        let error = Err(InvalidTokenError {
            message: format!(
                "Invalid character at {}:{}",
                self.input.line, self.input.col
            ),
            line_as_string: self.input.get_current_line().to_string(),
            line: self.input.line,
            col: self.input.col,
        });
        self.input.next();
        self.has_error = true;
        error
    }

    fn read_while(&mut self, predicate: &mut dyn FnMut(char) -> bool) -> String {
        let mut return_string = String::new();
        while !self.input.eof() && predicate(self.input.peek()) {
            return_string.push(self.input.next());
        }
        return_string
    }

    fn skip_comment(&mut self) {
        self.read_while(&mut |x| x != '\n');
    }

    fn read_string(&mut self) -> Option<Token> {
        let return_string = self.read_escaped('"');

        // if the string is None return none otherwise return a token
        if return_string.is_none() {
            return None;
        } else {
            let unwrapped_string = return_string.unwrap();
            return Some(Token::new(TokenType::String(unwrapped_string.clone()), &unwrapped_string, self.input.line, self.input.col));
        }
    }

    fn read_escaped(&mut self, end: char) -> Option<String> {
        let mut escaped = false;
        let mut return_string = String::new();

        self.input.next();
        loop {
            let ch = self.input.next();
            if escaped {
                return_string.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == end {
                break;
            } else if self.input.eof() {
                return None
            } else {
                return_string.push(ch);
            }
        }
        Some(return_string)
    }

    fn read_number(&mut self) -> Token {
        let mut has_dec_point = false;
        let number = self.read_while(&mut |ch: char| {
            if ch == '.' {
                if has_dec_point {
                    return false;
                }
                has_dec_point = true;
                return true;
            }
            return ch.is_digit(10);
        });

        let s: f64 = number.parse().unwrap();
        // TODO: The behaviour is the same for float and int, in the future, divide them
        // If it is an integer, return an integer token
        if !has_dec_point {
            return Token::new(TokenType::Number(s), &number, self.input.line, self.input.col)
        }

        // Otherwise return a float token
        Token::new(TokenType::Number(s), &number, self.input.line, self.input.col)
    }

    fn read_ident(&mut self) -> Token {
        let identifier = self.read_while(&mut is_id);

        Token::new(
            if is_keyword(&identifier) {
                TokenType::Keyword(identifier.clone())
            } else {
                TokenType::Identifier(identifier.clone())
            },
            &identifier,
            self.input.line,
            self.input.col
        )
    }

    fn read_punctuation(&mut self) -> Option<Token> {
        // First start looking for single lexemes and get those out of the way
        // so they don't get compounded with other lexemes
        
        let next_char = self.input.peek();
        let single_token_type: TokenType = match next_char {
            ';' => TokenType::Semicolon,
            ',' => TokenType::Comma,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            '.' => TokenType::Dot,
            _ => TokenType::EOF,
        };

        if single_token_type != TokenType::EOF {
            return Some(Token::new(single_token_type, &self.input.next().to_string(), self.input.line, self.input.col));
        }

        let punctuation = self.read_while(&mut is_punctuation);
        let token_type = match punctuation.as_str() {
            "=" => TokenType::Equal,
            "==" => TokenType::EqualEqual,
            "!=" => TokenType::BangEqual,
            ">" => TokenType::Greater,
            ">=" => TokenType::GreaterEqual,
            "<" => TokenType::Less,
            "<=" => TokenType::LessEqual,
            "!" => TokenType::Bang,
            "-" => TokenType::Minus,
            "+" => TokenType::Plus,
            "/" => TokenType::Slash,
            "*" => TokenType::Star,
            _ => return None
        };
        Some(Token::new(
            token_type,
            &punctuation,
            self.input.line,
            self.input.col))
    }

    pub fn peek(&self) -> Option<Token> {
        if !self.has_started {
            Option::None
        } else {
            Option::Some(self.current.clone())
        }
    }

    pub fn next(&mut self) -> Result<Token, InvalidTokenError> {
        if !self.has_started {
            self.has_started = true;
        }
        self.current = match self.read_next() {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        Result::Ok(self.current.clone())
    }

    pub fn eof(&mut self) -> bool {
        !self.peek().is_none() && self.peek().unwrap().token_type == TokenType::EOF
    }
}

fn is_keyword(word: &String) -> bool {
    KEYWORDS.binary_search(&word.as_str()).is_ok()
}

fn is_id_start(ch: char) -> bool {
    Regex::new(r"[[^0-9*]&&\p{Emoji}a-zA-Zλ]")
        .unwrap()
        .is_match(ch.to_string().as_str())
}

fn is_punctuation(ch: char) -> bool {
    PUNCTS.binary_search(&ch).is_ok()
}

fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

fn is_id(ch: char) -> bool {
    is_id_start(ch) || "!?0123456789".contains(ch)
}

#[cfg(test)]
mod tests {
    use super::*; // Import the module you're testing

    #[test]
    fn test_tokenizer_simple_input() {
        let input = "1 + 2".to_string();
        
        let result_of_lexer = run_lexer(&input);
        match result_of_lexer {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 4);

                // Verify specific tokens
                assert_eq!(tokens[0].token_type, TokenType::Number(1.0));
                assert_eq!(tokens[1].token_type, TokenType::Plus);
                assert_eq!(tokens[2].token_type, TokenType::Number(2.0));
                assert_eq!(tokens[3].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_lexer_with_invalid_input() {
        let input = "1 + @".to_string();
        let result = run_lexer(&input);

        match result {
            Ok(_) => panic!("Expected an error, but got tokens"),
            Err(e) => assert!(e.contains("Lexer error")),
        }
    }

    #[test]
    fn test_tokenizer_multiple_operators() {
        let input = "3 * (4 - 2) / 7".to_string();
        
        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 10); // Tokens: 3, *, (, 4, -, 2, ), /, EOF

                assert_eq!(tokens[0].token_type, TokenType::Number(3.0));
                assert_eq!(tokens[1].token_type, TokenType::Star);
                assert_eq!(tokens[2].token_type, TokenType::LeftParen);
                assert_eq!(tokens[3].token_type, TokenType::Number(4.0));
                assert_eq!(tokens[4].token_type, TokenType::Minus);
                assert_eq!(tokens[5].token_type, TokenType::Number(2.0));
                assert_eq!(tokens[6].token_type, TokenType::RightParen);
                assert_eq!(tokens[7].token_type, TokenType::Slash);
                assert_eq!(tokens[8].token_type, TokenType::Number(7.0));
                assert_eq!(tokens[9].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_tokenizer_keywords() {
        let input = "if else while for".to_string();

        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 5); // Keywords + EOF
                
                assert_eq!(tokens[0].token_type, TokenType::Keyword("if".to_string()));
                assert_eq!(tokens[1].token_type, TokenType::Keyword("else".to_string()));
                assert_eq!(tokens[2].token_type, TokenType::Keyword("while".to_string()));
                assert_eq!(tokens[3].token_type, TokenType::Keyword("for".to_string()));
                assert_eq!(tokens[4].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_tokenizer_strings() {
        let input = "\"hello\" + \"world\"".to_string();

        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 4); // Strings + Plus + EOF

                assert_eq!(tokens[0].token_type, TokenType::String("hello".to_string()));
                assert_eq!(tokens[1].token_type, TokenType::Plus);
                assert_eq!(tokens[2].token_type, TokenType::String("world".to_string()));
                assert_eq!(tokens[3].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_lexer_empty_input() {
        let input = "".to_string();

        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 1); // Only EOF token
                assert_eq!(tokens[0].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_lexer_unterminated_string() {
        let input = "\"hello".to_string(); // Unterminated string

        let result = run_lexer(&input);
        match result {
            Ok(_) => panic!("Expected an error, but got tokens"),
            Err(e) => assert!(e.contains("Lexer error")), // Ensure an error is returned
        }
    }

    #[test]
    fn test_lexer_handles_whitespace() {
        let input = "  12   +   3   ".to_string(); // Input with extra spaces

        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 4); // Numbers, operator, and EOF

                assert_eq!(tokens[0].token_type, TokenType::Number(12.0));
                assert_eq!(tokens[1].token_type, TokenType::Plus);
                assert_eq!(tokens[2].token_type, TokenType::Number(3.0));
                assert_eq!(tokens[3].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_lexer_invalid_character_in_expression() {
        let input = "5 + 3 $".to_string(); // Invalid character '$'

        let result = run_lexer(&input);
        match result {
            Ok(_) => panic!("Expected an error, but got tokens"),
            Err(e) => assert!(e.contains("Lexer error")), // Ensure an error is returned
        }
    }

    #[test]
    fn test_lexer_nested_parentheses() {
        let input = "( ( 1 + 2 ) * 3 )".to_string(); // Nested parentheses

        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 10); // Tokens + EOF

                assert_eq!(tokens[0].token_type, TokenType::LeftParen);
                assert_eq!(tokens[1].token_type, TokenType::LeftParen);
                assert_eq!(tokens[2].token_type, TokenType::Number(1.0));
                assert_eq!(tokens[3].token_type, TokenType::Plus);
                assert_eq!(tokens[4].token_type, TokenType::Number(2.0));
                assert_eq!(tokens[5].token_type, TokenType::RightParen);
                assert_eq!(tokens[6].token_type, TokenType::Star);
                assert_eq!(tokens[7].token_type, TokenType::Number(3.0));
                assert_eq!(tokens[8].token_type, TokenType::RightParen);
                assert_eq!(tokens[9].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_lexer_large_numbers() {
        let input = "1234567890 + 9876543210".to_string();

        let result = run_lexer(&input);
        match result {
            Ok(tokens) => {
                assert_eq!(tokens.len(), 4); // Numbers + Plus + EOF

                assert_eq!(tokens[0].token_type, TokenType::Number(1234567890.0));
                assert_eq!(tokens[1].token_type, TokenType::Plus);
                assert_eq!(tokens[2].token_type, TokenType::Number(9876543210.0));
                assert_eq!(tokens[3].token_type, TokenType::EOF);
            },
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    // Abstraction for running the lexer for an arbitrary input, for testing only
    fn run_lexer(input: &String) -> Result<Vec<Token>, String> {
        let mut input_stream = InputStream::new(input);
        let mut lexer = TokenStream::new(&mut input_stream);

        let mut tokens = Vec::new();
        while !lexer.eof() {
            match lexer.next() {
                Ok(new_token) => tokens.push(new_token),
                Err(e) => {
                    return Err(format!("Lexer error: {}", e));
                }
            }
        }

        Ok(tokens)
    }
}