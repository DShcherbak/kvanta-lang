use std::env;

use kvanta_compiler::vm::InterpretResult;
use kvanta_compiler::compiler::compile;

fn interpret(source: String) -> InterpretResult {
    compile(source);
    InterpretResult::Ok
}

fn repl() {
    let mut line : String = String::new();
    loop {
        println!("> ");
        std::io::stdin().read_line(&mut line).expect("Failed to read line");
        interpret(line);
        line = String::new();
    }
}

fn run_file(args: &[String]) -> InterpretResult{
    let filename = &args[1];
    match std::fs::read_to_string(filename) {
        Ok(contents) => interpret(contents),
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