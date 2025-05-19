//! Command type




#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    SetTitle(String),
    SetCursorStyle(CursorStyle),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CursorStyle {
    SteadyBlock,
    SteadyBar,
    SteadyUnderline,
    BlinkingBlock,
    BlinkingBar,
    BlinkingUnderline,
}
