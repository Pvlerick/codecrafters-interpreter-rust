use common::parser;
use interpreter_starter_rust::resolver::Resolver;

mod common;

#[test]
fn resolver_empty() {
    let res = parser::parse_content("");
    assert!(res.is_ok());
    let mut sut = Resolver::new();
    let res = sut.resolve(&res.unwrap());
    assert!(res.is_ok());
    assert_eq!(0, sut.resolve_table.keys().count());
}

#[test]
fn resolver_one_variable() {
    let res = parser::parse_content(
        r#"{
    var foo = 42;
    print foo;
}"#,
    );
    assert!(res.is_ok());
    let mut sut = Resolver::new();
    let res = sut.resolve(&res.unwrap());
    assert!(res.is_ok());
    assert_eq!(1, sut.resolve_table.keys().count());
}
