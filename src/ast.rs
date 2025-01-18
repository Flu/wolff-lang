use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block {
        statements: Vec<Stmt>
    },
    Expression {
        expression: Expr
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>
    },
    Print {
        expression: Expr
    },
    Var {
        name: Token,
        initializer: Expr
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Variable {
        name: Token
    }
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
            Expr::Assign { name, value } => {
                visitor.visit_assign_expr(name, value)
            }
            Expr::Binary { left, operator, right } => {
                visitor.visit_binary_expr(left, operator, right)
            }
            Expr::Logical { left, operator, right } => {
                visitor.visit_logical_expr(left, operator, right)
            }
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Variable { name } => visitor.visit_variable_expr(name)
        }
    }
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        match self {
            Stmt::If {..} => visitor.visit_if_stmt(self),
            Stmt::Block { statements } => visitor.visit_block_stmt(statements),
            Stmt::Expression { expression } => visitor.visit_stmt_stmt(expression),
            Stmt::Print { expression } => visitor.visit_print_stmt(expression),
            Stmt::Var { name, initializer } => visitor.visit_var_stmt(name, initializer)
        }
    }
}

pub trait ExprVisitor<T> {
    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> T;
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_grouping_expr(&mut self, expression: &Expr) -> T;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_logical_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_literal_expr(&mut self, value: &LiteralValue) -> T;
    fn visit_variable_expr(&mut self, name: &Token) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_if_stmt(&mut self, if_stmt: &Stmt) -> T;
    fn visit_block_stmt(&mut self, block: &Vec<Stmt>) -> T;
    fn visit_stmt_stmt(&mut self, expr: &Expr) -> T;
    fn visit_print_stmt(&mut self, expr: &Expr) -> T;
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> T;
}

pub struct AstPrinter;

impl ExprVisitor<String> for AstPrinter {
    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> String {
        format!("({} {} {})", "assign", name.lexeme, value.accept(self))
    }

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

    fn visit_logical_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> String {
        let left_str = left.accept(self);
        let right_str = right.accept(self);
        if operator.token_type == TokenType::Keyword("and".to_string()) {
            return format!("(and {} {})", left_str, right_str);
        } else if operator.token_type == TokenType::Keyword("or".to_string()) {
            return format!("(or {} {})", left_str, right_str);
        }
        panic!("Trying to print a logical expression that doesn't use 'and' or 'or'");
    }

    fn visit_literal_expr(&mut self, value: &LiteralValue) -> String {
        match value {
            LiteralValue::Number(num) => num.to_string(),
            LiteralValue::Text(text) => format!("\"{}\"", text),
            LiteralValue::Bool(boolean) => boolean.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_variable_expr(&mut self, name: &Token) -> String {
        name.lexeme.clone()
    }
}

impl StmtVisitor<String> for AstPrinter {
    fn visit_if_stmt(&mut self, if_stmt: &Stmt) -> String {
        return match if_stmt {
            Stmt::If { condition, then_branch, else_branch: None } => {
                format!("(if {:?} {:?})", condition.accept(self), then_branch.accept(self))
            },
            Stmt::If { condition, then_branch, else_branch } => {
                format!("(if {:?} {:?} {:?})", condition.accept(self), then_branch.accept(self), else_branch.as_ref().unwrap().accept(self))
            }
            _ => panic!("Tried to print an if statement that is not an if statement")
        };
    }

    fn visit_block_stmt(&mut self, block: &Vec<Stmt>) -> String {
        format!("(block [{:?}])", block)
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> String {
        let expr_str = expr.accept(self);
        format!("(print_stmt {expr_str})")
    }

    fn visit_stmt_stmt(&mut self, expr: &Expr) -> String {
        let expr_str = expr.accept(self);
        format!("(expr_stmt {expr_str})")
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> String {
        let expr_str = initializer.accept(self);
        let variable_name = &name.lexeme;
        format!("(declare {variable_name} {expr_str})")
    }
}