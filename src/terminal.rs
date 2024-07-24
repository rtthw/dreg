//! Terminal



use std::io::Write;

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show}, 
    event::{DisableMouseCapture, EnableMouseCapture}, 
    ExecutableCommand as _,
    execute, 
    queue, 
    style::{
        Attribute as CtAttribute, Attributes as CtAttributes, Color as CColor, Colors, Print, SetAttribute, SetBackgroundColor, SetColors, SetForegroundColor
    }, 
    terminal::{disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(feature = "underline-color")]
use crossterm::style::SetUnderlineColor;

use crate::prelude::*;



// ================================================================================================



pub struct Terminal<W: Write> {
    ctx: Context,
    writer: W,
    settings: TerminalSettings,
    /// Holds the results of the current and previous draw calls. The two are compared at the end
    /// of each draw pass to output the necessary updates to the terminal
    buffers: [Buffer; 2],
    /// Index of the current buffer in the previous array
    current: usize,
    /// Whether the cursor is currently hidden
    hidden_cursor: bool,
    /// Area of the viewport
    viewport_area: Rect,
    /// Last known size of the terminal. Used to detect if the internal buffers have to be resized.
    last_known_size: Rect,
    /// Last known position of the cursor. Used to find the new area when the viewport is inlined
    /// and the terminal resized.
    last_known_cursor_pos: (u16, u16),
    /// Number of frames rendered up until current time.
    frame_count: usize,
}

impl<W: Write> Terminal<W> {
    pub fn new(mut writer: W, settings: TerminalSettings) -> Result<Self> {
        bind_terminal(&mut writer, &settings)?;

        let (width, height) = crossterm::terminal::size()?;
        let terminal_size = Rect::new(0, 0, width, height);
        let size = match settings.viewport {
            Viewport::Fullscreen => terminal_size, // | Viewport::Inline(_)
            // Viewport::Fixed(area) => area,
        };
        let (viewport_area, cursor_pos) = match settings.viewport {
            Viewport::Fullscreen => (size, (0, 0)),
            // Viewport::Inline(height) => compute_inline_size(&mut backend, height, size, 0)?,
            // Viewport::Fixed(area) => (area, (area.left(), area.top())),
        };

        Ok(Self {
            ctx: Context::default(),
            writer,
            settings,
            buffers: [Buffer::empty(viewport_area), Buffer::empty(viewport_area)],
            current: 0,
            hidden_cursor: false,
            viewport_area,
            last_known_size: size,
            last_known_cursor_pos: cursor_pos,
            frame_count: 0,
        })
    }

    pub fn release(self) -> Result<()> {
        release_terminal(self.writer, self.settings)?;
        Ok(())
    }

    pub fn poll_input(&self, timeout: std::time::Duration) -> Option<Input> {
        if crossterm::event::poll(timeout).ok()? {
            Some(Input::from(crossterm::event::read().ok()?))
        } else {
            None
        }
    }

    pub fn render_on_input(
        &mut self, 
        timeout: std::time::Duration, 
        render_fn: impl FnOnce(&mut Frame),
    ) -> Result<()> {
        if let Some(input) = self.poll_input(timeout) {
            self.ctx.handle_input(input);
            self.render(render_fn)?;
        }
        Ok(())
    }

    pub fn render_on_update(
        &mut self, 
        timeout: std::time::Duration, 
        last_tick: std::time::Duration,
        render_fn: impl FnOnce(&mut Frame),
    ) -> Result<()> {
        if let Some(input) = self.poll_input(timeout) {
            self.ctx.handle_input(input);
            #[cfg(feature = "anim")]
            if self.ctx.animating() {
                let mut anims = self.ctx.take_animations();
                self.render(|frame| {
                    render_fn(frame);
                    for (anim, area) in anims.iter_mut() {
                        let _ = anim.process(last_tick, frame.buffer_mut(), *area);
                    }
                })?;
                anims.retain(|(anim, _)| anim.running());
                // Force rerender if all animations have completed.
                if anims.is_empty() {
                    self.ctx.force_render_next_frame = true;
                }
                self.ctx.place_animations(anims);
            } else {
                self.render(render_fn)?;
            }
            #[cfg(not(feature = "anim"))]
            {
                self.render(render_fn)?;
            }
        } else {
            #[cfg(feature = "anim")]
            if self.ctx.animating() {
                let mut anims = self.ctx.take_animations();
                self.render(|frame| {
                    render_fn(frame);
                    for (anim, area) in anims.iter_mut() {
                        let _ = anim.process(last_tick, frame.buffer_mut(), *area);
                    }
                })?;
                anims.retain(|(anim, _)| anim.running());
                self.ctx.place_animations(anims);
            } else if self.ctx.force_render_next_frame {
                self.render(render_fn)?;
            }

            #[cfg(not(feature = "anim"))]
            {
                if self.ctx.force_render_next_frame {
                    self.render(render_fn)?;
                }
            }
        }
        Ok(())
    }

