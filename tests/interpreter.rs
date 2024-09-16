use interpreter_starter_rust::{interpreter::Interpreter, parser::Parser, scanner::Scanner};

use crate::common::TempFile;

mod common;

#[test]
fn evaluate_literal_string() {
    let mut tmp_file = TempFile::with_content("\"foo\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("foo", res.unwrap());
}

#[test]
fn evaluate_literal_numeric() {
    let mut tmp_file = TempFile::with_content("42");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("42", res.unwrap());
}

#[test]
fn evaluate_grouping_literal() {
    let mut tmp_file = TempFile::with_content("(42)");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("42", res.unwrap());
}

#[test]
fn evaluate_unary() {
    let mut tmp_file = TempFile::with_content("-42");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("-42", res.unwrap());
}

#[test]
fn evaluate_true() {
    let mut tmp_file = TempFile::with_content("true");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_false() {
    let mut tmp_file = TempFile::with_content("false");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_nil() {
    let mut tmp_file = TempFile::with_content("nil");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("nil", res.unwrap());
}

#[test]
fn evaluate_not_1() {
    let mut tmp_file = TempFile::with_content("!false");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_not_2() {
    let mut tmp_file = TempFile::with_content("!!false");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_not_3() {
    let mut tmp_file = TempFile::with_content("!!42");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_addition() {
    let mut tmp_file = TempFile::with_content("1 + 2");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("3", res.unwrap());
}

#[test]
fn evaluate_substraction() {
    let mut tmp_file = TempFile::with_content("5 - 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("2", res.unwrap());
}

#[test]
fn evaluate_division_1() {
    let mut tmp_file = TempFile::with_content("20 / 2");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("10", res.unwrap());
}

#[test]
fn evaluate_division_2() {
    let mut tmp_file = TempFile::with_content("14 / 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("4.666666666666667", res.unwrap());
}

#[test]
fn evaluate_multiplication() {
    let mut tmp_file = TempFile::with_content("4 * 5");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("20", res.unwrap());
}

#[test]
fn evaluate_string_concat() {
    let mut tmp_file = TempFile::with_content("\"hello\" + \" world!\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("hello world!", res.unwrap());
}

#[test]
fn evaluate_greater_1() {
    let mut tmp_file = TempFile::with_content("5 > 4");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_greater_2() {
    let mut tmp_file = TempFile::with_content("3 > 4");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_greater_equal_1() {
    let mut tmp_file = TempFile::with_content("5 >= 4");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_greater_equal_2() {
    let mut tmp_file = TempFile::with_content("3 >= 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_greater_equal_3() {
    let mut tmp_file = TempFile::with_content("2 >= 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_less_1() {
    let mut tmp_file = TempFile::with_content("5 < 4");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_less_2() {
    let mut tmp_file = TempFile::with_content("3 < 4");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_less_equal_1() {
    let mut tmp_file = TempFile::with_content("5 <= 4");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_less_equal_2() {
    let mut tmp_file = TempFile::with_content("3 <= 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_less_equal_3() {
    let mut tmp_file = TempFile::with_content("2 <= 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_equal_1() {
    let mut tmp_file = TempFile::with_content("2 == 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_equal_2() {
    let mut tmp_file = TempFile::with_content("5 == 5");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_equal_3() {
    let mut tmp_file = TempFile::with_content("\"foo\" == 5");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_equal_4() {
    let mut tmp_file = TempFile::with_content("\"foo\" == \"bar\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_equal_5() {
    let mut tmp_file = TempFile::with_content("\"bar\" == \"bar\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_not_equal_1() {
    let mut tmp_file = TempFile::with_content("2 != 3");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("true", res.unwrap());
}

#[test]
fn evaluate_not_equal_2() {
    let mut tmp_file = TempFile::with_content("5 != 5");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_ok());
    assert_eq!("false", res.unwrap());
}

#[test]
fn evaluate_runtime_error_1() {
    let mut tmp_file = TempFile::with_content("-\"muffin\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_err());
    assert_eq!("Operand must be a number.", res.unwrap_err().to_string());
}

#[test]
fn evaluate_runtime_error_2() {
    let mut tmp_file = TempFile::with_content("3 / \"muffin\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_err());
    assert_eq!("Operands must be numbers.", res.unwrap_err().to_string());
}

#[test]
fn evaluate_runtime_error_3() {
    let mut tmp_file = TempFile::with_content("3 + \"muffin\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_err());
    assert_eq!(
        "Operands must be two numbers or two strings.",
        res.unwrap_err().to_string()
    );
}

#[test]
fn evaluate_runtime_error_4() {
    let mut tmp_file = TempFile::with_content("3 - \"muffin\"");
    let mut scanner = Scanner::new(tmp_file.reader());
    let parser = Parser::new(scanner.scan_tokens().unwrap());
    let interpreter = Interpreter::new(parser.parse().unwrap());
    let res = interpreter.evaluate();
    assert!(res.is_err());
    assert_eq!(
        "Operands must be two numbers or two strings.",
        res.unwrap_err().to_string()
    );
}
