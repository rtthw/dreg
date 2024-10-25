//! Buffer



use compact_str::CompactString;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr as _;

use crate::prelude::*;



/// A set of [`Cell`]s that represent a single rendered frame of your program.
#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct Buffer {
    /// The area represented by this buffer.
    pub area: Rect,
    /// The content of this buffer.
    pub content: Vec<Cell>,
}

impl Buffer {
    /// Get a Buffer with all cells set to [`Cell::EMPTY`].
    #[must_use]
    pub fn empty(area: Rect) -> Self {
        Self::filled(area, Cell::EMPTY)
    }

    /// Get a [`Buffer`] with all cells initialized as clones of the given [`Cell`].
    #[must_use]
    pub fn filled(area: Rect, cell: Cell) -> Self {
        let size = area.area() as usize;
        let content = vec![cell; size];
        Self { area, content }
    }

    /// Get the content of this buffer as a slice.
    pub fn content(&self) -> &[Cell] {
        &self.content
    }

    /// Get the [`Rect`] covered by this buffer.
    pub const fn area(&self) -> &Rect {
        &self.area
    }

    /// Get a reference to [`Cell`] at the given coordinates.
    #[track_caller]
    pub fn get(&self, x: u16, y: u16) -> &Cell {
        let i = self.index_of(x, y);
        &self.content[i]
    }

    /// Get a mutable reference to the [`Cell`] at the given coordinates.
    #[track_caller]
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let i = self.index_of(x, y);
        &mut self.content[i]
    }

    /// Get the index of the [`Cell`] in this [`Buffer`]'s contents for the given global (x, y)
    /// coordinates.
    ///
    /// Global coordinates are offset by the Buffer's area offset (`x`/`y`).
    ///
    /// # Panics
    ///
    /// Panics when given an coordinate that is outside of this Buffer's area.
    #[track_caller]
    pub fn index_of(&self, x: u16, y: u16) -> usize {
        debug_assert!(
            x >= self.area.left()
                && x < self.area.right()
                && y >= self.area.top()
                && y < self.area.bottom(),
            "Trying to access position outside the buffer: x={x}, y={y}, area={:?}",
            self.area
        );
        ((y - self.area.y) * self.area.width + (x - self.area.x)) as usize
    }

    /// Get the (global) coordinates of a [`Cell`] from its index.
    ///
    /// Global coordinates are offset by the Buffer's area offset (`x`/`y`).
    ///
    /// # Panics
    ///
    /// Panics when given an index that is outside the Buffer's content.
    pub fn pos_of(&self, i: usize) -> (u16, u16) {
        debug_assert!(
            i < self.content.len(),
            "Trying to get the coords of a cell outside the buffer: i={i} len={}",
            self.content.len()
        );
        (
            self.area.x + (i as u16) % self.area.width,
            self.area.y + (i as u16) / self.area.width,
        )
    }

    /// Write a string to this [`Buffer`], starting at the position (x, y).
    pub fn set_string<T, S>(&mut self, x: u16, y: u16, string: T, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.set_stringn(x, y, string, usize::MAX, style);
    }

    /// Write at most the first `n` characters of a string to this [`Buffer`], if enough space is
    /// available until the end of the line.
    ///
    /// Use [`Buffer::set_string`] when the maximum amount of characters can be printed.
    pub fn set_stringn<T, S>(
        &mut self,
        mut x: u16,
        y: u16,
        string: T,
        max_width: usize,
        style: S,
    ) -> (u16, u16)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        let max_width = max_width.try_into().unwrap_or(u16::MAX);
        let mut remaining_width = self.area.right().saturating_sub(x).min(max_width);
        let graphemes = UnicodeSegmentation::graphemes(string.as_ref(), true)
            .map(|symbol| (symbol, symbol.width() as u16))
            .filter(|(_symbol, width)| *width > 0)
            .map_while(|(symbol, width)| {
                remaining_width = remaining_width.checked_sub(width)?;
                Some((symbol, width))
            });
        let style = style.into();
        for (symbol, width) in graphemes {
            self.get_mut(x, y).set_symbol(symbol).set_style(style);
            let next_symbol = x + width;
            x += 1;
            // Reset following cells if multi-width (they would be hidden by the grapheme),
            while x < next_symbol {
                self.get_mut(x, y).reset();
                x += 1;
            }
        }
        (x, y)
    }

    /// Set the style for all [`Cell`]s in the given `area`.
    ///
    /// `style` accepts any type that is convertible to a [`Style`] object
    ///     (e.g. [`Style`], [`Color`], or your own type that implements [`Into<Style>`]).
    pub fn set_style<S: Into<Style>>(&mut self, area: Rect, style: S) {
        let style = style.into();
        let area = self.area.intersection(area);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                self.get_mut(x, y).set_style(style);
            }
        }
    }

    /// Resize this [`Buffer`] so that the mapped area matches the given `area` and that the buffer
    /// length is equal to area.width * area.height.
    pub fn resize(&mut self, area: Rect) {
        let length = area.area() as usize;
        if self.content.len() > length {
            self.content.truncate(length);
        } else {
            self.content.resize(length, Cell::EMPTY);
        }
        self.area = area;
    }

    /// Reset all [`Cell`]s in this [`Buffer`].
    ///
    /// Internally, this just calls [`Cell::reset`] for all cells in this buffer.
    pub fn reset(&mut self) {
        for cell in &mut self.content {
            cell.reset();
        }
    }

    /// Merge another [`Buffer`] into this one.
    ///
    /// This will overwrite this buffer's contents with the `other` buffer's contents, if
    /// necessary.
    pub fn merge(&mut self, other: &Self) {
        let area = self.area.union(other.area);
        self.content.resize(area.area() as usize, Cell::EMPTY);

        // Move original content to the appropriate space
        let size = self.area.area() as usize;
        for i in (0..size).rev() {
            let (x, y) = self.pos_of(i);
            // New index in content
            let k = ((y - area.y) * area.width + x - area.x) as usize;
            if i != k {
                self.content[k] = self.content[i].clone();
                self.content[i].reset();
            }
        }

        // Push content of the other buffer into this one (may erase previous
        // data)
        let size = other.area.area() as usize;
        for i in 0..size {
            let (x, y) = other.pos_of(i);
            // New index in content
            let k = ((y - area.y) * area.width + x - area.x) as usize;
            self.content[k] = other.content[i].clone();
        }
        self.area = area;
    }

    /// Build a minimal sequence of coordinates and Cells necessary to update the buffer from
    /// `self` to `other`.
    ///
    /// This assumes that both buffers are well-formed, no double-width cell is followed by a
    /// non-blank cell.
    ///
    /// # Multi-width characters handling:
    ///
    /// ```text
    /// (Index:) `01`
    /// Prev:    `コ`
    /// Next:    `aa`
    /// Updates: `0: a, 1: a'
    /// ```
    ///
    /// ```text
    /// (Index:) `01`
    /// Prev:    `a `
    /// Next:    `コ`
    /// Updates: `0: コ` (double width symbol at index 0 - skip index 1)
    /// ```
    ///
    /// ```text
    /// (Index:) `012`
    /// Prev:    `aaa`
    /// Next:    `aコ`
    /// Updates: `0: a, 1: コ` (double width symbol at index 1 - skip index 2)
    /// ```
    pub fn diff<'a>(&self, other: &'a Self) -> Vec<(u16, u16, &'a Cell)> {
        let prev_buf = &self.content;
        let next_buf = &other.content;

        let mut updates: Vec<(u16, u16, &Cell)> = vec![];
        // Cells invalidated by drawing/replacing preceding multi-width characters:
        let mut invalidated: usize = 0;
        // Cells from the current buffer to skip due to preceding multi-width characters taking
        // their place (the skipped cells should be blank anyway), or due to per-cell-skipping:
        let mut to_skip: usize = 0;
        for (i, (current, previous)) in next_buf.iter().zip(prev_buf.iter()).enumerate() {
            if !current.skip && (current != previous || invalidated > 0) && to_skip == 0 {
                let (x, y) = self.pos_of(i);
                updates.push((x, y, &next_buf[i]));
            }

            to_skip = current.symbol().width().saturating_sub(1);

            let affected_width = std::cmp::max(
                current.symbol().width(),
                previous.symbol().width(),
            );
            invalidated = std::cmp::max(affected_width, invalidated).saturating_sub(1);
        }
        updates
    }
}

