//! Crossterm Platform


use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture,
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        ModifierKeyCode, MouseButton, MouseEvent, MouseEventKind,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    queue,
    style::{
        Attribute as CtAttribute,
        Attributes as CtAttributes,
        Color as CColor, Colors, Print, SetAttribute, SetBackgroundColor, SetColors, SetForegroundColor
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen
    },
    ExecutableCommand as _,
};
use dreg_core::prelude::*;



pub mod prelude {
    pub extern crate crossterm;
    pub use crate::{
        CrosstermPlatform,
        crossterm_attribute_to_dreg_modifier,
        crossterm_attributes_to_dreg_modifier,
        crossterm_color_to_dreg_color,
        crossterm_keycode_to_dreg_scancode,
        dreg_color_to_crossterm_color,
    };
}



/// The platform for running dreg programs through the terminal.
pub struct CrosstermPlatform {
    /// Holds the results of the current and previous render calls. The two are compared at the end
    /// of each render pass to output the necessary updates to the terminal.
    buffers: [Buffer; 2],
    /// The index of the current buffer in the previous array.
    current: usize,

    viewport_area: Rect,
    last_known_size: Rect,
}

impl Platform for CrosstermPlatform {
    fn run(mut self, mut program: impl Program) -> Result<()> {
        bind_terminal()?;
        while !program.should_exit() {
            if crossterm::event::poll(std::time::Duration::from_millis(31))? {
                let event = crossterm::event::read()?;
                handle_crossterm_event(&mut program, event);
            }

            self.autoresize()?;

            let size = self.size()?;
            let frame = Frame {
                area: size,
                buffer: &mut self.buffers[self.current],
            };

            program.update(frame);
            self.flush()?;
            self.swap_buffers();
            stdout().flush()?;
        }
        release_terminal()?;

        Ok(())
    }
}

impl CrosstermPlatform {
    /// Create a new instance of the crossterm platform.
    pub fn new() -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        let size = Rect::new(0, 0, width, height);

        Ok(Self {
            buffers: [Buffer::empty(size), Buffer::empty(size)],
            current: 0,
            viewport_area: size,
            last_known_size: size,
        })
    }

    pub fn size(&self) -> std::io::Result<Rect> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Rect::new(0, 0, width, height))
    }

    fn resize(&mut self, size: Rect) -> std::io::Result<()> {
        self.set_viewport_area(size);
        execute!(stdout(), Clear(crossterm::terminal::ClearType::All))?;

        self.last_known_size = size;
        Ok(())
    }

    fn set_viewport_area(&mut self, area: Rect) {
        self.buffers[self.current].resize(area);
        self.buffers[1 - self.current].resize(area);
        self.viewport_area = area;
    }

    /// Resizes if the terminal's size doesn't match the previous size.
    fn autoresize(&mut self) -> std::io::Result<()> {
        let size = self.size()?;
        if size != self.last_known_size {
            self.resize(size)?;
        }
        Ok(())
    }

    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let updates = previous_buffer.diff(current_buffer);
        // if let Some((col, row, _)) = updates.last() {
        //     self.last_known_cursor_pos = (*col, *row);
        // }
        let content = updates.into_iter();

        let mut writer = stdout();
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        #[cfg(feature = "underline-color")]
        let mut underline_color = Color::Reset;
        let mut modifier = Modifier::empty();
        let mut last_pos: Option<(u16, u16)> = None;
        for (x, y, cell) in content {
            // Move the cursor if the previous location was not (x - 1, y)
            if !matches!(last_pos, Some(p) if x == p.0 + 1 && y == p.1) {
                queue!(writer, MoveTo(x, y))?;
            }
            last_pos = Some((x, y));
            if cell.modifier != modifier {
                let diff = ModifierDiff {
                    from: modifier,
                    to: cell.modifier,
                };
                diff.queue(&mut writer)?;
                modifier = cell.modifier;
            }
            if cell.fg != fg || cell.bg != bg {
                queue!(
                    writer,
                    SetColors(Colors::new(
                        dreg_color_to_crossterm_color(cell.fg),
                        dreg_color_to_crossterm_color(cell.bg),
                    ))
                )?;
                fg = cell.fg;
                bg = cell.bg;
            }
            #[cfg(feature = "underline-color")]
            if cell.underline_color != underline_color {
                let color = dreg_color_to_crossterm_color(cell.underline_color);
                queue!(writer, SetUnderlineColor(color))?;
                underline_color = cell.underline_color;
            }

            queue!(writer, Print(cell.symbol()))?;
        }

        #[cfg(feature = "underline-color")]
        return queue!(
            writer,
            SetForegroundColor(CColor::Reset),
            SetBackgroundColor(CColor::Reset),
            SetUnderlineColor(CColor::Reset),
            SetAttribute(CtAttribute::Reset),
        );
        #[cfg(not(feature = "underline-color"))]
        return queue!(
            writer,
            SetForegroundColor(CColor::Reset),
            SetBackgroundColor(CColor::Reset),
            SetAttribute(CtAttribute::Reset),
        );
    }
}



