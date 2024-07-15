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
                    match token.token_type {
                        TokenType::Unknown => {
                            has_error = true;
                            eprintln!("{}", token);
                        }
                        _ => println!("{}", token),
                    }
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
        let line_number = line_number + 1;
        let chars = line_content.chars().collect::<Vec<_>>();
        let mut chars_iterator = chars.iter().peekable();
        let mut item = chars_iterator.next();

        while item.is_some() {
            let c = item.unwrap();

            use TokenType::*;
            match c {
                '(' => tokens.push(Token::new(LeftParenthesis, "(", line_number)),
                ')' => tokens.push(Token::new(RightParenthesis, ")", line_number)),
                '{' => tokens.push(Token::new(LeftBrace, "{", line_number)),
                '}' => tokens.push(Token::new(RightBrace, "}", line_number)),
                ',' => tokens.push(Token::new(Comma, ",", line_number)),
                '.' => tokens.push(Token::new(Dot, ".", line_number)),
                '-' => tokens.push(Token::new(Minus, "-", line_number)),
                '+' => tokens.push(Token::new(Plus, "+", line_number)),
                ';' => tokens.push(Token::new(Semicolon, ";", line_number)),
                '*' => tokens.push(Token::new(Star, "*", line_number)),
                '=' if chars_iterator.next_if(|&&i| i == '=').is_some() => {
                    tokens.push(Token::new(EqualEqual, "==", line_number))
                }
                '=' => tokens.push(Token::new(Equal, "=", line_number)),
                '!' if chars_iterator.next_if(|&&i| i == '=').is_some() => {
                    tokens.push(Token::new(BangEqual, "!=", line_number))
                }
                '!' => tokens.push(Token::new(Bang, "!", line_number)),
                '<' if chars_iterator.next_if(|&&i| i == '=').is_some() => {
                    tokens.push(Token::new(LessEqual, "<=", line_number))
                }
                '>' if chars_iterator.next_if(|&&i| i == '=').is_some() => {
                    tokens.push(Token::new(GreaterEqual, ">=", line_number))
                }
                ' ' => {}
                _ => tokens.push(Token::new(Unknown, c.to_string(), line_number)),
            }

            item = chars_iterator.next();
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
    fn new<S: AsRef<str>>(token_type: TokenType, lexeme: S, line: usize) -> Self {
        Token {
            token_type,
            lexeme: lexeme.as_ref().to_string(),
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

#[derive(PartialEq)]
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
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    LessEqual,
    GreaterEqual,
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
            TokenType::Equal => write!(f, "EQUAL"),
            TokenType::EqualEqual => write!(f, "EQUAL_EQUAL"),
            TokenType::Bang => write!(f, "BANG"),
            TokenType::BangEqual => write!(f, "BANG_EQUAL"),
            TokenType::LessEqual => write!(f, "LESS_EQUAL"),
            TokenType::GreaterEqual => write!(f, "GREATER_EQUAL"),
        }
    }
}
