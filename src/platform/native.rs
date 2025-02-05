//! Native Platform
//!
//! Currently, dreg uses glutin for its native implementation.



use epaint_default_fonts::HACK_REGULAR;
use glow::HasContext as _;
use glow_glyph::ab_glyph::{self, Font as _, ScaleFont as _};
use glutin::event::{KeyboardInput, MouseButton, WindowEvent};

use crate::{Buffer, Frame, Input, Program, Scancode};



/// Run a dreg program inside a native desktop application.
#[derive(Default)]
pub struct NativePlatform {
    args: NativeArgs,
}

impl super::Platform for NativePlatform {
    fn run(self, mut program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title(self.args.title)
            .with_inner_size(glutin::dpi::LogicalSize::new(self.args.size.0, self.args.size.1))
            .with_resizable(self.args.resizable);
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)?;
        let context = unsafe { context.make_current().expect("failed to grab OpenGL context") };
        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                context.get_proc_address(s) as *const _
            })
        };

        let font_arc = ab_glyph::FontArc::try_from_slice(HACK_REGULAR)?;
        let mut glyph_brush = glow_glyph::GlyphBrushBuilder::using_font(font_arc).build(&gl);

        context.window().request_redraw();

        let clear_color = program.clear_color().as_rgba_f32();
        unsafe {
            gl.enable(glow::FRAMEBUFFER_SRGB); // Enable automatic conversion to/from sRGB.
            gl.enable(glow::BLEND); // Enable alpha blending.
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA); // Set blend function.
            gl.clear_color(clear_color[0], clear_color[1], clear_color[2], clear_color[3]);
        }

        let size = context.window().inner_size();
        let mut width = size.width as f32;
        let mut height = size.height as f32;
        let mut cols = 1;
        let mut rows = 1;

        let mut cell_width = 1.0;
        let mut cell_height = 1.0;

        event_loop.run(move |event, _target, control_flow| {
            program.update();
            let scale = program.scale();

            match event {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Focused(focused) => {
                        program.on_input(Input::FocusChange(focused));
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        let KeyboardInput { scancode, state, .. } = input;
                        match state {
                            glutin::event::ElementState::Pressed => {
                                program.on_input(Input::KeyDown(Scancode(scancode as _)));
                                context.window().request_redraw();
                            }
                            glutin::event::ElementState::Released => {
                                program.on_input(Input::KeyUp(Scancode(scancode as _)));
                                context.window().request_redraw();
                            }
                        }
                        // if let Some(scancode) = {
                        //     match physical_key {
                        //         PhysicalKey::Code(keycode) => {
                        //             keycode_to_scancode(keycode)
                        //         }
                        //         PhysicalKey::Unidentified(_) => None,
                        //     }
                        // } {
                        //     if state.is_pressed() {
                        //         program.on_input(Input::KeyDown(scancode));
                        //         window.request_redraw();
                        //     } else {
                        //         program.on_input(Input::KeyUp(scancode));
                        //         window.request_redraw();
                        //     }
                        // }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if let Some(scancode) = mouse_button_to_scancode(button) {
                            match state {
                                glutin::event::ElementState::Pressed => {
                                    program.on_input(Input::KeyDown(scancode));
                                    context.window().request_redraw();
                                }
                                glutin::event::ElementState::Released => {
                                    program.on_input(Input::KeyUp(scancode));
                                    context.window().request_redraw();
                                }
                            }
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            glutin::event::MouseScrollDelta::LineDelta(_h, v) => {
                                let scancode = if v.is_sign_positive() {
                                    Scancode::SCROLLUP
                                } else {
                                    Scancode::SCROLLDOWN
                                };
                                program.on_input(Input::KeyDown(scancode));
                                context.window().request_redraw();
                            }
                            glutin::event::MouseScrollDelta::PixelDelta(_pos) => {}
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let col = (position.x as f32 / cell_width).floor() as u16;
                        let row = (position.y as f32 / cell_height).floor() as u16;
                        program.on_input(Input::MouseMove(col, row));
                        context.window().request_redraw();
                    }
                    WindowEvent::Resized(size) => {
                        width = size.width as f32;
                        height = size.height as f32;
                        cols = ((width / cell_width).floor() as u16).saturating_sub(1);
                        rows = ((height / cell_height).floor() as u16).saturating_sub(1);

                        let scale = program.scale();
                        let font = glyph_brush.fonts()[0].as_scaled(scale);
                        let fullsize_glyph_id = font.glyph_id(' ');
                        cell_width = font.h_advance(fullsize_glyph_id)
                            + font.h_side_bearing(fullsize_glyph_id);
                        cell_height = font.height() + font.line_gap();
                        // println!("W: {}, H: {}, C: {}, R: {}", width, height, cols, rows);
                        context.resize(size);
                        unsafe { gl.viewport( 0, 0, size.width as _, size.height as _); }
                        context.window().request_redraw();
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => {
                        *control_flow = glutin::event_loop::ControlFlow::Poll;
                    }
                }
                glutin::event::Event::RedrawRequested(_for_window_id) => {
                    unsafe { gl.clear(glow::COLOR_BUFFER_BIT) }

                    let mut buffer = Buffer { content: vec![] };
                    let mut frame = Frame {
                        cols,
                        rows,
                        buffer: &mut buffer,
                        should_exit: false,
                    };

                    program.render(&mut frame);

                    // TODO: This needs optimization.
                    for text in &buffer.content {
                        let x_pos = cell_width * text.x as f32;
                        let y_pos = cell_height * text.y as f32;

                        glyph_brush.queue(glow_glyph::Section {
                            screen_position: (x_pos, y_pos),
                            bounds: (width, height),
                            text: vec![glow_glyph::Text::default()
                                .with_text(text.content.as_str())
                                .with_color(text.fg.as_rgba_f32())
                                .with_scale(scale)],
                            ..glow_glyph::Section::default()
                        });
                    }

                    glyph_brush.draw_queued(&gl, width as u32, height as u32).unwrap();
                    context.swap_buffers().unwrap();
                }
                _ => {
                    *control_flow = glutin::event_loop::ControlFlow::Poll;
                }
            }
        });
    }
}

