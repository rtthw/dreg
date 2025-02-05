//! Terminal Platform
//!
//! Currently, dreg uses crossterm for its terminal implementation.



use crossterm::ExecutableCommand as _;

use crate::{Buffer, Frame, Program};



/// Run a dreg program inside a terminal emulator.
pub struct TerminalPlatform {
    /// Holds the results of the current and previous render calls. The two are compared at the end
    /// of each render pass to output only the necessary updates to the terminal.
    buffers: [Buffer; 2],
    /// The index of the current buffer in the previous array.
    current: usize,
}

impl super::Platform for TerminalPlatform {
    fn run(mut self, mut program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        bind_terminal()?;

        'main_loop: loop {
            if crossterm::event::poll(std::time::Duration::from_millis(31))? {
                match crossterm::event::read()? {
                    _ => {}
                }
            }
            let (cols, rows) = crossterm::terminal::size()?;

            let mut frame = Frame {
                cols,
                rows,
                buffer: &mut self.buffers[self.current],
                should_exit: false,
            };

            program.render(&mut frame);

            if frame.should_exit {
                break 'main_loop;
            }

            self.flush()?;
            self.swap_buffers();
        }

        release_terminal()?;

        Ok(())
    }
}

impl TerminalPlatform {
    /// Clear the inactive buffer and swap it with the current buffer.
    fn swap_buffers(&mut self) {
        self.buffers[1 - self.current].clear();
        self.current = 1 - self.current;
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}



fn bind_terminal() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    writer.execute(crossterm::event::EnableMouseCapture)?;
    writer.execute(crossterm::event::EnableFocusChange)?;
    writer.execute(crossterm::terminal::EnterAlternateScreen)?;
    writer.execute(crossterm::event::PushKeyboardEnhancementFlags(
        crossterm::event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | crossterm::event::KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
    ))?;
    writer.execute(crossterm::cursor::Hide)?;
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        release_terminal().unwrap();
        original_hook(panic);
    }));

    Ok(())
}

fn release_terminal() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = std::io::stdout();
    crossterm::terminal::disable_raw_mode()?;
    writer.execute(crossterm::event::DisableMouseCapture)?;
    writer.execute(crossterm::event::DisableFocusChange)?;
    writer.execute(crossterm::terminal::LeaveAlternateScreen)?;
    writer.execute(crossterm::event::PopKeyboardEnhancementFlags)?;
    writer.execute(crossterm::cursor::Show)?;

    Ok(())
}
