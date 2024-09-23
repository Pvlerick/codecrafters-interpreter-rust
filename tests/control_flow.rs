use crate::common::interpreter;

mod common;

#[test]
fn r#if_true() {
    let (output, err) = interpreter::run_content(
        r#"if true {
    print 42
}"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn r#if_false() {
    let (output, err) = interpreter::run_content(
        r#"if false {
    print 42
}"#,
    );
    assert!(err.is_none());
    assert_eq!("", output);
}

#[test]
fn r#if_using_variable() {
    let (output, err) = interpreter::run_content(
        r#"var foo = true;
if foo {
    print 42
}"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}
