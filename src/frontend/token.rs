#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Token {
    Func,
    Extern,
    If,
    Else,
    For,
    While,
    Return,
    Var,

    Identifier(String),
    Integer(u64),

    Assign,

    LogicalOr,
    LogicalAnd,

    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,

    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,

    LShift,
    RShift,

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    BitwiseNot,
    LogicalNot,

    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,

    Colon,
    SemiColon,
    Comma,

    EOF,
}
