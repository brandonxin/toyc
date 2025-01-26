use std::cell::RefCell;
use std::collections::HashMap;

use self::regalloc::NaiveRegisterAllocator;
use crate::aarch64::{ConditionCode, Context, Func, Label, Memory, Module, RegOrImm, Register};
use crate::ir;

mod regalloc;

pub struct Codegen<'m> {
    unit: &'m Module<'m>,
    ctx: &'m Context<'m>,

    func_map: HashMap<String, &'m Label<'m>>,
}

impl<'m> Codegen<'m> {
    pub fn new(module: &'m Module<'m>) -> Codegen<'m> {
        Codegen {
            unit: module,
            ctx: module.context(),
            func_map: HashMap::new(),
        }
    }

    pub fn unit(&self) -> &Module<'m> {
        self.unit
    }

    pub fn visit_unit(&mut self, unit: &'m ir::Module<'m>, no_regalloc: bool) {
        let functions_ir = unit.functions();
        let mut functions = self.unit.functions_mut();

        for func in functions_ir.iter() {
            let f = self.ctx.new_func(func.name().to_string());
            f.set_prologue(self.ctx.new_label(format!("_{}", func.name())));
            f.set_epilogue(self.ctx.new_label(format!("{}_epilogue", func.name())));
            self.func_map.insert(func.name().to_owned(), f.prologue());
            functions.push(f);

            let mut codegen = FunctionCG::new(self.ctx, &self.func_map, func, f);
            codegen.visit_function(no_regalloc);
        }
    }
}

struct FunctionCG<'m, 'cg> {
    ctx: &'m Context<'m>,
    func_map: &'cg HashMap<String, &'m Label<'m>>,
    func_ir: &'m ir::Func<'m>,
    target: &'m Func<'m>,
    value_map: HashMap<&'m dyn ir::Value, Operand<'m>>,
    block_map: HashMap<&'m ir::BasicBlock<'m>, &'m Label<'m>>,
    curr_label: Option<&'m Label<'m>>,
    next_vreg_id: u64,
    next_stack_offset: i64,
}

impl<'m, 'cg> FunctionCG<'m, 'cg> {
    fn new(
        context: &'m Context<'m>,
        func_map: &'cg HashMap<String, &'m Label<'m>>,
        func_ir: &'m ir::Func<'m>,
        func: &'m Func<'m>,
    ) -> FunctionCG<'m, 'cg> {
        FunctionCG {
            ctx: context,
            func_map,
            func_ir,
            target: func,
            value_map: HashMap::new(),
            block_map: HashMap::new(),
            curr_label: None,
            next_vreg_id: 0,
            next_stack_offset: 0,
        }
    }

    fn visit_function(&'cg mut self, no_regalloc: bool) {
        // https://developer.arm.com/documentation/102374/0102/Procedure-Call-Standard
        // X0-X7 -- Parameter and Result Registers
        for (reg, param) in self.func_ir.params().iter().enumerate() {
            let param = *param;
            let param = param as &dyn ir::Value;
            self.value_map.insert(param, Operand::Reg(self.ctx.x(reg)));
        }

        for constant in self.func_ir.constants().iter() {
            self.value_map
                .insert(*constant, Operand::Imm(constant.value()));
        }

        for block in self.func_ir.blocks().iter() {
            self.block_map.insert(
                *block,
                self.ctx
                    .new_label(format!("{}_{}", self.target.name(), block.name())),
            );
        }

        for block in self.func_ir.blocks().iter() {
            self.visit_block(block);
        }

        if no_regalloc {
            return;
        }
        let mut allocator = NaiveRegisterAllocator::new(self.ctx, self.target, self);
        allocator.run();
    }

    fn visit_block(&mut self, block: &'m ir::BasicBlock<'m>) {
        self.target
            .body_mut()
            .push(self.block_map.get(&block).unwrap());
        self.curr_label = Some(self.block_map.get(&block).unwrap());
        for inst in block.instructions().iter() {
            self.visit_instruction(inst);
        }
    }