fn bind_terminal() -> Result<()> {
    let mut writer = stdout();
    enable_raw_mode()?;
    writer.execute(EnableMouseCapture)?;
    writer.execute(EnterAlternateScreen)?;
    writer.execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES
    ))?;
    writer.execute(Hide)?;
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



fn handle_crossterm_event(program: &mut impl Program, event: Event) {
    match event {
        Event::Key(KeyEvent { code, modifiers, kind, .. }) => {
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
            scancodes.extend(crossterm_keycode_to_dreg_scancode(code));
            match kind {
                KeyEventKind::Press => {
                    for scancode in scancodes {
                        program.on_input(Input::KeyDown(scancode));
                    }
                }
                KeyEventKind::Release => {
                    for scancode in scancodes {
                        program.on_input(Input::KeyUp(scancode));
                    }
                }
                _ => {} // Do nothing.
            }
        }
        Event::Mouse(MouseEvent { kind, column, row, .. }) => {
            match kind {
                MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                    program.on_input(Input::MouseMove(column, row));
                }
                MouseEventKind::Down(btn) => {
                    let code = match btn {
                        MouseButton::Left => Scancode::LMB,
                        MouseButton::Right => Scancode::RMB,
                        MouseButton::Middle => Scancode::MMB,
                    };
                    program.on_input(Input::KeyDown(code));
                }
                MouseEventKind::Up(btn) => {
                    let code = match btn {
                        MouseButton::Left => Scancode::LMB,
                        MouseButton::Right => Scancode::RMB,
                        MouseButton::Middle => Scancode::MMB,
                    };
                    program.on_input(Input::KeyUp(code));
                }
                _ => {} // TODO: Handle scroll wheel events.
            }
        }
        Event::FocusGained => {
            program.on_input(Input::FocusChange(true));
        }
        Event::FocusLost => {
            program.on_input(Input::FocusChange(false));
        }
        Event::Resize(new_cols, new_rows) => {
            program.on_input(Input::Resize(new_cols, new_rows));
        }
        _ => {}
    }
}

