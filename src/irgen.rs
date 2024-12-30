use std::cell::RefCell;
use std::collections::HashMap;
use typed_arena;

use super::ast;
use super::ir;

pub struct NestedScope<'ctx> {
    stack: RefCell<Vec<HashMap<String, &'ctx dyn ir::Value>>>,
}

impl<'ctx> NestedScope<'ctx> {
    pub fn new() -> NestedScope<'ctx> {
        NestedScope {
            stack: RefCell::new(vec![HashMap::new()]),
        }
    }

    pub fn new_scope(&'ctx self) -> ScopeGuard<'ctx> {
        let mut stack = self.stack.borrow_mut();
        stack.push(HashMap::new());
        ScopeGuard::new(self)
    }

    pub fn lookup(&self, name: &str) -> Option<&'ctx dyn ir::Value> {
        let stack = self.stack.borrow();
        for scope in stack.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(*val);
            }
        }
        None
    }

    pub fn update(&self, name: &str, val: &'ctx dyn ir::Value) {
        let mut stack = self.stack.borrow_mut();
        stack.last_mut().unwrap().insert(String::from(name), val);
    }
}

pub struct ScopeGuard<'ctx> {
    scope: &'ctx NestedScope<'ctx>,
}

impl<'ctx> ScopeGuard<'ctx> {
    fn new(scope: &'ctx NestedScope<'ctx>) -> ScopeGuard<'ctx> {
        ScopeGuard { scope }
    }
}

impl<'ctx> Drop for ScopeGuard<'ctx> {
    fn drop(&mut self) {
        let mut stack = self.scope.stack.borrow_mut();
        stack.pop();
    }
}

struct Arena<'ctx> {
    next_id: RefCell<usize>,
    functions: typed_arena::Arena<ir::Function<'ctx>>,
    parameters: typed_arena::Arena<ir::Parameter>,
    basic_blocks: typed_arena::Arena<ir::BasicBlock<'ctx>>,
    instructions: typed_arena::Arena<ir::Instruction<'ctx>>,
    constants: typed_arena::Arena<ir::Constant>,
}

