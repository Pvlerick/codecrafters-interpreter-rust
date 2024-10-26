use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    io::{stdout, BufRead, Write},
    ops::Deref,
    rc::Rc,
};

use crate::{
    environment::Environment,
    errors::{ErrorMessage, InterpreterError},
    parser::{Expr, Parser, Statement},
    resolver::{HashableExpr, Resolver},
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    parser: Option<Parser>,
    global_environment: Environment<Type>,
    resolve_table: Option<HashMap<HashableExpr, usize>>,
    output: Rc<RefCell<dyn Write>>,
    pub has_parsing_errors: bool,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Self::with_output(parser, Rc::new(RefCell::new(stdout())))
    }

    pub fn with_output(parser: Parser, output: Rc<RefCell<dyn Write>>) -> Self {
        Self {
            parser: Some(parser),
            global_environment: Interpreter::new_global_environment(),
            resolve_table: None,
            output,
            has_parsing_errors: false,
        }
    }

    pub fn build<R>(reader: R, output: Rc<RefCell<dyn Write>>) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        let parser = Parser::build(reader)?;

        Ok(Interpreter::with_output(parser, output))
    }

    fn new_global_environment() -> Environment<Type> {
        let env = Environment::<Type>::new();
        env.define("clock", Type::Function(Rc::new(native_functions::Clock {})));
        env.define("env", Type::Function(Rc::new(native_functions::Env {})));
        env
    }

    pub fn evaluate(&mut self) -> Result<(), InterpreterError> {
        match self.parser.take() {
            Some(mut parser) => {
                match parser.parse_expression()? {
                    Some(expr) => {
                        let result =
                            self.eval(&Interpreter::new_global_environment(), &Rc::new(expr))?;
                        write!(self.output.borrow_mut(), "{}", result)
                            .expect("cannot write to output");
                    }
                    _ => {}
                }

                if let Some(errors) = parser.errors() {
                    return Err(errors);
                }

                Ok(())
            }
            None => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                "Interpreter's statements have already been consumed",
                None,
            ))),
        }
    }

    pub fn run(&mut self) -> Result<(), InterpreterError> {
        let environment = self.global_environment.clone();
        match self.parser.take() {
            Some(mut parser) => {
                let statements = parser.parse()?.collect::<Vec<_>>();
                let mut resolver = Resolver::new();
                resolver.resolve(&statements)?;
                self.resolve_table = Some(resolver.resolve_table);
                for statement in statements {
                    self.execute_statement(&statement, &environment)?;
                }

                if let Some(errors) = parser.errors() {
                    return Err(errors);
                }

                Ok(())
            }
            None => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                "Interpreter's statements have already been consumed",
                None,
            ))),
        }
    }

    fn execute_statement(
        &mut self,
        statement: &Statement,
        environment: &Environment<Type>,
    ) -> Result<StatementResult, InterpreterError> {
        match statement {
            Statement::Class(token, _methods) => {
                environment.define(&token.lexeme, Type::Nil);
                let class = LoxClass::new(token.lexeme.to_owned());
                environment
                    .assign(&token.lexeme, Type::Instance(Rc::new(class)))
                    .expect("should never fail");
                Ok(StatementResult::Empty)
            }
            Statement::Return(expr) => Ok(match expr {
                Some(expr) => StatementResult::Return(self.eval(environment, expr)?),
                None => StatementResult::Return(Type::Nil),
            }),
            Statement::Variable(token, Some(expr)) => {
                let value = self.eval(environment, expr)?;
                environment.define(&token.lexeme, value);
                Ok(StatementResult::Empty)
            }
            Statement::Variable(token, None) => {
                environment.define(&token.lexeme, Type::Nil);
                Ok(StatementResult::Empty)
            }
            Statement::Print(expr) => {
                let res = self.eval(environment, &expr)?;
                writeln!(self.output.borrow_mut(), "{}", res).expect("cannot write to output");
                Ok(StatementResult::Empty)
            }
            Statement::Expression(expr) => {
                self.eval(environment, &expr)?;
                Ok(StatementResult::Empty)
            }
            Statement::Block(statements) => {
                let mut enclosing_environment = environment.enclose();
                for statement in statements.iter() {
                    match self.execute_statement(statement, &mut enclosing_environment)? {
                        StatementResult::Return(t) => return Ok(StatementResult::Return(t)),
                        _ => {}
                    }
                }
                Ok(StatementResult::Empty)
            }
            Statement::If(condition, then_branch, None) => {
                if Interpreter::is_truthy(&self.eval(environment, &condition)?) {
                    self.execute_statement(then_branch, environment)
                } else {
                    Ok(StatementResult::Empty)
                }
            }
            Statement::If(condition, then_branch, Some(else_branch)) => {
                if Interpreter::is_truthy(&self.eval(environment, &condition)?) {
                    self.execute_statement(then_branch, environment)
                } else {
                    self.execute_statement(else_branch, environment)
                }
            }
            Statement::While(condition, body) => {
                while Interpreter::is_truthy(&self.eval(environment, condition)?) {
                    match self.execute_statement(body, environment)? {
                        StatementResult::Return(t) => return Ok(StatementResult::Return(t)),
                        _ => {}
                    }
                }
                Ok(StatementResult::Empty)
            }
        }
    }
    fn eval(
        &mut self,
        environment: &Environment<Type>,
        expression: &Rc<Expr>,
    ) -> Result<Type, InterpreterError> {
        match expression.deref() {
            Expr::Literal(token) => match token.token_type {
                TokenType::True => Ok(Type::Boolean(true)),
                TokenType::False => Ok(Type::Boolean(false)),
                TokenType::Nil => Ok(Type::Nil),
                _ => Ok(token
                    .literal
                    .as_ref()
                    .expect("token should have a literal")
                    .as_ref()
                    .into()),
            },
            Expr::Logical(token, left, right) => match token.token_type {
                TokenType::And | TokenType::Or => {
                    let left = self.eval(environment, left)?;
                    match (token.token_type, Interpreter::is_truthy(&left)) {
                        (TokenType::Or, true) => Ok(left),
                        (TokenType::And, false) => Ok(left),
                        _ => self.eval(environment, right),
                    }
                }
                _ => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                    "Logical operator should be 'or' or 'and'",
                    Some(token.line),
                ))),
            },
            Expr::Grouping(e) => self.eval(environment, e),
            Expr::Unary(token, expr) => match token.token_type {
                TokenType::Minus => match self.eval(environment, expr)? {
                    Type::Number(n) => Ok(Type::Number(-n)),
                    _ => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                        "Operand must be a number",
                        Some(token.line),
                    ))),
                },
                TokenType::Bang => Ok(Type::Boolean(!Interpreter::is_truthy(
                    &self.eval(environment, expr)?,
                ))),
                _ => panic!("oh no..."),
            },
            Expr::Binary(token, left, right) => match (
                token.token_type,
                self.eval(environment, left)?,
                self.eval(environment, right)?,
            ) {
                (TokenType::Plus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a + b)),
                (TokenType::Plus, Type::String(a), Type::String(b)) => {
                    Ok(Type::String(Rc::new(format!("{}{}", a, b))))
                }
                (TokenType::Plus, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be two numbers or two strings",
                    token.line,
                )),
                (TokenType::Minus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a - b)),
                (TokenType::Minus, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be two numbers or two strings",
                    token.line,
                )),
                (TokenType::Slash, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a / b)),
                (TokenType::Slash, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be numbers",
                    token.line,
                )),
                (TokenType::Star, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a * b)),
                (TokenType::Greater, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a > b)),
                (TokenType::GreaterEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a >= b))
                }
                (TokenType::Less, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a < b)),
                (TokenType::LessEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a <= b))
                }
                (TokenType::EqualEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a == b))
                }
                (TokenType::BangEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a != b))
                }
                (TokenType::EqualEqual, Type::Boolean(a), Type::Boolean(b)) => {
                    Ok(Type::Boolean(a == b))
                }
                (TokenType::BangEqual, Type::Boolean(a), Type::Boolean(b)) => {
                    Ok(Type::Boolean(a != b))
                }
                (TokenType::EqualEqual, Type::String(a), Type::String(b)) => {
                    Ok(Type::Boolean(a == b))
                }
                (TokenType::BangEqual, Type::String(a), Type::String(b)) => {
                    Ok(Type::Boolean(a != b))
                }
                (TokenType::EqualEqual, _, _) => Ok(Type::Boolean(false)),
                (TokenType::BangEqual, _, _) => Ok(Type::Boolean(false)),
                _ => Err(InterpreterError::evaluating(
                    "Unrecognized binary expression",
                    token.line,
                )),
            },
            Expr::Variable(token) => {
                let distance = self
                    .resolve_table
                    .as_ref()
                    .unwrap()
                    .get(&HashableExpr::from(expression.clone()));
                match (
                    distance.and_then(|i| environment.get_at(&token.lexeme, *i)),
                    self.global_environment.get(&token.lexeme),
                ) {
                    (Some(value), _) => Ok(value.clone()),
                    (None, Some(value)) => Ok(value.clone()),
                    _ => Err(InterpreterError::evaluating(
                        format!("Undefined variable in scope: '{}'", token.lexeme),
                        token.line,
                    )),
                }
            }
            Expr::Assignment(token, expr) => {
                let name = token.lexeme.to_owned();
                let value = self.eval(environment, expr)?;
                match environment.assign(name, value.clone()) {
                    Ok(()) => Ok(value),
                    Err(()) => Err(InterpreterError::evaluating(
                        format!("Undefined variable '{}'", token.lexeme),
                        token.line,
                    )),
                }
            }
            Expr::Call(callee, right_paren, arguments) => {
                let callee = self.eval(environment, callee)?;

                let mut args = Vec::new();
                for arg in arguments.iter() {
                    args.push(self.eval(environment, arg)?);
                }

                match callee {
                    Type::Function(func) => {
                        if args.len() != func.arity() {
                            return InterpreterError::evaluating(
                                format!(
                                    "Expected {} arguments for function '{}' but got {}",
                                    func.arity(),
                                    func,
                                    args.len()
                                ),
                                right_paren.line,
                            )
                            .into();
                        }
                        match func.call(self, args, right_paren.line)? {
                            StatementResult::Empty | StatementResult::Return(Type::Nil) => {
                                Ok(Type::Nil)
                            }
                            StatementResult::Return(t) => Ok(t),
                        }
                    }
                    Type::Instance(_inst) => {
                        todo!();
                    }
                    _ => Err(InterpreterError::evaluating(
                        "Can only call functions and instances",
                        right_paren.line,
                    )),
                }
            }
            Expr::Function(token, fun) => Ok(Type::Function(Rc::new(LoxFunction::new(
                token.as_ref().map(|i| i.lexeme.to_owned()),
                fun.parameters
                    .iter()
                    .map(|i| i.lexeme.to_string())
                    .collect(),
                fun.body.clone(),
                environment.clone(),
            )))),
            Expr::Get(expr, token) => match self.eval(environment, expr)? {
                Type::Instance(instance) => instance.get(&token.lexeme),
                _ => Err(InterpreterError::evaluating(
                    "Only instances have properties",
                    token.line,
                )),
            },
        }
    }

    fn is_truthy(t: &Type) -> bool {
        match t {
            Type::Nil => false,
            Type::Boolean(b) => *b,
            _ => true,
        }
    }
}

