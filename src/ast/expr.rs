#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    Integer {
        value: u64,
    },
    Variable {
        name: String,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        callee: String,
        arguments: Vec<Expr>,
    },
}

#[derive(PartialEq, Eq, Debug)]
pub enum UnaryOp {
    Neg,
    BitwiseNot,
    LogicalNot,
}

#[derive(PartialEq, Eq, Debug)]
pub enum BinaryOp {
    Assignment,

    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
