use std::io::sink;

use interpreter_starter_rust::interpreter::Interpreter;

use crate::common::TempFile;

mod common;

#[test]
fn run_print_string() {
    let mut tmp_file = TempFile::with_content("print \"foo\";");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!("foo\n", String::from_utf8_lossy(&output));
}

#[test]
fn run_print_true() {
    let mut tmp_file = TempFile::with_content("print true;");
    let mut interpreter = Interpreter::build(tmp_file.reader()).unwrap();
    let mut output = Vec::new();
    let res = interpreter.run(&mut output, &mut sink());
    assert!(res.is_ok());
    assert_eq!("true\n", String::from_utf8_lossy(&output));
}