impl NativePlatform {
    pub fn with_args(args: NativeArgs) -> Self {
        Self { args }
    }
}



/// Arguments provided to the native platform when it runs.
pub struct NativeArgs {
    /// Window title.
    ///
    /// Defaults to `"Untitled"`.
    pub title: String,
    /// Initial window size, in logical (pre-scaled) pixels.
    ///
    /// Defaults to `(1280, 720)`.
    pub size: (u16, u16),
    /// Whether the window is resizable.
    ///
    /// Defaults to `true`.
    pub resizable: bool,
}

impl Default for NativeArgs {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            size: (1280, 720),
            resizable: true,
        }
    }
}



// TODO: Finish this.
// fn keycode_to_scancode(keycode: KeyCode) -> Option<Scancode> {
//     Some( match keycode {
//         KeyCode::ArrowLeft => Scancode::LEFT,
//         KeyCode::ArrowRight => Scancode::RIGHT,
//         KeyCode::ArrowUp => Scancode::UP,
//         KeyCode::ArrowDown => Scancode::DOWN,

//         KeyCode::Minus => Scancode::MINUS,
//         KeyCode::Equal => Scancode::EQUAL,
//         KeyCode::Enter => Scancode::ENTER,
//         KeyCode::Home => Scancode::HOME,
//         KeyCode::End => Scancode::END,
//         KeyCode::Escape => Scancode::ESC,

//         KeyCode::KeyA => Scancode::A,
//         KeyCode::KeyB => Scancode::B,
//         KeyCode::KeyC => Scancode::C,
//         KeyCode::KeyD => Scancode::D,
//         KeyCode::KeyE => Scancode::E,
//         KeyCode::KeyF => Scancode::F,
//         KeyCode::KeyG => Scancode::G,
//         KeyCode::KeyH => Scancode::H,
//         KeyCode::KeyI => Scancode::I,
//         KeyCode::KeyJ => Scancode::J,
//         KeyCode::KeyK => Scancode::K,
//         KeyCode::KeyL => Scancode::L,
//         KeyCode::KeyM => Scancode::M,
//         KeyCode::KeyN => Scancode::N,
//         KeyCode::KeyO => Scancode::O,
//         KeyCode::KeyP => Scancode::P,
//         KeyCode::KeyQ => Scancode::Q,
//         KeyCode::KeyR => Scancode::R,
//         KeyCode::KeyS => Scancode::S,
//         KeyCode::KeyT => Scancode::T,
//         KeyCode::KeyU => Scancode::U,
//         KeyCode::KeyV => Scancode::V,
//         KeyCode::KeyW => Scancode::W,
//         KeyCode::KeyX => Scancode::X,
//         KeyCode::KeyY => Scancode::Y,
//         KeyCode::KeyZ => Scancode::Z,

//         KeyCode::F1 => Scancode::F1,
//         KeyCode::F2 => Scancode::F2,
//         KeyCode::F3 => Scancode::F3,
//         KeyCode::F4 => Scancode::F4,
//         KeyCode::F5 => Scancode::F5,
//         KeyCode::F6 => Scancode::F6,
//         KeyCode::F7 => Scancode::F7,
//         KeyCode::F8 => Scancode::F8,
//         KeyCode::F9 => Scancode::F9,
//         KeyCode::F10 => Scancode::F10,

//         _ => { return None; }
//     })
// }

// TODO: Finish this.
fn mouse_button_to_scancode(button: MouseButton) -> Option<Scancode> {
    Some(match button {
        MouseButton::Left => Scancode::LMB,
        MouseButton::Right => Scancode::RMB,
        MouseButton::Middle => Scancode::MMB,
        // MouseButton::Back => Scancode::MOUSE_BACK,
        // MouseButton::Forward => Scancode::MOUSE_FORWARD,
        _ => { return None; }
    })
}
