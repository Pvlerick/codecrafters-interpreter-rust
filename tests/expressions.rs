use crate::common::{interpreter, parser};

mod common;

#[test]
fn parse_content_to_expression_bang_equal() {
    let res = parser::parse_content_to_expression("(12 != 13)");
    assert!(res.is_ok());
    assert_eq!(
        "(group (!= 12.0 13.0))",
        format!("{}", res.unwrap().unwrap())
    );
}

#[test]
fn parse_content_to_expression_equal_equal() {
    let res = parser::parse_content_to_expression("\"foo\"==\"bar\"");
    assert!(res.is_ok());
    assert_eq!("(== foo bar)", format!("{}", res.unwrap().unwrap()));
}

#[test]
fn parse_complex_expression_1() {
    let res = parser::parse_content_to_expression("(85 + 64 - 59) > (30 - 85) * 2");
    assert!(res.is_ok());
    assert_eq!(
        "(> (group (- (+ 85.0 64.0) 59.0)) (* (group (- 30.0 85.0)) 2.0))",
        format!("{}", res.unwrap().unwrap())
    );
}

#[test]
fn run_expression_statements() {
    let (output, err) = interpreter::run_content(
        r#"(85 + 64 - 59) > (30 - 85) * 2;
print !false;
"world" + "hello" + "foo" + "quz" == "worldhellofooquz";
print !false;"#,
    );
    assert_none!(err);
    assert_eq!(
        r#"true
true
"#,
        output
    );
}
