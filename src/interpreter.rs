use std::{
    error::Error,
    fmt::Display,
    io::{sink, BufRead, Write},
};

use crate::{
    parser::{Expr, Parser, Statement, StatementsIterator},
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    statements: Option<StatementsIterator>,
}

impl Interpreter {
    pub fn new(statements: StatementsIterator) -> Self {
        Interpreter {
            statements: Some(statements),
        }
    }

    pub fn build<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        let parser = Parser::build(reader)?;
        let statements = parser.parse()?;

        Ok(Interpreter::new(statements))
    }

    pub fn evaluate<T>(mut self, output: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: Write,
    {
        println!("FOO");
        match self.statements.take() {
            Some(statements) => {
                println!("BAR");
                for statement in statements {
                    println!("BAZ");
                    let mut sink = sink();
                    match self.execute(statement, &mut sink)? {
                        Some(value) => {
                            writeln!(output, "{}", value)?;
                        }
                        None => {}
                    }
                }
                println!("END");
                Ok(())
            }
            None => Err("Interpreter's statements have already been consumed".into()),
        }
    }

    pub fn run<T>(mut self, output: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: Write,
    {
        match self.statements.take() {
            Some(statements) => {
                for statement in statements {
                    self.execute(statement, output)?;
                }
                Ok(())
            }
            None => Err("Interpreter's statements have already been consumed".into()),
        }
    }

    fn execute<T>(
        &self,
        statement: Statement,
        output: &mut T,
    ) -> Result<Option<Type>, Box<dyn Error>>
    where
        T: Write,
    {
        match statement {
            Statement::Print(expr) => {
                writeln!(output, "{}", eval(&expr)?)?;
                Ok(None)
            }
            Statement::Expression(expr) => Ok(Some(eval(&expr)?)),
        }
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
        Expr::Grouping(e) => eval(e),
        Expr::Unary(t, e) => match t.token_type {
            TokenType::Minus => match eval(e)? {
                Type::Number(n) => Ok(Type::Number(-n)),
                _ => Err("Operand must be a number.".into()),
            },
            TokenType::Bang => Ok(Type::Boolean(!is_truthy(eval(e)?))),
            _ => panic!("oh no..."),
        },
        Expr::Binary(t, l, r) => match (t.token_type, eval(l)?, eval(r)?) {
            (TokenType::Plus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a + b)),
            (TokenType::Plus, Type::String(a), Type::String(b)) => {
                Ok(Type::String(format!("{}{}", a, b)))
            }
            (TokenType::Plus, _, _) => Err("Operands must be two numbers or two strings.".into()),
            (TokenType::Minus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a - b)),
            (TokenType::Minus, _, _) => Err("Operands must be two numbers or two strings.".into()),
            (TokenType::Slash, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a / b)),
            (TokenType::Slash, _, _) => Err("Operands must be numbers.".into()),
            (TokenType::Star, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a * b)),
            (TokenType::Greater, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a > b)),
            (TokenType::GreaterEqual, Type::Number(a), Type::Number(b)) => {
                Ok(Type::Boolean(a >= b))
            }
            (TokenType::Less, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a < b)),
            (TokenType::LessEqual, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a <= b)),
            (TokenType::EqualEqual, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a == b)),
            (TokenType::BangEqual, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a != b)),
            (TokenType::EqualEqual, Type::String(a), Type::String(b)) => Ok(Type::Boolean(a == b)),
            (TokenType::BangEqual, Type::String(a), Type::String(b)) => Ok(Type::Boolean(a != b)),
            (TokenType::EqualEqual, _, _) => Ok(Type::Boolean(false)),
            (TokenType::BangEqual, _, _) => Ok(Type::Boolean(false)),
            _ => Err("hello".into()),
        },
    }
}

fn is_truthy(t: Type) -> bool {
    match t {
        Type::Nil => false,
        Type::Boolean(b) => b,
        _ => true,
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
