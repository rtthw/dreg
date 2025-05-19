


use dreg::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    Terminal::new().run(Showcase {
        input_context: InputContext::default(),
    })
}



struct Showcase {
    input_context: InputContext,
}

impl Program for Showcase {
    fn render(&mut self, frame: &mut Frame) {
        let (left_area, right_area) = frame.area().hsplit_portion(0.2);

        Rectangle {
            style: Style {
                fg: Some(Color::from_rgb(89, 89, 89)),
                ..Default::default()
            },
            rect_style: RectangleStyle::Normal,
        }.render(left_area, frame.buffer);

        Rectangle {
            style: Style {
                fg: Some(Color::from_rgb(137, 137, 151)),
                ..Default::default()
            },
            rect_style: RectangleStyle::Double,
        }.render(right_area, frame.buffer);

        self.input_context.end_frame();
    }

    fn input(&mut self, input: Input) {
        self.input_context.handle_input(input);

        if self.input_context.is_key_down(&Scancode::Q) {
            if self.input_context.keys_down().len() == 1 {
                std::process::exit(0);
            }
        }
    }
}
