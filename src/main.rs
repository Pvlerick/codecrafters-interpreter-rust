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
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            let mut has_error = false;
            if !file_contents.is_empty() {
                for token in tokenize(file_contents) {
                    if token.token_type == TokenType::Unknown {
                        has_error = true;
                    }

                    println!("{}", token);
                }
            }

            println!("EOF  null");

            if has_error {
                std::process::exit(65);
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
        let line_number = line_number + 1;
        for (char_position, c) in chars.iter().enumerate() {
            let lexeme = chars[char_position..char_position + c.len_utf8()]
                .iter()
                .map(|&i| i as u8)
                .collect::<Vec<_>>();
            match c {
                '(' => tokens.push(Token::new(TokenType::LeftParenthesis, lexeme, line_number)),
                ')' => tokens.push(Token::new(TokenType::RightParenthesis, lexeme, line_number)),
                '{' => tokens.push(Token::new(TokenType::LeftBrace, lexeme, line_number)),
                '}' => tokens.push(Token::new(TokenType::RightBrace, lexeme, line_number)),
                ',' => tokens.push(Token::new(TokenType::Comma, lexeme, line_number)),
                '.' => tokens.push(Token::new(TokenType::Dot, lexeme, line_number)),
                '-' => tokens.push(Token::new(TokenType::Minus, lexeme, line_number)),
                '+' => tokens.push(Token::new(TokenType::Plus, lexeme, line_number)),
                ';' => tokens.push(Token::new(TokenType::Semicolon, lexeme, line_number)),
                '*' => tokens.push(Token::new(TokenType::Star, lexeme, line_number)),
                ' ' => {}
                _ => tokens.push(Token::new(TokenType::Unknown, lexeme, line_number)),
            }
        }
    }

    tokens
}

struct Token {
    token_type: TokenType,
    lexeme: String,
    _literal: Option<String>,
    line: usize,
}

impl Token {
    fn new(token_type: TokenType, lexeme: Vec<u8>, line: usize) -> Self {
        Token {
            token_type,
            lexeme: String::from_utf8(lexeme).unwrap(),
            _literal: None,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.token_type {
            TokenType::Unknown => write!(
                f,
                "[line {}] Error: Unexpected character: {}",
                self.line, self.lexeme
            ),
            _ => write!(f, "{} {} null", self.token_type, self.lexeme),
        }
    }
}

#[derive(Debug, PartialEq)]
enum TokenType {
    Unknown,
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TokenType::Unknown => Ok(()),
            TokenType::LeftParenthesis => write!(f, "LEFT_PAREN"),
            TokenType::RightParenthesis => write!(f, "RIGHT_PAREN"),
            TokenType::LeftBrace => write!(f, "LEFT_BRACE"),
            TokenType::RightBrace => write!(f, "RIGHT_BRACE"),
            TokenType::Comma => write!(f, "COMMA"),
            TokenType::Dot => write!(f, "DOT"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Semicolon => write!(f, "SEMICOLON"),
            TokenType::Star => write!(f, "STAR"),
        }
    }
}
