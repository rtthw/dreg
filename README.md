
# Dreg

<details>
<summary>Table of Contents</summary>

- [Quickstart](#quickstart)
- [Overview](#overview)
  - [Design Philosophy](#design-philosophy)
- [Acknowledgments](#acknowledgments)
- [License](#license)

</details>

## Quickstart

```rust
use dreg::prelude::*;

fn main() {
    let mut term = Terminal::new(std::io::stdout(), TerminalSettings::default()).unwrap();
    let mut prog = MyProgram {
        should_quit: false,
    };
    while !prog.should_quit {
        term.render_on_input(std::time::Duration::from_millis(31), |frame| {
            frame.render_with_context(&mut prog, frame.size())
        }).unwrap();
    }
    term.release().unwrap();
}

struct MyProgram {
    should_quit: false,
}

impl Program for MyProgram {
    fn render(&mut self, ctx: &mut Context, area: Rect, buf: &mut Buffer) {
        if let Some(input) = ctx.take_last_input() {
            match input {
                Input::KeyDown(KeyCode::Char('q'), _) => {
                    self.should_quit = true;
                }
                _ => {}
            }
        }
        let (top_area, bottom_area) = area
            .inner_centered(31, 4)
            .vsplit_len(3);
        if ctx.left_clicked(&top_area) {
            self.should_quit = true;
            return;
        }
        let block_style = if ctx.hovered(&top_area) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };
        Block::bordered().style(block_style).render(top_area, buf);
        Label::styled("Hover Me!", block_style).render(top_area.inner(Margin::new(1, 1)), buf);
        Label::styled(
            "press [q] or click here to quit", 
            Style::default().fg(Color::DarkGray),
        ).render(bottom_area, buf);
    }
}

```

## Overview

### Design Philosophy

Dreg started out as a fork of the [`ratatui`] crate (the successor to [`tui-rs`]) after I realized how bloated it had become.

The design of Dreg has been radical simplicity from the very start.

## Acknowledgments

- [`ratatui`] & [`tui-rs`], for the original inspiration for the project.

## License


[`ratatui`]: https://docs.rs/ratatui/latest/ratatui/
[`tui-rs`]: https://docs.rs/tui/latest/tui/