impl std::fmt::Debug for Buffer {
    /// Writes a debug representation of the buffer to the given formatter.
    ///
    /// The format is like a pretty printed struct, with the following fields:
    /// * `area`: displayed as `Rect { x: 1, y: 2, width: 3, height: 4 }`
    /// * `content`: displayed as a list of strings representing the content of the buffer
    /// * `styles`: displayed as a list of: `{ x: 1, y: 2, fg: Color::Red, bg: Color::Blue,
    ///   modifier: Modifier::BOLD }` only showing a value when there is a change in style.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Buffer {{\n    area: {:?}", &self.area))?;

        if self.area.is_empty() {
            return f.write_str("\n}");
        }

        f.write_str(",\n    content: [\n")?;
        let mut last_style = None;
        let mut styles = vec![];
        for (y, line) in self.content.chunks(self.area.width as usize).enumerate() {
            let mut overwritten = vec![];
            let mut skip: usize = 0;
            f.write_str("        \"")?;
            for (x, c) in line.iter().enumerate() {
                if skip == 0 {
                    f.write_str(c.symbol())?;
                } else {
                    overwritten.push((x, c.symbol()));
                }
                skip = std::cmp::max(skip, c.symbol().width()).saturating_sub(1);
                #[cfg(feature = "underline-color")]
                {
                    let style = (c.fg, c.bg, c.underline_color, c.modifier);
                    if last_style != Some(style) {
                        last_style = Some(style);
                        styles.push((x, y, c.fg, c.bg, c.underline_color, c.modifier));
                    }
                }
                #[cfg(not(feature = "underline-color"))]
                {
                    let style = (c.fg, c.bg, c.modifier);
                    if last_style != Some(style) {
                        last_style = Some(style);
                        styles.push((x, y, c.fg, c.bg, c.modifier));
                    }
                }
            }
            f.write_str("\",")?;
            if !overwritten.is_empty() {
                f.write_fmt(format_args!(
                    " // hidden by multi-width symbols: {overwritten:?}"
                ))?;
            }
            f.write_str("\n")?;
        }
        f.write_str("    ],\n    styles: [\n")?;
        for s in styles {
            #[cfg(feature = "underline-color")]
            f.write_fmt(format_args!(
                "        x: {}, y: {}, fg: {:?}, bg: {:?}, underline: {:?}, modifier: {:?},\n",
                s.0, s.1, s.2, s.3, s.4, s.5
            ))?;
            #[cfg(not(feature = "underline-color"))]
            f.write_fmt(format_args!(
                "        x: {}, y: {}, fg: {:?}, bg: {:?}, modifier: {:?},\n",
                s.0, s.1, s.2, s.3, s.4
            ))?;
        }
        f.write_str("    ]\n}")?;
        Ok(())
    }
}



