


use dreg::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    Terminal::new().run(App {
        shutdown: false,
        editor: Editor {
            content: include_str!("sample.txt").to_string(),
            cursor_pos: (0, 0),
        },
    })
}



struct App {
    shutdown: bool,
    editor: Editor,
}

impl Program for App {
    fn render(&mut self, frame: &mut Frame) {
        if self.shutdown {
            frame.should_exit = true;
            return;
        }
        if frame.cols < 80 || frame.rows < 20 {
            render_frame_size_warning(frame);
            return;
        }

        self.editor.render(frame);
    }

    fn input(&mut self, input: Input) {
        match input {
            Input::KeyDown(Scancode::Q) => {
                self.shutdown = true;
            }
            _ => {}
        }
    }
}

fn render_frame_size_warning(frame: &mut Frame) {
    if frame.cols < 20 || frame.rows < 3 {
        // The window is too small for even the warning.
        if frame.cols >= 2 && frame.rows >= 2 {
            Rectangle {
                style: Style {
                    fg: Some(GRAY_5),
                    ..Default::default()
                },
                rect_style: RectangleStyle::Round,
            }
            .render(frame.area(), frame.buffer);
        }
        return;
    }

    Rectangle {
        style: Style {
            fg: Some(GRAY_5),
            ..Default::default()
        },
        rect_style: RectangleStyle::Round,
    }
    .render(frame.area(), frame.buffer);

    let text_area = frame.area().inner_centered(18, 3);
    frame.buffer.set_stringn(
        text_area.x + 1,
        text_area.y,
        "Window too small",
        16,
        Style::default(),
    );
    frame.buffer.set_stringn(
        text_area.x,
        text_area.y + 1,
        "Resize this window",
        18,
        Style::default(),
    );
    frame.buffer.set_stringn(
        text_area.x + 1,
        text_area.y + 2,
        "to at least 80x20",
        18,
        Style::default(),
    );
}



struct Editor {
    content: String,
    cursor_pos: (u16, u16),
}

impl Editor {
    fn render(&mut self, frame: &mut Frame) {
        Rectangle {
            style: Style {
                fg: Some(GRAY_5),
                ..Default::default()
            },
            rect_style: RectangleStyle::Round,
        }
        .render(frame.area(), frame.buffer);

        // Remember, we know that the frame's width is at least 80 cols, so the side panel is at
        // least 15 cols ((80 - 2, from margin) * 0.2).
        let (_side_panel_area, working_area) = frame.area().shrink(2, 2).hsplit_portion(0.2);
        if working_area.w > 80 {
            // TODO: Render the overflow line.
        }
    }
}



pub const GRAY_0: Color = Color::from_rgb(13, 13, 23); // 0d0d17
pub const GRAY_1: Color = Color::from_rgb(29, 29, 39); // 1d1d27
pub const GRAY_2: Color = Color::from_rgb(43, 43, 53); // 2b2b35
pub const GRAY_3: Color = Color::from_rgb(59, 59, 67); // 3b3b43
pub const GRAY_4: Color = Color::from_rgb(73, 73, 83); // 494953
pub const GRAY_5: Color = Color::from_rgb(89, 89, 109); // 59596d
pub const GRAY_6: Color = Color::from_rgb(113, 113, 127); // 71717f
pub const GRAY_7: Color = Color::from_rgb(139, 139, 149); // 8b8b95
pub const GRAY_8: Color = Color::from_rgb(163, 163, 173); // a3a3ad
pub const GRAY_9: Color = Color::from_rgb(191, 191, 197); // bfbfc5
