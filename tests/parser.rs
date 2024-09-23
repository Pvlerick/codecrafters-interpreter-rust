mod common;

use interpreter_starter_rust::parser::Parser;

use crate::common::TempFile;

#[test]
fn parser_empty() {
    let mut tmp_file = TempFile::with_content("");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_err());
}

#[test]
fn parser_invalid_grammar() {
    let mut tmp_file = TempFile::with_content("(false 123.456 \"test\"");
    let mut parser = Parser::build(tmp_file.reader()).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_err());
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
}
