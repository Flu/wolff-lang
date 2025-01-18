use std::collections::HashMap;
use crate::errors::InterpreterRuntimeError;
use crate::lexer::{Token, TokenType};
use crate::ast::{Expr, ExprVisitor, LiteralValue, Stmt, StmtVisitor};

pub struct AstInterpreter {
    environment: Environment
}

pub struct Environment {
    values: Vec<HashMap<String, LiteralValue>>,
}

impl Environment {

    pub fn new() -> Self {
        let mut global_scope = Vec::new();
        global_scope.push(HashMap::new());

        Environment {
            values: global_scope,
        }
    }

    pub fn create_new_scope(&mut self) {
        self.values.push(HashMap::new());
    }

    pub fn delete_most_recent_scope(&mut self) {
        self.values.remove(self.values.len()-1);
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.last_mut().unwrap().insert(name, value);
    }

    pub fn get(&self, name: String) -> Option<&LiteralValue> {
        for scope in self.values.iter().rev() {
            let maybe_lit = scope.get(&name);
            if maybe_lit.is_some() {
                return maybe_lit;
            }
        }
        return None;
    }

    pub fn assign(&mut self, name: String, value: LiteralValue) -> Result<LiteralValue, InterpreterRuntimeError> {
        for scope in self.values.iter_mut().rev() {
            if scope.contains_key(&name) {
                *scope.get_mut(&name).unwrap() = value.clone();
                return Ok(value);
            }
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

    fn visit_logical_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<LiteralValue, InterpreterRuntimeError> {
        let left_value = self.evaluate(left)?;

        if operator.token_type == TokenType::Keyword("and".to_string()) {
            if left_value == LiteralValue::Bool(false) {
                return Ok(LiteralValue::Bool(false));
            }
            return Ok(self.evaluate(right)?);
        }

        if operator.token_type == TokenType::Keyword("or".to_string()) {
            if left_value == LiteralValue::Bool(true) {
                return Ok(LiteralValue::Bool(true));
            }
            return Ok(self.evaluate(right)?);
        }

        Err(InterpreterRuntimeError {
            message: format!("Illegal use of logical {} between operands", operator.lexeme),
            line: operator.line,
            col: operator.col
        })
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

    fn visit_if_stmt(&mut self, if_stmt: &Stmt) -> Result<(), InterpreterRuntimeError> {
        match if_stmt {
            Stmt::If { condition, then_branch, else_branch } => {
                let condition_result = self.evaluate(condition)?;
                if condition_result == LiteralValue::Bool(true) {

                    self.execute(&then_branch)?;
                    return Ok(());
                } else if condition_result == LiteralValue::Bool(false) && else_branch.is_some() {

                    self.execute(&else_branch.as_ref().unwrap())?;
                    return Ok(());
                }

                // TODO: do something about the fact that expressions don't have any location information
                // We need to show the error location to the user, but right now, there isn't much I can do about it
                return Err(InterpreterRuntimeError {
                    message: "If condition must evaluate to a boolean value".to_string(),
                    line: 0,
                    col: 0
                });
            },
            _ => panic!("Trying to execute an if statement that is not an if statement")
        };
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<(), InterpreterRuntimeError> {
        let condition_result = self.evaluate(condition)?;

        // FIXME: this is crusty as fuck, please change it to something more robust when/if I add types
        if condition_result != LiteralValue::Bool(true) && condition_result != LiteralValue::Bool(false) {
            return Err(InterpreterRuntimeError {
                message: "While condition must evaluate to a boolean value".to_string(),
                line: 0,
                col: 0
            });
        }

        while self.evaluate(condition)? == LiteralValue::Bool(true) {
            self.execute(body)?;
        }

        Ok(())
    }

    fn visit_block_stmt(&mut self, block: &Vec<Stmt>) -> Result<(), InterpreterRuntimeError> {
        self.execute_block(block)?;
    
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

    pub fn execute_block(&mut self, statements: &Vec<Stmt>) -> Result<(), InterpreterRuntimeError> {
        self.environment.create_new_scope();

        for statement in statements.iter() {
            self.execute(statement)?;
        }

        // Restore the previous environment
        self.environment.delete_most_recent_scope();
        Ok(())
    }
}