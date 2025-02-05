//! Terminal Platform
//!
//! Currently, dreg uses crossterm for its terminal implementation.



use crossterm::ExecutableCommand as _;

use crate::Program;



/// Run a dreg program inside a terminal emulator.
pub struct TerminalPlatform;

impl super::Platform for TerminalPlatform {
    fn run(self, program: impl Program) -> Result<(), Box<dyn std::error::Error>> {
        bind_terminal()?;

        // TODO: Main loop.

        release_terminal()?;

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
