


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
            area: frame.area(),
            fg: Color::from_rgb(89, 89, 109),
            style: RectangleStyle::Round,
        }.render(frame);

        let text_area = frame.area().inner_centered(13, 1);
        Text::new("Hello, World!")
            .with_position(text_area.x, text_area.y)
            .render(frame);
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
