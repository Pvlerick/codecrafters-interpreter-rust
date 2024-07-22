use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::iter::{Enumerate, Peekable};
use std::str::Chars;
use std::usize;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = args.get(1).and_then(|i| Some(i.as_str()));

    match command {
        None => run_prompt(),
        Some("tokenize") => {
            if args.len() < 3 {
                writeln!(io::stderr(), "Usage: {} tokenize <file_path>", args[0]).unwrap();
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

fn run_file(file_path: &str) {
    let file_contents = fs::read_to_string(file_path).unwrap_or_else(|_| {
        eprintln!("Failed to read file {}", file_path);
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

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("cannot flush stdout");

        let mut buf = String::new();
        let _ = io::stdin()
            .read_line(&mut buf)
            .expect("cannot read REPL line");

        let _ = tokenize(&buf);
    }
}

fn tokenize<'a>(content: &'a str) -> bool {
    let mut has_errors = false;
    let scanner = Scanner::new(content);

    let tokens = scanner
        .scan_tokens()
        .collect::<Vec<Result<Token<'a>, String>>>();

    for token in tokens {
        match token {
            Ok(token) => println!("{}", token),
            Err(message) => {
                has_errors = true;
                eprintln!("{}", message);
            }
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

    fn scan_tokens(&self) -> impl Iterator<Item = Result<Token<'a>, String>> {
        TokensIterator::new(self.content)
    }
}

struct TokensIterator<'a> {
    content: &'a str,
    content_iterator: Peekable<Chars<'a>>,
    position: usize,
    line: usize,
    has_reached_eof: bool,
}

impl<'a> TokensIterator<'a> {
    fn new(content: &'a str) -> Self {
        TokensIterator {
            content,
            content_iterator: content.chars().peekable(),
            position: 0,
            line: 1,
            has_reached_eof: false,
        }
    }

    fn next(&mut self) -> Option<char> {
        let item = self.content_iterator.next();

        match item {
            None => {
                self.has_reached_eof = true;
                return None;
            }
            Some(c) => {
                if c == '\n' {
                    self.line += 1;
                }
                self.position += 1;
                return Some(c);
            }
        }
    }

    fn next_is(&mut self, sought: char) -> bool {
        match self.content_iterator.peek() {
            Some(&c) if c == sought => {
                self.next();
                true
            }
            _ => false,
        }
    }

    fn advance_until(&mut self, stop: char) {
        loop {
            let item = self.next();
            match item {
                Some(c) if c == stop => return,
                None => return,
                _ => {}
            }
        }
    }
}

impl<'a> Iterator for TokensIterator<'a> {
    type Item = Result<Token<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;

        if self.has_reached_eof {
            return None;
        }

        loop {
            let item = self.next();

            if item.is_none() {
                self.has_reached_eof = true;
                return Some(Ok(Token::new(EOF, "")));
            }

            let character = item.unwrap();

            match character {
                '(' => return Some(Ok(Token::new(LeftParenthesis, "("))),
                ')' => return Some(Ok(Token::new(RightParenthesis, ")"))),
                '{' => return Some(Ok(Token::new(LeftBrace, "{"))),
                '}' => return Some(Ok(Token::new(RightBrace, "}"))),
                ',' => return Some(Ok(Token::new(Comma, ","))),
                '.' => return Some(Ok(Token::new(Dot, "."))),
                '-' => return Some(Ok(Token::new(Minus, "-"))),
                '+' => return Some(Ok(Token::new(Plus, "+"))),
                ';' => return Some(Ok(Token::new(Semicolon, ";"))),
                '*' => return Some(Ok(Token::new(Star, "*"))),
                '=' if self.next_is('=') => return Some(Ok(Token::new(EqualEqual, "=="))),
                '=' => return Some(Ok(Token::new(Equal, "="))),
                '!' if self.next_is('=') => return Some(Ok(Token::new(BangEqual, "!="))),
                '!' => return Some(Ok(Token::new(Bang, "!"))),
                '<' if self.next_is('=') => return Some(Ok(Token::new(LessEqual, "<="))),
                '<' => return Some(Ok(Token::new(Less, "<"))),
                '>' if self.next_is('=') => return Some(Ok(Token::new(GreaterEqual, ">="))),
                '>' => return Some(Ok(Token::new(Greater, ">"))),
                '/' if self.next_is('/') => {
                    self.advance_until('\n');
                }
                '/' => return Some(Ok(Token::new(Slash, "/"))),
                '"' => {
                    let start = self.position;
                    let mut end = start;
                    loop {
                        let item = self.next();
                        match item {
                            Some(c) if c == '"' => break,
                            Some(_) => end += 1,
                            None => {
                                return Some(Err(format!(
                                    "[line {}] Error: Unterminated string.",
                                    self.line
                                )))
                            }
                        }
                    }
                    return Some(Ok(Token::with_literal(
                        String,
                        &self.content[start - 1..=end],
                        &self.content[start..end],
                    )));
                }
                ' ' | '\r' | '\n' | '\t' => {}
                _ => {
                    return Some(Err(format!(
                        "[line {}] Error: Unexpected character: {}",
                        self.line, character
                    )))
                }
            }
        }
    }
}

struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    literal: Option<&'a str>,
}

impl<'a> Token<'a> {
    fn new(token_type: TokenType, lexeme: &'a str) -> Self {
        Token {
            token_type,
            lexeme,
            literal: None,
        }
    }

    fn with_literal(token_type: TokenType, lexeme: &'a str, literal: &'a str) -> Self {
        Token {
            token_type,
            lexeme,
            literal: Some(literal),
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.token_type,
            self.lexeme,
            self.literal.unwrap_or("null")
        )
    }
}

#[derive(PartialEq)]
enum TokenType {
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
    String,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
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
            TokenType::String => write!(f, "STRING"),
        }
    }
}
