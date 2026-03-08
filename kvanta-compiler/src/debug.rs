use crate::chunk::*;

fn simple_instruction(code: OpCode, offset: &mut usize) {
    println!("{:?}", code);
    *offset += 1;
}

fn one_arg_instruction(code: OpCode, chunk: &Chunk, offset: &mut usize) {
    *offset += 1;
    if let Some(const_id) = chunk.get(*offset)
        && let Some(const_value) = chunk.get_constant(*const_id as usize)
    {
        println!("{:?} {:?}", code, const_value);
    } else {
        println!("{:?} NO_ARG", code);
    }
    *offset += 1;
}

pub fn print_instruction(chunk: &Chunk, offset: &mut usize) {
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
            OpCode::Constant => one_arg_instruction(code, chunk, offset),
            _ => simple_instruction(code, offset),
        },
    }
}

pub fn print(chunk: &Chunk, name: &str) {
    println!("=== {} ===", name);
    let mut offset: usize = 0;
    let size = chunk.len();
    while offset < size {
        print_instruction(chunk, &mut offset);
    }
}
