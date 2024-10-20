//! Web Assembly Platform for Dreg



use std::{cell::RefCell, rc::Rc};

use dreg_core::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};
use web_sys::{js_sys, CanvasRenderingContext2d, HtmlCanvasElement};



pub mod prelude {
    pub extern crate wasm_bindgen;
    pub extern crate web_sys;
    pub use crate::WasmPlatform;
}



#[derive(Clone)]
pub struct WasmPlatform {
    runner: Rc<RefCell<Option<Runner>>>,
    frame: Rc<RefCell<Option<AnimationFrameRequest>>>,
    resize_observer: Rc<RefCell<Option<ResizeObserverContext>>>,
}

impl Platform for WasmPlatform {
    fn run(self, program: impl Program) -> Result<()> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas")
            .ok_or(anyhow::anyhow!("document should have a canvas"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| anyhow::anyhow!("canvas ID should correspond to a canvas element"))?;

        let runner = Runner::new(Box::new(program), canvas)?;
        {
            // Make sure the canvas can be given focus.
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex
            runner.canvas().set_tab_index(0);

            // Don't outline the canvas when it has focus:
            runner.canvas().style().set_property("outline", "none")
                .map_err(|e| anyhow::anyhow!("could not set canvas style: {e:?}"))?;
            runner.canvas().style().set_property("background-color", "#1e1f22")
                .map_err(|e| anyhow::anyhow!("could not set canvas style: {e:?}"))?;
        }
        self.runner.replace(Some(runner));
        {
            install_event_handlers(&self)
                .map_err(|e| anyhow::anyhow!("could not install event handlers: {e:?}"))?;
            install_resize_observer(&self)
                .map_err(|e| anyhow::anyhow!("could not install resize observer: {e:?}"))?;
        }

        Ok(())
    }
}

impl WasmPlatform {
    pub fn new() -> Self {
        Self {
            runner: Rc::new(RefCell::new(None)),
            frame: Default::default(),
            resize_observer: Default::default(),
        }
    }

    fn try_lock(&self) -> Option<std::cell::RefMut<'_, Runner>> {
        let lock = self.runner.try_borrow_mut().ok()?;
        std::cell::RefMut::filter_map(lock, |lock| -> Option<&mut Runner> { lock.as_mut() }).ok()
    }

    fn request_animation_frame(&self) -> Result<(), wasm_bindgen::JsValue> {
        if self.frame.borrow().is_some() {
            // There is already an animation frame in flight.
            return Ok(());
        }

        let window = web_sys::window().unwrap();
        let closure = Closure::once({
            let proxy = self.clone();
            move || {
                // We can render now, so clear the animation frame.
                // This drops the `closure` and allows another animation frame to be scheduled.
                let _ = proxy.frame.take();
                update_platform(&proxy)
            }
        });

        let id = window.request_animation_frame(closure.as_ref().unchecked_ref())?;
        self.frame.borrow_mut().replace(AnimationFrameRequest {
            id,
            _closure: closure,
        });

        Ok(())
    }

    fn set_resize_observer(
        &self,
        resize_observer: web_sys::ResizeObserver,
        closure: Closure<dyn FnMut(js_sys::Array)>,
    ) {
        self.resize_observer.borrow_mut().replace(ResizeObserverContext {
            resize_observer,
            closure,
        });
    }
}

pub struct Runner {
    program: Box<dyn Program>,
    canvas: HtmlCanvasElement,
    canvas_context: CanvasRenderingContext2d,
    context: Context,
    buffers: [Buffer; 2],
    current: usize,
    font_size: u16,
    glyph_width: u16,
    last_known_size: (u32, u32),
    dimensions: (u16, u16),
}

impl Runner {
    fn new(program: Box<dyn Program>, canvas: HtmlCanvasElement) -> Result<Self> {
        let canvas_context = canvas.get_context("2d")
            .map_err(|_| anyhow::anyhow!("canvas should support 2D rendering"))?
            .ok_or(anyhow::anyhow!("canvas 2D rendering context should exist"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| anyhow::anyhow!("canvas 2D should be a rendering context"))?;
        // let text_metrics = canvas_context.measure_text("█")
        //     .map_err(|_| anyhow::anyhow!("canvas context cannot measure text"))?;

        Ok(Self {
            program,
            canvas,
            canvas_context,
            context: Context::default(),
            buffers: [Buffer::empty(Rect::ZERO), Buffer::empty(Rect::ZERO)],
            current: 0,
            font_size: 31,
            glyph_width: 19, // text_metrics.width() as u16,
            last_known_size: (0, 0),
            dimensions: (0, 0),
        })
    }

    fn size(&self) -> Rect {
        Rect::new(0, 0, self.dimensions.0, self.dimensions.1)
    }

    fn autoresize(&mut self, size: (u32, u32)) {
        if self.last_known_size != size {
            let width = size.0 as u16 / self.glyph_width;
            let height = size.1 as u16 / self.font_size;

            self.resize(Rect::new(0, 0, width, height));
            self.last_known_size = size;
        }
    }

    fn resize(&mut self, area: Rect) {
        self.buffers[self.current].resize(area);
        self.buffers[1 - self.current].resize(area);
        self.dimensions = (area.width, area.height);
    }

    fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    fn update(&mut self) {
        self.autoresize((self.canvas.width(), self.canvas.height()));
        let size = self.size();
        let frame = Frame {
            context: &mut self.context,
            area: size,
            buffer: &mut self.buffers[self.current],
        };
        self.program.update(frame);
        self.render();
        self.swap_buffers();
    }

    fn render(&self) {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let updates = previous_buffer.diff(current_buffer).into_iter();

        self.canvas_context.set_font("31px \"Courier New\", monospace");
        self.canvas_context.set_text_align("center");
        self.canvas_context.set_fill_style_str("#bcbec4");
        for (x, y, cell) in updates {
            let (real_x, real_y) = (self.glyph_width * (x + 1), self.font_size * (y + 1));
            let _r = self.canvas_context.fill_text(cell.symbol(), real_x as f64, real_y as f64);
        }
    }

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }
}



