use std::{cell::RefCell, fmt::Display, io::BufRead, rc::Rc};

use crate::{
    errors::{InterpreterError, ParsingErrorsBuilder},
    scanner::{Scanner, Token, TokenType, TokensIterator},
};

/* Grammar:

program        → declaration* EOF ;
declaration    → funcDecl | varDecl | statement ;
funcDecl       → "fun" function ;
function       → IDENTIFIER "(" parameters? ")" block ;
parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
statement      → exprStmt | ifStmt | whileStmt | forStmt | returnStmt | printStmt | block ;
exprStmt       → expression ";" ;
ifStmt         → "if" "(" expression ")" statement ( "else" statement )? ;
whileStmt      → "while" "(" expression ")" statement ;
forStmt        → "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
returnStmt     → "return" expression? ";" ;
printStmt      → "print" expression ";" ;
block          → "{" declaration* "}" ;
expression     → assignment ;
assignment     → IDENTIFIER "=" assignment | logic_or ;
logic_or       → logic_and ( "or" logic_and )* ;
logic_or       → equality ( "and" equality )* ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary | call ;
call           → primary ( "(" arguments? ")" )* ;
arguments      → expression ( "," expression )* ;
primary        → NUMBER | STRING | "true" | "false" | "nil" | IDENTIFIER | "(" expression ")" ;

*/

pub struct Parser {
    tokens: Option<TokensIterator>,
    errors: Rc<RefCell<Option<ParsingErrorsBuilder>>>,
}

