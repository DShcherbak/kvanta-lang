pub mod alert;
pub mod compiler;

use alert::alert;
use compiler::{CompilationResult, InternalExecutable};

use kvanta_compiler::{canvas::CanvasCommand, vm::*};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Runtime {
    vm: VM
}

#[wasm_bindgen]
impl Runtime {
    pub fn new() -> Self {
        Runtime {vm: VM::new(CommonMemory::new())}
    }

    pub fn execute(&mut self, compilation: CompilationResult) {
        if compilation.code != 0 {
            alert("Compilation fail");
            return;
        }
        let InternalExecutable { function, common } =
            InternalExecutable::from(compilation.get_executable());
        self.vm = VM::new(common);
        self.vm.call(function, 0);
        self.vm.run();
    }

    pub fn get_command(&self) -> String {
        if let Some(command) = self.vm.get_command() {
            match command {
                CanvasCommand::Exit => "EXIT".to_string(),
                CanvasCommand::Circle(x,y,r) => format!("circle {} {} {}", x, y, r)
            }
        } else {
            "".to_string()
        }
    }
}
