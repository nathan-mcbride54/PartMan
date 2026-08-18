//! INV-004's derivations, under ADR-0033's rule that a derived value is a
//! derivation and not an observation.
//!
//! Two things are derivations and the list is closed at two: free extents and
//! alignment. Each is "recomputed at use from the detected inputs it names, is
//! never stored, and carries no observation set and no confidence of its own —
//! its trustworthiness is its inputs', which carry the observation sets".
//!
//! Both halves of that are structural here rather than documented. Nothing in
//! this module is a field of anything: [`alignment`] is a function over the
//! inputs it names, and its result carries no provenance of its own, so there
//! is no shape in which a stored derivation could be written down. A test
//! holds that the crate's stored device shape names neither derivation.
//!
//! **The unfit-input arm is the reason this module has a refusal type.** A
//! derivation over an input whose observation set derives `unavailable` or
//! `conflicting` must not be presented as a value; the input's own state is
//! surfaced instead, "so a guess can never wear a computation's clothes". An
//! `inferred` input is fit — the input's confidence travels by reference, and
//! copying it onto the derivation is the stored-confidence shape ADR-C4 made
//! unconstructible.
//!
//! **Free extents are not presented on this platform at all**, and the ground
//! is INV-004's own: the derivation "MUST NOT be presented at all where the
//! host declares a table scheme the build cannot name". This contract
//! declares no scheme for any device, because it builds no partition-table
//! node — see [`free_extents`].

use partman_domain::canonical::Value;
use partman_domain::model::provenance::{Confidence, PropertyObservations};

use crate::observation::Interface;

/// The observation key carrying a device's addressable unit.
pub const LOGICAL_BLOCK_SIZE: &str = "logical_block_size";

/// The observation key carrying a device's write granularity.
pub const PHYSICAL_BLOCK_SIZE: &str = "physical_block_size";

/// Why a derivation was not presented.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Withheld {
    /// ADR-0033's named arm: the input's observation set derives a state
    /// that cannot carry a computation, and that state is what is surfaced.
    InputState {
        /// The input property's key.
        input: String,
        /// The input's own derived confidence — `Unavailable` or
        /// `Conflicting`.
        state: Confidence,
    },
    /// The input is fit by confidence but carries no value this derivation
    /// can use: a positively determined absence is `authoritative` and still
    /// leaves nothing to compute with, and a value that is not a block size
    /// is not one either.
    ///
    /// ADR-0033 does not name this arm. It is handled the same way and for
    /// the same reason — a derivation that filled the gap would be a guess
    /// wearing a computation's clothes — and it is recorded here rather than
    /// folded into the arm above, because calling a present-but-unusable
    /// value `unavailable` would misreport the input's own state, which is
    /// the one thing this arm exists to surface faithfully.
    NoUsableValue {
        /// The input property's key.
        input: String,
        /// What was wrong with it.
        reason: String,
    },
    /// The derivation is not offered for this platform's contract at all.
    NotPresented {
        /// The requirement's own ground for withholding it.
        ground: String,
    },
}

/// The outcome of one derivation.
///
/// Deliberately not `Option`: a withheld derivation carries why, and SAFE-005
/// puts the absent case on the side that must say something.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Derived<T> {
    /// Computed from inputs fit to carry it.
    Presented(T),
    /// Not presented; the input's own state is surfaced instead.
    Withheld(Withheld),
}

/// A device's alignment, as INV-004 names it.
///
/// Returned by value from a function over the inputs it names, and held in no
/// field anywhere: this type exists only in a caller's hands, at use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alignment {
    /// The addressable unit, in bytes.
    pub logical_bytes: u64,
    /// The unit a placement should be a multiple of, in bytes.
    pub granularity_bytes: u64,
}

/// The free-extent derivation, which this contract does not present.
///
/// INV-004: the derivation "MUST NOT be presented at all where the host
/// declares a table scheme the build cannot name, or where a partition the
/// authenticated names place in the host's address space is not one the
/// derivation subtracts; the inputs' own state is surfaced instead."
///
/// Both grounds hold here, and for one reason: this contract builds no
/// partition-table node, so it declares no scheme and places no partition.
/// ADR-0036 put the choice to this increment in terms — "either designate a
/// client-readable table-role source or record that the solver reserves
/// nothing on Linux client drafts until HLP-002 re-discovery supplies a table
/// node" — and the package document records the second, with the measured
/// grounds for declining the first.
///
/// This is a function rather than an omission because an absent surface and a
/// refusing one are different things to a consumer, and SAFE-005 puts absence
/// on the refusing side.
#[must_use]
pub fn free_extents() -> Derived<Vec<(u64, u64)>> {
    Derived::Withheld(Withheld::NotPresented {
        ground: "this contract builds no partition-table node, so it declares no table scheme \
                 and places no partition in any device's address space"
            .to_owned(),
    })
}

/// Derive one device's alignment from the geometry inputs it names.
///
/// Recomputed at each call from the caller's observation sets. Nothing is
/// cached, and the result carries neither an observation set nor a confidence
/// — a reader wanting to know how far to trust it reads the inputs', which is
/// what ADR-0033 means by trustworthiness travelling by reference.
#[must_use]
pub fn alignment(properties: &[(String, PropertyObservations)]) -> Derived<Alignment> {
    let logical = match block_size(properties, LOGICAL_BLOCK_SIZE) {
        Ok(value) => value,
        Err(withheld) => return Derived::Withheld(withheld),
    };
    let physical = match block_size(properties, PHYSICAL_BLOCK_SIZE) {
        Ok(value) => value,
        Err(withheld) => return Derived::Withheld(withheld),
    };
    Derived::Presented(Alignment {
        logical_bytes: logical,
        granularity_bytes: physical,
    })
}

/// The observation key one geometry input is published under.
fn geometry_key(property: &str) -> String {
    format!("{}:{property}", Interface::Sysfs.label())
}

/// Read one geometry input, applying ADR-0033's gate before its value.
///
/// The confidence gate runs **first**, deliberately. An input whose set
/// derives `conflicting` may still hold a parsable value — that is what makes
/// it conflicting — and reaching for the value before the state would let the
/// derivation pick one of two disagreeing reads and present the result as
/// though nothing disagreed.
fn block_size(
    properties: &[(String, PropertyObservations)],
    property: &str,
) -> Result<u64, Withheld> {
    let key = geometry_key(property);
    let Some((_, observations)) = properties.iter().find(|(name, _)| *name == key) else {
        return Err(Withheld::NoUsableValue {
            input: key,
            reason: "the contract reported no such property".to_owned(),
        });
    };

    let state = observations
        .derive_confidence()
        .map_err(|error| Withheld::NoUsableValue {
            input: key.clone(),
            reason: format!("the input's confidence could not be derived: {error}"),
        })?;
    if matches!(state, Confidence::Unavailable | Confidence::Conflicting) {
        return Err(Withheld::InputState { input: key, state });
    }

    let mut observed =
        observations
            .observations
            .iter()
            .filter_map(|observation| match &observation.outcome {
                partman_domain::model::provenance::Outcome::Observed {
                    value: Value::Text(text),
                } => Some(text.as_str()),
                _ => None,
            });
    let Some(text) = observed.next() else {
        return Err(Withheld::NoUsableValue {
            input: key,
            reason: "the property carries no observed textual value".to_owned(),
        });
    };
    text.parse::<u64>()
        .ok()
        .filter(|bytes| *bytes != 0)
        .ok_or(Withheld::NoUsableValue {
            input: key,
            reason: format!("`{text}` is not a non-zero block size"),
        })
}
