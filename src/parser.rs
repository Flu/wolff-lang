use core::panic;

use crate::ast::{Expr, Stmt, LiteralValue};
use crate::lexer::*;
use crate::errors::ParserError;

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
            match self.declaration() {
                Ok(stmt) => statements.push(Ok(stmt)),
                Err(e) => {
                    statements.push(Err(e));
                    self.synchronize();
                }
            }
        }

        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        if self.match_tokens_with_value(&[TokenType::Keyword("var".to_string())]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = self.consume(TokenType::Identifier("".to_string()), "Expected variable name")?;

        let initializer_expression;
        if self.match_tokens(&[TokenType::Equal]) {
            initializer_expression = self.expression()?;
        } else {
            return Err(ParserError {
                message: "Variable can't be declared but not initialized".to_string(),
                line: name.line,
                col: name.col
            })
        }

        self.consume(TokenType::Semicolon, "Expected semicolon after declaration")?;
        return Ok(Stmt::Var {
            name,
            initializer: initializer_expression
        });
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_tokens_with_value(&[TokenType::Keyword("if".to_string())]) {
            return self.if_statement();
        }
        if self.match_tokens_with_value(&[TokenType::Keyword("print".to_string())]) {
            return self.print_statement();
        }

        if self.match_tokens_with_value(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block { statements: self.block_statement()? });
        }

        self.expression_statement()
    }

    fn if_statement(&mut self,) -> Result<Stmt, ParserError> {
        let condition: Expr = self.expression()?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self.match_tokens_with_value(&[TokenType::Keyword("else".to_string())]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        return Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch })
    }

    fn block_statement(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let declaration = self.declaration()?;
            statements.push(declaration);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after a block")?;
        return Ok(statements);
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
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr: Expr = self.or()?;

        if self.match_tokens(&[TokenType::Equal]) {
            let equals: Token = self.previous();
            let value: Expr = self.assignment()?;

            return match expr {
                Expr::Variable { ref name } => Ok(Expr::Assign { name: name.clone(), value: Box::new(value) }),
                _ => Err(ParserError {
                    message: "Invalid l-value for assignment".to_string(),
                    line: equals.line,
                    col: equals.col
                })
            };
        }
        return Ok(expr);
    }

    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut expr: Expr = self.and()?;

        while self.match_tokens(&[TokenType::Keyword("or".to_string())]) {
            let operator: Token = self.previous();
            let right: Expr = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right)
            };
        }
        return Ok(expr);
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut expr: Expr = self.equality()?;

        while self.match_tokens(&[TokenType::Keyword("and".to_string())]) {
            let operator: Token = self.previous();
            let right: Expr = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right)
            };
        }
        return Ok(expr);
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
        if self.match_tokens(&[TokenType::Identifier("".to_string())]) {
            let token = self.previous();
            match token.token_type {
                TokenType::Identifier(_) => return Ok(Expr::Variable {
                    name: token
                }),
                _ => panic!("Something went terribly wrong in parsing. Expecting a variable name.")
            }
        }
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