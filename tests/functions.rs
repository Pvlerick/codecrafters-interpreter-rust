use crate::common::interpreter;

mod common;

#[test]
fn function_clock() {
    let (output, err) = interpreter::run_content("print clock()");
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn function_print_42() {
    let (output, err) = interpreter::run_content(
        r#"fun print42() {
    print 42;
}

print42();
print42();"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n42\n", output);
}

#[test]
fn function_print_sum() {
    let (output, err) = interpreter::run_content(
        r#"fun printSum(a, b) {
    print a + b;
}

printSum(42, 5);"#,
    );
    assert!(err.is_none());
    assert_eq!("47\n", output);
}

#[test]
fn function_return() {
    let (output, err) = interpreter::run_content(
        r#"fun sum(a, b) {
    return a + b;
}

print sum(37, 5);"#,
    );
    assert!(err.is_none());
    assert_eq!("42\n", output);
}

#[test]
fn function_err_not_a_function() {
    let (_, err) = interpreter::run_content(r#""not a function();"#);
    assert!(err.is_some());
}
