//! Schema-level canonical handling for fields declared to be sets.
//!
//! `pce/1` deliberately has no Set kind. A schema declares which of its array
//! fields are sets, then calls this module at that field's boundary. Ordinary
//! arrays retain their schema-defined order and never pass through these
//! functions.

use core::fmt;

use super::encode::{encode_at_depth, write_array_head};
use super::{Error as CanonicalError, MAX_DEPTH, Value};

/// Why a schema-declared set array could not be encoded or validated.
///
/// This is intentionally distinct from [`CanonicalError`]. `pce/1` accepts an
/// array in any order; only a schema can say that one particular array is a set
/// and therefore subject to strict ordering and uniqueness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The set array itself would occur beyond the codec's depth limit.
    SetDepthLimitExceeded {
        /// The actual depth supplied by the schema traversal.
        depth: usize,
    },
    /// One element could not be canonically encoded at its actual depth.
    ElementNotEncodable {
        /// Zero-based position in the caller's element sequence.
        index: usize,
        /// The canonical codec rule the element violated.
        source: CanonicalError,
    },
    /// Two logical elements have identical canonical bytes.
    DuplicateElement {
        /// Zero-based position of one duplicate in the caller's sequence.
        first: usize,
        /// Zero-based position of the other duplicate.
        second: usize,
    },
    /// A decoded set array is not strictly ascending.
    NotStrictlyIncreasing {
        /// Zero-based position of the preceding element.
        previous: usize,
        /// Zero-based position of the element that precedes it bytewise.
        current: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetDepthLimitExceeded { depth } => write!(
                formatter,
                "set array at depth {depth} exceeds the canonical limit {MAX_DEPTH}"
            ),
            Self::ElementNotEncodable { index, source } => {
                write!(formatter, "set element {index} is not encodable: {source}")
            }
            Self::DuplicateElement { first, second } => write!(
                formatter,
                "set elements {first} and {second} have identical canonical bytes"
            ),
            Self::NotStrictlyIncreasing { previous, current } => write!(
                formatter,
                "set element {current} does not follow element {previous} in canonical byte order"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ElementNotEncodable { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Encode a schema-declared set as a `pce/1` array.
///
/// Elements are sorted by an unsigned lexicographic comparison of their full
/// canonical bytes. Equal encodings are rejected rather than silently
/// deduplicated.
///
/// `set_depth` is the array's actual depth in the complete artifact: the root
/// is zero, so its elements begin at one. Requiring this argument is
/// load-bearing. Encoding every sort key as a standalone depth-zero value would
/// accept an element that the decoder must reject after it is spliced into a
/// deep enclosing artifact.
///
/// The returned bytes include the array head and are ready for a schema encoder
/// to place at that exact depth. Ordinary semantic arrays must use
/// [`super::encode`] instead.
///
/// # Errors
///
/// Returns [`Error::SetDepthLimitExceeded`] if the set array itself is too
/// deep, [`Error::ElementNotEncodable`] if any element violates a canonical
/// codec rule at its inherited depth, or [`Error::DuplicateElement`] if two
/// elements encode identically.
pub fn encode_array(elements: &[Value], set_depth: usize) -> Result<Vec<u8>, Error> {
    require_set_depth(set_depth)?;
    let mut keyed = encode_elements(elements, set_depth)?;
    keyed.sort_by(|left, right| left.bytes.cmp(&right.bytes));

    for pair in keyed.windows(2) {
        if pair[0].bytes == pair[1].bytes {
            let first = pair[0].original_index.min(pair[1].original_index);
            let second = pair[0].original_index.max(pair[1].original_index);
            return Err(Error::DuplicateElement { first, second });
        }
    }

    let mut out = Vec::new();
    write_array_head(&mut out, keyed.len());
    for element in keyed {
        out.extend_from_slice(&element.bytes);
    }
    Ok(out)
}

/// Validate the observed order of a decoded schema-declared set array.
///
/// This function never sorts or repairs. A schema decoder calls it after the
/// ordinary `pce/1` decoder has produced the array's elements, at the array's
/// actual depth in the enclosing artifact.
///
/// # Errors
///
/// Returns the same depth and element errors as [`encode_array`], plus
/// [`Error::DuplicateElement`] or [`Error::NotStrictlyIncreasing`] when the
/// observed array is not a strict canonical set.
pub fn validate_array(elements: &[Value], set_depth: usize) -> Result<(), Error> {
    require_set_depth(set_depth)?;
    let keyed = encode_elements(elements, set_depth)?;

    for (current, pair) in keyed.windows(2).enumerate() {
        match pair[0].bytes.cmp(&pair[1].bytes) {
            core::cmp::Ordering::Less => {}
            core::cmp::Ordering::Equal => {
                return Err(Error::DuplicateElement {
                    first: current,
                    second: current + 1,
                });
            }
            core::cmp::Ordering::Greater => {
                return Err(Error::NotStrictlyIncreasing {
                    previous: current,
                    current: current + 1,
                });
            }
        }
    }
    Ok(())
}

struct EncodedElement {
    original_index: usize,
    bytes: Vec<u8>,
}

fn require_set_depth(set_depth: usize) -> Result<(), Error> {
    if set_depth > MAX_DEPTH {
        Err(Error::SetDepthLimitExceeded { depth: set_depth })
    } else {
        Ok(())
    }
}

fn encode_elements(elements: &[Value], set_depth: usize) -> Result<Vec<EncodedElement>, Error> {
    let element_depth = set_depth + 1;
    elements
        .iter()
        .enumerate()
        .map(|(index, element)| {
            encode_at_depth(element, element_depth)
                .map(|bytes| EncodedElement {
                    original_index: index,
                    bytes,
                })
                .map_err(|source| Error::ElementNotEncodable { index, source })
        })
        .collect()
}
