use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::iter::Peekable;
use std::str::Chars;
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
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            if !file_contents.is_empty() {
                let content = file_contents.leak();

                let has_errors = tokenize(content);

                if has_errors {
                    std::process::exit(65);
                }
            }
        }
        "repl_tokenize" => loop {
            print!("> ");
            io::stdout().flush().expect("cannot flush stdout");

            let mut buf = String::new();
            let _ = io::stdin()
                .read_line(&mut buf)
                .expect("cannot read REPL line");

            let _ = tokenize(buf.leak());
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(64);
        }
    }
}

fn tokenize<'a>(content: &'a str) -> bool {
    let mut has_errors = false;
    let scanner = Scanner::new(content);

    let tokens = scanner.scan_tokens().collect::<Vec<Token<'a>>>();

    for token in tokens {
        match token.token_type {
            TokenType::Unknown => {
                has_errors = true;
                eprintln!("{}", token);
            }
            _ => println!("{}", token),
        }
    }

    has_errors
}

struct Scanner<'a> {
    content: &'a str,
}

impl<'a> Scanner<'a> {
    fn new(content: &'a str) -> Self {
        Scanner { content }
    }

    fn scan_tokens(&self) -> impl Iterator<Item = Token<'a>> {
        TokensIterator::new(self.content)
    }
}

struct TokensIterator<'a> {
    content: Peekable<Chars<'a>>,
    line: usize,
    is_in_string: bool,
    is_in_line_comment: bool,
    has_reached_eof: bool,
}

impl<'a> TokensIterator<'a> {
    fn new(content: &'a str) -> Self {
        TokensIterator {
            content: content.chars().peekable(),
            line: 1,
            is_in_string: false,
            is_in_line_comment: false,
            has_reached_eof: false,
        }
    }
}

impl<'a> Iterator for TokensIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;

        if self.has_reached_eof {
            return None;
        }

        loop {
            let c = self.content.next();

            if c.is_none() {
                self.has_reached_eof = true;
                return Some(Token::new(EOF, "", self.line));
            }

            let c = c.unwrap();

            match c {
                '(' => return Some(Token::new(LeftParenthesis, "(", self.line)),
                ')' => return Some(Token::new(RightParenthesis, ")", self.line)),
                '{' => return Some(Token::new(LeftBrace, "{", self.line)),
                '}' => return Some(Token::new(RightBrace, "}", self.line)),
                ',' => return Some(Token::new(Comma, ",", self.line)),
                '.' => return Some(Token::new(Dot, ".", self.line)),
                '-' => return Some(Token::new(Minus, "-", self.line)),
                '+' => return Some(Token::new(Plus, "+", self.line)),
                ';' => return Some(Token::new(Semicolon, ";", self.line)),
                '*' => return Some(Token::new(Star, "*", self.line)),
                '=' if self.content.next_if_eq(&&'=').is_some() => {
                    return Some(Token::new(EqualEqual, "==", self.line))
                }
                '=' => return Some(Token::new(Equal, "=", self.line)),
                '!' if self.content.next_if_eq(&&'=').is_some() => {
                    return Some(Token::new(BangEqual, "!=", self.line))
                }
                '!' => return Some(Token::new(Bang, "!", self.line)),
                '<' if self.content.next_if_eq(&&'=').is_some() => {
                    return Some(Token::new(LessEqual, "<=", self.line))
                }
                '<' => return Some(Token::new(Less, "<", self.line)),
                '>' if self.content.next_if_eq(&&'=').is_some() => {
                    return Some(Token::new(GreaterEqual, ">=", self.line))
                }
                '>' => return Some(Token::new(Greater, ">", self.line)),
                '/' if self.content.next_if_eq(&&'/').is_some() => {
                    self.is_in_line_comment = true;
                }
                '/' => return Some(Token::new(Slash, "/", self.line)),
                '\n' => {
                    self.is_in_line_comment = false;
                    self.line += 1;
                }
                ' ' | '\r' | '\t' => {}
                _ => {
                    return Some(Token::new(Unknown, c.to_string().leak(), self.line));
                }
            }
        }
    }
}

struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    _literal: Option<&'a str>,
    line: usize,
}

impl<'a> Token<'a> {
    fn new(token_type: TokenType, lexeme: &'a str, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            _literal: None,
            line,
        }
    }
}

impl Display for Token<'_> {
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
    EOF,
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
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Slash,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TokenType::Unknown => Ok(()),
            TokenType::EOF => write!(f, "EOF"),
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
            TokenType::Less => write!(f, "LESS"),
            TokenType::Greater => write!(f, "GREATER"),
            TokenType::GreaterEqual => write!(f, "GREATER_EQUAL"),
            TokenType::Slash => write!(f, "SLASH"),
        }
    }
}
