//! Program Type



use crate::{Color, Frame, Input};



/// The object responsible for rendering [`Buffer`]s, handling user [`Input`], and responding to
/// [`Platform`] requests,
pub trait Program: 'static {
    /// Update the program's state. This method is called every tick, regardless of user input.
    fn update(&mut self) {}

    /// Render to the program's window.
    fn render(&mut self, frame: &mut Frame);

    /// This function is called whenever the running platform receives some user [`Input`].
    fn on_input(&mut self, input: Input);

    /// The ['Color'] used to clear the screen with.
    fn clear_color(&self) -> Color;
}
