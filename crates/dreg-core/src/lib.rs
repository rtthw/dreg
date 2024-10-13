//! Dreg Core Functionality


pub mod buffer;
pub mod context;
pub mod primitives;
pub mod style;

pub mod prelude {
    pub use crate::{
        buffer::{Buffer, Cell},
        primitives::Rect,
        style::{Color, ColorMode, Modifier, Style},
        context::{Context, Input, Scancode},
        Frame,
        Platform,
    };
}

use prelude::*;

pub trait Platform {
    fn ctx(&mut self) -> &mut Context;

    fn render(&mut self, render_fn: impl FnMut(&mut Frame));
}

pub struct Frame<'a> {
    pub buffer: &'a mut Buffer,
}
