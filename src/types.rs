//! Core Types



pub mod area;
pub mod color;
pub mod input;
pub mod text;
pub mod text_modifier;

pub use area::*;
pub use color::*;
pub use input::*;
pub use text::*;
pub use text_modifier::*;



pub struct Frame<'a> {
    /// The width of the frame, in cells.
    pub cols: u16,
    /// The height of the frame, in cells.
    pub rows: u16,
    /// The frame's [`Buffer`].
    pub buffer: &'a mut Buffer,
    /// Flag to indicate whether the platform should safely exit at the end of this frame.
    pub should_exit: bool,
}

impl<'a> Frame<'a> {
    /// Render the given ['Text'] to the frame.
    pub fn render(&mut self, text: Text) {
        self.buffer.render(text);
    }
}



#[derive(Eq, PartialEq)]
pub struct Buffer {
    /// The buffer's contents.
    pub content: Vec<Text>,
}

impl Buffer {
    /// Create a new empty buffer.
    pub fn empty() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    /// Render the given ['Text'] to the buffer.
    pub fn render(&mut self, text: Text) {
        self.content.push(text);
    }

    /// Clear all [`Text`] pieces in the buffer.
    pub fn clear(&mut self) {
        self.content.clear();
    }
}
