//! Core Types



pub mod color;
pub mod input;
pub mod text;
pub mod text_modifier;

pub use color::*;
pub use input::*;
pub use text::*;
pub use text_modifier::*;



pub struct Frame<'a> {
    /// The width of the frame, in pixels.
    pub width: f32,
    /// The height of the frame, in pixels.
    pub height: f32,
    /// The frame's [`Buffer`].
    pub buffer: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    /// Render the given ['Text'] to the frame.
    pub fn render(&mut self, text: Text) {
        self.buffer.content.push(text);
    }
}



pub struct Buffer {
    /// The buffer's contents.
    pub content: Vec<Text>,
}
