//! Web Assembly Platform for Dreg



use dreg_core::prelude::*;
use wasm_bindgen::JsCast as _;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};



pub struct WasmPlatform {
    context: Context,
    buffers: [Buffer; 2],
    current: usize,
    last_known_size: Rect,
}

impl Platform for WasmPlatform {
    fn run(mut self, mut program: impl Program) -> Result<()> {
        let window = web_sys::window()
            .ok_or(anyhow::anyhow!("no global window exists"))?;
        let document = window.document()
            .ok_or(anyhow::anyhow!("should have a document on window"))?;
        // let body = document.body()
        //     .ok_or(anyhow::anyhow!("document should have a body"))?;
        let canvas = document.get_element_by_id("canvas")
            .ok_or(anyhow::anyhow!("document should have a canvas"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| anyhow::anyhow!("canvas ID should correspond to a canvas element"))?;
        let canvas_ctx = canvas.get_context("2d")
            .map_err(|_| anyhow::anyhow!("canvas should support 2D rendering"))?
            .ok_or(anyhow::anyhow!("canvas 2D rendering context should exist"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| anyhow::anyhow!("canvas 2D should be a rendering context"))?;

        while !program.should_exit() {
            let size = self.size(&canvas);
            self.autoresize(&canvas_ctx, size)?;

            let frame = Frame {
                context: &mut self.context,
                area: size,
                buffer: &mut self.buffers[self.current],
            };

            program.update(frame);
            self.render();
            self.swap_buffers();
        }

        Ok(())
    }
}

impl WasmPlatform {
    fn size(&self, canvas: &HtmlCanvasElement) -> Rect {
        Rect::new(0, 0, canvas.width() as u16, canvas.height() as u16)
    }

    fn autoresize(&mut self, canvas_ctx: &CanvasRenderingContext2d, size: Rect) -> Result<()> {
        if self.last_known_size != size {
            let text_metrics = canvas_ctx.measure_text(" ")
                .map_err(|_| anyhow::anyhow!("canvas context cannot measure text"))?;
        }

        Ok(())
    }

    fn render(&self) {}

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }
}
