use crate::chunk::*;
use crate::vm::VM;

fn simple_instruction(code: OpCode, offset: &mut usize) {
    println!("{:?}", code);
    *offset += 1;
}

fn one_arg_instruction(code: OpCode, chunk: &Chunk, offset: &mut usize, vm: &VM) {
    *offset += 1;
    if let Some(const_id) = chunk.get(*offset)
        && let Some(const_value) = vm.common.constants.get(*const_id as usize)
    {
        println!("{:?} {:?}", code, const_value);
    } else {
        println!("{:?} NO_ARG", code);
    }
    *offset += 1;
}

fn byte_instruction(code: OpCode, chunk: &Chunk, offset: &mut usize) {
    *offset += 1;
    if let Some(arg) = chunk.get(*offset) {
        println!("{:?} {}", code, arg);
    } else {
        println!("{:?} NO_ARG", code);
    }
    *offset += 1;
}

fn jump_instruction(code: OpCode, chunk: &Chunk, offset: &mut usize, sign: bool) {
    *offset += 1;
    if let Some(jump) = chunk.read_short(offset) {
        let jump_offset = if sign { *offset + jump } else { *offset - jump };
        println!("{:?} {}", code, jump_offset);
    } else {
        println!("{:?} NO_ARG", code);
    }
}

pub fn print_instruction(chunk: &Chunk, offset: &mut usize, vm: &VM) {
    let offset_value = *offset;
    print!("{} ", offset_value);
    if offset_value > 0 && chunk.lines[offset_value] == chunk.lines[offset_value - 1] {
        print!("| ");
    } else {
        print!("{} ", chunk.lines[offset_value]);
    }
    let instruction = from(chunk[offset_value]);
    match instruction {
        None => println!("DISSASEMBLE_ERROR"),
        Some(code) => match code {
            OpCode::Constant | OpCode::GetGlobal | OpCode::SetGlobal | OpCode::DefineGlobal => one_arg_instruction(code, chunk, offset, vm),
            OpCode::GetLocal | OpCode::SetLocal | OpCode::Call => byte_instruction(code, chunk, offset),
            OpCode::Jump | OpCode::JumpIfFalse | OpCode::Loop => jump_instruction(code, chunk, offset, code != OpCode::Loop),
            _ => simple_instruction(code, offset),
        },
    }
}

pub fn print(chunk: &Chunk, name: &str, vm: &VM) {
    println!("=== {} ===", name);
    let mut offset: usize = 0;
    let size = chunk.len();
    while offset < size {
        print_instruction(chunk, &mut offset, vm);
    }
}
