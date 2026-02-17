use std::fmt;

use wasm_bindgen::prelude::*;
use quanta_parser::error::{Error, ErrorType};
use crate::runtime::Runtime;
//use crate::linear_runtime;

#[derive(Clone, Debug, Copy)]
enum CommandStatus {
    OkDraw = 0,
    OkNoDraw = 1,
    End = 2,
    Error = 3,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct CommandBlock{
    status: CommandStatus,
    commands: Vec<String>,
    pub sleep_for: i32
}

#[wasm_bindgen]
impl CommandBlock {
    pub fn get_commands(&self) -> Vec<String> {
        self.commands.clone()
    }

    pub fn get_status(&self) -> u8 {
        self.status as u8
    }

    pub fn set_status(&mut self, status: u8) {
        self.status = match status {
            0 => CommandStatus::OkDraw,
            1 => CommandStatus::OkNoDraw,
            2 => CommandStatus::End,
            3 => CommandStatus::Error,
            _ => CommandStatus::OkNoDraw
        }
    }
}

impl CommandBlock {
    pub fn new() -> CommandBlock {
        CommandBlock { commands: vec![], sleep_for: 0, status: CommandStatus::OkNoDraw }
    }

    pub fn push(&mut self, command: String) -> CommandBlock {
        self.commands.push(command);
        self.clone()
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct RuntimeError {
    pub error_code: u32,
    error_message: String,
    pub start_row: usize,
    pub start_column: usize,
    pub end_row: usize,
    pub end_column: usize
}

#[wasm_bindgen]
impl RuntimeError {
    pub fn get_error_message(&self) -> String {
        self.error_message.clone()
    }
}

impl RuntimeError {
    pub fn zero() -> RuntimeError {
        RuntimeError { error_code: 0, error_message: "".to_string(), start_row: 0, start_column: 0, end_row: 0, end_column: 0 }
    }

    pub fn new(error: Error) -> RuntimeError {
        RuntimeError { 
            error_code: match error.error_type {
                ErrorType::ParseError => {1},
                ErrorType::LogicError=> {2},
                ErrorType::TypeError=> {3},
                ErrorType::RuntimeError=> {4},
            }, 
            error_message: error.message.to_string(), 
            start_row: error.start.0,
            start_column: error.start.1,
            end_row: error.finish.0,
            end_column: error.finish.1
        }
    }
}


#[wasm_bindgen]
pub struct CompilationMessage {
    pub error_code: u32,
    error: Option<RuntimeError>,
    runtime: Option<Runtime>,
    
}


// pub struct LinearCompilationMessage {
//     pub error_code: u32,
//     error_message: String,
//     runtime: Option<linear_runtime::Runtime>
// }

#[wasm_bindgen]
impl CompilationMessage {
    #[wasm_bindgen]
    pub fn get_runtime(&self) -> Runtime {
        self.runtime.clone().unwrap()
    }

    pub fn get_error(&self) -> RuntimeError {
        self.error.clone().unwrap()
    }

    #[wasm_bindgen]
    pub fn get_error_message(&self) -> String {
        self.error.as_ref().unwrap().error_message.clone()
    }
}

impl CompilationMessage {

    pub(crate) fn ok(runtime: Runtime) -> CompilationMessage {
        CompilationMessage {
            error_code: 0,
            error: None,
            runtime: Some(runtime),
        }
    }

    pub(crate) fn create_error_message(error: Error) -> CompilationMessage {
        let er_code = match error.error_type {
            ErrorType::ParseError => {1},
            ErrorType::LogicError=> {2},
            ErrorType::TypeError=> {3},
            ErrorType::RuntimeError=> {4},
        };
        CompilationMessage {
            error_code: er_code,
            runtime: None,
            error: Some(RuntimeError {
                error_code: er_code, 
                error_message: error.message.to_string(), 
                start_row: error.start.0,
                start_column: error.start.1,
                end_row: error.finish.0,
                end_column: error.finish.1
            }),
        }
    }
}

// impl LinearCompilationMessage {


//     pub fn get_runtime(&self) -> linear_runtime::Runtime {
//         self.runtime.clone().unwrap()
//     }

//     pub fn get_error_message(&self) -> String {
//         self.error_message.clone()
//     }


//     pub(crate) fn ok(runtime: linear_runtime::Runtime) -> LinearCompilationMessage {
//         LinearCompilationMessage {
//             error_code: 0,
//             error_message: "".to_string(),
//             runtime: Some(runtime)
//         }
//     }

//     pub(crate) fn create_error_message(error: Error) -> LinearCompilationMessage {
//         match error.error_type {
//             ErrorType::ParseError => {
//                 LinearCompilationMessage { error_code:1, error_message: error.message.to_string(), runtime: None }
//             }
//             ErrorType::LogicError=> {
//                 LinearCompilationMessage { error_code:2, error_message: error.message.to_string(), runtime: None }
//             }
//             ErrorType::TypeError=> {
//                 LinearCompilationMessage { error_code:3, error_message: error.message.to_string(), runtime: None }
//             }
//             ErrorType::RuntimeError=> {
//                 LinearCompilationMessage { error_code:4, error_message: error.message.to_string(), runtime: None }
//             }
//         }
//     }
// }



impl fmt::Display for CompilationMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n", self.error_code as usize)?;
        write!(f, "{}\n", self.get_error_message())?;
        Ok(())
    }
}