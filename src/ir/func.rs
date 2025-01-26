use super::{BasicBlock, Constant, Inst, Param, Value};
use std::cell::{Ref, RefCell};
use std::fmt;
use std::hash::{Hash, Hasher};

pub struct Func<'m> {
    name: String,
    params: Vec<&'m Param>,
    constants: RefCell<Vec<&'m Constant>>,
    blocks: RefCell<Vec<&'m BasicBlock<'m>>>,
    insert_point: RefCell<&'m BasicBlock<'m>>,
}

impl<'m> Func<'m> {
    pub fn new(name: String, params: Vec<&'m Param>, entry: &'m BasicBlock<'m>) -> Func<'m> {
        Func {
            name,
            params,
            constants: RefCell::new(vec![]),
            blocks: RefCell::new(vec![entry]),
            insert_point: RefCell::new(entry),
        }
    }

    pub fn add_block(&self, block: &'m BasicBlock<'m>) {
        self.blocks.borrow_mut().push(block);
    }

    pub fn insert_point(&self) -> &'m BasicBlock<'m> {
        *self.insert_point.borrow()
    }

    pub fn set_insert_point(&self, block: &'m BasicBlock<'m>) {
        *self.insert_point.borrow_mut() = block;
    }

    pub fn add_instruction(&self, inst: &'m Inst<'m>) {
        self.insert_point.borrow().add_instruction(inst);
    }

    pub fn add_constant(&self, constant: &'m Constant) {
        self.constants.borrow_mut().push(constant);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &Vec<&'m Param> {
        &self.params
    }

    pub fn constants(&self) -> Ref<Vec<&'m Constant>> {
        self.constants.borrow()
    }

    pub fn blocks(&self) -> Ref<Vec<&'m BasicBlock<'m>>> {
        self.blocks.borrow()
    }
}

impl fmt::Display for Func<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO If this function is just a prototype, print 'extern' instead of
        // 'define'
        write!(f, "define @{}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param.name())?;
        }
        write!(f, ") {{")?;
        for block in self.blocks.borrow().iter() {
            write!(f, "\n{}", block)?;
        }
        write!(f, "\n}}")
    }
}

impl<'m> PartialEq for &'m Func<'m> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(*self, *other)
    }
}

impl<'m> Eq for &'m Func<'m> {}

impl<'m> Hash for &'m Func<'m> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(*self, state)
    }
}
