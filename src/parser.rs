use core::panic;

use crate::lexer::*;
use crate::errors::{ParserError, InterpreterRuntimeError};

pub trait ExprVisitor<T> {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_grouping_expr(&mut self, expression: &Expr) -> T;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_literal_expr(&mut self, value: &LiteralValue) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_stmt_expr(&mut self, expr: &Expr) -> T;
    fn visit_print_expr(&mut self, expr: &Expr) -> T;
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expression: Expr
    },
    Print {
        expression: Expr
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Number(f64),
    Text(String),
    Bool(bool),
    Nil,
}

impl Expr {
    /// Accept a visitor for traversing this expression
    pub fn accept<T>(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        match self {
            Expr::Binary { left, operator, right } => {
                visitor.visit_binary_expr(left, operator, right)
            }
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
        }
    }
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        match self {
            Stmt::Expression { expression } => visitor.visit_stmt_expr(expression),
            Stmt::Print { expression } => visitor.visit_print_expr(expression)
        }
    }
}

pub struct AstPrinter;

impl ExprVisitor<String> for AstPrinter {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        let left_str = left.accept(self);
        let right_str = right.accept(self);
        format!("({} {} {})", operator.token_type, left_str, right_str)
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> String {
        let expr_str = expression.accept(self);
        format!("(group {})", expr_str)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> String {
        let right_str = right.accept(self);
        format!("({} {})", operator.lexeme, right_str)
    }

    fn visit_literal_expr(&mut self, value: &LiteralValue) -> String {
        match value {
            LiteralValue::Number(num) => num.to_string(),
            LiteralValue::Text(text) => format!("\"{}\"", text),
            LiteralValue::Bool(boolean) => boolean.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }
}

impl StmtVisitor<String> for AstPrinter {
    fn visit_print_expr(&mut self, expr: &Expr) -> String {
        let expr_str = expr.accept(self);
        format!("(print_stmt {expr_str})")
    }

    fn visit_stmt_expr(&mut self, expr: &Expr) -> String {
        let expr_str = expr.accept(self);
        format!("(expr_stmt {expr_str})")
    }
}

pub struct AstInterpreter;

impl ExprVisitor<Result<LiteralValue, InterpreterRuntimeError>> for AstInterpreter {
    fn visit_literal_expr(&mut self, value: &LiteralValue) -> Result<LiteralValue, InterpreterRuntimeError> {
        Ok(value.clone())
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        self.evaluate(expression)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        let right_value = self.evaluate(right)?;

        match (operator, right_value) {
            (Token { token_type: TokenType::Bang, lexeme: _, line: _, col: _}, LiteralValue::Bool(boolean)) => {
                return Ok(LiteralValue::Bool(!boolean));
            }
            (Token { token_type: TokenType::Bang, lexeme: _, line: _, col: _}, LiteralValue::Nil) => {
                return Ok(LiteralValue::Bool(true));
            }
            (Token { token_type: TokenType::Bang, lexeme: _, line: _, col: _}, _) => {
                return Ok(LiteralValue::Bool(false));
            }
            (Token { token_type: TokenType::Minus, lexeme: _, line: _, col: _}, LiteralValue::Number(number)) => {
                return Ok(LiteralValue::Number(-number));
            }
            _ => return Err(InterpreterRuntimeError {
                message: format!("Illegal use of {} for operand", operator.lexeme),
                line: operator.line,
                col: operator.col
            })
        }
    }

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        match (operator, left_value, right_value) {
            // MINUS OPERATOR
            (Token { token_type: TokenType::Minus, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Number(rhs - lhs));
            },
            // SLASH OPERATOR
            (Token { token_type: TokenType::Slash, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Number(rhs / lhs));
            },
            // PLUS OPERATOR
            (Token { token_type: TokenType::Plus, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Number(rhs + lhs));
            },
            // STAR OPERATOR
            (Token { token_type: TokenType::Star, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Number(rhs * lhs));
            },
            (Token { token_type: TokenType::Star, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Text(lhs)) => {
                // TODO: this is a truncating cast. When implementing integers, be careful for such uses
                return Ok(LiteralValue::Text(lhs.repeat(rhs as usize)));
            },
            // PLUS OPERATOR FOR STRINGS
            (Token { token_type: TokenType::Plus, lexeme: _, line: _, col: _}, LiteralValue::Text(rhs), LiteralValue::Text(lhs)) => {
                return Ok(LiteralValue::Text(format!("{}{}", rhs, lhs)));
            },
            // GREATER THAN OPERATOR
            (Token { token_type: TokenType::Greater, lexeme: _, line: _, col: _}, LiteralValue::Text(rhs), LiteralValue::Text(lhs)) => {
                return Ok(LiteralValue::Bool(rhs > lhs));
            },
            (Token { token_type: TokenType::Greater, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Bool(rhs > lhs));
            },
            // GREATER OR EQUAL THAN OPERATOR
            (Token { token_type: TokenType::GreaterEqual, lexeme: _, line: _, col: _}, LiteralValue::Text(rhs), LiteralValue::Text(lhs)) => {
                return Ok(LiteralValue::Bool(rhs >= lhs));
            },
            (Token { token_type: TokenType::GreaterEqual, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Bool(rhs >= lhs));
            },
            // LESS THAN OPERATOR
            (Token { token_type: TokenType::Less, lexeme: _, line: _, col: _}, LiteralValue::Text(rhs), LiteralValue::Text(lhs)) => {
                return Ok(LiteralValue::Bool(rhs < lhs));
            },
            (Token { token_type: TokenType::Less, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Bool(rhs < lhs));
            },
            // LESS OR EQUAL THAN OPERATOR
            (Token { token_type: TokenType::LessEqual, lexeme: _, line: _, col: _}, LiteralValue::Text(rhs), LiteralValue::Text(lhs)) => {
                return Ok(LiteralValue::Bool(rhs <= lhs));
            },
            (Token { token_type: TokenType::LessEqual, lexeme: _, line: _, col: _}, LiteralValue::Number(rhs), LiteralValue::Number(lhs)) => {
                return Ok(LiteralValue::Bool(rhs <= lhs));
            },
            // EQUALITY OPERATOR
            (Token { token_type: TokenType::EqualEqual, lexeme: _, line: _, col: _}, rhs, lhs) => {
                return Ok(LiteralValue::Bool(rhs == lhs));
            },
            // INEQUALITY OPERATOR
            (Token { token_type: TokenType::BangEqual, lexeme: _, line: _, col: _}, rhs, lhs) => {
                return Ok(LiteralValue::Bool(rhs != lhs));
            },
            // If we're here, it means there's an illegal use of an operator, so return an error specifying that
            _ => Err(InterpreterRuntimeError {
                message: format!("Illegal use of {} between operands", operator.lexeme),
                line: operator.line,
                col: operator.col
            })
        }
    }
}

impl StmtVisitor<Result<(), InterpreterRuntimeError>> for AstInterpreter {
    fn visit_print_expr(&mut self, expr: &Expr) -> Result<(), InterpreterRuntimeError> {
        let lit_value = self.evaluate(expr)?;

        match lit_value {
            LiteralValue::Number(number) => println!("{number}"),
            LiteralValue::Text(string) => println!("{string}"),
            LiteralValue::Bool(boolean) => println!("{boolean}"),
            LiteralValue::Nil => println!("nil"),
        }
        Ok(())
    }

    fn visit_stmt_expr(&mut self, expr: &Expr) -> Result<(), InterpreterRuntimeError> {
        self.evaluate(expr)?;
        Ok(())
    }
}

impl AstInterpreter {
    fn evaluate(&mut self, expression: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        return expression.accept(self);
    }
}

pub struct Parser<'a> {
    current: usize,
    tokens: &'a Vec<Token>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    pub fn new(token_vector: &'a Vec<Token>) -> Self {
        Parser {
            current: 0,
            tokens: token_vector,
            had_error: false,
            panic_mode: false
        }
    }

