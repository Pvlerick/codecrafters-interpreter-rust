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

#[derive(Debug)]
enum FunctionType {
    Function,
    Method,
    Initializer,
}

#[derive(Debug)]
enum ClassType {
    Class,
    SubClass,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, Variable>>,
    current_class: Option<ClassType>,
    current_function: Option<FunctionType>,
    pub resolve_table: HashMap<HashableExpr, usize>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            current_class: None,
            current_function: None,
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
            Statement::Class(name, methods, super_class) => {
                let enclosing_class = self.current_class.take();

                if let Some(super_class) = super_class {
                    match super_class.deref() {
                        Expr::Variable(super_class) if super_class.lexeme == name.lexeme => {
                            return Err(InterpreterError::resolving(
                                "A class can't inherit from itself",
                                Some(name.line),
                            ));
                        }
                        Expr::Variable(_) => {}
                        _ => {
                            return Err(InterpreterError::resolving(
                                "A class can only inherit from a valid identifier",
                                Some(name.line),
                            ));
                        }
                    }

                    self.resolve_expression(super_class.clone())?;

                    self.begin_scope();
                    self.declare_and_define("super");

                    self.current_class = Some(ClassType::SubClass);
                } else {
                    self.current_class = Some(ClassType::Class);
                }

                self.begin_scope();

                self.declare(name)?;
                self.define(name);

                self.declare_and_define("this");

                for method in methods {
                    match method.as_deref() {
                        Some(Expr::Function(Some(token), method)) if token.lexeme == "init" => {
                            self.resolve_function(method, FunctionType::Initializer)?
                        }
                        Some(Expr::Function(_, method)) => {
                            self.resolve_function(method, FunctionType::Method)?
                        }
                        _ => {
                            return Err(InterpreterError::resolving(
                                "expression is not a method",
                                Some(name.line),
                            ))
                        }
                    }
                }

                self.end_scope();

                if super_class.is_some() {
                    self.end_scope();
                }

                self.current_class = enclosing_class;

                Ok(())
            }
            Statement::Block(statements) => {
                self.begin_scope();
                self.resolve(&statements)?;
                self.end_scope();
                Ok(())
            }
            Statement::Variable(name, initializer) => {
                self.declare(&name)?;
                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer.clone())?;
                }
                self.define(&name);
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
            Statement::Return(expr) => match (&self.current_function, expr) {
                (Some(FunctionType::Initializer), Some(_)) => Err(InterpreterError::resolving(
                    "Can't return a value from an initializer",
                    None,
                )),
                (Some(_), Some(expr)) => self.resolve_expression(expr.clone()),
                (Some(_), None) => Ok(()),
                (None, _) => Err(InterpreterError::resolving(
                    "Can't return from top level code",
                    None,
                )),
            },
            Statement::While(condition, body) => {
                self.resolve_expression(condition.clone())?;
                self.resolve_statement(body)
            }
        }
    }

    fn resolve_expression(&mut self, expr: Rc<Expr>) -> Result<(), InterpreterError> {
        match expr.deref() {
            Expr::Super(token, _) => match self.current_class {
                Some(ClassType::SubClass) => {
                    self.resolve_local(expr.clone(), &token.lexeme);
                    Ok(())
                }
                Some(ClassType::Class) => Err(InterpreterError::resolving(
                    "Can't use or 'super' in a class that has no super class",
                    Some(token.line),
                )),
                None => Err(InterpreterError::resolving(
                    "Can't use or 'super' outside of a class",
                    Some(token.line),
                )),
            },
            Expr::This(token) => {
                if self.current_class.is_none() {
                    return Err(InterpreterError::resolving(
                        "Can't use 'this' outside of a class",
                        Some(token.line),
                    ));
                }

                self.resolve_local(expr.clone(), &token.lexeme);
                Ok(())
            }
            Expr::Set(instance, _, value) => {
                self.resolve_expression(instance.clone())?;
                self.resolve_expression(value.clone())
            }
            Expr::Get(expr, _) => self.resolve_expression(expr.clone()),
            Expr::Variable(ref token) => {
                if self.scopes.last().map_or(false, |i| {
                    i.get(&token.lexeme).map(|i| !i.is_defined).unwrap_or(false)
                }) {
                    return Err(InterpreterError::resolving(
                        format!("Variable '{}' is used in its own initializer", token.lexeme),
                        Some(token.line),
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
                    self.define(token);
                }
                self.resolve_function(fun, FunctionType::Function)
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

    fn resolve_function(
        &mut self,
        function: &Function,
        function_type: FunctionType,
    ) -> Result<(), InterpreterError> {
        self.begin_scope();

        let enclosing_function = self.current_function.take();
        self.current_function = Some(function_type);

        for param in function.parameters.iter() {
            self.declare(&param)?;
            self.define(&param);
        }

        self.resolve_statement(&function.body)?;

        self.end_scope();

        self.current_function = enclosing_function;
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

    fn define(&mut self, token: &Token) {
        self.scopes.last_mut().and_then(|i| {
            i.entry(token.lexeme.to_string())
                .and_modify(|i| i.mark_as_defined());
            None::<()>
        });
    }

    fn declare_and_define(&mut self, name: &str) {
        let mut variable = Variable::new();
        variable.mark_as_defined();
        self.scopes
            .last_mut()
            .and_then(|i| i.insert(name.to_owned(), variable));
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
