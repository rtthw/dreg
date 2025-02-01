//! Native Platform
//!
//! Currently, dreg uses winit and softbuffer for its native implementation.



use std::rc::Rc;

use winit::{event::{KeyEvent, WindowEvent}, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder};

use crate::{Input, Program, Scancode};



pub struct NativePlatform;

impl super::Platform for NativePlatform {
    // TODO: Something like "run_with_args" for window properties and such.
    fn run(self, mut program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new()?;
        let window = Rc::new(WindowBuilder::new().build(&event_loop)?);

        let context = softbuffer::Context::new(window.clone())?;
        let mut surface = softbuffer::Surface::new(&context, window.clone())?;

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
                                PhysicalKey::Unidentified(_) => {
                                    None
                                }
                            }
                        } {
                            if state.is_pressed() {
                                program.on_input(Input::KeyDown(scancode))
                            } else {
                                program.on_input(Input::KeyUp(scancode))
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // TODO: Actually draw the program buffer.
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
