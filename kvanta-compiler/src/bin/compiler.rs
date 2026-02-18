use std::rc::Rc;

use kvanta_compiler::chunk::*;
//use kvanta_compiler::debug::*;
use kvanta_compiler::value::*;
use kvanta_compiler::vm::*;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::Float(1.2));
    chunk.push_code(OpCode::OpConstant, 0);
    chunk.push(constant as u8, 0);
    chunk.push_code(OpCode::OpNegate, 0);
    chunk.push_code(OpCode::OpReturn, 123);
    let _ = interpret(Rc::new(chunk));
}
