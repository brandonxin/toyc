use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::aarch64::codegen::{Context, FunctionCG};
use crate::aarch64::inst::{Memory, RegOrImm};
use crate::aarch64::{Func, Inst, Label, Register};

pub struct NaiveRegisterAllocator<'m, 'cg> {
    ctx: &'m Context<'m>,
    func: &'m Func<'m>,
    func_cg: &'cg mut FunctionCG<'m, 'cg>,
    map: HashMap<u64, Memory<'m>>,
}

impl<'m, 'cg> NaiveRegisterAllocator<'m, 'cg> {
    pub fn new(
        ctx: &'m Context<'m>,
        func: &'m Func<'m>,
        func_cg: &'cg mut FunctionCG<'m, 'cg>,
    ) -> NaiveRegisterAllocator<'m, 'cg> {
        NaiveRegisterAllocator {
            ctx,
            func,
            func_cg,
            map: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        // Process all labels -- Replace all virtual registers with physical
        // registers.
        self.process_label(self.func.prologue());
        for label in self.func.body().iter() {
            self.process_label(label);
        }
        self.process_label(self.func.epilogue());

        // Now all virtual registers gone, and we know the stack frame size.
        // Insert instructions for extending stack, saving and adjusting
        // frame pointer register.
        let frame_size = self.func_cg.stack_frame_size();
        let frame_size = (frame_size + 15) & !15;
        {
            let mut prologue = self.func.prologue().insts_mut();

            prologue.insert(
                0,
                self.ctx
                    .add(self.ctx.x(29), self.ctx.sp(), RegOrImm::Imm(frame_size)),
            );

            prologue.insert(
                0,
                self.ctx.stp(
                    self.ctx.x(29),
                    self.ctx.x(30),
                    Memory::Stack {
                        offset: frame_size as i64,
                    },
                ),
            );

            prologue.insert(
                0,
                self.ctx
                    .sub(self.ctx.sp(), self.ctx.sp(), RegOrImm::Imm(frame_size + 16)),
            );
        }

        {
            let mut epilogue = self.func.epilogue().insts_mut();

            epilogue.push(self.ctx.ldp(
                self.ctx.x(29),
                self.ctx.x(30),
                Memory::Stack {
                    offset: frame_size as i64,
                },
            ));

            epilogue.push(self.ctx.add(
                self.ctx.sp(),
                self.ctx.sp(),
                RegOrImm::Imm(frame_size + 16),
            ));

            epilogue.push(self.ctx.ret());
        }
    }

    fn process_label(&mut self, label: &Label<'m>) {
        let mut insts = label.insts_mut();
        let mut i = 0;
        while i < insts.len() {
            // If we are loading to a virtual register, we remember the
            // memory operand and erase this instruction. In this way,
            // we postpone the load until we need that value.
            if let Inst::Ldr { dst, src } = insts[i] {
                if let Register::Virtual(r) = *dst.borrow() {
                    self.map.insert(*r, src.clone());
                    insts.remove(i);
                    continue;
                }
                // The target operand doesn't matter, because the
                // instruction is removed.
            }

            // Otherwise, we collect all the virtual registers that are
            // used as source operands and destionation operands
            // respectively. All source virtual registers should already
            // have a stack slot assigned. We insert a load instruction
            // for each source virtual register. All destination virtual
            // registers will now be assigned a stack slot. We replace
            // the destination virtual register with a physical register
            // and store the value to the assigned stack slot.
            let mut read = vec![];
            let mut written = vec![];
            insts[i].collect_vregs(&mut read, &mut written);

            // Each read virtual register in this instruction should
            // have a correspondng stack slot. We load them to a
            // physical register, then replace the virtual register with
            // the physical register.
            // FIXME We should check if the designated physical register is used by the
            // original instruction.
            for (j, r) in read.iter_mut().enumerate() {
                let Register::Virtual(id) = **r else {
                    panic!("not a virtual register");
                };
                let preg = self.ctx.x(8 + j);
                let ptr = self.map.get(id).unwrap();
                insts.insert(i + j, self.ctx.ldr(preg, ptr.clone()));

                **r = preg;
            }

            // Let i points to the original instruction
            i += read.len();

            // Each written virtal register in this instruction will be
            // spilled to a new stack slot. We replace the virtual
            // register with a physical one, then store the physical one
            // into the slot.
            // FIXME We should check if the designated physical register is used by the
            // original instruction.
            for (j, vreg) in written.iter_mut().enumerate() {
                let Register::Virtual(id) = **vreg else {
                    panic!("not a virtual register");
                };
                let preg = self.ctx.x(8 + j + read.len());
                let ptr = match self.map.entry(*id) {
                    Entry::Occupied(e) => e.get().clone(),
                    Entry::Vacant(e) => {
                        let ptr = self.func_cg.new_stack_slot();
                        e.insert(ptr.clone());
                        ptr
                    }
                };
                insts.insert(i + j + 1, self.ctx.str(preg, ptr));

                **vreg = preg;
            }

            i += written.len();
            i += 1;
        }
    }
}
