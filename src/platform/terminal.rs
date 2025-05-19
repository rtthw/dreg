//! Terminal Platform
//!
//! Currently, dreg uses crossterm for its terminal implementation.



use std::io::Write as _;

use crossterm::{
    event::{
        KeyCode, KeyEvent, KeyEventKind, KeyModifiers, ModifierKeyCode,
        MouseButton, MouseEvent, MouseEventKind,
    }, queue, style::{Attribute, Color as CtColor, SetAttribute}, ExecutableCommand as _
};

use crate::{Buffer, Color, Frame, Input, Program, Scancode, TextModifier};



/// Run a dreg program inside a terminal emulator.
pub struct Terminal {
    /// Holds the results of the current and previous render calls. The two are compared at the end
    /// of each render pass to output only the necessary updates to the terminal.
    buffers: [Buffer; 2],
    /// The index of the current buffer in the previous array.
    current: usize,
}

impl super::Platform for Terminal {
    fn run(mut self, mut program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        bind_terminal()?;

        'main_loop: loop {
            if crossterm::event::poll(std::time::Duration::from_millis(31))? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
                        let mut scancodes = vec![];
                        if modifiers != KeyModifiers::NONE {
                            for m in modifiers.iter() {
                                match m {
                                    KeyModifiers::SHIFT => scancodes.push(Scancode::L_SHIFT),
                                    KeyModifiers::ALT => scancodes.push(Scancode::L_ALT),
                                    KeyModifiers::CONTROL => scancodes.push(Scancode::L_CTRL),
                                    _ => {} // TODO: Handle other modifiers.
                                }
                            }
                        }
                        scancodes.extend(keycode_to_scancode(code));
                        match kind {
                            KeyEventKind::Press | KeyEventKind::Repeat => {
                                for scancode in scancodes {
                                    program.input(Input::KeyDown(scancode));
                                }
                            }
                            KeyEventKind::Release => {
                                for scancode in scancodes {
                                    program.input(Input::KeyUp(scancode));
                                }
                            }
                        }
                    }
                    crossterm::event::Event::Mouse(MouseEvent { kind, column, row, .. }) => {
                        match kind {
                            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                                program.input(Input::MouseMove(column, row));
                            }
                            MouseEventKind::Down(btn) => {
                                let code = match btn {
                                    MouseButton::Left => Scancode::LMB,
                                    MouseButton::Right => Scancode::RMB,
                                    MouseButton::Middle => Scancode::MMB,
                                };
                                program.input(Input::KeyDown(code));
                            }
                            MouseEventKind::Up(btn) => {
                                let code = match btn {
                                    MouseButton::Left => Scancode::LMB,
                                    MouseButton::Right => Scancode::RMB,
                                    MouseButton::Middle => Scancode::MMB,
                                };
                                program.input(Input::KeyUp(code));
                            }
                            // SEE: https://github.com/rtthw/dreg/issues/7
                            MouseEventKind::ScrollUp => {
                                program.input(Input::KeyDown(Scancode::SCROLLUP));
                                program.input(Input::KeyUp(Scancode::SCROLLUP));
                            }
                            MouseEventKind::ScrollDown => {
                                program.input(Input::KeyDown(Scancode::SCROLLDOWN));
                                program.input(Input::KeyUp(Scancode::SCROLLDOWN));
                            }
                            _ => {} // TODO: ScrollRight and ScrollLeft handling.
                        }
                    }
                    crossterm::event::Event::FocusGained => {
                        program.input(Input::FocusChange(true));
                    }
                    crossterm::event::Event::FocusLost => {
                        program.input(Input::FocusChange(false));
                    }
                    crossterm::event::Event::Resize(new_cols, new_rows) => {
                        program.input(Input::Resize(new_cols, new_rows));
                    }
                    _ => {}
                }
            }
            // TODO: Optimize this by storing terminal size?
            let (cols, rows) = crossterm::terminal::size()?;

            let mut frame = Frame {
                cols,
                rows,
                buffer: &mut self.buffers[self.current],
                should_exit: false,
            };

            program.render(&mut frame);

            if frame.should_exit {
                break 'main_loop;
            }

            self.flush()?;
            self.swap_buffers();
            std::io::stdout().flush()?;
        }

        release_terminal()?;

        Ok(())
    }
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            buffers: [Buffer::empty(), Buffer::empty()],
            current: 0,
        }
    }

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].clear();
        self.current = 1 - self.current;
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];

        // TODO: Actually perform a diff here.
        if previous_buffer == current_buffer {
            return Ok(());
        }

        let mut writer = std::io::stdout();

        crossterm::queue!(
            writer,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        )?;

        let mut fg = Color::RESET;
        let mut bg = Color::RESET;
        let mut modifier = TextModifier::empty();

        for text in current_buffer.content.iter() {
            if text.fg != fg || text.bg != bg {
                queue!(
                    writer,
                    crossterm::style::SetColors(crossterm::style::Colors::new(
                        color_to_crossterm_color(text.fg),
                        color_to_crossterm_color(text.bg),
                    )),
                )?;
                fg = text.fg;
                bg = text.bg;
            }
            if text.modifier != modifier {
                let diff = ModifierDiff {
                    from: modifier,
                    to: text.modifier,
                };
                diff.queue(&mut writer)?;
                modifier = text.modifier;
            }
            let mut line_num = 0;
            for line in text.content.lines() {
                queue!(writer, crossterm::cursor::MoveTo(text.x, text.y + line_num))?;
                queue!(writer, crossterm::style::Print(line))?;
                line_num += 1;
            }
        }

        crossterm::queue!(
            writer,
            crossterm::style::SetForegroundColor(CtColor::Reset),
            crossterm::style::SetBackgroundColor(CtColor::Reset),
            crossterm::style::SetAttribute(Attribute::Reset),
        )
    }
}



