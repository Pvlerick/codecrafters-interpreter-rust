// use interpreter_starter_rust::{parser::Parser, scanner::Scanner};
//
// use crate::common::TempFile;
//
// mod common;
//
// #[test]
// fn interpret_addition() {
//     let mut tmp_file = TempFile::with_content("1 + 2");
//     let mut scanner = Scanner::new(tmp_file.reader());
//     let mut parser = Parser::new(scanner.scan_tokens().unwrap().map(|i| i.unwrap()));
//     assert_eq!("3", interpreter.evaluate().unwrap());
// }
