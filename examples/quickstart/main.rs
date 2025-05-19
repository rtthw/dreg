


use dreg::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    Terminal::new().run(MyApp {
        shutdown: false,
    })
}



struct MyApp {
    shutdown: bool,
}

impl Program for MyApp {
    fn render(&mut self, frame: &mut Frame) {
        if self.shutdown {
            frame.should_exit = true;
            return;
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
