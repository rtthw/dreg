//! Context



pub use crossterm::event::{KeyCode, KeyModifiers, MouseButton};

use crate::prelude::*;



// ================================================================================================



pub struct Context {
    pub force_render_next_frame: bool,

    last_input: Option<Input>,

    #[cfg(feature = "anim")]
    running_animations: Vec<(Animation, Rect)>,

    last_mouse_pos: Option<Pos>,
    lmb_down_at: Option<Pos>,
    rmb_down_at: Option<Pos>,
    mmb_down_at: Option<Pos>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            force_render_next_frame: false,
            last_input: None,
            #[cfg(feature = "anim")]
            running_animations: vec![],
            last_mouse_pos: None,
            lmb_down_at: None,
            rmb_down_at: None,
            mmb_down_at: None,
        }
    }
}

impl Context {
    pub fn take_last_input(&mut self) -> Option<Input> {
        self.last_input.take()
    }

    pub (crate) fn handle_input(&mut self, input: Input) {
        self.last_input = Some(input);
        match input {
            Input::MouseMove(pos, _mods) => {
                self.last_mouse_pos = Some(pos);
            }
            Input::MouseDown(pos, btn) => {
                match btn {
                    MouseButton::Left => self.lmb_down_at = Some(pos),
                    MouseButton::Right => self.rmb_down_at = Some(pos),
                    MouseButton::Middle => self.mmb_down_at = Some(pos),
                }
            }
            Input::MouseUp(_pos, btn) => {
                match btn {
                    MouseButton::Left => self.lmb_down_at = None,
                    MouseButton::Right => self.rmb_down_at = None,
                    MouseButton::Middle => self.mmb_down_at = None,
                }
            }
            _ => {}
        }
    }
}

impl Context {
    pub fn hovered(&self, area: &Rect) -> bool {
        self.last_mouse_pos.is_some_and(|p| area.contains(p))
    }

    pub fn left_clicked(&self, area: &Rect) -> bool {
        self.lmb_down_at.is_some_and(|pos| area.contains(pos))
    }

    pub fn right_clicked(&self, area: &Rect) -> bool {
        self.rmb_down_at.is_some_and(|pos| area.contains(pos))
    }

    pub fn middle_clicked(&self, area: &Rect) -> bool {
        self.mmb_down_at.is_some_and(|pos| area.contains(pos))
    }
}

#[cfg(feature = "anim")]
impl Context {
    pub fn animating(&self) -> bool {
        !self.running_animations.is_empty()
    }

    pub fn take_animations(&mut self) -> Vec<(Animation, Rect)> {
        self.running_animations.drain(..).collect()
    }

    pub fn place_animations(&mut self, anims: Vec<(Animation, Rect)>) {
        self.running_animations = anims;
    }

    pub fn start_animation(&mut self, (anim, area): (Animation, Rect)) {
        self.running_animations.push((anim, area))
    }
}

#[derive(Clone, Copy)]
pub enum Input {
    KeyDown(KeyCode, KeyModifiers),
    KeyUp(KeyCode, KeyModifiers),

    MouseMove(Pos, KeyModifiers),
    MouseDown(Pos, MouseButton),
    MouseUp(Pos, MouseButton),
    MouseDrag(Pos, MouseButton),

    ScrollUp,
    ScrollDown,

    Null,
}

impl From<crossterm::event::Event> for Input {
    fn from(value: crossterm::event::Event) -> Self {
        match value {
            crossterm::event::Event::Key(event) => {
                let crossterm::event::KeyEvent {
                    code, 
                    modifiers: mods, 
                    kind, 
                    ..
                } = event;
                match kind {
                    crossterm::event::KeyEventKind::Press => Input::KeyDown(code, mods),
                    crossterm::event::KeyEventKind::Release => Input::KeyUp(code, mods),
                    _ => Input::Null,
                }
            }
            crossterm::event::Event::Mouse(event) => {
                let crossterm::event::MouseEvent {
                    kind,
                    column: x,
                    row: y,
                    modifiers: mods,
                } = event;
                match kind {
                    crossterm::event::MouseEventKind::Moved => Input::MouseMove(Pos(x, y), mods),
                    crossterm::event::MouseEventKind::Down(btn) => Input::MouseDown(Pos(x, y), btn),
                    crossterm::event::MouseEventKind::Up(btn) => Input::MouseUp(Pos(x, y), btn),
                    crossterm::event::MouseEventKind::Drag(btn) => Input::MouseDrag(Pos(x, y), btn),
                    crossterm::event::MouseEventKind::ScrollUp => Input::ScrollUp,
                    crossterm::event::MouseEventKind::ScrollDown => Input::ScrollDown,
                    _ => Input::Null,
                }
            }
            _ => Input::Null,
        }
    }
}



// ================================================================================================
