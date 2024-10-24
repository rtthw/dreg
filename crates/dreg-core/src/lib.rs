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
        run_program,
    };
}

use prelude::*;



pub fn run_program(program: impl Program, platform: impl Platform) -> Result<()> {
    platform.run(program)?;
    Ok(())
}



pub trait Program: 'static {
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

    fn should_exit(&self) -> bool;
}

pub trait Platform {
    fn run(self, program: impl Program) -> Result<()>;
}

pub struct Frame<'a> {
    pub area: Rect,
    pub buffer: &'a mut Buffer,
}
