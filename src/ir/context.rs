use std::cell::RefCell;

use typed_arena::Arena;

use super::{BasicBlock, Constant, Func, Inst, Param, Value};

pub struct Context<'m> {
    next_id: RefCell<usize>,
    func: Arena<Func<'m>>,
    param: Arena<Param>,
    basic_block: Arena<BasicBlock<'m>>,
    inst: Arena<Inst<'m>>,
    constant: Arena<Constant>,
}

impl<'m> Context<'m> {
    pub fn new() -> Context<'m> {
        Context {
            next_id: RefCell::new(0),
            func: Arena::new(),
            param: Arena::new(),
            basic_block: Arena::new(),
            inst: Arena::new(),
            constant: Arena::new(),
        }
    }

    pub fn reset_id(&self) {
        *self.next_id.borrow_mut() = 0;
    }

    fn next_id(&self) -> usize {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;
        id
    }

    fn next_name(&self) -> String {
        format!("%{}", self.next_id())
    }

    pub fn new_function(&'m self, name: String, params: Vec<&'m Param>) -> &'m mut Func<'m> {
        self.func
            .alloc(Func::new(name, params, self.new_basic_block()))
    }

    pub fn new_parameter(&self, name: String) -> &Param {
        self.param.alloc(Param::new(self.next_name()))
    }

    pub fn new_basic_block(&'m self) -> &'m BasicBlock<'m> {
        self.basic_block.alloc(BasicBlock::new(self.next_id()))
    }

    pub fn alloca(&self) -> &Inst<'m> {
        self.inst.alloc(Inst::alloca(self.next_name()))
    }

    pub fn store(&self, value: &'m dyn Value, ptr: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::store(self.next_name(), value, ptr))
    }

    pub fn load(&self, ptr: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::load(self.next_name(), ptr))
    }

    pub fn or(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::or(self.next_name(), op0, op1))
    }

    pub fn xor(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::xor(self.next_name(), op0, op1))
    }

    pub fn and(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::and(self.next_name(), op0, op1))
    }

    pub fn lshl(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::lshl(self.next_name(), op0, op1))
    }

    pub fn lshr(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::lshr(self.next_name(), op0, op1))
    }

    pub fn ashr(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::ashr(self.next_name(), op0, op1))
    }

    pub fn eq(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::eq(self.next_name(), op0, op1))
    }

    pub fn ne(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::ne(self.next_name(), op0, op1))
    }

    pub fn gt(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::gt(self.next_name(), op0, op1))
    }

    pub fn ge(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::ge(self.next_name(), op0, op1))
    }

    pub fn lt(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::lt(self.next_name(), op0, op1))
    }

    pub fn le(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::le(self.next_name(), op0, op1))
    }

    pub fn add(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::add(self.next_name(), op0, op1))
    }

    pub fn sub(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::sub(self.next_name(), op0, op1))
    }

    pub fn mul(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::mul(self.next_name(), op0, op1))
    }

    pub fn div(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::div(self.next_name(), op0, op1))
    }

    pub fn modulo(&self, op0: &'m dyn Value, op1: &'m dyn Value) -> &Inst<'m> {
        self.inst.alloc(Inst::modulo(self.next_name(), op0, op1))
    }

    pub fn jump(&self, target: &'m BasicBlock<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::jump(self.next_name(), target))
    }

    pub fn cjump(
        &self,
        cond: &'m dyn Value,
        then_block: &'m BasicBlock<'m>,
        else_block: &'m BasicBlock<'m>,
    ) -> &Inst<'m> {
        self.inst
            .alloc(Inst::cjump(self.next_name(), cond, then_block, else_block))
    }

    pub fn call(&self, name: String, callee: &'m Func<'m>, args: Vec<&'m dyn Value>) -> &Inst<'m> {
        self.inst.alloc(Inst::call(name, callee, args))
    }

    pub fn ret(&self, value: Option<&'m dyn Value>) -> &Inst<'m> {
        self.inst.alloc(Inst::ret(self.next_name(), value))
    }

    pub fn new_constant(&self, value: u64) -> &Constant {
        self.constant.alloc(Constant::new(self.next_name(), value))
    }
}
