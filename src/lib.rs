//! Dreg
//!
//! A simple text-based user interface library.



pub mod prelude {
    pub use dreg_core::*;
    #[cfg(feature = "crossterm")]
    pub use dreg_crossterm::prelude::*;
}
