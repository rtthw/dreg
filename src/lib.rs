//! Dreg
//! 
//! A terminal application development library.
//! 
//! ## Simple Sample
//! 
//! ```rust
//! 
//! use dreg::prelude::*;
//! 
//! fn main() {
//!     let mut prog = MyProgram {
//!         should_quit: false,
//!     };
//!     let mut term = Terminal::new();
//!     while !prog.should_quit {
//!     }
//! }
//! 
//! struct MyProgram {
//!     should_quit: bool,
//! }
//! 
//! impl Program for MyProgram {
//!     fn render(&mut self, ctx: &mut Context, area: Rect, buf: &mut Buffer) {
//!     }
//! }
//! 
//! ```



#[cfg(feature = "anim")]
pub mod anim;
pub mod block;
pub mod buffer;
pub mod ctx;
#[cfg(feature = "img")]
pub mod image;
pub mod label;
pub mod primitives;
pub mod shapes;
pub mod style;
pub mod terminal;

pub mod prelude {
    #[cfg(feature = "anim")]
    pub use crate::anim::{
        Animation, AnimationTimer, CellFilter, CellIterator, CellSelector, Effect,
        coalesce,
        dissolve,
    };
    #[cfg(feature = "img")]
    pub use crate::image::Image;
    #[cfg(all(feature = "kitty_img", feature = "kitty_img"))]
    pub use crate::image::kitty::*;
    pub use crate::{
        block::{Block, BorderType, Borders, Clear},
        buffer::{Buffer, Cell},
        ctx::{Context, Input, KeyCode, KeyModifiers},
        label::Label,
        primitives::{Margin, Offset, Padding, Pos, Rect},
        shapes::line::{Line, LineCapping, LineDirection, LineType},
        style::{Color, Modifier, ColorMode, Style},
        terminal::{Element, Frame, Program, Terminal, TerminalSettings},
    };
}
