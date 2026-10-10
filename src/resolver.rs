use std::collections::HashMap;

use crate::parsing::{ExprTree, LitValue, Stmt};

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    function_type: FunctionType
}

#[derive(Clone, Copy)]
enum FunctionType {
    Function,
    None,
}

impl Resolver {
    pub fn new(globals: &Vec<impl ToString>) -> Self {
        let mut resolver = Resolver {
            scopes: vec![HashMap::new()],
            function_type: FunctionType::None,
        };

        for global in globals {
            resolver.declare(global.to_string());
            resolver.define(global.to_string())
        }
        resolver
    }

    pub fn resolve_program(&mut self, statements: &mut Vec<Stmt>) -> Result<(), ResolverError> {
        for stmt in statements {
            self.resolve_stmt(stmt)?;
        }

        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) -> Result<(), ResolverError> {
        match stmt {
            Stmt::ExprStmt(expr) | Stmt::PrintStmt(expr) => self.resolve_expr(expr)?,
            Stmt::Var(name, initializer) => {
                self.declare(name.clone());
                if !matches!(initializer, ExprTree::Literal(LitValue::Nil)) {
                    self.resolve_expr(initializer)?;
                }
                self.define(name.clone());
            }
            Stmt::Return(return_expr) => {
                if let FunctionType::None = self.function_type {
                    return Err(ResolverError::TopLevelReturn)
                }
                if let Some(expr) = return_expr {
                    self.resolve_expr(expr)?;
                }
            }
            Stmt::IfStmt(expr, stmt, else_stmt) => {
                self.resolve_expr(expr)?;
                self.resolve_stmt(stmt)?;
                if let Some(stmt) = else_stmt {
                    self.resolve_stmt(stmt)?;
                }
            }
            Stmt::WhileLoop(expr, stmt) => {
                self.resolve_expr(expr)?;
                self.resolve_stmt(stmt)?;
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scope();
            }
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &mut ExprTree) -> Result<(), ResolverError> {
        match expr {
            ExprTree::Literal(_) => (),
            ExprTree::Logical(_, lhs, rhs) | ExprTree::Binary(_, lhs, rhs) => {
                self.resolve_expr(lhs)?;
                self.resolve_expr(rhs)?;
            }
            ExprTree::Assignment {
                name,
                assigned_expr,
                ..
            } => {
                let name = name.clone();
                self.resolve_expr(assigned_expr)?;
                self.resolve_local(expr, name)?;
            }
            ExprTree::Unary(_, expr) | ExprTree::Grouping(expr) => self.resolve_expr(expr)?,
            ExprTree::Variable { name, .. } => {
                let name = name.clone();
                if !*self
                    .scopes
                    .first()
                    .map(|s| s.get(&name))
                    .flatten()
                    .unwrap_or(&true)
                {
                    return Err(ResolverError::SelfReferentialDecl);
                }

                self.resolve_local(expr, name)?;
            }
            ExprTree::Function { params, body } => {
                let previous_function = self.function_type;
                self.function_type = FunctionType::Function;

                self.begin_scope();
                for param in params {
                    self.declare(param.clone());
                    self.define(param.clone());
                }
                for stmt in body {
                    self.resolve_stmt(stmt)?;
                }

                self.end_scope();
                self.function_type = previous_function;
            }
            ExprTree::Call {
                token: _,
                callee,
                args,
            } => {
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_local(&mut self, expr: &mut ExprTree, name: String) -> Result<(), ResolverError> {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name) {
                self.resolve_referenced_scope(expr, self.scopes.len() - 1 - i);
                return Ok(());
            }
            // INTRODUCE COMPILE TIME UNASSIGNED ERROR ?
        }

        Ok(())
    }

    fn resolve_referenced_scope(&mut self, expr: &mut ExprTree, calculated_steps: usize) {
        if let ExprTree::Variable { env_steps, .. } | ExprTree::Assignment { env_steps, .. } = expr
        {
            *env_steps = calculated_steps;
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: String) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name, false);
    }

    fn define(&mut self, name: String) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes.last_mut().unwrap().insert(name, true);
    }
}

#[derive(Debug)]
pub enum ResolverError {
    SelfReferentialDecl,
    TopLevelReturn,
}