impl<'ctx> Arena<'ctx> {
    pub fn new() -> Arena<'ctx> {
        Arena {
            next_id: RefCell::new(0),
            functions: typed_arena::Arena::new(),
            parameters: typed_arena::Arena::new(),
            basic_blocks: typed_arena::Arena::new(),
            instructions: typed_arena::Arena::new(),
            constants: typed_arena::Arena::new(),
        }
    }

    pub fn reset_id(&self) {
        *self.next_id.borrow_mut() = 0;
    }

    fn next_id(&self) -> usize {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() += 1;
        id
    }

    fn next_name(&self) -> String {
        format!("%{}", self.next_id())
    }

    pub fn new_function(
        &'ctx self,
        name: String,
        params: Vec<&'ctx ir::Parameter>,
    ) -> &'ctx mut ir::Function<'ctx> {
        self.functions.alloc(ir::Function::new(
            name,
            params,
            self.new_basic_block(),
        ))
    }

    pub fn new_parameter(&'ctx self, name: String) -> &'ctx ir::Parameter {
        self.parameters.alloc(ir::Parameter::new(self.next_name()))
    }

    pub fn new_basic_block(&'ctx self) -> &'ctx ir::BasicBlock<'ctx> {
        self.basic_blocks.alloc(ir::BasicBlock::new(self.next_id()))
    }

    pub fn alloca(&'ctx self) -> &'ctx ir::Instruction<'ctx> {
        self.instructions
            .alloc(ir::Instruction::alloca(self.next_name()))
    }

    pub fn store(
        &'ctx self,
        value: &'ctx dyn ir::Value,
        ptr: &'ctx dyn ir::Value,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions.alloc(ir::Instruction::store(
            self.next_name(),
            value,
            ptr,
        ))
    }

    pub fn load(
        &'ctx self,
        ptr: &'ctx dyn ir::Value,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions
            .alloc(ir::Instruction::load(self.next_name(), ptr))
    }

    pub fn add(
        &'ctx self,
        op0: &'ctx dyn ir::Value,
        op1: &'ctx dyn ir::Value,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions.alloc(ir::Instruction::add(
            self.next_name(),
            op0,
            op1,
        ))
    }

    pub fn sub(
        &'ctx self,
        op0: &'ctx dyn ir::Value,
        op1: &'ctx dyn ir::Value,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions.alloc(ir::Instruction::sub(
            self.next_name(),
            op0,
            op1,
        ))
    }

    pub fn mul(
        &'ctx self,
        op0: &'ctx dyn ir::Value,
        op1: &'ctx dyn ir::Value,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions.alloc(ir::Instruction::mul(
            self.next_name(),
            op0,
            op1,
        ))
    }

    pub fn div(
        &'ctx self,
        op0: &'ctx dyn ir::Value,
        op1: &'ctx dyn ir::Value,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions.alloc(ir::Instruction::div(
            self.next_name(),
            op0,
            op1,
        ))
    }

    pub fn jump(
        &'ctx self,
        target: &'ctx ir::BasicBlock<'ctx>,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions
            .alloc(ir::Instruction::jump(self.next_name(), target))
    }

    pub fn cjump(
        &'ctx self,
        cond: &'ctx dyn ir::Value,
        then_block: &'ctx ir::BasicBlock<'ctx>,
        else_block: &'ctx ir::BasicBlock<'ctx>,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions.alloc(ir::Instruction::cjump(
            self.next_name(),
            cond,
            then_block,
            else_block,
        ))
    }

    pub fn call(
        &'ctx self,
        name: String,
        callee: &'ctx ir::Function<'ctx>,
        args: Vec<&'ctx dyn ir::Value>,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions
            .alloc(ir::Instruction::call(name, callee, args))
    }

    pub fn ret(
        &'ctx self,
        value: Option<&'ctx dyn ir::Value>,
    ) -> &'ctx ir::Instruction<'ctx> {
        self.instructions
            .alloc(ir::Instruction::ret(self.next_name(), value))
    }

    pub fn new_constant(&'ctx self, value: u64) -> &'ctx ir::Constant {
        self.constants
            .alloc(ir::Constant::new(self.next_name(), value))
    }
}

pub struct Context<'ctx> {
    unit: ir::CompilationUnit<'ctx>,
    arena: Arena<'ctx>,
    scope: NestedScope<'ctx>,
}

