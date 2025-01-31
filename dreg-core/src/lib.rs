//! Dreg Core



pub mod color;
pub mod input;
pub mod text_modifier;

pub use color::*;
pub use input::*;
pub use text_modifier::*;

use compact_str::CompactString;



/// The object responsible for rendering [`Buffer`]s, handling user [`Input`], and responding to
/// [`Platform`] requests,
pub trait Program: 'static {
    /// Update the program's state. This method is called every frame, regardless of user input.
    fn update(&mut self, frame: Frame);

    // /// This function is called whenever the running platform receives some user [`Input`].
    // fn on_input(&mut self, input: Input);

    /// This function is called every frame to determine whether the program should exit.
    fn should_exit(&self) -> bool;
}



pub struct Frame<'a> {
    pub buffer: &'a mut Buffer,
}

impl<'a> Frame<'a> {
    pub fn render(&mut self, text: Text) {
        self.buffer.content.push(text);
    }
}



pub struct Buffer {
    content: Vec<Text>,
}

pub struct Text {
    content: CompactString,
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
    pub fn with_content(&mut self, content: &str) -> &mut Self {
        self.content = CompactString::new(content);
        self
    }

    /// Set the text's position.
    pub fn with_position(&mut self, x: u16, y: u16) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the text's x position.
    pub fn with_x(&mut self, x: u16) -> &mut Self {
        self.x = x;
        self
    }

    /// Set the text's x position.
    pub fn with_y(&mut self, y: u16) -> &mut Self {
        self.y = y;
        self
    }
}
