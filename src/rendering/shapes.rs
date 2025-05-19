//! Renderable Shapes



use crate::{Area, Buffer, Style};



pub struct Rectangle {
    pub style: Style,
    pub rect_style: RectangleStyle,
}

impl Rectangle {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            rect_style: RectangleStyle::Normal,
        }
    }

    pub fn render(self, area: Area, buf: &mut Buffer) {
        let (lt, rt, lb, rb, h, v) = self.rect_style.characters();

        for y in area.top()..area.bottom() {
            buf.get_mut(area.x, y)
                .set_char(v)
                .set_style(self.style);
            buf.get_mut(area.right().saturating_sub(1), y)
                .set_char(v)
                .set_style(self.style);
        }
        for x in area.left()..area.right() {
            buf.get_mut(x, area.y)
                .set_char(h)
                .set_style(self.style);
            buf.get_mut(x, area.bottom().saturating_sub(1))
                .set_char(h)
                .set_style(self.style);
        }
        buf.get_mut(area.x, area.y)
            .set_char(lt)
            .set_style(self.style);
        buf.get_mut(area.right().saturating_sub(1), area.y)
            .set_char(rt)
            .set_style(self.style);
        buf.get_mut(area.right().saturating_sub(1), area.bottom().saturating_sub(1))
            .set_char(rb)
            .set_style(self.style);
        buf.get_mut(area.x, area.bottom().saturating_sub(1))
            .set_char(lb)
            .set_style(self.style);
    }
}

/// Comparison of normal, heavy, double, and round styles:
/// ```text
/// ┌─┐┏━┓╔═╗╭─╮
/// │ │┃ ┃║ ║│ │
/// └─┘┗━┛╚═╝╰─╯
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub enum RectangleStyle {
    /// ```text
    /// ┌─┐
    /// │ │
    /// └─┘
    /// ```
    #[default]
    Normal,
    /// ```text
    /// ┏━┓
    /// ┃ ┃
    /// ┗━┛
    /// ```
    Heavy,
    /// ```text
    /// ╔═╗
    /// ║ ║
    /// ╚═╝
    /// ```
    Double,
    /// ```text
    /// ╭─╮
    /// │ │
    /// ╰─╯
    /// ```
    Round,
}

impl RectangleStyle {
    const fn characters(&self) -> (char, char, char, char, char, char) {
        match self {
            RectangleStyle::Normal => ('┌', '┐', '└', '┘', '─', '│'),
            RectangleStyle::Heavy => ('┏', '┓', '┗', '┛', '━', '┃'),
            RectangleStyle::Double => ('╔', '╗', '╚', '╝', '═', '║'),
            RectangleStyle::Round => ('╭', '╮', '╰', '╯', '─', '│'),
        }
    }
}