pub fn crossterm_keycode_to_dreg_scancode(code: KeyCode) -> Vec<Scancode> {
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



/// The `ModifierDiff` struct is used to calculate the difference between two `Modifier` values.
/// This is useful when updating the terminal display, as it allows for more efficient updates by
/// only sending the necessary changes.
struct ModifierDiff {
    pub from: Modifier,
    pub to: Modifier,
}

impl ModifierDiff {
    fn queue<W: std::io::Write>(self, mut w: W) -> std::io::Result<()> {
        //use crossterm::Attribute;
        let removed = self.from - self.to;
        if removed.contains(Modifier::REVERSED) {
            queue!(w, SetAttribute(CtAttribute::NoReverse))?;
        }
        if removed.contains(Modifier::BOLD) {
            queue!(w, SetAttribute(CtAttribute::NormalIntensity))?;
            if self.to.contains(Modifier::DIM) {
                queue!(w, SetAttribute(CtAttribute::Dim))?;
            }
        }
        if removed.contains(Modifier::ITALIC) {
            queue!(w, SetAttribute(CtAttribute::NoItalic))?;
        }
        if removed.contains(Modifier::UNDERLINED) {
            queue!(w, SetAttribute(CtAttribute::NoUnderline))?;
        }
        if removed.contains(Modifier::DIM) {
            queue!(w, SetAttribute(CtAttribute::NormalIntensity))?;
        }
        if removed.contains(Modifier::CROSSED_OUT) {
            queue!(w, SetAttribute(CtAttribute::NotCrossedOut))?;
        }
        if removed.contains(Modifier::SLOW_BLINK) || removed.contains(Modifier::RAPID_BLINK) {
            queue!(w, SetAttribute(CtAttribute::NoBlink))?;
        }

        let added = self.to - self.from;
        if added.contains(Modifier::REVERSED) {
            queue!(w, SetAttribute(CtAttribute::Reverse))?;
        }
        if added.contains(Modifier::BOLD) {
            queue!(w, SetAttribute(CtAttribute::Bold))?;
        }
        if added.contains(Modifier::ITALIC) {
            queue!(w, SetAttribute(CtAttribute::Italic))?;
        }
        if added.contains(Modifier::UNDERLINED) {
            queue!(w, SetAttribute(CtAttribute::Underlined))?;
        }
        if added.contains(Modifier::DIM) {
            queue!(w, SetAttribute(CtAttribute::Dim))?;
        }
        if added.contains(Modifier::CROSSED_OUT) {
            queue!(w, SetAttribute(CtAttribute::CrossedOut))?;
        }
        if added.contains(Modifier::SLOW_BLINK) {
            queue!(w, SetAttribute(CtAttribute::SlowBlink))?;
        }
        if added.contains(Modifier::RAPID_BLINK) {
            queue!(w, SetAttribute(CtAttribute::RapidBlink))?;
        }

        Ok(())
    }
}

pub fn crossterm_attribute_to_dreg_modifier(value: CtAttribute) -> Modifier {
    // `Attribute*s*` (note the *s*) contains multiple `Attribute`
    // We convert `Attribute` to `Attribute*s*` (containing only 1 value) to avoid implementing
    // the conversion again
    crossterm_attributes_to_dreg_modifier(CtAttributes::from(value))
}

pub fn crossterm_attributes_to_dreg_modifier(value: CtAttributes) -> Modifier {
    let mut res = Modifier::empty();

    if value.has(CtAttribute::Bold) {
        res |= Modifier::BOLD;
    }
    if value.has(CtAttribute::Dim) {
        res |= Modifier::DIM;
    }
    if value.has(CtAttribute::Italic) {
        res |= Modifier::ITALIC;
    }
    if value.has(CtAttribute::Underlined)
        || value.has(CtAttribute::DoubleUnderlined)
        || value.has(CtAttribute::Undercurled)
        || value.has(CtAttribute::Underdotted)
        || value.has(CtAttribute::Underdashed)
    {
        res |= Modifier::UNDERLINED;
    }
    if value.has(CtAttribute::SlowBlink) {
        res |= Modifier::SLOW_BLINK;
    }
    if value.has(CtAttribute::RapidBlink) {
        res |= Modifier::RAPID_BLINK;
    }
    if value.has(CtAttribute::Reverse) {
        res |= Modifier::REVERSED;
    }
    if value.has(CtAttribute::Hidden) {
        res |= Modifier::HIDDEN;
    }
    if value.has(CtAttribute::CrossedOut) {
        res |= Modifier::CROSSED_OUT;
    }

    res
}



/// Convert a dreg [`Color`] to a crossterm-compatible color.
pub fn dreg_color_to_crossterm_color(color: Color) -> CColor {
    match color {
        Color::Reset => CColor::Reset,
        Color::Black => CColor::Black,
        Color::Red => CColor::DarkRed,
        Color::Green => CColor::DarkGreen,
        Color::Yellow => CColor::DarkYellow,
        Color::Blue => CColor::DarkBlue,
        Color::Magenta => CColor::DarkMagenta,
        Color::Cyan => CColor::DarkCyan,
        Color::Gray => CColor::Grey,
        Color::DarkGray => CColor::DarkGrey,
        Color::LightRed => CColor::Red,
        Color::LightGreen => CColor::Green,
        Color::LightBlue => CColor::Blue,
        Color::LightYellow => CColor::Yellow,
        Color::LightMagenta => CColor::Magenta,
        Color::LightCyan => CColor::Cyan,
        Color::White => CColor::White,
        Color::Indexed(i) => CColor::AnsiValue(i),
        Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
    }
}

/// Convert a crossterm-compatible color to a dreg [`Color`].
pub fn crossterm_color_to_dreg_color(value: CColor) -> Color {
    match value {
        CColor::Reset => Color::Reset,
        CColor::Black => Color::Black,
        CColor::DarkRed => Color::Red,
        CColor::DarkGreen => Color::Green,
        CColor::DarkYellow => Color::Yellow,
        CColor::DarkBlue => Color::Blue,
        CColor::DarkMagenta => Color::Magenta,
        CColor::DarkCyan => Color::Cyan,
        CColor::Grey => Color::Gray,
        CColor::DarkGrey => Color::DarkGray,
        CColor::Red => Color::LightRed,
        CColor::Green => Color::LightGreen,
        CColor::Blue => Color::LightBlue,
        CColor::Yellow => Color::LightYellow,
        CColor::Magenta => Color::LightMagenta,
        CColor::Cyan => Color::LightCyan,
        CColor::White => Color::White,
        CColor::Rgb { r, g, b } => Color::Rgb(r, g, b),
        CColor::AnsiValue(v) => Color::Indexed(v),
    }
}
