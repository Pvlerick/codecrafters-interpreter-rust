use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::usize;

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

struct Scanner<R>
where
    R: BufRead + 'static,
{
    reader: Option<R>,
}

impl<R> Scanner<R>
where
    R: BufRead + 'static,
{
    fn new(reader: R) -> Self {
        Scanner {
            reader: Some(reader),
        }
    }

    fn scan_tokens(
        &mut self,
    ) -> Result<impl Iterator<Item = Result<Token, String>>, Box<dyn Error>> {
        match self.reader.take() {
            Some(reader) => Ok(TokensIterator::new(reader)),
            None => Err(format!("Scanner's reader has already been consumed").into()),
        }
    }
}

struct TokensIterator {
    has_reached_eof: bool,
    content: Box<dyn Iterator<Item = char>>,
    buffer: VecDeque<Option<char>>,
    line: usize,
}

static KEYWORDS: &[(&str, TokenType)] = &[
    ("and", TokenType::And),
    ("class", TokenType::Class),
    ("else", TokenType::Else),
    ("false", TokenType::False),
    ("for", TokenType::For),
    ("fun", TokenType::Fun),
    ("if", TokenType::If),
    ("nil", TokenType::Nil),
    ("or", TokenType::Or),
    ("print", TokenType::Print),
    ("return", TokenType::Return),
    ("super", TokenType::Super),
    ("this", TokenType::This),
    ("true", TokenType::True),
    ("var", TokenType::Var),
    ("while", TokenType::While),
];

impl TokensIterator {
    fn new<R>(reader: R) -> Self
    where
        R: BufRead + 'static,
    {
        const BUFFER_SIZE: usize = 3;

        let mut content = reader.lines().flat_map(|i| {
            i.expect("can't read file content")
                .chars()
                .chain(Some('\n'))
                .collect::<Vec<_>>()
        });

        let mut buffer = VecDeque::with_capacity(BUFFER_SIZE);
        for _ in 0..BUFFER_SIZE {
            buffer.push_back(content.next());
        }

        TokensIterator {
            has_reached_eof: false,
            content: Box::new(content),
            buffer,
            line: 1,
        }
    }

    fn next(&mut self) -> Option<char> {
        if self.has_reached_eof {
            return None;
        }

        let item = self.buffer.pop_front().unwrap_or(None);

        match item {
            None => {
                self.has_reached_eof = true;
                return None;
            }
            Some(c) => {
                if c == '\n' {
                    self.line += 1;
                }
                self.buffer.push_back(self.content.next());
                return Some(c);
            }
        }
    }

    fn peek(&self) -> Option<&char> {
        self.buffer[0].as_ref()
    }

    fn peek_peek(&self) -> Option<&char> {
        self.buffer[1].as_ref()
    }

    fn peek_matches(&self, condition: fn(char) -> bool) -> bool {
        match self.peek() {
            Some(&c) => condition(c),
            _ => false,
        }
    }

    fn peek_peek_matches(&self, condition: fn(char) -> bool) -> bool {
        match self.peek_peek() {
            Some(&c) => condition(c),
            _ => false,
        }
    }

    fn next_is(&mut self, sought: char) -> bool {
        match self.buffer[0].as_ref() {
            Some(&c) if c == sought => {
                let _ = self.next();
                true
            }
            _ => false,
        }
    }

    fn advance_while(&mut self, condition: fn(char) -> bool, buf: &mut String) -> bool {
        loop {
            if self.peek().is_none() {
                return false;
            }

            if self.peek().is_some_and(|i| !condition(*i)) {
                return true;
            }

            match self.next() {
                Some(c) => buf.push(c),
                None => {}
            }
        }
    }

    fn handle_line_comment(&mut self) {
        self.advance_while(|i| i != '\n', &mut String::new());
    }

    fn handle_string(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let mut buf = "\"".to_string();
        if self.advance_while(|i| i != '"', &mut buf) && self.next_is('"') {
            buf.push('"');
            return Ok(Token::with_literal(
                TokenType::String,
                buf[1..buf.len() - 1].to_string(),
                Literal::String(buf),
            ));
        } else {
            return Err(format!("[line {}] Error: Unterminated string.", start_line));
        }
    }

