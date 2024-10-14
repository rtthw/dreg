#![allow(clippy::unreadable_literal)]



use std::{fmt, str::FromStr};



#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Style {
    pub color_mode: ColorMode,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    #[cfg(feature = "underline-color")]
    pub underline_color: Option<Color>,
    pub add_modifier: Modifier,
    pub sub_modifier: Modifier,
}

impl Style {
    pub const fn new() -> Self {
        Self {
            color_mode: ColorMode::overwrite(),
            fg: None,
            bg: None,
            #[cfg(feature = "underline-color")]
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
    }

    pub const fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub const fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub const fn color_mode(mut self, color_mode: ColorMode) -> Self {
        self.color_mode = color_mode;
        self
    }

    pub const fn add_modifier(mut self, modifier: Modifier) -> Self {
        self.sub_modifier = self.sub_modifier.difference(modifier);
        self.add_modifier = self.add_modifier.union(modifier);
        self
    }

    pub const fn remove_modifier(mut self, modifier: Modifier) -> Self {
        self.add_modifier = self.add_modifier.difference(modifier);
        self.sub_modifier = self.sub_modifier.union(modifier);
        self
    }
}

impl Style {
    pub fn patch<S: Into<Self>>(mut self, other: S) -> Self {
        let other: Style = other.into();
        match other.color_mode {
            ColorMode::Overwrite => {
                self.fg = other.fg.or(self.fg);
                self.bg = other.bg.or(self.bg);
            }
            ColorMode::Additive => {
                if let Some(other_fg) = other.fg {
                    if let Some(self_fg) = self.fg {
                        let other_rgb = other_fg.as_rgb();
                        let self_rgb = self_fg.as_rgb();

                        // let r = ((other_rgb.0 as u16) * (self_rgb.0 as u16)).div_ceil(255) as u8;
                        // let g = ((other_rgb.1 as u16) * (self_rgb.1 as u16)).div_ceil(255) as u8;
                        // let b = ((other_rgb.2 as u16) * (self_rgb.2 as u16)).div_ceil(255) as u8;
                        let r = other_rgb.0.saturating_add(self_rgb.0);
                        let g = other_rgb.1.saturating_add(self_rgb.1);
                        let b = other_rgb.2.saturating_add(self_rgb.2);

                        self.fg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.fg = Some(other_fg);
                    }
                }
                if let Some(other_bg) = other.bg {
                    if let Some(self_bg) = self.bg {
                        let other_rgb = other_bg.as_rgb();
                        let self_rgb = self_bg.as_rgb();

                        let r = other_rgb.0.saturating_add(self_rgb.0);
                        let g = other_rgb.1.saturating_add(self_rgb.1);
                        let b = other_rgb.2.saturating_add(self_rgb.2);
                        // let r = ((other_rgb.0 as u16) * (self_rgb.0 as u16)).div_ceil(255) as u8;
                        // let g = ((other_rgb.1 as u16) * (self_rgb.1 as u16)).div_ceil(255) as u8;
                        // let b = ((other_rgb.2 as u16) * (self_rgb.2 as u16)).div_ceil(255) as u8;

                        self.bg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.bg = Some(other_bg);
                    }
                }
            }
            ColorMode::Subtractive => {
                if let Some(other_fg) = other.fg {
                    if let Some(self_fg) = self.fg {
                        let other_rgb = other_fg.as_rgb();
                        let self_rgb = self_fg.as_rgb();
                        let r = other_rgb.0.saturating_sub(self_rgb.0);
                        let g = other_rgb.1.saturating_sub(self_rgb.1);
                        let b = other_rgb.2.saturating_sub(self_rgb.2);

                        self.fg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.fg = Some(other_fg);
                    }
                }
                if let Some(other_bg) = other.bg {
                    if let Some(self_bg) = self.bg {
                        let other_rgb = other_bg.as_rgb();
                        let self_rgb = self_bg.as_rgb();
                        let r = other_rgb.0.saturating_sub(self_rgb.0);
                        let g = other_rgb.1.saturating_sub(self_rgb.1);
                        let b = other_rgb.2.saturating_sub(self_rgb.2);

                        self.bg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.bg = Some(other_bg);
                    }
                }
            }
            ColorMode::Blend => {
                if let Some(other_fg) = other.fg {
                    if let Some(self_fg) = self.fg {
                        let other_rgb = other_fg.as_rgb();
                        let self_rgb = self_fg.as_rgb();
                        let r = other_rgb.0.saturating_add(self_rgb.0.saturating_sub(other_rgb.0));
                        let g = other_rgb.1.saturating_add(self_rgb.1.saturating_sub(other_rgb.1));
                        let b = other_rgb.2.saturating_add(self_rgb.2.saturating_sub(other_rgb.2));

                        self.fg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.fg = Some(other_fg);
                    }
                }
                if let Some(other_bg) = other.bg {
                    if let Some(self_bg) = self.bg {
                        let other_rgb = other_bg.as_rgb();
                        let self_rgb = self_bg.as_rgb();
                        let r = other_rgb.0.saturating_add(self_rgb.0.saturating_sub(other_rgb.0));
                        let g = other_rgb.1.saturating_add(self_rgb.1.saturating_sub(other_rgb.1));
                        let b = other_rgb.2.saturating_add(self_rgb.2.saturating_sub(other_rgb.2));

                        self.bg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.bg = Some(other_bg);
                    }
                }
            }
            ColorMode::Mix => {
                if let Some(other_fg) = other.fg {
                    if let Some(self_fg) = self.fg {
                        let other_rgb = other_fg.as_rgb();
                        let self_rgb = self_fg.as_rgb();
                        let r = self_rgb.0.saturating_add(other_rgb.0.saturating_sub(self_rgb.0));
                        let g = self_rgb.1.saturating_add(other_rgb.1.saturating_sub(self_rgb.1));
                        let b = self_rgb.2.saturating_add(other_rgb.2.saturating_sub(self_rgb.2));

                        self.fg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.fg = Some(other_fg);
                    }
                }
                if let Some(other_bg) = other.bg {
                    if let Some(self_bg) = self.bg {
                        let other_rgb = other_bg.as_rgb();
                        let self_rgb = self_bg.as_rgb();
                        let r = self_rgb.0.saturating_add(other_rgb.0.saturating_sub(self_rgb.0));
                        let g = self_rgb.1.saturating_add(other_rgb.1.saturating_sub(self_rgb.1));
                        let b = self_rgb.2.saturating_add(other_rgb.2.saturating_sub(self_rgb.2));

                        self.bg = Some(Color::Rgb(r, g, b));
                    } else {
                        self.bg = Some(other_bg);
                    }
                }
            }
        }

        #[cfg(feature = "underline-color")]
        {
            self.underline_color = other.underline_color.or(self.underline_color);
        }

        self.add_modifier.remove(other.sub_modifier);
        self.add_modifier.insert(other.add_modifier);
        self.sub_modifier.remove(other.add_modifier);
        self.sub_modifier.insert(other.sub_modifier);

        self
    }
}



// ================================================================================================



/// The way in which an [`Element`] is rendered to the screen.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorMode {
    /// Ignore the buffer's current contents and overwrite cells with the colors provided to the
    /// renderer.
    #[default]
    Overwrite,
    /// Add the renderer's colors to the current cells in the buffer.
    Additive,
    Subtractive,
    /// Blend the renderer's colors with the current cells in the buffer. This is absolutely
    /// necessary for transparent images and overlays.
    Blend,
    Mix,
}

