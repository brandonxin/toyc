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

    pub fn name(&self) -> &str {
        &self.func_name
    }

    pub fn return_type(&self) -> &str {
        &self.return_type
    }

    pub fn params(&self) -> &Vec<Parameter> {
        &self.params
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

    pub fn prototype(&self) -> &Prototype {
        &self.proto
    }

    pub fn body(&self) -> &Stmt {
        &self.body
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

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }
}

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

    Lt,

    Add,
    Sub,
    Mul,
    Div,
}
