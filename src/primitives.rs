//! Primitive Types



// ================================================================================================



#[derive(Clone, Copy)]
pub struct Pos(pub u16, pub u16);

impl Pos {
    #[inline(always)]
    pub const fn new(x: u16, y: u16) -> Self {
        Self(x, y)
    }

    #[inline(always)]
    pub const fn x(&self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub const fn y(&self) -> u16 {
        self.1
    }

    #[inline(always)]
    pub const fn col(&self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub const fn row(&self) -> u16 {
        self.1
    }
}



// ================================================================================================



/// A rectangular area.
///
/// A simple rectangle used in the computation of the layout and to give widgets a hint about the
/// area they are supposed to render to.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect {
    /// The x coordinate of the top left corner of the `Rect`.
    pub x: u16,
    /// The y coordinate of the top left corner of the `Rect`.
    pub y: u16,
    /// The width of the `Rect`.
    pub width: u16,
    /// The height of the `Rect`.
    pub height: u16,
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}

impl Rect {
    /// A zero sized Rect at position 0,0
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    /// Creates a new `Rect`, with width and height limited to keep the area under max `u16`. If
    /// clipped, aspect ratio will be preserved.
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

    /// The area of the `Rect`. If the area is larger than the maximum value of `u16`, it will be
    /// clamped to `u16::MAX`.
    pub const fn area(self) -> u16 {
        self.width.saturating_mul(self.height)
    }

    /// Returns true if the `Rect` has no area.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Returns the left coordinate of the `Rect`.
    pub const fn left(self) -> u16 {
        self.x
    }

    /// Returns the right coordinate of the `Rect`. This is the first coordinate outside of the
    /// `Rect`.
    ///
    /// If the right coordinate is larger than the maximum value of u16, it will be clamped to
    /// `u16::MAX`.
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// Returns the top coordinate of the `Rect`.
    pub const fn top(self) -> u16 {
        self.y
    }

    /// Returns the bottom coordinate of the `Rect`. This is the first coordinate outside of the
    /// `Rect`.
    ///
    /// If the bottom coordinate is larger than the maximum value of u16, it will be clamped to
    /// `u16::MAX`.
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Returns a new `Rect` inside the current one, with the given margin on each side.
    ///
    /// If the margin is larger than the `Rect`, the returned `Rect` will have no area.
    #[must_use = "method returns the modified value"]
    pub const fn inner(self, margin: Margin) -> Self {
        let doubled_margin_horizontal = margin.horizontal.saturating_mul(2);
        let doubled_margin_vertical = margin.vertical.saturating_mul(2);

        if self.width < doubled_margin_horizontal || self.height < doubled_margin_vertical {
            Self::ZERO
        } else {
            Self {
                x: self.x.saturating_add(margin.horizontal),
                y: self.y.saturating_add(margin.vertical),
                width: self.width.saturating_sub(doubled_margin_horizontal),
                height: self.height.saturating_sub(doubled_margin_vertical),
            }
        }
    }

    /// Moves the `Rect` without modifying its size.
    ///
    /// Moves the `Rect` according to the given offset without modifying its [`width`](Rect::width)
    /// or [`height`](Rect::height).
    /// - Positive `x` moves the whole `Rect` to the right, negative to the left.
    /// - Positive `y` moves the whole `Rect` downward, negative upward.
    ///
    /// See [`Offset`] for details.
    #[must_use = "method returns the modified value"]
    pub fn offset(self, offset: Offset) -> Self {
        Self {
            x: i32::from(self.x)
                .saturating_add(offset.x())
                .clamp(0, i32::from(u16::MAX - self.width)) as u16,
            y: i32::from(self.y)
                .saturating_add(offset.y())
                .clamp(0, i32::from(u16::MAX - self.height)) as u16,
            ..self
        }
    }

    /// Returns a new `Rect` that contains both the current one and the given one.
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

    /// Returns a new `Rect` that is the intersection of the current one and the given one.
    ///
    /// If the two `Rect`s do not intersect, the returned `Rect` will have no area.
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

    /// Returns true if the two `Rect`s intersect.
    pub const fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Returns true if the given position is inside the `Rect`.
    ///
    /// The position is considered inside the `Rect` if it is on the `Rect`'s border.
    pub const fn contains(self, position: Pos) -> bool {
        position.x() >= self.x
            && position.x() < self.right()
            && position.y() >= self.y
            && position.y() < self.bottom()
    }

    /// Clamp this `Rect` to fit inside the other `Rect`.
    ///
    /// If the width or height of this `Rect` is larger than the other `Rect`, it will be clamped to
    /// the other `Rect`'s width or height.
    ///
    /// If the left or top coordinate of this `Rect` is smaller than the other `Rect`, it will be
    /// clamped to the other `Rect`'s left or top coordinate.
    ///
    /// If the right or bottom coordinate of this `Rect` is larger than the other `Rect`, it will be
    /// clamped to the other `Rect`'s right or bottom coordinate.
    ///
    /// This is different from [`Rect::intersection`] because it will move this `Rect` to fit inside
    /// the other `Rect`, while [`Rect::intersection`] instead would keep this `Rect`'s position and
    /// truncate its size to only that which is inside the other `Rect`.
    #[must_use = "method returns the modified value"]
    pub fn clamp(self, other: Self) -> Self {
        let width = self.width.min(other.width);
        let height = self.height.min(other.height);
        let x = self.x.clamp(other.x, other.right().saturating_sub(width));
        let y = self.y.clamp(other.y, other.bottom().saturating_sub(height));
        Self::new(x, y, width, height)
    }

    // /// Indents the x value of the `Rect` by a given `offset`.
    // pub(crate) const fn indent_x(self, offset: u16) -> Self {
    //     Self {
    //         x: self.x.saturating_add(offset),
    //         width: self.width.saturating_sub(offset),
    //         ..self
    //     }
    // }
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
            Rect::new(self.width - length, self.y, length, self.height),
            Rect::new(self.x, self.y, self.width - length, self.height),
        )
    }

    pub fn vsplit_inverse_len(&self, length: u16) -> (Self, Self) {
        if length >= self.height {
            return (Rect::ZERO, *self);
        }
        (
            Rect::new(self.x, self.height - length, self.width, length),
            Rect::new(self.x, self.y, self.width, self.height - length),
        )
    }

    // pub fn split_h<const N: usize>(&self, portions: [u16; N]) -> [Self; N] {
    //     if portions.len() > self.width as usize {
    //     }
    // }

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
}



// ================================================================================================



#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Margin {
    pub horizontal: u16,
    pub vertical: u16,
}

impl Margin {
    pub const fn new(horizontal: u16, vertical: u16) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

impl std::fmt::Display for Margin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.horizontal, self.vertical)
    }
}



// ================================================================================================



#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Padding {
    pub l: u16,
    pub r: u16,
    pub t: u16,
    pub b: u16,
}

impl Padding {
    /// `Padding` with all fields set to `0`.
    pub const ZERO: Self = Self {
        l: 0,
        r: 0,
        t: 0,
        b: 0,
    };
}



// ================================================================================================



#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Offset(i32, i32);

impl Offset {
    #[inline(always)]
    pub const fn x(&self) -> i32 {
        self.0
    }

    #[inline(always)]
    pub const fn y(&self) -> i32 {
        self.1
    }

    #[inline(always)]
    pub const fn col(&self) -> i32 {
        self.0
    }

    #[inline(always)]
    pub const fn row(&self) -> i32 {
        self.1
    }
}



// ================================================================================================


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
}
