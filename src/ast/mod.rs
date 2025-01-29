mod decl;
mod expr;
mod func;
mod stmt;
mod type_expr;

pub type Module = Vec<GlobalDecl>;

pub use self::decl::GlobalDecl;
pub use self::expr::{BinaryOp, Expr, UnaryOp};
pub use self::func::{Func, FuncDecl, Param};
pub use self::stmt::Stmt;
pub use self::type_expr::TypeExpr;
