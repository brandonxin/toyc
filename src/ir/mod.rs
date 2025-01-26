use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt;

mod basicblock;
mod constant;
mod context;
mod func;
mod inst;
mod param;
mod value;

pub use basicblock::BasicBlock;
pub use constant::Constant;
pub use context::Context;
pub use func::Func;
pub use inst::{Inst, InstKind};
pub use param::Param;
pub use value::Value;

mod codegen;
pub use codegen::Codegen;

pub struct Module<'m> {
    context: Context<'m>,
    functions: RefCell<Vec<&'m Func<'m>>>,
    function_table: RefCell<HashMap<String, &'m Func<'m>>>,
}

impl<'m> Module<'m> {
    pub fn new() -> Module<'m> {
        Module {
            context: Context::new(),
            functions: RefCell::new(Vec::new()),
            function_table: RefCell::new(HashMap::new()),
        }
    }

    pub fn context(&self) -> &Context<'m> {
        &self.context
    }

    pub fn functions(&self) -> Ref<Vec<&'m Func<'m>>> {
        self.functions.borrow()
    }

    pub fn add_function(&self, func: &'m Func<'m>) {
        let name = func.name();
        let mut funcs = self.functions.borrow_mut();
        let mut func_table = self.function_table.borrow_mut();
        if func_table.contains_key(name) {
            panic!("Function with the same name '{name}' already exists");
        }
        func_table.insert(String::from(name), func);
        funcs.push(func);
    }

    pub fn get_function(&self, name: &str) -> Option<&'m Func<'m>> {
        self.function_table.borrow_mut().get(name).map(|f| &**f)
    }

    pub fn dump<W: std::io::Write>(&self, out: &mut W) -> std::io::Result<()> {
        for func in self.functions().iter() {
            writeln!(out, "{}", func)?;
        }
        Ok(())
    }
}

impl fmt::Display for Module<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for func in self.functions.borrow().iter() {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}

impl Default for Module<'_> {
    fn default() -> Self {
        Self::new()
    }
}
