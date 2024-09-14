use interpreter_starter_rust::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

use crate::common::TempFile;

mod common;

#[test]
fn evaluate_literal_string() {
    let mut tmp_file = TempFile::with_content("\"foo\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let interpreter = Interpreter::new(parser.parse().unwrap());
    assert_eq!("foo", interpreter.evaluate());
}

#[test]
fn evaluate_literal_numeric() {
    let mut tmp_file = TempFile::with_content("42");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let interpreter = Interpreter::new(parser.parse().unwrap());
    assert_eq!("42", interpreter.evaluate());
}

#[test]
fn evaluate_grouping_literal() {
    let mut tmp_file = TempFile::with_content("(42)");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let interpreter = Interpreter::new(parser.parse().unwrap());
    assert_eq!("42", interpreter.evaluate());
}

#[test]
fn evaluate_unary() {
    let mut tmp_file = TempFile::with_content("-42");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let interpreter = Interpreter::new(parser.parse().unwrap());
    assert_eq!("-42", interpreter.evaluate());
}

#[test]
fn evaluate_addition() {
    let mut tmp_file = TempFile::with_content("1 + 2");
    let mut scanner = Scanner::new(tmp_file.reader());
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let interpreter = Interpreter::new(parser.parse().unwrap());
    assert_eq!("3", interpreter.evaluate());
}
