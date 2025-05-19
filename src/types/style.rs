//! Styling



use super::{Color, Modifier};



#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub add_modifier: Modifier,
    pub sub_modifier: Modifier,
}

impl Into<Style> for Color {
    fn into(self) -> Style {
        Style {
            fg: Some(self),
            ..Default::default()
        }
    }
}

impl Into<Style> for (Color, Color) {
    fn into(self) -> Style {
        Style {
            fg: Some(self.0),
            bg: Some(self.1),
            ..Default::default()
        }
    }
}

impl Into<Style> for Modifier {
    fn into(self) -> Style {
        Style {
            add_modifier: self,
            ..Default::default()
        }
    }
}

impl Style {
    /// Set the foreground [`Color`] for this style.
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set the background [`Color`] for this style.
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Add a [`Modifier`] to this style.
    pub const fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.sub_modifier = self.sub_modifier.difference(modifier);
        self.add_modifier = self.add_modifier.union(modifier);
        self
    }

    /// Remove a [`Modifier`] from this style.
    pub const fn remove_modifier(mut self, modifier: Modifier) -> Self {
        self.add_modifier = self.add_modifier.difference(modifier);
        self.sub_modifier = self.sub_modifier.union(modifier);
        self
    }

    /// Add [`Modifier::BOLD`] to this style.
    pub const fn bold(self) -> Self {
        self.add_modifier(Modifier::BOLD)
    }

    /// Add [`Modifier::DIM`] to this style.
    pub const fn dim(self) -> Self {
        self.add_modifier(Modifier::DIM)
    }

    /// Add [`Modifier::ITALIC`] to this style.
    pub const fn italic(self) -> Self {
        self.add_modifier(Modifier::ITALIC)
    }

    /// Add [`Modifier::UNDERLINED`] to this style.
    pub const fn underlined(self) -> Self {
        self.add_modifier(Modifier::UNDERLINED)
    }

    /// Add [`Modifier::REVERSED`] to this style.
    pub const fn reversed(self) -> Self {
        self.add_modifier(Modifier::REVERSED)
    }

    /// Add [`Modifier::HIDDEN`] to this style.
    pub const fn hidden(self) -> Self {
        self.add_modifier(Modifier::HIDDEN)
    }

    /// Add [`Modifier::CROSSED_OUT`] to this style.
    pub const fn crossed_out(self) -> Self {
        self.add_modifier(Modifier::CROSSED_OUT)
    }
}
