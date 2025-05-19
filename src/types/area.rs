//! Area



use super::InputContext;



/// An area is a portion of the screen.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Area {
    /// This area's leftmost column index.
    pub x: u16,
    /// This area's topmost row index.
    pub y: u16,
    /// The total number of columns this area covers.
    pub w: u16,
    /// The total number of rows this area covers.
    pub h: u16,
}

// Constructors.
impl Area {
    /// An area with no size, at the origin.
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    /// Create a new area with the given values.
    pub const fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }
}

// Utilities.
impl Area {
    /// Whether this area is empty.
    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub const fn left(self) -> u16 {
        self.x
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.w)
    }

    pub const fn top(self) -> u16 {
        self.y
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.h)
    }

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
    /// Create a new area centered inside this one that has been shrunken by the given margins.
    pub const fn shrink(self, margin_x: u16, margin_y: u16) -> Self {
        let doubled_margin_horizontal = margin_x.saturating_mul(2);
        let doubled_margin_vertical = margin_y.saturating_mul(2);

        if self.w < doubled_margin_horizontal || self.h < doubled_margin_vertical {
            Self::ZERO
        } else {
            Self {
                x: self.x.saturating_add(margin_x),
                y: self.y.saturating_add(margin_y),
                w: self.w.saturating_sub(doubled_margin_horizontal),
                h: self.h.saturating_sub(doubled_margin_vertical),
            }
        }
    }

    /// Create a new area centered inside this one with the given width and height.
    pub fn inner_centered(&self, width: u16, height: u16) -> Self {
        let x = self.x + (self.w.saturating_sub(width) / 2);
        let y = self.y + (self.h.saturating_sub(height) / 2);
        Self::new(x, y, width.min(self.w), height.min(self.h))
    }

    /// Horizontally split this area at the given length.
    pub fn hsplit_len(&self, len: u16) -> (Self, Self) {
        (
            Self { x: self.x, y: self.y, w: len, h: self.h },
            Self { x: self.x + len, y: self.y, w: self.w.saturating_sub(len), h: self.h },
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
            Self { x: self.x, y: self.y + len, w: self.w, h: self.h.saturating_sub(len) },
        )
    }

    /// Vertically split this area at the given portion (from 0.0 to 1.0).
    pub fn vsplit_portion(&self, portion: f32) -> (Self, Self) {
        let len = (self.w as f32 * portion) as u16;
        self.hsplit_len(len)
    }

    /// Get a set of 1-height areas that will fit into this one's rows.
    pub fn rows(&self) -> Vec<Self> {
        (0..self.h)
            .into_iter()
            .map(|row_index| {
                Self::new(self.x, self.y + row_index, self.w, self.h)
            })
            .collect()
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
