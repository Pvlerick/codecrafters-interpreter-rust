use crate::common::interpreter;

mod common;

#[test]
fn function_clock() {
    let (_, err) = interpreter::run_content("print clock();");
    assert!(err.is_none());
}

#[test]
fn function_env() {
    std::env::set_var("LOX_TEST_ENV_VAR", "foo");
    let (output, err) = interpreter::run_content("print env(\"LOX_TEST_ENV_VAR\");");
    assert!(err.is_none());
    assert_eq!("foo\n", output);
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
