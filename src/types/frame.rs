//! Frame type



use super::{Area, Buffer, Command};



pub struct Frame<'a> {
    /// The width of the frame, in cells.
    pub cols: u16,
    /// The height of the frame, in cells.
    pub rows: u16,
    /// The frame's [`Buffer`].
    pub buffer: &'a mut Buffer,
    /// A set of [`Command`]s to be processed at the end of this frame.
    pub commands: &'a mut Vec<Command>,
    pub cursor: Option<(u16, u16)>,
    /// Flag to indicate whether the platform should safely exit at the end of this frame.
    pub should_exit: bool,
}

impl<'a> Frame<'a> {
    /// Get this frame's [`Area`].
    pub fn area(&self) -> Area {
        self.buffer.area
    }
}
