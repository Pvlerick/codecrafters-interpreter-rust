use std::{
    error::Error,
    io::{stderr, BufReader},
};

use interpreter_starter_rust::interpreter::Interpreter;

use super::reader::StrReader;

#[allow(dead_code)]
pub fn run_content(content: &'static str) -> (String, Option<Box<dyn Error>>) {
    let mut interpreter = Interpreter::build(BufReader::new(StrReader::new(content))).unwrap();
    let mut output = Vec::new();

    let res = interpreter.run(&mut output, &mut stderr());
    (String::from_utf8_lossy(&output).to_string(), res.err())
}

#[allow(dead_code)]
pub fn evaluate_content(content: &'static str) -> (String, Option<Box<dyn Error>>) {
    let mut interpreter = Interpreter::build(BufReader::new(StrReader::new(content))).unwrap();
    let mut output = Vec::new();

    let res = interpreter.evaluate(&mut output, &mut stderr());
    (String::from_utf8_lossy(&output).to_string(), res.err())
}
