//! Text Type



use compact_str::CompactString;

use crate::{Color, TextModifier};



pub struct Text {
    pub(crate) content: CompactString,
    pub x: u16,
    pub y: u16,
    /// The foreground color for the text.
    pub fg: Color,

    /// The background color for the text.
    pub bg: Color,

    /// The modifier for the text.
    pub modifier: TextModifier,
}

impl Default for Text {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Text {
    /// An empty piece of text.
    pub const EMPTY: Self = Self::new(" ");

    /// Create a new piece of text with the given content.
    ///
    /// This works at compile time and puts the content onto the stack. It will fail to build when
    /// the content doesn't fit onto the stack and needs to be placed on the heap.
    ///
    /// Use `Self::default().set_content()` in that case. See [`CompactString::const_new`] for more
    /// details.
    pub const fn new(content: &'static str) -> Self {
        Self {
            content: CompactString::const_new(content),
            x: 0,
            y: 0,
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: TextModifier::empty(),
        }
    }

    /// Set the text's content.
    pub fn with_content(mut self, content: &str) -> Self {
        self.content = CompactString::new(content);
        self
    }

    /// Set the text's position.
    pub fn with_position(mut self, x: u16, y: u16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the text's x position.
    pub fn with_x(mut self, x: u16) -> Self {
        self.x = x;
        self
    }

    /// Set the text's x position.
    pub fn with_y(mut self, y: u16) -> Self {
        self.y = y;
        self
    }
}
