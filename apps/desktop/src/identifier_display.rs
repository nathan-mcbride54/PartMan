//! Lossless and bounded presentation of hostile operating-system identifiers.
//!
//! Linux and other Unix-family adapters can observe arbitrary bytes. Windows
//! adapters can observe WTF-16 containing unpaired surrogates. Neither is
//! safely representable by a lossy UTF-8 conversion. This module keeps those
//! values authoritative in Rust and derives an injective, domain-separated
//! ASCII representation for display. The bounded derivative is deliberately
//! not reversible and must never be used for lookup or authorization.

use std::{fmt, hash::Hash};

const BYTE_PREFIX: &str = "b:";
const WTF16_PREFIX: &str = "w:";
const BACKSLASH_U16: u16 = 0x005c;
const TRUNCATION_MARKER: char = '…';
const MINIMUM_DISPLAY_CHARACTERS: usize = 3;

/// Default maximum character count for an identifier in a list or topology row.
pub const DEFAULT_IDENTIFIER_DISPLAY_CHARACTERS: usize = 96;

/// An exact identifier representation retained by Rust.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RawIdentifier {
    /// Arbitrary bytes, including invalid UTF-8 and embedded NUL bytes.
    Bytes(Box<[u8]>),
    /// Exact Windows UTF-16 code units, including unpaired surrogates.
    Wtf16(Box<[u16]>),
}

impl RawIdentifier {
    /// Produce the complete injective escaped representation.
    ///
    /// The result is presentation data. Code that needs the identifier must
    /// retain this [`RawIdentifier`] rather than decoding the string.
    #[must_use]
    pub fn full_display(&self) -> FullIdentifierDisplay {
        match self {
            Self::Bytes(bytes) => {
                FullIdentifierDisplay(encode_full(BYTE_PREFIX, &ByteUnits(bytes)))
            }
            Self::Wtf16(units) => {
                FullIdentifierDisplay(encode_full(WTF16_PREFIX, &Wtf16Units(units)))
            }
        }
    }

    /// Produce a token-safe bounded representation for dense visual surfaces.
    ///
    /// The result preserves the domain prefix and cuts only between complete
    /// literal or escape tokens. It is not reversible when [`is_truncated`](
    /// BoundedIdentifierDisplay::is_truncated) is true.
    #[must_use]
    pub fn bounded_display(&self, limit: DisplayLimit) -> BoundedIdentifierDisplay {
        match self {
            Self::Bytes(bytes) => encode_bounded(BYTE_PREFIX, &ByteUnits(bytes), limit),
            Self::Wtf16(units) => encode_bounded(WTF16_PREFIX, &Wtf16Units(units), limit),
        }
    }
}

/// Complete, collision-safe escaped presentation of a raw identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FullIdentifierDisplay(String);

impl FullIdentifierDisplay {
    /// Borrow the escaped presentation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FullIdentifierDisplay {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FullIdentifierDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Bounded, presentation-only derivative of a raw identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BoundedIdentifierDisplay {
    text: String,
    truncated: bool,
}

impl BoundedIdentifierDisplay {
    /// Borrow the bounded presentation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Whether source tokens were omitted and the truncation marker was added.
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }
}

impl AsRef<str> for BoundedIdentifierDisplay {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for BoundedIdentifierDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Validated maximum number of Unicode scalar values in a bounded display.
///
/// Canonical identifier tokens are ASCII. The only non-ASCII scalar the
/// bounded representation can add is its one-character truncation marker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DisplayLimit(usize);

impl DisplayLimit {
    /// Validate a bounded-display character limit.
    ///
    /// # Errors
    ///
    /// Returns [`DisplayLimitError`] when the limit cannot hold the two-character
    /// domain prefix and the one-character truncation marker.
    pub const fn new(characters: usize) -> Result<Self, DisplayLimitError> {
        if characters < MINIMUM_DISPLAY_CHARACTERS {
            Err(DisplayLimitError)
        } else {
            Ok(Self(characters))
        }
    }

    /// The validated character limit.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

impl Default for DisplayLimit {
    fn default() -> Self {
        Self(DEFAULT_IDENTIFIER_DISPLAY_CHARACTERS)
    }
}

impl TryFrom<usize> for DisplayLimit {
    type Error = DisplayLimitError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// A display limit is too small to identify the representation and truncation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisplayLimitError;

impl fmt::Display for DisplayLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "identifier display limit must be at least {MINIMUM_DISPLAY_CHARACTERS} characters"
        )
    }
}

impl std::error::Error for DisplayLimitError {}

