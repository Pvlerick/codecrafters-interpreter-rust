mod common;

use interpreter_starter_rust::parser::Parser;

use crate::common::TempFile;

#[test]
fn parser_boolean() {
    let mut tmp_file = TempFile::with_content("true");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_ok());
    assert_eq!("true", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn parser_number() {
    let mut tmp_file = TempFile::with_content("123.456");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_ok());
    assert_eq!("123.456", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn parser_expression_1() {
    let mut tmp_file = TempFile::with_content("(12 != 13)");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_ok());
    assert_eq!(
        "(group (!= 12.0 13.0))",
        format!("{}", res.unwrap().unwrap())
    );
}

#[test]
fn parser_expression_2() {
    let mut tmp_file = TempFile::with_content("\"foo\"!=\"bar\"");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_ok());
    assert_eq!("(!= foo bar)", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn parser_empty() {
    let mut tmp_file = TempFile::with_content("");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_err());
    // assert_eq!("", res.unwrap_err().to_string());
}

#[test]
fn parser_invalid_grammar() {
    let mut tmp_file = TempFile::with_content("(false 123.456 \"test\"");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
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
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_err());
    // assert_eq!("Expect ')' after expression.", res.unwrap_err().to_string());
}

#[test]
fn parser_print_statement() {
    let mut tmp_file = TempFile::with_content("print 42;");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_err());
    // assert_eq!("(print 42)", res.unwrap_err().to_string());
}
