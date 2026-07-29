//! sRGB colour, WCAG contrast, CIELAB difference, and colour-vision simulation.
//!
//! Every number this module produces is *computed* from a colour's channels.
//! Nothing here reads a recorded ratio, because a recorded ratio is a claim and
//! the point of the harness is to stop the token set making claims.
//!
//! The formulae are anchored outside this repository:
//!
//! - Relative luminance and contrast ratio are WCAG 2.x definitions, and the
//!   tests pin them to values the specification itself publishes: black on
//!   white is exactly `21:1`, any colour against itself is exactly `1:1`.
//! - The colour-vision matrices are Machado, Oliveira and Fernandes (2009),
//!   applied in linear RGB. Their digits are not taken on trust: the tests
//!   assert the *defining qualitative property* of each deficiency instead, so a
//!   mistyped coefficient shows up as a red/green pair that failed to converge
//!   rather than as a plausible-looking number nobody can check.

use std::fmt;

/// An 8-bit-per-channel sRGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Srgb {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

/// Why a colour string could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorError {
    /// The string did not start with `#`, or was not 7 characters long.
    Shape(String),
    /// The string contained a character that is not a hexadecimal digit.
    Digit(String),
}

impl fmt::Display for ColorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(found) => write!(
                formatter,
                "expected a colour of the form #RRGGBB, found {found:?}"
            ),
            Self::Digit(found) => write!(
                formatter,
                "colour {found:?} contains a character that is not a hexadecimal digit"
            ),
        }
    }
}

impl std::error::Error for ColorError {}

impl Srgb {
    /// Parse `#RRGGBB`.
    ///
    /// Deliberately strict: no three-digit shorthand, no `rgb()`, no named
    /// colours, no alpha. A token file is written once and read by machines, so
    /// there is nothing to gain from accepting several spellings of one colour
    /// and a contrast figure computed from a misread string to lose.
    ///
    /// # Errors
    ///
    /// [`ColorError::Shape`] if the string is not exactly `#` followed by six
    /// characters, and [`ColorError::Digit`] if any of those six is not a
    /// hexadecimal digit.
    ///
    /// # Panics
    ///
    /// It does not. The `expect` below is unreachable: length and every digit
    /// are checked immediately above it, so parsing two verified hexadecimal
    /// characters into a `u8` cannot fail. It is written as `expect` rather
    /// than `unwrap_or(0)` because a silent zero would be a colour nobody
    /// wrote, and this crate exists to stop exactly that.
    pub fn parse(text: &str) -> Result<Self, ColorError> {
        let digits = text
            .strip_prefix('#')
            .filter(|rest| rest.len() == 6)
            .ok_or_else(|| ColorError::Shape(text.to_owned()))?;
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ColorError::Digit(text.to_owned()));
        }
        let channel = |range: std::ops::Range<usize>| {
            u8::from_str_radix(&digits[range], 16).expect("verified hexadecimal above")
        };
        Ok(Self {
            r: channel(0..2),
            g: channel(2..4),
            b: channel(4..6),
        })
    }

    /// The channels as linear-light values, undoing the sRGB transfer function.
    fn linear(self) -> [f64; 3] {
        [self.r, self.g, self.b].map(|channel| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.040_45 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        })
    }

    /// WCAG relative luminance.
    #[must_use]
    pub fn relative_luminance(self) -> f64 {
        let [r, g, b] = self.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

/// WCAG contrast ratio between two colours, in the range `1.0 ..= 21.0`.
///
/// Symmetric by construction: the lighter colour is placed on top rather than
/// the caller being trusted to pass them in a particular order.
#[must_use]
pub fn contrast_ratio(one: Srgb, other: Srgb) -> f64 {
    let a = one.relative_luminance();
    let b = other.relative_luminance();
    let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
    (lighter + 0.05) / (darker + 0.05)
}

/// A colour-vision deficiency to simulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deficiency {
    /// Red-insensitive.
    Protanopia,
    /// Green-insensitive.
    Deuteranopia,
    /// Blue-insensitive.
    Tritanopia,
}

