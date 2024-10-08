use std::collections::HashMap;

use crate::parser::{Expr, Function, Statement};

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn resolve(&mut self, statements: &Vec<Statement>) {
        for statement in statements {
            self.resolve_statement(&statement);
        }
    }

    fn resolve_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Block(statements) => {
                self.begin_scope();
                self.resolve(&statements);
                self.end_scope();
            }
            Statement::Variable(name, initializer) => {
                self.declare(&name);
                if let Some(initializer) = initializer {
                    self.resolve_expression(&initializer);
                }
                self.define(&name);
            }
            Statement::Expression(expr) => {
                self.resolve_expression(expr);
            }
            Statement::If(condition, then_branch, else_branch) => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch);
                }
            }
            Statement::Print(expr) => self.resolve_expression(expr),
            Statement::Return(None) => {}
            Statement::Return(Some(expr)) => self.resolve_expression(expr),
            Statement::While(condition, body) => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
        }
    }

    fn resolve_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(ref token) => {
                if self
                    .scopes
                    .last()
                    .map_or(false, |i| *i.get(&token.lexeme).unwrap_or(&false))
                {
                    todo!()
                }

                self.resolve_local(&expr, &token.lexeme);
            }
            Expr::Assignment(token, expr) => {
                self.resolve_expression(expr);
                self.resolve_local(expr, &token.lexeme);
            }
            Expr::Function(name, fun) => {
                if let Some(name) = name {
                    self.declare(name);
                    self.define(name);
                }
                self.resolve_function(fun);
            }
            Expr::Binary(_, left, right) => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Call(callee, _, arguments) => {
                self.resolve_expression(callee);
                for arg in arguments.iter() {
                    self.resolve_expression(arg);
                }
            }
            Expr::Grouping(expr) => self.resolve_expression(expr),
            Expr::Literal(_) => {}
            Expr::Logical(_, left, right) => {
                self.resolve_expression(left);
                self.resolve_expression(right);
            }
            Expr::Unary(_, expr) => self.resolve_expression(expr),
        }
    }

    fn resolve_function(&mut self, function: &Function) {
        self.begin_scope();
        for param in function.parameters.iter() {
            self.declare(&param.lexeme);
            self.define(&param.lexeme);
        }
        self.resolve_statement(&function.body);
        self.end_scope();
    }

    fn resolve_local<T: ToString>(&self, _expr: &Expr, name: &T) {
        let mut _count = 0;
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name.to_string()) {
                todo!();
                // self.interpreter.resolve(expr, count);
                // return;
            }
            _count += 1;
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare<T: ToString>(&mut self, name: &T) {
        self.scopes
            .last_mut()
            .and_then(|i| i.insert(name.to_string(), false));
    }

    fn define<T: ToString>(&mut self, name: &T) {
        self.scopes
            .last_mut()
            .and_then(|i| i.insert(name.to_string(), true));
    }
}
