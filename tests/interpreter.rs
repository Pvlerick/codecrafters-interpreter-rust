use crate::common::interpreter;

mod common;

#[test]
fn evaluate_runtime_error_1() {
    let (_, err) = interpreter::evaluate_content("-\"muffin\"");
    assert_some!(err);
    assert_eq!(
        "[line 1] Error: Operand must be a number.",
        err.unwrap().to_string()
    );
}

#[test]
fn evaluate_runtime_error_2() {
    let (_, err) = interpreter::evaluate_content("3 / \"muffin\"");
    assert_some!(err);
    assert_eq!(
        "[line 1] Error: Operands must be numbers.",
        err.unwrap().to_string()
    );
}

#[test]
fn evaluate_runtime_error_3() {
    let (_, err) = interpreter::evaluate_content("3 + \"muffin\"");
    assert_some!(err);
    assert_eq!(
        "[line 1] Error: Operands must be two numbers or two strings.",
        err.unwrap().to_string()
    );
}

#[test]
fn evaluate_runtime_error_4() {
    let (_, err) = interpreter::evaluate_content("3 - \"muffin\"");
    assert_some!(err);
    assert_eq!(
        "[line 1] Error: Operands must be two numbers or two strings.",
        err.unwrap().to_string()
    );
}
