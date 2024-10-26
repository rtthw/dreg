//! Dreg Core Functionality



pub mod buffer;
pub mod input;
pub mod primitives;
pub mod style;

pub mod prelude {
    pub use anyhow::Result;
    pub use crate::{
        buffer::{Buffer, Cell},
        primitives::Rect,
        style::{Color, ColorMode, Modifier, Style},
        input::{InputContext, Input, Scancode},
        Frame,
        Platform,
        Program,
    };
}

use prelude::*;



/// The object responsible for rendering [`Buffer`]s, handling user [`Input`], and responding to
/// [`Platform`] requests,
pub trait Program: 'static {
    /// Update the program's state.
    ///
    /// A "correct" platform implementation will call this every frame, regardless of user input.
    fn update(&mut self, frame: Frame);

    /// This function is called whenever the running platform receives some user [`Input`].
    fn on_input(&mut self, input: Input);

    /// This function is called whenever the running platform needs some information from the
    /// program.
    ///
    /// # Notes
    /// - Never called by terminals.
    /// - Called each update pass on web.
    fn on_platform_request(&mut self, request: &str) -> Option<&str>;

    /// This function is called every frame to determine whether the program should exit safely at
    /// the end of the current frame.
    fn should_exit(&self) -> bool;
}

/// The object responsible for handling the running [`Program`].
pub trait Platform {
    /// Run this platform with the given program.
    fn run(self, program: impl Program) -> Result<()>;
}

/// A single instance of the current display state.
pub struct Frame<'a> {
    pub area: Rect,
    pub buffer: &'a mut Buffer,
    /// A set of commands that will be processed by the platform at the end of this frame.
    pub commands: &'a mut Vec<&'a str>,
}

impl<'a> Frame<'a> {
    /// Shorthand for `&mut self.buffer`.
    pub fn buf(&'a mut self) -> &'a mut Buffer {
        &mut self.buffer
    }

    /// Get the size of this frame.
    pub fn size(&self) -> (u16, u16) {
        (self.area.width, self.area.height)
    }

    /// Queue a new command to be processed by the platform at the end of this frame.
    pub fn queue(&mut self, command: &'a str) {
        self.commands.push(command)
    }

    /// Queue an iterator of commands to be processed by the platform at the end of this frame.
    pub fn queue_many(&mut self, command_iter: impl Iterator<Item = &'a str>) {
        self.commands.extend(command_iter)
    }
}
