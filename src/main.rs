use std::env;
use std::fs::File;
use std::io::{self, stdout, BufReader, Write};

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

use crate::errors::InterpreterError;

pub mod environment;
pub mod errors;
pub mod interpreter;
pub mod parser;
pub mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = args.get(1).and_then(|i| Some(i.as_str()));

    match command {
        None => tokenize_repl(),
        Some("tokenize") => {
            if args.len() < 3 {
                writeln!(io::stderr(), "Usage: {} tokenize <file_path>", args[0]).unwrap();
                return;
            }

            tokenize_file(&args[2]);
        }
        Some("parse") => {
            if args.len() < 3 {
                writeln!(io::stderr(), "Usage: {} parse <file_path>", args[0]).unwrap();
                return;
            }

            parse_file(&args[2]);
        }
        Some("evaluate") => {
            if args.len() < 3 {
                writeln!(io::stderr(), "Usage: {} evaluate <file_path>", args[0]).unwrap();
                return;
            }

            evaluate_file(&args[2]);
        }
        Some("run") => {
            if args.len() < 3 {
                writeln!(io::stderr(), "Usage: {} evaluate <file_path>", args[0]).unwrap();
                return;
            }

            run_file(&args[2]);
        }
        Some(command) => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(64);
        }
    }
}

fn tokenize_file(file_path: &str) {
    let mut has_errors = false;

    let file = File::open(file_path).expect(format!("cannot open file {}", file_path).as_str());

    let scanner = Scanner::new(BufReader::new(file));
    for item in scanner.scan().expect("failed to scan tokens") {
        match item {
            Ok(token) => println!("{}", token),
            Err(message) => {
                has_errors = true;
                eprintln!("{}", message);
            }
        }
    }

    if has_errors {
        std::process::exit(65);
    }
}

fn tokenize_repl() {
    loop {
        print!("> ");
        io::stdout().flush().expect("cannot flush stdout");

        let mut buf = String::new();
        let _ = io::stdin()
            .read_line(&mut buf)
            .expect("cannot read REPL line");

        let scanner = Scanner::new(BufReader::new(buf.leak().as_bytes()));
        for item in scanner.scan().expect("failed to scan tokens") {
            match item {
                Ok(token) => println!("{}", token),
                Err(message) => {
                    eprintln!("{}", message);
                }
            }
        }
    }
}

fn parse_file(file_path: &str) {
    let file = File::open(file_path).expect(format!("cannot open file {}", file_path).as_str());

    let scanner = Scanner::new(BufReader::new(file));
    let tokens = scanner.scan();

    match tokens {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse_expression() {
                Ok(Some(expr)) => {
                    println!("{}", expr);
                }
                Ok(None) => {}
                Err(error) => {
                    eprintln!("{}", error);
                    std::process::exit(65);
                }
            }

            if let Some(errors) = parser.errors() {
                eprintln!("{}", errors);
                std::process::exit(65);
            }
        }
        Err(error) => println!("{}", error),
    }
}

fn evaluate_file(file_path: &str) {
    let file = File::open(file_path).expect(format!("cannot open file {}", file_path).as_str());

    let scanner = Scanner::new(BufReader::new(file));
    let tokens = scanner.scan();

    match tokens {
        Ok(tokens) => {
            let parser = Parser::new(tokens);
            let mut interpreter = Interpreter::new(parser);

            match interpreter.evaluate(&mut stdout()) {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("{}", error);
                    std::process::exit(70);
                }
            }

            if interpreter.has_parsing_errors {
                std::process::exit(65);
            }
        }
        Err(error) => println!("{}", error),
    }
}

fn run_file(file_path: &str) {
    let file = File::open(file_path).expect(format!("cannot open file {}", file_path).as_str());

    let scanner = Scanner::new(BufReader::new(file));
    let tokens = scanner.scan();

    match tokens {
        Ok(tokens) => {
            let parser = Parser::new(tokens);
            let mut interpreter = Interpreter::new(parser);

            match interpreter.run(&mut stdout()) {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("{}", error);
                    match error {
                        InterpreterError::ScanningError(_)
                        | InterpreterError::ScanningErrors(_) => std::process::exit(70),
                        InterpreterError::ParsingErrors(_)
                        | InterpreterError::InterpreterError(_)
                        | InterpreterError::RuntimeError(_) => std::process::exit(65),
                    }
                }
            }
        }
        Err(error) => println!("{}", error),
    }
}
