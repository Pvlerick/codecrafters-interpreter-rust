use std::{cell::RefCell, error::Error, fmt::Display, io::BufRead, rc::Rc};

use crate::{
    errors::TokenError,
    scanner::{Scanner, Token, TokenType, TokensIterator},
    utils::StopOnFirstErrorIterator,
};

/* Grammar:


program        → statement* EOF ;
statement      → exprStmt | printStmt ;
exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;
expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;

*/

pub struct Parser {
    tokens: Option<TokensIterator>,
    errors: Rc<RefCell<Option<Vec<String>>>>,
}

macro_rules! gramar_rule {
    ($name:ident, $base:ident, $token_types:expr) => {
        fn $name(&mut self) -> Result<Option<Expr>, TokenError> {
            let mut ret_expr: Expr;

            match self.$base()? {
                Some(expr) => {
                    ret_expr = expr;
                    while let Some(operator) = self.peek_matches($token_types)? {
                        match self.$base()? {
                            Some(right) => ret_expr = Expr::binary(operator, ret_expr, right),
                            None => return Ok(None),
                        }
                    }

                    Ok(Some(ret_expr))
                }
                None => Ok(None),
            }
        }
    };
}

impl Parser {
    pub fn new(tokens: TokensIterator) -> Self {
        Self {
            tokens: Some(tokens),
            errors: Rc::new(RefCell::new(None)),
        }
    }

    pub fn build<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        let scanner = Scanner::new(reader);
        let tokens = scanner.scan()?;

        Ok(Self::new(tokens))
    }

    pub fn parse(&mut self) -> Result<StatementsIterator, Box<dyn Error>> {
        match self.tokens.take() {
            Some(tokens) => Ok(StatementsIterator::new(tokens, self.errors.clone())),
            None => Err("Parser's tokens have already been consumed".into()),
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expr, Box<dyn Error>> {
        todo!()
    }

    pub fn errors(&self) -> Option<Vec<String>> {
        match self.tokens {
            None => self.errors.take(),
            Some(_) => None, // Parsing didn't occur yet
        }
    }
}

pub struct StatementsIterator {
    tokens: StopOnFirstErrorIterator<TokensIterator, Token, TokenError>,
    peeked: Option<Token>,
    errors: Rc<RefCell<Option<Vec<String>>>>,
}

impl Iterator for StatementsIterator {
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        match self.statement() {
            Ok(item) => item,
            Err(error) => {
                self.errors
                    .borrow_mut()
                    .get_or_insert_with(|| Vec::new())
                    .push(format!("{}", error));
                None
            }
        }
    }
}

impl StatementsIterator {
    pub fn new(tokens: TokensIterator, errors: Rc<RefCell<Option<Vec<String>>>>) -> Self {
        Self {
            tokens: StopOnFirstErrorIterator::new(tokens),
            peeked: None,
            errors,
        }
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

    fn error(&mut self, msg: String) {
        self.errors
            .borrow_mut()
            .get_or_insert_with(|| Vec::new())
            .push(msg);
    }

    pub fn statement(&mut self) -> Result<Option<Statement>, TokenError> {
        match self.next_token() {
            Ok(Some(token)) => match token.token_type {
                TokenType::EOF => Ok(None),
                TokenType::Print => self.print_statement(),
                _ => self.expression_statement(),
            },
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn print_statement(&mut self) -> Result<Option<Statement>, TokenError> {
        match self.expression() {
            Ok(Some(expr)) => {
                self.consume_semicolon()?;
                Ok(Some(Statement::Print(expr)))
            }
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>, TokenError> {
        match self.expression() {
            Ok(Some(expr)) => {
                self.consume_semicolon()?;
                Ok(Some(Statement::Expression(expr)))
            }
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn expression(&mut self) -> Result<Option<Expr>, TokenError> {
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

    fn unary(&mut self) -> Result<Option<Expr>, TokenError> {
        use TokenType::*;
        if let Some(operator) = self.peek_matches([Bang, Minus])? {
            return match self.unary()? {
                Some(right) => Ok(Some(Expr::unary(operator, right))),
                None => Ok(None),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Option<Expr>, TokenError> {
        match self.next_token() {
            Ok(Some(token)) => {
                use TokenType::*;
                match token.token_type {
                    False | True | Nil | Number | String => {
                        return Ok(Some(Expr::literal(token)));
                    }
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
                    token_type => {
                        self.error(format!("Unexpected token: {}", token_type));
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(Some(Expr::literal(Token::new(TokenType::EOF, "", 0)))),
            Err(error) => Err(error),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, TokenError> {
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

    fn peek_matches<M: TokenTypeMatcher>(
        &mut self,
        matcher: M,
    ) -> Result<Option<Token>, TokenError> {
        match &self.peeked {
            Some(token) if matcher.matches(&token.token_type) => Ok(self.peeked.take()),
            Some(_) => Ok(None),
            None => {
                self.peeked = self.tokens.next();
                self.peek_matches(matcher)
            }
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), TokenError> {
        match self.peek_matches(TokenType::Semicolon) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err("Expect ';' after expression.".to_owned().into()),
            Err(error) => Err(error),
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
pub enum Statement {
    Print(Expr),
    Expression(Expr),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            Print(expr) => write!(f, "print {}", expr),
            Expression(expr) => write!(f, "{}", expr),
        }
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
