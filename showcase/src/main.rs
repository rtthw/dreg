


use dreg::*;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    NativePlatform::with_args(NativeArgs {
        title: "Something".to_string(),
        ..Default::default()
    })
        .run(Showcase)
}



struct Showcase;

impl Program for Showcase {
    fn render(&mut self, frame: &mut Frame) {
        let main_area = Area {
            x: 0,
            y: 0,
            w: frame.cols,
            h: frame.rows,
        };
        let (left_area, right_area) = main_area.hsplit_portion(0.2);

        Rectangle {
            area: left_area,
            fg: Color::from_rgb(89, 89, 89),
            style: RectangleStyle::Normal,
        }.render(frame);

        frame.render(Text::new("Something")
            .with_position(1, 1)
            .with_fg(Color::from_rgb(89, 89, 97)));

        Rectangle {
            area: right_area,
            fg: Color::from_rgb(137, 137, 151),
            style: RectangleStyle::Double,
        }.render(frame);
    }

    fn on_input(&mut self, input: Input) {
        match input {
            Input::KeyDown(Scancode::Q) => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
