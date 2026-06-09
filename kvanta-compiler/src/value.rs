use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String
}

#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Boolean(bool),
    Nil,
    String(i32),
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

