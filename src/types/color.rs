


use std::{fmt, str::FromStr};



#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Color {
    /// Resets the foreground or background color.
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
    /// Note that this is sometimes called `silver` or `white` but we use `white` for bright white.
    Gray,
    /// ANSI Color: Bright Black. Foreground: 90, Background: 100
    ///
    /// Note that this is sometimes called `light black` or `bright black` but we use `dark gray`.
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
    ///
    /// Sometimes called `bright white` or `light white`.
    White,

    Ansi(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    /// Create a color from its red, green, and blue channels.
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(r, g, b)
    }

    /// Create an RGB color from a u32. The u32 should be in the format `0x00RRGGBB`.
    pub const fn from_rgb_u32(u: u32) -> Self {
        let r = (u >> 16) as u8;
        let g = (u >> 8) as u8;
        let b = u as u8;
        Self::Rgb(r, g, b)
    }
}



/// Error type indicating a failure to parse a color string.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse color")
    }
}

impl std::error::Error for ParseColorError {}

/// Converts a string representation to a `Color` instance.
impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rgba) = parse_hex_color(s) {
            Ok(Self::Rgb(rgba[0], rgba[1], rgba[2]))
        } else {
            return Err(ParseColorError);
        }
    }
}

fn parse_hex_color(input: &str) -> Option<[u8; 4]> {
    if !input.starts_with('#') {
        return None;
    }
    match input.len() {
        7 => {
            let r = u8::from_str_radix(input.get(1..3)?, 16).ok()?;
            let g = u8::from_str_radix(input.get(3..5)?, 16).ok()?;
            let b = u8::from_str_radix(input.get(5..7)?, 16).ok()?;

            Some([r, g, b, 255])
        }
        9 => {
            let r = u8::from_str_radix(input.get(1..3)?, 16).ok()?;
            let g = u8::from_str_radix(input.get(3..5)?, 16).ok()?;
            let b = u8::from_str_radix(input.get(5..7)?, 16).ok()?;
            let a = u8::from_str_radix(input.get(7..9)?, 16).ok()?;

            Some([r, g, b, a])
        }
        _ => None,
    }
}

impl Color {
    /// Converts an HSL representation to a color.
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
    /// use dreg::Color;
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
    // Color([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255])
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