impl ColorMode {
    pub const fn overwrite() -> Self {
        Self::Overwrite
    }

    pub const fn additive() -> Self {
        Self::Additive
    }

    pub const fn subtractive() -> Self {
        Self::Subtractive
    }

    pub const fn blend() -> Self {
        Self::Blend
    }

    pub const fn mix() -> Self {
        Self::Mix
    }
}



bitflags::bitflags! {
    /// Modifier changes the way a piece of text is displayed.
    ///
    /// They are bitflags so they can easily be composed.
    ///
    /// `From<Modifier> for Style` is implemented so you can use `Modifier` anywhere that accepts
    /// `Into<Style>`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use dreg::prelude::*;
    ///
    /// let m = Modifier::BOLD | Modifier::ITALIC;
    /// ```
    #[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
    pub struct Modifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINED        = 0b0000_0000_1000;
        const SLOW_BLINK        = 0b0000_0001_0000;
        const RAPID_BLINK       = 0b0000_0010_0000;
        const REVERSED          = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const CROSSED_OUT       = 0b0001_0000_0000;
    }
}

/// Implement the `Debug` trait for `Modifier` manually.
///
/// This will avoid printing the empty modifier as 'Borders(0x0)' and instead print it as 'NONE'.
impl fmt::Debug for Modifier {
    /// Format the modifier as `NONE` if the modifier is empty or as a list of flags separated by
    /// `|` otherwise.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "NONE");
        }
        write!(f, "{}", self.0)
    }
}



