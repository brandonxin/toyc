mod scope;

use crate::ast;
use crate::ir;
use scope::NestedScope;

use super::func;

pub struct Codegen<'m> {
    unit: &'m ir::Module<'m>,
    ctx: &'m ir::Context<'m>,
    scope: NestedScope<'m>,
}

impl<'m> Codegen<'m> {
    pub fn new(module: &'m ir::Module<'m>) -> Codegen<'m> {
        Codegen {
            unit: module,
            ctx: module.context(),
            scope: NestedScope::new(),
        }
    }

    fn make_function(&'m self, proto: &ast::FuncDecl) -> &'m mut ir::Func<'m> {
        let mut params = Vec::<&'m ir::Param>::new();

        for param in proto.params() {
            params.push(self.ctx.new_parameter(String::from(param.name())));
        }

        self.ctx.new_function(String::from(proto.name()), params)
    }

    pub fn visit_unit(&'m self, unit: &ast::Module) {
        for decl in unit {
            match decl {
                ast::GlobalDecl::FuncDecl(proto) => {
                    // TODO: Distinguish function and extern function; function
                    // requires an entry block, extern function does not.
                    unimplemented!();
                }
                ast::GlobalDecl::Function(func) => {
                    self.ctx.reset_id();

                    let func_ir = match self.unit.get_function(func.prototype().name()) {
                        Some(func) => {
                            // TODO As we don't handle function declaration yet,
                            // and function cannot be defined twice, if we reach
                            // here, it means the function is already defined.
                            panic!("Function already exists");
                        }
                        None => {
                            let func = &*self.make_function(func.prototype());
                            self.unit.add_function(func);
                            func
                        }
                    };

                    self.visit_func(func, func_ir);
                }
            }
        }
    }

    fn visit_func(&'m self, func_ast: &ast::Func, func_ir: &'m ir::Func<'m>) {
        let params = func_ast.prototype().params();
        let param_values = func_ir.params();

        let _guard = self.scope.new_scope();
        for (param_ast, param_ir) in params.iter().zip(param_values.iter()) {
            let alloca = self.ctx.alloca();
            self.scope.update(param_ast.name(), alloca);
            func_ir.add_instruction(alloca);
        }
        for (param_ast, param_ir) in params.iter().zip(param_values.iter()) {
            let alloca = self.scope.lookup(param_ast.name()).unwrap();
            let param = *param_ir;
            let store = self.ctx.store(param, alloca);
            func_ir.add_instruction(store);
        }

        self.visit_stmt(func_ast.body(), func_ir);
    }

    fn visit_stmt(&'m self, stmt: &ast::Stmt, func_ir: &'m ir::Func<'m>) {
        match stmt {
            ast::Stmt::Block { stmts } => {
                let _guard = self.scope.new_scope();
                for stmt in stmts {
                    self.visit_stmt(stmt, func_ir);
                }
            }
            ast::Stmt::IfElse {
                cond,
                then_stmt,
                else_stmt: Some(else_stmt),
            } => {
                let start_point = func_ir.insert_point();

                let mut cond_val = self.visit_expr(cond, func_ir);
                if cond_val.is_lvalue() {
                    let load = self.ctx.load(cond_val);
                    func_ir.add_instruction(load);
                    cond_val = load;
                }

                // Generate the then block
                let then_block = self.ctx.new_basic_block();
                func_ir.add_block(then_block);
                func_ir.set_insert_point(then_block);
                self.visit_stmt(then_stmt, func_ir);
                let then_end = func_ir.insert_point();

                // Generate the else block
                let else_block = self.ctx.new_basic_block();
                func_ir.add_block(else_block);
                func_ir.set_insert_point(else_block);
                self.visit_stmt(else_stmt, func_ir);
                let else_end = func_ir.insert_point();

                func_ir.set_insert_point(start_point);
                func_ir.add_instruction(self.ctx.cjump(cond_val, then_block, else_block));

                let exit_block = self.ctx.new_basic_block();

                if !then_end.instructions().last().unwrap().is_terminator() {
                    func_ir.set_insert_point(then_end);
                    func_ir.add_instruction(self.ctx.jump(exit_block));
                }

                if !else_end.instructions().last().unwrap().is_terminator() {
                    func_ir.set_insert_point(else_end);
                    func_ir.add_instruction(self.ctx.jump(exit_block));
                }

                // If both then and else block ends with a terminator, exit_block is
                // unreachable. However, there might be code that follows the
                // if-else statement, so we keep it.
                func_ir.add_block(exit_block);
                func_ir.set_insert_point(exit_block);
            }
            ast::Stmt::IfElse {
                cond,
                then_stmt,
                else_stmt: None,
            } => {
                let start_point = func_ir.insert_point();

                let mut cond_val = self.visit_expr(cond, func_ir);
                if cond_val.is_lvalue() {
                    let load = self.ctx.load(cond_val);
                    func_ir.add_instruction(load);
                    cond_val = load;
                }

                // Generate the then block
                let then_block = self.ctx.new_basic_block();
                func_ir.add_block(then_block);
                func_ir.set_insert_point(then_block);
                self.visit_stmt(then_stmt, func_ir);
                let then_end = func_ir.insert_point();

                let exit_block = self.ctx.new_basic_block();

                func_ir.set_insert_point(start_point);
                func_ir.add_instruction(self.ctx.cjump(cond_val, then_block, exit_block));

                if !then_end.instructions().last().unwrap().is_terminator() {
                    func_ir.set_insert_point(then_end);
                    func_ir.add_instruction(self.ctx.jump(exit_block));
                }

                func_ir.add_block(exit_block);
                func_ir.set_insert_point(exit_block);
            }
            ast::Stmt::While { cond, body } => {
                let cond_block = self.ctx.new_basic_block();
                let body_block = self.ctx.new_basic_block();
                let end_block = self.ctx.new_basic_block();
                func_ir.add_block(cond_block);
                func_ir.add_block(body_block);
                func_ir.add_block(end_block);

                // Jump to the condition block
                let jump = self.ctx.jump(cond_block);
                func_ir.add_instruction(jump);

                // Generate the condition block
                func_ir.set_insert_point(cond_block);
                let mut cond_val = self.visit_expr(cond, func_ir);
                if cond_val.is_lvalue() {
                    let load = self.ctx.load(cond_val);
                    func_ir.add_instruction(load);
                    cond_val = load;
                }
                let cjump = self.ctx.cjump(cond_val, body_block, end_block);
                func_ir.add_instruction(cjump);

                // Generate the body block
                func_ir.set_insert_point(body_block);
                self.visit_stmt(body, func_ir);
                let jump = self.ctx.jump(cond_block);
                func_ir.add_instruction(jump);

                func_ir.set_insert_point(end_block);
            }
            ast::Stmt::VarDecl {
                var_name,
                type_name,
                expr,
            } => {
                let alloca = self.ctx.alloca();
                func_ir.add_instruction(alloca);

                self.scope.update(var_name, alloca);

                if let Some(expr) = expr {
                    let value = self.visit_expr(expr, func_ir);
                    let value = if value.is_lvalue() {
                        let load = self.ctx.load(value);
                        func_ir.add_instruction(load);
                        load
                    } else {
                        value
                    };
                    let store = self.ctx.store(value, alloca);
                    func_ir.add_instruction(store);
                }
            }
            ast::Stmt::Return { expr } => {
                let value = expr.as_ref().map(|expr| self.visit_expr(expr, func_ir));
                // If value is Some, and it is an lvalue, load it.
                let value = value.map(|value| {
                    if value.is_lvalue() {
                        let load = self.ctx.load(value);
                        func_ir.add_instruction(load);
                        load
                    } else {
                        value
                    }
                });
                let ret = self.ctx.ret(value);
                func_ir.add_instruction(ret);
            }
            ast::Stmt::ExprStmt { expr } => {
                self.visit_expr(expr, func_ir);
            }
        }
    }

    fn visit_expr(&'m self, expr: &ast::Expr, func_ir: &'m ir::Func<'m>) -> &'m dyn ir::Value {
        match expr {
            ast::Expr::Integer { value } => {
                let constant = self.ctx.new_constant(*value);
                func_ir.add_constant(constant);
                constant
            }
            ast::Expr::Variable { name } => self.scope.lookup(name).unwrap(),
            ast::Expr::Unary { op, operand } => {
                let mut operand_val = self.visit_expr(operand, func_ir);
                if operand_val.is_lvalue() {
                    let load = self.ctx.load(operand_val);
                    func_ir.add_instruction(load);
                    operand_val = load;
                }
                match op {
                    ast::UnaryOp::Neg => {
                        let zero = self.ctx.new_constant(0);
                        let sub = self.ctx.sub(zero, operand_val);
                        func_ir.add_constant(zero);
                        func_ir.add_instruction(sub);
                        sub
                    }
                    ast::UnaryOp::BitwiseNot => {
                        let neg1 = self.ctx.new_constant(u64::MAX);
                        let not = self.ctx.xor(operand_val, neg1);
                        func_ir.add_constant(neg1);
                        func_ir.add_instruction(not);
                        not
                    }
                    _ => unimplemented!(),
                }
            }
            ast::Expr::Binary { op, lhs, rhs } => {
                let mut lhs_val = self.visit_expr(lhs, func_ir);
                let mut rhs_val = self.visit_expr(rhs, func_ir);
                if *op != ast::BinaryOp::Assignment && lhs_val.is_lvalue() {
                    let load = self.ctx.load(lhs_val);
                    func_ir.add_instruction(load);
                    lhs_val = load;
                }
                if rhs_val.is_lvalue() {
                    let load = self.ctx.load(rhs_val);
                    func_ir.add_instruction(load);
                    rhs_val = load;
                }
                match op {
                    ast::BinaryOp::Assignment => {
                        let store = self.ctx.store(rhs_val, lhs_val);
                        func_ir.add_instruction(store);
                        rhs_val
                    }
                    ast::BinaryOp::BitwiseOr => {
                        let or = self.ctx.or(lhs_val, rhs_val);
                        func_ir.add_instruction(or);
                        or
                    }
                    ast::BinaryOp::BitwiseXor => {
                        let xor = self.ctx.xor(lhs_val, rhs_val);
                        func_ir.add_instruction(xor);
                        xor
                    }
                    ast::BinaryOp::BitwiseAnd => {
                        let and = self.ctx.and(lhs_val, rhs_val);
                        func_ir.add_instruction(and);
                        and
                    }
                    ast::BinaryOp::LShift => {
                        let lshift = self.ctx.lshl(lhs_val, rhs_val);
                        func_ir.add_instruction(lshift);
                        lshift
                    }
                    ast::BinaryOp::RShift => {
                        let rshift = self.ctx.ashr(lhs_val, rhs_val);
                        func_ir.add_instruction(rshift);
                        rshift
                    }
                    ast::BinaryOp::Eq => {
                        let eq = self.ctx.eq(lhs_val, rhs_val);
                        func_ir.add_instruction(eq);
                        eq
                    }
                    ast::BinaryOp::Ne => {
                        let ne = self.ctx.ne(lhs_val, rhs_val);
                        func_ir.add_instruction(ne);
                        ne
                    }
                    ast::BinaryOp::Gt => {
                        let gt = self.ctx.gt(lhs_val, rhs_val);
                        func_ir.add_instruction(gt);
                        gt
                    }
                    ast::BinaryOp::Ge => {
                        let ge = self.ctx.ge(lhs_val, rhs_val);
                        func_ir.add_instruction(ge);
                        ge
                    }
                    ast::BinaryOp::Lt => {
                        let lt = self.ctx.lt(lhs_val, rhs_val);
                        func_ir.add_instruction(lt);
                        lt
                    }
                    ast::BinaryOp::Le => {
                        let le = self.ctx.le(lhs_val, rhs_val);
                        func_ir.add_instruction(le);
                        le
                    }
                    ast::BinaryOp::Add => {
                        let add = self.ctx.add(lhs_val, rhs_val);
                        func_ir.add_instruction(add);
                        add
                    }
                    ast::BinaryOp::Sub => {
                        let sub = self.ctx.sub(lhs_val, rhs_val);
                        func_ir.add_instruction(sub);
                        sub
                    }
                    ast::BinaryOp::Mul => {
                        let mul = self.ctx.mul(lhs_val, rhs_val);
                        func_ir.add_instruction(mul);
                        mul
                    }
                    ast::BinaryOp::Div => {
                        let div = self.ctx.div(lhs_val, rhs_val);
                        func_ir.add_instruction(div);
                        div
                    }
                    ast::BinaryOp::Mod => {
                        let modulo = self.ctx.modulo(lhs_val, rhs_val);
                        func_ir.add_instruction(modulo);
                        modulo
                    }
                    _ => unimplemented!(),
                }
            }
            ast::Expr::Call { callee, arguments } => {
                let callee_ir = self.unit.get_function(callee).unwrap();
                let mut args = Vec::<&'m dyn ir::Value>::new();
                for arg in arguments {
                    let arg = self.visit_expr(arg, func_ir);
                    if arg.is_lvalue() {
                        let load = self.ctx.load(arg);
                        func_ir.add_instruction(load);
                        args.push(load);
                    } else {
                        args.push(arg);
                    }
                }
                let call = self.ctx.call(String::from(callee), callee_ir, args);
                func_ir.add_instruction(call);
                call
            }
        }
    }
}
