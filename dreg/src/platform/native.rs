//! Native Platform



pub struct NativePlatform;

impl super::Platform for NativePlatform {
    fn run(self, program: impl dreg_core::Program) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
