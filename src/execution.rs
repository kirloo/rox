use crate::{native_functions, parsing::*, scanning::*};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

fn evaluation_error(error: EvalError) {
    crate::report_raw(&error.to_string())
}

type EnvRef = Rc<RefCell<Environment>>;

struct Environment {
    namespace: HashMap<String, Value>,
    enclosing: Option<EnvRef>,
}

impl Environment {
    fn new() -> Self {
        let env = Environment {
            namespace: HashMap::new(),
            enclosing: None,
        };

        env
    }

    fn new_inner(outer: EnvRef) -> Self {
        Environment {
            namespace: HashMap::new(),
            enclosing: Some(outer),
        }
    }

    fn define(&mut self, name: String, value: Value) {
        self.namespace.insert(name, value);
    }

    fn lookup(&self, name: &str) -> Result<Value, EvalError> {
        if let Some(value) = self.namespace.get(name) {
            return Ok(value.clone());
        }

        match &self.enclosing {
            Some(outer) => outer.as_ref().borrow().lookup(name),
            None => Err(EvalError {
                line: 0,
                kind: EvalErrorKind::Unassigned(name.to_string()),
            }),
        }
    }

    fn assign(&mut self, name: &str, value: Value) -> Result<(), EvalError> {
        if self.namespace.contains_key(name) {
            self.namespace.insert(name.to_string(), value);
            return Ok(());
        }

        match &self.enclosing {
            Some(outer) => outer.borrow_mut().assign(name, value),
            None => Err(EvalError {
                line: 0,
                kind: EvalErrorKind::Unassigned(name.to_string()),
            }),
        }
    }
}

fn root_env(current: EnvRef) -> EnvRef {
    let Some(enclosing) = current.borrow().enclosing.clone() else {
        return current;
    };

    root_env(enclosing)
}

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Func {
        params: Vec<String>,
        function: FuncImpl,
    },
}

pub type NativeFn = fn(Vec<Value>) -> Result<Value, EvalError>;

#[derive(Clone, Debug)]
pub enum FuncImpl {
    LoxFunc(Vec<Stmt>),
    Native(NativeFn),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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
            Self::Func { params, function } => "a function".to_string(),
        }
    }

    /// Discriminant name
    fn disc_name(&self) -> &str {
        match *self {
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Bool(_) => "bool",
            Self::Nil => "nil",
            Self::Func { .. } => "function",
        }
    }
}

#[derive(Debug)]
pub struct EvalError {
    line: usize,
    kind: EvalErrorKind,
}

