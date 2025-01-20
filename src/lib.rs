//! Dreg
//!
//! A simple text-based user interface library.



pub mod prelude {
    pub use dreg_core::prelude::*;
    #[cfg(feature = "term")]
    pub use dreg_term::prelude::*;
    #[cfg(feature = "wasm")]
    pub use dreg_wasm::prelude::*;
}
