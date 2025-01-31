//! Dreg Core



pub mod color;
pub use color::*;

use compact_str::CompactString;



/// The object responsible for rendering [`Buffer`]s, handling user [`Input`], and responding to
/// [`Platform`] requests,
pub trait Program: 'static {
    /// Update the program's state. This method is called every frame, regardless of user input.
    fn update(&mut self, frame: Frame);

    // /// This function is called whenever the running platform receives some user [`Input`].
    // fn on_input(&mut self, input: Input);

    /// This function is called every frame to determine whether the program should exit.
    fn should_exit(&self) -> bool;
}



pub struct Frame<'a> {
    pub buffer: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    pub fn render(&mut self, text: Text) {
        self.buffer.content.push(text);
    }
}



pub struct Buffer {
    content: Vec<Text>,
}

pub struct Text {
    content: CompactString,
    x: u16,
    y: u16,
}
