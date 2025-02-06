


use std::{fmt, str::FromStr};



/// A 32-bit color.
///
/// # Theming
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Color([u8; 4]);

pub struct ColorSet {
    pub reset: Color,
    pub black: Color,       // black
    pub red: Color,         // red
    pub green: Color,       // green
    pub orange: Color,      // yellow
    pub blue: Color,        // blue
    pub purple: Color,      // magenta
    pub cyan: Color,        // cyan
    pub gray: Color,        // white
    pub darkgray: Color,    // lightblack
    pub pink: Color,        // lightred
    pub lime: Color,        // lightgreen
    pub yellow: Color,      // lightyellow
    pub sky: Color,         // lightblue
    pub magenta: Color,     // lightmagenta
    pub turquoise: Color,   // lightcyan
    pub white: Color,       // lightwhite
}

// Constants.
impl Color {
    pub const NONE: Self = Self([0, 0, 0, 0]);
}

impl Color {
    /// Create a new color from its RGBA channels.
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    /// Create a new color from a u32. The u32 should be in the format `0xRRGGBBAA`.
    pub const fn from_u32(u: u32) -> Self {
        let r = (u >> 24) as u8;
        let g = (u >> 16) as u8;
        let b = (u >> 8) as u8;
        let a = u as u8;
        Self([r, g, b, a])
    }

    /// Create a new RGB color from a u32. The u32 should be in the format `0x00RRGGBB`.
    pub const fn from_rgb_u32(u: u32) -> Self {
        let r = (u >> 16) as u8;
        let g = (u >> 8) as u8;
        let b = u as u8;
        Self([r, g, b, 255])
    }

    /// Convert this color into a u32 encoded as `0xRRGGBBAA`.
    pub fn as_u32(&self) -> u32 {
        u32::from_be_bytes(self.0)
    }

    /// Convert this color into a u32 encoded as `0x00RRGGBB`.
    pub fn as_rgb_u32(&self) -> u32 {
        u32::from_be_bytes([0, self.r(), self.g(), self.b()])
    }

    pub fn as_rgba_f32(&self) -> [f32; 4] {
        [
            linear_f32_from_gamma_u8(self.r()),
            linear_f32_from_gamma_u8(self.g()),
            linear_f32_from_gamma_u8(self.b()),
            self.a() as f32 / 255.0,
        ]
    }

    /// Multiply this color's channels by the given gamma factor.
    #[inline]
    pub fn gamma_multiply(self, factor: f32) -> Self {
        let Self([r, g, b, a]) = self;
        Self([
            (r as f32 * factor + 0.5) as u8,
            (g as f32 * factor + 0.5) as u8,
            (b as f32 * factor + 0.5) as u8,
            (a as f32 * factor + 0.5) as u8,
        ])
    }

    /// Get a tuple of this color's RGB channel values.
    pub fn as_rgb(&self) -> (u8, u8, u8) {
        (self.0[0], self.0[1], self.0[2])
    }

    /// Get a tuple of this color's RGBA channel values.
    pub fn as_rgba(&self) -> (u8, u8, u8, u8) {
        (self.0[0], self.0[1], self.0[2], self.0[3])
    }

    /// Get an array of this color's RGBA channel values.
    pub fn as_rgba_array(&self) -> [u8; 4] {
        self.0
    }

    /// The red channel value.
    #[inline]
    pub fn r(&self) -> u8 {
        self.0[0]
    }

    /// The green channel value.
    #[inline]
    pub fn g(&self) -> u8 {
        self.0[1]
    }

    /// The blue channel value.
    #[inline]
    pub fn b(&self) -> u8 {
        self.0[2]
    }

    /// The alpha channel value.
    #[inline]
    pub fn a(&self) -> u8 {
        self.0[3]
    }
}

fn linear_f32_from_gamma_u8(s: u8) -> f32 {
    if s <= 10 {
        s as f32 / 3294.6
    } else {
        ((s as f32 + 14.025) / 269.025).powf(2.4)
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
            Ok(Self(rgba))
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

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Color([r, g, b, a]) = self;
        write!(f, "#{r:02X}{g:02X}{b:02X}{a:02X}")
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
    /// assert_eq!(color, Color([255, 255, 255, 255]));
    ///
    /// let color: Color = Color::from_hsl(0.0, 0.0, 0.0);
    /// assert_eq!(color, Color([0, 0, 0, 255]));
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
    Color([
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
        255,
    ])
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