    pub fn render(&mut self, render_fn: impl FnOnce(&mut Frame)) -> Result<()> {
        self.try_render(|frame| {
            render_fn(frame);
            Ok(())
        })
    }

    #[cfg(feature = "anim")]
    pub fn render_animation(
        &mut self,
        effect: &mut Animation,
        area: Rect,
        last_tick: std::time::Duration,
    ) {
        effect.process(
            last_tick,
            self.current_buffer_mut().1,
            area
        );
    }

    fn try_render<F>(&mut self, render_fn: F) -> Result<()> 
    where
        F: FnOnce(&mut Frame) -> std::io::Result<()>,
    {
        self.ctx.force_render_next_frame = false;

        // Autoresize - otherwise we get glitches if shrinking or potential desync between widgets
        // and the terminal (if growing), which may OOB.
        self.autoresize()?;

        let mut frame = self.get_frame();

        render_fn(&mut frame)?;

        // We can't change the cursor position right away because we have to flush the frame to
        // stdout first. But we also can't keep the frame around, since it holds a &mut to
        // Buffer. Thus, we're taking the important data out of the Frame and dropping it.
        let cursor_position = frame.cursor_position;

        self.flush()?;

        match cursor_position {
            None => self.hide_cursor()?,
            Some((x, y)) => {
                self.show_cursor()?;
                self.set_cursor(x, y)?;
            }
        }

        self.swap_buffers();

        // Flush the writer.
        self.writer.flush()?;

        // let completed_frame = CompletedFrame {
        //     buffer: &self.buffers[1 - self.current],
        //     area: self.last_known_size,
        //     count: self.frame_count,
        // };

        // Increment the frame count before returning.
        self.frame_count = self.frame_count.wrapping_add(1);

        Ok(())
    }

    /// Get a Frame object which provides a consistent view into the terminal state for rendering.
    fn get_frame(&mut self) -> Frame {
        let count = self.frame_count;
        Frame {
            cursor_position: None,
            viewport_area: self.viewport_area,
            ctx_buffer: self.current_buffer_mut(),
            count,
        }
    }

    /// Gets the current buffer as a mutable reference.
    fn current_buffer_mut(&mut self) -> (&mut Context, &mut Buffer) {
        (&mut self.ctx, &mut self.buffers[self.current])
    }

    // /// Obtains a difference between the previous and the current buffer and passes it to the
    // /// current backend for drawing.
    // fn flush(&mut self) -> std::io::Result<()> {
    //     let previous_buffer = &self.buffers[1 - self.current];
    //     let current_buffer = &self.buffers[self.current];
    //     let updates = previous_buffer.diff(current_buffer);
    //     if let Some((col, row, _)) = updates.last() {
    //         self.last_known_cursor_pos = (*col, *row);
    //     }
    //     self.draw(updates.into_iter())
    // }

    fn set_viewport_area(&mut self, area: Rect) {
        self.buffers[self.current].resize(area);
        self.buffers[1 - self.current].resize(area);
        self.viewport_area = area;
    }

    /// Updates the Terminal so that internal buffers match the requested size.
    ///
    /// Requested size will be saved so the size can remain consistent when rendering. This leads
    /// to a full clear of the screen.
    fn resize(&mut self, size: Rect) -> std::io::Result<()> {
        let next_area = match self.settings.viewport {
            Viewport::Fullscreen => size,
            // Viewport::Inline(height) => {
            //     let offset_in_previous_viewport = self
            //         .last_known_cursor_pos
            //         .1
            //         .saturating_sub(self.viewport_area.top());
            //     compute_inline_size(&mut self.backend, height, size, offset_in_previous_viewport)?.0
            // }
            // Viewport::Fixed(area) => area,
        };
        self.set_viewport_area(next_area);
        self.clear()?;

        self.last_known_size = size;
        Ok(())
    }

