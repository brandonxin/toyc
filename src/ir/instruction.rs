use super::{BasicBlock, Value};

pub enum InstKind<'a> {
    Alloca,
    // <0: val> -> *<1: ptr>
    Store(&'a dyn Value, &'a dyn Value),
    // result := *<0: ptr>
    Load(&'a dyn Value),

    // result := <0: op0> + <1: op1>
    Add(&'a dyn Value, &'a dyn Value),
    // result := <0: op0> - <1: op1>
    Sub(&'a dyn Value, &'a dyn Value),
    // result := <0: op0> * <1: op1>
    Mul(&'a dyn Value, &'a dyn Value),
    // result := <0: op0> / <1: op1>
    Div(&'a dyn Value, &'a dyn Value),

    // goto <0: target>
    Jump(&'a BasicBlock<'a>),
    // if <0: cond> != $0 goto <1: target1> else goto <2: target2>
    CJump(&'a dyn Value, &'a BasicBlock<'a>, &'a BasicBlock<'a>),

    // result := call <0: callee>(<1: args...>)
    Call(String, Vec<&'a dyn Value>),
    // return <0: val?>
    Return(Option<&'a dyn Value>),
}

pub struct Instruction<'a> {
    name: String,
    inst: InstKind<'a>,
}

impl<'a> Instruction<'a> {
    pub fn alloca(name: String) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Alloca,
        })
    }

    pub fn store(
        name: String,
        val: &'a dyn Value,
        ptr: &'a dyn Value,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Store(val, ptr),
        })
    }

    pub fn load(name: String, ptr: &'a dyn Value) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Load(ptr),
        })
    }

    pub fn add(
        name: String,
        op0: &'a dyn Value,
        op1: &'a dyn Value,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Add(op0, op1),
        })
    }

    pub fn sub(
        name: String,
        op0: &'a dyn Value,
        op1: &'a dyn Value,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Sub(op0, op1),
        })
    }

    pub fn mul(
        name: String,
        op0: &'a dyn Value,
        op1: &'a dyn Value,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Mul(op0, op1),
        })
    }

    pub fn div(
        name: String,
        op0: &'a dyn Value,
        op1: &'a dyn Value,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Div(op0, op1),
        })
    }

    pub fn jump(name: String, target: &'a BasicBlock) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Jump(target),
        })
    }

    pub fn cjump(
        name: String,
        cond: &'a dyn Value,
        target1: &'a BasicBlock,
        target2: &'a BasicBlock,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::CJump(cond, target1, target2),
        })
    }

    pub fn call(
        name: String,
        callee: String,
        args: Vec<&'a dyn Value>,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Call(callee, args),
        })
    }

    pub fn return_(
        name: String,
        val: Option<&'a dyn Value>,
    ) -> Box<Instruction<'a>> {
        Box::<Instruction>::new(Instruction {
            name,
            inst: InstKind::Return(val),
        })
    }
}

impl<'a> Value for Instruction<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_lvalue(&self) -> bool {
        match &self.inst {
            InstKind::Alloca => true,
            _ => false,
        }
    }
}
