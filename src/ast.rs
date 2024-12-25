pub type CompilationUnit = Vec<Declaration>;

#[derive(PartialEq, Eq, Debug)]
pub enum Declaration {
    Prototype(Prototype),
    Function(Function),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Prototype {
    func_name: String,
    return_type: String,
    params: Vec<Parameter>,
}

impl Prototype {
    pub fn new(
        func_name: String,
        return_type: String,
        params: Vec<Parameter>,
    ) -> Prototype {
        return Prototype {
            func_name,
            return_type,
            params,
        };
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Function {
    proto: Prototype,
    body: Stmt,
}

impl Function {
    pub fn new(proto: Prototype, body: Stmt) -> Function {
        Function { proto, body }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Parameter {
    name: String,
    type_name: String,
}

impl Parameter {
    pub fn new(name: String, type_name: String) -> Parameter {
        Parameter { name, type_name }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Stmt {
    Block(Block),
    IfElse(IfElse),
    While(While),
    VarDecl(VarDecl),
    Return(Return),
    ExprStmt(Expr),
}

type Block = Vec<Stmt>;

#[derive(PartialEq, Eq, Debug)]
pub struct IfElse {
    cond: Box<Expr>,
    then_stmt: Box<Stmt>,
    else_stmt: Option<Box<Stmt>>,
}

impl IfElse {
    pub fn new(cond: Expr, then_stmt: Stmt, else_stmt: Option<Stmt>) -> IfElse {
        IfElse {
            cond: Box::<Expr>::new(cond),
            then_stmt: Box::<Stmt>::new(then_stmt),
            else_stmt: match else_stmt {
                Some(s) => Some(Box::<Stmt>::new(s)),
                None => None,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct While {
    cond: Box<Expr>,
    body: Box<Stmt>,
}

impl While {
    pub fn new(cond: Expr, body: Stmt) -> While {
        While {
            cond: Box::<Expr>::new(cond),
            body: Box::<Stmt>::new(body),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct VarDecl {
    var_name: String,
    type_name: String,
    expr: Option<Box<Expr>>,
}

impl VarDecl {
    pub fn new(
        var_name: String,
        type_name: String,
        expr: Option<Expr>,
    ) -> VarDecl {
        VarDecl {
            var_name,
            type_name,
            expr: match expr {
                Some(e) => Some(Box::<Expr>::new(e)),
                None => None,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Return {
    expr: Option<Box<Expr>>,
}

impl Return {
    pub fn new(expr: Option<Expr>) -> Return {
        Return {
            expr: match expr {
                Some(e) => Some(Box::<Expr>::new(e)),
                None => None,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    Integer(Integer),
    Variable(Variable),
    Unary(Unary),
    Binary(Binary),
    Call(Call),
}

pub type Integer = u64;

pub type Variable = String;

#[derive(PartialEq, Eq, Debug)]
pub enum UnaryOp {
    Neg,
    BitwiseNot,
    LogicalNot,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Unary {
    op: UnaryOp,
    operand: Box<Expr>,
}

impl Unary {
    pub fn new(op: UnaryOp, operand: Expr) -> Unary {
        Unary {
            op,
            operand: Box::<Expr>::new(operand),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum BinaryOp {
    Assignment,

    Lt,

    Add,
    Sub,
    Mul,
    Div,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Binary {
    op: BinaryOp,
    lhs: Box<Expr>,
    rhs: Box<Expr>,
}

impl Binary {
    pub fn new(op: BinaryOp, lhs: Expr, rhs: Expr) -> Binary {
        Binary {
            op,
            lhs: Box::<Expr>::new(lhs),
            rhs: Box::<Expr>::new(rhs),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Call {
    callee: String,
    arguments: Vec<Expr>,
}

impl Call {
    pub fn new(callee: String, arguments: Vec<Expr>) -> Call {
        Call { callee, arguments }
    }
}