    /// Queries the backend for size and resizes if it doesn't match the previous size.
    fn autoresize(&mut self) -> std::io::Result<()> {
        // fixed viewports do not get autoresized
        if matches!(self.settings.viewport, Viewport::Fullscreen) { // | Viewport::Inline(_)
            let size = self.size()?;
            if size != self.last_known_size {
                self.resize(size)?;
            }
        };
        Ok(())
    }

    fn size(&self) -> std::io::Result<Rect> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Rect::new(0, 0, width, height))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let updates = previous_buffer.diff(current_buffer);
        if let Some((col, row, _)) = updates.last() {
            self.last_known_cursor_pos = (*col, *row);
        }
        let content = updates.into_iter();

        // ---

        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        #[cfg(feature = "underline-color")]
        let mut underline_color = Color::Reset;
        let mut modifier = Modifier::empty();
        let mut last_pos: Option<(u16, u16)> = None;
        for (x, y, cell) in content {
            // Move the cursor if the previous location was not (x - 1, y)
            if !matches!(last_pos, Some(p) if x == p.0 + 1 && y == p.1) {
                queue!(self.writer, MoveTo(x, y))?;
            }
            last_pos = Some((x, y));
            if cell.modifier != modifier {
                let diff = ModifierDiff {
                    from: modifier,
                    to: cell.modifier,
                };
                diff.queue(&mut self.writer)?;
                modifier = cell.modifier;
            }
            if cell.fg != fg || cell.bg != bg {
                queue!(
                    self.writer,
                    SetColors(Colors::new(cell.fg.into(), cell.bg.into()))
                )?;
                fg = cell.fg;
                bg = cell.bg;
            }
            #[cfg(feature = "underline-color")]
            if cell.underline_color != underline_color {
                let color = CColor::from(cell.underline_color);
                queue!(self.writer, SetUnderlineColor(color))?;
                underline_color = cell.underline_color;
            }

            queue!(self.writer, Print(cell.symbol()))?;
        }

        #[cfg(feature = "underline-color")]
        return queue!(
            self.writer,
            SetForegroundColor(CColor::Reset),
            SetBackgroundColor(CColor::Reset),
            SetUnderlineColor(CColor::Reset),
            SetAttribute(CtAttribute::Reset),
        );
        #[cfg(not(feature = "underline-color"))]
        return queue!(
            self.writer,
            SetForegroundColor(CColor::Reset),
            SetBackgroundColor(CColor::Reset),
            SetAttribute(CtAttribute::Reset),
        );
    }

    /// Clear the terminal and force a full redraw on the next draw call.
    fn clear(&mut self) -> std::io::Result<()> {
        match self.settings.viewport {
            Viewport::Fullscreen => self.clear_region(ClearType::All)?,
            // Viewport::Inline(_) => {
            //     self.backend
            //         .set_cursor(self.viewport_area.left(), self.viewport_area.top())?;
            //     self.backend.clear_region(ClearType::AfterCursor)?;
            // }
            // Viewport::Fixed(area) => {
            //     for row in area.top()..area.bottom() {
            //         self.backend.set_cursor(0, row)?;
            //         self.backend.clear_region(ClearType::AfterCursor)?;
            //     }
            // }
        }
        // Reset the back buffer to make sure the next update will redraw everything.
        self.buffers[1 - self.current].reset();
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> std::io::Result<()> {
        execute!(
            self.writer,
            Clear(match clear_type {
                ClearType::All => crossterm::terminal::ClearType::All,
                ClearType::AfterCursor => crossterm::terminal::ClearType::FromCursorDown,
                ClearType::BeforeCursor => crossterm::terminal::ClearType::FromCursorUp,
                ClearType::CurrentLine => crossterm::terminal::ClearType::CurrentLine,
                ClearType::UntilNewLine => crossterm::terminal::ClearType::UntilNewLine,
            })
        )
    }

    /// Clears the inactive buffer and swaps it with the current buffer
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;
    }

    /// Hides the cursor.
    pub fn hide_cursor(&mut self) -> std::io::Result<()> {
        execute!(self.writer, Hide)?;
        self.hidden_cursor = true;
        Ok(())
    }

    /// Shows the cursor.
    pub fn show_cursor(&mut self) -> std::io::Result<()> {
        execute!(self.writer, Show)?;
        self.hidden_cursor = false;
        Ok(())
    }

    /// Gets the current cursor position.
    ///
    /// This is the position of the cursor after the last draw call and is returned as a tuple of
    /// `(x, y)` coordinates.
    pub fn get_cursor(&mut self) -> std::io::Result<(u16, u16)> {
        crossterm::cursor::position()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    /// Sets the cursor position.
    pub fn set_cursor(&mut self, x: u16, y: u16) -> std::io::Result<()> {
        execute!(self.writer, MoveTo(x, y))?;
        self.last_known_cursor_pos = (x, y);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TerminalSettings {
    viewport: Viewport,
    mouse_support: bool,
    alternate_screen: bool,
    release_on_panic: bool,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            viewport: Viewport::Fullscreen,
            mouse_support: true,
            alternate_screen: true,
            release_on_panic: true,
        }
    }
}

