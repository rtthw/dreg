//! Program Type



use crate::{Color, Frame, Input};



/// The object responsible for rendering [`Buffer`]s, handling user [`Input`], and responding to
/// [`Platform`] requests,
pub trait Program: 'static {
    /// Update the program's state. This method is called every frame, regardless of user input.
    fn update(&mut self, frame: &mut Frame);

    /// This function is called whenever the running platform receives some user [`Input`].
    fn on_input(&mut self, input: Input);

    fn clear_color(&self) -> Color;
}
