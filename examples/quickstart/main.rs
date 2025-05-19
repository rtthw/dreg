


use dreg::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    Terminal::new().run(MyApp {
        initialized: false,
        shutdown: false,
    })
}



struct MyApp {
    initialized: bool,
    shutdown: bool,
}

impl Program for MyApp {
    fn render(&mut self, frame: &mut Frame) {
        if self.shutdown {
            frame.should_exit = true;
            return;
        }
        if !self.initialized {
            frame.commands.push(Command::SetCursorStyle(CursorStyle::BlinkingBar));
            // frame.commands.push(Command::SetTitle("MyApp".to_string()));
        }

        Rectangle {
            style: Style {
                fg: Some(Color::Rgb(89, 89, 109)),
                ..Default::default()
            },
            rect_style: RectangleStyle::Round,
        }
        .render(frame.area(), frame.buffer);

        let text_area = frame.area().inner_centered(13, 1);
        frame.buffer.set_stringn(text_area.x, text_area.y, "Hello, World!", 13, Style::default());

        frame.cursor = Some((13, 3));
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
