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
        let canvas_ctx = canvas.get_context("2d")
            .map_err(|_| anyhow::anyhow!("canvas should support 2D rendering"))?
            .ok_or(anyhow::anyhow!("canvas 2D rendering context should exist"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| anyhow::anyhow!("canvas 2D should be a rendering context"))?;

        while !program.should_exit() {

        }

        Ok(())
    }
}
