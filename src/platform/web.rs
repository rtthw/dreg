//! Web Platform



use crate::Program;



pub struct WebPlatform;

impl super::Platform for WebPlatform {
    fn run(self, _program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
