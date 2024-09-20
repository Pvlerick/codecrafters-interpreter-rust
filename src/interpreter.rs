use std::{
    error::Error,
    fmt::Display,
    io::{BufRead, Write},
    rc::Rc,
};

use crate::{
    environment::Environment,
    parser::{Declaration, Expr, Parser, Statement},
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    parser: Option<Parser>,
    environment: Environment<Type>,
    pub has_parsing_errors: bool,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Self {
            parser: Some(parser),
            environment: Environment::new(),
            has_parsing_errors: false,
        }
    }

    pub fn build<R>(reader: R) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        let parser = Parser::build(reader)?;

        Ok(Interpreter::new(parser))
    }

    pub fn evaluate<T, U>(
        &mut self,
        output: &mut T,
        err_output: &mut U,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Write,
        U: Write,
    {
        match self.parser.take() {
            Some(mut parser) => {
                match parser.parse_expression()? {
                    Some(expr) => {
                        let result = self.eval(&expr)?;
                        write!(output, "{}", result)?;
                    }
                    _ => {}
                }

                if let Some(errors) = parser.errors() {
                    self.has_parsing_errors = true;
                    write!(err_output, "{}", errors)?
                }

                Ok(())
            }
            None => Err("Interpreter's statements have already been consumed".into()),
        }
    }

    pub fn run<T, U>(&mut self, output: &mut T, err_output: &mut U) -> Result<(), Box<dyn Error>>
    where
        T: Write,
        U: Write,
    {
        match self.parser.take() {
            Some(mut parser) => {
                for declaration in parser.parse()? {
                    let _ = self.execute(&declaration, output)?;
                }

                if let Some(errors) = parser.errors() {
                    self.has_parsing_errors = true;
                    write!(err_output, "{}", errors)?
                }

                Ok(())
            }
            None => Err("Interpreter's statements have already been consumed".into()),
        }
    }

    fn execute<T>(
        &mut self,
        declaration: &Declaration,
        output: &mut T,
    ) -> Result<Option<Type>, Box<dyn Error>>
    where
        T: Write,
    {
        match declaration {
            Declaration::Variable(name, Some(expression)) => {
                let name = name.to_owned();
                let value = self.eval(expression)?;
                self.environment.set(name, value);
                Ok(None)
            }
            Declaration::Variable(name, None) => {
                self.environment.set(name.to_owned(), Type::Nil);
                Ok(None)
            }
            Declaration::Statement(Statement::Print(expr)) => {
                writeln!(output, "{}", self.eval(expr)?)?;
                Ok(None)
            }
            Declaration::Statement(Statement::Expression(expr)) => Ok(Some(self.eval(&expr)?)),
        }
    }

    fn eval(&mut self, expression: &Expr) -> Result<Type, Box<dyn Error>> {
        match expression {
            Expr::Literal(token) => match token.token_type {
                TokenType::True => Ok(Type::Boolean(true)),
                TokenType::False => Ok(Type::Boolean(false)),
                TokenType::Nil => Ok(Type::Nil),
                _ if token.literal.is_some() => Ok(token.literal.as_ref().unwrap().as_ref().into()),
                _ => panic!("oh no..."),
            },
            Expr::Grouping(e) => self.eval(e),
            Expr::Unary(t, e) => match t.token_type {
                TokenType::Minus => match self.eval(e)? {
                    Type::Number(n) => Ok(Type::Number(-n)),
                    _ => Err("Operand must be a number.".into()),
                },
                TokenType::Bang => Ok(Type::Boolean(!Interpreter::is_truthy(self.eval(e)?))),
                _ => panic!("oh no..."),
            },
            Expr::Binary(t, l, r) => match (t.token_type, self.eval(l)?, self.eval(r)?) {
                (TokenType::Plus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a + b)),
                (TokenType::Plus, Type::String(a), Type::String(b)) => {
                    Ok(Type::String(Rc::new(format!("{}{}", a, b))))
                }
                (TokenType::Plus, _, _) => {
                    Err("Operands must be two numbers or two strings.".into())
                }
                (TokenType::Minus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a - b)),
                (TokenType::Minus, _, _) => {
                    Err("Operands must be two numbers or two strings.".into())
                }
                (TokenType::Slash, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a / b)),
                (TokenType::Slash, _, _) => Err("Operands must be numbers.".into()),
                (TokenType::Star, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a * b)),
                (TokenType::Greater, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a > b)),
                (TokenType::GreaterEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a >= b))
                }
                (TokenType::Less, Type::Number(a), Type::Number(b)) => Ok(Type::Boolean(a < b)),
                (TokenType::LessEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a <= b))
                }
                (TokenType::EqualEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a == b))
                }
                (TokenType::BangEqual, Type::Number(a), Type::Number(b)) => {
                    Ok(Type::Boolean(a != b))
                }
                (TokenType::EqualEqual, Type::Boolean(a), Type::Boolean(b)) => {
                    Ok(Type::Boolean(a == b))
                }
                (TokenType::BangEqual, Type::Boolean(a), Type::Boolean(b)) => {
                    Ok(Type::Boolean(a != b))
                }
                (TokenType::EqualEqual, Type::String(a), Type::String(b)) => {
                    Ok(Type::Boolean(a == b))
                }
                (TokenType::BangEqual, Type::String(a), Type::String(b)) => {
                    Ok(Type::Boolean(a != b))
                }
                (TokenType::EqualEqual, _, _) => Ok(Type::Boolean(false)),
                (TokenType::BangEqual, _, _) => Ok(Type::Boolean(false)),
                _ => Err("Unrecognized binary expression".into()),
            },
            Expr::Variable(token) => match self.environment.get(&token.lexeme) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Undefined variable '{}'.", token.lexeme).into()),
            },
            Expr::Assignment(token, expr) => {
                let name = token.lexeme.to_owned();
                let value = self.eval(expr)?;
                self.environment.set(name, value.clone());
                Ok(value)
            }
        }
    }

    fn is_truthy(t: Type) -> bool {
        match t {
            Type::Nil => false,
            Type::Boolean(b) => b,
            _ => true,
        }
    }
}

#[derive(Clone)]
enum Type {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
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

impl From<&Literal> for Type {
    fn from(value: &Literal) -> Self {
        match value {
            Literal::Digit(n) => Type::Number(*n),
            Literal::String(s) => Type::String(s.clone()),
        }
    }
}
