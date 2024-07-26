use std::env;
use std::fs::File;
use std::io::{self, BufReader, Write};

use scanner::Scanner;

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
    for item in scanner.scan_tokens().unwrap() {
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
        for item in scanner.scan_tokens().unwrap() {
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
    let _tokens = scanner.scan_tokens();
}
