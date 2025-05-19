//! Cell Type



use compact_str::CompactString;

use crate::{Color, Modifier};

use super::Style;



#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub(crate) symbol: CompactString,
    /// The foreground color for the cell.
    pub fg: Color,
    /// The background color for the cell.
    pub bg: Color,
    /// The modifier for the cell.
    pub modifier: Modifier,
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
            modifier: Modifier::empty(),
        }
    }

    /// Set the cell's content.
    pub fn with_content(mut self, content: &str) -> Self {
        self.symbol = CompactString::new(content);
        self
    }

    /// Get this cell's symbol as a string slice.
    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    /// Set this cell's symbol to the provided string slice.
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = CompactString::new(symbol);
        self
    }

    /// Set this cell's symbol to the provided character.
    pub fn set_char(&mut self, ch: char) -> &mut Self {
        let mut buf = [0; 4];
        self.symbol = CompactString::new(ch.encode_utf8(&mut buf));
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
        self.modifier.insert(style.add_modifier);
        self.modifier.remove(style.sub_modifier);
        self
    }

    /// Reset this cell to the [`Cell::EMPTY`] state.
    pub fn reset(&mut self) {
        self.symbol = CompactString::const_new(" ");
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        self.modifier = Modifier::empty();
    }
}
