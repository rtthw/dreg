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

impl Area {
    /// Horizontally split this area at the given length.
    pub fn hsplit_len(&self, len: u16) -> (Self, Self) {
        (
            Self { x: self.x, y: self.y, w: len, h: self.h },
            Self { x: len, y: self.y, w: self.w.saturating_sub(len), h: self.h },
        )
    }

    /// Horizontally split this area at the given portion (from 0.0 to 1.0).
    pub fn hsplit_portion(&self, portion: f32) -> (Self, Self) {
        let len = (self.w as f32 * portion) as u16;
        self.hsplit_len(len)
    }

    /// Vertically split this area at the given length.
    pub fn vsplit_len(&self, len: u16) -> (Self, Self) {
        (
            Self { x: self.x, y: self.y, w: self.w, h: len },
            Self { x: self.x, y: len, w: self.w, h: self.h.saturating_sub(len) },
        )
    }

    /// Vertically split this area at the given portion (from 0.0 to 1.0).
    pub fn vsplit_portion(&self, portion: f32) -> (Self, Self) {
        let len = (self.w as f32 * portion) as u16;
        self.hsplit_len(len)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_containment() {
        let area = Area { x: 0, y: 0, w: 5, h: 7 };
        assert!(area.contains(0, 0));

        assert!(area.contains(0, 6));
        assert!(area.contains(4, 6));
        assert!(area.contains(4, 0));

        assert!(!area.contains(0, 7));
        assert!(!area.contains(5, 7));
        assert!(!area.contains(5, 0));
    }

    #[test]
    fn area_splitting() {
        let main_area = Area { x: 0, y: 0, w: 5, h: 7 };

        let (left_area, right_area) = main_area.hsplit_len(3);

        assert_eq!(left_area, Area { x: 0, y: 0, w: 3, h: 7 });
        assert_eq!(right_area, Area { x: 3, y: 0, w: 2, h: 7 });

        let (top_area, bottom_area) = main_area.vsplit_len(3);

        assert_eq!(top_area, Area { x: 0, y: 0, w: 5, h: 3 });
        assert_eq!(bottom_area, Area { x: 0, y: 3, w: 5, h: 4 });

        let (left_area, right_area) = main_area.hsplit_portion(0.6);

        assert_eq!(left_area, Area { x: 0, y: 0, w: 3, h: 7 });
        assert_eq!(right_area, Area { x: 3, y: 0, w: 2, h: 7 });
    }
}
