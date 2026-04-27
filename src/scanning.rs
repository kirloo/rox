
fn scan_error(line : usize, message : &str) {
    crate::report(line, "", message);
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Vec<char>,
    pub line: usize,
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
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect::<Vec<char>>(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }


    fn add_token(&mut self, token_type: TokenType) {
        let token = Token {
            token_type,
            lexeme: self.source[self.start..self.current]
                .iter()
                .map(|c| *c)
                .collect(),
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
            scan_error(self.line, "Unterminated string.");
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
            _ => scan_error(self.line, "Unexpected character."),
        };
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:?} {:?}",
            self.token_type, self.lexeme
        )
    }
}

impl Token {
    pub fn lexeme_string(&self) -> String {
        self.lexeme.iter().collect::<String>()
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
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