fn bind_terminal() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    writer.execute(crossterm::event::EnableMouseCapture)?;
    writer.execute(crossterm::event::EnableFocusChange)?;
    writer.execute(crossterm::terminal::EnterAlternateScreen)?;
    writer.execute(crossterm::event::PushKeyboardEnhancementFlags(
        crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | crossterm::event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
    ))?;
    writer.execute(crossterm::cursor::Hide)?;
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        release_terminal().unwrap();
        original_hook(panic);
    }));

    Ok(())
}

fn release_terminal() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = std::io::stdout();
    crossterm::terminal::disable_raw_mode()?;
    writer.execute(crossterm::event::DisableMouseCapture)?;
    writer.execute(crossterm::event::DisableFocusChange)?;
    writer.execute(crossterm::terminal::LeaveAlternateScreen)?;
    writer.execute(crossterm::event::PopKeyboardEnhancementFlags)?;
    writer.execute(crossterm::cursor::Show)?;

    Ok(())
}

fn keycode_to_scancode(code: KeyCode) -> Vec<Scancode> {
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
        KeyCode::PageUp => { scancodes.push(Scancode::PAGEUP); },
        KeyCode::PageDown => { scancodes.push(Scancode::PAGEDOWN); },

        _ => {}
    }

    scancodes
}

fn color_to_crossterm_color(color: Color) -> CtColor {
    match color {
        Color::RESET => CtColor::Reset,
        Color([0, 0, 1, i]) => CtColor::AnsiValue(i),
        Color([255, r, g, b]) => CtColor::Rgb { r, g, b },
        _ => unreachable!("tried to translate invalid color tag"),
    }
}

/// The `ModifierDiff` struct is used to calculate the difference between two `TextModifier`s.
/// This is useful when updating the terminal display, as it allows for more efficient updates by
/// only sending the necessary changes.
struct ModifierDiff {
    pub from: TextModifier,
    pub to: TextModifier,
}

impl ModifierDiff {
    fn queue<W: std::io::Write>(self, mut w: W) -> std::io::Result<()> {
        //use crossterm::Attribute;
        let removed = self.from - self.to;
        if removed.contains(TextModifier::REVERSED) {
            queue!(w, SetAttribute(Attribute::NoReverse))?;
        }
        if removed.contains(TextModifier::BOLD) {
            queue!(w, SetAttribute(Attribute::NormalIntensity))?;
            if self.to.contains(TextModifier::DIM) {
                queue!(w, SetAttribute(Attribute::Dim))?;
            }
        }
        if removed.contains(TextModifier::ITALIC) {
            queue!(w, SetAttribute(Attribute::NoItalic))?;
        }
        if removed.contains(TextModifier::UNDERLINED) {
            queue!(w, SetAttribute(Attribute::NoUnderline))?;
        }
        if removed.contains(TextModifier::DIM) {
            queue!(w, SetAttribute(Attribute::NormalIntensity))?;
        }
        if removed.contains(TextModifier::CROSSED_OUT) {
            queue!(w, SetAttribute(Attribute::NotCrossedOut))?;
        }
        if removed.contains(TextModifier::SLOW_BLINK) || removed.contains(TextModifier::RAPID_BLINK) {
            queue!(w, SetAttribute(Attribute::NoBlink))?;
        }

        let added = self.to - self.from;
        if added.contains(TextModifier::REVERSED) {
            queue!(w, SetAttribute(Attribute::Reverse))?;
        }
        if added.contains(TextModifier::BOLD) {
            queue!(w, SetAttribute(Attribute::Bold))?;
        }
        if added.contains(TextModifier::ITALIC) {
            queue!(w, SetAttribute(Attribute::Italic))?;
        }
        if added.contains(TextModifier::UNDERLINED) {
            queue!(w, SetAttribute(Attribute::Underlined))?;
        }
        if added.contains(TextModifier::DIM) {
            queue!(w, SetAttribute(Attribute::Dim))?;
        }
        if added.contains(TextModifier::CROSSED_OUT) {
            queue!(w, SetAttribute(Attribute::CrossedOut))?;
        }
        if added.contains(TextModifier::SLOW_BLINK) {
            queue!(w, SetAttribute(Attribute::SlowBlink))?;
        }
        if added.contains(TextModifier::RAPID_BLINK) {
            queue!(w, SetAttribute(Attribute::RapidBlink))?;
        }

        Ok(())
    }
}
