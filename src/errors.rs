use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum InterpreterError {
    ScanningError(ErrorMessage),
    ScanningErrors(Vec<ErrorMessage>),
    ParsingErrors(Vec<ErrorMessage>),
    InterpreterError(ErrorMessage),
    RuntimeError(ErrorMessage),
}

impl InterpreterError {
    pub fn scanning<T: ToString>(message: T) -> InterpreterError {
        InterpreterError::ScanningError(ErrorMessage::new(message, None))
    }

    pub fn parsing<T: ToString>(message: T) -> InterpreterError {
        InterpreterError::ScanningError(ErrorMessage::new(message.to_string(), None))
    }

    pub fn evaluating<T: ToString>(message: T, line: usize) -> InterpreterError {
        InterpreterError::InterpreterError(ErrorMessage::new(message, Some(line)))
    }
}

impl Error for InterpreterError {}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::ScanningError(msg) | InterpreterError::InterpreterError(msg) => {
                write!(f, "{}", msg)
            }
            InterpreterError::ScanningErrors(msgs) | InterpreterError::ParsingErrors(msgs) => {
                for msg in msgs {
                    writeln!(f, "{}", msg)?;
                }
                Ok(())
            }
            InterpreterError::RuntimeError(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug)]
pub struct ErrorMessage {
    message: String,
    line: Option<usize>,
}

impl ErrorMessage {
    pub fn new<T: ToString>(message: T, line: Option<usize>) -> Self {
        Self {
            message: message.to_string(),
            line,
        }
    }
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "[line {}] Error: {}.", line, self.message),
            None => write!(f, "Error: {}.", self.message),
        }
    }
}

pub struct ParsingErrorsBuilder {
    errors: Vec<ErrorMessage>,
}

impl ParsingErrorsBuilder {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add<T: ToString>(&mut self, message: T, line: usize) {
        self.errors.push(ErrorMessage::new(message, Some(line)));
    }

    pub fn build(self) -> InterpreterError {
        InterpreterError::ParsingErrors(self.errors)
    }
}

#[derive(Debug)]
pub struct TokenError {
    pub message: String,
    pub line: usize,
}

impl TokenError {
    pub fn new<T>(msg: T, line: usize) -> Self
    where
        T: Into<String>,
    {
        Self {
            message: msg.into(),
            line,
        }
    }
}

impl<T> Into<Result<T, Self>> for TokenError {
    fn into(self) -> Result<T, Self> {
        Err(self)
    }
}

impl Error for TokenError {}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.message)
    }
}

#[derive(Debug)]
pub struct ParsingErrors {
    errors: Vec<String>,
}

impl ParsingErrors {
    pub fn new(errors: Vec<String>) -> Self {
        Self { errors }
    }
}

impl Error for ParsingErrors {}

impl From<Vec<String>> for ParsingErrors {
    fn from(value: Vec<String>) -> Self {
        Self { errors: value }
    }
}

impl Display for ParsingErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for error in self.errors.iter() {
            writeln!(f, "{}", error)?
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct EvaluationError {
    message: String,
}

impl Error for EvaluationError {}

impl From<String> for EvaluationError {
    fn from(value: String) -> Self {
        Self { message: value }
    }
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
