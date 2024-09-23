use std::error::Error;

use interpreter_starter_rust::parser::{Expr, Parser};

use super::TempFile;

#[allow(dead_code)]
pub fn parse_content(content: &str) -> Result<Option<Expr>, Box<dyn Error>> {
    let mut tmp_file = TempFile::with_content(content);
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    parser.parse_expression()
}
