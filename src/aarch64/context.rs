use std::cell::RefCell;

use typed_arena::Arena;

use super::{ConditionCode, Func, Inst, Label, Memory, RegOrImm, Register};

pub struct Context<'m> {
    label: Arena<Label<'m>>,
    func: Arena<Func<'m>>,
    inst: Arena<Inst<'m>>,
    preg: Vec<Register>,
    vreg: Arena<Register>,
}

impl<'m> Context<'m> {
    pub fn new() -> Context<'m> {
        let mut pregs = vec![];
        for i in 0..32 {
            pregs.push(Register::Physical(i));
        }

        Context {
            label: Arena::new(),
            func: Arena::new(),
            inst: Arena::new(),
            preg: pregs,
            vreg: Arena::new(),
        }
    }

    pub fn new_label(&self, name: String) -> &Label<'m> {
        self.label.alloc(Label::new(name))
    }

    pub fn new_func(&self, name: String) -> &mut Func<'m> {
        self.func.alloc(Func::new(name))
    }

    pub fn mov(&self, dst: &'m Register, src: RegOrImm<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Mov {
            dst: RefCell::new(dst),
            src,
        })
    }

    pub fn ldr(&self, dst: &'m Register, src: Memory<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Ldr {
            dst: RefCell::new(dst),
            src,
        })
    }

    pub fn ldp(&self, dst1: &'m Register, dst2: &'m Register, src: Memory<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Ldp {
            dst1: RefCell::new(dst1),
            dst2: RefCell::new(dst2),
            src,
        })
    }

    pub fn str(&self, src: &'m Register, dst: Memory<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Str {
            src: RefCell::new(src),
            dst,
        })
    }

    pub fn stp(&self, src1: &'m Register, src2: &'m Register, dst: Memory<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Stp {
            src1: RefCell::new(src1),
            src2: RefCell::new(src2),
            dst,
        })
    }

    pub fn b(&self, label: &'m Label<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::B { label })
    }

    pub fn cbnz(&self, src: &'m Register, label: &'m Label<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Cbnz {
            src: RefCell::new(src),
            label,
        })
    }

    pub fn bl(&self, callee: &'m Label<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Bl { callee })
    }

    pub fn ret(&self) -> &Inst<'m> {
        self.inst.alloc(Inst::Ret)
    }

    pub fn cmp(&self, src1: &'m Register, src2: RegOrImm<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Cmp {
            src1: RefCell::new(src1),
            src2,
        })
    }

    pub fn cset(&self, dst: &'m Register, cond: ConditionCode) -> &Inst<'m> {
        self.inst.alloc(Inst::Cset {
            dst: RefCell::new(dst),
            cond,
        })
    }

    pub fn add(&self, dst: &'m Register, src1: &'m Register, src2: RegOrImm<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Add {
            dst: RefCell::new(dst),
            src1: RefCell::new(src1),
            src2,
        })
    }

    pub fn sub(&self, dst: &'m Register, src1: &'m Register, src2: RegOrImm<'m>) -> &Inst<'m> {
        self.inst.alloc(Inst::Sub {
            dst: RefCell::new(dst),
            src1: RefCell::new(src1),
            src2,
        })
    }

    pub fn mul(&self, dst: &'m Register, src1: &'m Register, src2: &'m Register) -> &Inst<'m> {
        self.inst.alloc(Inst::Mul {
            dst: RefCell::new(dst),
            src1: RefCell::new(src1),
            src2: RefCell::new(src2),
        })
    }

    pub fn sdiv(&self, dst: &'m Register, src1: &'m Register, src2: &'m Register) -> &Inst<'m> {
        self.inst.alloc(Inst::Sdiv {
            dst: RefCell::new(dst),
            src1: RefCell::new(src1),
            src2: RefCell::new(src2),
        })
    }

    pub fn msub(
        &self,
        dst: &'m Register,
        src1: &'m Register,
        src2: &'m Register,
        src3: &'m Register,
    ) -> &Inst<'m> {
        self.inst.alloc(Inst::Msub {
            dst: RefCell::new(dst),
            src1: RefCell::new(src1),
            src2: RefCell::new(src2),
            src3: RefCell::new(src3),
        })
    }

    pub fn new_vreg(&self, r: Register) -> &Register {
        self.vreg.alloc(r)
    }

    pub fn x(&self, i: usize) -> &Register {
        &self.preg[i]
    }

    pub fn sp(&self) -> &Register {
        // https://en.wikipedia.org/wiki/Calling_convention#ARM_(A64)
        // x31 (SP): Stack pointer or a zero register, depending on context.
        &self.preg[31]
    }
}
