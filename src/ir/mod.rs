use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    fmt,
};

pub mod instruction;

pub use instruction::Instruction;

pub trait Value {
    fn name(&self) -> &str;

    fn is_lvalue(&self) -> bool {
        false
    }
}

pub struct CompilationUnit<'ctx> {
    functions: RefCell<Vec<&'ctx Function<'ctx>>>,
    function_table: RefCell<HashMap<String, &'ctx Function<'ctx>>>,
}

impl<'ctx> CompilationUnit<'ctx> {
    pub fn new() -> CompilationUnit<'ctx> {
        CompilationUnit {
            functions: RefCell::new(Vec::new()),
            function_table: RefCell::new(HashMap::new()),
        }
    }

    pub fn add_function(&self, func: &'ctx Function<'ctx>) {
        let name = func.name();
        let mut funcs = self.functions.borrow_mut();
        let mut func_table = self.function_table.borrow_mut();
        if func_table.contains_key(name) {
            panic!("Function with the same name '{name}' already exists");
        }
        func_table.insert(String::from(name), func);
        funcs.push(func);
    }

    pub fn get_function(&self, name: &str) -> Option<&'ctx Function<'ctx>> {
        self.function_table.borrow_mut().get(name).map(|f| &**f)
    }
}

impl<'ctx> fmt::Display for CompilationUnit<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for func in self.functions.borrow().iter() {
            write!(f, "{}\n", func)?;
        }
        Ok(())
    }
}

pub struct Function<'ctx> {
    name: String,
    params: Vec<&'ctx Parameter>,

    blocks: RefCell<Vec<&'ctx BasicBlock<'ctx>>>,
    insert_point: RefCell<&'ctx BasicBlock<'ctx>>,
}

impl<'ctx> Function<'ctx> {
    pub fn new(
        name: String,
        params: Vec<&'ctx Parameter>,
        entry: &'ctx BasicBlock<'ctx>,
    ) -> Function<'ctx> {
        Function {
            name,
            params,
            blocks: RefCell::new(vec![entry]),
            insert_point: RefCell::new(entry),
        }
    }

    pub fn add_block(&self, block: &'ctx BasicBlock<'ctx>) {
        self.blocks.borrow_mut().push(block);
    }

    pub fn set_insert_point(&self, block: &'ctx BasicBlock<'ctx>) {
        *self.insert_point.borrow_mut() = block;
    }

    pub fn add_instruction(&self, inst: &'ctx Instruction<'ctx>) {
        self.insert_point.borrow().add_instruction(inst);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &Vec<&'ctx Parameter> {
        &self.params
    }
}

impl<'ctx> fmt::Display for Function<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO If this function is just a prototype, print 'extern' instead of
        // 'define'
        write!(f, "define @{}(", self.name)?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param.name())?;
        }
        write!(f, ") {{")?;
        for block in self.blocks.borrow().iter() {
            write!(f, "\n{}", block)?;
        }
        write!(f, "\n}}")
    }
}

pub struct Parameter {
    name: String,
}

impl Parameter {
    pub fn new(name: String) -> Parameter {
        Parameter { name }
    }
}

impl Value for Parameter {
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct BasicBlock<'ctx> {
    name: String,
    instructions: RefCell<Vec<&'ctx Instruction<'ctx>>>,
}

impl<'ctx> BasicBlock<'ctx> {
    pub fn new(id: usize) -> BasicBlock<'ctx> {
        BasicBlock {
            name: format!("bb_{}", id),
            instructions: RefCell::new(Vec::new()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_instruction(&self, inst: &'ctx Instruction<'ctx>) {
        self.instructions.borrow_mut().push(inst);
    }

    pub fn get_instructions(&self) -> Ref<Vec<&'ctx Instruction<'ctx>>> {
        self.instructions.borrow()
    }
}

impl<'ctx> fmt::Display for BasicBlock<'ctx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.name)?;
        for inst in self.instructions.borrow().iter() {
            write!(f, "\n\t{}", inst)?;
        }
        Ok(())
    }
}

pub struct Constant {
    name: String,
    value: u64,
}

impl Constant {
    pub fn new(_name: String, value: u64) -> Constant {
        Constant {
            name: format!("${value}"),
            value,
        }
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Value for Constant {
    fn name(&self) -> &str {
        &self.name
    }
}
