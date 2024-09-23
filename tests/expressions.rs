use crate::common::parser;

mod common;

#[test]
fn parse_expression_bang_equal() {
    let res = parser::parse_content("(12 != 13)");
    assert!(res.is_ok());
    assert_eq!(
        "(group (!= 12.0 13.0))",
        format!("{}", res.unwrap().unwrap())
    );
}

#[test]
fn parse_expression_equal_equal() {
    let res = parser::parse_content("\"foo\"==\"bar\"");
    assert!(res.is_ok());
    assert_eq!("(== foo bar)", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn parser_complex_expression_1() {
    let res = parser::parse_content("(85 + 64 - 59) > (30 - 85) * 2");
    assert!(res.is_ok());
    assert_eq!(
        "(> (group (- (+ 85.0 64.0) 59.0)) (* (group (- 30.0 85.0)) 2.0))",
        format!("{}", res.unwrap().unwrap())
    );
}