#[derive(Debug)]
pub enum EvalErrorKind {
    Unary(Token, Value),
    Binary(Token, Value, Value),
    Unassigned(String),
    TypeError { expected: String, found: Value },
    ArgumentMismatch { expected: usize, found: usize },
    // Evil variant
    Return(Value),
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
            EvalErrorKind::TypeError { expected, found } => write!(
                f,
                "[line {}] Error at {}: expected type {}, found {}",
                self.line, found, expected, found,
            ),
            EvalErrorKind::ArgumentMismatch { expected, found } => write!(
                f,
                "[line {}] Error in call: expected {} arguments, found {}.",
                self.line, expected, found
            ),
            EvalErrorKind::Return(_) => write!(
                f,
                "[line {}] Error at 'return': return statement outside function",
                self.line
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
    let mut global_env = Environment::new();

    global_env.define(
        "clock".to_string(),
        Value::Func {
            params: vec![],
            function: FuncImpl::Native(native_functions::clock),
        },
    );

    let global_envref = Rc::new(RefCell::new(global_env));

    //let base_env = Environment::new_inner(global_envref);
    //let base_envref = Rc::new(RefCell::new(base_env));

    for stmt in program {
        // TODO handle errors
        execute(stmt, global_envref.clone()).unwrap();
    }
}

fn execute(stmt: Stmt, env: EnvRef) -> Result<(), EvalError> {
    match stmt {
        Stmt::ExprStmt(expr) => {
            evaluate(expr, env)?;
        }
        Stmt::PrintStmt(expr) => println!("{}", evaluate(expr, env)?),
        Stmt::Var(ident, expr) => {
            let evaluated = evaluate(expr, env.clone())?;
            env.borrow_mut().define(ident, evaluated);
        }
        Stmt::Block(statements) => {
            execute_block(statements, env)?;
        }
        Stmt::IfStmt(cond, b1, b2) => match evaluate(cond, env.clone())? {
            Value::Bool(bool) => {
                if bool {
                    execute(*b1, env)?;
                } else if let Some(b2) = b2 {
                    execute(*b2, env)?;
                }
            }
            val => {
                return Err(EvalError {
                    line: 0,
                    kind: EvalErrorKind::TypeError {
                        expected: "bool".to_string(),
                        found: val,
                    },
                });
            }
        },
        Stmt::WhileLoop(cond, body) => {
            while is_truthy(&evaluate(cond.clone(), env.clone())?) {
                execute(*body.clone(), env.clone())?
            }
        }
        Stmt::Return(maybe_expr) => {
            let value = if let Some(expr) = maybe_expr {
                evaluate(expr, env)?
            } else {
                Value::Nil
            };
            return Err(EvalError {
                line: 0,
                kind: EvalErrorKind::Return(value),
            });
        }
    }
    Ok(())
}

fn execute_block(block: Vec<Stmt>, env: EnvRef) -> Result<(), EvalError> {
    let env = Environment::new_inner(env);
    let envref = Rc::new(RefCell::new(env));

    for stmt in block {
        execute(stmt, envref.clone())?;
    }

    Ok(())
}

fn execute_function(block: Vec<Stmt>, env: EnvRef) -> Result<Value, EvalError> {
    let env = Environment::new_inner(env);
    let func_envref = Rc::new(RefCell::new(env));

    for stmt in block {
        match execute(stmt, func_envref.clone()) {
            Ok(_) => (),
            Err(EvalError {
                line: _,
                kind: EvalErrorKind::Return(value),
            }) => return Ok(value),
            Err(e) => return Err(e),
        }
    }

    Ok(Value::Nil)
}

fn evaluate(expression: ExprTree, env: EnvRef) -> Result<Value, EvalError> {
    match expression {
        ExprTree::Grouping(subexpr) => evaluate(*subexpr, env),

        ExprTree::Unary(token, subexpr) => evaluate_unary(token, *subexpr, env),
        ExprTree::Binary(token, left, right) => evaluate_binary(token, *left, *right, env),
        ExprTree::Logical(token, left, right) => {
            let left_value = evaluate(*left, env.clone())?;
            if matches!(token.token_type, TokenType::Or) {
                if is_truthy(&left_value) {
                    return Ok(left_value);
                }
            } else if !is_truthy(&left_value) {
                return Ok(left_value);
            }
            evaluate(*right, env)
        }

        ExprTree::Literal(LitValue::False) => Ok(Value::Bool(false)),
        ExprTree::Literal(LitValue::True) => Ok(Value::Bool(true)),
        ExprTree::Literal(LitValue::Number(x)) => Ok(Value::Number(x)),
        ExprTree::Literal(LitValue::StringLit(s)) => Ok(Value::String(s)),
        ExprTree::Literal(LitValue::Nil) => Ok(Value::Nil),
        ExprTree::Variable(name) => env.as_ref().borrow().lookup(&name),
        ExprTree::Function { params, body } => Ok(Value::Func {
            params,
            function: FuncImpl::LoxFunc(body),
        }),
        ExprTree::Assignment(name, expr_tree) => {
            let value = evaluate(*expr_tree, env.clone())?;
            env.as_ref().borrow_mut().assign(&name, value.clone())?;
            Ok(value)
        }

        ExprTree::Call {
            token,
            callee,
            args,
        } => {
            let arg_values = args
                .into_iter()
                .map(|arg| evaluate(arg, env.clone()))
                .collect::<Result<Vec<Value>, EvalError>>()?;

            let result = match evaluate(*callee, env.clone())? {
                Value::Func { params, function } => {
                    call_function(env, params, arg_values, function)
                }

                val => {
                    return Err(EvalError {
                        line: token.line,
                        kind: EvalErrorKind::TypeError {
                            expected: "callable".to_string(),
                            found: val,
                        },
                    });
                }
            };

            result
        }
    }
}

fn call_function(
    env: EnvRef,
    params: Vec<String>,
    args: Vec<Value>,
    callee: FuncImpl,
) -> Result<Value, EvalError> {
    if params.len() != args.len() {
        return Err(EvalError {
            line: 0,
            kind: EvalErrorKind::ArgumentMismatch {
                expected: params.len(),
                found: args.len(),
            },
        });
    }

    match callee {
        FuncImpl::Native(native_fn) => native_fn(args),
        FuncImpl::LoxFunc(block) => {
            let global_env = root_env(env);
            let mut func_scope = Environment::new_inner(global_env);

            for (param, arg) in params.into_iter().zip(args.into_iter()) {
                func_scope.define(param, arg);
            }

            let func_envref = Rc::new(RefCell::new(func_scope));

            let return_value = execute_function(block, func_envref)?;

            Ok(return_value)
        }
    }
}

fn evaluate_unary(token: Token, expr: ExprTree, env: EnvRef) -> Result<Value, EvalError> {
    let subvalue = evaluate(expr, env)?;

    match (&token.token_type, subvalue) {
        (TokenType::Bang, subvalue) => Ok(Value::Bool(!is_truthy(&subvalue))),
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
    env: EnvRef,
) -> Result<Value, EvalError> {
    let leftvalue = evaluate(left, env.clone())?;
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

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(false) | Value::Nil => false,
        _ => true,
    }
}
