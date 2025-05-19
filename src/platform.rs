//! Platform



mod terminal;
pub use terminal::*;

use crate::Program;



pub trait Platform {
    /// Run the given ['Program'].
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>>;
}
