use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    io::{stdout, BufRead, Write},
    ops::Deref,
    rc::Rc,
};

use crate::{
    environment::Environment,
    errors::{ErrorMessage, InterpreterError},
    parser::{Expr, Parser, Statement},
    resolver::{HashableExpr, Resolver},
    scanner::{Literal, TokenType},
};

pub struct Interpreter {
    parser: Option<Parser>,
    global_environment: Environment<Type>,
    resolve_table: Option<HashMap<HashableExpr, usize>>,
    output: Rc<RefCell<dyn Write>>,
    pub has_parsing_errors: bool,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Self::with_output(parser, Rc::new(RefCell::new(stdout())))
    }

    pub fn with_output(parser: Parser, output: Rc<RefCell<dyn Write>>) -> Self {
        Self {
            parser: Some(parser),
            global_environment: Interpreter::new_global_environment(),
            resolve_table: None,
            output,
            has_parsing_errors: false,
        }
    }

    pub fn build<R>(reader: R, output: Rc<RefCell<dyn Write>>) -> Result<Self, Box<dyn Error>>
    where
        R: BufRead + 'static,
    {
        let parser = Parser::build(reader)?;

        Ok(Interpreter::with_output(parser, output))
    }

    fn new_global_environment() -> Environment<Type> {
        let env = Environment::<Type>::new();
        env.define(
            "clock",
            Type::Function(Rc::new(RefCell::new(native_functions::Clock {}))),
        );
        env.define(
            "env",
            Type::Function(Rc::new(RefCell::new(native_functions::Env {}))),
        );
        env
    }

    pub fn evaluate(&mut self) -> Result<(), InterpreterError> {
        match self.parser.take() {
            Some(mut parser) => {
                match parser.parse_expression()? {
                    Some(expr) => {
                        let result =
                            self.eval(&Interpreter::new_global_environment(), &Rc::new(expr))?;
                        write!(self.output.borrow_mut(), "{}", result)
                            .expect("cannot write to output");
                    }
                    _ => {}
                }

                if let Some(errors) = parser.errors() {
                    return Err(errors);
                }

                Ok(())
            }
            None => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                "Interpreter's statements have already been consumed",
                None,
            ))),
        }
    }

    pub fn run(&mut self) -> Result<(), InterpreterError> {
        let environment = self.global_environment.clone();
        match self.parser.take() {
            Some(mut parser) => {
                let statements = parser.parse()?.collect::<Vec<_>>();
                let mut resolver = Resolver::new();
                resolver.resolve(&statements)?;
                self.resolve_table = Some(resolver.resolve_table);
                for statement in statements {
                    self.execute_statement(&statement, &environment)?;
                }

                if let Some(errors) = parser.errors() {
                    return Err(errors);
                }

                Ok(())
            }
            None => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                "Interpreter's statements have already been consumed",
                None,
            ))),
        }
    }

    fn execute_statement(
        &mut self,
        statement: &Statement,
        environment: &Environment<Type>,
    ) -> Result<StatementResult, InterpreterError> {
        match statement {
            Statement::Class(name, methods_expressions, super_class) => {
                environment.define(&name.lexeme, Type::Nil);

                let mut env = environment.clone();

                let super_class = if let Some(super_class) = super_class {
                    env = environment.enclose();
                    Some(self.eval(environment, super_class)?)
                } else {
                    None
                };

                let mut methods = HashMap::new();
                for method_expression in methods_expressions.iter().filter_map(|i| i.as_ref()) {
                    let func = self.eval(&env, &method_expression)?;
                    match func {
                        Type::Function(ref f) => {
                            methods.insert(f.deref().borrow().name().to_owned(), func.clone());
                        }
                        _ => {
                            return Err(InterpreterError::InterpreterError(ErrorMessage::new(
                                "class can only contain functions",
                                Some(name.line),
                            )))
                        }
                    }
                }

                let class = match super_class {
                    Some(Type::Class(super_class)) => {
                        env.define("super", Type::Class(super_class.clone()));
                        LoxClass::with_superclass(name.lexeme.to_owned(), methods, super_class)
                    }
                    Some(_) => {
                        return Err(InterpreterError::InterpreterError(ErrorMessage::new(
                            "super class must be a class type",
                            Some(name.line),
                        )))
                    }
                    None => LoxClass::new(name.lexeme.to_owned(), methods),
                };

                environment
                    .assign(&name.lexeme, Type::Class(Rc::new(class)))
                    .expect("should never fail");

                Ok(StatementResult::Empty)
            }
            Statement::Return(expr) => Ok(match expr {
                Some(expr) => StatementResult::Return(self.eval(environment, expr)?),
                None => StatementResult::Return(Type::Nil),
            }),
            Statement::Variable(token, Some(expr)) => {
                let value = self.eval(environment, expr)?;
                environment.define(&token.lexeme, value);
                Ok(StatementResult::Empty)
            }
            Statement::Variable(token, None) => {
                environment.define(&token.lexeme, Type::Nil);
                Ok(StatementResult::Empty)
            }
            Statement::Print(expr) => {
                let res = self.eval(environment, &expr)?;
                writeln!(self.output.borrow_mut(), "{}", res).expect("cannot write to output");
                Ok(StatementResult::Empty)
            }
            Statement::Expression(expr) => {
                self.eval(environment, &expr)?;
                Ok(StatementResult::Empty)
            }
            Statement::Block(statements) => {
                let mut enclosing_environment = environment.enclose();
                for statement in statements.iter() {
                    match self.execute_statement(statement, &mut enclosing_environment)? {
                        StatementResult::Return(t) => return Ok(StatementResult::Return(t)),
                        _ => {}
                    }
                }
                Ok(StatementResult::Empty)
            }
            Statement::If(condition, then_branch, None) => {
                if Interpreter::is_truthy(&self.eval(environment, &condition)?) {
                    self.execute_statement(then_branch, environment)
                } else {
                    Ok(StatementResult::Empty)
                }
            }
            Statement::If(condition, then_branch, Some(else_branch)) => {
                if Interpreter::is_truthy(&self.eval(environment, &condition)?) {
                    self.execute_statement(then_branch, environment)
                } else {
                    self.execute_statement(else_branch, environment)
                }
            }
            Statement::While(condition, body) => {
                while Interpreter::is_truthy(&self.eval(environment, condition)?) {
                    match self.execute_statement(body, environment)? {
                        StatementResult::Return(t) => return Ok(StatementResult::Return(t)),
                        _ => {}
                    }
                }
                Ok(StatementResult::Empty)
            }
        }
    }
    fn eval(
        &mut self,
        environment: &Environment<Type>,
        expression: &Rc<Expr>,
    ) -> Result<Type, InterpreterError> {
        match expression.deref() {
            Expr::Literal(token) => match token.token_type {
                TokenType::True => Ok(Type::Boolean(true)),
                TokenType::False => Ok(Type::Boolean(false)),
                TokenType::Nil => Ok(Type::Nil),
                _ => Ok(token
                    .literal
                    .as_ref()
                    .expect("token should have a literal")
                    .as_ref()
                    .into()),
            },
            Expr::Logical(token, left, right) => match token.token_type {
                TokenType::And | TokenType::Or => {
                    let left = self.eval(environment, left)?;
                    match (token.token_type, Interpreter::is_truthy(&left)) {
                        (TokenType::Or, true) => Ok(left),
                        (TokenType::And, false) => Ok(left),
                        _ => self.eval(environment, right),
                    }
                }
                _ => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                    "Logical operator should be 'or' or 'and'",
                    Some(token.line),
                ))),
            },
            Expr::Grouping(e) => self.eval(environment, e),
            Expr::Unary(token, expr) => match token.token_type {
                TokenType::Minus => match self.eval(environment, expr)? {
                    Type::Number(n) => Ok(Type::Number(-n)),
                    _ => Err(InterpreterError::InterpreterError(ErrorMessage::new(
                        "Operand must be a number",
                        Some(token.line),
                    ))),
                },
                TokenType::Bang => Ok(Type::Boolean(!Interpreter::is_truthy(
                    &self.eval(environment, expr)?,
                ))),
                _ => panic!("oh no..."),
            },
            Expr::Binary(token, left, right) => match (
                token.token_type,
                self.eval(environment, left)?,
                self.eval(environment, right)?,
            ) {
                (TokenType::Plus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a + b)),
                (TokenType::Plus, Type::String(a), Type::String(b)) => {
                    Ok(Type::String(Rc::new(format!("{}{}", a, b))))
                }
                (TokenType::Plus, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be two numbers or two strings",
                    token.line,
                )),
                (TokenType::Minus, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a - b)),
                (TokenType::Minus, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be two numbers or two strings",
                    token.line,
                )),
                (TokenType::Slash, Type::Number(a), Type::Number(b)) => Ok(Type::Number(a / b)),
                (TokenType::Slash, _, _) => Err(InterpreterError::evaluating(
                    "Operands must be numbers",
                    token.line,
                )),
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
                _ => Err(InterpreterError::evaluating(
                    "Unrecognized binary expression",
                    token.line,
                )),
            },
            Expr::Variable(token) | Expr::This(token) => {
                match (
                    self.get_distance(expression.clone())
                        .and_then(|i| environment.get_at(&token.lexeme, *i)),
                    self.global_environment.get(&token.lexeme),
                ) {
                    (Some(value), _) => Ok(value.clone()),
                    (None, Some(value)) => Ok(value.clone()),
                    _ => Err(InterpreterError::evaluating(
                        format!("Undefined variable in scope: '{}'", token.lexeme),
                        token.line,
                    )),
                }
            }
            Expr::Assignment(token, expr) => {
                let name = token.lexeme.to_owned();
                let value = self.eval(environment, expr)?;
                match environment.assign(name, value.clone()) {
                    Ok(()) => Ok(value),
                    Err(()) => Err(InterpreterError::evaluating(
                        format!("Undefined variable '{}'", token.lexeme),
                        token.line,
                    )),
                }
            }
            Expr::Call(callee, right_paren, arguments) => {
                let callee = self.eval(environment, callee)?;

                let mut args = Vec::new();
                for arg in arguments.iter() {
                    args.push(self.eval(environment, arg)?);
                }

                match callee {
                    Type::Function(func) => {
                        let func = func.deref().borrow();
                        if args.len() != func.arity() {
                            return InterpreterError::evaluating(
                                format!(
                                    "Expected {} arguments for function '{}' but got {}",
                                    func.arity(),
                                    func,
                                    args.len()
                                ),
                                right_paren.line,
                            )
                            .into();
                        }
                        match func.borrow().call(self, args, right_paren.line)? {
                            StatementResult::Empty | StatementResult::Return(Type::Nil) => {
                                Ok(Type::Nil)
                            }
                            StatementResult::Return(t) => Ok(t),
                        }
                    }
                    Type::Class(class) => {
                        let instance = LoxInstance::new(class.clone());
                        if let Some(Type::Function(ctor)) = class.deref().find_method("init") {
                            ctor.deref().borrow_mut().bind(Some(instance.clone()));
                            ctor.deref()
                                .borrow_mut()
                                .call(self, args, right_paren.line)?;
                        }
                        Ok(Type::Instance(instance))
                    }
                    _ => Err(InterpreterError::evaluating(
                        "Can only call functions, instances and methods",
                        right_paren.line,
                    )),
                }
            }
            Expr::Function(token, fun) => {
                Ok(Type::Function(Rc::new(RefCell::new(LoxFunction::new(
                    token.as_ref().map(|i| i.lexeme.to_owned()),
                    fun.parameters
                        .iter()
                        .map(|i| i.lexeme.to_string())
                        .collect(),
                    fun.body.clone(),
                    environment.clone(),
                )))))
            }
            Expr::Get(expr, token) => match self.eval(environment, expr)? {
                Type::Instance(instance) => Ok(instance.deref().borrow().get(&token.lexeme)),
                _ => Err(InterpreterError::evaluating(
                    "Only instances have properties",
                    token.line,
                )),
            },
            Expr::Set(name, token, value) => {
                match (
                    self.eval(environment, name)?,
                    self.eval(environment, value)?,
                ) {
                    (Type::Instance(instance), value) => {
                        instance.borrow_mut().set(&token.lexeme, value);
                        Ok(Type::Nil)
                    }
                    _ => Err(InterpreterError::evaluating(
                        "Can only set properties on instances",
                        token.line,
                    )),
                }
            }
            Expr::Super(token, method) => {
                match self.get_distance(expression.clone()).map(|i| {
                    (
                        environment.get_at(&token.lexeme, *i),
                        environment.get_at("this", *i - 1),
                    )
                }) {
                    Some((Some(Type::Class(super_class)), Some(this))) => {
                        match (super_class.find_method(&method.lexeme), this) {
                            (Some(Type::Function(fun)), Type::Instance(this)) => {
                                fun.borrow_mut().bind(Some(this));
                                Ok(Type::Function(fun))
                            }
                            _ => Err(InterpreterError::evaluating(
                                format!("Method '{}' not found on the super class", &method.lexeme),
                                method.line,
                            )),
                        }
                    }
                    _ => Err(InterpreterError::evaluating(
                        format!("Undefined 'super' or '{}' in scope", method.lexeme),
                        method.line,
                    )),
                }
            }
        }
    }

    fn is_truthy(t: &Type) -> bool {
        match t {
            Type::Nil => false,
            Type::Boolean(b) => *b,
            _ => true,
        }
    }

    fn get_distance(&self, expr: Rc<Expr>) -> Option<&usize> {
        self.resolve_table
            .as_ref()
            .unwrap()
            .get(&HashableExpr::from(expr.clone()))
    }
}

