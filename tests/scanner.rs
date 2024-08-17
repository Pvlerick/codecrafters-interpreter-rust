mod common;

use interpreter_starter_rust::scanner::{Scanner, TokenType};

use crate::common::TempFile;

#[test]
fn scanner_simple() {
    let mut tmp_file = TempFile::with_content("1 + 2");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan_tokens()
        .unwrap()
        .map(|i| i.unwrap().token_type)
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(vec![Number, Plus, Number, EOF], tokens);
}

#[test]
fn scanner_identifiers() {
    let mut tmp_file = TempFile::with_content("test");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan_tokens()
        .unwrap()
        .map(|i| i.unwrap().token_type)
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(vec![Identifier, EOF], tokens);
}

#[test]
fn scanner_keywords() {
    let mut tmp_file = TempFile::with_content("true false class");
    let mut scanner = Scanner::new(tmp_file.reader());
    let tokens = scanner
        .scan_tokens()
        .unwrap()
        .map(|i| i.unwrap().token_type)
        .collect::<Vec<_>>();
    use TokenType::*;
    assert_eq!(vec![True, False, Class, EOF], tokens);
}
