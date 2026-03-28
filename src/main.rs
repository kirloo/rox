use std::env;

use std::fmt::Formatter;
use std::sync::atomic::{AtomicBool, Ordering};

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rox [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        runFile(&args[1]);
    } else {
        runPrompt();
    }
}

fn runFile(script_file: &str) {
    let script = std::fs::read_to_string(script_file).unwrap();
    run(&script);
    if HAD_ERROR.load(Ordering::Relaxed) {
        std::process::exit(65);
    }
}

fn runPrompt() {
    let stdin = std::io::stdin();
    let mut buf = String::new();

    loop {
        buf.clear();
        println!("> ");
        let bytes_read = stdin.read_line(&mut buf).unwrap();
        if bytes_read == 0 {
            break;
        }
        run(&buf);
    }
}

fn run(script: &str) {
    let mut scanner = Scanner::new(script);

    let tokens: Vec<Token> = scanner.scan_tokens();

    for t in tokens {
        println!("{}", t)
    }
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, error_where: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, error_where, message);
    HAD_ERROR.store(true, Ordering::Relaxed);
}

struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Clone)]
struct Token {
    token_type: TokenType,
    lexeme: Vec<char>,
    literal: usize,
    line: usize,
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '_'
}

fn is_alphanumeric(c: char) -> bool {
    is_digit(c) || is_alpha(c)
}

fn keywords(chars: &[char]) -> Option<TokenType> {
    match chars {
        ['a', 'n', 'd'] => Some(TokenType::And),
        ['c', 'l', 'a', 's', 's'] => Some(TokenType::Class),
        ['e', 'l', 's', 'e'] => Some(TokenType::Else),
        ['f', 'a', 'l', 's', 'e'] => Some(TokenType::False),
        ['f', 'o', 'r'] => Some(TokenType::For),
        ['f', 'u', 'n'] => Some(TokenType::Fun),
        ['i', 'f'] => Some(TokenType::If),
        ['n', 'i', 'l'] => Some(TokenType::Nil),
        ['o', 'r'] => Some(TokenType::Or),
        ['p', 'r', 'i', 'n', 't'] => Some(TokenType::Print),
        ['r', 'e', 't', 'u', 'r', 'n'] => Some(TokenType::Return),
        ['s', 'u', 'p', 'e', 'r'] => Some(TokenType::Super),
        ['t', 'h', 'i', 's'] => Some(TokenType::This),
        ['t', 'r', 'u', 'e'] => Some(TokenType::True),
        ['v', 'a', 'r'] => Some(TokenType::Var),
        ['w', 'h', 'i', 'l', 'e'] => Some(TokenType::While),
        _ => None,
    }
}

impl Scanner {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect::<Vec<char>>(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.clone()
    }

    fn add_token(&mut self, token_type: TokenType) {
        let token = Token {
            token_type,
            lexeme: self.source[self.start..self.current]
                .iter()
                .map(|c| *c)
                .collect(),
            literal: 0,
            line: self.line,
        };

        self.tokens.push(token);
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn peek_match(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        return self.source[self.current];
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source[self.current + 1];
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string.");
            return;
        }

        // closing "
        self.advance();

        let value: String = self.source[(self.start + 1)..(self.current - 1)]
            .iter()
            .collect::<String>();

        self.add_token(TokenType::String(value));
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let value: f64 = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse()
            .expect("consumed substring should be valid float");

        self.add_token(TokenType::Number(value));
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        if let Some(keyword) = keywords(&self.source[self.start..self.current]) {
            self.add_token(keyword);
            return;
        }

        let identifier_name = self.source[self.start..self.current]
            .iter()
            .collect::<String>();

        self.add_token(TokenType::Identifier(identifier_name));
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),

            '!' if self.peek_match('=') => self.add_token(TokenType::BangEqual),
            '!' => self.add_token(TokenType::Bang),
            '=' if self.peek_match('=') => self.add_token(TokenType::EqualEqual),
            '=' => self.add_token(TokenType::Equal),
            '<' if self.peek_match('=') => self.add_token(TokenType::LessEqual),
            '<' => self.add_token(TokenType::Less),
            '>' if self.peek_match('=') => self.add_token(TokenType::GreaterEqual),
            '>' => self.add_token(TokenType::Greater),

            '/' if self.peek_match('/') => {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenType::Slash),

            ' ' => (),
            '\r' => (),
            '\t' => (),

            '\n' => self.line += 1,

            '"' => self.string(),
            c if is_digit(c) => self.number(),
            c if is_alpha(c) => self.identifier(),

            '\0' => (),
            _ => error(self.line, "Unexpected character."),
        };
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:?} {:?} {}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

#[derive(Debug, Clone)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(String),
    String(String),
    Number(f64),

    And,
    Class,
    Else,
    False,
    Fun,
    For,
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

    EOF,
}
