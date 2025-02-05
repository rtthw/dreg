//! Platform



pub mod native;
pub mod terminal;
pub mod web;

pub use native::*;
pub use terminal::*;
pub use web::*;

use crate::Program;



pub trait Platform {
    /// Run the given ['Program'].
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>>;
}