// https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-fnonce-and-closureonce-with-requestanimationframe
struct AnimationFrameRequest {
    id: i32,
    /// The callback given to `request_animation_frame`, stored here both to prevent it
    /// from being canceled, and from having to `.forget()` it.
    _closure: Closure<dyn FnMut() -> Result<(), wasm_bindgen::JsValue>>,
}

struct ResizeObserverContext {
    resize_observer: web_sys::ResizeObserver,
    closure: Closure<dyn FnMut(js_sys::Array)>,
}



fn update_platform(platform: &WasmPlatform) -> Result<(), wasm_bindgen::JsValue> {
    // Only paint and schedule if there has been no panic
    if let Some(mut runner_lock) = platform.try_lock() {
        runner_lock.update();
        drop(runner_lock);
        platform.request_animation_frame()?;
    }
    Ok(())
}

// TODO: Implement the event-handling system.
fn install_event_handlers(_proxy: &WasmPlatform) -> Result<(), JsValue> {
    Ok(())
}

fn install_resize_observer(proxy: &WasmPlatform) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new({
        let runner_ref = proxy.clone();
        move |entries: js_sys::Array| {
            // Only call the wrapped closure if the egui code has not panicked
            if let Some(mut runner_lock) = runner_ref.try_lock() {
                let canvas = runner_lock.canvas();
                let (width, height) = match get_display_size(&entries) {
                    Ok(v) => v,
                    Err(_e) => {
                        // TODO: Logging.
                        return;
                    }
                };
                canvas.set_width(width);
                canvas.set_height(height);

                // Force an update.
                runner_lock.update();
                drop(runner_lock);
                // we rely on the resize observer to trigger the first `request_animation_frame`:
                if let Err(_e) = runner_ref.request_animation_frame() {
                    // TODO: Logging.
                };
            }
        }
    }) as Box<dyn FnMut(js_sys::Array)>);

    let observer = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref())?;
    let options = web_sys::ResizeObserverOptions::new();
    options.set_box(web_sys::ResizeObserverBoxOptions::ContentBox);
    if let Some(runner_lock) = proxy.try_lock() {
        observer.observe_with_options(runner_lock.canvas(), &options);
        drop(runner_lock);
        proxy.set_resize_observer(observer, closure);
    }

    Ok(())
}

// Code ported to Rust from:
// https://webglfundamentals.org/webgl/lessons/webgl-resizing-the-canvas.html
fn get_display_size(resize_observer_entries: &js_sys::Array) -> Result<(u32, u32), JsValue> {
    let width;
    let height;
    let mut dpr = web_sys::window().unwrap().device_pixel_ratio();

    let entry: web_sys::ResizeObserverEntry = resize_observer_entries.at(0).dyn_into()?;
    if JsValue::from_str("devicePixelContentBoxSize").js_in(entry.as_ref()) {
        // NOTE: Only this path gives the correct answer for most browsers.
        // Unfortunately this doesn't work perfectly everywhere.
        let size: web_sys::ResizeObserverSize =
            entry.device_pixel_content_box_size().at(0).dyn_into()?;
        width = size.inline_size();
        height = size.block_size();
        dpr = 1.0; // no need to apply
    } else if JsValue::from_str("contentBoxSize").js_in(entry.as_ref()) {
        let content_box_size = entry.content_box_size();
        let idx0 = content_box_size.at(0);
        if !idx0.is_undefined() {
            let size: web_sys::ResizeObserverSize = idx0.dyn_into()?;
            width = size.inline_size();
            height = size.block_size();
        } else {
            // legacy
            let size = JsValue::clone(content_box_size.as_ref());
            let size: web_sys::ResizeObserverSize = size.dyn_into()?;
            width = size.inline_size();
            height = size.block_size();
        }
    } else {
        // legacy
        let content_rect = entry.content_rect();
        width = content_rect.width();
        height = content_rect.height();
    }

    Ok(((width.round() * dpr) as u32, (height.round() * dpr) as u32))
}
