//! Web Platform



use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};
use web_sys::{js_sys, CanvasRenderingContext2d, EventTarget, HtmlCanvasElement};

use crate::{Buffer, Color, Frame, Input, Platform, Program, Scancode, TextModifier};



/// The platform for running dreg programs on web targets.
#[derive(Clone)]
pub struct WebPlatform {
    runner: Rc<RefCell<Option<Runner>>>,
    frame: Rc<RefCell<Option<AnimationFrameRequest>>>,
    resize_observer: Rc<RefCell<Option<ResizeObserverContext>>>,
    event_handles: Rc<RefCell<Vec<EventHandle>>>,
}

impl Platform for WebPlatform {
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas")
            .expect("document should have a canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("canvas ID should correspond to a canvas element");

        let runner = Runner::new(Box::new(program), canvas)?;
        {
            // Make sure the canvas can be given focus.
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex
            runner.canvas().set_tab_index(0);

            // Don't outline the canvas when it has focus:
            runner.canvas().style().set_property("outline", "none")
                .map_err(|e| format!("could not set canvas style: {e:?}"))?;
        }
        self.runner.replace(Some(runner));
        {
            install_event_handlers(&self)
                .map_err(|e| format!("could not install event handlers: {e:?}"))?;
            install_resize_observer(&self)
                .map_err(|e| format!("could not install resize observer: {e:?}"))?;
        }

        Ok(())
    }
}

impl WebPlatform {
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
    fn new(program: Box<dyn Program>, canvas: HtmlCanvasElement) -> Result<Self, Box<dyn std::error::Error>> {
        let canvas_context = canvas.get_context("2d")
            .expect("canvas should support 2D rendering")
            .expect("canvas 2D rendering context should exist")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("canvas 2D should be a rendering context");

        Ok(Self {
            program,
            canvas,
            canvas_context,
            buffers: [Buffer::empty(), Buffer::empty()],
            current: 0,
            font_height: 0.0,
            glyph_width: 0.0,
            last_known_size: (0, 0),
            dimensions: (0, 0),
        })
    }

    // fn size(&self) -> Rect {
    //     Rect::new(0, 0, self.dimensions.0, self.dimensions.1)
    // }

    fn autoresize(&mut self, size: (u32, u32)) {
        if self.last_known_size != size {
            let font = format!("{}px monospace", self.program.scale() as u16);
            self.canvas_context.set_font(&font);
            let text_metrics = self.canvas_context.measure_text("█").unwrap();
            self.font_height = text_metrics.actual_bounding_box_ascent().abs()
                + text_metrics.actual_bounding_box_descent().abs();
            self.glyph_width = text_metrics.actual_bounding_box_left().abs()
                + text_metrics.actual_bounding_box_right().abs();
            let width = size.0 as f64 / self.glyph_width;
            let height = size.1 as f64 / self.font_height;

            self.dimensions = (width.floor() as u16, height.floor() as u16);
            self.buffers[1 - self.current].clear();
            self.last_known_size = size;
        }
    }

    fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    fn update(&mut self) {
        self.autoresize((self.canvas.width(), self.canvas.height()));
        let mut frame = Frame {
            cols: self.dimensions.0,
            rows: self.dimensions.1,
            buffer: &mut self.buffers[self.current],
            should_exit: false,
        };
        self.program.render(&mut frame);
        self.render();
        self.swap_buffers();
    }

    fn render(&mut self) {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];

        // TODO: Actually perform a diff here.
        if previous_buffer == current_buffer {
            return;
        }

        let fg_color = "#b7b7c0".to_string();
        let bg_color = "#1e1e22".to_string();
        let line_width = 2.0;

        self.canvas_context.set_text_align("left");
        self.canvas_context.set_text_baseline("top");

        'update_loop: for text in &current_buffer.content {
            let (cell_w, cell_h) = (self.glyph_width, self.font_height);
            let (cell_x, cell_y) = (cell_w * text.x as f64, cell_h * text.y as f64);

            let mut font = format!("{}px monospace", self.program.scale() as u16);
            self.canvas_context.clear_rect(cell_x, cell_y, cell_w, cell_h);

            let mut fg_style = if text.fg == Color::RESET {
                fg_color.clone()
            } else {
                text.fg.to_string().to_lowercase()
            };
            let mut bg_style = if text.bg == Color::RESET {
                bg_color.clone()
            } else {
                text.bg.to_string().to_lowercase()
            };
            let mut draw_line_at: Option<f64> = None;
            for m in text.modifier.iter() {
                match m {
                    TextModifier::HIDDEN => {
                        continue 'update_loop;
                    }
                    TextModifier::CROSSED_OUT => {
                        draw_line_at = Some(cell_y + (cell_h * 0.5));
                    }
                    TextModifier::UNDERLINED => {
                        draw_line_at = Some(cell_y + cell_h);
                    }
                    TextModifier::REVERSED => {
                        std::mem::swap(&mut fg_style, &mut bg_style);
                    }
                    TextModifier::BOLD => {
                        font = format!("bold {font}");
                    }
                    TextModifier::ITALIC => {
                        font = format!("italic {font}");
                    }
                    _ => {}
                }
            }
            self.canvas_context.set_font(&font);
            self.canvas_context.set_fill_style_str(&bg_style);
            self.canvas_context.fill_rect(cell_x, cell_y, cell_w, cell_h);
            self.canvas_context.set_fill_style_str(&fg_style);
            let _r = self.canvas_context.fill_text(&text.content, cell_x, cell_y);
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
        self.buffers[1 - self.current].clear();
        self.current = 1 - self.current;
    }
}



