#[derive(Debug, Clone)]
pub enum Value {
    Float(f32),
}

pub type ValueArray = Vec<Value>;
