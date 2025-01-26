use super::expr::Expr;

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
        var_name: String,
        type_name: String,
        expr: Option<Box<Expr>>,
    },
    Return {
        expr: Option<Box<Expr>>,
    },
    ExprStmt {
        expr: Box<Expr>,
    },
}
