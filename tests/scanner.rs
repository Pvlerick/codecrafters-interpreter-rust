mod common;

use std::rc::Rc;

use interpreter_starter_rust::scanner::{Literal, Scanner, Token, TokenType};

use crate::common::TempFile;

#[test]
fn scanner_string_literal() {
    let mut tmp_file = TempFile::with_content("foo \"hello\"");
    let scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan()
        .unwrap()
        .map(|i| i.unwrap())
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(
        vec![
            Token::new(Identifier, "foo".to_string(), 1),
            Token::with_literal(
                String,
                "\"hello\"".to_string(),
                Literal::String(Rc::new("hello".to_string())),
                1
            ),
            Token::new(EOF, "", 2)
        ],
        tokens
    );
}

#[test]
fn scanner_digit_literal() {
    let mut tmp_file = TempFile::with_content("bar 123.456");
    let scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan()
        .unwrap()
        .map(|i| i.unwrap())
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(
        vec![
            Token::new(Identifier, "bar".to_string(), 1),
            Token::with_literal(Number, "123.456".to_string(), Literal::Digit(123.456f64), 1),
            Token::new(EOF, "", 2)
        ],
        tokens
    );
}

#[test]
fn scanner_numbers() {
    let mut tmp_file = TempFile::with_content("1 + 2");
    let scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan()
        .unwrap()
        .map(|i| i.unwrap())
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(vec![Number, Plus, Number, EOF], tokens);
}

#[test]
fn scanner_identifiers() {
    let mut tmp_file = TempFile::with_content("test");
    let scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan()
        .unwrap()
        .map(|i| i.unwrap())
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(vec![Identifier, EOF], tokens);
}

#[test]
fn scanner_keywords() {
    let mut tmp_file = TempFile::with_content("true false class");
    let scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan()
        .unwrap()
        .map(|i| i.unwrap())
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(vec![True, False, Class, EOF], tokens);
}
