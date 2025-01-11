use crate::lexer::*;

pub trait Visitor<T> {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_grouping_expr(&mut self, expression: &Expr) -> T;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_literal_expr(&mut self, value: &LiteralValue) -> T;
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
    Nil,
}

impl Expr {
    /// Accept a visitor for traversing this expression
    pub fn accept<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
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

pub struct AstPrinter;

impl Visitor<String> for AstPrinter {
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
            LiteralValue::Nil => "nil".to_string(),
        }
    }
}

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