use crate::common::interpreter;

mod common;

#[test]
fn r#if_true() {
    let (output, err) = interpreter::run_content("if (true) print 42;");
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn r#if_false() {
    let (output, err) = interpreter::run_content("if (false) print 42;");
    assert!(err.is_none());
    assert_eq!("", output);
}

#[test]
fn r#if_using_variable() {
    let (output, err) = interpreter::run_content(
        r#"var foo = true;
if (foo) print 42;"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn r#if_else_1() {
    let (output, err) = interpreter::run_content(
        r#"if (true) {
    print 42;
} else {
    print 84;
}"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn r#if_else_2() {
    let (output, err) = interpreter::run_content(
        r#"if (false) {
    print 42;
} else {
    print 84;
}"#,
    );
    assert!(err.is_none());
    assert_eq!("84\n", output);
}

#[test]
fn r#if_emtpy_else() {
    let (output, err) = interpreter::run_content(
        r#"if (false) {
    print 42;
} else {
}"#,
    );
    assert!(err.is_none());
    assert_eq!("", output);
}

#[test]
fn r#while_add_print_total() {
    let (output, err) = interpreter::run_content(
        r#"var i = 0;
while (i < 3) i = i + 1;
print i;"#,
    );
    assert!(err.is_none());
    assert_eq!("3\n", output);
}

#[test]
fn r#while_add_print_in_loop() {
    let (output, err) = interpreter::run_content(
        r#"var i = 0;
while (i < 3) {
    print i;
    i = i + 1;
}"#,
    );
    assert!(err.is_none());
    assert_eq!("0\n1\n2\n", output);
}