/// ANSI Color
///
/// All colors from the [ANSI color table] are supported (though some names are not exactly the
/// same).
///
/// | Color Name     | Color                   | Foreground | Background |
/// |----------------|-------------------------|------------|------------|
/// | `black`        | [`Color::Black`]        | 30         | 40         |
/// | `red`          | [`Color::Red`]          | 31         | 41         |
/// | `green`        | [`Color::Green`]        | 32         | 42         |
/// | `yellow`       | [`Color::Yellow`]       | 33         | 43         |
/// | `blue`         | [`Color::Blue`]         | 34         | 44         |
/// | `magenta`      | [`Color::Magenta`]      | 35         | 45         |
/// | `cyan`         | [`Color::Cyan`]         | 36         | 46         |
/// | `gray`*        | [`Color::Gray`]         | 37         | 47         |
/// | `darkgray`*    | [`Color::DarkGray`]     | 90         | 100        |
/// | `lightred`     | [`Color::LightRed`]     | 91         | 101        |
/// | `lightgreen`   | [`Color::LightGreen`]   | 92         | 102        |
/// | `lightyellow`  | [`Color::LightYellow`]  | 93         | 103        |
/// | `lightblue`    | [`Color::LightBlue`]    | 94         | 104        |
/// | `lightmagenta` | [`Color::LightMagenta`] | 95         | 105        |
/// | `lightcyan`    | [`Color::LightCyan`]    | 96         | 106        |
/// | `white`*       | [`Color::White`]        | 97         | 107        |
///
/// - `gray` is sometimes called `white` - this is not supported as we use `white` for bright white
/// - `gray` is sometimes called `silver` - this is supported
/// - `darkgray` is sometimes called `light black` or `bright black` (both are supported)
/// - `white` is sometimes called `light white` or `bright white` (both are supported)
/// - we support `bright` and `light` prefixes for all colors
/// - we support `-` and `_` and ` ` as separators for all colors
/// - we support both `gray` and `grey` spellings
///
/// `From<Color> for Style` is implemented by creating a style with the foreground color set to the
/// given color. This allows you to use colors anywhere that accepts `Into<Style>`.
///
/// # Example
///
/// ```
/// use std::str::FromStr;
///
/// use ratatui::prelude::*;
///
/// assert_eq!(Color::from_str("red"), Ok(Color::Red));
/// assert_eq!("red".parse(), Ok(Color::Red));
/// assert_eq!("lightred".parse(), Ok(Color::LightRed));
/// assert_eq!("light red".parse(), Ok(Color::LightRed));
/// assert_eq!("light-red".parse(), Ok(Color::LightRed));
/// assert_eq!("light_red".parse(), Ok(Color::LightRed));
/// assert_eq!("lightRed".parse(), Ok(Color::LightRed));
/// assert_eq!("bright red".parse(), Ok(Color::LightRed));
/// assert_eq!("bright-red".parse(), Ok(Color::LightRed));
/// assert_eq!("silver".parse(), Ok(Color::Gray));
/// assert_eq!("dark-grey".parse(), Ok(Color::DarkGray));
/// assert_eq!("dark gray".parse(), Ok(Color::DarkGray));
/// assert_eq!("light-black".parse(), Ok(Color::DarkGray));
/// assert_eq!("white".parse(), Ok(Color::White));
/// assert_eq!("bright white".parse(), Ok(Color::White));
/// ```
///
/// [ANSI color table]: https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Color {
    /// Resets the foreground or background color
    #[default]
    Reset,
    /// ANSI Color: Black. Foreground: 30, Background: 40
    Black,
    /// ANSI Color: Red. Foreground: 31, Background: 41
    Red,
    /// ANSI Color: Green. Foreground: 32, Background: 42
    Green,
    /// ANSI Color: Yellow. Foreground: 33, Background: 43
    Yellow,
    /// ANSI Color: Blue. Foreground: 34, Background: 44
    Blue,
    /// ANSI Color: Magenta. Foreground: 35, Background: 45
    Magenta,
    /// ANSI Color: Cyan. Foreground: 36, Background: 46
    Cyan,
    /// ANSI Color: White. Foreground: 37, Background: 47
    ///
    /// Note that this is sometimes called `silver` or `white` but we use `white` for bright white
    Gray,
    /// ANSI Color: Bright Black. Foreground: 90, Background: 100
    ///
    /// Note that this is sometimes called `light black` or `bright black` but we use `dark gray`
    DarkGray,
    /// ANSI Color: Bright Red. Foreground: 91, Background: 101
    LightRed,
    /// ANSI Color: Bright Green. Foreground: 92, Background: 102
    LightGreen,
    /// ANSI Color: Bright Yellow. Foreground: 93, Background: 103
    LightYellow,
    /// ANSI Color: Bright Blue. Foreground: 94, Background: 104
    LightBlue,
    /// ANSI Color: Bright Magenta. Foreground: 95, Background: 105
    LightMagenta,
    /// ANSI Color: Bright Cyan. Foreground: 96, Background: 106
    LightCyan,
    /// ANSI Color: Bright White. Foreground: 97, Background: 107
    /// Sometimes called `bright white` or `light white` in some terminals
    White,
    /// An RGB color.
    ///
    /// Note that only terminals that support 24-bit true color will display this correctly.
    /// Notably versions of Windows Terminal prior to Windows 10 and macOS Terminal.app do not
    /// support this.
    ///
    /// If the terminal does not support true color, code using the  [`TermwizBackend`] will
    /// fallback to the default text color. Crossterm and Termion do not have this capability and
    /// the display will be unpredictable (e.g. Terminal.app may display glitched blinking text).
    /// See <https://github.com/ratatui-org/ratatui/issues/475> for an example of this problem.
    ///
    /// See also: <https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit>
    ///
    /// [`TermwizBackend`]: crate::backend::TermwizBackend
    Rgb(u8, u8, u8),
    /// An 8-bit 256 color.
    ///
    /// See also <https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit>
    Indexed(u8),
}

