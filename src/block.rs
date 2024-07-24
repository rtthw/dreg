//! Block



use crate::prelude::*;



// ================================================================================================



bitflags::bitflags! {
    /// Bitflags that can be composed to set the visible borders essentially on the block element.
    #[derive(Default, Clone, Copy, Eq, PartialEq, Hash)]
    pub struct Borders: u8 {
        /// Show no border (default)
        const NONE   = 0b0000;
        /// Show the top border
        const TOP    = 0b0001;
        /// Show the right border
        const RIGHT  = 0b0010;
        /// Show the bottom border
        const BOTTOM = 0b0100;
        /// Show the left border
        const LEFT   = 0b1000;
        /// Show all borders
        const ALL = Self::TOP.bits() | Self::RIGHT.bits() | Self::BOTTOM.bits() | Self::LEFT.bits();
    }
}

/// The type of border for a [`Block`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BorderType {
    /// A plain, simple border.
    ///
    /// This is the default
    ///
    /// # Example
    ///
    /// ```plain
    /// ┌───────┐
    /// │       │
    /// └───────┘
    /// ```
    #[default]
    Plain,
    /// A plain border with rounded corners.
    ///
    /// # Example
    ///
    /// ```plain
    /// ╭───────╮
    /// │       │
    /// ╰───────╯
    /// ```
    Rounded,
    /// A doubled border.
    ///
    /// Note this uses one character that draws two lines.
    ///
    /// # Example
    ///
    /// ```plain
    /// ╔═══════╗
    /// ║       ║
    /// ╚═══════╝
    /// ```
    Double,
    /// A thick border.
    ///
    /// # Example
    ///
    /// ```plain
    /// ┏━━━━━━━┓
    /// ┃       ┃
    /// ┗━━━━━━━┛
    /// ```
    Thick,
}

impl BorderType {
    /// Convert this `BorderType` into the corresponding `BorderSet` of border symbols.
    const fn border_symbols(border_type: Self) -> BorderSet {
        match border_type {
            Self::Plain => BORDERSET_PLAIN,
            Self::Rounded => BORDERSET_ROUNDED,
            Self::Double => BORDERSET_DOUBLE,
            Self::Thick => BORDERSET_THICK,
        }
    }

    /// Convert this `BorderType` into the corresponding `BorderSet` of border symbols.
    pub const fn to_border_set(self) -> BorderSet {
        Self::border_symbols(self)
    }
}

#[derive(Clone, Default, Eq, Hash, PartialEq)]
pub struct Block {
    style: Style,
    borders: Borders,
    border_style: Style,
    /// The symbols used to render the border. The default is plain lines but one can choose to
    /// have rounded or doubled lines in stead, or a custom set of symbols.
    border_set: BorderSet,
    padding: Padding,
}

impl Element for Block {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        buf.set_style(area, self.style);
        self.render_borders(area, buf);
        // self.render_titles(area, buf);
    }
}

impl Block {
    pub const fn new() -> Self {
        Self {
            // titles: Vec::new(),
            // titles_style: Style::new(),
            // titles_alignment: Alignment::Left,
            // titles_position: Position::Top,
            borders: Borders::NONE,
            border_style: Style::new(),
            border_set: BorderType::Plain.to_border_set(),
            style: Style::new(),
            padding: Padding::ZERO,
        }
    }

    pub const fn bordered() -> Self {
        let mut block = Self::new();
        block.borders = Borders::ALL;
        block
    }
    
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_set = border_type.to_border_set();
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
}

impl Block {
    // pub fn has_title_at_position(&self, position: Position) -> bool {
    //     self.titles
    //         .iter()
    //         .any(|title| title.position.unwrap_or(self.titles_position) == position)
    // }

    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;
        if self.borders.intersects(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1).min(inner.right());
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::TOP) { //|| self.has_title_at_position(Position::Top) {
            inner.y = inner.y.saturating_add(1).min(inner.bottom());
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::BOTTOM) {//|| self.has_title_at_position(Position::Bottom) {
            inner.height = inner.height.saturating_sub(1);
        }

        inner.x = inner.x.saturating_add(self.padding.l);
        inner.y = inner.y.saturating_add(self.padding.t);

        inner.width = inner
            .width
            .saturating_sub(self.padding.l + self.padding.r);
        inner.height = inner
            .height
            .saturating_sub(self.padding.t + self.padding.b);

        inner
    }
}

impl Block {
    fn render_borders(&self, area: Rect, buf: &mut Buffer) {
        self.render_left_side(area, buf);
        self.render_top_side(area, buf);
        self.render_right_side(area, buf);
        self.render_bottom_side(area, buf);

        self.render_bottom_right_corner(buf, area);
        self.render_top_right_corner(buf, area);
        self.render_bottom_left_corner(buf, area);
        self.render_top_left_corner(buf, area);
    }

