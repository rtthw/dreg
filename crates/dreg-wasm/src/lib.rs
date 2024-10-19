//! Web Assembly Platform for Dreg



use dreg_core::prelude::*;
use wasm_bindgen::JsCast as _;



pub struct WasmPlatform {

}

impl Platform for WasmPlatform {
    fn run(self, program: impl Program) -> Result<()> {
        let window = web_sys::window()
            .ok_or(anyhow::anyhow!("no global window exists"))?;
        let document = window.document()
            .ok_or(anyhow::anyhow!("should have a document on window"))?;
        let body = document.body()
            .ok_or(anyhow::anyhow!("document should have a body"))?;
        let canvas = document.get_element_by_id("canvas")
            .ok_or(anyhow::anyhow!("document should have a canvas"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| anyhow::anyhow!("canvas ID should correspond to a canvas element"))?;

        while !program.should_exit() {

        }

        Ok(())
    }
}
