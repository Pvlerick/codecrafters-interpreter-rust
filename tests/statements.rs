use crate::common::parser;

mod common;

#[test]
fn parse_print() {
    let res = parser::parse_content("print 42;");
    assert!(res.is_err());
}
