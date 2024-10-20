//! Web Assembly Platform for Dreg



pub mod util;

use dreg_core::prelude::*;
use wasm_bindgen::JsCast as _;
use web_sys::CanvasRenderingContext2d;



pub mod prelude {
    pub extern crate wasm_bindgen;
    pub extern crate web_sys;
    pub use crate::WasmPlatform;
}



pub struct WasmPlatform {
    context: Context,
    buffers: [Buffer; 2],
    current: usize,
    font_size: u16,
    last_known_size: (u32, u32),
    dimensions: (u16, u16),
}

impl Platform for WasmPlatform {
    fn run(mut self, mut program: impl Program) -> Result<()> {
        let window = util::window();
        let document = util::document(&window);
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
            self.autoresize(&canvas_ctx, (canvas.width(), canvas.height()))?;

            let size = self.size();
            let frame = Frame {
                context: &mut self.context,
                area: size,
                buffer: &mut self.buffers[self.current],
            };

            program.update(frame);
            self.render(&canvas_ctx);
            self.swap_buffers();
        }

        Ok(())
    }
}

impl WasmPlatform {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
            buffers: [Buffer::empty(Rect::ZERO), Buffer::empty(Rect::ZERO)],
            current: 0,
            font_size: 17,
            last_known_size: (0, 0),
            dimensions: (0, 0),
        }
    }

    fn size(&self) -> Rect {
        Rect::new(0, 0, self.dimensions.0, self.dimensions.1)
    }

    fn autoresize(&mut self, canvas_ctx: &CanvasRenderingContext2d, size: (u32, u32)) -> Result<()> {
        if self.last_known_size != size {
            let text_metrics = canvas_ctx.measure_text("█")
                .map_err(|_| anyhow::anyhow!("canvas context cannot measure text"))?;
            let width = size.0 as f64 / text_metrics.width();
            let height = size.1 as u16 / self.font_size;

            self.resize(Rect::new(0, 0, width as u16, height));
            self.last_known_size = size;
        }

        Ok(())
    }

    fn resize(&mut self, area: Rect) {
        self.buffers[self.current].resize(area);
        self.buffers[1 - self.current].resize(area);
        self.dimensions = (area.width, area.height);
    }

    fn render(&self, canvas_ctx: &CanvasRenderingContext2d) {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let updates = previous_buffer.diff(current_buffer).into_iter();

        for (x, y, cell) in updates {
            let (real_x, real_y) = (self.dimensions.0 * x, self.dimensions.1 * y);
            let _res = canvas_ctx.fill_text(cell.symbol(), real_x as f64, real_y as f64);
        }
    }

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }
}