impl<'ctx> Context<'ctx> {
    pub fn new() -> Context<'ctx> {
        Context {
            unit: ir::CompilationUnit::new(),
            arena: Arena::new(),
            scope: NestedScope::new(),
        }
    }

    pub fn unit(&self) -> &ir::CompilationUnit<'ctx> {
        &self.unit
    }

    fn make_function(
        &'ctx self,
        proto: &ast::Prototype,
    ) -> &'ctx mut ir::Function<'ctx> {
        let mut params = Vec::<&'ctx ir::Parameter>::new();

        for param in proto.params() {
            params
                .push(self.arena.new_parameter(String::from(param.get_name())));
        }

        self.arena.new_function(String::from(proto.name()), params)
    }

    pub fn visit_unit(&'ctx self, unit: &ast::CompilationUnit) {
        for decl in unit {
            match decl {
                ast::Declaration::Prototype(proto) => {
                    // TODO: Distinguish function and extern function; function
                    // requires an entry block, extern function does not.
                    unimplemented!();
                }
                ast::Declaration::Function(func) => {
                    self.arena.reset_id();

                    let func_ir =
                        match self.unit.get_function(func.prototype().name()) {
                            Some(func) => {
                                // TODO As we don't handle function declaration yet,
                                // and function cannot be defined twice, if we reach
                                // here, it means the function is already defined.
                                panic!("Function already exists");
                            }
                            None => {
                                let func =
                                    &*self.make_function(func.prototype());
                                self.unit.add_function(func);
                                func
                            }
                        };

                    self.visit_func(func, func_ir);
                }
            }
        }
    }

    fn visit_func(
        &'ctx self,
        func_ast: &ast::Function,
        func_ir: &'ctx ir::Function<'ctx>,
    ) {
        let params = func_ast.prototype().params();
        let param_values = func_ir.params();

        let _guard = self.scope.new_scope();
        for (param_ast, param_ir) in params.iter().zip(param_values.iter()) {
            let alloca = self.arena.alloca();
            self.scope.update(param_ast.get_name(), alloca);
            func_ir.add_instruction(alloca);
        }
        for (param_ast, param_ir) in params.iter().zip(param_values.iter()) {
            let alloca = self.scope.lookup(param_ast.get_name()).unwrap();
            let param = *param_ir;
            let store = self.arena.store(param, alloca);
            func_ir.add_instruction(store);
        }

        self.visit_stmt(func_ast.body(), func_ir);
    }

    fn visit_stmt(
        &'ctx self,
        stmt: &ast::Stmt,
        func_ir: &'ctx ir::Function<'ctx>,
    ) {
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
                else_stmt,
            } => {
                let then_block = self.arena.new_basic_block();
                let else_block = self.arena.new_basic_block();
                let end_block = self.arena.new_basic_block();
                func_ir.add_block(then_block);
                func_ir.add_block(else_block);
                func_ir.add_block(end_block);

                let mut cond_val = self.visit_expr(cond, func_ir);
                if cond_val.is_lvalue() {
                    let load = self.arena.load(cond_val);
                    func_ir.add_instruction(load);
                    cond_val = load;
                }
                if else_stmt.is_some() {
                    let cjump =
                        self.arena.cjump(cond_val, then_block, else_block);
                    func_ir.add_instruction(cjump);
                } else {
                    let cjump =
                        self.arena.cjump(cond_val, then_block, end_block);
                    func_ir.add_instruction(cjump);
                }

                // Generate the then block
                func_ir.set_insert_point(then_block);
                self.visit_stmt(then_stmt, func_ir);
                let jump = self.arena.jump(end_block);
                func_ir.add_instruction(jump);

                // Generate the else block
                if let Some(else_stmt) = else_stmt {
                    func_ir.set_insert_point(else_block);
                    self.visit_stmt(else_stmt, func_ir);
                    let jump = self.arena.jump(end_block);
                    func_ir.add_instruction(jump);
                }

                func_ir.set_insert_point(end_block);
            }
            ast::Stmt::While { cond, body } => {
                let cond_block = self.arena.new_basic_block();
                let body_block = self.arena.new_basic_block();
                let end_block = self.arena.new_basic_block();
                func_ir.add_block(cond_block);
                func_ir.add_block(body_block);
                func_ir.add_block(end_block);

                // Jump to the condition block
                let jump = self.arena.jump(cond_block);
                func_ir.add_instruction(jump);

                // Generate the condition block
                func_ir.set_insert_point(cond_block);
                let mut cond_val = self.visit_expr(cond, func_ir);
                if cond_val.is_lvalue() {
                    let load = self.arena.load(cond_val);
                    func_ir.add_instruction(load);
                    cond_val = load;
                }
                let cjump = self.arena.cjump(cond_val, body_block, end_block);
                func_ir.add_instruction(cjump);

                // Generate the body block
                func_ir.set_insert_point(body_block);
                self.visit_stmt(body, func_ir);
                let jump = self.arena.jump(cond_block);
                func_ir.add_instruction(jump);

                func_ir.set_insert_point(end_block);
            }
            ast::Stmt::VarDecl {
                var_name,
                type_name,
                expr,
            } => {
                let alloca = self.arena.alloca();
                self.scope.update(var_name, alloca);

                if let Some(expr) = expr {
                    let value = self.visit_expr(expr, func_ir);
                    let store = self.arena.store(value, alloca);
                    func_ir.add_instruction(store);
                }
            }
            ast::Stmt::Return { expr } => {
                let value = match expr {
                    Some(expr) => Some(self.visit_expr(expr, func_ir)),
                    None => None,
                };
                let ret = self.arena.ret(value);
                func_ir.add_instruction(ret);
            }
            ast::Stmt::ExprStmt { expr } => {
                self.visit_expr(expr, func_ir);
            }
        }
    }

    fn visit_expr(
        &'ctx self,
        expr: &ast::Expr,
        func_ir: &'ctx ir::Function<'ctx>,
    ) -> &'ctx dyn ir::Value {
        match expr {
            ast::Expr::Integer { value } => self.arena.new_constant(*value),
            ast::Expr::Variable { name } => self.scope.lookup(name).unwrap(),
            ast::Expr::Unary { op, operand } => {
                let mut operand_val = self.visit_expr(operand, func_ir);
                if operand_val.is_lvalue() {
                    let load = self.arena.load(operand_val);
                    func_ir.add_instruction(load);
                    operand_val = load;
                }
                match op {
                    ast::UnaryOp::Neg => {
                        let zero = self.arena.new_constant(0);
                        let sub = self.arena.sub(zero, operand_val);
                        func_ir.add_instruction(sub);
                        sub
                    }
                    _ => unimplemented!(),
                }
            }
            ast::Expr::Binary { op, lhs, rhs } => {
                let mut lhs_val = self.visit_expr(lhs, func_ir);
                let mut rhs_val = self.visit_expr(rhs, func_ir);
                if *op != ast::BinaryOp::Assignment && lhs_val.is_lvalue() {
                    let load = self.arena.load(lhs_val);
                    func_ir.add_instruction(load);
                    lhs_val = load;
                }
                if rhs_val.is_lvalue() {
                    let load = self.arena.load(rhs_val);
                    func_ir.add_instruction(load);
                    rhs_val = load;
                }
                match op {
                    ast::BinaryOp::Assignment => {
                        let store = self.arena.store(rhs_val, lhs_val);
                        func_ir.add_instruction(store);
                        rhs_val
                    }
                    ast::BinaryOp::Add => {
                        let add = self.arena.add(lhs_val, rhs_val);
                        func_ir.add_instruction(add);
                        add
                    }
                    ast::BinaryOp::Sub => {
                        let sub = self.arena.sub(lhs_val, rhs_val);
                        func_ir.add_instruction(sub);
                        sub
                    }
                    ast::BinaryOp::Mul => {
                        let mul = self.arena.mul(lhs_val, rhs_val);
                        func_ir.add_instruction(mul);
                        mul
                    }
                    ast::BinaryOp::Div => {
                        let div = self.arena.div(lhs_val, rhs_val);
                        func_ir.add_instruction(div);
                        div
                    }
                    _ => unimplemented!(),
                }
            }
            ast::Expr::Call { callee, arguments } => {
                let callee_ir = self.unit.get_function(callee).unwrap();
                let mut args = Vec::<&'ctx dyn ir::Value>::new();
                for arg in arguments {
                    args.push(self.visit_expr(arg, func_ir));
                }
                let call =
                    self.arena.call(String::from(callee), callee_ir, args);
                func_ir.add_instruction(call);
                call
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn fib() {
        let src = String::from(
            r"func fib(n: Int64) : Int64 {
    if n {
        if n - 1 {
            return fib(n - 1) + fib(n - 2);
        } else {
            return 1;
        }
    } else {
        return 1;
    }
}

",
        );

        let mut parser = Parser::new(src.as_bytes());
        let mut unit = ast::CompilationUnit::new();
        parser.parse(&mut unit);

        let mut contex = Context::new();
        contex.visit_unit(&unit);

        let expected = r"define @fib(%0) {
bb_1:
	%2 = alloca
	store %0, %2
	%7 = load %2
	cjump %7, bb_4, bb_5
bb_4:
	%13 = load %2
	%14 = sub %13, $1
	cjump %14, bb_9, bb_10
bb_5:
	return $1
	jump bb_6
bb_6:
bb_9:
	%17 = load %2
	%18 = sub %17, $1
	fib = call @fib(%18)
	%20 = load %2
	%21 = sub %20, $2
	fib = call @fib(%21)
	%22 = add fib, fib
	return %22
	jump bb_11
bb_10:
	return $1
	jump bb_11
bb_11:
	jump bb_6
}
";
        assert_eq!(format!("{}", contex.unit()), expected);
    }
}
