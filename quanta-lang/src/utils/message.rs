use std::fmt;

use wasm_bindgen::prelude::*;
use quanta_parser::error::{Error, ErrorType};
use crate::runtime::Runtime;
//use crate::linear_runtime;


#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct CommandBlock{
    commands: Vec<String>,
    pub sleep_for: i32,
    pub should_draw_frame: bool
}

#[wasm_bindgen]
impl CommandBlock {
    pub fn get_commands(&self) -> Vec<String> {
        self.commands.clone()
    }
}

impl CommandBlock {
    pub fn new() -> CommandBlock {
        CommandBlock { commands: vec![], sleep_for: 0, should_draw_frame: false }
    }

    pub fn push(&mut self, command: String) -> CommandBlock {
        self.commands.push(command);
        self.clone()
    }
}


#[wasm_bindgen]
pub struct CompilationMessage {
    pub error_code: u32,
    error_message: String,
    runtime: Option<Runtime>,
    pub start_row: usize,
    pub start_column: usize,
    pub end_row: usize,
    pub end_column: usize
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

    #[wasm_bindgen]
    pub fn get_error_message(&self) -> String {
        self.error_message.clone()
    }
}

impl CompilationMessage {

    pub(crate) fn ok(runtime: Runtime) -> CompilationMessage {
        CompilationMessage {
            error_code: 0,
            error_message: "".to_string(),
            runtime: Some(runtime),
            start_row: 0,
            start_column: 0,
            end_row: 0,
            end_column: 0
        }
    }

    pub(crate) fn create_error_message(error: Error) -> CompilationMessage {
        CompilationMessage {
            error_code: { 
                match error.error_type {
                    ErrorType::ParseError => {1},
                    ErrorType::LogicError=> {2},
                    ErrorType::TypeError=> {3},
                    ErrorType::RuntimeError=> {4},
                }
            }, 
            error_message: error.message.to_string(), 
            runtime: None,
            start_row: error.start.0,
            start_column: error.start.1,
            end_row: error.finish.0,
            end_column: error.finish.1
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
        write!(f, "{}\n", self.error_message)?;
        Ok(())
    }
}