//! Opaque, path-independent selection identifiers for the desktop view model.
//!
//! Selection IDs are session/view-model keys, not device identity, and cannot
//! authorize an operation. Their fixed ASCII wire form avoids passing a `u64`
//! through a renderer numeric type and is validated again when a UI callback
//! returns it. The type can enforce canonical representation and membership;
//! the future view model remains responsible for allocating IDs independently
//! of paths/display text and keeping each ID associated with the correct item.

use std::{collections::BTreeSet, fmt, num::NonZeroU64};

const WIRE_PREFIX: &str = "sid:";
const WIRE_HEX_DIGITS: usize = 16;
const WIRE_LENGTH: usize = WIRE_PREFIX.len() + WIRE_HEX_DIGITS;

/// Opaque nonzero selection key for one view-model lifetime.
///
/// This type alone does not prove that an ID is stable or correctly associated
/// with an authoritative model item.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelectionId(NonZeroU64);

impl SelectionId {
    /// Construct a nonzero opaque selection ID.
    ///
    /// The caller must allocate the value independently of paths and display
    /// text, reject duplicates in the containing [`SelectionRegistry`], and
    /// preserve the item-to-ID association for the view-model lifetime. This
    /// constructor validates only the reserved-zero rule.
    ///
    /// # Errors
    ///
    /// Returns [`SelectionIdError`] for zero, which is reserved as an invalid
    /// or absent value at foreign presentation boundaries.
    pub const fn new(value: u64) -> Result<Self, SelectionIdError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(SelectionIdError),
        }
    }

    /// Encode the ID as a fixed-width canonical ASCII wire value.
    #[must_use]
    pub fn to_wire(self) -> SelectionWire {
        SelectionWire(format!("{WIRE_PREFIX}{:016X}", self.0.get()))
    }

    /// Parse and validate the exact canonical wire form.
    ///
    /// # Errors
    ///
    /// Returns a specific [`SelectionWireError`] for a wrong length, prefix,
    /// non-uppercase-hex digit, or reserved zero value.
    pub fn from_wire(value: &str) -> Result<Self, SelectionWireError> {
        if value.len() != WIRE_LENGTH {
            return Err(SelectionWireError::InvalidLength);
        }
        let digits = value
            .strip_prefix(WIRE_PREFIX)
            .ok_or(SelectionWireError::InvalidPrefix)?;
        if !digits
            .bytes()
            .all(|digit| digit.is_ascii_digit() || (b'A'..=b'F').contains(&digit))
        {
            return Err(SelectionWireError::InvalidDigit);
        }
        let parsed =
            u64::from_str_radix(digits, 16).map_err(|_| SelectionWireError::InvalidDigit)?;
        Self::new(parsed).map_err(|SelectionIdError| SelectionWireError::Zero)
    }
}

/// A zero selection ID was requested.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionIdError;

impl fmt::Display for SelectionIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("selection ID zero is reserved")
    }
}

impl std::error::Error for SelectionIdError {}

/// Canonical fixed-width ASCII representation passed through the UI boundary.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelectionWire(String);

impl SelectionWire {
    /// Borrow the canonical wire representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for SelectionWire {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SelectionWire {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Why a selection wire value was not canonical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionWireError {
    /// The value was not exactly `sid:` plus sixteen ASCII characters.
    InvalidLength,
    /// The domain prefix was not exactly lowercase `sid:`.
    InvalidPrefix,
    /// A digit was not canonical uppercase hexadecimal.
    InvalidDigit,
    /// The parsed value was the reserved zero ID.
    Zero,
}

impl fmt::Display for SelectionWireError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLength => "selection wire value has the wrong length",
            Self::InvalidPrefix => "selection wire value has the wrong prefix",
            Self::InvalidDigit => "selection wire value is not uppercase hexadecimal",
            Self::Zero => "selection wire value contains the reserved zero ID",
        })
    }
}

impl std::error::Error for SelectionWireError {}

/// Exact set of selection IDs accepted by one view-model snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectionRegistry {
    ids: BTreeSet<SelectionId>,
}

impl SelectionRegistry {
    /// Build a registry while rejecting duplicate IDs.
    ///
    /// # Errors
    ///
    /// Returns [`SelectionRegistryError::Duplicate`] for the first repeated ID.
    pub fn new(ids: impl IntoIterator<Item = SelectionId>) -> Result<Self, SelectionRegistryError> {
        let mut unique = BTreeSet::new();
        for id in ids {
            if !unique.insert(id) {
                return Err(SelectionRegistryError::Duplicate(id));
            }
        }
        Ok(Self { ids: unique })
    }

    /// Number of registered opaque IDs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether this view-model snapshot has no selectable IDs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Validate a callback wire value and resolve it only if it is registered.
    ///
    /// # Errors
    ///
    /// Returns [`SelectionLookupError::InvalidWire`] for a malformed callback
    /// or [`SelectionLookupError::Unknown`] for a canonical but stale or
    /// invented ID.
    pub fn resolve_wire(&self, wire: &str) -> Result<SelectionId, SelectionLookupError> {
        let id = SelectionId::from_wire(wire).map_err(SelectionLookupError::InvalidWire)?;
        if self.ids.contains(&id) {
            Ok(id)
        } else {
            Err(SelectionLookupError::Unknown(id))
        }
    }
}

/// A view-model snapshot contained the same opaque ID more than once.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionRegistryError {
    /// The repeated ID.
    Duplicate(SelectionId),
}

impl fmt::Display for SelectionRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(id) => write!(formatter, "duplicate selection ID {}", id.to_wire()),
        }
    }
}

impl std::error::Error for SelectionRegistryError {}

/// A UI callback did not name a currently registered selection ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionLookupError {
    /// The callback string was not a canonical selection wire value.
    InvalidWire(SelectionWireError),
    /// The callback carried a canonical but unregistered ID.
    Unknown(SelectionId),
}

impl fmt::Display for SelectionLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWire(error) => write!(formatter, "invalid selection callback: {error}"),
            Self::Unknown(id) => write!(
                formatter,
                "selection callback names unavailable ID {}",
                id.to_wire()
            ),
        }
    }
}

impl std::error::Error for SelectionLookupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidWire(error) => Some(error),
            Self::Unknown(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
