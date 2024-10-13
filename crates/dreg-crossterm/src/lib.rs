//! Crossterm Platform


use dreg_core::prelude::*;

pub mod prelude {
    pub extern crate crossterm;
    pub use crate::CrosstermPlatform;
}

pub struct CrosstermPlatform {
    ctx: Context,

    /// Holds the results of the current and previous draw calls. The two are compared at the end
    /// of each draw pass to output the necessary updates to the terminal
    buffers: [Buffer; 2],
    /// Index of the current buffer in the previous array
    current: usize,
}

impl Platform for CrosstermPlatform {
    fn run(self, program: impl dreg_core::Program) -> Result<()> {
        Ok(())
    }
}
