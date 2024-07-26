use std::fmt::Display;

use crate::scanner::{Token, TokenType};

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn parse<T>(&self, tokens: T)
    where
        T: Iterator<Item = Token>,
    {
        let x = Expr::binary(
            Token::new(TokenType::Star, "*"),
            Expr::unary(
                Token::new(TokenType::Minus, "-"),
                Expr::literal(Token::new(TokenType::Number, "123")),
            ),
            Expr::grouping(Expr::Literal(Token::new(TokenType::Number, "4567"))),
        );

        println!("expr: {}", x);
    }
}

pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>),
}

impl Expr {
    fn binary(token: Token, left: Expr, right: Expr) -> Self {
        Expr::Binary(token, Box::new(left), Box::new(right))
    }

    fn grouping(expr: Expr) -> Self {
        Expr::Grouping(Box::new(expr))
    }

    fn literal(token: Token) -> Self {
        Expr::Literal(token)
    }

    fn unary(token: Token, expr: Expr) -> Self {
        Expr::Unary(token, Box::new(expr))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;
        match self {
            Binary(token, left, right) => write!(f, "({} {} {})", token.display(), left, right,),
            Grouping(expr) => write!(f, "(group {})", expr),
            Literal(token) => write!(f, "{}", token.display()),
            Unary(token, expr) => write!(f, "({} {})", token.display(), expr),
        }
    }
}

