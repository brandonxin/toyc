use std::cell::{Ref, RefCell, RefMut};

use super::Label;

pub struct Func<'m> {
    name: String,
    prologue: Option<&'m Label<'m>>,
    epilogue: Option<&'m Label<'m>>,
    body: RefCell<Vec<&'m Label<'m>>>,
}

impl<'m> Func<'m> {
    pub fn new(name: String) -> Func<'m> {
        Func {
            name,
            prologue: None,
            epilogue: None,
            body: RefCell::new(Vec::new()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn prologue(&self) -> &'m Label<'m> {
        self.prologue.unwrap()
    }

    pub fn set_prologue(&mut self, prologue: &'m Label<'m>) {
        self.prologue = Some(prologue);
    }

    pub fn epilogue(&self) -> &'m Label<'m> {
        self.epilogue.unwrap()
    }

    pub fn set_epilogue(&mut self, epilogue: &'m Label<'m>) {
        self.epilogue = Some(epilogue);
    }

    pub fn body(&self) -> Ref<Vec<&'m Label<'m>>> {
        self.body.borrow()
    }

    pub fn body_mut(&self) -> RefMut<Vec<&'m Label<'m>>> {
        self.body.borrow_mut()
    }
}
