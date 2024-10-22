//! Web Assembly Platform for Dreg



use std::{cell::RefCell, rc::Rc};

use dreg_core::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};
use web_sys::{js_sys, CanvasRenderingContext2d, EventTarget, HtmlCanvasElement};



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

    pub fn add_event_listener<E: wasm_bindgen::JsCast>(
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

        Ok(Self {
            program,
            canvas,
            canvas_context,
            context: Context::default(),
            buffers: [Buffer::empty(Rect::ZERO), Buffer::empty(Rect::ZERO)],
            current: 0,
            font_size: 31,
            glyph_width: 19,
            last_known_size: (0, 0),
            dimensions: (0, 0),
        })
    }

    fn size(&self) -> Rect {
        Rect::new(0, 0, self.dimensions.0, self.dimensions.1)
    }

    fn autoresize(&mut self, size: (u32, u32)) {
        if let Some(font_size) = self.program.on_platform_request("font_size") {
            if let Ok(font_size) = font_size.parse::<u16>() {
                self.font_size = font_size;
                self.glyph_width = font_size / 2;
            }
        }
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

        // HACK: This is so convoluted because `format!` returns a String.
        let font = self.program.on_platform_request("font")
            .and_then(|s| Some(s.to_string()))
            .unwrap_or(format!("{}px monospace", self.font_size));
        // let text_color = self.program.on_platform_request("web::default_fill_style")
        //     .unwrap_or("#bcbec4");
        self.canvas_context.set_font(&font);
        // self.canvas_context.set_fill_style_str(text_color);
        self.canvas_context.set_text_align("center");
        self.canvas_context.set_text_baseline("middle");
        for (x, y, cell) in updates {
            let (cell_w, cell_h) = (self.glyph_width as f64, self.font_size as f64 * 1.1);
            let (cell_x, cell_y) = (cell_w * (x as f64 + 1.0), cell_h * (y as f64+ 1.0));

            self.canvas_context.clear_rect(
                cell_x - (cell_w * 0.5),
                cell_y - (cell_h * 0.5),
                cell_w,
                cell_h,
            );
            self.canvas_context.set_fill_style_str(&cell.bg.to_string().to_lowercase());
            self.canvas_context.fill_rect(
                cell_x - (cell_w * 0.5),
                cell_y - (cell_h * 0.5),
                cell_w,
                cell_h,
            );
            self.canvas_context.set_fill_style_str(&cell.fg.to_string().to_lowercase());
            let _r = self.canvas_context.fill_text(cell.symbol(), cell_x, cell_y);
        }
    }

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }
}



// https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-fnonce-and-closureonce-with-requestanimationframe
#[allow(unused)]
struct AnimationFrameRequest {
    id: i32,
    /// The callback given to `request_animation_frame`, stored here both to prevent it
    /// from being canceled, and from having to `.forget()` it.
    _closure: Closure<dyn FnMut() -> Result<(), wasm_bindgen::JsValue>>,
}

#[allow(unused)]
struct ResizeObserverContext {
    resize_observer: web_sys::ResizeObserver,
    closure: Closure<dyn FnMut(js_sys::Array)>,
}

#[allow(unused)]
struct EventHandle {
    target: web_sys::EventTarget,
    event_name: String,
    closure: Closure<dyn FnMut(web_sys::Event)>,
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
fn install_event_handlers(platform: &WasmPlatform) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = platform.try_lock().unwrap().canvas().clone();

    install_mousemove(platform, &document)?;
    install_pointerdown(platform, &canvas)?;
    install_pointerup(platform, &document)?;

    Ok(())
}

fn install_mousemove(platform: &WasmPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "mousemove", |event: web_sys::MouseEvent, runner| {
        let (x, y) = pos_from_mouse_event(&event, runner.dimensions);
        runner.program.on_input(Input::MouseMove(x, y));
        event.prevent_default();
    })
}

fn install_pointerdown(platform: &WasmPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "pointerdown", |event: web_sys::PointerEvent, runner| {
        // let (x, y) = pos_from_mouse_event(&event, runner.dimensions);
        if let Some(scancode) = scancode_from_mouse_event(&event) {
            runner.program.on_input(Input::KeyDown(scancode));
            event.prevent_default();
        }
    })
}

fn install_pointerup(platform: &WasmPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "pointerup", |event: web_sys::PointerEvent, runner| {
        // let (x, y) = pos_from_mouse_event(&event, runner.dimensions);
        if let Some(scancode) = scancode_from_mouse_event(&event) {
            runner.program.on_input(Input::KeyUp(scancode));
            event.prevent_default();
        }
    })
}

fn install_resize_observer(platform: &WasmPlatform) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new({
        let platform = platform.clone();
        move |entries: js_sys::Array| {
            // Only call the wrapped closure if the egui code has not panicked
            if let Some(mut runner_lock) = platform.try_lock() {
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
                if let Err(_e) = platform.request_animation_frame() {
                    // TODO: Logging.
                };
            }
        }
    }) as Box<dyn FnMut(js_sys::Array)>);

    let observer = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref())?;
    let options = web_sys::ResizeObserverOptions::new();
    options.set_box(web_sys::ResizeObserverBoxOptions::ContentBox);
    if let Some(runner_lock) = platform.try_lock() {
        observer.observe_with_options(runner_lock.canvas(), &options);
        drop(runner_lock);
        platform.set_resize_observer(observer, closure);
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

fn pos_from_mouse_event(event: &web_sys::MouseEvent, (cols, rows): (u16, u16)) -> (u16, u16) {
    (
        event.screen_x() as u16 / rows,
        event.screen_y() as u16 / cols,
    )
}

fn scancode_from_mouse_event(event: &web_sys::MouseEvent) -> Option<Scancode> {
    match event.button() {
        0 => Some(Scancode::LMB),
        1 => Some(Scancode::MMB),
        2 => Some(Scancode::RMB),
        3 => Some(Scancode::MOUSE_BACK),
        4 => Some(Scancode::MOUSE_FORWARD),
        _ => None,
    }
}
