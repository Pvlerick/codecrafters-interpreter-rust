use std::{cell::RefCell, io::BufReader, rc::Rc};

use interpreter_starter_rust::{errors::InterpreterError, interpreter::Interpreter};

use super::reader::StrReader;

#[allow(dead_code)]
pub fn run_content(content: &'static str) -> (String, Option<InterpreterError>) {
    let output = Rc::new(RefCell::new(Vec::new()));
    let mut interpreter =
        Interpreter::build(BufReader::new(StrReader::new(content)), output.clone()).unwrap();

    let res = interpreter.run();
    let output = String::from_utf8_lossy(output.borrow().as_slice()).to_string();
    (output, res.err())
}

#[allow(dead_code)]
pub fn evaluate_content(content: &'static str) -> (String, Option<InterpreterError>) {
    let output = Rc::new(RefCell::new(Vec::new()));
    let mut interpreter =
        Interpreter::build(BufReader::new(StrReader::new(content)), output.clone()).unwrap();

    let res = interpreter.evaluate();
    let output = String::from_utf8_lossy(output.borrow().as_slice()).to_string();
    (output, res.err())
}