    fn render_left_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::LEFT) {
            for y in area.top()..area.bottom() {
                buf.get_mut(area.left(), y)
                    .set_symbol(self.border_set.vertical_left)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_top_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::TOP) {
            for x in area.left()..area.right() {
                buf.get_mut(x, area.top())
                    .set_symbol(self.border_set.horizontal_top)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_right_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::RIGHT) {
            let x = area.right() - 1;
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(self.border_set.vertical_right)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_bottom_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::BOTTOM) {
            let y = area.bottom() - 1;
            for x in area.left()..area.right() {
                buf.get_mut(x, y)
                    .set_symbol(self.border_set.horizontal_bottom)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_bottom_right_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            buf.get_mut(area.right() - 1, area.bottom() - 1)
                .set_symbol(self.border_set.bottom_right)
                .set_style(self.border_style);
        }
    }

    fn render_top_right_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::RIGHT | Borders::TOP) {
            buf.get_mut(area.right() - 1, area.top())
                .set_symbol(self.border_set.top_right)
                .set_style(self.border_style);
        }
    }

    fn render_bottom_left_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::LEFT | Borders::BOTTOM) {
            buf.get_mut(area.left(), area.bottom() - 1)
                .set_symbol(self.border_set.bottom_left)
                .set_style(self.border_style);
        }
    }

    fn render_top_left_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::LEFT | Borders::TOP) {
            buf.get_mut(area.left(), area.top())
                .set_symbol(self.border_set.top_left)
                .set_style(self.border_style);
        }
    }
}



// ================================================================================================



#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Clear;

impl Element for Clear {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).reset();
            }
        }
    }
}



// ================================================================================================



#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct LineSet {
    pub vertical: &'static str,
    pub horizontal: &'static str,
    pub top_right: &'static str,
    pub top_left: &'static str,
    pub bottom_right: &'static str,
    pub bottom_left: &'static str,
    pub vertical_left: &'static str,
    pub vertical_right: &'static str,
    pub horizontal_down: &'static str,
    pub horizontal_up: &'static str,
    pub cross: &'static str,
}

impl Default for LineSet {
    fn default() -> Self {
        LINESET_NORMAL
    }
}

pub const LINESET_NORMAL: LineSet = LineSet {
    vertical: VERTICAL,
    horizontal: HORIZONTAL,
    top_right: TOP_RIGHT,
    top_left: TOP_LEFT,
    bottom_right: BOTTOM_RIGHT,
    bottom_left: BOTTOM_LEFT,
    vertical_left: VERTICAL_LEFT,
    vertical_right: VERTICAL_RIGHT,
    horizontal_down: HORIZONTAL_DOWN,
    horizontal_up: HORIZONTAL_UP,
    cross: CROSS,
};

pub const LINESET_ROUNDED: LineSet = LineSet {
    top_right: ROUNDED_TOP_RIGHT,
    top_left: ROUNDED_TOP_LEFT,
    bottom_right: ROUNDED_BOTTOM_RIGHT,
    bottom_left: ROUNDED_BOTTOM_LEFT,
    ..LINESET_NORMAL
};

pub const LINESET_DOUBLE: LineSet = LineSet {
    vertical: DOUBLE_VERTICAL,
    horizontal: DOUBLE_HORIZONTAL,
    top_right: DOUBLE_TOP_RIGHT,
    top_left: DOUBLE_TOP_LEFT,
    bottom_right: DOUBLE_BOTTOM_RIGHT,
    bottom_left: DOUBLE_BOTTOM_LEFT,
    vertical_left: DOUBLE_VERTICAL_LEFT,
    vertical_right: DOUBLE_VERTICAL_RIGHT,
    horizontal_down: DOUBLE_HORIZONTAL_DOWN,
    horizontal_up: DOUBLE_HORIZONTAL_UP,
    cross: DOUBLE_CROSS,
};

pub const LINESET_THICK: LineSet = LineSet {
    vertical: THICK_VERTICAL,
    horizontal: THICK_HORIZONTAL,
    top_right: THICK_TOP_RIGHT,
    top_left: THICK_TOP_LEFT,
    bottom_right: THICK_BOTTOM_RIGHT,
    bottom_left: THICK_BOTTOM_LEFT,
    vertical_left: THICK_VERTICAL_LEFT,
    vertical_right: THICK_VERTICAL_RIGHT,
    horizontal_down: THICK_HORIZONTAL_DOWN,
    horizontal_up: THICK_HORIZONTAL_UP,
    cross: THICK_CROSS,
};


// ================================================================================================



#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BorderSet {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub vertical_left: &'static str,
    pub vertical_right: &'static str,
    pub horizontal_top: &'static str,
    pub horizontal_bottom: &'static str,
}

impl Default for BorderSet {
    fn default() -> Self {
        BORDERSET_PLAIN
    }
}

