use std::rc::Rc;

use super::{Stmt, TypeSpecifier};

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
    ret_ty: Rc<TypeSpecifier>,
    params: Vec<Param>,
}

impl FuncDecl {
    pub fn new(name: String, ret_ty: TypeSpecifier, params: Vec<Param>) -> FuncDecl {
        FuncDecl {
            name,
            ret_ty: Rc::new(ret_ty),
            params,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ret_ty(&self) -> Rc<TypeSpecifier> {
        self.ret_ty.clone()
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Param {
    name: String,
    ty: Rc<TypeSpecifier>,
}

impl Param {
    pub fn new(name: String, ty: TypeSpecifier) -> Param {
        Param {
            name,
            ty: Rc::new(ty),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> Rc<TypeSpecifier> {
        self.ty.clone()
    }
}
