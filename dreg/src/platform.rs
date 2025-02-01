//! Platform



pub mod native;
pub mod web;

pub use native::*;
pub use web::*;

use dreg_core::Program;



pub trait Platform {
    /// Run the given ['Program'].
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>>;
}
