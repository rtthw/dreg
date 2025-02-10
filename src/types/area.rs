//! Area



use super::InputContext;



#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Area {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

// Utilities.
impl Area {
    /// Returns true if the given coordinates are within this area.
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.w)
            && y < self.y.saturating_add(self.h)
    }
}

// Input.
impl Area {
    /// Akin to `input_context.left_clicked(&self)`.
    pub fn hovered(&self, input_context: &InputContext) -> bool {
        input_context.hovered(self)
    }

    /// Akin to `input_context.left_clicked(&self)`.
    pub fn left_clicked(&self, input_context: &InputContext) -> bool {
        input_context.left_clicked(self)
    }

    /// Akin to `input_context.right_clicked(&self)`.
    pub fn right_clicked(&self, input_context: &InputContext) -> bool {
        input_context.right_clicked(self)
    }

    /// Akin to `input_context.middle_clicked(&self)`.
    pub fn middle_clicked(&self, input_context: &InputContext) -> bool {
        input_context.middle_clicked(self)
    }
}
