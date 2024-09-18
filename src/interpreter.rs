use std::{
    error::Error,
    fmt::Display,
    io::{sink, BufRead, Write},
};

use crate::{
    parser::{Expr, Parser, Statement},
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

    pub fn evaluate<T, U>(
        &mut self,
        output: &mut T,
        err_output: &mut U,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Write,
        U: Write,
    {
        match self.parser.take() {
            Some(mut parser) => {
                for statement in parser.parse()? {
                    match self.execute(&statement, &mut sink())? {
                        Some(value) => {
                            println!("val: {}", value);
                            writeln!(output, "{}", value)?;
                        }
                        None => {}
                    }
                }

                if let Some(errors) = parser.errors() {
                    self.has_parsing_errors = true;
                    for error in errors {
                        writeln!(err_output, "{}", error)?;
                    }
                }

                Ok(())
            }
            None => Err("Interpreter's statements have already been consumed".into()),
        }
    }

    pub fn run<T, U>(&mut self, output: &mut T, err_output: &mut U) -> Result<(), Box<dyn Error>>
    where
        T: Write,
        U: Write,
    {
        match self.parser.take() {
            Some(mut parser) => {
                for statement in parser.parse()? {
                    match self.execute(&statement, output)? {
                        Some(value) => {
                            println!("val: {}", value);
                            writeln!(output, "{}", value)?;
                        }
                        None => {}
                    }
                }

                if let Some(errors) = parser.errors() {
                    self.has_parsing_errors = true;
                    for error in errors {
                        writeln!(err_output, "{}", error)?;
                    }
                }

                Ok(())
            }
            None => Err("Interpreter's statements have already been consumed".into()),
        }
    }

    fn execute<T>(
        &self,
        statement: &Statement,
        output: &mut T,
    ) -> Result<Option<Type>, Box<dyn Error>>
    where
        T: Write,
    {
        match statement {
            Statement::Print(expr) => {
                writeln!(output, "{}", Interpreter::eval(&expr)?)?;
                Ok(None)
            }
            Statement::Expression(expr) => Ok(Some(Interpreter::eval(&expr)?)),
        }
    }

    fn eval(expression: &Expr) -> Result<Type, Box<dyn Error>> {
        match expression {
            // Expr::Literal(t) => t.literal.as_ref().unwrap().to_string(),
            Expr::Literal(t) => match t.token_type {
                TokenType::True => Ok(Type::Boolean(true)),
                TokenType::False => Ok(Type::Boolean(false)),
                TokenType::Nil => Ok(Type::Nil),
                _ => match &t.literal {
                    Some(Literal::Digit(n)) => Ok(Type::Number(*n)),
                    Some(Literal::String(s)) => Ok(Type::String(s.clone())),
                    _ => panic!("oh no..."),
                },
            },
            Expr::Grouping(e) => Interpreter::eval(e),
            Expr::Unary(t, e) => match t.token_type {
                TokenType::Minus => match Interpreter::eval(e)? {
                    Type::Number(n) => Ok(Type::Number(-n)),
                    _ => Err("Operand must be a number.".into()),
                },
                TokenType::Bang => Ok(Type::Boolean(!Interpreter::is_truthy(Interpreter::eval(
                    e,
                )?))),
                _ => panic!("oh no..."),
            },
            Expr::Binary(t, l, r) => {
                match (t.token_type, Interpreter::eval(l)?, Interpreter::eval(r)?) {
                    (TokenType::Plus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a + b)),
                    (TokenType::Plus, Type::String(a), Type::String(b)) => {
                        Ok(Type::String(format!("{}{}", a, b)))
                    }
                    (TokenType::Plus, _, _) => {
                        Err("Operands must be two numbers or two strings.".into())
                    }
                    (TokenType::Minus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a - b)),
                    (TokenType::Minus, _, _) => {
                        Err("Operands must be two numbers or two strings.".into())
                    }
                    (TokenType::Slash, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a / b)),
                    (TokenType::Slash, _, _) => Err("Operands must be numbers.".into()),
                    (TokenType::Star, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a * b)),
                    (TokenType::Greater, Type::Number(a), Type::Number(b)) => {
                        Ok(Type::Boolean(a > b))
                    }
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
                    (TokenType::EqualEqual, Type::String(a), Type::String(b)) => {
                        Ok(Type::Boolean(a == b))
                    }
                    (TokenType::BangEqual, Type::String(a), Type::String(b)) => {
                        Ok(Type::Boolean(a != b))
                    }
                    (TokenType::EqualEqual, _, _) => Ok(Type::Boolean(false)),
                    (TokenType::BangEqual, _, _) => Ok(Type::Boolean(false)),
                    _ => Err("hello".into()),
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

enum Type {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
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

impl From<Literal> for Type {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Digit(n) => Type::Number(n),
            Literal::String(s) => Type::String(s),
        }
    }
}
