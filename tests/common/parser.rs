use std::io::BufReader;

use interpreter_starter_rust::{
    errors::InterpreterError,
    parser::{Expr, Parser, Statement},
};

use super::reader::StrReader;

#[allow(dead_code)]
pub fn parse_content(content: &'static str) -> Result<Vec<Statement>, InterpreterError> {
    let mut parser = build_parser(content);
    parser.parse().map(|i| i.collect::<Vec<_>>())
}

#[allow(dead_code)]
pub fn parse_content_to_expression(
    content: &'static str,
) -> Result<Option<Expr>, InterpreterError> {
    let mut parser = build_parser(content);
    parser.parse_expression()
}

fn build_parser(content: &'static str) -> Parser {
    Parser::build(BufReader::new(StrReader::new(content))).unwrap()
}
