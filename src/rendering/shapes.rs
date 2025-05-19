//! Renderable Shapes



use crate::{Area, Color, Frame, Cell};



pub struct Rectangle {
    pub area: Area,
    pub fg: Color,
    pub style: RectangleStyle,
}

impl Rectangle {
    pub fn new(area: Area, fg: Color) -> Self {
        Self {
            area,
            fg,
            style: RectangleStyle::Normal,
        }
    }

    pub fn render(self, frame: &mut Frame) {
        if self.area.w < 2 || self.area.h < 2 {
            return;
        }

        let hbar_num = self.area.w.saturating_sub(2) as usize;
        let vbar_num = self.area.h.saturating_sub(2) as usize;

        let chars = self.style.characters();

        let row_str = chars[4].to_string().repeat(hbar_num);
        let vline = format!("\n{}", chars[5]).repeat(vbar_num);

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
    const fn characters(&self) -> [char; 6] {
        match self {
            RectangleStyle::Normal => ['┌', '┐', '└', '┘', '─', '│'],
            RectangleStyle::Heavy => ['┏', '┓', '┗', '┛', '━', '┃'],
            RectangleStyle::Double => ['╔', '╗', '╚', '╝', '═', '║'],
            RectangleStyle::Round => ['╭', '╮', '╰', '╯', '─', '│'],
        }
    }
}
