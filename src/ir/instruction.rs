use super::{BasicBlock, Function, Value};
use std::fmt;

pub enum InstKind<'ctx> {
    Alloca,
    // <0: val> -> *<1: ptr>
    Store(&'ctx dyn Value, &'ctx dyn Value),
    // result := *<0: ptr>
    Load(&'ctx dyn Value),

    // result := <0: op0> + <1: op1>
    Add(&'ctx dyn Value, &'ctx dyn Value),
    // result := <0: op0> - <1: op1>
    Sub(&'ctx dyn Value, &'ctx dyn Value),
    // result := <0: op0> * <1: op1>
    Mul(&'ctx dyn Value, &'ctx dyn Value),
    // result := <0: op0> / <1: op1>
    Div(&'ctx dyn Value, &'ctx dyn Value),

    // goto <0: target>
    Jump(&'ctx BasicBlock<'ctx>),
    // if <0: cond> != $0 goto <1: target1> else goto <2: target2>
    CJump(
        &'ctx dyn Value,
        &'ctx BasicBlock<'ctx>,
        &'ctx BasicBlock<'ctx>,
    ),

    // result := call <0: callee>(<1: args...>)
    Call(&'ctx Function<'ctx>, Vec<&'ctx dyn Value>),
    // return <0: val?>
    Return(Option<&'ctx dyn Value>),
}

pub struct Instruction<'ctx> {
    name: String,
    inst: InstKind<'ctx>,
}

impl<'ctx> Instruction<'ctx> {
    pub fn alloca(name: String) -> Self {
        Self {
            name,
            inst: InstKind::Alloca,
        }
    }

    pub fn store(
        name: String,
        val: &'ctx dyn Value,
        ptr: &'ctx dyn Value,
    ) -> Self {
        Self {
            name,
            inst: InstKind::Store(val, ptr),
        }
    }

    pub fn load(name: String, ptr: &'ctx dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Load(ptr),
        }
    }

    pub fn add(
        name: String,
        op0: &'ctx dyn Value,
        op1: &'ctx dyn Value,
    ) -> Self {
        Self {
            name,
            inst: InstKind::Add(op0, op1),
        }
    }

    pub fn sub(
        name: String,
        op0: &'ctx dyn Value,
        op1: &'ctx dyn Value,
    ) -> Self {
        Self {
            name,
            inst: InstKind::Sub(op0, op1),
        }
    }

    pub fn mul(
        name: String,
        op0: &'ctx dyn Value,
        op1: &'ctx dyn Value,
    ) -> Self {
        Self {
            name,
            inst: InstKind::Mul(op0, op1),
        }
    }

    pub fn div(
        name: String,
        op0: &'ctx dyn Value,
        op1: &'ctx dyn Value,
    ) -> Self {
        Self {
            name,
            inst: InstKind::Div(op0, op1),
        }
    }

    pub fn jump(name: String, target: &'ctx BasicBlock<'ctx>) -> Self {
        Self {
            name,
            inst: InstKind::Jump(target),
        }
    }

    pub fn cjump(
        name: String,
        cond: &'ctx dyn Value,
        target1: &'ctx BasicBlock<'ctx>,
        target2: &'ctx BasicBlock<'ctx>,
    ) -> Self {
        Self {
            name,
            inst: InstKind::CJump(cond, target1, target2),
        }
    }

    pub fn call(
        name: String,
        callee: &'ctx Function<'ctx>,
        args: Vec<&'ctx dyn Value>,
    ) -> Self {
        Self {
            name,
            inst: InstKind::Call(callee, args),
        }
    }

    pub fn ret(name: String, val: Option<&'ctx dyn Value>) -> Self {
        Self {
            name,
            inst: InstKind::Return(val),
        }
    }
}

impl<'ctx> Value for Instruction<'ctx> {
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

impl<'ctx> fmt::Display for Instruction<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.inst {
            InstKind::Alloca => write!(f, "{} = alloca", self.name),
            InstKind::Store(val, ptr) => {
                write!(f, "store {}, {}", val.name(), ptr.name())
            }
            InstKind::Load(ptr) => {
                write!(f, "{} = load {}", self.name, ptr.name())
            }
            InstKind::Add(op0, op1) => {
                write!(f, "{} = add {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Sub(op0, op1) => {
                write!(f, "{} = sub {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Mul(op0, op1) => {
                write!(f, "{} = mul {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Div(op0, op1) => {
                write!(f, "{} = div {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Jump(target) => write!(f, "jump {}", target.name()),
            InstKind::CJump(cond, target1, target2) => {
                write!(
                    f,
                    "cjump {}, {}, {}",
                    cond.name(),
                    target1.name(),
                    target2.name()
                )
            }
            InstKind::Call(callee, args) => {
                write!(f, "{} = call @{}(", self.name, callee.name())?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg.name())?;
                }
                write!(f, ")")
            }
            InstKind::Return(val) => {
                write!(f, "return")?;
                if let Some(val) = val {
                    write!(f, " {}", val.name())?;
                }
                Ok(())
            }
        }
    }
}
