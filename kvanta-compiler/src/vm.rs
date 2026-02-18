use std::rc::Rc;
use crate::value::Value;

use crate::chunk::*;

struct VM {
    chunk: Rc<Chunk>,
    ip: usize,
    stack: Vec<Value>
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl VM {
    fn run(&mut self) -> InterpretResult {
        loop {
            if let Some(code) = self.chunk.get(self.ip).and_then(|x| from(*x)) {
                match code {
                    OpCode::OpReturn => {
                        let const_value = self.pop();
                        println!("{:?}", const_value);
                        return InterpretResult::Ok;
                    },
                    OpCode::OpConstant => {
                        self.ip += 1;
                        if let Some(id) = self.chunk.get(self.ip) 
                            && let Some(const_value) = self.chunk.get_constant(*id as usize) 
                        {
                            self.push((*const_value).clone());
                        } 
                        else {
                            println!("ERR: NO CONSTANTS");
                        }
                    },
                    OpCode::OpNegate => {
                        match self.pop() {
                            Value::Float(x) => self.push(Value::Float(-x))
                        }
                    }
                }
            } else {
                println!("ERR: END OF EXECUTION");
                return InterpretResult::RuntimeError;
            }
            self.ip+=1;
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Float(0.0))
    }
}

pub fn interpret(chunk: Rc<Chunk>) -> InterpretResult {
    let mut vm = VM {chunk, ip: 0, stack: vec![]};
    vm.run()
}