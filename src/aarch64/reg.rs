use std::cmp::Ordering;
use std::fmt;

pub enum Register {
    // https://developer.arm.com/documentation/102374/0102/Registers-in-AArch64---general-purpose-registers
    // 31 general purpose registers: Each register can be used as a 64-bit X register (X0..X30)
    //
    // https://en.wikipedia.org/wiki/Calling_convention#ARM_(A64)
    // x0 to x7: Argument values passed to and results returned from a subroutine.
    // x8 (XR): Indirect return value address.
    // x9 to x15: Local variables, caller saved.
    // x16 (IP0) and x17 (IP1): Intra-Procedure-call scratch registers.
    // x18 (PR): Platform register. Used for some operating-system-specific special purpose, or an
    // additional caller-saved register. x19 to x28: Callee-saved.
    // x29 (FP): Frame pointer.
    // x30 (LR): Procedure link register, used to return from subroutines.
    // x31 (SP): Stack pointer or a zero register, depending on context.
    Physical(u64),
    Virtual(u64),
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::Physical(id) => match id.cmp(&31) {
                Ordering::Equal => write!(f, "sp"),
                Ordering::Less => write!(f, "x{}", id),
                Ordering::Greater => panic!("Invalid physical register id: {}", id),
            },
            Register::Virtual(id) => write!(f, "_t{}", id),
        }
    }
}
