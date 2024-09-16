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

impl Display for ScanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct TokenError {
    message: String,
}

impl Error for TokenError {}

impl From<String> for TokenError {
    fn from(value: String) -> Self {
        TokenError { message: value }
    }
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct ParsingError {
    messages: Vec<String>,
}

impl Error for ParsingError {}

impl From<Vec<String>> for ParsingError {
    fn from(value: Vec<String>) -> Self {
        ParsingError { messages: value }
    }
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.messages.join("\n"))
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
