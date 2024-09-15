use std::env;
use std::fs::File;
use std::io::{self, BufReader, Write};

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

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
        Some(command) => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(64);
        }
    }
}

fn tokenize_file(file_path: &str) {
    let mut has_errors = false;

    let file = File::open(file_path).expect(format!("cannot open file {}", file_path).as_str());

    let mut scanner = Scanner::new(BufReader::new(file));
    for item in scanner.scan_tokens().expect("failed to scan tokens") {
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

        let mut scanner = Scanner::new(BufReader::new(buf.leak().as_bytes()));
        for item in scanner.scan_tokens().expect("failed to scan tokens") {
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

    let mut scanner = Scanner::new(BufReader::new(file));
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().filter_map(|i| i.ok()));

    match parser.parse() {
        Ok(expr) => println!("{}", expr),
        Err(errors) => {
            for e in errors {
                println!("{}", e);
            }
            std::process::exit(65);
        }
    }
}

fn evaluate_file(file_path: &str) {
    let file = File::open(file_path).expect(format!("cannot open file {}", file_path).as_str());

    let mut scanner = Scanner::new(BufReader::new(file));
    let mut parser = Parser::new(scanner.scan_tokens().unwrap().filter_map(|i| i.ok()));
    let interpreter = Interpreter::new(parser.parse().unwrap());
    match interpreter.evaluate() {
        Ok(res) => println!("{}", res),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(70);
        }
    }
}
