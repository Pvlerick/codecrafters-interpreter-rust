mod common;

use interpreter_starter_rust::{parser::Parser, scanner::Scanner};

use crate::common::TempFile;

#[test]
fn parser_number() {
    let mut tmp_file = TempFile::with_content("123.456");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    assert_eq!("123.456", parser.parse());
}

#[test]
fn parser_expression() {
    let mut tmp_file = TempFile::with_content("(12 != 13)");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    assert_eq!("(group (!= 12.0 13.0))", parser.parse());
}
