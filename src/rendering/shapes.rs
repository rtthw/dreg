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

        let hbar_num = self.area.w.saturating_sub(1) as usize;
        let vbar_num = self.area.h.saturating_sub(2) as usize;

        let chars = self.style.characters();

        // FIXME: This works, but it's ugly and probably inefficient.
        let row_str = chars[4].to_string().repeat(hbar_num.saturating_sub(1));
        let middle_rows = format!("\n{: <hbar_num$}{}", chars[5], chars[5]).repeat(vbar_num);
        let content = format!(
            "{}{}{}{}\n{}{}{}",
            chars[0], &row_str, chars[1], middle_rows, chars[2], &row_str, chars[3],
        );

        frame.render(Text::default()
            .with_content(&content) // Can't use `Text::new` for local variable.
            .with_fg(self.fg)
            .with_x(self.area.x)
            .with_y(self.area.y))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum RectangleStyle {
    #[default]
    Normal,
    Heavy,
}

impl RectangleStyle {
    const fn characters(&self) -> [char; 6] {
        match self {
            RectangleStyle::Normal => ['┌', '┐', '└', '┘', '─', '│'],
            RectangleStyle::Heavy => ['┏', '┓', '┗', '┛', '━', '┃'],
        }
    }
}
