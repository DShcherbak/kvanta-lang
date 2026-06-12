use kvanta_compiler::compiler::compile;
use kvanta_compiler::{value::Function, vm::CommonMemory};
use wasm_bindgen::prelude::*;

use crate::alert::alert;

#[wasm_bindgen]
pub struct CompilationResult {
    pub code: usize,
    executable: Option<Executable>,
}

#[wasm_bindgen]
impl CompilationResult {
    pub fn get_executable(&self) -> Executable {
        match &self.executable {
            Some(exec) => exec.clone(),
            None => panic!("No executable!"),
        }
    }
}

#[wasm_bindgen]
pub struct Compiler {}

#[wasm_bindgen]
impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    pub fn compile(&self, source: String) -> CompilationResult {
        match compile(source) {
            Ok((function, common)) => CompilationResult {
                code: 0,
                executable: Some(Executable { function, common }),
            },
            Err(mes) => {
                alert(&mes);
                CompilationResult {
                    code: 1,
                    executable: None,
                }
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Executable {
    function: Function,
    common: CommonMemory,
}

pub struct InternalExecutable {
    pub function: Function,
    pub common: CommonMemory,
}

impl From<Executable> for InternalExecutable {
    fn from(exec: Executable) -> Self {
        InternalExecutable {
            function: exec.function,
            common: exec.common,
        }
    }
}
