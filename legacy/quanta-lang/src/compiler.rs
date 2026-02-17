
use quanta_parser::{parse_ast};
//use crate::linear_runtime;
use crate::program::create_program;
use crate::utils::canvas::construct_canvas;
use crate::utils::message::{CompilationMessage};
//use crate::utils::message::{LinearCompilationMessage};
use crate::{Compiler, runtime::Runtime};

impl Compiler {
    pub async fn compile(&mut self, source : &str) -> CompilationMessage {
        match parse_ast(source) {
            Ok(ast) => {
                let mut program = create_program(ast);
                match program.type_check() {
                    Err(error) =>  {
                        CompilationMessage::create_error_message(error)
                    },
                    Ok(_) => {
                        let (c, r) = construct_canvas();
                        CompilationMessage::ok(Runtime::new(program, c, r).await)
                    }
                }
            },
            Err(err) => {
                CompilationMessage::create_error_message(err)
            }
        }
    }

    // pub fn linear_compile(&mut self, source : &str) -> LinearCompilationMessage {
    //     match parse_ast(source) {
    //         Ok(ast) => {
    //             let mut program = create_program(ast);
    //             match program.type_check() {
    //                 Err(error) =>  {
    //                     LinearCompilationMessage::create_error_message(error)
    //                 },
    //                 Ok(_) => {
    //                     let (c, r) = construct_canvas();
    //                     LinearCompilationMessage::ok(linear_runtime::Runtime::new(program, c, r))
    //                 }
    //             }
    //         },
    //         Err(err) => {
    //             LinearCompilationMessage::create_error_message(err)
    //         }
    //     }
    // }

    
}



