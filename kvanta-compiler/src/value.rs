#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
    Boolean(bool),
    Nil,
    String(i32),
}

pub type ValueArray = Vec<Value>;
