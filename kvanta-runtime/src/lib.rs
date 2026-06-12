use kvanta_compiler::value::Function;
use wasm_bindgen::prelude::*;
use kvanta_compiler::vm::*;
use kvanta_compiler::compiler::compile;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct CompilationResult {
    pub code : usize,
    executable: Option<Executable>
}

#[wasm_bindgen]
impl CompilationResult {
    pub fn get_executable(&self) -> Executable {
        match &self.executable {
            Some(exec) => exec.clone(),
            None => panic!("No executable!")
        }
    }
}

#[wasm_bindgen]
pub struct Compiler {

}

#[wasm_bindgen]
impl Compiler {
    pub fn new() -> Self {
        Compiler{}
    }

    pub fn compile(&self, source: String) -> CompilationResult {
        match compile(source) {
            Ok((function, common)) => CompilationResult { 
                code: 0, 
                executable: Some(Executable { function, common }) 
            },
            Err(mes) => {
                alert(&mes);
                CompilationResult { code: 1, executable: None }
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Executable {
    function: Function,
    common: CommonMemory
}

#[wasm_bindgen]
pub struct Runtime {
}

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
        let Executable {function, common} = compilation.get_executable();
        let mut vm = VM::new(common);
        vm.call(function, 0);
        vm.run();
    }
}