enum StatementResult {
    Return(Type),
    Empty,
}

#[derive(Debug, Clone)]
enum Type {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
    Function(Rc<dyn Function>),
    Instance(Rc<dyn Instance>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Nil => write!(f, "nil"),
            Type::Number(n) => write!(f, "{}", n),
            Type::String(s) => write!(f, "{}", s),
            Type::Boolean(b) => write!(f, "{}", b),
            Type::Function(fun) => write!(f, "{}", fun),
            Type::Instance(class) => write!(f, "{}", class),
        }
    }
}

impl From<&Literal> for Type {
    fn from(value: &Literal) -> Self {
        match value {
            Literal::Digit(n) => Type::Number(*n),
            Literal::String(s) => Type::String(s.clone()),
        }
    }
}

trait Function: Debug + Display {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Type>,
        line: usize,
    ) -> Result<StatementResult, InterpreterError>;

    fn arity(&self) -> usize;
}

struct LoxFunction {
    name: String,
    parameters: Vec<String>,
    body: Rc<Statement>,
    closure: Environment<Type>,
}

impl LoxFunction {
    fn new(
        name: Option<String>,
        parameters: Vec<String>,
        body: Rc<Statement>,
        closure: Environment<Type>,
    ) -> Self {
        Self {
            name: name
                .map(|i| i.to_string())
                .unwrap_or_else(|| "__<fun_anon>".to_owned()),
            parameters,
            body,
            closure,
        }
    }
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoxFunction {{ name: \"{:?}\", parameters: {:?}, body: {:?} }}",
            self.name, self.parameters, self.body
        )
    }
}

