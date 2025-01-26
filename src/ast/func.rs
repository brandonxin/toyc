use super::stmt::Stmt;

#[derive(PartialEq, Eq, Debug)]
pub struct Func {
    decl: FuncDecl,
    body: Stmt,
}

impl Func {
    pub fn new(proto: FuncDecl, body: Stmt) -> Func {
        Func { decl: proto, body }
    }

    pub fn prototype(&self) -> &FuncDecl {
        &self.decl
    }

    pub fn body(&self) -> &Stmt {
        &self.body
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct FuncDecl {
    name: String,
    tyname: String,
    params: Vec<Param>,
}

impl FuncDecl {
    pub fn new(name: String, tyname: String, params: Vec<Param>) -> FuncDecl {
        FuncDecl {
            name,
            tyname,
            params,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tyname(&self) -> &str {
        &self.tyname
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Param {
    name: String,
    tyname: String,
}

impl Param {
    pub fn new(name: String, tyname: String) -> Param {
        Param { name, tyname }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tyname(&self) -> &str {
        &self.tyname
    }
}
