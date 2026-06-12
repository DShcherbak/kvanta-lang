use std::collections::HashMap;
use crate::value::{Function, Value};

use crate::chunk::*;

#[derive(Clone)]
pub struct CommonMemory {
    pub heap: Vec<String>,
    pub functions: Vec<Function>,
    pub constants: Vec<Value>
}

impl CommonMemory {
    pub fn new() -> Self {
        CommonMemory { heap: vec![] , functions: vec![], constants: vec![] }
    }
}

pub struct CallFrame {
    function: Function,
    ip: usize,
    slot_start: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    pub common: CommonMemory,
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
                $s.push(Value::Bool(a $op b));
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
    fn frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let ip = self.frame().ip;
        let byte = self.frame_mut().function.chunk.get(ip).cloned();
        if byte.is_some() {
            self.frame_mut().ip += 1;
        }
        byte
    }

    fn read_short(&mut self) -> Option<usize> {
        let ip = self.frame().ip;
        let chunk = &self.frame_mut().function.chunk;
        let high = chunk.get(ip).cloned();
        let low = chunk.get(ip + 1).cloned();
        if let (Some(high), Some(low)) = (high, low) {
            self.frame_mut().ip += 2;
            Some(((high as usize) << 8) | (low as usize))
        } else {
            None
        }
    }

    pub fn run(&mut self) -> InterpretResult {
        println!("====== MAIN EXECUTION ======");
        loop {
            if let Some(code) = self.read_byte().and_then(from) {
                //println!("Executing: {:?}", code);
                match code {
                    OpCode::Return => {
                        let return_value = self.pop();
                        self.frames.pop();
                        if self.frames.is_empty() {
                            return InterpretResult::Ok;
                        }
                        self.push(return_value);
                    },
                    OpCode::Constant => {
                        if let Some(id) = self.read_byte() {
                            let temp_value = self.common.constants.get(id as usize);
                            if let Some(const_value) = temp_value.cloned() {
                                self.push(const_value.clone());
                                //println!("Constant ID: {}", id);
                                //println!("Constant Value: {:?}", const_value);
                            } else {
                                println!("ERR: INVALID CONSTANT ID");
                            }
                        } 
                        else {
                            println!("ERR: NO CONSTANTS 1");
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
                                let a_str = self.common.heap.get(a as usize);
                                let b_str = self.common.heap.get(b as usize);
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
                    OpCode::True => self.push(Value::Bool(true)),
                    OpCode::False => self.push(Value::Bool(false)),
                    OpCode::Nil => self.push(Value::Nil),
                    OpCode::Not => {
                        match self.peek(0) {
                            Value::Bool(x) => {
                                self.pop();
                                self.push(Value::Bool(!x))
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
                        self.push(Value::Bool(self.compare(a,b)));
                    },
                    OpCode::Greater => binary_op_bin!(self, >),
                    OpCode::Less => binary_op_bin!(self, <),
                    OpCode::Print => {
                        let value = self.pop();
                        println!("Print: {:?}", value);
                    },
                    OpCode::Pop => { 
                        let _ = self.pop();
                        // let x = self.pop();
                        // println!("Pop: {:?}", x);
                    },
                    OpCode::DefineGlobal => {
                        if let Some(id) = self.read_byte() 
                            && let Some(const_value) = self.common.constants.get(id as usize)
                        {
                            if let Value::String(const_id) = const_value 
                            && let Some(const_str) = self.common.heap.get(*const_id as usize)
                            {
                                //println!("DefineGlobal Name: {}", const_str);
                                self.variables.insert(const_str.to_string(), self.peek(0));
                                self.pop();
                                //println!("All variables: {:?}", self.variables);
                            }
                            else {
                                println!("ERR: INVALID VARIABLE NAME");
                            }
                        } 
                        else {
                            println!("ERR: NO CONSTANTS 2");
                        }
                    },
                    OpCode::GetGlobal => {
                        if let Some(id) = self.read_byte() 
                            && let Some(const_value) = self.common.constants.get(id as usize)
                        {
                           // println!("GetGlobal ID: {}", id);
                            if let Value::String(const_id) = const_value 
                            && let Some(const_str) = self.common.heap.get(*const_id as usize)
                            {
                               // println!("GetGlobal Name: {}", const_str);
                             //   println!("All variables: {:?}", self.variables);
                                if let Some(value) = self.variables.get(const_str) {
                                    self.push(value.clone());
                                } else {
                                    self.runtime_error(&format!("Undefined variable '{}'.", const_str));
                                    return InterpretResult::RuntimeError;
                                }
                            }
                            else {
                                println!("ERR: INVALID VARIABLE NAME");
                            }
                        } 
                        else {
                            println!("ERR: NO CONSTANTS 3");
                        }
                    },
                    OpCode::SetGlobal => {
                        if let Some(id) = self.read_byte() 
                            && let Some(const_value) = self.common.constants.get(id as usize)
                        {
                            if let Value::String(const_id) = const_value 
                            && let Some(const_str) = self.common.heap.get(*const_id as usize)
                            {
                                if self.variables.contains_key(const_str) {
                                    let value = self.peek(0);
                                    self.variables.insert(const_str.to_string(), value);
                                } else {
                                    self.runtime_error(&format!("Undefined variable '{}'.", const_str));
                                    return InterpretResult::RuntimeError;
                                }
                            }
                            else {
                                println!("ERR: INVALID VARIABLE NAME");
                            }
                        } 
                        else {
                            println!("ERR: NO CONSTANTS 4");
                        }
                    }
                    OpCode::GetLocal => {
                        if let Some(id) = self.read_byte() {
                            let value = self.stack.get(self.frame().slot_start + id as usize).cloned();
                            if let Some(value) = value {
                                self.push(value);
                            } else {
                                println!("ERR: INVALID LOCAL VARIABLE ID");
                            }
                        } else {
                            println!("ERR: NO LOCAL VARIABLES");
                        }
                    },
                    OpCode::SetLocal => {
                        if let Some(id) = self.read_byte() {
                            let slot = self.frame().slot_start + id as usize;
                            self.stack[slot] = self.peek(0);
                        } else {
                            println!("ERR: NO LOCAL VARIABLES");
                        }
                    },
                    OpCode::JumpIfFalse => {
                        if let Some(offset) = self.read_short() {
                            if let Value::Bool(condition) = self.peek(0) {
                                if !condition {
                                    self.frame_mut().ip += offset;
                                }
                            } else {
                                println!("ERR: CONDITION MUST BE A BOOLEAN");
                                return InterpretResult::RuntimeError;
                            }
                        } else {
                            println!("ERR: NO JUMP OFFSET");
                            return InterpretResult::RuntimeError;
                        }
                    },
                    OpCode::Jump => {
                        if let Some(offset) = self.read_short() {
                            self.frame_mut().ip += offset;
                        } else {
                            println!("ERR: NO JUMP OFFSET");
                            return InterpretResult::RuntimeError;
                        }
                    },
                    OpCode::Loop => {
                        if let Some(offset) = self.read_short() {
                            self.frame_mut().ip -= offset;
                        } else {
                            println!("ERR: NO LOOP OFFSET");
                            return InterpretResult::RuntimeError;
                        }
                    },
                    OpCode::Call => {
                        if let Some(arg_count) = self.read_byte() {
                            let callee = self.peek(arg_count as usize);
                            if let Value::Function(fun_id) = callee {
                                if let Some(function) = self.common.functions.get(fun_id as usize).cloned() {
                                    self.call(function, arg_count);
                                } else {
                                    println!("ERR: INVALID FUNCTION ID");
                                    return InterpretResult::RuntimeError;
                                }
                            } else {
                                println!("ERR: CAN ONLY CALL FUNCTIONS");
                                return InterpretResult::RuntimeError;
                            }
                        } else {
                            println!("ERR: NO ARG COUNT");
                            return InterpretResult::RuntimeError;
                        }
                    },
                }
            } else {
                println!("ERR: END OF EXECUTION");
                return InterpretResult::RuntimeError;
            }
        }
    }

    pub fn call(&mut self, function: Function, arg_count: u8) {
        if arg_count as usize != function.arity {
            self.runtime_error(&format!("Expected {} arguments but got {}", function.arity, arg_count));
            return;
        }
        self.frames.push(CallFrame {
            function,
            ip: 0,
            slot_start: self.stack.len() - arg_count as usize,
        });
        if self.frames.len() > 64 {
            self.runtime_error("Stack overflow.");
        }
    }

    fn compare(&self, a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Nil, Value::Nil) => true,
            (Value::String(x), Value::String(y)) => {
                if x == y {
                    return true;
                }
                let x_str = self.common.heap.get(x as usize);
                let y_str = self.common.heap.get(y as usize);
                if let (Some(x_str), Some(y_str)) = (x_str, y_str) {
                    x_str == y_str
                } else {                    
                    false
                }
            },
            _ => false
        }
    }

    fn take_string(&mut self, s: String) -> i32 {
        self.common.heap.push(s);
        (self.common.heap.len() - 1) as i32
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Float(0.0))
    }

    fn peek(&self, distance: usize) -> Value {
        self.stack.get(self.stack.len() - 1 - distance).unwrap_or(&Value::Float(0.0)).clone()
    }

    pub fn new(common: CommonMemory) -> Self {
        Self {
            frames: vec![],
            stack: vec![],
            variables: HashMap::new(),
            common
        }
    }

    fn runtime_error(&self, message: &str) {
        println!("Runtime error: {}", message);

        for frame in self.frames.iter().rev() {
            println!("[line {}] in {}", frame.function.chunk.lines[frame.ip], frame.function.name);
        }
    }
}