impl Color {
    /// Convert a u32 to a Color
    ///
    /// The u32 should be in the format 0x00RRGGBB.
    pub const fn from_u32(u: u32) -> Self {
        let r = (u >> 16) as u8;
        let g = (u >> 8) as u8;
        let b = u as u8;
        Self::Rgb(r, g, b)
    }

    pub fn as_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Rgb(r, g, b) => (*r, *g, *b),
            _ => (0, 0, 0),
        }
    }
}



/// Error type indicating a failure to parse a color string.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse Colors")
    }
}

impl std::error::Error for ParseColorError {}

/// Converts a string representation to a `Color` instance.
///
/// The `from_str` function attempts to parse the given string and convert it to the corresponding
/// `Color` variant. It supports named colors, RGB values, and indexed colors. If the string cannot
/// be parsed, a `ParseColorError` is returned.
///
/// See the [`Color`] documentation for more information on the supported color names.
impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(
            // There is a mix of different color names and formats in the wild.
            // This is an attempt to support as many as possible.
            match s
                .to_lowercase()
                .replace([' ', '-', '_'], "")
                .replace("bright", "light")
                .replace("grey", "gray")
                .replace("silver", "gray")
                .replace("lightblack", "darkgray")
                .replace("lightwhite", "white")
                .replace("lightgray", "white")
                .as_ref()
            {
                "reset" => Self::Reset,
                "black" => Self::Black,
                "red" => Self::Red,
                "green" => Self::Green,
                "yellow" => Self::Yellow,
                "blue" => Self::Blue,
                "magenta" => Self::Magenta,
                "cyan" => Self::Cyan,
                "gray" => Self::Gray,
                "darkgray" => Self::DarkGray,
                "lightred" => Self::LightRed,
                "lightgreen" => Self::LightGreen,
                "lightyellow" => Self::LightYellow,
                "lightblue" => Self::LightBlue,
                "lightmagenta" => Self::LightMagenta,
                "lightcyan" => Self::LightCyan,
                "white" => Self::White,
                _ => {
                    if let Ok(index) = s.parse::<u8>() {
                        Self::Indexed(index)
                    } else if let Some((r, g, b)) = parse_hex_color(s) {
                        Self::Rgb(r, g, b)
                    } else {
                        return Err(ParseColorError);
                    }
                }
            },
        )
    }
}