enum StatementResult {
    Return(Type),
    Empty,
}

#[derive(Debug, Clone)]
enum Type {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
    Function(Rc<RefCell<dyn Function>>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<dyn Instance>>),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Nil => write!(f, "nil"),
            Type::Number(n) => write!(f, "{}", n),
            Type::String(s) => write!(f, "{}", s),
            Type::Boolean(b) => write!(f, "{}", b),
            Type::Function(fun) => write!(f, "{}", fun.deref().borrow()),
            Type::Class(class) => write!(f, "{}", class),
            Type::Instance(instance) => write!(f, "{}", instance.deref().borrow()),
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

trait Function: Debug + Display {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Type>,
        line: usize,
    ) -> Result<StatementResult, InterpreterError>;

    fn arity(&self) -> usize;
    fn name(&self) -> &str;
    fn bind(&mut self, this: Option<Rc<RefCell<dyn Instance>>>);
}

struct LoxFunction {
    name: String,
    parameters: Vec<String>,
    body: Rc<Statement>,
    closure: Environment<Type>,
}

impl LoxFunction {
    fn new(
        name: Option<String>,
        parameters: Vec<String>,
        body: Rc<Statement>,
        closure: Environment<Type>,
    ) -> Self {
        Self {
            name: name
                .map(|i| i.to_string())
                .unwrap_or_else(|| "__<fun_anon>".to_owned()),
            parameters,
            body,
            closure,
        }
    }
}

