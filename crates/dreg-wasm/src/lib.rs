//! Web Assembly Platform for Dreg



mod event_handlers;

use std::{cell::RefCell, rc::Rc};

use dreg_core::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast as _};
use web_sys::{js_sys, CanvasRenderingContext2d, HtmlCanvasElement};

use event_handlers::*;



pub mod prelude {
    pub extern crate wasm_bindgen;
    pub extern crate web_sys;
    pub use crate::WasmPlatform;
}



/// The platform for running dreg programs on web targets.
#[derive(Clone)]
pub struct WasmPlatform {
    runner: Rc<RefCell<Option<Runner>>>,
    frame: Rc<RefCell<Option<AnimationFrameRequest>>>,
    resize_observer: Rc<RefCell<Option<ResizeObserverContext>>>,
    event_handles: Rc<RefCell<Vec<EventHandle>>>,
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
    /// Create a new instance of the wasm platform.
    pub fn new() -> Self {
        Self {
            runner: Rc::new(RefCell::new(None)),
            frame: Default::default(),
            resize_observer: Default::default(),
            event_handles: Rc::new(RefCell::new(Default::default())),
        }
    }

    fn try_lock(&self) -> Option<std::cell::RefMut<'_, Runner>> {
        let lock = self.runner.try_borrow_mut().ok()?;
        std::cell::RefMut::filter_map(lock, |lock| -> Option<&mut Runner> { lock.as_mut() }).ok()
    }

    fn add_event_listener<E: wasm_bindgen::JsCast>(
        &self,
        target: &web_sys::EventTarget,
        event_name: &'static str,
        mut closure: impl FnMut(E, &mut Runner) + 'static,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let runner_ref = self.clone();

        // Create a JS closure based on the FnMut provided.
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            // Only call the wrapped closure if the platform is still available.
            if let Some(mut runner_lock) = runner_ref.try_lock() {
                // Cast the event to the expected event type.
                let event = event.unchecked_into::<E>();
                closure(event, &mut runner_lock);
            }
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Add the event listener to the target.
        target.add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;

        // TODO: Remember the event to unsubscribe on poisoning.
        let handle = EventHandle {
            target: target.clone(),
            event_name: event_name.to_owned(),
            closure,
        };
        self.event_handles.borrow_mut().push(handle);

        Ok(())
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

/// Internal object used as a proxy between the wasm platform and the dreg program.
///
/// Unfortunately, this added complexity is necessary because of the absolute horror that is
/// contemporary web development.
struct Runner {
    program: Box<dyn Program>,
    canvas: HtmlCanvasElement,
    canvas_context: CanvasRenderingContext2d,
    buffers: [Buffer; 2],
    current: usize,
    font_height: f64,
    glyph_width: f64,
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

        Ok(Self {
            program,
            canvas,
            canvas_context,
            buffers: [Buffer::empty(Rect::ZERO), Buffer::empty(Rect::ZERO)],
            current: 0,
            font_height: 0.0,
            glyph_width: 0.0,
            last_known_size: (0, 0),
            dimensions: (0, 0),
        })
    }

    fn size(&self) -> Rect {
        Rect::new(0, 0, self.dimensions.0, self.dimensions.1)
    }

    fn autoresize(&mut self, size: (u32, u32)) {
        if self.last_known_size != size {
            let font = self.program.on_platform_request("font").unwrap_or("30px monospace");
            self.canvas_context.set_font(&font);
            let text_metrics = self.canvas_context.measure_text("█").unwrap();
            self.font_height = text_metrics.actual_bounding_box_ascent().abs()
                + text_metrics.actual_bounding_box_descent().abs();
            self.glyph_width = text_metrics.actual_bounding_box_left().abs()
                + text_metrics.actual_bounding_box_right().abs();
            let width = size.0 as f64 / self.glyph_width;
            let height = size.1 as f64 / self.font_height;

            self.resize(Rect::new(0, 0, width.floor() as u16, height.floor() as u16));
            self.last_known_size = size;
        }
    }

    fn resize(&mut self, area: Rect) {
        self.buffers[self.current].resize(area);
        self.buffers[1 - self.current].resize(area);
        self.dimensions = (area.width, area.height);
        self.buffers[1 - self.current].reset();
    }

    fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    fn update(&mut self) {
        self.autoresize((self.canvas.width(), self.canvas.height()));
        let mut commands = vec![];
        let size = self.size();
        let frame = Frame {
            area: size,
            buffer: &mut self.buffers[self.current],
            commands: &mut commands,
        };
        self.program.update(frame);
        self.render();
        self.swap_buffers();
    }

    fn render(&mut self) {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let updates = previous_buffer.diff(current_buffer).into_iter();

        let fg_color = self.program.on_platform_request("web::default_fg_style")
            .unwrap_or("#bcbec4")
            .to_string();
        let bg_color = self.program.on_platform_request("web::default_bg_style")
            .unwrap_or("#1e1f22")
            .to_string();
        let line_width = self.program.on_platform_request("web::default_line_width")
            .unwrap_or("2.0")
            .parse::<f64>()
            .unwrap_or(2.0);

        self.canvas_context.set_text_align("left");
        self.canvas_context.set_text_baseline("top");

        'update_loop: for (x, y, cell) in updates {
            let (cell_w, cell_h) = (self.glyph_width, self.font_height);
            let (cell_x, cell_y) = (cell_w * x as f64, cell_h * y as f64);

            let mut font = self.program.on_platform_request("font")
                .unwrap_or("30px monospace")
                .to_string();
            self.canvas_context.clear_rect(cell_x, cell_y, cell_w, cell_h);

            let mut fg_style = if cell.fg == Color::Reset {
                fg_color.clone()
            } else {
                cell.fg.to_string().to_lowercase()
            };
            let mut bg_style = if cell.bg == Color::Reset {
                bg_color.clone()
            } else {
                cell.bg.to_string().to_lowercase()
            };
            let mut draw_line_at: Option<f64> = None;
            for m in cell.modifier.iter() {
                match m {
                    Modifier::HIDDEN => {
                        continue 'update_loop;
                    }
                    Modifier::CROSSED_OUT => {
                        draw_line_at = Some(cell_y + (cell_h * 0.5));
                    }
                    Modifier::UNDERLINED => {
                        draw_line_at = Some(cell_y + cell_h);
                    }
                    Modifier::REVERSED => {
                        std::mem::swap(&mut fg_style, &mut bg_style);
                    }
                    Modifier::BOLD => {
                        font = format!("bold {font}");
                    }
                    Modifier::ITALIC => {
                        font = format!("italic {font}");
                    }
                    _ => {}
                }
            }
            self.canvas_context.set_font(&font);
            self.canvas_context.set_fill_style_str(&bg_style);
            self.canvas_context.fill_rect(cell_x, cell_y, cell_w, cell_h);
            self.canvas_context.set_fill_style_str(&fg_style);
            let _r = self.canvas_context.fill_text(cell.symbol(), cell_x, cell_y);
            if let Some(line_y_pos) = draw_line_at {
                self.canvas_context.begin_path();
                self.canvas_context.set_stroke_style_str(&fg_style);
                self.canvas_context.set_line_width(line_width);
                self.canvas_context.move_to(cell_x, line_y_pos);
                self.canvas_context.line_to(cell_x + cell_w, line_y_pos);
                self.canvas_context.stroke();
            }
        }
    }

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }
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
