//! Primitive Types



/// A rectangular area.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Rect {
    /// The x coordinate of the top left corner of this rect.
    pub x: u16,
    /// The y coordinate of the top left corner of this rect.
    pub y: u16,
    /// The width of this rect.
    pub width: u16,
    /// The height of this rect.
    pub height: u16,
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}

impl Rect {
    /// A zero-sized rect at position (0,0).
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    /// Create a new rect, with width and height limited to keep the area under max `u16`. If
    /// clipped, its aspect ratio will be preserved.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        let max_area = u16::MAX;
        let (clipped_width, clipped_height) =
            if u32::from(width) * u32::from(height) > u32::from(max_area) {
                let aspect_ratio = f64::from(width) / f64::from(height);
                let max_area_f = f64::from(max_area);
                let height_f = (max_area_f / aspect_ratio).sqrt();
                let width_f = height_f * aspect_ratio;
                (width_f as u16, height_f as u16)
            } else {
                (width, height)
            };

        Self {
            x,
            y,
            width: clipped_width,
            height: clipped_height,
        }
    }

    /// The area of this rect. If the area is larger than the maximum value of `u16`, it will be
    /// clamped to `u16::MAX`.
    pub const fn area(self) -> u16 {
        self.width.saturating_mul(self.height)
    }

    /// Whether this rect has no area.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Get the left coordinate of this rect.
    pub const fn left(self) -> u16 {
        self.x
    }

    /// Get the right coordinate of this rect. This is the first column outside the rect's area.
    ///
    /// If the right coordinate is larger than the maximum value of u16, it will be clamped to
    /// `u16::MAX`.
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Get the top coordinate of this rect.
    pub const fn top(self) -> u16 {
        self.y
    }

    /// Get the bottom coordinate of this rect. This is the first row outside the rect's area.
    ///
    /// If the bottom coordinate is larger than the maximum value of u16, it will be clamped to
    /// `u16::MAX`.
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Get a new rect inside the current one, with the given margin on each side.
    ///
    /// If the margin is larger than the `Rect`, the returned `Rect` will have no area.
    #[must_use = "method returns the modified value"]
    pub const fn inner(self, margin_x: u16, margin_y: u16) -> Self {
        let doubled_margin_horizontal = margin_x.saturating_mul(2);
        let doubled_margin_vertical = margin_y.saturating_mul(2);

        if self.width < doubled_margin_horizontal || self.height < doubled_margin_vertical {
            Self::ZERO
        } else {
            Self {
                x: self.x.saturating_add(margin_x),
                y: self.y.saturating_add(margin_y),
                width: self.width.saturating_sub(doubled_margin_horizontal),
                height: self.height.saturating_sub(doubled_margin_vertical),
            }
        }
    }

    /// Move this rect without modifying its size.
    ///
    /// Moves the `Rect` according to the given offset without modifying its [`width`](Rect::width)
    /// or [`height`](Rect::height).
    /// - Positive `x` moves the whole `Rect` to the right, negative to the left.
    /// - Positive `y` moves the whole `Rect` downward, negative upward.
    #[must_use = "method returns the modified value"]
    pub fn offset(self, x: i32, y: i32) -> Self {
        Self {
            x: i32::from(self.x)
                .saturating_add(x)
                .clamp(0, i32::from(u16::MAX - self.width)) as u16,
            y: i32::from(self.y)
                .saturating_add(y)
                .clamp(0, i32::from(u16::MAX - self.height)) as u16,
            ..self
        }
    }

    /// Get a new rect that contains both the current one and the given one.
    #[must_use = "method returns the modified value"]
    pub fn union(self, other: Self) -> Self {
        let x1 = std::cmp::min(self.x, other.x);
        let y1 = std::cmp::min(self.y, other.y);
        let x2 = std::cmp::max(self.right(), other.right());
        let y2 = std::cmp::max(self.bottom(), other.bottom());
        Self {
            x: x1,
            y: y1,
            width: x2.saturating_sub(x1),
            height: y2.saturating_sub(y1),
        }
    }

    /// Get a new rect that is the intersection of the current one and the given one.
    ///
    /// If the two rects do not intersect, the returned rect will have no area.
    #[must_use = "method returns the modified value"]
    pub fn intersection(self, other: Self) -> Self {
        let x1 = std::cmp::max(self.x, other.x);
        let y1 = std::cmp::max(self.y, other.y);
        let x2 = std::cmp::min(self.right(), other.right());
        let y2 = std::cmp::min(self.bottom(), other.bottom());
        Self {
            x: x1,
            y: y1,
            width: x2.saturating_sub(x1),
            height: y2.saturating_sub(y1),
        }
    }

    /// Whether this rect and the given `other` one intersect one another.
    pub const fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Whether the given position is inside this rect.
    ///
    /// The position is considered inside this rect if it is on the border.
    pub const fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x
            && x < self.right()
            && y >= self.y
            && y < self.bottom()
    }

    /// Clamp this rect to fit inside `other`.
    ///
    /// If the width or height of this rect is larger than the other one, it will be clamped to
    /// the other rect's width or height.
    ///
    /// If the left or top coordinate of this rect is smaller than the other one, it will be
    /// clamped to the other rect's left or top coordinate.
    ///
    /// If the right or bottom coordinate of this rect is larger than the other one, it will be
    /// clamped to the other rect's right or bottom coordinate.
    ///
    /// This is different from [`Rect::intersection`] because it will move this rect to fit
    /// inside the other rect, while [`Rect::intersection`] instead would keep this rect's
    /// position and truncate its size to only that which is inside the other rect.
    #[must_use = "method returns the modified value"]
    pub fn clamp(self, other: Self) -> Self {
        let width = self.width.min(other.width);
        let height = self.height.min(other.height);
        let x = self.x.clamp(other.x, other.right().saturating_sub(width));
        let y = self.y.clamp(other.y, other.bottom().saturating_sub(height));
        Self::new(x, y, width, height)
    }
}

