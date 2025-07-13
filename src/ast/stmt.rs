use std::rc::Rc;

use super::{Expr, TypeSpecifier};

#[derive(PartialEq, Eq, Debug)]
pub enum Stmt {
    Block {
        stmts: Vec<Stmt>,
    },
    IfElse {
        cond: Box<Expr>,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Stmt>,
    },
    VarDecl {
        name: String,
        ty: Rc<TypeSpecifier>,
        expr: Option<Box<Expr>>,
    },
    Return {
        expr: Option<Box<Expr>>,
    },
    Expr {
        expr: Box<Expr>,
    },
}
