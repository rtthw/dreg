//! Native Platform



use crate::Program;



pub struct NativePlatform;

impl super::Platform for NativePlatform {
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
