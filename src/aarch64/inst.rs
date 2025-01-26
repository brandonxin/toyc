use std::cell::{Ref, RefCell, RefMut};
use std::fmt;

use super::{Label, Register};

pub enum Inst<'m> {
    Mov {
        dst: RefCell<&'m Register>,
        src: RegOrImm<'m>,
    },
    Ldr {
        dst: RefCell<&'m Register>,
        src: Memory<'m>,
    },
    Ldp {
        dst1: RefCell<&'m Register>,
        dst2: RefCell<&'m Register>,
        src: Memory<'m>,
    },
    Str {
        src: RefCell<&'m Register>,
        dst: Memory<'m>,
    },
    Stp {
        src1: RefCell<&'m Register>,
        src2: RefCell<&'m Register>,
        dst: Memory<'m>,
    },
    B {
        label: &'m Label<'m>,
    },
    Cbnz {
        src: RefCell<&'m Register>,
        label: &'m Label<'m>,
    },
    Bl {
        callee: &'m Label<'m>,
    },
    Ret,

    Cset {
        dst: RefCell<&'m Register>,
        cond: ConditionCode,
    },
    Cmp {
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },

    Orr {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Eor {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    And {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Lsl {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Lsr {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Asr {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Add {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Sub {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RegOrImm<'m>,
    },
    Mul {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RefCell<&'m Register>,
    },
    Sdiv {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RefCell<&'m Register>,
    },
    Msub {
        dst: RefCell<&'m Register>,
        src1: RefCell<&'m Register>,
        src2: RefCell<&'m Register>,
        src3: RefCell<&'m Register>,
    },
    Mvn {
        dst: RefCell<&'m Register>,
        src: RefCell<&'m Register>,
    },
}

impl<'m> Inst<'m> {
    fn collect_vregs_from_reg_or_imm(
        reg_or_imm: &'m RegOrImm<'m>,
        read: &mut Vec<RefMut<&'m Register>>,
    ) {
        match reg_or_imm {
            RegOrImm::Reg(r) => Self::collect_vregs_from_reg(r, read),
            RegOrImm::Imm(_) => {}
        }
    }

    fn collect_vregs_from_mem(mem: &'m Memory<'m>, read: &mut Vec<RefMut<&'m Register>>) {
        match mem {
            Memory::Base { register }
            | Memory::BaseOffset {
                register,
                offset: _,
            } => {
                Self::collect_vregs_from_reg(register, read);
            }
            Memory::Stack { offset: _ } => {}
        }
    }

    fn collect_vregs_from_reg(
        reg: &'m RefCell<&'m Register>,
        read: &mut Vec<RefMut<&'m Register>>,
    ) {
        let r = reg.borrow_mut();
        if let Register::Virtual(_) = *r {
            read.push(r)
        }
    }

    pub fn collect_vregs(
        &'m self,
        read: &mut Vec<RefMut<&'m Register>>,
        written: &mut Vec<RefMut<&'m Register>>,
    ) {
        match self {
            Self::Mov { dst, src } => {
                Self::collect_vregs_from_reg(dst, written);
                Self::collect_vregs_from_reg_or_imm(src, read);
            }
            Self::Ldr { dst, src } => {
                Self::collect_vregs_from_reg(dst, written);
                Self::collect_vregs_from_mem(src, read);
            }
            Self::Ldp { dst1, dst2, src } => {
                Self::collect_vregs_from_reg(dst1, written);
                Self::collect_vregs_from_reg(dst2, written);
                Self::collect_vregs_from_mem(src, read);
            }

            // For STR and STP, the registers inside `dst` will be read, not be written, so we
            // collect them into read.
            Self::Str { src, dst } => {
                Self::collect_vregs_from_reg(src, read);
                Self::collect_vregs_from_mem(dst, read);
            }
            Self::Stp { src1, src2, dst } => {
                Self::collect_vregs_from_reg(src1, read);
                Self::collect_vregs_from_reg(src2, read);
                Self::collect_vregs_from_mem(dst, read);
            }

            Self::Cbnz { src, label: _ } => {
                Self::collect_vregs_from_reg(src, read);
            }
            Self::Cmp { src1, src2 } => {
                Self::collect_vregs_from_reg(src1, read);
                Self::collect_vregs_from_reg_or_imm(src2, read);
            }

            Self::Cset { dst, cond: _ } => {
                Self::collect_vregs_from_reg(dst, written);
            }
            Self::Add { dst, src1, src2 }
            | Self::Sub { dst, src1, src2 }
            | Self::Orr { dst, src1, src2 }
            | Self::Eor { dst, src1, src2 }
            | Self::And { dst, src1, src2 }
            | Self::Lsl { dst, src1, src2 }
            | Self::Lsr { dst, src1, src2 }
            | Self::Asr { dst, src1, src2 } => {
                Self::collect_vregs_from_reg(dst, written);
                Self::collect_vregs_from_reg(src1, read);
                Self::collect_vregs_from_reg_or_imm(src2, read);
            }
            Self::Mul { dst, src1, src2 } | Self::Sdiv { dst, src1, src2 } => {
                Self::collect_vregs_from_reg(dst, written);
                Self::collect_vregs_from_reg(src1, read);
                Self::collect_vregs_from_reg(src2, read);
            }
            Self::Msub {
                dst,
                src1,
                src2,
                src3,
            } => {
                Self::collect_vregs_from_reg(dst, written);
                Self::collect_vregs_from_reg(src1, read);
                Self::collect_vregs_from_reg(src2, read);
                Self::collect_vregs_from_reg(src3, read);
            }
            Self::Mvn { dst, src } => {
                Self::collect_vregs_from_reg(dst, written);
                Self::collect_vregs_from_reg(src, read);
            }

            Self::B { label: _ } | Self::Bl { callee: _ } | Self::Ret => {}
            _ => unimplemented!(),
        }
    }
}

impl fmt::Display for Inst<'_> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Inst::Mov { dst, src } => write!(out, "mov\t{}, {}", dst.borrow(), src)?,
            Inst::Ldr { dst, src } => write!(out, "ldr\t{}, {}", dst.borrow(), src)?,
            Inst::Ldp { dst1, dst2, src } => {
                write!(out, "ldp\t{}, {}, {}", dst1.borrow(), dst2.borrow(), src)?
            }
            Inst::Str { src, dst } => write!(out, "str\t{}, {}", src.borrow(), dst)?,
            Inst::Stp { src1, src2, dst } => {
                write!(out, "stp\t{}, {}, {}", src1.borrow(), src2.borrow(), dst)?
            }
            Inst::B { label } => write!(out, "b\t{}", label.name())?,
            Inst::Cbnz { src, label } => write!(out, "cbnz\t{}, {}", src.borrow(), label.name())?,
            Inst::Bl { callee } => write!(out, "bl\t{}", callee.name())?,
            Inst::Ret => write!(out, "ret")?,
            Inst::Cmp { src1, src2 } => write!(out, "cmp\t{}, {}", src1.borrow(), src2)?,
            Inst::Cset { dst, cond } => write!(out, "cset\t{}, {}", dst.borrow(), cond)?,
            Inst::Orr { dst, src1, src2 } => {
                write!(out, "orr\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Eor { dst, src1, src2 } => {
                write!(out, "eor\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::And { dst, src1, src2 } => {
                write!(out, "and\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Lsl { dst, src1, src2 } => {
                write!(out, "lsl\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Lsr { dst, src1, src2 } => {
                write!(out, "lsr\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Asr { dst, src1, src2 } => {
                write!(out, "asr\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Add { dst, src1, src2 } => {
                write!(out, "add\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Sub { dst, src1, src2 } => {
                write!(out, "sub\t{}, {}, {}", dst.borrow(), src1.borrow(), src2)?
            }
            Inst::Mul { dst, src1, src2 } => write!(
                out,
                "mul\t{}, {}, {}",
                dst.borrow(),
                src1.borrow(),
                src2.borrow()
            )?,
            Inst::Sdiv { dst, src1, src2 } => write!(
                out,
                "sdiv\t{}, {}, {}",
                dst.borrow(),
                src1.borrow(),
                src2.borrow()
            )?,
            Inst::Msub {
                dst,
                src1,
                src2,
                src3,
            } => write!(
                out,
                "msub\t{}, {}, {}, {}",
                dst.borrow(),
                src1.borrow(),
                src2.borrow(),
                src3.borrow()
            )?,
            Inst::Mvn { dst, src } => write!(out, "mvn\t{}, {}", dst.borrow(), src.borrow())?,
        }
        Ok(())
    }
}

// ARM Condition codes
// https://developer.arm.com/documentation/dui0379/e/arm-and-thumb-instructions/condition-codes
pub enum ConditionCode {
    EQ, // Equal
    NE, // Not equal
    CS, // Carry set / unsigned higher or same
    CC, // Carry clear / unsigned lower
    MI, // Minus / negative
    PL, // Plus / positive or zero
    VS, // Overflow
    VC, // No overflow
    HI, // Unsigned higher
    LS, // Unsigned lower or same
    GE, // Signed greater than or equal
    LT, // Signed less than
    GT, // Signed greater than
    LE, // Signed less than or equal
    AL, // Always (unconditional)
}

impl fmt::Display for ConditionCode {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConditionCode::EQ => write!(out, "eq")?,
            ConditionCode::NE => write!(out, "ne")?,
            ConditionCode::CS => write!(out, "cs")?,
            ConditionCode::CC => write!(out, "cc")?,
            ConditionCode::MI => write!(out, "mi")?,
            ConditionCode::PL => write!(out, "pl")?,
            ConditionCode::VS => write!(out, "vs")?,
            ConditionCode::VC => write!(out, "vc")?,
            ConditionCode::HI => write!(out, "hi")?,
            ConditionCode::LS => write!(out, "ls")?,
            ConditionCode::GE => write!(out, "ge")?,
            ConditionCode::LT => write!(out, "lt")?,
            ConditionCode::GT => write!(out, "gt")?,
            ConditionCode::LE => write!(out, "le")?,
            ConditionCode::AL => write!(out, "al")?,
        }
        Ok(())
    }
}

pub enum RegOrImm<'m> {
    Reg(RefCell<&'m Register>),
    Imm(u64),
}

impl fmt::Display for RegOrImm<'_> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegOrImm::Reg(reg) => write!(out, "{}", reg.borrow())?,
            RegOrImm::Imm(imm) => write!(out, "#{}", imm)?,
        }
        Ok(())
    }
}

#[derive(Clone)]
pub enum Memory<'m> {
    Base {
        register: RefCell<&'m Register>,
    },
    BaseOffset {
        register: RefCell<&'m Register>,
        offset: i64,
    },
    Stack {
        offset: i64,
    },
}

impl fmt::Display for Memory<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base { register } => write!(f, "[{}]", *register.borrow()),
            Self::BaseOffset { register, offset } => {
                if *offset == 0 {
                    write!(f, "[{}]", *register.borrow())
                } else {
                    write!(f, "[{}, #{}]", *register.borrow(), offset)
                }
            }
            Self::Stack { offset } => {
                if *offset == 0 {
                    write!(f, "[sp]")
                } else {
                    write!(f, "[sp, #{}]", offset)
                }
            }
        }
    }
}
