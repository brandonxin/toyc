use std::collections::HashMap;

pub mod instruction;

use instruction::Instruction;

pub trait Value {
    fn name(&self) -> &str;

    fn is_lvalue(&self) -> bool {
        false
    }
}

pub struct CompilationUnit<'a> {
    functions: Vec<Box<Function<'a>>>,
    function_table: HashMap<String, &'a Function<'a>>,
}

pub struct Function<'a> {
    name: String,
    params: Vec<Box<Parameter>>,
    blocks: Vec<Box<BasicBlock<'a>>>,
    constants: Vec<Box<Constant>>,
    insert_point: Option<&'a BasicBlock<'a>>,
    next_value_id: usize,
    next_block_id: usize,
}

pub struct BasicBlock<'a> {
    name: String,
    instructions: Vec<Box<Instruction<'a>>>,
}

pub struct Parameter {
    name: String,
    type_name: String,
}

pub struct Constant {
    name: String,
    value: u64,
}
