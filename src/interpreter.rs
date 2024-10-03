use std::{
    cell::RefCell,
    error::Error,
    fmt::{Debug, Display},
    io::{stdout, BufRead, Write},
    rc::Rc,
};

use crate::{
    environment::Environment,
    errors::{ErrorMessage, InterpreterError},
    parser::{Expr, Parser, Statement},
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    parser: Option<Parser>,
    global_environment: Environment<Type>,
    output: Rc<RefCell<dyn Write>>,
    pub has_parsing_errors: bool,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Self {
            parser: Some(parser),
            global_environment: Interpreter::new_global_environment(),
            output: Rc::new(RefCell::new(stdout())),
            has_parsing_errors: false,
        }
    }

    pub fn with_output(parser: Parser, output: Rc<RefCell<dyn Write>>) -> Self {
        Self {
            parser: Some(parser),
            global_environment: Interpreter::new_global_environment(),
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

        env.define(
            "clock",
            Type::Function("clock".to_owned(), Rc::new(native_functions::Clock {})),
        );

        env.define(
            "env",
            Type::Function("env".to_owned(), Rc::new(native_functions::Env {})),
        );

        env
    }

    pub fn evaluate(&mut self) -> Result<(), InterpreterError> {
        match self.parser.take() {
            Some(mut parser) => {
                match parser.parse_expression()? {
                    Some(expr) => {
                        let result = self.eval(&Interpreter::new_global_environment(), &expr)?;
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
                for statement in parser.parse()? {
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
            Statement::Return(expr) => Ok(match expr {
                Some(expr) => StatementResult::Return(self.eval(environment, expr)?),
                None => StatementResult::Return(Type::Nil),
            }),
            Statement::Function(name, parameters, body) => {
                environment.define(
                    name,
                    Type::Function(
                        name.to_owned(),
                        Rc::new(LoxFunction::new(
                            name,
                            parameters.iter().map(|i| i.lexeme.to_string()).collect(),
                            body.clone(),
                        )),
                    ),
                );

                Ok(StatementResult::Empty)
            }
            Statement::Variable(name, Some(expr)) => {
                let name = name.to_owned();
                let value = self.eval(environment, expr)?;
                environment.define(name, value);
                Ok(StatementResult::Empty)
            }
            Statement::Variable(name, None) => {
                environment.define(name, Type::Nil);
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
                    let _ = self.execute_statement(body, environment);
                }
                Ok(StatementResult::Empty)
            }
        }
    }
    fn eval(
        &mut self,
        environment: &Environment<Type>,
        expression: &Expr,
    ) -> Result<Type, InterpreterError> {
        match expression {
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
            Expr::Variable(token) => match environment.get(&token.lexeme) {
                Some(value) => Ok(value.clone()),
                None => Err(InterpreterError::evaluating(
                    format!("Undefined variable '{}'", token.lexeme),
                    token.line,
                )),
            },
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
                    Type::Function(name, func) => {
                        if args.len() != func.arity() {
                            return InterpreterError::evaluating(
                                format!(
                                    "Expected {} arguments for function '{}' but got {}",
                                    func.arity(),
                                    name,
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
                    _ => Err(InterpreterError::evaluating(
                        "Can only call functions and classes",
                        right_paren.line,
                    )),
                }
            }
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
    Function(String, Rc<dyn Function>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Nil => write!(f, "nil"),
            Type::Number(n) => write!(f, "{}", n),
            Type::String(s) => write!(f, "{}", s),
            Type::Boolean(b) => write!(f, "{}", b),
            Type::Function(n, _) => write!(f, "{}()", n),
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

#[derive(Debug)]
struct LoxFunction {
    name: String,
    parameters: Vec<String>,
    body: Rc<Statement>,
}

impl LoxFunction {
    fn new<T: ToString>(name: T, parameters: Vec<String>, body: Rc<Statement>) -> Self {
        Self {
            name: name.to_string(),
            parameters,
            body,
        }
    }
}

impl Function for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Type>,
        _: usize,
    ) -> Result<StatementResult, InterpreterError> {
        let env = Environment::enclose(&interpreter.global_environment);

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
        write!(f, "{}()", self.name)
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
            write!(f, "clock()")
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
            write!(f, "env(key)")
        }
    }
}
