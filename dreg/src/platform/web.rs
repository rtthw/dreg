//! Web Platform



pub struct WebPlatform;

impl super::Platform for WebPlatform {
    fn run(self, program: impl dreg_core::Program) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
