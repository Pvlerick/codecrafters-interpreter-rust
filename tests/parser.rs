mod common;

use interpreter_starter_rust::{parser::Parser, scanner::Scanner};

use crate::common::TempFile;

#[test]
fn parser_number() {
    let mut tmp_file = TempFile::with_content("123.456");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner.scan_tokens();
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap().collect::<Vec<_>>();
    assert!(tokens.iter().all(|i| i.is_ok()));
    let mut parser = Parser::new(tokens.into_iter().map(|i| i.unwrap()));
    assert_eq!("123.456", parser.parse().unwrap());
}

#[test]
fn parser_expression() {
    let mut tmp_file = TempFile::with_content("(12 != 13)");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner.scan_tokens();
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap().collect::<Vec<_>>();
    assert!(tokens.iter().all(|i| i.is_ok()));
    let mut parser = Parser::new(tokens.into_iter().map(|i| i.unwrap()));
    assert_eq!("(group (!= 12.0 13.0))", parser.parse().unwrap());
}

#[test]
fn parser_empty() {
    let mut tmp_file = TempFile::with_content("");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_err());
    assert_eq!("Expect expression.", res.unwrap_err()[0]);
}

#[test]
fn parser_invalid_grammar() {
    let mut tmp_file = TempFile::with_content("(false 123.456 \"test\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_err());
    assert_eq!("Expect ')' after expression.", res.unwrap_err()[0]);
}
