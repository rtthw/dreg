//! Native Platform
//!
//! Currently, dreg uses winit and softbuffer for its native implementation.



use std::{num::NonZeroU32, rc::Rc};

use ab_glyph::{Font, ScaleFont};
use epaint_default_fonts::HACK_REGULAR;
use winit::{event::{KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

use crate::{Buffer, Frame, Input, Program, Scancode};



pub struct NativePlatform;

impl super::Platform for NativePlatform {
    // TODO: Something like "run_with_args" for window properties and such.
    fn run(self, mut program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let window = Rc::new(WindowBuilder::new().build(&event_loop)?);

        let context = softbuffer::Context::new(window.clone())?;
        let mut surface = softbuffer::Surface::new(&context, window.clone())?;

        let font = ab_glyph::FontRef::try_from_slice(HACK_REGULAR)?;

        let mut width = 1.0;
        let mut height = 1.0;

        event_loop.run(|event, target| {
            target.set_control_flow(winit::event_loop::ControlFlow::Poll);

            match event {
                winit::event::Event::WindowEvent { event, .. } => match event {
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
                    WindowEvent::CursorMoved { position, .. } => {
                        program.on_input(Input::MouseMove(position.x as u32, position.y as u32));
                        window.request_redraw();
                    }
                    WindowEvent::Resized(size) => {
                        width = size.width as f32;
                        height = size.height as f32;
                        let (new_width, new_height) = (
                            NonZeroU32::new(size.width),
                            NonZeroU32::new(size.height),
                        );
                        surface.resize(new_width.unwrap(), new_height.unwrap()).unwrap();
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
                            width,
                            height,
                            buffer: &mut buffer,
                        };

                        program.update(&mut frame);

                        // TODO: This needs optimization.
                        for text in &buffer.content {
                            let font = font.as_scaled(text.scale);
                            let mut x_cursor = text.x as f32;
                            let y_cursor = text.y as f32;
                            for ch in text.content.chars() {
                                let glyph_id = font.glyph_id(ch);
                                let glyph = glyph_id.with_scale_and_position(
                                    text.scale,
                                    ab_glyph::point(x_cursor, y_cursor),
                                );
                                x_cursor += font.h_advance(glyph_id);
                                if let Some(outline) = font.outline_glyph(glyph) {
                                    let y_advance = outline.px_bounds().min.y;
                                    outline.draw(|x, y, c| {
                                        if c > 0.1 {
                                            surface_buffer[(
                                                (y as f32 + y_advance) * width
                                                + (x as f32 + x_cursor)
                                            ) as usize] = text.fg.gamma_multiply(c).as_rgb_u32();
                                        }
                                    });
                                }
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
        KeyCode::Equal => Scancode::EQUAL,
        KeyCode::Enter => Scancode::ENTER,
        KeyCode::End => Scancode::END,
        KeyCode::Escape => Scancode::ESC,

        KeyCode::KeyA => Scancode::A,
        KeyCode::KeyB => Scancode::B,
        KeyCode::KeyC => Scancode::C,
        KeyCode::KeyD => Scancode::D,
        KeyCode::KeyE => Scancode::E,

        _ => { return None; }
    })
}
