//! Dreg Core



pub mod color;
pub mod input;
pub mod text;
pub mod text_modifier;

pub use color::*;
pub use input::*;
pub use text::*;
pub use text_modifier::*;



/// The object responsible for rendering [`Buffer`]s, handling user [`Input`], and responding to
/// [`Platform`] requests,
pub trait Program: 'static {
    /// Update the program's state. This method is called every frame, regardless of user input.
    fn update(&mut self, frame: Frame);

    /// This function is called whenever the running platform receives some user [`Input`].
    fn on_input(&mut self, input: Input);

    /// This function is called every frame to determine whether the program should exit.
    fn should_exit(&self) -> bool;
}



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