impl Rect {
    pub fn hsplit_portion(&self, portion: f32) -> (Self, Self) {
        let width_a = (self.width as f32 * portion).floor() as u16;
        let width_b = self.width - width_a;
        (
            Rect::new(self.x, self.y, width_a, self.height),
            Rect::new(self.x + width_a, self.y, width_b, self.height),
        )
    }

    pub fn vsplit_portion(&self, portion: f32) -> (Self, Self) {
        let height_a = (self.height as f32 * portion).floor() as u16;
        let height_b = self.height - height_a;
        (
            Rect::new(self.x, self.y, self.width, height_a),
            Rect::new(self.x, self.y + height_a, self.width, height_b),
        )
    }

    pub fn hsplit_len(&self, length: u16) -> (Self, Self) {
        if length >= self.width {
            return (*self, Rect::ZERO);
        }
        (
            Rect::new(self.x, self.y, length, self.height),
            Rect::new(self.x + length, self.y, self.width - length, self.height),
        )
    }

    pub fn vsplit_len(&self, length: u16) -> (Self, Self) {
        if length >= self.height {
            return (*self, Rect::ZERO);
        }
        (
            Rect::new(self.x, self.y, self.width, length),
            Rect::new(self.x, self.y + length, self.width, self.height - length),
        )
    }

    pub fn hsplit_inverse_portion(&self, portion: f32) -> (Self, Self) {
        let width_a = (self.width as f32 * portion).floor() as u16;
        let width_b = self.width - width_a;
        (
            Rect::new(self.x + width_b, self.y, width_a, self.height),
            Rect::new(self.x, self.y, width_b, self.height),
        )
    }

    pub fn vsplit_inverse_portion(&self, portion: f32) -> (Self, Self) {
        let height_a = (self.height as f32 * portion).floor() as u16;
        let height_b = self.height - height_a;
        (
            Rect::new(self.x, self.y + height_b, self.width, height_a),
            Rect::new(self.x, self.y, self.width, height_b),
        )
    }

    pub fn hsplit_inverse_len(&self, length: u16) -> (Self, Self) {
        if length >= self.width {
            return (Rect::ZERO, *self);
        }
        (
            Rect::new(self.x + (self.width - length), self.y, length, self.height),
            Rect::new(self.x, self.y, self.width - length, self.height),
        )
    }

    pub fn vsplit_inverse_len(&self, length: u16) -> (Self, Self) {
        if length >= self.height {
            return (Rect::ZERO, *self);
        }
        (
            Rect::new(self.x, self.y + (self.height - length), self.width, length),
            Rect::new(self.x, self.y, self.width, self.height - length),
        )
    }

    pub fn inner_centered(&self, width: u16, height: u16) -> Self {
        let x = self.x + (self.width.saturating_sub(width) / 2);
        let y = self.y + (self.height.saturating_sub(height) / 2);
        Rect::new(x, y, width.min(self.width), height.min(self.height))
    }

