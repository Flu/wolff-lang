use crate::input_stream::InputStream;
use std::fmt;
use regex::Regex;

const KEYWORDS: &'static [&'static str] = &["if", "then", "else", "lambda", "λ", "true", "false"];
const OP_CHARS: &'static [char] = &['+', '-', '*', '/', '%', '=', '&', '|', '^', '<', '>', '!'];
const PUNCTS: &'static [char] = &[',', ';', '(', ')', '{', '}', '[', ']'];

pub struct TokenStream {
    input: InputStream,
    current: Option<Token>
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String
}

#[derive(Clone)]
pub enum TokenType {
    Punctuation,
    Numeral,
    String,
    Keyword,
    Variable,
    Operation
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_token = match self {
            TokenType::Keyword => "Keyword",
            TokenType::Punctuation => "Punctuation",
            TokenType::Numeral => "Numeral",
            TokenType::String => "String",
            TokenType::Variable => "Variable",
            TokenType::Operation => "Operation",
        };
        write!(f, "{}", string_token)
    }
}

impl Token {
    pub fn new(token_type: TokenType, value: &String) -> Self {
        Token {
            token_type: token_type,
            value: value.to_owned()
        }
    }
}

impl TokenStream {
    pub fn new(input: &mut InputStream) -> Self {
        TokenStream {
            input: input.clone(),
            current: None
        }
    }

    fn read_next(&mut self) -> Option<Token> {
        // If the input char is whitespace, continue reading until it isn't
        self.read_while(&mut is_whitespace);

        // If input is EOF, return None
        if self.input.eof() {
            return None
        }

        // Peek at the next character in the input stream to figure out what we need to do
        let ch = self.input.peek();

        // The next line is a comment, so ignore it and try again after newline
        if ch == '#' {
            self.skip_comment();
            return self.read_next();
        }
        if ch == '"' {
            return Option::Some(self.read_string());
        }
        if ch.is_digit(10) {
            return Option::Some(self.read_number());
        }
        if is_id_start(ch) {
            return Option::Some(self.read_ident());
        }
        if is_punctuation(ch) {
            return Option::Some(Token::new(TokenType::Punctuation, &self.input.next().to_string()));
        }
        if is_op_char(ch) {
            return Option::Some(Token::new(TokenType::Operation, &self.read_while(&mut is_op_char)));
        }

        //panic!(format!("Unknown character at {}:{}. Aborting", self.input.line, self.input.col));
        None
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
        self.input.next();
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
        
        Token::new(if is_keyword(&identifier) { TokenType::Keyword } else { TokenType::Variable}, &identifier)
    }

    pub fn peek(&mut self) -> Option<Token> {
        if self.current.is_none() {
            self.current = self.read_next();
        }
        self.current.clone()
    }

    pub fn next(&mut self) -> Option<Token> {
        let tok = self.current.clone();
        self.current = None;
        if tok.is_none() {
            self.current = self.read_next();
            self.current.clone()
        } else {
            tok
        }
    }

    pub fn eof(&mut self) -> bool {
        self.peek().is_none()
    }
}

fn is_keyword(word: &String) -> bool {
    KEYWORDS.iter().any(|&i| i==word)
}

fn is_op_char(ch: char) -> bool {
    OP_CHARS.iter().any(|&i| i==ch)
}

fn is_id_start(ch: char) -> bool {
    Regex::new(r"[a-zA-Zλ_]").unwrap().is_match(ch.to_string().as_str())
}

fn is_punctuation(ch: char) -> bool {
    PUNCTS.iter().any(|&i| i==ch)
}

fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace()
}

fn is_id(ch: char) -> bool {
    is_id_start(ch) || "-!?*0123456789".contains(ch)
}

