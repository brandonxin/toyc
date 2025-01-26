use super::{BasicBlock, Func, Value};
use std::fmt;

pub enum InstKind<'m> {
    Alloca,
    // <0: val> -> *<1: ptr>
    Store(&'m dyn Value, &'m dyn Value),
    // result := *<0: ptr>
    Load(&'m dyn Value),

    Eq(&'m dyn Value, &'m dyn Value),
    Ne(&'m dyn Value, &'m dyn Value),
    Gt(&'m dyn Value, &'m dyn Value),
    Ge(&'m dyn Value, &'m dyn Value),
    Lt(&'m dyn Value, &'m dyn Value),
    Le(&'m dyn Value, &'m dyn Value),

    // result := <0: op0> + <1: op1>
    Add(&'m dyn Value, &'m dyn Value),
    // result := <0: op0> - <1: op1>
    Sub(&'m dyn Value, &'m dyn Value),
    // result := <0: op0> * <1: op1>
    Mul(&'m dyn Value, &'m dyn Value),
    // result := <0: op0> / <1: op1>
    Div(&'m dyn Value, &'m dyn Value),
    // result := <0: op0> % <1: op1>
    Mod(&'m dyn Value, &'m dyn Value),

    // goto <0: target>
    Jump(&'m BasicBlock<'m>),
    // if <0: cond> != $0 goto <1: target1> else goto <2: target2>
    CJump(&'m dyn Value, &'m BasicBlock<'m>, &'m BasicBlock<'m>),

    // result := call <0: callee>(<1: args...>)
    Call(&'m Func<'m>, Vec<&'m dyn Value>),
    // return <0: val?>
    Return(Option<&'m dyn Value>),
}

pub struct Inst<'m> {
    name: String,
    inst: InstKind<'m>,
}

impl<'m> Inst<'m> {
    pub fn kind(&self) -> &InstKind<'m> {
        &self.inst
    }

    pub fn is_terminator(&self) -> bool {
        matches!(
            &self.inst,
            InstKind::Jump(_) | InstKind::CJump(_, _, _) | InstKind::Return(_)
        )
    }

    pub fn alloca(name: String) -> Self {
        Self {
            name,
            inst: InstKind::Alloca,
        }
    }

    pub fn store(name: String, val: &'m dyn Value, ptr: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Store(val, ptr),
        }
    }

    pub fn load(name: String, ptr: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Load(ptr),
        }
    }

    pub fn eq(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Eq(op0, op1),
        }
    }

    pub fn ne(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Ne(op0, op1),
        }
    }

    pub fn gt(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Gt(op0, op1),
        }
    }

    pub fn ge(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Ge(op0, op1),
        }
    }

    pub fn lt(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Lt(op0, op1),
        }
    }

    pub fn le(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Le(op0, op1),
        }
    }

    pub fn add(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Add(op0, op1),
        }
    }

    pub fn sub(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Sub(op0, op1),
        }
    }

    pub fn mul(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Mul(op0, op1),
        }
    }

    pub fn div(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Div(op0, op1),
        }
    }

    pub fn modulo(name: String, op0: &'m dyn Value, op1: &'m dyn Value) -> Self {
        Self {
            name,
            inst: InstKind::Mod(op0, op1),
        }
    }

    pub fn jump(name: String, target: &'m BasicBlock<'m>) -> Self {
        Self {
            name,
            inst: InstKind::Jump(target),
        }
    }

    pub fn cjump(
        name: String,
        cond: &'m dyn Value,
        target1: &'m BasicBlock<'m>,
        target2: &'m BasicBlock<'m>,
    ) -> Self {
        Self {
            name,
            inst: InstKind::CJump(cond, target1, target2),
        }
    }

    pub fn call(name: String, callee: &'m Func<'m>, args: Vec<&'m dyn Value>) -> Self {
        Self {
            name,
            inst: InstKind::Call(callee, args),
        }
    }

    pub fn ret(name: String, val: Option<&'m dyn Value>) -> Self {
        Self {
            name,
            inst: InstKind::Return(val),
        }
    }
}

impl Value for Inst<'_> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_lvalue(&self) -> bool {
        matches!(&self.inst, InstKind::Alloca)
    }
}

impl fmt::Display for Inst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.inst {
            InstKind::Alloca => write!(f, "{} = alloca", self.name),
            InstKind::Store(val, ptr) => {
                write!(f, "store {}, {}", val.name(), ptr.name())
            }
            InstKind::Load(ptr) => {
                write!(f, "{} = load {}", self.name, ptr.name())
            }
            InstKind::Eq(op0, op1) => {
                write!(f, "{} = eq {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Ne(op0, op1) => {
                write!(f, "{} = ne {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Gt(op0, op1) => {
                write!(f, "{} = gt {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Ge(op0, op1) => {
                write!(f, "{} = ge {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Lt(op0, op1) => {
                write!(f, "{} = lt {}, {}", self.name, op0.name(), op1.name())
            }
            InstKind::Le(op0, op1) => {
                write!(f, "{} = le {}, {}", self.name, op0.name(), op1.name())
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
            InstKind::Mod(op0, op1) => {
                write!(f, "{} = mod {}, {}", self.name, op0.name(), op1.name())
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