fn bind_terminal<W: Write>(writer: &mut W, settings: &TerminalSettings) -> Result<()> {
    enable_raw_mode()?;
    if settings.mouse_support {
        writer.execute(EnableMouseCapture)?;
    }
    if settings.alternate_screen {
        writer.execute(EnterAlternateScreen)?;
    }
    if settings.release_on_panic {
        // let writer = writer.clone();
        let settings = settings.clone();
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            release_terminal(std::io::stderr(), settings).unwrap();
            original_hook(panic);
        }));
    }
    Ok(())
}

fn release_terminal<W: Write>(mut writer: W, settings: TerminalSettings) -> Result<()> {
    disable_raw_mode()?;
    if settings.mouse_support {
        writer.execute(DisableMouseCapture)?;
    }
    if settings.alternate_screen {
        writer.execute(LeaveAlternateScreen)?;
    }
    execute!(writer, Show)?;
    Ok(())
}



// ================================================================================================



pub struct Frame<'a> {
    /// Where should the cursor be after drawing this frame?
    ///
    /// If `None`, the cursor is hidden and its position is controlled by the backend. If `Some((x,
    /// y))`, the cursor is shown and placed at `(x, y)` after the call to `Terminal::draw()`.
    pub(crate) cursor_position: Option<(u16, u16)>,

    /// The area of the viewport
    pub(crate) viewport_area: Rect,

    /// The buffer that is used to draw the current frame
    pub(crate) ctx_buffer: (&'a mut Context, &'a mut Buffer),

    /// The frame count indicating the sequence number of this frame.
    pub(crate) count: usize,
}

impl<'a> Frame<'a> {
    pub const fn size(&self) -> Rect {
        self.viewport_area
    }

    pub fn consume_input(&mut self, input: Input) {
        self.ctx_buffer.0.handle_input(input)
    }

    pub fn render(&mut self, rend: &mut impl Element, area: Rect) {
        rend.render(area, self.ctx_buffer.1)
    }

