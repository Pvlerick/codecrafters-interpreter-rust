use std::{error::Error, io::BufReader};

use interpreter_starter_rust::parser::{Expr, Parser};

use super::reader::StrReader;

#[allow(dead_code)]
pub fn parse_content(content: &'static str) -> Result<Option<Expr>, Box<dyn Error>> {
    let mut parser = Parser::build(BufReader::new(StrReader::new(content))).unwrap();
    parser.parse_expression()
}
