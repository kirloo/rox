use crate::{parsing::*, scanning::*};
use std::collections::BTreeMap;

fn evaluation_error(error: EvalError) {
    crate::report_raw(&error.to_string())
}

struct Environment {
    namespace: BTreeMap<String, Value>,
}

impl Environment {
    fn new() -> Self {
        let namespace = BTreeMap::new();
        Environment { namespace }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Value {
    fn stringify(self) -> String {
        match self {
            Self::Number(x) => {
                let text = x.to_string();
                if text.ends_with(".0") {
                    text[..text.len() - 2].to_string()
                } else {
                    text
                }
            }
            Self::String(s) => s,
            Self::Bool(b) => b.to_string(),
            Self::Nil => "nil".to_string(),
        }
    }

    /// Discriminant name
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
    line: usize,
    kind: EvalErrorKind,
}

#[derive(Debug)]
enum EvalErrorKind {
    Unary(Token, Value),
    Binary(Token, Value, Value),
    Unassigned(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            EvalErrorKind::Unary(token, v) => write!(
                f,
                "[line {}] Error at {}: unary operator cannot take argument of type '{}'",
                self.line,
                token.lexeme_string(),
                v.disc_name()
            ),
            EvalErrorKind::Binary(token, v1, v2) => write!(
                f,
                "[line {}] Error at {}: binary operator cannot take arguments of type '{}' and '{}'",
                self.line,
                token.lexeme_string(),
                v1.disc_name(),
                v2.disc_name()
            ),
            EvalErrorKind::Unassigned(ident) => write!(
                f,
                "[line {}] Error at {}: unassigned identifier '{}'",
                self.line, ident, ident
            ),
        }
    }
}

impl std::error::Error for EvalError {}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().stringify())
    }
}

pub fn interpret(program: Vec<Stmt>) {
    let mut environment = Environment::new();

    for stmt in program {
        execute(stmt, &mut environment).unwrap();
    }
}

fn execute(stmt: Stmt, env: &mut Environment) -> Result<(), EvalError> {
    match stmt {
        Stmt::ExprStmt(expr) => {
            evaluate(expr, env)?;
        }
        Stmt::PrintStmt(expr) => println!("{}", evaluate(expr, env)?),
        Stmt::Var(ident, expr) => {
            env.namespace.insert(ident, evaluate(expr, &env)?);
        }
        Stmt::Assignment(_, _) => todo!(),
    }
    Ok(())
}

fn evaluate(expression: ExprTree, env: &Environment) -> Result<Value, EvalError> {
    match expression {
        ExprTree::Grouping(subexpr) => evaluate(*subexpr, env),

        ExprTree::Unary(token, subexpr) => evaluate_unary(token, *subexpr, env),
        ExprTree::Binary(token, left, right) => evaluate_binary(token, *left, *right, env),

        ExprTree::Literal(LitValue::False) => Ok(Value::Bool(false)),
        ExprTree::Literal(LitValue::True) => Ok(Value::Bool(true)),
        ExprTree::Literal(LitValue::Number(x)) => Ok(Value::Number(x)),
        ExprTree::Literal(LitValue::StringLit(s)) => Ok(Value::String(s)),
        ExprTree::Literal(LitValue::Nil) => Ok(Value::Nil),
        ExprTree::Variable(name) => match env.namespace.get(&name) {
            Some(value) => Ok(value.clone()),
            None => Err(EvalError {
                line: 0,
                kind: EvalErrorKind::Unassigned(name.to_string()),
            }),
        },
    }
}

fn evaluate_unary(token: Token, expr: ExprTree, env: &Environment) -> Result<Value, EvalError> {
    let subvalue = evaluate(expr, env)?;

    match (&token.token_type, subvalue) {
        (TokenType::Bang, subvalue) => Ok(Value::Bool(!is_truthy(subvalue))),
        (TokenType::Minus, Value::Number(x)) => Ok(Value::Number(-x)),
        (_, value) => Err(EvalError {
            line: token.line,
            kind: EvalErrorKind::Unary(token, value),
        }),
    }
}

fn evaluate_binary(
    token: Token,
    left: ExprTree,
    right: ExprTree,
    env: &Environment,
) -> Result<Value, EvalError> {
    let leftvalue = evaluate(left, env)?;
    let rightvalue = evaluate(right, env)?;

    match (&token.token_type, leftvalue, rightvalue) {
        (TokenType::Plus, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
        (TokenType::Minus, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x - y)),
        (TokenType::Slash, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x / y)),
        (TokenType::Star, Value::Number(x), Value::Number(y)) => Ok(Value::Number(x * y)),

        (TokenType::Plus, Value::String(s1), Value::String(s2)) => {
            Ok(Value::String(format!("{}{}", s1, s2)))
        }

        (TokenType::BangEqual, left, right) => Ok(Value::Bool(left != right)),
        (TokenType::EqualEqual, left, right) => Ok(Value::Bool(left == right)),

        (TokenType::Less, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x < y)),
        (TokenType::LessEqual, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x <= y)),
        (TokenType::Greater, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x > y)),
        (TokenType::GreaterEqual, Value::Number(x), Value::Number(y)) => Ok(Value::Bool(x >= y)),

        (TokenType::And, Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 && b2)),
        (TokenType::Or, Value::Bool(b1), Value::Bool(b2)) => Ok(Value::Bool(b1 || b2)),

        (_, left, right) => Err(EvalError {
            line: token.line,
            kind: EvalErrorKind::Binary(token, left, right),
        }),
    }
}

fn is_truthy(value: Value) -> bool {
    match value {
        Value::Bool(false) | Value::Nil => false,
        _ => true,
    }
}
