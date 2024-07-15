use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::usize;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            writeln!(io::stderr(), "Logs from your program will appear here!").unwrap();

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            if !file_contents.is_empty() {
                for token in tokenize(file_contents) {
                    println!("{}", token);
                }
                println!("EOF  null");
            } else {
                println!("EOF  null");
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}

fn tokenize(content: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (line_number, line_content) in content.lines().enumerate() {
        let chars = line_content.chars().collect::<Vec<_>>();
        for (char_position, c) in chars.iter().enumerate() {
            let lexeme = chars[char_position..char_position + c.len_utf8()]
                .iter()
                .map(|&i| i as u8)
                .collect::<Vec<_>>();
            match c {
                '(' => tokens.push(Token::left_par(lexeme, line_number)),
                ')' => tokens.push(Token::right_par(lexeme, line_number)),
                _ => {}
            }
        }
    }

    tokens
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<String>,
    line: usize,
}

impl Token {
    fn left_par(lexeme: Vec<u8>, line: usize) -> Token {
        Token {
            token_type: TokenType::LeftParenthesis,
            lexeme: String::from_utf8(lexeme).unwrap(),
            literal: None,
            line,
        }
    }

    fn right_par(lexeme: Vec<u8>, line: usize) -> Token {
        Token {
            token_type: TokenType::RightParenthesis,
            lexeme: String::from_utf8(lexeme).unwrap(),
            literal: None,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} null", self.token_type, self.lexeme)
    }
}

#[derive(Debug)]
enum TokenType {
    LeftParenthesis,
    RightParenthesis,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TokenType::LeftParenthesis => write!(f, "LEFT_PAREN"),
            TokenType::RightParenthesis => write!(f, "RIGHT_PAREN"),
        }
    }
}
