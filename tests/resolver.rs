use common::parser;
use interpreter_starter_rust::resolver::Resolver;

mod common;

#[test]
fn resolver_empty() {
    let res = parser::parse_content("");
    assert!(res.is_ok());
    let mut sut = Resolver::new();
    sut.resolve(&res.unwrap());
}

#[test]
fn resolver_one_variable() {
    let res = parser::parse_content("var foo = 42");
    assert!(res.is_ok());
    let mut sut = Resolver::new();
    sut.resolve(&res.unwrap());
}
