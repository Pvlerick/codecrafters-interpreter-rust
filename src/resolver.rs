use std::collections::HashMap;

use crate::parser::{Expr, Function, Statement};

pub struct Resolver {
    scopes: Vec<HashMap<String, Variable>>,
    pub resolve_table: Vec<usize>,
    next_index: usize,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            resolve_table: Vec::new(),
            next_index: 0,
        }
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
                if self.scopes.last().map_or(false, |i| {
                    i.get(&token.lexeme).map(|i| i.is_defined).unwrap_or(false)
                }) {
                    // Report error: variable used in its own initializer
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

    fn resolve_local<T: ToString>(&mut self, _expr: &Expr, name: &T) {
        for scope in self.scopes.iter().enumerate().rev() {
            if let Some(variable) = scope.1.get(&name.to_string()) {
                self.resolve_table[variable.index] = scope.0;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn inc_index(&mut self) -> usize {
        self.next_index += 1;
        self.next_index
    }

    fn declare<T: ToString>(&mut self, name: &T) {
        let index = self.inc_index();
        self.scopes
            .last_mut()
            .and_then(|i| i.insert(name.to_string(), Variable::new(index)));
    }

    fn define<T: ToString>(&mut self, name: &T) {
        self.scopes.last_mut().and_then(|i| {
            i.entry(name.to_string()).and_modify(|i| i.define());
            None::<T>
        });
    }
}

struct Variable {
    is_defined: bool,
    index: usize,
}

impl Variable {
    fn new(index: usize) -> Self {
        Self {
            is_defined: false,
            index,
        }
    }

    fn define(&mut self) {
        self.is_defined = true;
    }
}
