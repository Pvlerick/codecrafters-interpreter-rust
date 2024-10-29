use crate::common::interpreter;

mod common;

#[test]
fn print_class() {
    let (output, err) = interpreter::run_content(
        r#"class Doughnut {
    tastes() {
        return "good";
    }
}

class BostonCream < Doughnut {}

var bc = BostonCream();
print bc.tastes();"#,
    );
    assert_none!(err);
    assert_eq!("good\n", output);
}

#[test]
fn cant_inherit_from_non_existing() {
    let (_, err) = interpreter::run_content("class A < B {}");
    assert_some!(err);
}

#[test]
fn cant_inherit_from_self() {
    let (_, err) = interpreter::run_content("class A < A {}");
    assert_some!(err);
}

#[test]
fn must_inherit_from_class() {
    let (_, err) = interpreter::run_content(
        r#"var NotAClass = "I'm not a class";
        class A < NotAClass {}"#,
    );
    assert_some!(err);
}
