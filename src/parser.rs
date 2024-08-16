use std::{fmt::Display, iter::Peekable};

use crate::scanner::{Token, TokenType};

/* Grammar:

expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;
*/

pub struct Parser<T>
where
    T: Iterator<Item = Token>,
{
    tokens: Peekable<T>,
}

macro_rules! gramar_rule {
    ($name:ident, $base:ident, $($token_type:expr),+) => {
        fn $name(&mut self) -> Expr {
            let mut token_types = Vec::new();
            $(
                token_types.push($token_type);
            )+
            let mut expr = self.$base();

            while let Some(operator) = self.next_matches(&token_types) {
                let right = self.$base();
                expr = Expr::binary(operator, expr, right);
            }

            expr
        }
    };
}

impl<T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn new(tokens: T) -> Self {
        let tokens = tokens.peekable();
        Parser { tokens }
    }

    pub fn parse(&mut self) {
        let x = self.expression();

        println!("{}", x);
    }

    pub fn expression(&mut self) -> Expr {
        self.equality()
    }

    gramar_rule!(
        equality,
        comparison,
        TokenType::BangEqual,
        TokenType::EqualEqual
    );
    gramar_rule!(
        comparison,
        term,
        TokenType::Greater,
        TokenType::GreaterEqual,
        TokenType::Less,
        TokenType::LessEqual
    );
    gramar_rule!(term, factor, TokenType::Minus, TokenType::Plus);
    gramar_rule!(factor, unary, TokenType::Slash, TokenType::Star);

    fn unary(&mut self) -> Expr {
        use TokenType::*;
        if let Some(operator) = self.next_matches(&vec![Bang, Minus]) {
            let right = self.unary();
            return Expr::unary(operator, right);
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if let Some(token) = self.tokens.next() {
            use TokenType::*;
            match token.token_type {
                False | True | Nil | Number | String => return Expr::literal(token),
                LeftParenthesis => {
                    let expr = self.expression();
                    if let Some(_) = self.next_matches(&vec![RightParenthesis]) {
                        return Expr::grouping(expr);
                    } else {
                        panic!("booom");
                    }
                }
                tt => {
                    println!("eeeef: {}", tt);
                    panic!("eeeeeef");
                }
            }
        } else {
            Expr::literal(Token::new(TokenType::EOF, ""))
        }
    }

    fn next_matches(&mut self, types: &Vec<TokenType>) -> Option<Token> {
        self.tokens.next_if(|i| types.contains(&i.token_type))
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
