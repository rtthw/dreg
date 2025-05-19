<details>
<summary>Table of Contents</summary>

- [Dreg](#dreg)
  - [Quickstart](#quickstart)
  - [License](#license)

</details>

<!-- cargo-rdme start -->

<div align="center">

<br>[![Crate Badge]][Crate] [![Docs Badge]][Docs] [![License Badge]](./LICENSE)

</div>

# Dreg

A simple text-based user interface library.

## Quickstart

```rust
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
```

## License

[MIT](./LICENSE)

[Crate]: https://crates.io/crates/dreg
[Crate Badge]: https://img.shields.io/crates/v/dreg?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
[Docs Badge]: https://img.shields.io/docsrs/dreg?logo=rust&style=flat-square&logoColor=E05D44
[Docs]: https://docs.rs/dreg
[License Badge]: https://img.shields.io/crates/l/dreg?style=flat-square&color=1370D3
