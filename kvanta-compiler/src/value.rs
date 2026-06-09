use crate::chunk::Chunk;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Boolean(bool),
    Nil,
    String(i32),
}

pub type ValueArray = Vec<Value>;

#[derive(Debug, Clone)]
pub struct Function {
    arity: usize,
    pub chunk: Chunk,
    pub name: String
}

pub fn new_function(name: String) -> Function {
    Function {
        arity: 0,
        chunk: Chunk::new(),
        name,
    }
}

