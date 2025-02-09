//! Native Platform
//!
//! Currently, dreg uses `glyphon` for its native implementation.



use std::sync::Arc;

use epaint_default_fonts::HACK_REGULAR;
use pollster::FutureExt;
use winit::{event::{KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::{Buffer, Frame, Input, Program, Scancode};



/// Run a dreg program inside a native desktop application.
#[derive(Default)]
pub struct NativePlatform {
    args: NativeArgs,
    program: Option<Box<dyn Program>>,
    state: Option<State>,
}

impl super::Platform for NativePlatform {
    fn run(mut self, program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        self.program = Some(Box::new(program));
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

impl winit::application::ApplicationHandler for NativePlatform {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let (width, height) = (800, 600);
        let window_attributes = Window::default_attributes()
            .with_inner_size(winit::dpi::LogicalSize::new(width as f64, height as f64))
            .with_title("Untitled");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(State::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(program) = &mut self.program else { return; };
        let Some(State {
            cell_width,
            cell_height,
            cols,
            rows,
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
            window,
        }) = &mut self.state else { return; };

        match event {
            winit::event::WindowEvent::Focused(focused) => {
                program.on_input(Input::FocusChange(focused));
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
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
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
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
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_h, v) => {
                        let scancode = if v.is_sign_positive() {
                            Scancode::SCROLLUP
                        } else {
                            Scancode::SCROLLDOWN
                        };
                        program.on_input(Input::KeyDown(scancode));
                        window.request_redraw();
                    }
                    winit::event::MouseScrollDelta::PixelDelta(_pos) => {}
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let col = (position.x as f32 / *cell_width).floor() as u16;
                let row = (position.y as f32 / *cell_height).floor() as u16;
                program.on_input(Input::MouseMove(col, row));
                window.request_redraw();
            }
            winit::event::WindowEvent::Resized(size) => {
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(&device, &surface_config);
                *cols = ((size.width as f32 / *cell_width).floor() as u16).saturating_sub(1);
                *rows = ((size.height as f32 / *cell_height).floor() as u16).saturating_sub(1);

                window.request_redraw();
            }
            winit::event::WindowEvent::RedrawRequested => {
                viewport.update(
                    &queue,
                    glyphon::Resolution {
                        width: surface_config.width,
                        height: surface_config.height,
                    },
                );

                text_renderer.prepare(
                    device,
                    queue,
                    font_system,
                    atlas,
                    viewport,
                    [glyphon::TextArea {
                        buffer: text_buffer,
                        left: 10.0,
                        top: 10.0,
                        scale: 1.0,
                        bounds: glyphon::TextBounds {
                            left: 0,
                            top: 0,
                            right: 600,
                            bottom: 160,
                        },
                        default_color: glyphon::Color::rgb(255, 255, 255),
                        custom_glyphs: &[],
                    }],
                    swash_cache,
                ).unwrap();

                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { label: None }
                );

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    text_renderer.render(&atlas, &viewport, &mut pass).unwrap();
                }

                queue.submit(Some(encoder.finish()));
                frame.present();

                atlas.trim();
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

impl NativePlatform {
    pub fn with_args(args: NativeArgs) -> Self {
        Self { args, program: None, state: None }
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



struct State {
    cell_width: f32,
    cell_height: f32,
    cols: u16,
    rows: u16,

    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,

    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,
    text_buffer: glyphon::Buffer,

    window: Arc<Window>,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        let scale_factor = window.scale_factor();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on() // pollster
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .block_on() // pollster
            .unwrap();

        let surface = instance
            .create_surface(window.clone())
            .expect("Create surface");
        let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let mut font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut atlas = glyphon::TextAtlas::new(&device, &queue, &cache, swapchain_format);
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &device,
            wgpu::MultisampleState::default(),
            None,
        );
        let mut text_buffer = glyphon::Buffer::new(
            &mut font_system,
            glyphon::Metrics::new(30.0, 42.0),
        );

        let physical_width = (physical_size.width as f64 * scale_factor) as f32;
        let physical_height = (physical_size.height as f64 * scale_factor) as f32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(&mut font_system, "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z", glyphon::Attrs::new().family(glyphon::Family::Monospace), glyphon::Shaping::Advanced);
        text_buffer.shape_until_scroll(&mut font_system, false);

        Self {
            cell_width: 1.0,
            cell_height: 1.0,
            cols: 1,
            rows: 1,
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
            text_buffer,
            window,
        }
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
        // MouseButton::Back => Scancode::MOUSE_BACK,
        // MouseButton::Forward => Scancode::MOUSE_FORWARD,
        _ => { return None; }
    })
}