impl Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LoxFunction {{ name: \"{:?}\", parameters: {:?}, body: {:?} }}",
            self.name, self.parameters, self.body
        )
    }
}

impl Function for LoxFunction {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Type>,
        _: usize,
    ) -> Result<StatementResult, InterpreterError> {
        let env = Environment::enclose(&self.closure);

        for arg in self.parameters.iter().zip(arguments) {
            env.define(arg.0, arg.1);
        }

        interpreter.execute_statement(&self.body, &env)
    }

    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn bind(&mut self, this: Option<Rc<RefCell<dyn Instance>>>) {
        if this.is_some() {
            let env = self.closure.enclose();
            env.define("this", Type::Instance(this.unwrap()));
            self.closure = env;
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}

mod native_functions {
    use std::{
        env,
        fmt::Display,
        rc::Rc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::errors::{ErrorMessage, InterpreterError};

    use super::{Function, Interpreter, StatementResult, Type};

    #[derive(Debug)]
    pub struct Clock {}

    impl Function for Clock {
        fn arity(&self) -> usize {
            0
        }

        fn call(
            &self,
            _: &mut Interpreter,
            _: Vec<super::Type>,
            line: usize,
        ) -> Result<StatementResult, InterpreterError> {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => Ok(StatementResult::Return(Type::Number(
                    duration.as_secs() as f64
                ))),
                Err(error) => Err(InterpreterError::RuntimeError(ErrorMessage::new(
                    format!("System time error: {}", error),
                    Some(line),
                ))),
            }
        }

        fn name(&self) -> &str {
            "clock"
        }

        fn bind(&mut self, _: Option<Rc<std::cell::RefCell<dyn super::Instance>>>) {}
    }

    impl Display for Clock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "clock")
        }
    }

    #[derive(Debug)]
    pub struct Env {}

    impl Function for Env {
        fn arity(&self) -> usize {
            1
        }

        fn call(
            &self,
            _: &mut Interpreter,
            arguments: Vec<Type>,
            line: usize,
        ) -> Result<StatementResult, InterpreterError> {
            match arguments.as_slice() {
                [Type::String(key)] => match env::var(key.as_str()) {
                    Ok(value) => Ok(StatementResult::Return(Type::String(Rc::new(value)))),
                    Err(_) => Ok(StatementResult::Return(Type::Nil)),
                },
                _ => Err(InterpreterError::RuntimeError(ErrorMessage::new(
                    "Invalid argument to 'env' function",
                    Some(line),
                ))),
            }
        }

        fn name(&self) -> &str {
            "env"
        }

        fn bind(&mut self, _: Option<Rc<std::cell::RefCell<dyn super::Instance>>>) {}
    }

    impl Display for Env {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "env")
        }
    }
}

