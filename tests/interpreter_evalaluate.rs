use crate::common::interpreter;

mod common;

#[test]
fn evaluate_grouping_literal() {
    let (output, err) = interpreter::evaluate_content("(42)");
    assert!(err.is_none());
    assert_eq!("42", output);
}

#[test]
fn evaluate_unary() {
    let (output, err) = interpreter::evaluate_content("-42");
    assert!(err.is_none());
    assert_eq!("-42", output);
}

#[test]
fn evaluate_true() {
    let (output, err) = interpreter::evaluate_content("true");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_false() {
    let (output, err) = interpreter::evaluate_content("false");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_nil() {
    let (output, err) = interpreter::evaluate_content("nil");
    assert!(err.is_none());
    assert_eq!("nil", output);
}

#[test]
fn evaluate_not_1() {
    let (output, err) = interpreter::evaluate_content("!false");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_not_2() {
    let (output, err) = interpreter::evaluate_content("!!false");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_not_3() {
    let (output, err) = interpreter::evaluate_content("!!42");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_addition() {
    let (output, err) = interpreter::evaluate_content("1 + 2");
    assert!(err.is_none());
    assert_eq!("3", output);
}

#[test]
fn evaluate_substraction() {
    let (output, err) = interpreter::evaluate_content("5 - 3");
    assert!(err.is_none());
    assert_eq!("2", output);
}

#[test]
fn evaluate_division_1() {
    let (output, err) = interpreter::evaluate_content("20 / 2");
    assert!(err.is_none());
    assert_eq!("10", output);
}

#[test]
fn evaluate_division_2() {
    let (output, err) = interpreter::evaluate_content("14 / 3");
    assert!(err.is_none());
    assert_eq!("4.666666666666667", output);
}

#[test]
fn evaluate_multiplication() {
    let (output, err) = interpreter::evaluate_content("4 * 5");
    assert!(err.is_none());
    assert_eq!("20", output);
}

#[test]
fn evaluate_string_concat() {
    let (output, err) = interpreter::evaluate_content("\"hello\" + \" world!\"");
    assert!(err.is_none());
    assert_eq!("hello world!", output);
}

#[test]
fn evaluate_greater_1() {
    let (output, err) = interpreter::evaluate_content("5 > 4");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_greater_2() {
    let (output, err) = interpreter::evaluate_content("3 > 4");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_greater_equal_1() {
    let (output, err) = interpreter::evaluate_content("5 >= 4");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_greater_equal_2() {
    let (output, err) = interpreter::evaluate_content("3 >= 3");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_greater_equal_3() {
    let (output, err) = interpreter::evaluate_content("2 >= 3");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_less_1() {
    let (output, err) = interpreter::evaluate_content("5 < 4");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_less_2() {
    let (output, err) = interpreter::evaluate_content("3 < 4");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_less_equal_1() {
    let (output, err) = interpreter::evaluate_content("5 <= 4");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_less_equal_2() {
    let (output, err) = interpreter::evaluate_content("3 <= 3");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_less_equal_3() {
    let (output, err) = interpreter::evaluate_content("2 <= 3");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_equal_1() {
    let (output, err) = interpreter::evaluate_content("2 == 3");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_equal_2() {
    let (output, err) = interpreter::evaluate_content("5 == 5");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_equal_3() {
    let (output, err) = interpreter::evaluate_content("\"foo\" == 5");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_equal_4() {
    let (output, err) = interpreter::evaluate_content("\"foo\" == \"bar\"");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_equal_5() {
    let (output, err) = interpreter::evaluate_content("\"bar\" == \"bar\"");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_not_equal_1() {
    let (output, err) = interpreter::evaluate_content("2 != 3");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_not_equal_2() {
    let (output, err) = interpreter::evaluate_content("5 != 5");
    assert!(err.is_none());
    assert_eq!("false", output);
}

#[test]
fn evaluate_not_equal_3() {
    let (output, err) = interpreter::evaluate_content("\"foo\"!=\"bar\"");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_not_equal_4() {
    let (output, err) = interpreter::evaluate_content("true != false");
    assert!(err.is_none());
    assert_eq!("true", output);
}

#[test]
fn evaluate_runtime_error_1() {
    let (_, err) = interpreter::evaluate_content("-\"muffin\"");
    assert!(err.is_some());
    assert_eq!("Operand must be a number.", err.unwrap().to_string());
}

#[test]
fn evaluate_runtime_error_2() {
    let (_, err) = interpreter::evaluate_content("3 / \"muffin\"");
    assert!(err.is_some());
    assert_eq!("Operands must be numbers.", err.unwrap().to_string());
}

#[test]
fn evaluate_runtime_error_3() {
    let (_, err) = interpreter::evaluate_content("3 + \"muffin\"");
    assert!(err.is_some());
    assert_eq!(
        "Operands must be two numbers or two strings.",
        err.unwrap().to_string()
    );
}

#[test]
fn evaluate_runtime_error_4() {
    let (_, err) = interpreter::evaluate_content("3 - \"muffin\"");
    assert!(err.is_some());
    assert_eq!(
        "Operands must be two numbers or two strings.",
        err.unwrap().to_string()
    );
}
