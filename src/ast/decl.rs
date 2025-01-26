use super::func::{Func, FuncDecl};

#[derive(PartialEq, Eq, Debug)]
pub enum GlobalDecl {
    FuncDecl(FuncDecl),
    Function(Func),
}
