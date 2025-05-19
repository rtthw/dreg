//! Cell Type



use compact_str::CompactString;

use crate::{Color, TextModifier};

use super::Frame;



#[derive(Eq, PartialEq)]
pub struct Cell {
    pub(crate) symbol: CompactString,
    /// The foreground color for the cell.
    pub fg: Color,
    /// The background color for the cell.
    pub bg: Color,
    /// The modifier for the cell.
    pub modifier: TextModifier,
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Cell {
    /// An empty cell.
    pub const EMPTY: Self = Self::new(" ");

    /// Create a new cell with the given content.
    pub const fn new(content: &'static str) -> Self {
        Self {
            symbol: CompactString::const_new(content),
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: TextModifier::empty(),
        }
    }

    /// Set the cell's content.
    pub fn with_content(mut self, content: &str) -> Self {
        self.symbol = CompactString::new(content);
        self
    }

    /// Set the cell's foreground color.
    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg = color;
        self
    }

    /// Set the cell's background color.
    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    /// Add the given modifier to the cell.
    pub fn with_modifier(mut self, modifier: TextModifier) -> Self {
        self.modifier = self.modifier.union(modifier);
        self
    }

    /// Remove the given modifier from the cell.
    pub fn without_modifier(mut self, modifier: TextModifier) -> Self {
        self.modifier = self.modifier.difference(modifier);
        self
    }

    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = CompactString::new(symbol);
        self
    }

    /// Render this cell to the given [`Frame`].
    pub fn render(self, frame: &mut Frame) {
        frame.render(self);
    }

    /// Reset this cell to the [`Cell::EMPTY`] state.
    pub fn reset(&mut self) {
        self.symbol = CompactString::const_new(" ");
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        self.modifier = TextModifier::empty();
    }
}
