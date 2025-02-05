//! Terminal Platform
//!
//! Currently, dreg uses crossterm for its terminal implementation.



use crate::Program;



/// Run a dreg program inside a terminal emulator.
pub struct TerminalPlatform;

impl super::Platform for TerminalPlatform {
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
