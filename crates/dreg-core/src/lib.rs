//! Dreg Core Functionality


pub mod buffer;
pub mod context;
pub mod primitives;
pub mod style;

pub mod prelude {
    pub use anyhow::Result;
    pub use crate::{
        buffer::{Buffer, Cell},
        primitives::Rect,
        style::{Color, ColorMode, Modifier, Style},
        context::{Context, Input, Scancode},
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



pub trait Program {
    fn update(&mut self, frame: Frame);
    fn should_exit(&self) -> bool;
}

pub trait Platform {
    fn run(self, program: impl Program) -> Result<()>;
}

pub struct Frame<'a> {
    pub context: &'a mut Context,
    pub area: Rect,
    pub buffer: &'a mut Buffer,
}