    pub fn render_with_context(&mut self, rend: &mut impl Program, area: Rect) {
        rend.render(self.ctx_buffer.0, area, self.ctx_buffer.1)
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.ctx_buffer.1
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

pub trait Element {
    fn render(&mut self, area: Rect, buf: &mut Buffer);
}

pub trait Program {
    fn render(&mut self, ctx: &mut Context, area: Rect, buf: &mut Buffer);
}

/// Enum representing the different types of clearing operations that can be performed
/// on the terminal screen.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClearType {
    /// Clear the entire screen.
    All,
    /// Clear everything after the cursor.
    AfterCursor,
    /// Clear everything before the cursor.
    BeforeCursor,
    /// Clear the current line.
    CurrentLine,
    /// Clear everything from the cursor until the next newline.
    UntilNewLine,
}

#[derive(Clone, Copy, Debug)]
pub enum Viewport {
    Fullscreen,
}



// ================================================================================================
// === Interops between `crossterm` and `eor`.



/// The `ModifierDiff` struct is used to calculate the difference between two `Modifier`
/// values. This is useful when updating the terminal display, as it allows for more
/// efficient updates by only sending the necessary changes.
struct ModifierDiff {
    pub from: Modifier,
    pub to: Modifier,
}

impl ModifierDiff {
    fn queue<W: Write>(self, mut w: W) -> std::io::Result<()> {
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

impl From<CtAttribute> for Modifier {
    fn from(value: CtAttribute) -> Self {
        // `Attribute*s*` (note the *s*) contains multiple `Attribute`
        // We convert `Attribute` to `Attribute*s*` (containing only 1 value) to avoid implementing
        // the conversion again
        Self::from(CtAttributes::from(value))
    }
}

impl From<CtAttributes> for Modifier {
    fn from(value: CtAttributes) -> Self {
        let mut res = Self::empty();

        if value.has(CtAttribute::Bold) {
            res |= Self::BOLD;
        }
        if value.has(CtAttribute::Dim) {
            res |= Self::DIM;
        }
        if value.has(CtAttribute::Italic) {
            res |= Self::ITALIC;
        }
        if value.has(CtAttribute::Underlined)
            || value.has(CtAttribute::DoubleUnderlined)
            || value.has(CtAttribute::Undercurled)
            || value.has(CtAttribute::Underdotted)
            || value.has(CtAttribute::Underdashed)
        {
            res |= Self::UNDERLINED;
        }
        if value.has(CtAttribute::SlowBlink) {
            res |= Self::SLOW_BLINK;
        }
        if value.has(CtAttribute::RapidBlink) {
            res |= Self::RAPID_BLINK;
        }
        if value.has(CtAttribute::Reverse) {
            res |= Self::REVERSED;
        }
        if value.has(CtAttribute::Hidden) {
            res |= Self::HIDDEN;
        }
        if value.has(CtAttribute::CrossedOut) {
            res |= Self::CROSSED_OUT;
        }

        res
    }
}

impl From<Color> for CColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::Red => Self::DarkRed,
            Color::Green => Self::DarkGreen,
            Color::Yellow => Self::DarkYellow,
            Color::Blue => Self::DarkBlue,
            Color::Magenta => Self::DarkMagenta,
            Color::Cyan => Self::DarkCyan,
            Color::Gray => Self::Grey,
            Color::DarkGray => Self::DarkGrey,
            Color::LightRed => Self::Red,
            Color::LightGreen => Self::Green,
            Color::LightBlue => Self::Blue,
            Color::LightYellow => Self::Yellow,
            Color::LightMagenta => Self::Magenta,
            Color::LightCyan => Self::Cyan,
            Color::White => Self::White,
            Color::Indexed(i) => Self::AnsiValue(i),
            Color::Rgb(r, g, b) => Self::Rgb { r, g, b },
        }
    }
}

impl From<CColor> for Color {
    fn from(value: CColor) -> Self {
        match value {
            CColor::Reset => Self::Reset,
            CColor::Black => Self::Black,
            CColor::DarkRed => Self::Red,
            CColor::DarkGreen => Self::Green,
            CColor::DarkYellow => Self::Yellow,
            CColor::DarkBlue => Self::Blue,
            CColor::DarkMagenta => Self::Magenta,
            CColor::DarkCyan => Self::Cyan,
            CColor::Grey => Self::Gray,
            CColor::DarkGrey => Self::DarkGray,
            CColor::Red => Self::LightRed,
            CColor::Green => Self::LightGreen,
            CColor::Blue => Self::LightBlue,
            CColor::Yellow => Self::LightYellow,
            CColor::Magenta => Self::LightMagenta,
            CColor::Cyan => Self::LightCyan,
            CColor::White => Self::White,
            CColor::Rgb { r, g, b } => Self::Rgb(r, g, b),
            CColor::AnsiValue(v) => Self::Indexed(v),
        }
    }
}



// ================================================================================================



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_binds_and_releases() {
        let terminal = Terminal::new(std::io::stdout(), TerminalSettings::default());
        assert!(terminal.is_ok());
        assert!(terminal.unwrap().release().is_ok());
    }
}
