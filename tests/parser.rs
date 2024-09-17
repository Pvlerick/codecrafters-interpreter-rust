mod common;

use interpreter_starter_rust::parser::Parser;

use crate::common::TempFile;

#[test]
fn parser_boolean() {
    let mut tmp_file = TempFile::with_content("true");
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap().next().unwrap().to_string());
}

#[test]
fn parser_number() {
    let mut tmp_file = TempFile::with_content("123.456");
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_ok());
    assert_eq!("123.456", res.unwrap().next().unwrap().to_string());
}

#[test]
fn parser_expression() {
    let mut tmp_file = TempFile::with_content("(12 != 13)");
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_ok());
    assert_eq!(
        "(group (!= 12.0 13.0))",
        res.unwrap().next().unwrap().to_string()
    );
}

#[test]
fn parser_empty() {
    let mut tmp_file = TempFile::with_content("");
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_err());
    // assert_eq!("", res.unwrap_err().to_string());
}

#[test]
fn parser_invalid_grammar() {
    let mut tmp_file = TempFile::with_content("(false 123.456 \"test\"");
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_err());
    // assert_eq!("Expect ')' after expression.", res.unwrap_err().to_string());
}

#[test]
fn parser_invalid_grammar_multiple_error() {
    let mut tmp_file = TempFile::with_content(
        r#"(false
var a = bleh;
beh"#,
    );
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_err());
    // assert_eq!("Expect ')' after expression.", res.unwrap_err().to_string());
}

#[test]
fn parser_print_statement() {
    let mut tmp_file = TempFile::with_content("print 42;");
    let parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse();
    assert!(res.is_err());
    // assert_eq!("(print 42)", res.unwrap_err().to_string());
}
