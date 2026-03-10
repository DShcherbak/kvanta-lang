use std::env;

use kvanta_compiler::vm::InterpretResult;
use kvanta_compiler::compiler::compile;
use std::rc::Rc;
use kvanta_compiler::vm::VM;
use kvanta_compiler::debug::print;

fn interpret(vm: &mut VM, source: String) -> InterpretResult {
    match compile(source, vm.common.clone()) {
        Err(error) => {
            println!("Compile error: {}", error);
            InterpretResult::CompileError
        }
        Ok(chunk) => {
            // DEBUG
            print(&chunk, "code");
            vm.update_chunk(Rc::new(chunk));
            vm.run()
        },
    }
}

fn repl() {
    let mut line : String = String::new();
    let mut vm = VM::new();
    loop {
        println!("> ");
        std::io::stdin().read_line(&mut line).expect("Failed to read line");
        interpret(&mut vm, line);
        line = String::new();
    }
}

fn run_file(args: &[String]) -> InterpretResult{
    let filename = &args[1];
    let mut vm = VM::new();
    match std::fs::read_to_string(filename) {
        Ok(contents) => interpret(&mut vm, contents),
        Err(error) => {
            println!("Could not read file {}: {}", filename, error);
            InterpretResult::CompileError
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args);
    } else {
        println!("Usage: kvanta [script]");
    }
}