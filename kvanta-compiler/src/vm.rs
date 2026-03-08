use std::rc::Rc;
use crate::value::Value;

use crate::chunk::*;

pub struct VM {
    chunk: Rc<Chunk>,
    ip: usize,
    stack: Vec<Value>,
    heap: Vec<String>,
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

macro_rules! binary_op_bin {
    ($s:expr, $op:tt) => {
        match ($s.peek(0), $s.peek(1)) {
            (Value::Float(b), Value::Float(a)) => {
                $s.pop();
                $s.pop();
                $s.push(Value::Boolean(a $op b));
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
                    OpCode::Return => {
                        let const_value = self.pop();
                        if let Value::String(id) = const_value {
                            if let Some(string) = self.heap.get(id as usize) {
                                println!("\"{}\"", string);
                                return InterpretResult::Ok;
                            } else {
                                println!("ERR: INVALID STRING ID");
                                return InterpretResult::RuntimeError;
                            }
                        }
                        println!("{:?}", const_value);
                        return InterpretResult::Ok;
                    },
                    OpCode::Constant => {
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
                    OpCode::Negate => {
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
                    OpCode::Add => {
                        match (self.peek(0), self.peek(1)) {
                            (Value::Float(b), Value::Float(a)) => {
                                self.pop();
                                self.pop();
                                self.push(Value::Float(a + b));
                            },
                            (Value::String(b), Value::String(a)) => {
                                self.pop();
                                self.pop();
                                let a_str = self.heap.get(a as usize);
                                let b_str = self.heap.get(b as usize);
                                if let (Some(a_str), Some(b_str)) = (a_str, b_str) {
                                    let result = a_str.to_string() + b_str;
                                    let result_id = self.take_string(result);
                                    self.push(Value::String(result_id));
                                } else {
                                    println!("ERR: INVALID STRINGS");
                                }
                            },
                            _ => println!("ERR: OPERANDS MUST BE NUMBERS")
                        }
                    },
                    OpCode::Subtract => binary_op!(self, -),
                    OpCode::Multiply => binary_op!(self, *),
                    OpCode::Divide => binary_op!(self, /),
                    OpCode::True => self.push(Value::Boolean(true)),
                    OpCode::False => self.push(Value::Boolean(false)),
                    OpCode::Nil => self.push(Value::Nil),
                    OpCode::Not => {
                        match self.peek(0) {
                            Value::Boolean(x) => {
                                self.pop();
                                self.push(Value::Boolean(!x))
                            },
                            _ => {
                                self.runtime_error("Operand must be a boolean.");
                                return InterpretResult::RuntimeError;
                            }
                        }
                    },
                    OpCode::Equal => {
                        let a = self.peek(0);
                        let b = self.peek(1);
                        self.pop();
                        self.pop();
                        self.push(Value::Boolean(self.compare(a,b)));
                    },
                    OpCode::Greater => binary_op_bin!(self, >),
                    OpCode::Less => binary_op_bin!(self, <),
                }
            } else {
                println!("ERR: END OF EXECUTION");
                return InterpretResult::RuntimeError;
            }
            self.ip+=1;
        }
    }

    fn compare(&self, a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Nil, Value::Nil) => true,
            (Value::String(x), Value::String(y)) => {
                if x == y {
                    return true;
                }
                let x_str = self.heap.get(x as usize);
                let y_str = self.heap.get(y as usize);
                if let (Some(x_str), Some(y_str)) = (x_str, y_str) {
                    x_str == y_str
                } else {                    
                    false
                }
            },
            _ => false
        }
    }

    pub fn take_string(&mut self, s: String) -> i32 {
        self.heap.push(s);
        (self.heap.len() - 1) as i32
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

    pub fn new(chunk: Rc<Chunk>, heap: Vec<String>) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: vec![],
            heap,
        }
    }

    fn runtime_error(&self, message: &str) {
        let instruction = self.chunk.get(self.ip).unwrap_or(&0);
        println!("Runtime error: {}\n[line {}] in script", message, self.chunk.lines[*instruction as usize]);
    }
}

pub fn interpret(chunk: Rc<Chunk>, heap: Vec<String>) -> InterpretResult {
    let mut vm = VM::new(chunk, heap);
    vm.run()
}