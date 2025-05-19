//! Styling



use super::{Color, TextModifier};



#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub add_modifier: TextModifier,
    pub sub_modifier: TextModifier,
}
