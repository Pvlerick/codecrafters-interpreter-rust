mod common;

use crate::common::parser;

#[test]
fn parser_empty() {
    let res = parser::parse_content_to_expression("");
    assert!(res.is_err());
}

#[test]
fn parser_invalid_grammar() {
    let res = parser::parse_content_to_expression("(false 123.456 \"test\"");
    assert!(res.is_err());
}

#[test]
fn parser_invalid_grammar_multiple_error() {
    let res = parser::parse_content_to_expression(
        r#"(false
var a = bleh;
beh"#,
    );
    assert!(res.is_err());
}
