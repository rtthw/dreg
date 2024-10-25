

use dreg_core::prelude::*;
use wasm_bindgen::{closure::Closure, JsCast as _, JsValue};
use web_sys::{js_sys, EventTarget};

use crate::WasmPlatform;



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



pub fn install_event_handlers(platform: &WasmPlatform) -> Result<(), JsValue> {
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

fn install_hashchange(platform: &WasmPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "hashchange", |event: web_sys::HashChangeEvent, runner| {
        let req = format!("web::hashchange::{}", event.new_url());
        if let Some(_new_hash) = runner.program.on_platform_request(&req) {

        }
        event.prevent_default();
    })
}

fn install_keydown(platform: &WasmPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "keydown", |event: web_sys::KeyboardEvent, runner| {
        if let Some(scancode) = scancode_from_keyboard_event(&event) {
            runner.program.on_input(Input::KeyDown(scancode));
            event.prevent_default();
        }
    })
}

fn install_keyup(platform: &WasmPlatform, target: &EventTarget) -> Result<(), JsValue> {
    platform.add_event_listener(target, "keyup", |event: web_sys::KeyboardEvent, runner| {
        if let Some(scancode) = scancode_from_keyboard_event(&event) {
            runner.program.on_input(Input::KeyUp(scancode));
            event.prevent_default();
        }
    })
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

pub fn install_resize_observer(platform: &WasmPlatform) -> Result<(), JsValue> {
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
