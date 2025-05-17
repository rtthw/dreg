//! Renderable Shapes



use crate::{Area, Color, Frame, Text};



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

        // Can't use `Text::new` for local variables.
        frame.render(Text::default()
            .with_content(&format!("{}", chars[0]))
            .with_fg(self.fg)
            .with_position(self.area.x, self.area.y));
        frame.render(Text::default()
            .with_content(&format!("{}", chars[1]))
            .with_fg(self.fg)
            .with_position(self.area.x + self.area.w.saturating_sub(1), self.area.y));
        frame.render(Text::default()
            .with_content(&format!("{}", chars[2]))
            .with_fg(self.fg)
            .with_position(self.area.x, self.area.y + self.area.h.saturating_sub(1)));
        frame.render(Text::default()
            .with_content(&format!("{}", chars[3]))
            .with_fg(self.fg)
            .with_position(self.area.x + self.area.w.saturating_sub(1), self.area.y + self.area.h.saturating_sub(1)));

        frame.render(Text::default()
            .with_content(&vline)
            .with_fg(self.fg)
            .with_x(self.area.x)
            .with_y(self.area.y));
        frame.render(Text::default()
            .with_content(&vline)
            .with_fg(self.fg)
            .with_x(self.area.x + self.area.w.saturating_sub(1))
            .with_y(self.area.y));
        frame.render(Text::default()
            .with_content(&row_str)
            .with_fg(self.fg)
            .with_x(self.area.x + 1)
            .with_y(self.area.y));
        frame.render(Text::default()
            .with_content(&row_str)
            .with_fg(self.fg)
            .with_x(self.area.x + 1)
            .with_y(self.area.y + self.area.h.saturating_sub(1)));
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