    pub fn rows(&self) -> Vec<Self> {
        (0..self.height)
            .into_iter()
            .map(|row_index| {
                Rect::new(self.left(), self.top() + row_index, self.width, self.height)
            })
            .collect()
    }

    /// Split this rect evenly into 3 rows.
    pub fn vsplit_even3(&self) -> (Self, Self, Self) {
        if self.height < 3 {
            return (Rect::ZERO, Rect::ZERO, *self);
        }

        let main_height = self.height / 3;
        let first_2_rows_height = main_height * 2;
        let alt_height = self.height - first_2_rows_height;

        (
            Rect::new(self.x, self.y, self.width, main_height),
            Rect::new(self.x, self.y + main_height, self.width, main_height),
            Rect::new(self.x, self.y + first_2_rows_height, self.width, alt_height),
        )
    }

    /// Split this rect evenly into 3 columns.
    pub fn hsplit_even3(&self) -> (Self, Self, Self) {
        if self.width < 3 {
            return (Rect::ZERO, Rect::ZERO, *self);
        }

        let main_width = self.width / 3;
        let first_2_cols_width = main_width * 2;
        let alt_width = self.width - first_2_cols_width;

        (
            Rect::new(self.x, self.y, main_width, self.height),
            Rect::new(self.x + main_width, self.y, main_width, self.height),
            Rect::new(self.x + first_2_cols_width, self.y, alt_width, self.height),
        )
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_splitting() {
        let test_rect = Rect::new(0, 0, 10, 20);

        assert_eq!(test_rect.hsplit_len(3), (Rect::new(0, 0, 3, 20), Rect::new(3, 0, 7, 20)));
        assert_eq!(test_rect.hsplit_len(11), (Rect::new(0, 0, 10, 20), Rect::new(0, 0, 0, 0)));
        assert_eq!(test_rect.vsplit_len(7), (Rect::new(0, 0, 10, 7), Rect::new(0, 7, 10, 13)));
        assert_eq!(test_rect.vsplit_len(22), (Rect::new(0, 0, 10, 20), Rect::new(0, 0, 0, 0)));

        assert_eq!(
            test_rect.hsplit_inverse_len(3),
            (Rect::new(7, 0, 3, 20), Rect::new(0, 0, 7, 20)),
        );
    }

    #[test]
    fn rect_splitting_with_offset() {
        let test_rect = Rect::new(1, 1, 10, 20);

        assert_eq!(test_rect.hsplit_len(3), (Rect::new(1, 1, 3, 20), Rect::new(4, 1, 7, 20)));
        assert_eq!(test_rect.hsplit_len(11), (Rect::new(1, 1, 10, 20), Rect::new(0, 0, 0, 0)));
        assert_eq!(test_rect.vsplit_len(7), (Rect::new(1, 1, 10, 7), Rect::new(1, 8, 10, 13)));
        assert_eq!(test_rect.vsplit_len(22), (Rect::new(1, 1, 10, 20), Rect::new(0, 0, 0, 0)));

        assert_eq!(
            test_rect.hsplit_inverse_len(3),
            (Rect::new(8, 1, 3, 20), Rect::new(1, 1, 7, 20)),
        );
    }

    #[test]
    fn fixed_issue4() {
        let rect = Rect::new(3, 5, 7, 11);

        assert_eq!(
            rect.hsplit_inverse_len(2),
            (
                Rect::new(8, 5, 2, 11),
                Rect::new(3, 5, 5, 11),
            )
        )
    }

    #[test]
    fn rects_split_evenly() {
        let rect = Rect::new(0, 0, 5, 7);
        assert_eq!(
            rect.vsplit_even3(),
            (
                Rect::new(0, 0, 5, 2),
                Rect::new(0, 2, 5, 2),
                Rect::new(0, 4, 5, 3),
            ),
        );
        assert_eq!(
            rect.hsplit_even3(),
            (
                Rect::new(0, 0, 1, 7),
                Rect::new(1, 0, 1, 7),
                Rect::new(2, 0, 3, 7),
            ),
        );

        let rect = Rect::new(0, 0, 6, 9);
        assert_eq!(
            rect.vsplit_even3(),
            (
                Rect::new(0, 0, 6, 3),
                Rect::new(0, 3, 6, 3),
                Rect::new(0, 6, 6, 3),
            ),
        );
        assert_eq!(
            rect.hsplit_even3(),
            (
                Rect::new(0, 0, 2, 9),
                Rect::new(2, 0, 2, 9),
                Rect::new(4, 0, 2, 9),
            ),
        );
    }
}
