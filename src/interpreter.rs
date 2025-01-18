use std::collections::HashMap;
use crate::errors::InterpreterRuntimeError;
use crate::lexer::{Token, TokenType};
use crate::ast::{Expr, ExprVisitor, LiteralValue, Stmt, StmtVisitor};

pub struct AstInterpreter {
    environment: Environment
}

pub struct Environment {
    values: HashMap<String, LiteralValue>,
    enclosing: Box<Option<Environment>>
}

impl Environment {

    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Box::new(None),
        }
    }

    pub fn from_enclosing(enclosing: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Box::new(Some(enclosing))
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<&LiteralValue> {
        let maybe_value = self.values.get(&name);

        if maybe_value.is_none() && self.enclosing.is_some() {
            if let Some(some_environment) = self.enclosing.as_ref() {
                return some_environment.get(name);
            }
        }

        return maybe_value;
    }

    pub fn assign(&mut self, name: String, value: LiteralValue) -> Result<LiteralValue, InterpreterRuntimeError> {
        if self.values.contains_key(&name) {
            *self.values.get_mut(&name).unwrap() = value.clone();
            return Ok(value);
        }

        if self.enclosing.is_some() {
            return self.assign(name, value);
        }

        return Err(InterpreterRuntimeError {
            message: "Variable is not defined".to_string(),
            line: 0,
            col: 0
        });
    }
}

impl ExprVisitor<Result<LiteralValue, InterpreterRuntimeError>> for AstInterpreter {

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        let rvalue = self.evaluate(value)?;
        self.environment.assign(name.lexeme.clone(), rvalue.clone())?;
        return Ok(rvalue);
    }

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

    fn visit_variable_expr(&mut self, name: &Token) -> Result<LiteralValue, InterpreterRuntimeError> {
        match self.environment.get(name.lexeme.clone()) {
            Some(variable) => Ok(variable.clone()),
            None => Err(InterpreterRuntimeError {
                message: format!("The variable {} is not defined.", name.lexeme),
                line: name.line,
                col: name.col
            })
        }
    }
}

impl StmtVisitor<Result<(), InterpreterRuntimeError>> for AstInterpreter {
    fn visit_block_stmt(&mut self, block: &Vec<crate::ast::Stmt>) -> Result<(), InterpreterRuntimeError> {
        // Create a new environment
        let new_environment = Environment::from_enclosing(std::mem::replace(&mut self.environment, Environment::new()));
    
        // Execute the block in the new environment
        self.execute_block(block, new_environment)?;
    
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<(), InterpreterRuntimeError> {
        let lit_value = self.evaluate(expr)?;

        match lit_value {
            LiteralValue::Number(number) => println!("{number}"),
            LiteralValue::Text(string) => println!("{string}"),
            LiteralValue::Bool(boolean) => println!("{boolean}"),
            LiteralValue::Nil => println!("nil"),
        }
        Ok(())
    }

    fn visit_stmt_stmt(&mut self, expr: &Expr) -> Result<(), InterpreterRuntimeError> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> Result<(), InterpreterRuntimeError> {
        let value = self.evaluate(initializer)?;
        self.environment.define(name.lexeme.clone(), value);
        Ok(())
    }
}

impl AstInterpreter {
    pub fn new() -> Self {
        let interpreter = AstInterpreter {
            environment: Environment::new()
        };

        interpreter
    }

    pub fn from_environment(env: Environment) -> Self {
        let interpreter = AstInterpreter {
            environment: env
        };

        interpreter
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        return expression.accept(self);
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), InterpreterRuntimeError> {
        for stmt in statements.iter() {
            self.execute(stmt)?;
        }

        Ok(())
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), InterpreterRuntimeError> {
        statement.accept(self)
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Environment) -> Result<(), InterpreterRuntimeError> {
        let previous = std::mem::replace(&mut self.environment, environment);

        for statement in statements.iter() {
            self.execute(statement)?;
        }

        // Restore the previous environment
        self.environment = previous;
        Ok(())
    }
}