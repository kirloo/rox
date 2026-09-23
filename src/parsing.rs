use crate::scanning::*;

use std::mem::discriminant;

fn parse_error(token: Token, message: &str) {
    use std::mem::discriminant;
    if discriminant(&token.token_type) == discriminant(&TokenType::EOF) {
        crate::report(token.line, " at end", message);
    } else {
        crate::report(
            token.line,
            &format!(" at '{}'", token.lexeme.iter().collect::<String>()),
            message,
        );
    }
}

#[derive(Clone, Debug)]
pub struct ParseError {
    message: String,
    loc: ParseErrorLocation,
    line: usize,
}

#[derive(Clone, Debug)]
pub enum ParseErrorLocation {
    AtEnd,
    AtLexeme(String),
}

impl ParseError {
    fn need_more_tokens(line: usize) -> Self {
        ParseError {
            message: "expected more tokens".to_string(),
            line,
            loc: ParseErrorLocation::AtEnd,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            e @ ParseError {
                loc: ParseErrorLocation::AtEnd,
                ..
            } => write!(f, "[Line {}] Error at end: {}", e.line, e.message),
            e @ ParseError {
                loc: ParseErrorLocation::AtLexeme(lexeme),
                ..
            } => write!(f, "[Line {}] Error at {}: {}", e.line, lexeme, e.message),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    ExprStmt(ExprTree),
    PrintStmt(ExprTree),
    Var(String, ExprTree),
    Assignment(Token, ExprTree),
}

#[derive(Clone, Debug)]
pub enum ExprTree {
    Binary(Token, Box<ExprTree>, Box<ExprTree>),
    Unary(Token, Box<ExprTree>),
    Grouping(Box<ExprTree>),
    Literal(LitValue),
    Variable(String),
}

#[derive(Clone, Debug)]
pub enum LitValue {
    False,
    True,
    Nil,
    Number(f64),
    StringLit(String),
}

impl ExprTree {
    fn string_repr(&self) -> String {
        match self {
            ExprTree::Literal(lit) => lit.to_string(),
            ExprTree::Grouping(subexpr) => format!("(group {})", Self::to_string(subexpr)),
            ExprTree::Unary(token, subexpr) => {
                format!("({} {})", token.lexeme_string(), Self::to_string(subexpr))
            }
            ExprTree::Binary(token, leftexpr, rightexpr) => format!(
                "({} {} {})",
                token.lexeme_string(),
                Self::to_string(leftexpr),
                Self::to_string(rightexpr)
            ),
            ExprTree::Variable(ident) => ident.to_string(),
        }
    }
}

impl std::fmt::Display for ExprTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string_repr())
    }
}

impl std::fmt::Display for LitValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            LitValue::False => "False",
            LitValue::True => "True",
            LitValue::Nil => "Nil",
            LitValue::Number(num) => &num.to_string(),
            LitValue::StringLit(s) => s,
        };

        write!(f, "{}", repr)
    }
}

pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
}