    pub fn parse(&mut self) -> Vec<Result<Stmt, ParserError>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(Ok(stmt)),
                Err(e) => {
                     statements.push(Err(e));
                    self.synchronize();
                }
            }
        }

        statements
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_tokens_with_value(&[TokenType::Keyword("print".to_string())]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ; after statement.")?;

        return Ok(Stmt::Print {
            expression: expr
        });
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ; after expression.")?;

        return Ok(Stmt::Expression {
            expression: expr
        })
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match &self.peek().token_type {
                TokenType::Keyword(keyword) if [
                    "class", "else", "fun", "for", "if", "lambda", "print", "return", "super", "this", "var", "while", "λ"
                ].contains(&keyword.as_str()) => {
                    return;
                },
                _ => {}
            }

            self.advance();
        }
    }

    pub fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    pub fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParserError> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }
        
        let token = self.peek();
        self.error_at(&token, message);
        Err(ParserError {
            message: message.to_string(),
            line: token.line,
            col: token.col
        })
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut expr: Expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right)
            };
        }

        return Ok(expr);
    }

    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut expr: Expr = self.term()?;

        while self.match_tokens(&[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator: Token = self.previous();
            let right: Expr = self.term()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }

        return Ok(expr);
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut expr: Expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous();
            let right: Expr = self.factor()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }

        return Ok(expr);
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut expr: Expr = self.unary()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;
            expr = Expr::Binary { left: Box::new(expr), operator, right: Box::new(right) };
        }

        return Ok(expr);
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right)
            });
        }
        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.match_tokens_with_value(&[TokenType::Keyword("true".to_string())]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Bool(true)
            });
        }

        if self.match_tokens_with_value(&[TokenType::Keyword("false".to_string())]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Bool(false)
            });
        }

        if self.match_tokens_with_value(&[TokenType::Keyword("nil".to_string())]) {
            return Ok(Expr::Literal {
                value: LiteralValue::Nil
            });
        }

        if self.match_tokens(&[TokenType::Number(0.0)]) {
            match self.previous() {
                Token { token_type: TokenType::Number(number), lexeme: _, line: _, col: _ } => 
                    return Ok(Expr::Literal {
                        value: LiteralValue::Number(number)
                    }),
                _ => panic!("Something went terribly wrong in parsing. Expecting a number.")
            };
        }

        if self.match_tokens(&[TokenType::String("".to_string())]) {
            match self.previous() {
                Token { token_type: TokenType::String(string), lexeme: _, line: _, col: _ } =>
                    return Ok(Expr::Literal {
                        value: LiteralValue::Text(string)
                    }),
                _ => panic!("Something went terribly wrong in parsing. Expectin a string literal.")
            };
        }

        if self.match_tokens(&[TokenType::LeftParen]) {
            let expr: Expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr)
            });
        }

        let peeked_token = self.peek();
        return Err(ParserError {
            message: "Expected expression".to_string(),
            line: peeked_token.line,
            col: peeked_token.col
        })
    }

    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(&token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn match_tokens_with_value(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check_with_value(&token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type);
    }

    fn check_with_value(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return &self.peek().token_type == token_type;
    }

    fn is_at_end(&self) -> bool {
        return self.peek().token_type == TokenType::EOF;
    }

    fn peek(&self) -> Token {
        return self.tokens[self.current].clone();
    }

    fn previous(&self) -> Token {
        return self.tokens[self.current-1].clone();
    }

    pub fn error_at(&mut self, _: &Token, _: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        // TODO: No printing in the parser itself, the parser should return a list of errors instead
        // println!("Error at {}:{}: {}", token.line, token.col, message);
        self.had_error = true;
    }
}