use std::fmt::Display;

use crate::{
    parser::Expr,
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    expression: Expr,
}

impl Interpreter {
    pub fn new(expression: Expr) -> Self {
        Interpreter { expression }
    }

    pub fn evaluate(&self) -> String {
        eval(&self.expression).to_string()
    }
}

fn eval(expression: &Expr) -> Type {
    match expression {
        // Expr::Literal(t) => t.literal.as_ref().unwrap().to_string(),
        Expr::Literal(t) => match t.token_type {
            TokenType::True => Type::Boolean(true),
            TokenType::False => Type::Boolean(false),
            _ => match &t.literal {
                Some(Literal::Digit(n)) => Type::Number(*n),
                Some(Literal::String(s)) => Type::String(s.clone()),
                _ => panic!("unreachable"),
            },
        },
        Expr::Grouping(e) => eval(e),
        Expr::Unary(t, e) => match t.token_type {
            TokenType::Minus => match eval(e) {
                Type::Number(n) => Type::Number(-n),
                _ => panic!("unreachable"),
            },
            TokenType::Bang => match eval(e) {
                _ => panic!("foo"),
            },
            _ => panic!("unreachable arm"),
        },
        _ => panic!("bar"),
    }
}

enum Type {
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