impl Deficiency {
    /// Every deficiency, for exhaustive iteration in the harness and tests.
    pub const ALL: [Self; 3] = [Self::Protanopia, Self::Deuteranopia, Self::Tritanopia];

    /// The name used in harness output.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Protanopia => "protanopia",
            Self::Deuteranopia => "deuteranopia",
            Self::Tritanopia => "tritanopia",
        }
    }

    /// Machado et al. (2009), severity 1.0, row-major, operating on linear RGB.
    fn matrix(self) -> [[f64; 3]; 3] {
        match self {
            Self::Protanopia => [
                [0.152_286, 1.052_583, -0.204_868],
                [0.114_503, 0.786_281, 0.099_216],
                [-0.003_882, -0.048_116, 1.051_998],
            ],
            Self::Deuteranopia => [
                [0.367_322, 0.860_646, -0.227_968],
                [0.280_085, 0.672_501, 0.047_413],
                [-0.011_820, 0.042_940, 0.968_881],
            ],
            Self::Tritanopia => [
                [1.255_528, -0.076_749, -0.178_779],
                [-0.078_411, 0.930_809, 0.147_602],
                [0.004_733, 0.691_367, 0.303_900],
            ],
        }
    }
}

/// Simulate how a colour appears to a viewer with `deficiency`.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is clamped to 0..=255 and rounded before the cast, so it \
              is an exact non-negative integer inside u8's range"
)]
pub fn simulate(color: Srgb, deficiency: Deficiency) -> Srgb {
    let linear = color.linear();
    let matrix = deficiency.matrix();
    let encode = |value: f64| {
        let clamped = value.clamp(0.0, 1.0);
        let encoded = if clamped <= 0.003_130_8 {
            clamped * 12.92
        } else {
            1.055 * clamped.powf(1.0 / 2.4) - 0.055
        };
        // Round first, then clamp: the multiply can land a hair outside 0..=255
        // through floating-point error, and the cast below is only sound
        // because both have happened.
        (encoded * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Srgb {
        r: encode(matrix[0][0] * linear[0] + matrix[0][1] * linear[1] + matrix[0][2] * linear[2]),
        g: encode(matrix[1][0] * linear[0] + matrix[1][1] * linear[1] + matrix[1][2] * linear[2]),
        b: encode(matrix[2][0] * linear[0] + matrix[2][1] * linear[1] + matrix[2][2] * linear[2]),
    }
}

/// CIELAB coordinates under the D65 white point.
fn lab(color: Srgb) -> [f64; 3] {
    let [red, green, blue] = color.linear();
    // Linear sRGB to CIEXYZ, D65.
    let big_x = 0.412_456_4 * red + 0.357_576_1 * green + 0.180_437_5 * blue;
    let big_y = 0.212_672_9 * red + 0.715_152_2 * green + 0.072_175_0 * blue;
    let big_z = 0.019_333_9 * red + 0.119_192_0 * green + 0.950_304_1 * blue;

    // D65 reference white.
    let reference = [0.950_47, 1.0, 1.088_83];
    let transfer = |value: f64| {
        if value > 216.0 / 24389.0 {
            value.cbrt()
        } else {
            (841.0 / 108.0) * value + 4.0 / 29.0
        }
    };
    let [near_x, near_y, near_z] = [
        big_x / reference[0],
        big_y / reference[1],
        big_z / reference[2],
    ]
    .map(transfer);
    [
        116.0 * near_y - 16.0,
        500.0 * (near_x - near_y),
        200.0 * (near_y - near_z),
    ]
}

/// CIE76 colour difference.
///
/// The crudest of the delta-E formulae, and chosen deliberately: it is a plain
/// Euclidean distance in CIELAB that anyone can re-derive, and the harness uses
/// it only to catch two roles collapsing onto one another. It is **not** a
/// perceptual guarantee, and [`crate::audit`] does not present it as one.
#[must_use]
pub fn delta_e_76(one: Srgb, other: Srgb) -> f64 {
    let a = lab(one);
    let b = lab(other);
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests;
