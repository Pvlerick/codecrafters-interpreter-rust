mod common;

use interpreter_starter_rust::{parser::Parser, scanner::Scanner};

use crate::common::TempFile;

#[test]
fn parser_boolean() {
    let mut tmp_file = TempFile::with_content("true");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner.scan_tokens();
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap().collect::<Vec<_>>();
    assert!(tokens.iter().all(|i| i.is_ok()));
    let parser = Parser::new(tokens.into_iter().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap().to_string());
}

#[test]
fn parser_number() {
    let mut tmp_file = TempFile::with_content("123.456");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner.scan_tokens();
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap().collect::<Vec<_>>();
    assert!(tokens.iter().all(|i| i.is_ok()));
    let parser = Parser::new(tokens.into_iter().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_ok());
    assert_eq!("123.456", res.unwrap().to_string());
}

#[test]
fn parser_expression() {
    let mut tmp_file = TempFile::with_content("(12 != 13)");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner.scan_tokens();
    assert!(tokens.is_ok());
    let tokens = tokens.unwrap().collect::<Vec<_>>();
    assert!(tokens.iter().all(|i| i.is_ok()));
    let parser = Parser::new(tokens.into_iter().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_ok());
    assert_eq!("(group (!= 12.0 13.0))", res.unwrap().to_string());
}

#[test]
fn parser_empty() {
    let mut tmp_file = TempFile::with_content("");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_err());
    // CodeCrafter wants this, but the book says otherwise
    // https://craftinginterpreters.com/parsing-expressions.html#wiring-up-the-parser
    assert_eq!("", format!("{}", res.unwrap_err()));
}

#[test]
fn parser_invalid_grammar() {
    let mut tmp_file = TempFile::with_content("(false 123.456 \"test\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_err());
    assert_eq!(
        "Expect ')' after expression.",
        format!("{}", res.unwrap_err())
    );
}

#[test]
fn parser_invalid_grammar_multiple_error() {
    let mut tmp_file = TempFile::with_content(
        r#"(false
var a = bleh;
beh"#,
    );
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
    let res = parser.parse();
    assert!(res.is_err());
    let error = res.unwrap_err();
    assert_eq!("Expect ')' after expression.", format!("{}", error));
    // assert_eq!("Expect ')' after expression.", errors[1]);
    // assert_eq!("Expect ')' after expression.", errors[2]);
}