pub enum ParserOutput {
    Good(Vec<Stmt>),
    Bad(Vec<ParseError>),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { current: 0, tokens }
    }

    pub fn parse(mut self) -> ParserOutput {
        self.program()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
            || discriminant(&self.peek().expect("yea").token_type) == discriminant(&TokenType::EOF)
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        self.tokens
            .get(self.current)
            .ok_or_else(|| ParseError::need_more_tokens(self.previous().line))
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, tokentype: &TokenType) -> Result<bool, ParseError> {
        if self.is_at_end() {
            return Ok(false);
        }
        Ok(discriminant(&self.peek()?.token_type) == discriminant(tokentype))
    }

    fn advance(&mut self) -> Result<&Token, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::need_more_tokens(self.previous().line));
        }
        self.current += 1;
        Ok(self.previous())
    }

    fn consume(&mut self, tokentypes: &[TokenType], error_message: &str) -> Result<(), ParseError> {
        if !self.tokenmatch(tokentypes)? {
            let lexeme = self.peek()?.lexeme_string();
            return Err(ParseError {
                //message : format!("expected one of {:?}, not {}", tokentypes, lexeme),
                message: error_message.to_string(),
                line: self.peek()?.line,
                loc: ParseErrorLocation::AtLexeme(lexeme),
            });
        }
        Ok(())
    }

    /// Checks if current token matches any of the given TokenTypes, and advances one token if so
    fn tokenmatch(&mut self, types: &[TokenType]) -> Result<bool, ParseError> {
        for t in types {
            if self.check(t)? {
                self.advance()?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn program(&mut self) -> ParserOutput {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            ParserOutput::Good(statements)
        } else {
            ParserOutput::Bad(errors)
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt;

        if self.tokenmatch(&[TokenType::Print])? {
            stmt = Stmt::PrintStmt(self.expression()?);
        } else if self.tokenmatch(&[TokenType::Var])? {
            stmt = self.var_declaration()?;
        } else {
            stmt = Stmt::ExprStmt(self.expression()?);
        }

        self.consume(&[TokenType::Semicolon], "Expect ';' after expression")?;

        Ok(stmt)
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let next = self.advance()?;

        let TokenType::Identifier(identifier) = &next.token_type.clone() else {
            let loc = match self.peek()?.token_type {
                TokenType::EOF => ParseErrorLocation::AtEnd,
                _ => ParseErrorLocation::AtLexeme(self.peek()?.lexeme_string()),
            };

            return Err(ParseError {
                message: "Expected identifier".to_string(),
                loc,
                line: self.peek()?.line,
            });
        };

        let next: &Token = self.advance()?;
        let line = next.line;
        if !matches!(&next.token_type, TokenType::Equal) {
            let loc: ParseErrorLocation = match self.peek()?.token_type {
                TokenType::EOF => ParseErrorLocation::AtEnd,
                _ => ParseErrorLocation::AtLexeme(self.peek()?.lexeme_string()),
            };

            return Err(ParseError {
                message: "Expected '='".to_string(),
                loc,
                line,
            });
        };

        let assigned_expr = self.expression()?;

        let stmt = Stmt::Var(identifier.to_string(), assigned_expr);
        Ok(stmt)
    }

    fn expression(&mut self) -> Result<ExprTree, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<ExprTree, ParseError> {
        let mut expr: ExprTree = self.comparison()?;

        while self.tokenmatch(&[TokenType::EqualEqual, TokenType::BangEqual])? {
            let operator: Token = self.previous().clone();
            let right: ExprTree = self.comparison()?;
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<ExprTree, ParseError> {
        let mut expr: ExprTree = self.term()?;

        while self.tokenmatch(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ])? {
            let operator: Token = self.previous().clone();
            let right: ExprTree = self.term()?;
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<ExprTree, ParseError> {
        let mut expr: ExprTree = self.factor()?;

        while self.tokenmatch(&[TokenType::Minus, TokenType::Plus])? {
            let operator: Token = self.previous().clone();
            let right: ExprTree = self.factor()?;
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<ExprTree, ParseError> {
        let mut expr: ExprTree = self.unary()?;

        while self.tokenmatch(&[TokenType::Slash, TokenType::Star])? {
            let operator: Token = self.previous().clone();
            let right: ExprTree = self.unary()?;
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<ExprTree, ParseError> {
        if self.tokenmatch(&[TokenType::Bang, TokenType::Minus])? {
            let operator: Token = self.previous().clone();
            let right: ExprTree = self.unary()?;
            return Ok(ExprTree::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<ExprTree, ParseError> {
        let token = self.advance()?;
        let expr = match &token.token_type {
            TokenType::False => ExprTree::Literal(LitValue::False),
            TokenType::True => ExprTree::Literal(LitValue::True),
            TokenType::Nil => ExprTree::Literal(LitValue::Nil),
            TokenType::Number(n) => ExprTree::Literal(LitValue::Number(*n)),
            TokenType::String(s) => ExprTree::Literal(LitValue::StringLit(s.clone())),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(&[TokenType::RightParen], "Expect ')' after expression")?;
                ExprTree::Grouping(Box::new(expr))
            }
            TokenType::Identifier(name) => ExprTree::Variable(name.to_string()),

            TokenType::EOF => {
                return Err(ParseError {
                    line: token.line,
                    loc: ParseErrorLocation::AtEnd,
                    message: "Incomplete expression".to_string(),
                });
            }
            _ => {
                return Err(ParseError {
                    line: token.line,
                    loc: ParseErrorLocation::AtLexeme(token.lexeme_string()),
                    message: "Expected terminating token".to_string(),
                });
            }
        };

        Ok(expr)
    }

    fn synchronize(&mut self) {
        let _ = self.advance();

        while !self.is_at_end() {
            if discriminant(&self.previous().token_type) == discriminant(&TokenType::Semicolon) {
                return;
            }

            match self.peek().expect("should not be at end").token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            let _ = self.advance();
        }
    }
}

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn token_match_test() {
        let mut parser = Parser::new(vec![Token {
            token_type: TokenType::Number(1.0),
            lexeme: vec!['1'],
            line: 0,
        }]);

        assert!(
            !parser
                .tokenmatch(&[TokenType::Star, TokenType::Slash])
                .unwrap()
        );

        assert!(parser.tokenmatch(&[TokenType::Number(0f64)]).unwrap());
    }
}
