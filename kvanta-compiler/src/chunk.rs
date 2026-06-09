use num_enum::TryFromPrimitive;
use std::ops::Index;

#[derive(Debug, TryFromPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Return = 0,
    Constant = 1,
    Negate = 2,
    Add = 3,
    Subtract = 4,
    Multiply = 5,
    Divide = 6,
    True = 7,
    False = 8,
    Nil = 9,
    Not = 10,
    Equal = 11,
    Greater = 12,
    Less = 13,
    Print = 14,
    Pop = 15,
    DefineGlobal = 16,
    GetGlobal = 17,
    SetGlobal = 18,
    GetLocal = 19,
    SetLocal = 20,
    JumpIfFalse = 21,
    Jump = 22,
    Loop = 23,
}

pub fn from(value: u8) -> Option<OpCode> {
    value.try_into().ok()
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub chunk: Vec<u8>,
    pub lines: Vec<u32>,
}

impl Chunk {
    pub fn push(&mut self, byte: u8, line: u32) {
        self.chunk.push(byte);
        self.lines.push(line);
    }

    pub fn push_code(&mut self, code: OpCode, line: u32) {
        self.push(code as u8, line);
    }

    pub fn get(&self, offset: usize) -> Option<&u8> {
        self.chunk.get(offset)
    }

    pub fn new() -> Chunk {
        Chunk {
            chunk: vec![],
            lines: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunk.is_empty()
    }

    pub fn read_short(&self, offset: &mut usize) -> Option<usize> {
        if *offset + 1 >= self.chunk.len() {
            return None;
        }
        let high = self.chunk[*offset] as usize;
        let low = self.chunk[*offset + 1] as usize;
        *offset += 2;
        Some((high << 8) | low)
    }
}

impl Index<usize> for Chunk {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunk[index]
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
