mod utils;
mod compiler;
mod program;
mod execution;
//mod tests;
mod runtime;
//mod linear_execution;
//mod linear_runtime;

use wasm_bindgen::prelude::*;

use crate::utils::message::{CompilationMessage};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, quanta-lang!");
}



#[wasm_bindgen]
pub struct Compiler {


}

#[wasm_bindgen]
impl Compiler {
    pub fn new() -> Compiler {
        Compiler{}
    }

    pub async fn compile_code(&mut self, source : &str) -> CompilationMessage {
        self.compile(source).await
    }
}

#[test]
    fn test_file() {
        // let file_path = "../grammar/test.txt";

        // let contents = std::fs::read_to_string(file_path)
        //     .expect("Should have been able to read the file");
        //assert!(contents.len() > 0);
        //let result = Compiler::new().compile(&contents);
       // println!("{}\n=====================================", result.await);
    }