macro_rules! grammar_rule_binary {
    ($name:ident, $base:ident, $token_types:expr, $expr:ident) => {
        fn $name(&mut self) -> Result<Option<Expr>, ()> {
            let mut ret_expr: Expr;

            match self.$base()? {
                Some(expr) => {
                    ret_expr = expr;
                    while let Some(operator) = self.next_matches($token_types)? {
                        match self.$base()? {
                            Some(right) => ret_expr = Expr::$expr(operator, ret_expr, right),
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

    pub fn build<R>(reader: R) -> Result<Self, InterpreterError>
    where
        R: BufRead + 'static,
    {
        let scanner = Scanner::new(reader);
        let tokens = scanner.scan()?;

        Ok(Self::new(tokens))
    }

    pub fn parse(&mut self) -> Result<StatementsIterator, InterpreterError> {
        match self.tokens.take() {
            Some(tokens) => Ok(StatementsIterator::new(tokens, self.errors.clone())),
            None => Err(InterpreterError::parsing(
                "Parser's tokens have already been consumed",
            )),
        }
    }

    pub fn parse_expression(&mut self) -> Result<Option<Expr>, InterpreterError> {
        match self.tokens.take() {
            Some(tokens) => {
                match StatementsIterator::new(tokens, self.errors.clone()).next_expression() {
                    Some(expr) => Ok(Some(expr)),
                    None => Err(InterpreterError::parsing("No expression found")),
                }
            }
            None => Err(InterpreterError::parsing(
                "Parser's tokens have already been consumed",
            )),
        }
    }

    pub fn errors(&self) -> Option<InterpreterError> {
        match self.tokens {
            None => self.errors.take().map(|i| i.build()),
            Some(_) => None, // Parsing didn't occur yet
        }
    }
}

pub struct StatementsIterator {
    tokens: TokensIterator,
    peeked: Option<Token>,
    errors: Rc<RefCell<Option<ParsingErrorsBuilder>>>,
}

impl Iterator for StatementsIterator {
    type Item = Statement;

    fn next(&mut self) -> Option<Self::Item> {
        match self.declaration() {
            Ok(item) => item,
            Err(_) => match self.syncronize() {
                Ok(_) => self.next(),
                Err(_) => None,
            },
        }
    }
}

impl StatementsIterator {
    pub fn new(tokens: TokensIterator, errors: Rc<RefCell<Option<ParsingErrorsBuilder>>>) -> Self {
        Self {
            tokens,
            peeked: None,
            errors,
        }
    }

    fn next_expression(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(Some(expr)) => Some(expr),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    fn add_error<T: ToString, R>(&mut self, msg: T) -> Result<R, ()> {
        self.errors
            .borrow_mut()
            .get_or_insert_with(|| ParsingErrorsBuilder::new())
            .add(msg.to_string(), self.tokens.current_line());
        Err(())
    }

    fn syncronize(&mut self) -> Result<(), ()> {
        use TokenType::*;
        while let Some(token) = self.next_token()? {
            if token.token_type == Semicolon
                || matches!(
                    self.peek()?.map(|i| i.token_type),
                    Some(Class)
                        | Some(For)
                        | Some(Fun)
                        | Some(If)
                        | Some(Print)
                        | Some(Return)
                        | Some(Var)
                        | Some(While)
                )
            {
                return Ok(());
            }
        }

        Ok(())
    }

    fn declaration(&mut self) -> Result<Option<Statement>, ()> {
        match self.peek()? {
            Some(token) => match token.token_type {
                TokenType::EOF => Ok(None),
                TokenType::Fun => self.function(FunctionKind::Function),
                TokenType::Var => self.variable_declaration(),
                _ => Ok(self.statement()?.map(|i| i.into())),
            },
            None => Ok(None),
        }
    }

    fn function(&mut self, kind: FunctionKind) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::Fun, "Expect 'fun' in a function delcaration")?;

        let name = self.consume(TokenType::Identifier, format!("Expect {} name", kind))?;

        self.consume(
            TokenType::LeftParenthesis,
            format!("Expect '(' after {} name", kind),
        )?;

        let mut parameters = Vec::new();
        if !(self.peek_type(TokenType::RightParenthesis)?) {
            loop {
                if parameters.len() >= 255 {
                    return self.add_error("Can't have more than 255 parameters.");
                }
                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name")?);
                if self.next_matches(TokenType::Comma)?.is_none() {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParenthesis,
            format!("Expect ')' after parameters list"),
        )?;

        match self.block()? {
            Some(body) => Ok(Some(Statement::Function(
                name.lexeme,
                Box::new(parameters),
                Rc::new(body),
            ))),
            None => Ok(None),
        }
    }

    fn variable_declaration(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::Var, "Expect 'var' in variable declaration")?;
        match self.next_token()? {
            Some(token) if token.token_type == TokenType::Identifier => {
                let name = token.lexeme;
                let initializer = match self.peek()? {
                    Some(token) if token.token_type == TokenType::Equal => {
                        self.consume(TokenType::Equal, "Expect '=' after variable identifier")?;
                        self.expression()?
                    }
                    _ => None,
                };
                self.consume_semicolon()?;
                Ok(Some(Statement::Variable(name, initializer)))
            }
            _ => self.add_error("Expect variable name"),
        }
    }

    fn statement(&mut self) -> Result<Option<Statement>, ()> {
        match self.peek()? {
            Some(token) => match token.token_type {
                TokenType::If => self.if_statement(),
                TokenType::While => self.while_statement(),
                TokenType::For => self.for_statement(),
                TokenType::LeftBrace => self.block(),
                TokenType::Print => self.print_statement(),
                TokenType::Return => self.return_statement(),
                _ => self.expression_statement(),
            },
            None => Ok(None),
        }
    }

    fn if_statement(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::If, "Expect 'if' in if statement")?;
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParenthesis, "Expect ')' after if condition")?;
        let then_branch = self.statement()?;
        let else_branch = if self.next_matches(TokenType::Else)?.is_some() {
            self.statement()?
        } else {
            None
        };

        match (condition, then_branch) {
            (Some(condition), Some(then_branch)) => Ok(Some(Statement::If(
                condition,
                Box::new(then_branch),
                else_branch.map(|i| Box::new(i)),
            ))),
            _ => Ok(None),
        }
    }

    fn while_statement(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::While, "Expect 'while' in while statement")?;
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'while'")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParenthesis, "Expect ')' after condition")?;

        match (condition, self.statement()?) {
            (Some(condition), Some(body)) => Ok(Some(Statement::While(condition, Box::new(body)))),
            _ => Ok(None),
        }
    }

    fn for_statement(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::For, "Expect 'for' in for statement")?;
        self.consume(TokenType::LeftParenthesis, "Expect '(' after 'for'")?;

        let initializer = if self.next_matches(TokenType::Semicolon)?.is_some() {
            None
        } else if self.peek_type(TokenType::Var)? {
            self.variable_declaration()?
        } else {
            self.expression_statement()?
        };

        let condition = if self.peek_type(TokenType::Semicolon)? {
            None
        } else {
            self.expression()?
        };
        self.consume_semicolon()?;

        let increment = if self.peek_type(TokenType::RightParenthesis)? {
            None
        } else {
            self.expression()?
        };

        self.consume(TokenType::RightParenthesis, "Expect ')' after for clauses")?;

        let mut body = match self.statement()? {
            Some(statement) => statement,
            None => {
                return self.add_error("for statement needs to have a body");
            }
        };

        if let Some(increment) = increment {
            body = Statement::Block(Box::new(vec![body, Statement::Expression(increment)]))
        }

        body = Statement::While(
            if let Some(condition) = condition {
                condition
            } else {
                Expr::Literal(Token::new(
                    TokenType::True,
                    "true",
                    self.tokens.current_line(),
                ))
            },
            Box::new(body),
        );

        if let Some(initializer) = initializer {
            body = Statement::Block(Box::new(vec![initializer, body]));
        }

        return Ok(Some(body));
    }

    fn block(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::LeftBrace, "Expect '{' to start a block")?;
        let mut declarations = Vec::new();
        while self.peek()?.is_some() && self.next_matches(TokenType::RightBrace)?.is_none() {
            match self.declaration()? {
                Some(declaration) => declarations.push(declaration),
                None => return self.add_error("Expect '}' after block"),
            }
        }

        Ok(Some(Statement::Block(Box::new(declarations))))
    }

    fn print_statement(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::Print, "Expect 'print' in print statement")?;
        match self.expression()? {
            Some(expr) => {
                self.consume_semicolon()?;
                Ok(Some(Statement::Print(expr)))
            }
            None => Ok(None),
        }
    }

    fn return_statement(&mut self) -> Result<Option<Statement>, ()> {
        self.consume(TokenType::Return, "Expect 'return', in return statement")?;

        let expr = if !self.peek_type(TokenType::Semicolon)? {
            self.expression()?
        } else {
            None
        };

        self.consume_semicolon()?;

        Ok(Some(Statement::Return(expr)))
    }

    fn expression_statement(&mut self) -> Result<Option<Statement>, ()> {
        match self.expression()? {
            Some(expr) => {
                self.consume_semicolon()?;
                Ok(Some(Statement::Expression(expr)))
            }
            None => Ok(None),
        }
    }

    fn expression(&mut self) -> Result<Option<Expr>, ()> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Option<Expr>, ()> {
        let expr = self.logic_or()?;

        if self.next_matches(TokenType::Equal)?.is_some() {
            return match (self.assignment()?, expr) {
                (Some(value), Some(Expr::Variable(token))) => {
                    Ok(Some(Expr::assignment(token, value)))
                }
                _ => {
                    let _ = self.add_error::<_, ()>("Invalid assignment target");
                    Ok(None)
                }
            };
        }

        Ok(expr)
    }

    grammar_rule_binary!(logic_or, logic_and, TokenType::Or, logical);
    grammar_rule_binary!(logic_and, equality, TokenType::And, logical);
    grammar_rule_binary!(
        equality,
        comparison,
        [TokenType::BangEqual, TokenType::EqualEqual],
        binary
    );
    grammar_rule_binary!(
        comparison,
        term,
        [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ],
        binary
    );
    grammar_rule_binary!(term, factor, [TokenType::Minus, TokenType::Plus], binary);
    grammar_rule_binary!(factor, unary, [TokenType::Slash, TokenType::Star], binary);

    fn unary(&mut self) -> Result<Option<Expr>, ()> {
        use TokenType::*;
        if let Some(operator) = self.next_matches([Bang, Minus])? {
            return match self.unary()? {
                Some(right) => Ok(Some(Expr::unary(operator, right))),
                None => Ok(None),
            };
        }

        self.call()
    }

    fn call(&mut self) -> Result<Option<Expr>, ()> {
        let mut expr = self.primary()?;

        loop {
            if self.next_matches(TokenType::LeftParenthesis)?.is_some() {
                if let Some(callee) = expr {
                    expr = self.finish_call(callee)?;
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Option<Expr>, ()> {
        let mut arguments = Vec::new();

        if !(self.peek_type(TokenType::RightParenthesis)?) {
            loop {
                if arguments.len() >= 255 {
                    return self.add_error("Can't have more than 255 arguments.");
                }
                if let Some(expr) = self.expression()? {
                    arguments.push(expr);
                }
                if self.next_matches(TokenType::Comma)?.is_none() {
                    break;
                }
            }
        }

        let right_paren =
            self.consume(TokenType::RightParenthesis, "Expect ')' after arguments")?;

        Ok(Some(Expr::call(callee, right_paren, arguments)))
    }

    fn primary(&mut self) -> Result<Option<Expr>, ()> {
        match self.next_token() {
            Ok(Some(token)) => {
                use TokenType::*;
                match token.token_type {
                    Identifier => Ok(Some(Expr::variable(token))),
                    False | True | Nil | Number | String => Ok(Some(Expr::literal(token))),
                    LeftParenthesis => match self.expression()? {
                        Some(expr) => {
                            if self.next_matches(RightParenthesis)?.is_some() {
                                return Ok(Some(Expr::grouping(expr)));
                            } else {
                                self.add_error("Expect ')' after expression")?;
                                Ok(None)
                            }
                        }
                        None => Ok(None),
                    },
                    token_type => {
                        self.add_error(format!("Unexpected token: {}", token_type))?;
                        Ok(None)
                    }
                }
            }
            Ok(_) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, ()> {
        match self.peeked {
            Some(_) => Ok(self.peeked.take()),
            None => match self.tokens.next() {
                Some(Ok(token)) => Ok(Some(token)),
                None => Ok(None),
                Some(Err(error)) => self.add_error(error),
            },
        }
    }

    fn peek(&mut self) -> Result<Option<&Token>, ()> {
        match self.peeked {
            Some(_) => Ok(self.peeked.as_ref()),
            None => match self.tokens.next() {
                Some(Ok(token)) => {
                    self.peeked = Some(token);
                    Ok(self.peeked.as_ref())
                }
                None => Ok(None),
                Some(Err(error)) => self.add_error(error),
            },
        }
    }

    fn peek_type(&mut self, token_type: TokenType) -> Result<bool, ()> {
        Ok(self.peek()?.map_or(false, |i| i.token_type == token_type))
    }

    fn next_matches<M: TokenTypeMatcher>(&mut self, matcher: M) -> Result<Option<Token>, ()> {
        match &self.peeked {
            Some(token) if matcher.matches(&token.token_type) => Ok(self.peeked.take()),
            Some(_) => Ok(None),
            None => match self.tokens.next() {
                Some(Ok(token)) => {
                    self.peeked = Some(token);
                    self.next_matches(matcher)
                }
                None => Ok(None),
                Some(Err(error)) => self.add_error(error),
            },
        }
    }

    fn consume_semicolon(&mut self) -> Result<Token, ()> {
        self.consume(TokenType::Semicolon, "Expect ';' after expression")
    }

    fn consume<S: ToString>(
        &mut self,
        token_type: TokenType,
        error_message: S,
    ) -> Result<Token, ()> {
        match self.next_matches(token_type)? {
            Some(token) => Ok(token),
            None => self.add_error(error_message),
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
    Function(String, Box<Vec<Token>>, Rc<Statement>),
    Variable(String, Option<Expr>),
    Print(Expr),
    Return(Option<Expr>),
    Expression(Expr),
    Block(Box<Vec<Statement>>),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    While(Expr, Box<Statement>),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            Return(None) => write!(f, "return"),
            Return(Some(expr)) => write!(f, "return {}", expr),
            Function(name, params, body) => write!(
                f,
                "fun {}({}) {}",
                name,
                params
                    .iter()
                    .map(|i| i.lexeme.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                body
            ),
            Variable(name, None) => write!(f, "var {}", name),
            Variable(name, Some(expr)) => write!(f, "var {}={}", name, expr),
            Print(expr) => write!(f, "print {}", expr),
            Expression(expr) => write!(f, "{}", expr),
            Block(statements) => {
                writeln!(f, "{{")?;
                for statement in statements.iter() {
                    writeln!(f, "{}", statement)?;
                }
                writeln!(f, "}}")
            }
            If(condition, then_branch, None) => write!(f, "if {} then {}", condition, then_branch),
            If(condition, then_branch, Some(else_branch)) => {
                write!(
                    f,
                    "if {} then {} else {}",
                    condition, then_branch, else_branch
                )
            }
            While(condition, body) => write!(f, "while {} then {}", condition, body),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary(Token, Box<Expr>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token),
    Logical(Token, Box<Expr>, Box<Expr>),
    Unary(Token, Box<Expr>),
    Variable(Token),
    Assignment(Token, Box<Expr>),
    Call(Box<Expr>, Token, Box<Vec<Expr>>),
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
    fn logical(token: Token, left: Expr, right: Expr) -> Self {
        Self::Logical(token, Box::new(left), Box::new(right))
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

    fn call(callee: Expr, right_paren: Token, arguments: Vec<Expr>) -> Self {
        Self::Call(Box::new(callee), right_paren, Box::new(arguments))
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;
        match self {
            Binary(token, left, right) | Logical(token, left, right) => {
                write!(f, "({} {} {})", token.display(), left, right,)
            }
            Grouping(expr) => write!(f, "(group {})", expr),
            Literal(token) => write!(f, "{}", token.display()),
            Unary(token, expr) => write!(f, "({} {})", token.display(), expr),
            Variable(token) => write!(f, "(var \"{}\")", token.display()),
            Assignment(name, expr) => write!(f, "(assignment {}={})", name, expr),
            Call(callee, _, arguments) => {
                write!(
                    f,
                    "{}({})",
                    callee,
                    arguments
                        .iter()
                        .map(|i| format!("{}", i))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
    }
}

enum FunctionKind {
    Function,
}

impl Display for FunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionKind::Function => write!(f, "function"),
        }
    }
}
