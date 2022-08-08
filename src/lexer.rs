use crate::errors::InvalidTokenError;
use crate::input_stream::InputStream;

use regex::Regex;
use std::fmt;

const KEYWORDS: &'static [&'static str] = &["if", "then", "else", "lambda", "λ", "true", "false"];
const OP_CHARS: &'static [char] = &['+', '-', '*', '/', '%', '=', '&', '|', '^', '<', '>', '!'];
const PUNCTS: &'static [char] = &[',', '.', ';', '(', ')', '{', '}', '[', ']'];

pub struct TokenStream {
    input: InputStream,
    current: Token,
    has_started: bool,
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

#[derive(Clone, PartialEq)]
pub enum TokenType {
    Punctuation,
    Numeral,
    String,
    Keyword,
    Identifier,
    Operation,
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_token = match self {
            TokenType::Keyword => "Keyword",
            TokenType::Punctuation => "Punctuation",
            TokenType::Numeral => "Numeral",
            TokenType::String => "String",
            TokenType::Identifier => "Identifier",
            TokenType::Operation => "Operation",
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
    pub fn new(token_type: TokenType, value: &String) -> Self {
        Token {
            token_type: token_type,
            value: value.to_owned(),
        }
    }
}

impl TokenStream {
    pub fn new(input: &mut InputStream) -> Self {
        TokenStream {
            input: input.clone(),
            current: Token::new(TokenType::default(), &String::default()),
            has_started: false,
        }
    }

    fn read_next(&mut self) -> Result<Token, InvalidTokenError> {
        // If the input char is whitespace, continue reading until it isn't
        self.read_while(&mut is_whitespace);

        // If input is EOF, return EOF token
        if self.input.eof() {
            return Ok(Token::new(TokenType::Eof, &String::default()));
        }

        // Peek at the next character in the input stream to figure out what we need to do
        let ch = self.input.peek();

        // The next line is a comment, so ignore it and try again after newline
        if ch == '#' {
            self.skip_comment();
            return self.read_next();
        }

        if ch == '"' {
            return Ok(self.read_string());
        }

        if ch.is_digit(10) {
            return Ok(self.read_number());
        }

        if is_id_start(ch) {
            return Ok(self.read_ident());
        }

        if is_punctuation(ch) {
            return Ok(Token::new(
                TokenType::Punctuation,
                &self.input.next().to_string(),
            ));
        }

        if is_op_char(ch) {
            return Ok(Token::new(
                TokenType::Operation,
                &self.read_while(&mut is_op_char),
            ));
        }

        //Illegal character detected here, skip this one and return an error
        
        let error = Err(InvalidTokenError {
            message: format!(
                "Invalid character at {}:{}",
                self.input.line, self.input.col
            ),
            line_as_string: self.input.get_current_line().to_string(),
            line: self.input.line,
            col: self.input.col
        });
        self.input.next();
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

    fn read_string(&mut self) -> Token {
        Token::new(TokenType::String, &self.read_escaped('"'))
    }

    fn read_escaped(&mut self, end: char) -> String {
        let mut escaped = false;
        let mut return_string = String::new();

        self.input.next();
        while !self.input.eof() {
            let ch = self.input.next();
            if escaped {
                return_string.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == end {
                break;
            } else {
                return_string.push(ch);
            }
        }
        return_string
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

        Token::new(TokenType::Numeral, &number)
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
        )
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

fn is_op_char(ch: char) -> bool {
    OP_CHARS.iter().any(|&i| i == ch)
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
    is_id_start(ch) || "-!?*0123456789".contains(ch)
}