fn update_platform(platform: &WebPlatform) -> Result<(), wasm_bindgen::JsValue> {
    // Only paint and schedule if there has been no panic
    if let Some(mut runner_lock) = platform.try_lock() {
        runner_lock.update();
        drop(runner_lock);
        platform.request_animation_frame()?;
    }
    Ok(())
}

// https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-fnonce-and-closureonce-with-requestanimationframe
#[allow(unused)]
pub struct AnimationFrameRequest {
    pub id: i32,
    /// The callback given to `request_animation_frame`, stored here both to prevent it
    /// from being canceled, and from having to `.forget()` it.
    pub _closure: Closure<dyn FnMut() -> Result<(), wasm_bindgen::JsValue>>,
}

#[allow(unused)]
pub struct ResizeObserverContext {
    pub resize_observer: web_sys::ResizeObserver,
    pub closure: Closure<dyn FnMut(js_sys::Array)>,
}

#[allow(unused)]
pub struct EventHandle {
    pub target: web_sys::EventTarget,
    pub event_name: String,
    pub closure: Closure<dyn FnMut(web_sys::Event)>,
}



pub fn install_event_handlers(platform: &WebPlatform) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = platform.try_lock().unwrap().canvas().clone();

    install_hashchange(platform, &window)?;

    install_keydown(platform, &canvas)?;
    install_keyup(platform, &canvas)?;

    install_mousemove(platform, &document)?;
    install_pointerdown(platform, &canvas)?;
    install_pointerup(platform, &document)?;

    Ok(())
}

fn install_hashchange(platform: &WebPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "hashchange", |event: web_sys::HashChangeEvent, _runner| {
        let _req = format!("web::hashchange::{}", event.new_url());
        // if let Some(_new_hash) = runner.program.on_platform_request(&req) {
        // }
        event.prevent_default();
    })
}

fn install_keydown(platform: &WebPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "keydown", |event: web_sys::KeyboardEvent, runner| {
        if let Some(scancode) = scancode_from_keyboard_event(&event) {
            runner.program.on_input(Input::KeyDown(scancode));
            event.prevent_default();
        }
    })
}

fn install_keyup(platform: &WebPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "keyup", |event: web_sys::KeyboardEvent, runner| {
        if let Some(scancode) = scancode_from_keyboard_event(&event) {
            runner.program.on_input(Input::KeyUp(scancode));
            event.prevent_default();
        }
    })
}

fn install_mousemove(platform: &WebPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "mousemove", |event: web_sys::MouseEvent, runner| {
        let (x, y) = pos_from_mouse_event(&event, runner.dimensions);
        runner.program.on_input(Input::MouseMove(x, y));
        event.prevent_default();
    })
}

fn install_pointerdown(platform: &WebPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "pointerdown", |event: web_sys::PointerEvent, runner| {
        // let (x, y) = pos_from_mouse_event(&event, runner.dimensions);
        if let Some(scancode) = scancode_from_mouse_event(&event) {
            runner.program.on_input(Input::KeyDown(scancode));
            event.prevent_default();
        }
    })
}

fn install_pointerup(platform: &WebPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "pointerup", |event: web_sys::PointerEvent, runner| {
        // let (x, y) = pos_from_mouse_event(&event, runner.dimensions);
        if let Some(scancode) = scancode_from_mouse_event(&event) {
            runner.program.on_input(Input::KeyUp(scancode));
            event.prevent_default();
        }
    })
}

pub fn install_resize_observer(platform: &WebPlatform) -> Result<(), JsValue> {
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



fn pos_from_mouse_event(event: &web_sys::MouseEvent, (cols, rows): (u16, u16)) -> (u16, u16) {
    (
        event.screen_x() as u16 / rows,
        event.screen_y() as u16 / cols,
    )
}

// https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values
fn scancode_from_keyboard_event(event: &web_sys::KeyboardEvent) -> Option<Scancode> {
    match event.key().as_ref() {
        "Alt" => Some(Scancode::L_ALT),
        "CapsLock" => Some(Scancode::CAPSLOCK),
        "Control" => Some(Scancode::L_CTRL),
        // "Fn" => Some(Scancode::),
        "Shift" => Some(Scancode::L_SHIFT),
        // "Super" => Some(Scancode::SUPER),
        "Enter" => Some(Scancode::ENTER),
        "Tab" => Some(Scancode::TAB),
        " " => Some(Scancode::SPACE),
        "ArrowDown" => Some(Scancode::DOWN),
        "ArrowLeft" => Some(Scancode::LEFT),
        "ArrowRight" => Some(Scancode::RIGHT),
        "ArrowUp" => Some(Scancode::UP),
        "End" => Some(Scancode::END),
        "Home" => Some(Scancode::HOME),
        "PageDown" => Some(Scancode::PAGEDOWN),
        "PageUp" => Some(Scancode::PAGEUP),
        "Backspace" => Some(Scancode::BACKSPACE),
        "Delete" => Some(Scancode::DELETE),
        "Insert" => Some(Scancode::INSERT),
        // "ContextMenu" => Some(Scancode::MENU), // The one next to R_CTRL.
        "Escape" => Some(Scancode::ESC),
        "F1" => Some(Scancode::F1),
        "F2" => Some(Scancode::F2),
        "F3" => Some(Scancode::F3),
        "F4" => Some(Scancode::F4),
        "F5" => Some(Scancode::F5),
        "F6" => Some(Scancode::F6),
        "F7" => Some(Scancode::F7),
        "F8" => Some(Scancode::F8),
        "F9" => Some(Scancode::F9),
        "F10" => Some(Scancode::F10),
        // "F11" => Some(Scancode::F11),
        // "F12" => Some(Scancode::F12),

        key => {
            if key.len() != 1 {
                return None;
            }
            if let Some(c) = key.chars().last() {
                Some(Scancode::from_char(c).1)
            } else {
                None
            }
        }
    }
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
