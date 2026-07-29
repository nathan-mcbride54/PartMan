//! The `pce/1` canonical encoding, specified in `schemas/canonical-encoding.md`
//! and decided by ADR-C1.
//!
//! The plan hash is an authorization boundary, not a checksum: HLP-001 applies
//! plans by hash, HLP-003 binds interactive authorization to an exact hash, and
//! SEC-001 authorizes only exact hashes. Two properties therefore matter more
//! than convenience.
//!
//! * **No divergence.** One logical value has exactly one encoding, in every
//!   language.
//! * **No malleability.** [`decode`] rejects anything that is not the unique
//!   canonical encoding of the value it denotes, so bytes that were authorized
//!   cannot differ from bytes that describe what executes.
//!
//! Canonical encoding alone would not be enough. If the decoder accepted a
//! non-canonical encoding, an attacker could submit bytes that decode to an
//! approved plan yet hash differently. Strictness is required in both
//! directions.

mod decode;
mod encode;

use core::fmt;
use std::collections::BTreeMap;

pub use decode::decode;
pub use encode::encode;

/// Identifier of this encoding profile, per `schemas/canonical-encoding.md` §7.
pub const PROFILE: &str = "pce/1";

/// Maximum nesting depth accepted by [`decode`].
///
/// The decoder is recursive, so hostile input must not be able to exhaust the
/// stack. Exceeding this limit is [`Error::DepthLimitExceeded`], never a crash.
/// Real artifacts nest a few levels; this bound is far above them and far below
/// anything that threatens a default stack.
pub const MAX_DEPTH: usize = 128;

/// A value representable in the `pce/1` profile.
///
/// Nothing outside this set can be built or decoded. Floating-point values,
/// tags, `undefined`, and indefinite-length items are deliberately absent
/// rather than merely discouraged, which removes `-0.0`, NaN, infinity, and tag
/// confusion from the problem entirely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// An unsigned integer, `0 ..= u64::MAX`.
    Unsigned(u64),
    /// A negative integer, `i64::MIN ..= -1`.
    ///
    /// Encoding a non-negative value through this variant is a programming
    /// error and is reported as [`Error::NegativeOutOfRange`] by [`encode`]
    /// rather than silently producing a different number.
    Negative(i64),
    /// An uninterpreted byte string.
    Bytes(Vec<u8>),
    /// A UTF-8 text string. No Unicode normalization is applied.
    Text(String),
    /// An ordered sequence of values.
    Array(Vec<Value>),
    /// A map with unique text keys.
    ///
    /// A [`BTreeMap`] gives uniqueness and a stable iteration order, but its
    /// order is Rust's bytewise [`String`] ordering, which is *not* the
    /// length-first order this profile requires. [`encode`] sorts keys itself;
    /// callers must not assume iteration order is encoding order.
    Map(BTreeMap<String, Value>),
    /// A boolean.
    Bool(bool),
    /// The null value.
    Null,
}

/// A SHA-256 digest over canonical bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash([u8; 32]);

impl Hash {
    /// The raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The digest as lowercase hexadecimal.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut hex = String::with_capacity(64);
        for byte in self.0 {
            hex.push(nibble(byte >> 4));
            hex.push(nibble(byte & 0x0f));
        }
        hex
    }
}

const fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Hash({})", self.to_hex())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

/// Hash a value with SHA-256 over its canonical bytes (MODEL-005).
///
/// No prefix, salt, or length framing is added. Domain separation belongs
/// inside the value, as the `schema` and `schema_version` fields described in
/// `schemas/canonical-encoding.md` §5, so that the literal MODEL-005 rule is
/// preserved.
///
/// # Errors
///
/// Returns the same errors as [`encode`].
pub fn hash(value: &Value) -> Result<Hash, Error> {
    Ok(digest_of(&encode(value)?))
}

/// Hash bytes, after proving they are the canonical encoding of some value.
///
/// The proof is [`decode`] itself: it accepts only the unique canonical
/// encoding, so bytes that survive it are canonical by construction rather than
/// by the caller's say-so.
///
/// This replaced a `hash_canonical_bytes(&[u8]) -> Hash` whose documentation
/// said "use this only for bytes produced by `encode` or accepted by `decode`".
/// That is an instruction, not a guarantee — the plan hash is an authorization
/// boundary under HLP-001, HLP-003 and SEC-001, and a public function that
/// hashes whatever it is given is a way around strict decoding for anyone who
/// forgets. Nothing about the digest changed; only who is allowed to ask for one.
///
/// # Errors
///
/// Returns the [`decode`] error if `bytes` is not the unique canonical encoding
/// of a value — including a non-canonical encoding of a value that does exist.
pub fn hash_encoded(bytes: &[u8]) -> Result<Hash, Error> {
    decode(bytes)?;
    Ok(digest_of(bytes))
}