trait Instance: Debug + Display {
    fn get(&self, name: &str) -> Type;
    fn set(&mut self, name: &str, value: Type);
}

#[derive(Debug, Clone)]
struct LoxClass {
    name: String,
    methods: HashMap<String, Type>,
    super_class: Option<Rc<LoxClass>>,
}

impl LoxClass {
    fn new(name: String, methods: HashMap<String, Type>) -> Self {
        Self {
            name,
            methods,
            super_class: None,
        }
    }

    fn with_superclass(
        name: String,
        methods: HashMap<String, Type>,
        super_class: Rc<LoxClass>,
    ) -> Self {
        Self {
            name,
            methods,
            super_class: Some(super_class),
        }
    }

    fn find_method(&self, name: &str) -> Option<Type> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        } else {
            self.super_class
                .as_ref()
                .map(|i| i.find_method(name))
                .flatten()
        }
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "class {} {{...}}", self.name)
    }
}

#[derive(Debug)]
struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Type>,
    this: Option<Rc<RefCell<dyn Instance>>>,
}

impl LoxInstance {
    fn new(class: Rc<LoxClass>) -> Rc<RefCell<Self>> {
        let instance = Self {
            class: class.clone(),
            fields: HashMap::new(),
            this: None,
        };
        let instance = Rc::new(RefCell::new(instance));
        instance.deref().borrow_mut().this = Some(instance.clone());
        instance
    }
}

impl Instance for LoxInstance {
    fn get(&self, name: &str) -> Type {
        match (self.fields.get(name), self.class.find_method(name)) {
            (Some(value), _) => value.clone(),
            (None, Some(Type::Function(function))) => {
                function.deref().borrow_mut().bind(self.this.clone());
                return Type::Function(function);
            }
            _ => Type::Nil,
        }
    }

    fn set(&mut self, name: &str, value: Type) {
        self.fields.insert(name.to_owned(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
