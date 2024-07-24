//! Label Element



use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation as _;
use unicode_width::UnicodeWidthStr as _;

use crate::prelude::*;



// ================================================================================================



#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Label<'a> {
    /// The content of the label as a Clone-on-write string.
    pub content: Cow<'a, str>,
    /// The style of the label.
    pub style: Style,
}

impl<'a> Element for Label<'a> {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let Rect { mut x, y, .. } = area.intersection(buf.area);
        for (i, grapheme) in self.styled_graphemes(Style::default()).enumerate() {
            let symbol_width = grapheme.symbol.width();
            let next_x = x.saturating_add(symbol_width as u16);
            if next_x > area.intersection(buf.area).right() {
                break;
            }

            if i == 0 {
                // the first grapheme is always set on the cell
                buf.get_mut(x, y)
                    .set_symbol(grapheme.symbol)
                    .set_style(grapheme.style);
            } else if x == area.x {
                // there is one or more zero-width graphemes in the first cell, so the first cell
                // must be appended to.
                buf.get_mut(x, y)
                    .append_symbol(grapheme.symbol)
                    .set_style(grapheme.style);
            } else if symbol_width == 0 {
                // append zero-width graphemes to the previous cell
                buf.get_mut(x - 1, y)
                    .append_symbol(grapheme.symbol)
                    .set_style(grapheme.style);
            } else {
                // just a normal grapheme (not first, not zero-width, not overflowing the area)
                buf.get_mut(x, y)
                    .set_symbol(grapheme.symbol)
                    .set_style(grapheme.style);
            }

            // multi-width graphemes must clear the cells of characters that are hidden by the
            // grapheme, otherwise the hidden characters will be re-rendered if the grapheme is
            // overwritten.
            for x_hidden in (x + 1)..next_x {
                // it may seem odd that the style of the hidden cells are not set to the style of
                // the grapheme, but this is how the existing buffer.set_label() method works.
                buf.get_mut(x_hidden, y).reset();
            }
            x = next_x;
        }
    }
}

impl<'a> Label<'a> {
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn styled<T, S>(content: T, style: S) -> Self
    where
        T: Into<Cow<'a, str>>,
        S: Into<Style>,
    {
        Self {
            content: content.into(),
            style: style.into(),
        }
    }

    /// Returns the unicode width of the content held by this label.
    pub fn width(&self) -> usize {
        self.content.width()
    }

    /// Returns an iterator over the graphemes held by this label.
    ///
    /// `base_style` is the [`Style`] that will be patched with the `Label`'s `style` to get the
    /// resulting [`Style`].
    ///
    /// `base_style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`],
    /// or your own type that implements [`Into<Style>`]).
    pub fn styled_graphemes<S: Into<Style>>(
        &'a self,
        base_style: S,
    ) -> impl Iterator<Item = StyledGrapheme<'a>> {
        let style = base_style.into().patch(self.style);
        self.content
            .as_ref()
            .graphemes(true)
            .filter(|g| *g != "\n")
            .map(move |g| StyledGrapheme { symbol: g, style })
    }
}

impl<'a> Label<'a> {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.style = self.style.add_modifier(Modifier::BOLD);
        self
    }

    pub fn italic(mut self) -> Self {
        self.style = self.style.add_modifier(Modifier::ITALIC);
        self
    }

    pub fn dim(mut self) -> Self {
        self.style = self.style.add_modifier(Modifier::DIM);
        self
    }
}



// ================================================================================================



const NBSP: &str = "\u{00a0}";
const ZWSP: &str = "\u{200b}";

/// A grapheme associated with a style.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct StyledGrapheme<'a> {
    pub symbol: &'a str,
    pub style: Style,
}

impl<'a> StyledGrapheme<'a> {
    /// Creates a new `StyledGrapheme` with the given symbol and style.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    pub fn new<S: Into<Style>>(symbol: &'a str, style: S) -> Self {
        Self {
            symbol,
            style: style.into(),
        }
    }

    pub fn is_whitespace(&self) -> bool {
        let symbol = self.symbol;
        symbol == ZWSP || symbol.chars().all(char::is_whitespace) && symbol != NBSP
    }
}



// ================================================================================================
