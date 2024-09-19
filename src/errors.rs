use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ScanningError {
    message: String,
}

impl Error for ScanningError {}

impl From<String> for ScanningError {
    fn from(value: String) -> Self {
        ScanningError { message: value }
    }
}

impl From<&str> for ScanningError {
    fn from(value: &str) -> Self {
        ScanningError {
            message: value.to_owned(),
        }
    }
}

impl From<Vec<TokenError>> for ScanningError {
    fn from(value: Vec<TokenError>) -> Self {
        ScanningError {
            message: value
                .into_iter()
                .map(|i| i.message)
                .collect::<Vec<_>>()
                .join("\r"),
        }
    }
}

impl Display for ScanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
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

impl<T> Into<Result<T, TokenError>> for TokenError {
    fn into(self) -> Result<T, TokenError> {
        Err(self)
    }
}

impl Error for TokenError {}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}.", self.line, self.message)
    }
}

#[derive(Debug)]
pub struct ParsingErrors {
    errors: Vec<String>,
}

impl Error for ParsingErrors {}

impl From<Vec<String>> for ParsingErrors {
    fn from(value: Vec<String>) -> Self {
        ParsingErrors { errors: value }
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
        EvaluationError { message: value }
    }
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