    fn get_reg(&mut self, val: &dyn ir::Value) -> &'m Register {
        match *self.value_map.get(&val).unwrap() {
            Operand::Imm(i) => {
                let reg = self.new_vreg();
                self.emit(self.ctx.mov(reg, RegOrImm::Imm(i)));
                reg
            }
            Operand::Reg(r) => r,
            _ => panic!("Expected register operand"),
        }
    }

    fn get_mem(&self, val: &dyn ir::Value) -> Memory<'m> {
        match self.value_map.get(&val).unwrap() {
            Operand::Memory(m) => m.clone(),
            _ => panic!("Expected memory operand"),
        }
    }

    fn get_reg_or_imm(&self, val: &dyn ir::Value) -> RegOrImm<'m> {
        // FIXME If the constant is too large, aarch64 instruction cannot accept it as
        // an immediate operand. We need to split the constant and use a few more
        // instructions to construct the value.
        match self.value_map.get(&val).unwrap() {
            Operand::Imm(i) => RegOrImm::Imm(*i),
            Operand::Reg(r) => RegOrImm::Reg(RefCell::new(r)),
            _ => panic!("Expected register or immediate operand"),
        }
    }

    fn visit_instruction(&mut self, inst: &'m ir::Inst<'m>) {
        match inst.kind() {
            ir::InstKind::Alloca => {
                let stack_slot = self.new_stack_slot();
                self.value_map.insert(inst, Operand::Memory(stack_slot));
            }
            ir::InstKind::Store(val, ptr) => {
                let val = self.get_reg(*val);
                let ptr = self.get_mem(*ptr);

                self.emit(self.ctx.str(val, ptr.clone()));
            }
            ir::InstKind::Load(ptr) => {
                let dst = self.new_vreg();
                let ptr = self.get_mem(*ptr);
                self.value_map.insert(inst, Operand::Reg(dst));
                self.emit(self.ctx.ldr(dst, ptr.clone()));
            }
            ir::InstKind::Eq(lhs, rhs)
            | ir::InstKind::Ne(lhs, rhs)
            | ir::InstKind::Gt(lhs, rhs)
            | ir::InstKind::Ge(lhs, rhs)
            | ir::InstKind::Lt(lhs, rhs)
            | ir::InstKind::Le(lhs, rhs) => {
                let dst = self.new_vreg();
                self.value_map.insert(inst, Operand::Reg(dst));

                let src1 = self.get_reg(*lhs);
                let src2 = self.get_reg_or_imm(*rhs);

                let cc = match inst.kind() {
                    ir::InstKind::Eq(_, _) => ConditionCode::EQ,
                    ir::InstKind::Ne(_, _) => ConditionCode::NE,
                    ir::InstKind::Gt(_, _) => ConditionCode::GT,
                    ir::InstKind::Ge(_, _) => ConditionCode::GE,
                    ir::InstKind::Lt(_, _) => ConditionCode::LT,
                    ir::InstKind::Le(_, _) => ConditionCode::LE,
                    _ => unreachable!(),
                };
                self.emit(self.ctx.cmp(src1, src2));
                self.emit(self.ctx.cset(dst, cc));
            }
            ir::InstKind::Add(lhs, rhs)
            | ir::InstKind::Sub(lhs, rhs)
            | ir::InstKind::Or(lhs, rhs)
            | ir::InstKind::Xor(lhs, rhs)
            | ir::InstKind::And(lhs, rhs)
            | ir::InstKind::LShl(lhs, rhs)
            | ir::InstKind::LShr(lhs, rhs)
            | ir::InstKind::AShr(lhs, rhs) => {
                let dst = self.new_vreg();
                self.value_map.insert(inst, Operand::Reg(dst));

                let src1 = self.get_reg(*lhs);
                let src2 = self.get_reg_or_imm(*rhs);

                match inst.kind() {
                    ir::InstKind::Add(_, _) => {
                        self.emit(self.ctx.add(dst, src1, src2));
                    }
                    ir::InstKind::Sub(_, _) => {
                        self.emit(self.ctx.sub(dst, src1, src2));
                    }
                    ir::InstKind::Or(_, _) => {
                        self.emit(self.ctx.orr(dst, src1, src2));
                    }
                    ir::InstKind::Xor(_, _) => {
                        if let RegOrImm::Imm(u64::MAX) = src2 {
                            self.emit(self.ctx.mvn(dst, src1));
                        } else {
                            self.emit(self.ctx.eor(dst, src1, src2));
                        }
                    }
                    ir::InstKind::And(_, _) => {
                        self.emit(self.ctx.and(dst, src1, src2));
                    }
                    ir::InstKind::LShl(_, _) => {
                        self.emit(self.ctx.lsl(dst, src1, src2));
                    }
                    ir::InstKind::LShr(_, _) => {
                        self.emit(self.ctx.lsr(dst, src1, src2));
                    }
                    ir::InstKind::AShr(_, _) => {
                        self.emit(self.ctx.asr(dst, src1, src2));
                    }
                    _ => unreachable!(),
                }
            }
            ir::InstKind::Mul(lhs, rhs) | ir::InstKind::Div(lhs, rhs) => {
                let dst = self.new_vreg();
                self.value_map.insert(inst, Operand::Reg(dst));

                let src1 = self.get_reg(*lhs);
                let src2 = self.get_reg(*rhs);

                match inst.kind() {
                    ir::InstKind::Mul(_, _) => {
                        self.emit(self.ctx.mul(dst, src1, src2));
                    }
                    ir::InstKind::Div(_, _) => {
                        self.emit(self.ctx.sdiv(dst, src1, src2));
                    }
                    _ => unreachable!(),
                }
            }
            ir::InstKind::Mod(lhs, rhs) => {
                let tmp = self.new_vreg();
                let dst = self.new_vreg();
                self.value_map.insert(inst, Operand::Reg(dst));

                let src1 = self.get_reg(*lhs);
                let src2 = self.get_reg(*rhs);

                self.emit(self.ctx.sdiv(tmp, src1, src2));
                self.emit(self.ctx.msub(dst, tmp, src2, src1));
            }
            ir::InstKind::Jump(target) => {
                let label = self.block_map.get(target).unwrap();
                self.emit(self.ctx.b(label));
            }
            ir::InstKind::CJump(cond, ifbb, elsebb) => {
                let ifbb = *self.block_map.get(ifbb).unwrap();
                let elsebb = *self.block_map.get(elsebb).unwrap();
                let cond = self.get_reg(*cond);

                self.emit(self.ctx.cbnz(cond, ifbb));
                self.emit(self.ctx.b(elsebb));
            }
            ir::InstKind::Call(callee, args) => {
                for (i, arg) in args.iter().enumerate() {
                    let arg = self.get_reg_or_imm(*arg);
                    self.emit(self.ctx.mov(self.ctx.x(i), arg));
                }

                let callee = self.func_map.get(callee.name()).unwrap();
                self.emit(self.ctx.bl(callee));

                let ret = self.new_vreg();
                self.emit(
                    self.ctx
                        .mov(ret, RegOrImm::Reg(RefCell::new(self.ctx.x(0)))),
                );
                self.value_map.insert(inst, Operand::Reg(ret));
            }
            ir::InstKind::Return(Some(val)) => {
                let val = self.get_reg_or_imm(*val);
                self.emit(self.ctx.mov(self.ctx.x(0), val));
                self.emit(self.ctx.b(self.target.epilogue()));
            }
            ir::InstKind::Return(None) => {
                self.emit(self.ctx.ret());
            }
        }
    }

    fn new_vreg(&mut self) -> &'m Register {
        let reg = self.next_vreg_id;
        self.next_vreg_id += 1;

        self.ctx.new_vreg(Register::Virtual(reg))
    }

    fn new_stack_slot(&mut self) -> Memory<'m> {
        let offset = self.next_stack_offset;
        self.next_stack_offset += 8;
        Memory::Stack { offset }
    }

    fn stack_frame_size(&self) -> u64 {
        self.next_stack_offset as u64
    }

    fn emit(&self, inst: &'m super::Inst<'m>) {
        self.curr_label.unwrap().add_instruction(inst);
    }
}

enum Operand<'m> {
    Imm(u64),
    Reg(&'m Register),
    Memory(Memory<'m>),
}
