

use crate::{parsing::*, scanning::*};


fn evaluation_error(error : EvalError) {
    crate::report_raw(&error.to_string())
}




#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil
}


impl Value {
    fn stringify(self) -> String {
        match self {
            Self::Number(x) => {
                let text = x.to_string();
                if text.ends_with(".0") {
                    text[..text.len()-2].to_string()
                } else {
                    text
                }
            },
            Self::String(s) => s,
            Self::Bool(b) => b.to_string(),
            Self::Nil => "nil".to_string(),
        }
    }

    fn disc_name(&self) -> &str {
        match *self {
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Bool(_) => "bool",
            Self::Nil => "nil",
        }
    }
}

#[derive(Debug)]
struct EvalError {
    line : usize,
    op_token : Token,
    args : EvalErrorArgs
}

#[derive(Debug)]
enum EvalErrorArgs {
    Unary(Value),
    Binary(Value,Value),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = self.op_token.lexeme.iter().collect::<String>();

        match &self.args {
            EvalErrorArgs::Unary(v) => 
                write!(f, "[line {}] Error at {}: unary operator cannot take argument of type '{}'", self.line, op, v.disc_name()),
            EvalErrorArgs::Binary(v1,v2) => 
                write!(f, "[line {}] Error at {}: binary operator cannot take arguments of type '{}' and '{}'", self.line, op, v1.disc_name(), v2.disc_name()),
        }
    }
}

impl std::error::Error for EvalError {}





pub fn interpret(expr : ExprTree) {
    match evaluate(expr) {
        Ok(value) => println!("{}", value.stringify()),
        Err(e) => println!("{}", e)
    }
}



fn evaluate(expression : ExprTree) -> Result<Value, EvalError> {
    match expression {
        ExprTree::Grouping(subexpr) => evaluate(*subexpr),

        ExprTree::Unary(token, subexpr) => evaluate_unary(token, *subexpr),
        ExprTree::Binary(token, left, right) => evaluate_binary(token, *left, *right),
        
        ExprTree::Literal(LitValue::False) => Ok(Value::Bool(false)),
        ExprTree::Literal(LitValue::True) => Ok(Value::Bool(true)),
        ExprTree::Literal(LitValue::Number(x)) => Ok(Value::Number(x)),
        ExprTree::Literal(LitValue::StringLit(s)) => Ok(Value::String(s)),
        ExprTree::Literal(LitValue::Nil) => Ok(Value::Nil),
    }
}


fn evaluate_unary(token : Token, expr : ExprTree) -> Result<Value, EvalError> {

    let subvalue = evaluate(expr)?;

    match (&token.token_type, subvalue) {
        (TokenType::Bang, subvalue) => Ok(Value::Bool(!is_truthy(subvalue))),
        (TokenType::Minus, Value::Number(x)) => Ok(Value::Number(-x)),
        (_, value) => Err(EvalError { line: token.line, op_token: token, args: EvalErrorArgs::Unary(value) }),
    }
} 



fn evaluate_binary(token : Token, left : ExprTree, right : ExprTree) -> Result<Value, EvalError> {

    let leftvalue = evaluate(left)?;
    let rightvalue = evaluate(right)?;

    match (&token.token_type, leftvalue, rightvalue) {
        (TokenType::Plus, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
        (TokenType::Minus, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
        (TokenType::Slash, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x / y)),
        (TokenType::Star, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),

        (TokenType::Plus, Value::String(s1), Value::String(s2)) => Ok(Value::String(format!("{}{}", s1,s2))),

        (TokenType::BangEqual, left, right) => Ok(Value::Bool(left != right)),
        (TokenType::EqualEqual, left, right) => Ok(Value::Bool(left == right)),

        (TokenType::Less, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x < y)),
        (TokenType::LessEqual, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x <= y)),
        (TokenType::Greater, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x > y)),
        (TokenType::GreaterEqual, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x >= y)),

        (TokenType::And, Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 && b2)),
        (TokenType::Or, Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 || b2)),
        
        (_, left, right) => 
            Err(EvalError { line: token.line, op_token: token, args:EvalErrorArgs::Binary(left,right) }), // error
    }
}




fn is_truthy(value : Value) -> bool {
    match value {
        Value::Bool(false) | Value::Nil => false,
        _ => true
    }
}