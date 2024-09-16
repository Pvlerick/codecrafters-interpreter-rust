use std::{error::Error, fmt::Display, io::BufRead};

use crate::{
    errors::ParsingError,
    scanner::{Token, TokenType},
    utils::StopOnFirstErrorIterator,
};

/* Grammar:

expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

*/

pub struct Parser<T, E>
where
    T: Iterator<Item = Result<Token, E>>,
    E: Error,
{
    tokens: StopOnFirstErrorIterator<T, Token, E>,
    peeked: Option<Token>,
    errors: Option<Vec<String>>,
}

macro_rules! gramar_rule {
    ($name:ident, $base:ident, $token_types:expr) => {
        fn $name(&mut self) -> Result<Option<Expr>, E> {
            let mut expr = self.$base()?;

            match expr {
                Some(_) => {
                    while let Some(operator) = self.peek_matches($token_types)? {
                        match self.$base()? {
                            Some(right) => {
                                expr = Some(Expr::binary(operator, expr.unwrap(), right))
                            }
                            None => return Ok(None),
                        }
                    }

                    Ok(expr)
                }
                None => Ok(None),
            }
        }
    };
}

impl<T, E> Parser<T, E>
where
    T: Iterator<Item = Result<Token, E>>,
    E: Error + 'static,
{
    pub fn new(tokens: T) -> Self {
        Parser {
            tokens: StopOnFirstErrorIterator::new(tokens),
            peeked: None,
            errors: None,
        }
    }

    pub fn build<R>(_reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        // let scanner = Scanner::new(reader);
        Err("hello".into())
        // Ok(Parser::new(scanner.scan_tokens()?))
    }

    pub fn parse(mut self) -> Result<Expr, Box<dyn Error>> {
        match self.expression() {
            Ok(Some(e)) => Ok(e),
            Ok(None) => Err(Box::new(ParsingError::from(
                self.errors.unwrap_or_else(|| Vec::<String>::new()),
            ))),
            // Ok(None) => Err(self.errors.unwrap_or_else(|| Vec::<String>::new()).into()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn error(&mut self, error: String) {
        self.errors.get_or_insert(vec![]).push(error);
        // self.synchronize();
    }

    // #[allow(dead_code)]
    // fn synchronize(&mut self) {
    //     use TokenType::*;
    //     println!("synchronizing...");
    //     while let Some(token_type) = self.tokens.peek().map(|i| i.token_type) {
    //         match token_type {
    //             Semicolon | Class | For | Fun | If | Print | Return | Var | While => {}
    //             _ => _ = self.tokens.next(),
    //         }
    //     }
    //     println!("done.");
    // }

    pub fn expression(&mut self) -> Result<Option<Expr>, E> {
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

    fn unary(&mut self) -> Result<Option<Expr>, E> {
        use TokenType::*;
        if let Some(operator) = self.peek_matches([Bang, Minus])? {
            return match self.unary()? {
                Some(right) => Ok(Some(Expr::unary(operator, right))),
                None => Ok(None),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Option<Expr>, E> {
        if let Some(token) = self.tokens.next() {
            use TokenType::*;
            match token.token_type {
                False | True | Nil | Number | String => return Ok(Some(Expr::literal(token))),
                LeftParenthesis => {
                    let expr = self.expression()?;
                    match expr {
                        Some(expr) => {
                            if let Some(_) = self.peek_matches(RightParenthesis)? {
                                return Ok(Some(Expr::grouping(expr)));
                            } else {
                                self.error("Expect ')' after expression.".to_string());
                                Ok(None)
                            }
                        }
                        None => Ok(None),
                    }
                }
                _ => {
                    self.error("".to_string());
                    Ok(None)
                }
            }
        } else {
            Ok(Some(Expr::literal(Token::new(TokenType::EOF, "", 0))))
        }
    }

    fn next(&mut self) -> Result<Option<Token>, E> {
        match self.peeked {
            Some(_) => Ok(self.peeked.take()),
            None => match self.tokens.next() {
                Some(token) => Ok(Some(token)),
                None => match self.tokens.error.take() {
                    Some(e) => Err(e),
                    None => Ok(None),
                },
            },
        }
    }

    fn peek_matches<M: TokenTypeMatcher>(&mut self, matcher: M) -> Result<Option<Token>, E> {
        match &self.peeked {
            Some(token) if matcher.matches(&token.token_type) => Ok(self.peeked.take()),
            Some(_) => Ok(None),
            None => {
                self.peeked = self.tokens.next();
                self.peek_matches(matcher)
            }
        }
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

#[derive(Debug)]
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
