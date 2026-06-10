use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f32),
    Bool(bool),
    Nil,
    String(String),
    Variable(String),
    Function(i32),
}

pub type ValueArray = Vec<Value>;

pub fn new_function(name: String) -> Function {
    Function {
        arity: 0,
        chunk: Chunk::new(),
        name,
    }
}

