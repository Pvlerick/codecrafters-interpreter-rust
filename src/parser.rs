use std::{fmt::Display, iter::Peekable};

use crate::scanner::{Token, TokenType};

/* Grammar:

expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

*/

pub struct Parser<T>
where
    T: Iterator<Item = Token>,
{
    tokens: Peekable<T>,
    error: Option<Vec<String>>,
}

macro_rules! gramar_rule {
    ($name:ident, $base:ident, $token_types:expr) => {
        fn $name(&mut self) -> Result<Expr, ()> {
            let mut expr = self.$base()?;

            while let Some(operator) = self.next_matches($token_types) {
                let right = self.$base()?;
                expr = Expr::binary(operator, expr, right);
            }

            Ok(expr)
        }
    };
}

impl<T> Parser<T>
where
    T: Iterator<Item = Token>,
{
    pub fn new(tokens: T) -> Self {
        let tokens = tokens.peekable();
        Parser {
            tokens,
            error: None,
        }
    }

    pub fn parse(&mut self) -> Result<String, &Vec<String>> {
        match self.expression() {
            Ok(e) => Ok(e.to_string()),
            Err(_) => Err(self.error.as_ref().unwrap()),
        }
    }

    fn error(&mut self, error: String) {
        self.error.get_or_insert(vec![]).push(error);
        self.synchronize();
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        while let Some(token_type) = self.tokens.peek().map(|i| i.token_type) {
            match token_type {
                Semicolon | Class | For | Fun | If | Print | Return | Var | While => {}
                _ => _ = self.tokens.next(),
            }
        }
    }

    pub fn expression(&mut self) -> Result<Expr, ()> {
        self.equality()
    }

    gramar_rule!(
        equality,
        comparison,
        [TokenType::BangEqual, TokenType::EqualEqual]
    );
    gramar_rule!(
        comparison,
        term,
        [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ]
    );
    gramar_rule!(term, factor, [TokenType::Minus, TokenType::Plus]);
    gramar_rule!(factor, unary, [TokenType::Slash, TokenType::Star]);

    fn unary(&mut self) -> Result<Expr, ()> {
        use TokenType::*;
        if let Some(operator) = self.next_matches([Bang, Minus]) {
            let right = self.unary()?;
            return Ok(Expr::unary(operator, right));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        if let Some(token) = self.tokens.next() {
            use TokenType::*;
            match token.token_type {
                False | True | Nil | Number | String => return Ok(Expr::literal(token)),
                LeftParenthesis => {
                    let expr = self.expression()?;
                    if let Some(_) = self.next_matches(RightParenthesis) {
                        return Ok(Expr::grouping(expr));
                    } else {
                        self.error("Expect ')' after expression.".to_string());
                        Err(())
                    }
                }
                _ => {
                    self.error("Expect expression.".to_string());
                    Err(())
                }
            }
        } else {
            Ok(Expr::literal(Token::new(TokenType::EOF, "", 0)))
        }
    }

    fn next_matches<M: TokenTypeMatcher>(&mut self, matcher: M) -> Option<Token> {
        self.tokens.next_if(|i| matcher.matches(&i.token_type))
    }
}

trait TokenTypeMatcher {
    fn matches(&self, token_type: &TokenType) -> bool;
}

impl TokenTypeMatcher for TokenType {
    fn matches(&self, token_type: &TokenType) -> bool {
        self == token_type
    }
}

impl<const N: usize> TokenTypeMatcher for [TokenType; N] {
    fn matches(&self, token_type: &TokenType) -> bool {
        self.contains(token_type)
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
