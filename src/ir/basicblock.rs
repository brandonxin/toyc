use super::Inst;
use std::cell::{Ref, RefCell};
use std::fmt;
use std::hash::{Hash, Hasher};

pub struct BasicBlock<'m> {
    name: String,
    instructions: RefCell<Vec<&'m Inst<'m>>>,
}

impl<'m> BasicBlock<'m> {
    pub fn new(id: usize) -> BasicBlock<'m> {
        BasicBlock {
            name: format!("bb_{}", id),
            instructions: RefCell::new(Vec::new()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_instruction(&self, inst: &'m Inst<'m>) {
        self.instructions.borrow_mut().push(inst);
    }

    pub fn instructions(&self) -> Ref<Vec<&'m Inst<'m>>> {
        self.instructions.borrow()
    }
}

impl fmt::Display for BasicBlock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.name)?;
        for inst in self.instructions.borrow().iter() {
            write!(f, "\n\t{}", inst)?;
        }
        Ok(())
    }
}

impl<'m> PartialEq for &'m BasicBlock<'m> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(*self, *other)
    }
}

impl<'m> Eq for &'m BasicBlock<'m> {}

impl<'m> Hash for &'m BasicBlock<'m> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(*self, state)
    }
}
