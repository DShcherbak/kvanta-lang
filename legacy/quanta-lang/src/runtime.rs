use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local};
use quanta_parser::{ast::keys::key_to_number};

use crate::{execution::{Execution, Scope}, program::Program, utils::{canvas::{Canvas, CanvasReader}, message::{CommandBlock, RuntimeError}}};

use std::{collections::HashMap, sync::{Arc, Mutex}};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Runtime {
    main_execution: Execution,
    key_execution: Option<Execution>,
    mouse_execution: Option<Execution>,
    canvas: CanvasReader,
    runtime_error: Arc<Mutex<RuntimeError>>,
}


#[wasm_bindgen]
impl Runtime {
    pub fn execute(&self) {
        if self.runtime_error.lock().unwrap().error_code != 0 {
            return;
        }
        let runtime_error = Arc::clone(&self.runtime_error);
        let new_exec = self.main_execution.clone();
        spawn_local(async move {
            match new_exec.clone().execute().await {
                Ok(_) => {},
                Err(err) => {
                    let mut inner_error = runtime_error.lock().unwrap();
                    *inner_error = RuntimeError::new(err);
                }
            }
        });
    }

    pub fn execute_key(&self, key: String) {
        if let Some(key_code) = key_to_number(key.as_str()) {
            if let Some(exec) = self.key_execution.clone() {
                    spawn_local(async move {
                    match exec.clone().execute_key(key_code).await {
                    Ok(_) => {},
                    Err(err) => {
                        panic!("Got error: {}", err);
                    }
                }
                })
            }   
        }
    }

    pub fn execute_mouse(&self, x: i32, y:i32) {
        if let Some(exec) = self.mouse_execution.clone() {
                spawn_local(async move {
                match exec.clone().execute_mouse(x, y).await {
                Ok(_) => {},
                Err(err) => {
                    panic!("Got error: {}", err);
                }
            }
            })
        }   
    }

    pub fn get_commands(&mut self) -> Vec<CommandBlock> {
        let mut result = vec![];
        let mut block = CommandBlock::new();
        if self.runtime_error.lock().unwrap().error_code != 0 {
                block.set_status(3);
                result.push(block);
                return result;
            }
        
        for command in self.canvas.get_commands() {
            if self.runtime_error.lock().unwrap().error_code != 0 {
                block.set_status(3);
                result.push(block);
                return result;
            }
            if command.starts_with("sleep") {
                let time = command.split(' ').collect::<Vec<&str>>().get(1).unwrap().parse::<i32>().unwrap(); //parse i32
                block.sleep_for = time;
                result.push(block);
                block = CommandBlock::new();
            } else if command.starts_with("frame") {
                block.set_status(0);
                result.push(block);
                block = CommandBlock::new();
            } else if command.starts_with("end") {
                block.set_status(2);
                result.push(block);
                return result;
            } else {
                block.push(command);
            }
        }
        result.push(block);
        return result;
    }

    pub fn get_runtime_error(&self) -> RuntimeError {
        self.runtime_error.lock().unwrap().clone()
    }
}

impl Runtime {
    pub async fn new(prog : Program, canv: Canvas, canvas: CanvasReader) -> Runtime {
        //let exec = Execution::from_program(prog.clone(), canv);
        let global_vars = Arc::new(Mutex::new(HashMap::new()));
        let global_var_defs = Arc::new(Mutex::new(prog.global_vars));
        let fig_col = Arc::new(Mutex::new(String::from("#ffffff")));
        let lin_col = Arc::new(Mutex::new(String::from("#000000")));
        let lin_wid = Arc::new(Mutex::new(1));

        let exec = Execution {
            lines : prog.lines.clone(),
            scope : Arc::new(Mutex::new(Scope { variables: HashMap::new(), outer_scope: None })),
            global_vars: Arc::clone(&global_vars),
            canvas: canv.clone(),
            functions: prog.functions.clone(),
            figure_color: Arc::clone(&fig_col),
            line_color: Arc::clone(&lin_col),
            line_width: Arc::clone(&lin_wid),
            random_color: Arc::new(Mutex::new(0)),
            expanded_arrays: Arc::new(Mutex::new(prog.expanded_arrays.clone()))
        };

        let keyboard_exec = if exec.functions.contains_key("keyboard") {
            let mut c = exec.clone();
            c.scope = Arc::new(Mutex::new(Scope { variables: HashMap::new(), outer_scope: None }));
            Some(c)
        } else { 
            None 
        };

        let mouse_exec = if exec.functions.contains_key("mouse") { 
            let mut c = exec.clone();
            c.scope = Arc::new(Mutex::new(Scope { variables: HashMap::new(), outer_scope: None }));
            Some(c)
        } else { 
            None 
        };

        let defs = global_var_defs.lock().unwrap();

        let mut runtime_error = RuntimeError::zero();

        for (name, (_, expr)) in defs.iter() {
            let val = exec.calculate_expression(expr.clone()).await;
            match val {
                Ok(value) => {
                    exec.global_vars.lock().unwrap().insert(name.clone(), value)
                },
                Err(e) => {runtime_error = RuntimeError::new(e); break;}
            };
        }

        Runtime { 
            main_execution: exec, 
            key_execution: keyboard_exec,
            mouse_execution: mouse_exec,
            canvas: canvas,
            runtime_error: Arc::new(Mutex::new(runtime_error)),
        }
    }
}