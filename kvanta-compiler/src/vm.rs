use std::rc::Rc;
use crate::value::Value;

use crate::chunk::*;

pub struct VM {
    chunk: Rc<Chunk>,
    ip: usize,
    stack: Vec<Value>
}

// Accepts an operator, pops two values from the stack, applies the operator, and pushes the result back on the stack.
macro_rules! binary_op {
    ($s:expr, $op:tt) => {
        match ($s.peek(0), $s.peek(1)) {
            (Value::Float(b), Value::Float(a)) => {
                $s.pop();
                $s.pop();
                $s.push(Value::Float(a $op b));
            },
            _ => println!("ERR: OPERANDS MUST BE NUMBERS")
        }
    };
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError
}

impl VM {
    pub fn run(&mut self) -> InterpretResult {
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
                        match self.peek(0) {
                            Value::Float(x) => {
                                self.pop();
                                self.push(Value::Float(-x))
                            },
                            _ => {
                                self.runtime_error("Operand must be a number.");
                                return InterpretResult::RuntimeError;
                            }
                        }
                    },
                    OpCode::OpAdd => binary_op!(self, +),
                    OpCode::OpSubtract => binary_op!(self, -),
                    OpCode::OpMultiply => binary_op!(self, *),
                    OpCode::OpDivide => binary_op!(self, /),
                    OpCode::OpTrue => self.push(Value::Boolean(true)),
                    OpCode::OpFalse => self.push(Value::Boolean(false)),
                    OpCode::OpNil => self.push(Value::Nil),

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

    pub fn peek(&self, distance: usize) -> Value {
        self.stack.get(self.stack.len() - 1 - distance).unwrap_or(&Value::Float(0.0)).clone()
    }

    pub fn new(chunk: Rc<Chunk>) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: vec![]
        }
    }

    fn runtime_error(&self, message: &str) {
        println!("Runtime error: {}", message);
    }
}

pub fn interpret(chunk: Rc<Chunk>) -> InterpretResult {
    let mut vm = VM::new(chunk);
    vm.run()
}