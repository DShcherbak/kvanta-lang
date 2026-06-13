use crossbeam_channel::{Receiver, Sender};

pub enum CanvasCommand {
    Exit,
    Circle(i32, i32, i32)
}

#[derive(Debug, Clone)]
pub struct Canvas {
    commands: Sender<CanvasCommand>,
}

#[derive(Clone)]
pub struct CanvasReader {
    commands: Receiver<CanvasCommand>
}

pub fn construct_canvas() -> (Canvas, CanvasReader) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (Canvas { commands: tx }, CanvasReader { commands: rx })
}

impl Canvas {
    pub fn add_command(&mut self, c : CanvasCommand) {
        self.commands.send(c).expect("Compiler crashed, please try again!");
    }
}

impl CanvasReader {
    pub fn get_command(&self) -> Option<CanvasCommand> {
        self.commands.try_recv().ok()
    }
}