use std::{cell::RefCell, error::Error, fmt::Display, io::BufRead, rc::Rc};

use crate::{
    errors::{ParsingErrors, TokenError},
    scanner::{Scanner, Token, TokenType, TokensIterator},
    utils::StopOnFirstErrorIterator,
};

/* Grammar:

program        → declaration* EOF ;
declaration    → varDecl | statement ;
varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
statement      → exprStmt | printStmt | block ;
exprStmt       → expression ";" ;
printStmt      → "print" expression ";" ;
block          → "{" declaration* "}" ;
expression     → assignment ;
assignment     → IDENTIFIER "=" assignment | equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil" | IDENTIFIER | "(" expression ")" ;

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

    pub fn parse(&mut self) -> Result<DeclarationsIterator, Box<dyn Error>> {
        match self.tokens.take() {
            Some(tokens) => Ok(DeclarationsIterator::new(tokens, self.errors.clone())),
            None => Err("Parser's tokens have already been consumed".into()),
        }
    }

    pub fn parse_expression(&mut self) -> Result<Option<Expr>, Box<dyn Error>> {
        match self.tokens.take() {
            Some(tokens) => {
                match DeclarationsIterator::new(tokens, self.errors.clone()).next_expression() {
                    Some(expr) => Ok(Some(expr)),
                    None => Err("No expression found.".into()),
                }
            }
            None => Err("Parser's tokens have already been consumed".into()),
        }
    }

    pub fn errors(&self) -> Option<ParsingErrors> {
        match self.tokens {
            None => self.errors.take().map(|i| i.into()),
            Some(_) => None, // Parsing didn't occur yet
        }
    }
}

pub struct DeclarationsIterator {
    tokens: StopOnFirstErrorIterator<TokensIterator, Token, TokenError>,
    peeked: Option<Token>,
    errors: Rc<RefCell<Option<Vec<String>>>>,
}

impl Iterator for DeclarationsIterator {
    type Item = Declaration;

    fn next(&mut self) -> Option<Self::Item> {
        match self.declaration() {
            Ok(item) => item,
            Err(error) => {
                self.add_error(error.message);
                None
            }
        }
    }
}

impl DeclarationsIterator {
    pub fn new(tokens: TokensIterator, errors: Rc<RefCell<Option<Vec<String>>>>) -> Self {
        Self {
            tokens: StopOnFirstErrorIterator::new(tokens),
            peeked: None,
            errors,
        }
    }

    fn next_expression(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(Some(expr)) => Some(expr),
            Ok(None) => None,
            Err(error) => {
                self.add_error(error.message);
                None
            }
        }
    }

    fn add_error<T>(&mut self, msg: T)
    where
        T: Display,
    {
        self.errors
            .borrow_mut()
            .get_or_insert_with(|| Vec::new())
            .push(format!(
                "[line {}] Error: {}.",
                self.tokens.inner.current_line(),
                msg
            ));
    }

    fn declaration(&mut self) -> Result<Option<Declaration>, TokenError> {
        match self.peek()? {
            Some(token) => match token.token_type {
                TokenType::EOF => Ok(None),
                TokenType::Var => self.variable_declaration(),
                _ => self.statement().map(|i| i.map(|j| j.into())),
            },
            None => Ok(None),
        }
    }

    fn variable_declaration(&mut self) -> Result<Option<Declaration>, TokenError> {
        let _ = self.next_token()?; // Discard var token
        match self.next_token()? {
            Some(token) if token.token_type == TokenType::Identifier => {
                let name = token.lexeme;
                let initializer = match self.peek()? {
                    Some(token) if token.token_type == TokenType::Equal => {
                        let _ = self.next_token(); // Discard = token
                        self.expression()?
                    }
                    _ => None,
                };
                self.consume_semicolon()?;
                Ok(Some(Declaration::Variable(name, initializer)))
            }
            _ => {
                self.add_error("Expect variable name");
                Ok(None)
            }
        }
    }

    fn statement(&mut self) -> Result<Option<Statement>, TokenError> {
        match self.peek()? {
            Some(token) => match token.token_type {
                TokenType::LeftBrace => {
                    let _ = self.next_token()?; // Discard left brace
                    self.block()
                }
                TokenType::Print => {
                    let _ = self.next_token()?; // Discard first tokens as it's "print"
                    self.print_statement()
                }
                _ => self.expression_statement(),
            },
            None => Ok(None),
        }
    }

    fn block(&mut self) -> Result<Option<Statement>, TokenError> {
        let mut declarations = Vec::new();
        while self.peek()?.is_some() && self.peek_matches(TokenType::RightBrace)?.is_none() {
            match self.declaration()? {
                Some(declaration) => declarations.push(declaration),
                None => {
                    let token_type = self.peek()?.unwrap().token_type;
                    self.add_error(format!("Unexpected token: {}", token_type));
                    return Ok(None);
                }
            }
        }

        Ok(Some(Statement::Block(Box::new(declarations))))
    }

    fn print_statement(&mut self) -> Result<Option<Statement>, TokenError> {
        match self.expression()? {
            Some(expr) => {
                self.consume_semicolon()?;
                Ok(Some(Statement::Print(expr)))
            }
            None => Ok(None),
        }
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>, TokenError> {
        match self.expression()? {
            Some(expr) => {
                self.consume_semicolon()?;
                Ok(Some(Statement::Expression(expr)))
            }
            None => Ok(None),
        }
    }

    fn expression(&mut self) -> Result<Option<Expr>, TokenError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Option<Expr>, TokenError> {
        let expr = self.equality()?;

        if self.peek_matches(TokenType::Equal)?.is_some() {
            return match (self.assignment()?, expr) {
                (Some(value), Some(Expr::Variable(token))) => {
                    Ok(Some(Expr::assignment(token, value)))
                }
                _ => {
                    self.add_error("Invalid assignment target");
                    Ok(None)
                }
            };
        }

        Ok(expr)
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
                    Identifier => Ok(Some(Expr::variable(token))),
                    False | True | Nil | Number | String => Ok(Some(Expr::literal(token))),
                    LeftParenthesis => match self.expression()? {
                        Some(expr) => {
                            if self.peek_matches(RightParenthesis)?.is_some() {
                                return Ok(Some(Expr::grouping(expr)));
                            } else {
                                self.add_error("Expect ')' after expression");
                                Ok(None)
                            }
                        }
                        None => Ok(None),
                    },
                    token_type => {
                        self.add_error(format!("Unexpected token: {}", token_type));
                        Ok(None)
                    }
                }
            }
            Ok(_) => Ok(None),
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

    fn peek(&mut self) -> Result<Option<&Token>, TokenError> {
        match self.peeked {
            Some(_) => Ok(self.peeked.as_ref()),
            None => match self.tokens.next() {
                Some(token) => {
                    self.peeked = Some(token);
                    Ok(self.peeked.as_ref())
                }
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
            None => match self.tokens.next() {
                Some(token) => {
                    self.peeked = Some(token);
                    self.peek_matches(matcher)
                }
                None => Ok(None),
            },
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), TokenError> {
        match self.peek_matches(TokenType::Semicolon) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => TokenError::new(
                "Expect ';' after expression",
                self.tokens.inner.current_line(),
            )
            .into(),
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
pub enum Declaration {
    Variable(String, Option<Expr>),
    Statement(Statement),
}

impl Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Declaration::*;
        match self {
            Variable(name, Some(expr)) => write!(f, "var {}={}", name, expr),
            Variable(name, None) => write!(f, "var {}", name),
            Statement(statement) => write!(f, "{}", statement),
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Print(Expr),
    Expression(Expr),
    Block(Box<Vec<Declaration>>),
}

impl Into<Declaration> for Statement {
    fn into(self) -> Declaration {
        Declaration::Statement(self)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            Print(expr) => write!(f, "print {}", expr),
            Expression(expr) => write!(f, "{}", expr),
            Block(statements) => {
                writeln!(f, "{{")?;
                for statement in statements.iter() {
                    writeln!(f, "{}", statement)?;
                }
                writeln!(f, "}}")
            }
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assignment(Token, Box<Expr>),
}

impl Expr {
    fn binary(token: Token, left: Expr, right: Expr) -> Self {
        Self::Binary(token, Box::new(left), Box::new(right))
    }

    fn grouping(expr: Expr) -> Self {
        Self::Grouping(Box::new(expr))
    }

    fn literal(token: Token) -> Self {
        Self::Literal(token)
    }

    fn unary(token: Token, expr: Expr) -> Self {
        Self::Unary(token, Box::new(expr))
    }

    fn variable(token: Token) -> Self {
        Self::Variable(token)
    }

    fn assignment(token: Token, expr: Expr) -> Self {
        Self::Assignment(token, Box::new(expr))
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
            Variable(token) => write!(f, "(var \"{}\")", token.display()),
            Assignment(name, expr) => write!(f, "(assignment {}={})", name, expr),
        }
    }
}