    fn handle_digit(&mut self, initial_digit: char) -> Result<Token, String> {
        let mut buf = initial_digit.to_string();
        self.advance_while(|i| i.is_digit(10), &mut buf);
        if self.peek_matches(|i| i == '.') && self.peek_peek_matches(|i| i.is_digit(10)) {
            buf.push(self.next().unwrap());
            self.advance_while(|i| i.is_digit(10), &mut buf);
        }
        let value: f64 = buf.parse().expect("cannot parse f64");
        return Ok(Token::with_literal(
            TokenType::Number,
            buf,
            Literal::Digit(value),
        ));
    }

    fn handle_identifier_or_keyword(&mut self) -> Result<Token, String> {
        let mut buf = String::new();
        self.advance_while(|i| i.is_alphanumeric() || i == '_', &mut buf);
        return match TokensIterator::k(buf.as_str()) {
            Some(token_type) => Ok(Token::new(token_type, buf)),
            None => Ok(Token::new(TokenType::Identifier, buf)),
        };
    }

    fn k(identifier: &str) -> Option<TokenType> {
        KEYWORDS
            .binary_search_by(|(k, _)| k.cmp(&identifier))
            .map(|i| KEYWORDS[i].1)
            .ok()
    }
}

impl Iterator for TokensIterator {
    type Item = Result<Token, String>;

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
                '/' if self.next_is('/') => self.handle_line_comment(),
                '/' => return Some(Ok(Token::new(Slash, "/"))),
                '"' => return Some(self.handle_string()),
                c if c.is_digit(10) => return Some(self.handle_digit(c)),
                ' ' | '\r' | '\n' | '\t' => {}
                c if c.is_alphanumeric() || c == '_' => {
                    return Some(self.handle_identifier_or_keyword())
                }
                _ => {
                    return Some(Err(format!(
                        "[line {}] Error: Unexpected character: {}",
                        self.line, character
                    )))
                }
            };
        }
    }
}

struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
}

impl Token {
    fn new<S: ToString>(token_type: TokenType, lexeme: S) -> Self {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            literal: None,
        }
    }

    fn with_literal(token_type: TokenType, lexeme: String, literal: Literal) -> Self {
        Token {
            token_type,
            lexeme,
            literal: Some(literal),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let literal = self
            .literal
            .as_ref()
            .map_or_else(|| "null".to_string(), |i| format!("{}", i));

        write!(f, "{} {} {}", self.token_type, self.lexeme, literal)
    }
}

#[derive(PartialEq, Clone, Copy)]
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
    Number,
    Identifier,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
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
            TokenType::Number => write!(f, "NUMBER"),
            TokenType::Identifier => write!(f, "IDENTIFIER"),
            TokenType::And => write!(f, "AND"),
            TokenType::Class => write!(f, "CLASS"),
            TokenType::Else => write!(f, "ELSE"),
            TokenType::False => write!(f, "FALSE"),
            TokenType::For => write!(f, "FOR"),
            TokenType::Fun => write!(f, "FUN"),
            TokenType::If => write!(f, "IF"),
            TokenType::Nil => write!(f, "NIL"),
            TokenType::Or => write!(f, "OR"),
            TokenType::Print => write!(f, "PRINT"),
            TokenType::Return => write!(f, "RETURN"),
            TokenType::Super => write!(f, "SUPER"),
            TokenType::This => write!(f, "THIS"),
            TokenType::True => write!(f, "TRUE"),
            TokenType::Var => write!(f, "VAR"),
            TokenType::While => write!(f, "WHILE"),
        }
    }
}

enum Literal {
    String(String),
    Digit(f64),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(value) => write!(f, "{}", value),
            Literal::Digit(value) => {
                return if value.fract() == 0.0 {
                    write!(f, "{}.0", value)
                } else {
                    write!(f, "{}", value)
                };
            }
        }
    }
}