/// Border set with a single line width
///
/// ```text
/// ┌─────┐
/// │xxxxx│
/// │xxxxx│
/// └─────┘
/// ```
pub const BORDERSET_PLAIN: BorderSet = BorderSet {
    top_left: LINESET_NORMAL.top_left,
    top_right: LINESET_NORMAL.top_right,
    bottom_left: LINESET_NORMAL.bottom_left,
    bottom_right: LINESET_NORMAL.bottom_right,
    vertical_left: LINESET_NORMAL.vertical,
    vertical_right: LINESET_NORMAL.vertical,
    horizontal_top: LINESET_NORMAL.horizontal,
    horizontal_bottom: LINESET_NORMAL.horizontal,
};

/// Border set with a single line width and rounded corners
///
/// ```text
/// ╭─────╮
/// │xxxxx│
/// │xxxxx│
/// ╰─────╯
/// ```
pub const BORDERSET_ROUNDED: BorderSet = BorderSet {
    top_left: LINESET_ROUNDED.top_left,
    top_right: LINESET_ROUNDED.top_right,
    bottom_left: LINESET_ROUNDED.bottom_left,
    bottom_right: LINESET_ROUNDED.bottom_right,
    vertical_left: LINESET_ROUNDED.vertical,
    vertical_right: LINESET_ROUNDED.vertical,
    horizontal_top: LINESET_ROUNDED.horizontal,
    horizontal_bottom: LINESET_ROUNDED.horizontal,
};

/// Border set with a double line width
///
/// ```text
/// ╔═════╗
/// ║xxxxx║
/// ║xxxxx║
/// ╚═════╝
/// ```
pub const BORDERSET_DOUBLE: BorderSet = BorderSet {
    top_left: LINESET_DOUBLE.top_left,
    top_right: LINESET_DOUBLE.top_right,
    bottom_left: LINESET_DOUBLE.bottom_left,
    bottom_right: LINESET_DOUBLE.bottom_right,
    vertical_left: LINESET_DOUBLE.vertical,
    vertical_right: LINESET_DOUBLE.vertical,
    horizontal_top: LINESET_DOUBLE.horizontal,
    horizontal_bottom: LINESET_DOUBLE.horizontal,
};

/// Border set with a thick line width
///
/// ```text
/// ┏━━━━━┓
/// ┃xxxxx┃
/// ┃xxxxx┃
/// ┗━━━━━┛
/// ```
pub const BORDERSET_THICK: BorderSet = BorderSet {
    top_left: LINESET_THICK.top_left,
    top_right: LINESET_THICK.top_right,
    bottom_left: LINESET_THICK.bottom_left,
    bottom_right: LINESET_THICK.bottom_right,
    vertical_left: LINESET_THICK.vertical,
    vertical_right: LINESET_THICK.vertical,
    horizontal_top: LINESET_THICK.horizontal,
    horizontal_bottom: LINESET_THICK.horizontal,
};

// ================================================================================================



pub const VERTICAL: &str = "│";
pub const DOUBLE_VERTICAL: &str = "║";
pub const THICK_VERTICAL: &str = "┃";

pub const HORIZONTAL: &str = "─";
pub const DOUBLE_HORIZONTAL: &str = "═";
pub const THICK_HORIZONTAL: &str = "━";

pub const TOP_RIGHT: &str = "┐";
pub const ROUNDED_TOP_RIGHT: &str = "╮";
pub const DOUBLE_TOP_RIGHT: &str = "╗";
pub const THICK_TOP_RIGHT: &str = "┓";

pub const TOP_LEFT: &str = "┌";
pub const ROUNDED_TOP_LEFT: &str = "╭";
pub const DOUBLE_TOP_LEFT: &str = "╔";
pub const THICK_TOP_LEFT: &str = "┏";

pub const BOTTOM_RIGHT: &str = "┘";
pub const ROUNDED_BOTTOM_RIGHT: &str = "╯";
pub const DOUBLE_BOTTOM_RIGHT: &str = "╝";
pub const THICK_BOTTOM_RIGHT: &str = "┛";

pub const BOTTOM_LEFT: &str = "└";
pub const ROUNDED_BOTTOM_LEFT: &str = "╰";
pub const DOUBLE_BOTTOM_LEFT: &str = "╚";
pub const THICK_BOTTOM_LEFT: &str = "┗";

pub const VERTICAL_LEFT: &str = "┤";
pub const DOUBLE_VERTICAL_LEFT: &str = "╣";
pub const THICK_VERTICAL_LEFT: &str = "┫";

pub const VERTICAL_RIGHT: &str = "├";
pub const DOUBLE_VERTICAL_RIGHT: &str = "╠";
pub const THICK_VERTICAL_RIGHT: &str = "┣";

pub const HORIZONTAL_DOWN: &str = "┬";
pub const DOUBLE_HORIZONTAL_DOWN: &str = "╦";
pub const THICK_HORIZONTAL_DOWN: &str = "┳";

pub const HORIZONTAL_UP: &str = "┴";
pub const DOUBLE_HORIZONTAL_UP: &str = "╩";
pub const THICK_HORIZONTAL_UP: &str = "┻";

pub const CROSS: &str = "┼";
pub const DOUBLE_CROSS: &str = "╬";
pub const THICK_CROSS: &str = "╋";



// ================================================================================================
