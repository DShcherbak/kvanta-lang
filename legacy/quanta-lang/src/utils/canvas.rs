use wasm_bindgen::prelude::*;
use crossbeam_channel::{Receiver, Sender};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Canvas {
    commands: Sender<String>,
}

#[derive(Clone)]
pub struct CanvasReader {
    commands: Receiver<String>
}

pub fn construct_canvas() -> (Canvas, CanvasReader) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (Canvas { commands: tx }, CanvasReader { commands: rx })
}

impl Canvas {
    pub fn add_command(&mut self, c : String) {
        self.commands.send(c).expect("Compiler crashed, please try again!");
    }
}

impl CanvasReader {
    pub fn get_commands(&mut self) -> Vec<String> {
        self.commands.try_iter().collect()
    }
}