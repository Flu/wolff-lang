use crate::errors::InvalidTokenError;
use crate::input_stream::InputStream;

use regex::Regex;
use std::fmt;

const KEYWORDS: &'static [&'static str] = &[
    "if", "else", "lambda", "λ", "true", "false", "while", "loop", "for", "return", "let", "nil", "and", "or", "struct", "this"
];
const PUNCTS: &'static [char] = &['(', ')', '{', '}', ',', '.', '-', '+', ';', '+', '-', '*', '/', '%', '=', '&', '|', '^', '<', '>', '!'];

pub struct TokenStream {
    input: InputStream,
    current: Token,
    has_started: bool,
    has_error: bool,
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Clone, PartialEq)]
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
    Identifier,
    String,
    Integer,
    Numeral,
    // Keywords
    Keyword,
    // EOF token
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_token = match self {
            // Single character tokens
            TokenType::LeftParen
            | TokenType::RightParen
            | TokenType::LeftBrace
            | TokenType::RightBrace
            | TokenType::Comma
            | TokenType::Dot
            | TokenType::Minus
            | TokenType::Plus
            | TokenType::Semicolon
            | TokenType::Slash
            | TokenType::Star => "Single punctuation",
            // One or two character tokens
            TokenType::BangEqual
            | TokenType::Bang
            | TokenType::Equal
            | TokenType::EqualEqual
            | TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => "Punctuation",
            // Literals
            TokenType::Integer => "Integer",
            TokenType::Numeral => "Numeral",
            TokenType::String => "String",
            TokenType::Identifier => "Identifier",
            // Keywords
            TokenType::Keyword => "Keyword",
            // EOF token
            TokenType::Eof => "EOF",
        };
        write!(f, "{}", string_token)
    }
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Eof
    }
}

impl Token {
    pub fn new(token_type: TokenType, value: &String, line: usize, col: usize) -> Self {
        Token {
            token_type,
            value: value.to_owned(),
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
            return Ok(Token::new(TokenType::Eof, &String::default(), self.input.line, self.input.col));
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
            return Some(Token::new(TokenType::String, &return_string.unwrap(), self.input.line, self.input.col));
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

        // If it is an integer, return an integer token
        if !has_dec_point {
            return Token::new(TokenType::Integer, &number, self.input.line, self.input.col)
        }

        // Otherwise return a float token
        Token::new(TokenType::Numeral, &number, self.input.line, self.input.col)
    }

    fn read_ident(&mut self) -> Token {
        let identifier = self.read_while(&mut is_id);

        Token::new(
            if is_keyword(&identifier) {
                TokenType::Keyword
            } else {
                TokenType::Identifier
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
            _ => TokenType::Eof,
        };

        if single_token_type != TokenType::Eof {
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
        !self.peek().is_none() && self.peek().unwrap().token_type == TokenType::Eof
    }
}

fn is_keyword(word: &String) -> bool {
    KEYWORDS.iter().any(|&i| i == word)
}

fn is_id_start(ch: char) -> bool {
    Regex::new(r"[a-zA-Zλ_]")
        .unwrap()
        .is_match(ch.to_string().as_str())
}

fn is_punctuation(ch: char) -> bool {
    PUNCTS.iter().any(|&i| i == ch)
}

fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

fn is_id(ch: char) -> bool {
    is_id_start(ch) || "-!?0123456789".contains(ch)
}