trait EncodedUnits {
    fn len(&self) -> usize;
    fn token_len(&self, index: usize) -> usize;
    fn push_token(&self, index: usize, output: &mut String);
}

#[derive(Clone, Copy)]
struct ByteUnits<'a>(&'a [u8]);

impl EncodedUnits for ByteUnits<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn token_len(&self, index: usize) -> usize {
        byte_token_len(self.0[index])
    }

    fn push_token(&self, index: usize, output: &mut String) {
        push_byte_token(self.0[index], output);
    }
}

#[derive(Clone, Copy)]
struct Wtf16Units<'a>(&'a [u16]);

impl EncodedUnits for Wtf16Units<'_> {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn token_len(&self, index: usize) -> usize {
        wtf16_token_len(self.0[index])
    }

    fn push_token(&self, index: usize, output: &mut String) {
        push_wtf16_token(self.0[index], output);
    }
}

fn encode_full<U: EncodedUnits>(prefix: &str, units: &U) -> String {
    let mut output = String::new();
    output.push_str(prefix);
    for index in 0..units.len() {
        units.push_token(index, &mut output);
    }
    output
}

fn encode_bounded<U: EncodedUnits>(
    prefix: &str,
    units: &U,
    limit: DisplayLimit,
) -> BoundedIdentifierDisplay {
    let encoded_characters = (0..units.len()).fold(prefix.len(), |total, index| {
        total.saturating_add(units.token_len(index))
    });
    if encoded_characters <= limit.get() {
        return BoundedIdentifierDisplay {
            text: encode_full(prefix, units),
            truncated: false,
        };
    }

    let content_budget = limit.get() - prefix.len() - 1;
    let head_target = content_budget / 2;
    let tail_target = content_budget - head_target;
    let mut head_end = 0;
    let mut head_characters = 0;
    while head_end < units.len() {
        let token = units.token_len(head_end);
        if head_characters + token > head_target {
            break;
        }
        head_characters += token;
        head_end += 1;
    }

    let mut tail_start = units.len();
    let mut tail_characters = 0;
    while tail_start > head_end {
        let token = units.token_len(tail_start - 1);
        if tail_characters + token > tail_target {
            break;
        }
        tail_characters += token;
        tail_start -= 1;
    }

    let mut remaining = content_budget - head_characters - tail_characters;
    loop {
        let mut progressed = false;

        if tail_start > head_end {
            let token = units.token_len(tail_start - 1);
            if token <= remaining {
                tail_start -= 1;
                remaining -= token;
                progressed = true;
            }
        }

        if tail_start > head_end {
            let token = units.token_len(head_end);
            if token <= remaining {
                head_end += 1;
                remaining -= token;
                progressed = true;
            }
        }

        if !progressed {
            break;
        }
    }

    let mut text = String::new();
    text.push_str(prefix);
    for index in 0..head_end {
        units.push_token(index, &mut text);
    }
    text.push(TRUNCATION_MARKER);
    for index in tail_start..units.len() {
        units.push_token(index, &mut text);
    }

    debug_assert!(text.chars().count() <= limit.get());
    BoundedIdentifierDisplay {
        text,
        truncated: true,
    }
}

const fn byte_token_len(byte: u8) -> usize {
    match byte {
        0x20..=0x7e if byte != b'\\' => 1,
        b'\\' => 2,
        _ => 4,
    }
}

fn push_byte_token(byte: u8, output: &mut String) {
    match byte {
        0x20..=0x7e if byte != b'\\' => output.push(char::from(byte)),
        b'\\' => output.push_str("\\\\"),
        _ => {
            output.push_str("\\x");
            push_hex(output, u16::from(byte), 2);
        }
    }
}

const fn wtf16_token_len(unit: u16) -> usize {
    match unit {
        0x20..=0x7e if unit != BACKSLASH_U16 => 1,
        BACKSLASH_U16 => 2,
        _ => 6,
    }
}

fn push_wtf16_token(unit: u16, output: &mut String) {
    match unit {
        0x20..=0x7e if unit != BACKSLASH_U16 => {
            let scalar = char::from_u32(u32::from(unit)).expect("ASCII is a Unicode scalar");
            output.push(scalar);
        }
        BACKSLASH_U16 => output.push_str("\\\\"),
        _ => {
            output.push_str("\\u");
            push_hex(output, unit, 4);
        }
    }
}

fn push_hex(output: &mut String, value: u16, digits: usize) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for index in (0..digits).rev() {
        let shift = index * 4;
        let nibble = usize::from((value >> shift) & 0x0f);
        output.push(char::from(HEX[nibble]));
    }
}

#[cfg(test)]
mod tests;