impl Function for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Type>,
        _: usize,
    ) -> Result<StatementResult, InterpreterError> {
        let env = Environment::enclose(&self.closure);

        for arg in self.parameters.iter().zip(arguments) {
            env.define(arg.0, arg.1);
        }

        interpreter.execute_statement(&self.body, &env)
    }

    fn arity(&self) -> usize {
        self.parameters.len()
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fun {}({})", self.name, self.arity())
    }
}

mod native_functions {
    use std::{
        env,
        fmt::Display,
        rc::Rc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::errors::{ErrorMessage, InterpreterError};

    use super::{Function, Interpreter, StatementResult, Type};

    #[derive(Debug)]
    pub struct Clock {}

    impl Function for Clock {
        fn arity(&self) -> usize {
            0
        }

        fn call(
            &self,
            _: &mut Interpreter,
            _: Vec<super::Type>,
            line: usize,
        ) -> Result<StatementResult, InterpreterError> {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Ok(StatementResult::Return(Type::Number(
                    duration.as_secs() as f64
                ))),
                Err(error) => Err(InterpreterError::RuntimeError(ErrorMessage::new(
                    format!("System time error: {}", error),
                    Some(line),
                ))),
            }
        }
    }

    impl Display for Clock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "clock")
        }
    }

    #[derive(Debug)]
    pub struct Env {}

    impl Function for Env {
        fn arity(&self) -> usize {
            1
        }

        fn call(
            &self,
            _: &mut Interpreter,
            arguments: Vec<Type>,
            line: usize,
        ) -> Result<StatementResult, InterpreterError> {
            match arguments.as_slice() {
                [Type::String(key)] => match env::var(key.as_str()) {
                    Ok(value) => Ok(StatementResult::Return(Type::String(Rc::new(value)))),
                    Err(_) => Ok(StatementResult::Return(Type::Nil)),
                },
                _ => Err(InterpreterError::RuntimeError(ErrorMessage::new(
                    "Invalid argument to 'env' function",
                    Some(line),
                ))),
            }
        }
    }

    impl Display for Env {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "env")
        }
    }
}

