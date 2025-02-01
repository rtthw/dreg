//! Text Modifier Type



bitflags::bitflags! {
    /// A change in the way [text](crate::Text) is displayed.
    ///
    /// They are bitflags, so they can easily be composed.
    ///
    /// Some modifiers may not work on some platforms.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use dreg_core::prelude::*;
    ///
    /// let m = TextModifier::BOLD | TextModifier::ITALIC;
    /// ```
    #[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
    pub struct TextModifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINED        = 0b0000_0000_1000;
        const SLOW_BLINK        = 0b0000_0001_0000;
        const RAPID_BLINK       = 0b0000_0010_0000;
        const REVERSED          = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const CROSSED_OUT       = 0b0001_0000_0000;
    }
}

/// Implement the `Debug` trait for `TextModifier` manually.
///
/// This will avoid printing the empty modifier, and instead print it as 'NONE'.
impl std::fmt::Debug for TextModifier {
    /// Format the modifier as `NONE` if the modifier is empty or as a list of flags separated by
    /// `|` otherwise.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "NONE");
        }
        write!(f, "{}", self.0)
    }
}
