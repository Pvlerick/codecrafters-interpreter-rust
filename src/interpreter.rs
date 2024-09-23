use std::{
    error::Error,
    fmt::Display,
    io::{BufRead, Write},
    rc::Rc,
};

use crate::{
    environment::Environment,
    errors::{ErrorMessage, InterpreterError},
    parser::{Declaration, Expr, Parser, Statement},
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    parser: Option<Parser>,
    pub has_parsing_errors: bool,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Self {
            parser: Some(parser),
            has_parsing_errors: false,
        }
    }

    pub fn build<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        let parser = Parser::build(reader)?;

        Ok(Interpreter::new(parser))
    }

    pub fn evaluate<T>(&mut self, output: &mut T) -> Result<(), InterpreterError>
    where
        T: Write,
    {
        let environment = Environment::new();
        match self.parser.take() {
            Some(mut parser) => {
                match parser.parse_expression()? {
                    Some(expr) => {
                        let result = self.eval(&environment, &expr)?;
                        write!(output, "{}", result).expect("cannot write to output");
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

    pub fn run<T>(&mut self, output: &mut T) -> Result<(), InterpreterError>
    where
        T: Write,
    {
        let environment = Environment::new();
        match self.parser.take() {
            Some(mut parser) => {
                for declaration in parser.parse()? {
                    self.execute(&declaration, &environment, output)?;
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

    fn execute<T>(
        &mut self,
        declaration: &Declaration,
        environment: &Environment<Type>,
        output: &mut T,
    ) -> Result<Option<Type>, InterpreterError>
    where
        T: Write,
    {
        match declaration {
            Declaration::Variable(name, Some(expr)) => {
                let name = name.to_owned();
                let value = self.eval(environment, expr)?;
                environment.define(name, value);
                Ok(None)
            }
            Declaration::Variable(name, None) => {
                environment.define(name, Type::Nil);
                Ok(None)
            }
            Declaration::Statement(Statement::Print(expr)) => {
                writeln!(output, "{}", self.eval(environment, &expr)?)
                    .expect("cannot write to output");
                Ok(None)
            }
            Declaration::Statement(Statement::Expression(expr)) => {
                Ok(Some(self.eval(environment, &expr)?))
            }
            Declaration::Statement(Statement::Block(declarations)) => {
                let mut enclosing_environment = environment.enclose();
                for declaration in declarations.iter() {
                    self.execute(declaration, &mut enclosing_environment, output)?;
                }
                Ok(None)
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
            Expr::Grouping(e) => self.eval(environment, e),
            Expr::Unary(t, e) => match t.token_type {
                TokenType::Minus => match self.eval(environment, e)? {
                    Type::Number(n) => Ok(Type::Number(-n)),
                    _ => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                        "Operand must be a number.",
                        Some(t.line),
                    ))),
                },
                TokenType::Bang => Ok(Type::Boolean(!Interpreter::is_truthy(
                    self.eval(environment, e)?,
                ))),
                _ => panic!("oh no..."),
            },
            Expr::Binary(t, l, r) => match (
                t.token_type,
                self.eval(environment, l)?,
                self.eval(environment, r)?,
            ) {
                (TokenType::Plus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a + b)),
                (TokenType::Plus, Type::String(a), Type::String(b)) => {
                    Ok(Type::String(Rc::new(format!("{}{}", a, b))))
                }
                (TokenType::Plus, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be two numbers or two strings.",
                    t.line,
                )),
                (TokenType::Minus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a - b)),
                (TokenType::Minus, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be two numbers or two strings.",
                    t.line,
                )),
                (TokenType::Slash, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a / b)),
                (TokenType::Slash, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be numbers.",
                    t.line,
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
                    t.line,
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
        }
    }

    fn is_truthy(t: Type) -> bool {
        match t {
            Type::Nil => false,
            Type::Boolean(b) => b,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
enum Type {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Nil => write!(f, "nil"),
            Type::Number(n) => write!(f, "{}", n),
            Type::String(s) => write!(f, "{}", s),
            Type::Boolean(b) => write!(f, "{}", b),
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
