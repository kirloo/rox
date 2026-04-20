
use crate::scanning::*;

use std::mem::discriminant;



fn parse_error(token : Token, message: &str) {
    use std::mem::discriminant;
    if discriminant(&token.token_type) == discriminant(&TokenType::EOF) {
        crate::report(token.line, " at end", message);
    } else {
        crate::report(token.line, &format!(" at '{}'", token.lexeme.iter().collect::<String>()), message);
    }
}



#[derive(Clone)]
pub enum ExprTree {
    Binary(Token, Box<ExprTree>, Box<ExprTree>),
    Unary(Token, Box<ExprTree>),
    Grouping(Box<ExprTree>),
    Literal(LitValue),
}

#[derive(Clone)]
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
            ExprTree::Unary(token, subexpr) => format!(
                "({} {})",
                token.lexeme.iter().collect::<String>(),
                Self::to_string(subexpr)
            ),
            ExprTree::Binary(token, leftexpr, rightexpr) => format!(
                "({} {} {})",
                token.lexeme.iter().collect::<String>(),
                Self::to_string(leftexpr),
                Self::to_string(rightexpr)
            ),
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
    current : usize,
    tokens : Vec<Token>,
}


impl Parser {
    pub fn new(tokens : Vec<Token>) -> Self {
        Parser {
            current : 0,
            tokens,
        }
    }

    pub fn parse(mut self) -> ExprTree {
        // TODO error handling
        self.expression()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() ||
        discriminant(&self.peek().token_type) == discriminant(&TokenType::EOF)
    }

    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current-1]
    }
    
    fn check(&self, tokentype : &TokenType) -> bool {
        if self.is_at_end() { return false; }
        discriminant(&self.peek().token_type) == discriminant(tokentype)
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, tokentypes : &[TokenType], error_message : &str) {
        if !self.tokenmatch(tokentypes) {
            parse_error(self.peek().clone(), error_message);
        }
    }

    fn tokenmatch(&mut self, types : &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expression(&mut self) -> ExprTree {
        self.equality()
    }
    
    fn equality(&mut self) -> ExprTree {
        let mut expr: ExprTree = self.comparison();

        while self.tokenmatch(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator: Token = self.previous().clone();
            let right : ExprTree = self.comparison();
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }
    
        expr
    }


    fn comparison(&mut self) -> ExprTree {
        let mut expr: ExprTree = self.term();

        while self.tokenmatch(&[TokenType::Less, TokenType::LessEqual, TokenType::Greater, TokenType::GreaterEqual]) {
            let operator: Token = self.previous().clone();
            let right : ExprTree = self.term();
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }
    
        expr
    }

    fn term(&mut self) -> ExprTree {
        let mut expr: ExprTree = self.factor();

        while self.tokenmatch(&[TokenType::Minus, TokenType::Plus]) {
            let operator: Token = self.previous().clone();
            let right : ExprTree = self.factor();
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }
    
        expr
    }
    
    fn factor(&mut self) -> ExprTree {
        let mut expr: ExprTree = self.unary();

        while self.tokenmatch(&[TokenType::Slash, TokenType::Star]) {
            let operator: Token = self.previous().clone();
            let right : ExprTree = self.unary();
            expr = ExprTree::Binary(operator, Box::new(expr), Box::new(right));
        }
    
        expr
    }

    fn unary(&mut self) -> ExprTree {
        if self.tokenmatch(&[TokenType::Bang, TokenType::Minus]) {
            let operator: Token = self.previous().clone();
            let right : ExprTree = self.unary();
            return ExprTree::Unary(operator, Box::new(right));
        }
    
        self.primary()
    }

    fn primary(&mut self) -> ExprTree {
        match &self.advance().token_type {
            TokenType::False => ExprTree::Literal(LitValue::False),
            TokenType::True => ExprTree::Literal(LitValue::True),
            TokenType::Nil => ExprTree::Literal(LitValue::Nil),
            TokenType::Number(n) => ExprTree::Literal(LitValue::Number(*n)),
            TokenType::String(s) => ExprTree::Literal(LitValue::StringLit(s.clone())),
            TokenType::LeftParen => {
                
                let expr = self.expression();
                self.consume(&[TokenType::RightParen], "Expect ')' after expression");
                ExprTree::Grouping(Box::new(expr))
            }

            _ => panic!("syntax error bruh") // TODO handle
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if discriminant(&self.previous().token_type) == discriminant(&TokenType::Semicolon) { return }
            
            match self.peek().token_type {
                TokenType::Class | TokenType::Fun | 
                TokenType::Var | TokenType::For | 
                TokenType::If | TokenType::While | 
                TokenType::Print | TokenType::Return => return,
                _ => ()
            }

            self.advance();
        }
    }

}


#[cfg(test)]
mod parser_test {
    use super::*;
    
    #[test]
    fn token_match_test() {
        let mut parser = Parser::new(vec![Token { token_type: TokenType::Number(1.0), lexeme: vec!['1'], line: 0 }]);

        assert!(!parser.tokenmatch(&[TokenType::Star, TokenType::Slash]));

        assert!(parser.tokenmatch(&[TokenType::Number(0f64)]));
    }
}