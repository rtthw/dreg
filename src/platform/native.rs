//! Native Platform
//!
//! Currently, dreg uses `glyphon` for its native implementation.



use std::{hash::Hash as _, sync::Arc};

use pollster::FutureExt;
use winit::{event::{KeyEvent, MouseButton}, keyboard::{KeyCode, PhysicalKey}, window::Window};

use crate::{Buffer, Frame, Input, Program, Scancode, Text, TextModifier};



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
        let (width, height) = self.args.size;
        let window_attributes = Window::default_attributes()
            .with_inner_size(winit::dpi::LogicalSize::new(width as f64, height as f64))
            .with_resizable(self.args.resizable)
            .with_title(&self.args.title);
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
            cache,
            scale,
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
            window,
        }) = &mut self.state else { return; };

        match event {
            winit::event::WindowEvent::Focused(focused) => {
                program.input(Input::FocusChange(focused));
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
                        program.input(Input::KeyDown(scancode));
                        window.request_redraw();
                    } else {
                        program.input(Input::KeyUp(scancode));
                        window.request_redraw();
                    }
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                if let Some(scancode) = mouse_button_to_scancode(button) {
                    if state.is_pressed() {
                        program.input(Input::KeyDown(scancode));
                        window.request_redraw();
                    } else {
                        program.input(Input::KeyUp(scancode));
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
                        program.input(Input::KeyDown(scancode));
                        window.request_redraw();
                    }
                    winit::event::MouseScrollDelta::PixelDelta(_pos) => {}
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let col = (position.x as f32 / *cell_width).floor() as u16;
                let row = (position.y as f32 / *cell_height).floor() as u16;
                program.input(Input::MouseMove(col, row));
                window.request_redraw();
            }
            winit::event::WindowEvent::Resized(size) => {
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(&device, &surface_config);

                viewport.update(
                    &queue,
                    glyphon::Resolution {
                        width: surface_config.width,
                        height: surface_config.height,
                    },
                );

                let metrics = glyphon::Metrics::relative(*scale, 1.15);
                *cell_height = metrics.line_height;
                let mut measure_buf = glyphon::Buffer::new(font_system, metrics);
                measure_buf.set_text(
                    font_system,
                    " ",
                    &glyphon::Attrs::new().family(glyphon::Family::Monospace),
                    glyphon::Shaping::Advanced,
                );
                if let Some(layout) = measure_buf.layout_runs().next() {
                    *cell_width = layout.glyphs[0].w;
                }

                *cols = ((size.width as f32 / *cell_width).floor() as u16).saturating_sub(1);
                *rows = ((size.height as f32 / *cell_height).floor() as u16).saturating_sub(1);

                window.request_redraw();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let mut buffer = Buffer { content: vec![] };
                let mut frame = Frame {
                    cols: *cols,
                    rows: *rows,
                    buffer: &mut buffer,
                    should_exit: false,
                };

                program.render(&mut frame);

                // let start_time = std::time::Instant::now();

                let keys = buffer.content.iter()
                    .map(|t| {
                        let key = CacheKey::from((t, *scale));
                        let (hash, _entry) = cache.allocate(font_system, key);

                        hash
                    })
                    .collect::<Vec<_>>();

                // let prepare_time = std::time::Instant::now()
                //     .duration_since(start_time)
                //     .as_nanos();

                // if prepare_time >= 100_000 {
                //     println!("Prepare took {prepare_time}ns");
                // }

                let text_areas = buffer.content.iter().zip(keys.iter()).map(|(t, key)| {
                    let entry = cache.get(&key).unwrap();
                    let x_pos = *cell_width * t.x as f32;
                    let y_pos = *cell_height * t.y as f32;

                    glyphon::TextArea {
                        buffer: &entry.buffer,
                        left: x_pos,
                        top: y_pos,
                        scale: 1.0,
                        bounds: glyphon::TextBounds::default(),
                        default_color: glyphon::Color::rgba(
                            t.fg.r(),
                            t.fg.g(),
                            t.fg.b(),
                            t.fg.0[0],
                        ),
                        custom_glyphs: &[],
                    }
                });

                text_renderer.prepare(
                    device,
                    queue,
                    font_system,
                    atlas,
                    viewport,
                    text_areas,
                    swash_cache,
                ).unwrap();

                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor { label: None }
                );

                let clear_color = program.clear_color().as_3f32();
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: clear_color[0].into(),
                                    g: clear_color[1].into(),
                                    b: clear_color[2].into(),
                                    a: 1.0,
                                }),
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
                cache.trim();
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
    cache: Cache,

    scale: f32,
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

    window: Arc<Window>,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on() // pollster
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
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

        let font_system = glyphon::FontSystem::new();
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

        Self {
            cache: Cache::default(),
            scale: 20.0,
            cell_width: 10.0,
            cell_height: 12.0,
            cols: 80,
            rows: 24,
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            text_renderer,
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



#[derive(Clone, Copy, Debug)]
struct CacheKey<'a> {
    content: &'a str,
    modifier: TextModifier,
    size: f32,
}

impl CacheKey<'_> {
    fn hash<H: core::hash::Hasher>(self, mut hasher: H) -> u64 {
        self.content.hash(&mut hasher);
        self.modifier.hash(&mut hasher);
        self.size.to_bits().hash(&mut hasher);

        hasher.finish()
    }
}

impl<'a> From<(&'a Text, f32)> for CacheKey<'a> {
    fn from((value, size): (&'a Text, f32)) -> Self {
        Self {
            content: &value.content,
            modifier: value.modifier,
            size,
        }
    }
}

struct CacheEntry {
    buffer: glyphon::Buffer,
}

#[derive(Default)]
struct Cache {
    entries: rustc_hash::FxHashMap<u64, CacheEntry>,
    aliases: rustc_hash::FxHashMap<u64, u64>,
    recently_used: rustc_hash::FxHashSet<u64>,
}

impl Cache {
    fn get(&self, key: &u64) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    fn allocate<'a>(
        &mut self,
        font_system: &mut glyphon::FontSystem,
        key: CacheKey<'a>,
    ) -> (u64, &mut CacheEntry)
    {
        let hash = key.hash(rustc_hash::FxHasher::default());
        if let Some(hash) = self.aliases.get(&hash) {
            let _ = self.recently_used.insert(*hash);

            return (*hash, self.entries.get_mut(hash).unwrap());
        }

        if let std::collections::hash_map::Entry::Vacant(entry) = self.entries.entry(hash) {
            let metrics = glyphon::Metrics::relative(key.size, 1.15);
            let mut buffer = glyphon::Buffer::new(font_system, metrics);

            // buffer.set_size(
            //     font_system,
            //     Some(key.bounds.x),
            //     Some(key.bounds.y.max(key.line_height)),
            // );
            buffer.set_text(
                font_system,
                key.content,
                &glyphon::Attrs::new()
                    .family(glyphon::Family::Monospace)
                    .style(if key.modifier.contains(TextModifier::ITALIC) {
                        glyphon::Style::Italic
                    } else {
                        glyphon::Style::Normal
                    }),
                glyphon::Shaping::Advanced,
            );

            let _ = entry.insert(CacheEntry {
                buffer,
            });
        }

        let _ = self.recently_used.insert(hash);

        (hash, self.entries.get_mut(&hash).unwrap())
    }

    fn trim(&mut self) {
        self.entries.retain(|key, _| self.recently_used.contains(key));
        self.aliases.retain(|_, value| self.recently_used.contains(value));

        self.recently_used.clear();
    }
}
