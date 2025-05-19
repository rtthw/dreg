


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
            area: left_area,
            fg: Color::from_rgb(89, 89, 89),
            style: RectangleStyle::Normal,
        }.render(frame);

        Rectangle {
            area: right_area,
            fg: Color::from_rgb(137, 137, 151),
            style: RectangleStyle::Double,
        }.render(frame);

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
