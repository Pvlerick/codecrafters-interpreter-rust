use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
    rc::Rc,
};

use crate::{
    errors::{ErrorMessage, InterpreterError},
    parser::{Expr, Function, Statement},
    scanner::Token,
};

pub struct Resolver {
    scopes: Vec<HashMap<String, Variable>>,
    pub resolve_table: HashMap<HashableExpr, usize>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            resolve_table: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Statement>) -> Result<(), InterpreterError> {
        for statement in statements {
            self.resolve_statement(&statement)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Result<(), InterpreterError> {
        match statement {
            Statement::Block(statements) => {
                self.begin_scope();
                self.resolve(&statements)?;
                self.end_scope();
                Ok(())
            }
            Statement::Variable(token, initializer) => {
                self.declare(&token)?;
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer.clone())?;
                }
                self.define(&token);
                Ok(())
            }
            Statement::Expression(expr) => self.resolve_expression(expr.clone()),
            Statement::If(condition, then_branch, else_branch) => {
                self.resolve_expression(condition.clone())?;
                self.resolve_statement(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch)?;
                }
                Ok(())
            }
            Statement::Print(expr) => self.resolve_expression(expr.clone()),
            Statement::Return(None) => Ok(()),
            Statement::Return(Some(expr)) => self.resolve_expression(expr.clone()),
            Statement::While(condition, body) => {
                self.resolve_expression(condition.clone())?;
                self.resolve_statement(body)
            }
        }
    }

    fn resolve_expression(&mut self, expr: Rc<Expr>) -> Result<(), InterpreterError> {
        match expr.deref() {
            Expr::Variable(ref token) => {
                if self.scopes.last().map_or(false, |i| {
                    i.get(&token.lexeme).map(|i| !i.is_defined).unwrap_or(false)
                }) {
                    return Err(InterpreterError::evaluating(
                        format!("Variable '{}' is used in its own initializer", token.lexeme),
                        token.line,
                    ));
                }

                self.resolve_local(expr.clone(), &token.lexeme);
                Ok(())
            }
            Expr::Assignment(token, expr) => {
                self.resolve_expression(expr.clone())?;
                self.resolve_local(expr.clone(), &token.lexeme);
                Ok(())
            }
            Expr::Function(token, fun) => {
                if let Some(token) = token {
                    self.declare(token)?;
                    self.define(token);
                }
                self.resolve_function(fun)
            }
            Expr::Binary(_, left, right) => {
                self.resolve_expression(left.clone())?;
                self.resolve_expression(right.clone())?;
                Ok(())
            }
            Expr::Call(callee, _, arguments) => {
                self.resolve_expression(callee.clone())?;
                for arg in arguments.iter() {
                    self.resolve_expression(arg.clone())?;
                }
                Ok(())
            }
            Expr::Grouping(expr) => self.resolve_expression(expr.clone()),
            Expr::Literal(_) => Ok(()),
            Expr::Logical(_, left, right) => {
                self.resolve_expression(left.clone())?;
                self.resolve_expression(right.clone())
            }
            Expr::Unary(_, expr) => self.resolve_expression(expr.clone()),
        }
    }

    fn resolve_function(&mut self, function: &Function) -> Result<(), InterpreterError> {
        self.begin_scope();
        for param in function.parameters.iter() {
            self.declare(&param)?;
            self.define(&param);
        }
        self.resolve_statement(&function.body)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_local<T: ToString>(&mut self, expr: Rc<Expr>, name: &T) {
        for scope in self.scopes.iter().rev().enumerate() {
            if let Some(_) = scope.1.get(&name.to_string()) {
                self.resolve_table.insert(HashableExpr(expr), scope.0);
                break;
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, token: &Token) -> Result<(), InterpreterError> {
        dbg!(&self.scopes);
        if self
            .scopes
            .last()
            .map_or(false, |i| i.contains_key(&token.lexeme))
        {
            return Err(InterpreterError::InterpreterError(ErrorMessage::new(
                format!(
                    "Already a variable with name '{}' in this scope",
                    token.lexeme
                ),
                Some(token.line),
            )));
        }
        self.scopes
            .last_mut()
            .and_then(|i| i.insert(token.lexeme.to_owned(), Variable::new()));
        Ok(())
    }

    fn define<T: ToString>(&mut self, name: &T) {
        self.scopes.last_mut().and_then(|i| {
            i.entry(name.to_string())
                .and_modify(|i| i.mark_as_defined());
            None::<T>
        });
    }
}

#[derive(Debug)]
struct Variable {
    is_defined: bool,
}

impl Variable {
    fn new() -> Self {
        Self { is_defined: false }
    }

    fn mark_as_defined(&mut self) {
        self.is_defined = true;
    }
}

#[derive(Debug)]
pub struct HashableExpr(Rc<Expr>);

impl From<Rc<Expr>> for HashableExpr {
    fn from(value: Rc<Expr>) -> Self {
        HashableExpr(value.clone())
    }
}

impl Hash for HashableExpr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(&self.0), state);
    }
}

impl PartialEq for HashableExpr {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for HashableExpr {}
