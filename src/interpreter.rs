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
            TokenType::Nil => Type::Nil,
            _ => match &t.literal {
                Some(Literal::Digit(n)) => Type::Number(*n),
                Some(Literal::String(s)) => Type::String(s.clone()),
                _ => panic!("oh no..."),
            },
        },
        Expr::Grouping(e) => eval(e),
        Expr::Unary(t, e) => match t.token_type {
            TokenType::Minus => match eval(e) {
                Type::Number(n) => Type::Number(-n),
                _ => panic!("oh no..."),
            },
            TokenType::Bang => Type::Boolean(!is_truthy(eval(e))),
            _ => panic!("oh no..."),
        },
        Expr::Binary(t, l, r) => match (t.token_type, eval(l), eval(r)) {
            (TokenType::Plus, Type::Number(a), Type::Number(b)) => Type::Number(a + b),
            (TokenType::Minus, Type::Number(a), Type::Number(b)) => Type::Number(a - b),
            (TokenType::Slash, Type::Number(a), Type::Number(b)) => Type::Number(a / b),
            (TokenType::Star, Type::Number(a), Type::Number(b)) => Type::Number(a * b),
            (TokenType::Greater, Type::Number(a), Type::Number(b)) => Type::Boolean(a > b),
            (TokenType::GreaterEqual, Type::Number(a), Type::Number(b)) => Type::Boolean(a >= b),
            (TokenType::Less, Type::Number(a), Type::Number(b)) => Type::Boolean(a < b),
            (TokenType::LessEqual, Type::Number(a), Type::Number(b)) => Type::Boolean(a <= b),
            (TokenType::EqualEqual, Type::Number(a), Type::Number(b)) => Type::Boolean(a == b),
            (TokenType::BangEqual, Type::Number(a), Type::Number(b)) => Type::Boolean(a != b),
            (TokenType::Plus, Type::String(a), Type::String(b)) => {
                Type::String(format!("{}{}", a, b))
            }
            (TokenType::EqualEqual, Type::String(a), Type::String(b)) => Type::Boolean(a == b),
            (TokenType::BangEqual, Type::String(a), Type::String(b)) => Type::Boolean(a != b),
            (TokenType::EqualEqual, _, _) => Type::Boolean(false),
            (TokenType::BangEqual, _, _) => Type::Boolean(false),
            _ => panic!("oh no..."),
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