/// A buffer cell.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Cell {
    /// The string to be drawn in the cell.
    ///
    /// This accepts unicode grapheme clusters which might take up more than one cell.
    ///
    /// This is a [`CompactString`], which is a wrapper around [`String`] that uses a small inline
    /// buffer for short strings.
    ///
    /// See <https://github.com/ratatui-org/ratatui/pull/601> for more information.
    symbol: CompactString,

    /// The foreground color for the cell.
    pub fg: Color,

    /// The background color for the cell.
    pub bg: Color,

    /// The underline color for the cell.
    #[cfg(feature = "underline-color")]
    pub underline_color: Color,

    /// The modifier for the cell.
    pub modifier: Modifier,

    /// Whether the cell should be skipped when copying (diffing) the [`Buffer`] to the screen.
    pub skip: bool,
}

impl Cell {
    /// An empty `Cell` (no character).
    pub const EMPTY: Self = Self::new(" ");

    /// Create a new `Cell` with the given symbol.
    ///
    /// This works at compile time and puts the symbol onto the stack. It will fail to build when
    /// the symbol doesn't fit onto the stack and needs to be placed on the heap.
    ///
    /// Use `Self::default().set_symbol()` in that case. See [`CompactString::const_new`] for more
    /// details.
    pub const fn new(symbol: &'static str) -> Self {
        Self {
            symbol: CompactString::const_new(symbol),
            fg: Color::Reset,
            bg: Color::Reset,
            #[cfg(feature = "underline-color")]
            underline_color: Color::Reset,
            modifier: Modifier::empty(),
            skip: false,
        }
    }

    /// Get the symbol for this cell.
    #[must_use]
    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    /// Set the symbol of the cell.
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = CompactString::new(symbol);
        self
    }

    /// Set the symbol for this cell to a single character.
    pub fn set_char(&mut self, ch: char) -> &mut Self {
        let mut buf = [0; 4];
        self.symbol = CompactString::new(ch.encode_utf8(&mut buf));
        self
    }

    /// Set the foreground color for this cell.
    pub fn set_fg(&mut self, color: Color) -> &mut Self {
        self.fg = color;
        self
    }

    /// Set the background color for this cell.
    pub fn set_bg(&mut self, color: Color) -> &mut Self {
        self.bg = color;
        self
    }

    /// Set the style for this cell.
    ///
    /// `style` accepts any type that is convertible to a [`Style`] object
    ///     (e.g. [`Style`], [`Color`], or your own type that implements [`Into<Style>`]).
    pub fn set_style<S: Into<Style>>(&mut self, style: S) -> &mut Self {
        let style = style.into();
        if let Some(c) = style.fg {
            self.fg = c;
        }
        if let Some(c) = style.bg {
            self.bg = c;
        }
        #[cfg(feature = "underline-color")]
        if let Some(c) = style.underline_color {
            self.underline_color = c;
        }
        self.modifier.insert(style.add_modifier);
        self.modifier.remove(style.sub_modifier);
        self
    }

    /// Get the style for this cell.
    #[must_use]
    pub const fn style(&self) -> Style {
        Style {
            color_mode: ColorMode::overwrite(),
            fg: Some(self.fg),
            bg: Some(self.bg),
            #[cfg(feature = "underline-color")]
            underline_color: Some(self.underline_color),
            add_modifier: self.modifier,
            sub_modifier: Modifier::empty(),
        }
    }

    /// Set this cell to be skipped when copying (diffing) the buffer to the screen.
    ///
    /// This is helpful when you need to stop the buffer from overwriting a cell that is covered
    /// by an image.
    pub fn set_skip(&mut self, skip: bool) -> &mut Self {
        self.skip = skip;
        self
    }

    /// Reset this cell to the [`Cell::EMPTY`] state.
    pub fn reset(&mut self) {
        self.symbol = CompactString::const_new(" ");
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        #[cfg(feature = "underline-color")]
        {
            self.underline_color = Color::Reset;
        }
        self.modifier = Modifier::empty();
        self.skip = false;
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}
