use std::cell::{Ref, RefCell, RefMut};

use super::Inst;

pub struct Label<'m> {
    name: String,
    insts: RefCell<Vec<&'m Inst<'m>>>,
}

impl<'m> Label<'m> {
    pub fn new(name: String) -> Label<'m> {
        Label {
            name,
            insts: RefCell::new(Vec::new()),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn insts(&self) -> Ref<Vec<&'m Inst<'m>>> {
        self.insts.borrow()
    }

    pub fn insts_mut(&self) -> RefMut<Vec<&'m Inst<'m>>> {
        self.insts.borrow_mut()
    }

    pub fn add_instruction(&self, inst: &'m Inst<'m>) {
        self.insts.borrow_mut().push(inst);
    }
}
