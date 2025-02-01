//! Program Type



use crate::{Frame, Input};



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
