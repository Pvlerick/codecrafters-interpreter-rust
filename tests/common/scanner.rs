use std::io::BufReader;

use interpreter_starter_rust::scanner::{Scanner, Token};

use super::reader::StrReader;

#[allow(dead_code)]
pub fn scan_content(content: &'static str) -> Vec<Token> {
    let scanner = Scanner::new(BufReader::new(StrReader::new(content)));
    scanner
        .scan()
        .unwrap()
        .map(|i| i.unwrap())
        .collect::<Vec<_>>()
}
