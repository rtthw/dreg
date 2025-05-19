//! Buffer type



use unicode_width::UnicodeWidthStr as _;

use super::{Area, Cell, Style};



/// A set of [`Cell`]s.
#[derive(Eq, PartialEq)]
pub struct Buffer {
    /// The buffer's area.
    pub area: Area,
    /// The buffer's contents.
    pub content: Vec<Cell>,
}

impl Buffer {
    /// Create a new empty buffer.
    pub fn empty() -> Self {
        Self {
            area: Area::ZERO,
            content: Vec::new(),
        }
    }

    /// Reset all [`Cell`]s in this buffer.
    pub fn reset(&mut self) {
        for cell in &mut self.content {
            cell.reset();
        }
    }

    /// Resize this buffer so that the mapped area matches the given [`Area`] and that the buffer
    /// length is equal to `area.w` * `area.h`.
    pub fn resize(&mut self, area: Area) {
        let length = (area.w * area.h) as usize;
        if self.content.len() > length {
            self.content.truncate(length);
        } else {
            self.content.resize(length, Cell::EMPTY);
        }
        self.area = area;
    }

    /// Write a string to this buffer, starting at the position (x, y).
    pub fn set_string<T, S>(&mut self, x: u16, y: u16, string: T, style: S)
    where
        T: AsRef<str>,
        S: Into<Style>,
    {
        self.set_stringn(x, y, string, usize::MAX, style);
    }

    /// Write at most the first `n` characters of a string to this buffer, if enough space is
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
        let graphemes = unicode_segmentation::UnicodeSegmentation::graphemes(string.as_ref(), true)
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

    /// Get a mutable reference to the [`Cell`] at the given position.
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let i = self.index_of(x, y);
        &mut self.content[i]
    }

    /// Get the index of the [`Cell`] at the given position.
    pub fn index_of(&self, x: u16, y: u16) -> usize {
        ((y - self.area.y) * self.area.w + (x - self.area.x)) as usize
    }

    /// Get the position of the [`Cell`] at the given index.
    pub fn pos_of(&self, i: usize) -> (u16, u16) {
        (
            self.area.x + (i as u16) % self.area.w,
            self.area.y + (i as u16) / self.area.w,
        )
    }

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
            if (current != previous || invalidated > 0) && to_skip == 0 {
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
