

use crate::{parsing::*, scanning::*};


fn evaluation_error(line : usize, token : Token, message : &str) {
    crate::report(line, &format!(" at '{}'", token.lexeme.iter().collect::<String>()), message)
}



#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil
}


impl Value {
    fn discriminant_name(&self) -> &str {
        match *self {
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Bool(_) => "bool",
            Self::Nil => "nil",
        }
    }
}




fn evaluate(expression : ExprTree) -> Value {
    match expression {
        ExprTree::Grouping(subexpr) => evaluate(*subexpr),

        ExprTree::Unary(token, subexpr) => evaluate_unary(token, *subexpr),
        ExprTree::Binary(token, left, right) => evaluate_binary(token, *left, *right),
        
        ExprTree::Literal(LitValue::False) => Value::Bool(false),
        ExprTree::Literal(LitValue::True) => Value::Bool(true),
        ExprTree::Literal(LitValue::Number(x)) => Value::Number(x),
        ExprTree::Literal(LitValue::StringLit(s)) => Value::String(s),
        ExprTree::Literal(LitValue::Nil) => Value::Nil,
    }
}


// Maybe TokenType should be separated into BinaryToken and UnaryToken during parsing ???

// Use own Result type for propagating errors?

// Revise error handling in Scanner and Parser?

// Error types?

fn evaluate_unary(token : Token, expr : ExprTree) -> Value {

    let subvalue = evaluate(expr);

    match (token.token_type, subvalue) {
        (TokenType::Bang, subvalue) => Value::Bool(!is_truthy(subvalue)),
        (TokenType::Minus, Value::Number(x)) => Value::Number(-x),
        (token_type, value) => todo!(),
    }
} 



fn evaluate_binary(token : Token, left : ExprTree, right : ExprTree) -> Value {

    let leftvalue = evaluate(left);
    let rightvalue = evaluate(right);

    match (token.token_type, leftvalue, rightvalue) {
        (TokenType::Plus, Value::Number(x), Value::Number(y)) => Value::Number(x + y),
        (TokenType::Minus, Value::Number(x), Value::Number(y)) => Value::Number(x - y),
        (TokenType::Slash, Value::Number(x), Value::Number(y)) => Value::Number(x / y),
        (TokenType::Star, Value::Number(x), Value::Number(y)) => Value::Number(x * y),

        (TokenType::Plus, Value::String(s1), Value::String(s2)) => Value::String(format!("{}{}", s1,s2)),

        (TokenType::BangEqual, left, right) => Value::Bool(left != right),
        (TokenType::EqualEqual, left, right) => Value::Bool(left == right),

        (TokenType::Less, Value::Number(x), Value::Number(y)) => Value::Bool(x < y),
        (TokenType::LessEqual, Value::Number(x), Value::Number(y)) => Value::Bool(x <= y),
        (TokenType::Greater, Value::Number(x), Value::Number(y)) => Value::Bool(x > y),
        (TokenType::GreaterEqual, Value::Number(x), Value::Number(y)) => Value::Bool(x >= y),

        (TokenType::And, Value::Bool(b1), Value::Bool(b2)) => Value::Bool(b1 && b2),
        (TokenType::Or, Value::Bool(b1), Value::Bool(b2)) => Value::Bool(b1 || b2),
        
        (token_type, left, right) => todo!(), // error
    }
}




fn is_truthy(value : Value) -> bool {
    match value {
        Value::Bool(false) | Value::Nil => false,
        _ => true
    }
}