fn parse_hex_color(input: &str) -> Option<(u8, u8, u8)> {
    if !input.starts_with('#') || input.len() != 7 {
        return None;
    }
    let r = u8::from_str_radix(input.get(1..3)?, 16).ok()?;
    let g = u8::from_str_radix(input.get(3..5)?, 16).ok()?;
    let b = u8::from_str_radix(input.get(5..7)?, 16).ok()?;
    Some((r, g, b))
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reset => write!(f, "Reset"),
            Self::Black => write!(f, "Black"),
            Self::Red => write!(f, "Red"),
            Self::Green => write!(f, "Green"),
            Self::Yellow => write!(f, "Yellow"),
            Self::Blue => write!(f, "Blue"),
            Self::Magenta => write!(f, "Magenta"),
            Self::Cyan => write!(f, "Cyan"),
            Self::Gray => write!(f, "Gray"),
            Self::DarkGray => write!(f, "DarkGray"),
            Self::LightRed => write!(f, "LightRed"),
            Self::LightGreen => write!(f, "LightGreen"),
            Self::LightYellow => write!(f, "LightYellow"),
            Self::LightBlue => write!(f, "LightBlue"),
            Self::LightMagenta => write!(f, "LightMagenta"),
            Self::LightCyan => write!(f, "LightCyan"),
            Self::White => write!(f, "White"),
            Self::Rgb(r, g, b) => write!(f, "#{r:02X}{g:02X}{b:02X}"),
            Self::Indexed(i) => write!(f, "{i}"),
        }
    }
}

impl Color {
    /// Converts a HSL representation to a `Color::Rgb` instance.
    ///
    /// The `from_hsl` function converts the Hue, Saturation and Lightness values to a
    /// corresponding `Color` RGB equivalent.
    ///
    /// Hue values should be in the range [0, 360].
    /// Saturation and L values should be in the range [0, 100].
    /// Values that are not in the range are clamped to be within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::prelude::*;
    ///
    /// let color: Color = Color::from_hsl(360.0, 100.0, 100.0);
    /// assert_eq!(color, Color::Rgb(255, 255, 255));
    ///
    /// let color: Color = Color::from_hsl(0.0, 0.0, 0.0);
    /// assert_eq!(color, Color::Rgb(0, 0, 0));
    /// ```
    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        // Clamp input values to valid ranges
        let h = h.clamp(0.0, 360.0);
        let s = s.clamp(0.0, 100.0);
        let l = l.clamp(0.0, 100.0);

        // Delegate to the function for normalized HSL to RGB conversion
        normalized_hsl_to_rgb(h / 360.0, s / 100.0, l / 100.0)
    }
}

/// Converts normalized HSL (Hue, Saturation, Lightness) values to RGB (Red, Green, Blue) color
/// representation. H, S, and L values should be in the range [0, 1].
///
/// Based on <https://github.com/killercup/hsl-rs/blob/b8a30e11afd75f262e0550725333293805f4ead0/src/lib.rs>
fn normalized_hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> Color {
    // This function can be made into `const` in the future.
    // This comment contains the relevant information for making it `const`.
    //
    // If it is `const` and made public, users can write the following:
    //
    // ```rust
    // const SLATE_50: Color = normalized_hsl_to_rgb(0.210, 0.40, 0.98);
    // ```
    //
    // For it to be const now, we need `#![feature(const_fn_floating_point_arithmetic)]`
    // Tracking issue: https://github.com/rust-lang/rust/issues/57241
    //
    // We would also need to remove the use of `.round()` in this function, i.e.:
    //
    // ```rust
    // Color::Rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    // ```

    // Initialize RGB components
    let red: f64;
    let green: f64;
    let blue: f64;

    // Check if the color is achromatic (grayscale)
    if saturation == 0.0 {
        red = lightness;
        green = lightness;
        blue = lightness;
    } else {
        // Calculate RGB components for colored cases
        let q = if lightness < 0.5 {
            lightness * (1.0 + saturation)
        } else {
            lightness + saturation - lightness * saturation
        };
        let p = 2.0 * lightness - q;
        red = hue_to_rgb(p, q, hue + 1.0 / 3.0);
        green = hue_to_rgb(p, q, hue);
        blue = hue_to_rgb(p, q, hue - 1.0 / 3.0);
    }

    // Scale RGB components to the range [0, 255] and create a Color::Rgb instance
    Color::Rgb(
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
    )
}

/// Helper function to calculate RGB component for a specific hue value.
fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    // Adjust the hue value to be within the valid range [0, 1]
    let mut t = t;
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    // Calculate the RGB component based on the hue value
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
