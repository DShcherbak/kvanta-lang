use quanta_parser::error::Error;

use crate::{linear_execution::{Execution, Scope}, program::Program, utils::{canvas::{Canvas, CanvasReader}, message::CommandBlock}};

use std::{collections::HashMap, fmt::Debug, sync::{Arc, Mutex}};


#[derive(Clone)]
pub struct Runtime {
    main_execution: Execution,
    //key_execution: Option<Execution>,
    //mouse_execution: Option<Execution>,
    canvas: CanvasReader,
    global_error: Option<Error>,
}

impl Debug for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
         .field("execution", &self.main_execution)

         .finish()
    }
}


impl Runtime {
    pub fn execute(&self) -> Result<(), Error> {
        if self.global_error.is_some() {
            panic!("Got error: {}", self.global_error.clone().unwrap());
        }
        let new_exec = self.main_execution.clone();
        return  new_exec.clone().execute();
    }

    pub fn get_commands(&mut self) -> Vec<CommandBlock> {
        let mut result = vec![];
        let mut block = CommandBlock::new();
        
        for command in self.canvas.get_commands() {
            if command.starts_with("sleep") {
                let time = command.split(' ').collect::<Vec<&str>>().get(1).unwrap().parse::<i32>().unwrap(); //parse i32
                block.sleep_for = time;
                result.push(block);
                block = CommandBlock::new();
            } else if command.starts_with("frame") {
                block.should_draw_frame = true;
                result.push(block);
                block = CommandBlock::new();
            } else if command.starts_with("end") {
                block.sleep_for = -1;
                result.push(block);
                return result;
            } else {
                block.push(command);
            }
        }
        result.push(block);
        return result;
    }
}

impl Runtime {
    pub fn new(prog : Program, canv: Canvas, canvas: CanvasReader) -> Runtime {
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
        };

        // let keyboard_exec = if exec.functions.contains_key("keyboard") {
        //      Some(exec.clone())
        // } else { 
        //     None 
        // };

        // let mouse_exec = if exec.functions.contains_key("mouse") { 
        //     Some(exec.clone())
        // } else { 
        //     None 
        // };

        let defs = global_var_defs.lock().unwrap();

        let mut global_err = None;

        for (name, (_, expr)) in defs.iter() {
            let mut new_exec = exec.clone().create_subscope();
            let val = new_exec.calculate_expression(expr.clone());
            match val {
                Ok(value) => {
                    exec.global_vars.lock().unwrap().insert(name.clone(), value)
                },
                Err(e) => {global_err = Some(e); break;}
            };
        }

        Runtime { 
            main_execution: exec, 
            //key_execution: keyboard_exec,
            //mouse_execution: mouse_exec,
            canvas: canvas,
            global_error: global_err
        }
    }
}