/// SHA-256 over bytes this module has just produced or just validated.
///
/// Deliberately private. Every caller is inside this module and holds bytes that
/// [`encode`] returned or [`decode`] accepted a line earlier, so the precondition
/// is visible at the call site rather than asserted in a doc comment.
fn digest_of(bytes: &[u8]) -> Hash {
    use sha2::Digest as _;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Hash(hasher.finalize().into())
}

/// Why a value could not be encoded, or input could not be accepted.
///
/// Decode errors name the specific rule that was violated rather than a generic
/// "malformed", because the rule is the security property being enforced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A [`Value::Negative`] held a non-negative number.
    NegativeOutOfRange(i64),
    /// Input ended in the middle of an item.
    UnexpectedEnd,
    /// Bytes remained after the single top-level item.
    TrailingBytes(usize),
    /// An argument was encoded in more bytes than necessary (§2).
    NonShortestArgument,
    /// Additional information 28, 29, or 30, which are reserved.
    ReservedAdditionalInformation(u8),
    /// An indefinite-length item or a break stop code.
    IndefiniteLength,
    /// A CBOR tag, which this profile excludes entirely.
    TagNotAllowed(u64),
    /// A floating-point value, which this profile excludes entirely.
    FloatNotAllowed,
    /// A simple value other than `false`, `true`, or `null`.
    SimpleValueNotAllowed(u8),
    /// A major-type-1 argument above `i64::MAX`.
    NegativeTooLarge(u64),
    /// A map key that was not a text string.
    MapKeyNotText,
    /// Map keys out of order, or a duplicate key (§3).
    MapKeyNotStrictlyIncreasing {
        /// The offending key.
        key: String,
    },
    /// A text string that was not well-formed UTF-8.
    InvalidUtf8,
    /// A declared length exceeded the bytes actually available.
    LengthExceedsInput {
        /// The length the head claimed.
        declared: u64,
        /// The bytes actually remaining.
        remaining: usize,
    },
    /// Nesting exceeded [`MAX_DEPTH`].
    DepthLimitExceeded,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeOutOfRange(value) => {
                write!(formatter, "Value::Negative held non-negative {value}")
            }
            Self::UnexpectedEnd => formatter.write_str("input ended inside an item"),
            Self::TrailingBytes(count) => {
                write!(formatter, "{count} byte(s) after the top-level item")
            }
            Self::NonShortestArgument => {
                formatter.write_str("argument is not encoded in the shortest form")
            }
            Self::ReservedAdditionalInformation(value) => {
                write!(formatter, "reserved additional information {value}")
            }
            Self::IndefiniteLength => formatter.write_str("indefinite-length items are excluded"),
            Self::TagNotAllowed(tag) => write!(formatter, "tag {tag} is excluded"),
            Self::FloatNotAllowed => {
                formatter.write_str("floating-point values are excluded from this profile")
            }
            Self::SimpleValueNotAllowed(value) => {
                write!(formatter, "simple value {value} is excluded")
            }
            Self::NegativeTooLarge(argument) => {
                write!(formatter, "negative argument {argument} exceeds i64::MAX")
            }
            Self::MapKeyNotText => formatter.write_str("map keys must be text strings"),
            Self::MapKeyNotStrictlyIncreasing { key } => {
                write!(
                    formatter,
                    "map key {key:?} duplicates or precedes its predecessor"
                )
            }
            Self::InvalidUtf8 => formatter.write_str("text string is not well-formed UTF-8"),
            Self::LengthExceedsInput {
                declared,
                remaining,
            } => write!(
                formatter,
                "declared length {declared} exceeds {remaining} remaining byte(s)"
            ),
            Self::DepthLimitExceeded => {
                write!(formatter, "nesting deeper than {MAX_DEPTH}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Order two map keys as the profile requires: length first, then bytewise.
///
/// This is *not* Rust's [`String`] ordering. `"z"` precedes `"aa"` here because
/// it is shorter, which is what makes a bytewise comparison of fully encoded
/// keys give the same answer (§3).
pub(crate) fn compare_keys(left: &str, right: &str) -> core::cmp::Ordering {
    left.len()
        .cmp(&right.len())
        .then_with(|| left.as_bytes().cmp(right.as_bytes()))
}

#[cfg(test)]
mod tests;
