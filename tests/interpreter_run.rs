use interpreter_starter_rust::interpreter::Interpreter;

use crate::common::TempFile;

mod common;

#[test]
fn run_print_string() {
    let mut tmp_file = TempFile::with_content("print \"foo\"");
    let interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output);
    assert!(res.is_ok());
    assert_eq!(output, b"foo");
}