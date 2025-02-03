//! Native Platform
//!
//! Currently, dreg uses winit and softbuffer for its native implementation.



use std::{num::NonZeroU32, rc::Rc};

use ab_glyph::{Font, ScaleFont};
use epaint_default_fonts::HACK_REGULAR;
use winit::{event::{KeyEvent, MouseButton, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

use crate::{Buffer, Frame, Input, Program, Scancode};



/// Run a dreg program inside a native desktop application.
pub struct NativePlatform;

impl super::Platform for NativePlatform {
    // TODO: Something like "run_with_args" for window properties and such.
    fn run(self, mut program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let window = Rc::new(WindowBuilder::new().build(&event_loop)?);

        let context = softbuffer::Context::new(window.clone())?;
        let mut surface = softbuffer::Surface::new(&context, window.clone())?;

        let scale = program.scale();
        let font_ref = ab_glyph::FontRef::try_from_slice(HACK_REGULAR)?;
        let font = font_ref.as_scaled(scale);
        let fullsize_glyph_id = font.glyph_id(' ');
        let cell_width = font.h_advance(fullsize_glyph_id)
            + font.h_side_bearing(fullsize_glyph_id);
        let cell_height = font.height() + font.line_gap();

        let mut width = 1.0;
        let mut height = 1.0;
        let mut cols = 1;
        let mut rows = 1;

        event_loop.run(|event, target| {
            target.set_control_flow(winit::event_loop::ControlFlow::Poll);

            program.update();

            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Focused(focused) => {
                        program.on_input(Input::FocusChange(focused));
                        window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        let KeyEvent { physical_key, state, .. } = event;
                        if let Some(scancode) = {
                            match physical_key {
                                PhysicalKey::Code(keycode) => {
                                    keycode_to_scancode(keycode)
                                }
                                PhysicalKey::Unidentified(_) => None,
                            }
                        } {
                            if state.is_pressed() {
                                program.on_input(Input::KeyDown(scancode));
                                window.request_redraw();
                            } else {
                                program.on_input(Input::KeyUp(scancode));
                                window.request_redraw();
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(scancode) = mouse_button_to_scancode(button) {
                            if state.is_pressed() {
                                program.on_input(Input::KeyDown(scancode));
                                window.request_redraw();
                            } else {
                                program.on_input(Input::KeyUp(scancode));
                                window.request_redraw();
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let col = (position.x as f32 / cell_width).floor() as u16;
                        let row = (position.y as f32 / cell_height).floor() as u16;
                        program.on_input(Input::MouseMove(col, row));
                        window.request_redraw();
                    }
                    WindowEvent::Resized(size) => {
                        width = size.width as f32;
                        height = size.height as f32;
                        cols = ((width / cell_width).floor() as u16).saturating_sub(1);
                        rows = ((height / cell_height).floor() as u16).saturating_sub(1);
                        // println!("W: {}, H: {}, C: {}, R: {}", width, height, cols, rows);
                        let (new_width, new_height) = (
                            NonZeroU32::new(size.width),
                            NonZeroU32::new(size.height),
                        );
                        surface.resize(new_width.unwrap(), new_height.unwrap()).unwrap();
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        let size = window.inner_size();
                        width = size.width as f32;
                        height = size.height as f32;
                        let (new_width, new_height) = (
                            NonZeroU32::new(size.width),
                            NonZeroU32::new(size.height),
                        );
                        surface.resize(new_width.unwrap(), new_height.unwrap()).unwrap();

                        let mut surface_buffer = surface.buffer_mut().unwrap();
                        surface_buffer.fill(program.clear_color().as_rgb_u32());

                        let mut buffer = Buffer { content: vec![] };
                        let mut frame = Frame {
                            cols,
                            rows,
                            buffer: &mut buffer,
                        };

                        program.render(&mut frame);

                        // TODO: This needs optimization.
                        for text in &buffer.content {
                            let first_col = text.x;
                            let mut col = first_col;
                            let mut row = text.y;
                            for ch in text.content.chars() {
                                if ch.is_ascii_whitespace() {
                                    match ch {
                                        '\n' => {
                                            col = first_col;
                                            row += 1;
                                            continue;
                                        }
                                        ' ' => {
                                            col += 1;
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }

                                if col > cols || row > rows {
                                    continue;
                                }

                                let x_pos = cell_width * col as f32;
                                let y_pos = cell_height * row as f32;

                                let glyph_id = font.glyph_id(ch);
                                let glyph = glyph_id.with_scale_and_position(
                                    scale,
                                    ab_glyph::point(x_pos, y_pos),
                                );

                                if let Some(outline) = font.outline_glyph(glyph) {
                                    let real_x_pos = outline.px_bounds().min.x;
                                    let real_y_pos = outline.px_bounds().min.y;
                                    outline.draw(|x, y, c| {
                                        if c > 0.1 {
                                            surface_buffer[(
                                                (y as f32 + real_y_pos + cell_height) * width
                                                + (x as f32 + real_x_pos)
                                            ) as usize] = text.fg.gamma_multiply(c).as_rgb_u32();
                                        }
                                    });
                                }
                                col += 1;
                            }
                        }

                        surface_buffer.present().unwrap();
                    }
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _ => {} // Ignore all other window events.
                }
                _ => {} // Ignore all other winit events.
            }
        })?;

        Ok(())
    }
}



// TODO: Finish this.
fn keycode_to_scancode(keycode: KeyCode) -> Option<Scancode> {
    Some( match keycode {
        KeyCode::ArrowLeft => Scancode::LEFT,
        KeyCode::ArrowRight => Scancode::RIGHT,
        KeyCode::ArrowUp => Scancode::UP,
        KeyCode::ArrowDown => Scancode::DOWN,

        KeyCode::Minus => Scancode::MINUS,
        KeyCode::Equal => Scancode::EQUAL,
        KeyCode::Enter => Scancode::ENTER,
        KeyCode::Home => Scancode::HOME,
        KeyCode::End => Scancode::END,
        KeyCode::Escape => Scancode::ESC,

        KeyCode::KeyA => Scancode::A,
        KeyCode::KeyB => Scancode::B,
        KeyCode::KeyC => Scancode::C,
        KeyCode::KeyD => Scancode::D,
        KeyCode::KeyE => Scancode::E,
        KeyCode::KeyF => Scancode::F,
        KeyCode::KeyG => Scancode::G,
        KeyCode::KeyH => Scancode::H,
        KeyCode::KeyI => Scancode::I,
        KeyCode::KeyJ => Scancode::J,
        KeyCode::KeyK => Scancode::K,
        KeyCode::KeyL => Scancode::L,
        KeyCode::KeyM => Scancode::M,
        KeyCode::KeyN => Scancode::N,
        KeyCode::KeyO => Scancode::O,
        KeyCode::KeyP => Scancode::P,
        KeyCode::KeyQ => Scancode::Q,
        KeyCode::KeyR => Scancode::R,
        KeyCode::KeyS => Scancode::S,
        KeyCode::KeyT => Scancode::T,
        KeyCode::KeyU => Scancode::U,
        KeyCode::KeyV => Scancode::V,
        KeyCode::KeyW => Scancode::W,
        KeyCode::KeyX => Scancode::X,
        KeyCode::KeyY => Scancode::Y,
        KeyCode::KeyZ => Scancode::Z,

        KeyCode::F1 => Scancode::F1,
        KeyCode::F2 => Scancode::F2,
        KeyCode::F3 => Scancode::F3,
        KeyCode::F4 => Scancode::F4,
        KeyCode::F5 => Scancode::F5,
        KeyCode::F6 => Scancode::F6,
        KeyCode::F7 => Scancode::F7,
        KeyCode::F8 => Scancode::F8,
        KeyCode::F9 => Scancode::F9,
        KeyCode::F10 => Scancode::F10,

        _ => { return None; }
    })
}

// TODO: Finish this.
fn mouse_button_to_scancode(button: MouseButton) -> Option<Scancode> {
    Some(match button {
        MouseButton::Left => Scancode::LMB,
        MouseButton::Right => Scancode::RMB,
        MouseButton::Middle => Scancode::MMB,
        MouseButton::Back => Scancode::MOUSE_BACK,
        MouseButton::Forward => Scancode::MOUSE_FORWARD,
        _ => { return None; }
    })
}
