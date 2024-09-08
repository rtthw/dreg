//! Line Shape Element



use crate::prelude::*;



// ================================================================================================



pub struct Line {
    pub ty: LineType,
    pub direction: LineDirection,
    pub capping: LineCapping,
}

impl Element for Line {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        match self.direction {
            LineDirection::Horizontal => {
                let mut rect = area;
                if area.height == 0 {
                    return;
                } else if area.height > 1 {
                    rect = area.inner_centered(area.width, 1);
                }

                let symbol = match self.ty {
                    LineType::Normal => crate::block::HORIZONTAL,
                    LineType::Thick => crate::block::THICK_HORIZONTAL,
                };

                for x in (rect.left() + 1)..rect.right() {
                    buf.get_mut(x, rect.y).set_symbol(symbol);
                }

                match self.capping {
                    LineCapping::None => {
                        buf.get_mut(rect.left(), rect.y).set_symbol(symbol);
                        buf.get_mut(rect.right() - 1, rect.y).set_symbol(symbol);
                    }
                    LineCapping::Shortened => {
                        let (symbol_left, symbol_right) = match self.ty {
                            LineType::Normal => ("╶", "╴"),
                            LineType::Thick => ("╺", "╸"),
                        };
                        buf.get_mut(rect.left(), rect.y).set_symbol(symbol_left);
                        buf.get_mut(rect.right() - 1, rect.y).set_symbol(symbol_right);
                    }
                    LineCapping::Switched => {
                        let (symbol_left, symbol_right) = match self.ty {
                            LineType::Normal => ("╾", "╼"),
                            LineType::Thick => ("╼", "╾"),
                        };
                        buf.get_mut(rect.left(), rect.y).set_symbol(symbol_left);
                        buf.get_mut(rect.right() - 1, rect.y).set_symbol(symbol_right);
                    }
                }
            }
            LineDirection::Vertical => {
                let mut rect = area;
                if area.width == 0 {
                    return;
                } else if area.width > 1 {
                    rect = area.inner_centered(1, area.height);
                }

                let symbol = match self.ty {
                    LineType::Normal => crate::block::VERTICAL,
                    LineType::Thick => crate::block::THICK_VERTICAL,
                };

                for y in rect.top()..rect.bottom() {
                    buf.get_mut(rect.x, y).set_symbol(symbol);
                }

                match self.capping {
                    LineCapping::None => {
                        buf.get_mut(rect.x, rect.top()).set_symbol(symbol);
                        buf.get_mut(rect.x, rect.bottom() - 1).set_symbol(symbol);
                    }
                    LineCapping::Shortened => {
                        let (symbol_top, symbol_bottom) = match self.ty {
                            LineType::Normal => ("╷", "╵"),
                            LineType::Thick => ("╻", "╹"),
                        };
                        buf.get_mut(rect.x, rect.top()).set_symbol(symbol_top);
                        buf.get_mut(rect.x, rect.bottom() - 1).set_symbol(symbol_bottom);
                    }
                    LineCapping::Switched => {
                        let (symbol_top, symbol_bottom) = match self.ty {
                            LineType::Normal => ("╿", "╽"),
                            LineType::Thick => ("╽", "╿"),
                        };
                        buf.get_mut(rect.x, rect.top()).set_symbol(symbol_top);
                        buf.get_mut(rect.x, rect.bottom() - 1).set_symbol(symbol_bottom);
                    }
                }
            }
        }
    }
}

// Builder.
impl Line {
    pub fn horizontal() -> Self {
        Self {
            ty: LineType::default(),
            direction: LineDirection::Horizontal,
            capping: LineCapping::default(),
        }
    }
    pub fn vertical() -> Self {
        Self {
            ty: LineType::default(),
            direction: LineDirection::Vertical,
            capping: LineCapping::default(),
        }
    }

    pub fn thick(self) -> Self {
        Self {
            ty: LineType::Thick,
            ..self
        }
    }

    pub fn shortened(self) -> Self {
        Self {
            capping: LineCapping::Shortened,
            ..self
        }
    }

    pub fn switched(self) -> Self {
        Self {
            capping: LineCapping::Switched,
            ..self
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum LineDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Default)]
pub enum LineType {
    #[default]
    Normal,
    Thick,
}

#[derive(Default)]
pub enum LineCapping {
    /// Line will stop at the edge of the cell boundary.
    #[default]
    None,
    /// Line will stop before the edge of the cell boundary.
    Shortened,
    /// Line will change to the other line type.
    Switched,
}



// ================================================================================================
