#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f32),
    Boolean(bool),
    Nil
}

pub type ValueArray = Vec<Value>;
