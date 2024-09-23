mod common;

use std::rc::Rc;

use interpreter_starter_rust::scanner::{Literal, Token, TokenType};

use crate::common::{interpreter, parser, scanner};

#[test]
fn scan_string_literal() {
    let tokens = scanner::scan_content("foo \"hello\"");
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
fn scan_digit_literal() {
    let tokens = scanner::scan_content("bar 123.456");
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
fn scan_numbers() {
    let tokens = scanner::scan_content("1 + 2");
    use TokenType::*;
    assert_eq!(vec![Number, Plus, Number, EOF], tokens);
}

#[test]
fn scan_identifiers() {
    let tokens = scanner::scan_content("test");
    use TokenType::*;
    assert_eq!(vec![Identifier, EOF], tokens);
}

#[test]
fn scan_keywords() {
    let tokens = scanner::scan_content("true false class");
    use TokenType::*;
    assert_eq!(vec![True, False, Class, EOF], tokens);
}

#[test]
fn parse_boolean() {
    let res = parser::parse_content("true");
    assert!(res.is_ok());
    assert_eq!("true", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn parse_number() {
    let res = parser::parse_content("123.456");
    assert!(res.is_ok());
    assert_eq!("123.456", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn evaluate_literal_string() {
    let (output, err) = interpreter::evaluate_content("\"foo\"");
    assert!(err.is_none());
    assert_eq!("foo", output);
}

#[test]
fn evaluate_literal_numeric() {
    let (output, err) = interpreter::evaluate_content("42");
    assert!(err.is_none());
    assert_eq!("42", output);
}
