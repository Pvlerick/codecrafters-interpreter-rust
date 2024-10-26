use crate::common::interpreter;

mod common;

#[test]
fn r#or_string_number() {
    let (output, err) = interpreter::run_content(r#"print "foo" or 2;"#);
    assert_none!(err);
    assert_eq!("foo\n", output);
}

#[test]
fn r#or_nil_string() {
    let (output, err) = interpreter::run_content(r#"print nil or "bar";"#);
    assert_none!(err);
    assert_eq!("bar\n", output);
}

#[test]
fn r#if_or() {
    let (output, err) = interpreter::run_content(
        r#"if (true or false) {
    print 42;
}"#,
    );
    assert_none!(err);
    assert_eq!("42\n", output);
}

#[test]
fn r#if_and_1() {
    let (output, err) = interpreter::run_content(
        r#"if (true and false) {
    print 42;
}"#,
    );
    assert_none!(err);
    assert_eq!("", output);
}

#[test]
fn r#if_and_nil_number() {
    let (output, err) = interpreter::run_content(
        r#"if (nil and 42) {
    print 42;
} else {
    print 84;
}"#,
    );
    assert_none!(err);
    assert_eq!("84\n", output);
}
