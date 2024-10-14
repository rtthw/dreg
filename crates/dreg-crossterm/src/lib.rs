//! Crossterm Platform


use std::io::stdout;

use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyboardEnhancementFlags, ModifierKeyCode, MouseEvent, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand as _,
};
use dreg_core::prelude::*;

pub mod prelude {
    pub extern crate crossterm;
    pub use crate::CrosstermPlatform;
}

pub struct CrosstermPlatform {
    ctx: Context,

    /// Holds the results of the current and previous draw calls. The two are compared at the end
    /// of each draw pass to output the necessary updates to the terminal
    buffers: [Buffer; 2],
    /// Index of the current buffer in the previous array
    current: usize,
}

impl Platform for CrosstermPlatform {
    fn run(mut self, program: impl Program) -> Result<()> {
        bind_terminal()?;
        while !program.should_exit() {
            if crossterm::event::poll(std::time::Duration::from_millis(31))? {
                let event = crossterm::event::read()?;
                handle_crossterm_event(&mut self.ctx, event);
            }
        }
        release_terminal()?;

        Ok(())
    }
}

impl CrosstermPlatform {
    pub fn new(area: Rect) -> Self {
        Self {
            ctx: Context::default(),
            buffers: [Buffer::empty(area), Buffer::empty(area)],
            current: 0,
        }
    }

    pub fn size(&self) -> std::io::Result<Rect> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Rect::new(0, 0, width, height))
    }
}


fn bind_terminal() -> Result<()> {
    let mut writer = stdout();
    enable_raw_mode()?;
    writer.execute(EnableMouseCapture)?;
    writer.execute(EnterAlternateScreen)?;
    writer.execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    ))?;
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        release_terminal().unwrap();
        original_hook(panic);
    }));

    Ok(())
}

fn release_terminal() -> Result<()> {
    let mut writer = stdout();
    disable_raw_mode()?;
    writer.execute(DisableMouseCapture)?;
    writer.execute(LeaveAlternateScreen)?;
    writer.execute(PopKeyboardEnhancementFlags)?;
    writer.execute(Show)?;

    Ok(())
}



fn handle_crossterm_event(ctx: &mut Context, event: Event) {
    match event {
        Event::Key(KeyEvent { code, modifiers, kind, state }) => {
            let scancodes = translate_keycode(code);
            match kind {
                KeyEventKind::Press => {
                    for scancode in scancodes {
                        ctx.handle_key_down(scancode);
                    }
                }
                KeyEventKind::Release => {
                    for scancode in scancodes {
                        ctx.handle_key_up(&scancode);
                    }
                }
                _ => {} // Do nothing.
            }
        }
        Event::Mouse(MouseEvent { kind, column, row, modifiers }) => {

        }
        Event::FocusGained => {
            ctx.handle_input(Input::FocusChange(true));
        }
        Event::FocusLost => {
            ctx.handle_input(Input::FocusChange(false));
        }
        Event::Resize(new_cols, new_rows) => {
            ctx.handle_input(Input::Resize(new_cols, new_rows));
        }
        _ => {}
    }
}

fn translate_keycode(code: KeyCode) -> Vec<Scancode> {
    // All of `crossterm`'s keycodes translate to 2 or less scancodes.
    let mut scancodes = Vec::with_capacity(2);
    match code {
        KeyCode::Char(c) => {
            let (modifier, scancode) = Scancode::from_char(c);
            if let Some(mod_code) = modifier {
                scancodes.push(mod_code);
            }
            scancodes.push(scancode);
        }
        KeyCode::F(n) => {
            scancodes.push(match n {
                1 => Scancode::F1,
                2 => Scancode::F2,
                3 => Scancode::F3,
                4 => Scancode::F4,
                5 => Scancode::F5,
                6 => Scancode::F6,
                7 => Scancode::F7,
                8 => Scancode::F8,
                9 => Scancode::F9,
                10 => Scancode::F10,
                _ => Scancode::NULL,
            })
        }
        KeyCode::Modifier(mod_keycode) => match mod_keycode {
            ModifierKeyCode::LeftShift => { scancodes.push(Scancode::L_SHIFT); },
            ModifierKeyCode::LeftAlt => { scancodes.push(Scancode::L_ALT); },
            ModifierKeyCode::LeftControl => { scancodes.push(Scancode::L_CTRL); },

            ModifierKeyCode::RightShift => { scancodes.push(Scancode::R_SHIFT); },
            ModifierKeyCode::RightAlt => { scancodes.push(Scancode::R_ALT); },
            ModifierKeyCode::RightControl => { scancodes.push(Scancode::R_CTRL); },

            _ => {} // TODO: Handle other modifiers.
        }

        KeyCode::Esc => { scancodes.push(Scancode::ESC); },
        KeyCode::Backspace => { scancodes.push(Scancode::BACKSPACE); }
        KeyCode::Tab => { scancodes.push(Scancode::TAB); },
        KeyCode::BackTab => { scancodes.extend([Scancode::L_SHIFT, Scancode::TAB]); }
        KeyCode::Enter => { scancodes.push(Scancode::ENTER); },
        KeyCode::Delete => { scancodes.push(Scancode::DELETE); },
        KeyCode::Insert => { scancodes.push(Scancode::INSERT); },
        KeyCode::CapsLock => { scancodes.push(Scancode::CAPSLOCK); },

        KeyCode::Left => { scancodes.push(Scancode::LEFT); },
        KeyCode::Right => { scancodes.push(Scancode::RIGHT); },
        KeyCode::Up => { scancodes.push(Scancode::UP); },
        KeyCode::Down => { scancodes.push(Scancode::DOWN); },

        KeyCode::Home => { scancodes.push(Scancode::HOME); },
        KeyCode::End => { scancodes.push(Scancode::END); },
        KeyCode::PageUp => { scancodes.push(Scancode::PAGEDOWN); },
        KeyCode::PageDown => { scancodes.push(Scancode::PAGEUP); },

        _ => {}
    }

    scancodes
}
