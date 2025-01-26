use std::cell::{Ref, RefCell, RefMut};

mod codegen;
pub use codegen::Codegen;

mod context;
mod func;
mod inst;
mod label;
mod reg;

use context::Context;
use func::Func;
use inst::{ConditionCode, Inst, Memory, RegOrImm};
use label::Label;
use reg::Register;

pub struct Module<'m> {
    ctx: Context<'m>,
    externs: Vec<&'m Label<'m>>,
    functions: RefCell<Vec<&'m Func<'m>>>,
}

impl<'m> Module<'m> {
    pub fn new() -> Module<'m> {
        Module {
            ctx: Context::new(),
            externs: Vec::new(),
            functions: RefCell::new(Vec::new()),
        }
    }

    pub fn context(&self) -> &Context<'m> {
        &self.ctx
    }

    pub fn functions(&self) -> Ref<Vec<&'m Func<'m>>> {
        self.functions.borrow()
    }

    pub fn functions_mut(&self) -> RefMut<Vec<&'m Func<'m>>> {
        self.functions.borrow_mut()
    }

    pub fn dump<W: std::io::Write>(&self, out: &mut W) -> std::io::Result<()> {
        // TODO externs is not supported yet

        for func in self.functions().iter() {
            writeln!(out, "\t.global\t{}", func.prologue().name(),)?;
            writeln!(out, "\t.p2align\t2")?;

            self.dump_label(out, func.prologue())?;
            for label in func.body().iter() {
                self.dump_label(out, label)?;
            }
            self.dump_label(out, func.epilogue())?;
        }

        Ok(())
    }

    fn dump_label<W: std::io::Write>(&self, out: &mut W, label: &Label<'m>) -> std::io::Result<()> {
        writeln!(out, "{}:", label.name())?;
        for inst in label.insts().iter() {
            writeln!(out, "\t{}", inst)?;
        }

        Ok(())
    }
}