trait Instance: Debug + Display {
    #[allow(dead_code)]
    fn call_method(
        &self,
        name: String,
        interpreter: &mut Interpreter,
        arguments: Vec<Type>,
        line: usize,
    ) -> Result<StatementResult, InterpreterError>;

    fn get(&self, name: &str) -> Result<Type, InterpreterError>;
    #[allow(dead_code)]
    fn set(&mut self, name: &str, value: Type);
}

#[derive(Debug, Clone)]
struct LoxClass {
    name: String,
}

impl LoxClass {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Instance for LoxClass {
    fn call_method(
        &self,
        _name: String,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Type>,
        _line: usize,
    ) -> Result<StatementResult, InterpreterError> {
        todo!()
    }

    fn get(&self, _name: &str) -> Result<Type, InterpreterError> {
        todo!()
    }

    fn set(&mut self, _name: &str, _value: Type) {
        todo!()
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "class {} {{...}}", self.name)
    }
}

impl Function for LoxClass {
    fn call(
        &self,
        _: &mut Interpreter,
        _: Vec<Type>,
        _: usize,
    ) -> Result<StatementResult, InterpreterError> {
        let instance = LoxInstance::new(self.clone());
        Ok(StatementResult::Return(Type::Instance(Rc::new(instance))))
    }

    fn arity(&self) -> usize {
        0
    }
}

#[derive(Debug)]
struct LoxInstance {
    class: LoxClass,
    fields: HashMap<String, Type>,
}

impl LoxInstance {
    fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }
}

impl Instance for LoxInstance {
    fn call_method(
        &self,
        _name: String,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Type>,
        _line: usize,
    ) -> Result<StatementResult, InterpreterError> {
        Ok(StatementResult::Empty)
    }

    fn get(&self, name: &str) -> Result<Type, InterpreterError> {
        self.fields.get(name).map(|i| i.clone()).ok_or_else(|| {
            InterpreterError::RuntimeError(ErrorMessage::new("Undefined property '{name}'", None))
        })
    }

    fn set(&mut self, name: &str, value: Type) {
        self.fields.insert(name.to_owned(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
