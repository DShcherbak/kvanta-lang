pub mod alert;
pub mod compiler;

use alert::alert;
use compiler::{CompilationResult, InternalExecutable};

use kvanta_compiler::vm::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Runtime {}

#[wasm_bindgen]
impl Runtime {
    pub fn new() -> Self {
        Runtime {}
    }

    pub fn execute(&self, compilation: CompilationResult) {
        if compilation.code != 0 {
            alert("Compilation fail");
            return;
        }
        let InternalExecutable { function, common } =
            InternalExecutable::from(compilation.get_executable());
        let mut vm = VM::new(common);
        vm.call(function, 0);
        vm.run();
    }
}
