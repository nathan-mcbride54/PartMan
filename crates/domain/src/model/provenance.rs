//! MODEL-004 provenance observations (WP-010 increment 3c, ADR-C4).
//!
//! Every discovered property records, in the artifact **envelope**, the set
//! of observations that produced it. The four confidence values are
//! **derived from that set and never stored** — there is no constructor
//! taking a confidence, so a record claiming `authoritative` while holding
//! two disagreeing reads is unrepresentable, which is ADR-C4's whole point.
//!
//! A positively observed *absence* is a value, not an unavailability
//! (ADR-C4): conflating them collapses a blank device and an unreadable one
//! into the same record, which PART-001 would then initialize alike.

use crate::canonical::{self, Value};

/// How an adapter obtained an observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Read directly from the platform interface the evidence contract
    /// names.
    Direct,
    /// Computed or guessed from indirect evidence; derives `inferred`.
    Heuristic,
}

/// One adapter's answer for one property (MODEL-004).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    /// The reporting adapter's name.
    pub adapter: String,
    /// The reporting adapter's version.
    pub adapter_version: String,
    /// How the answer was obtained.
    pub method: Method,
    /// What the adapter found.
    pub outcome: Outcome,
}

/// The outcome of one observation attempt (ADR-C4's vocabulary).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A value was observed.
    Observed {
        /// The observed value, as a canonical value.
        value: Value,
    },
    /// The property was positively observed to be absent — a value, not an
    /// unavailability.
    ObservedAbsent,
    /// The adapter could not determine the property.
    Unavailable {
        /// Why not.
        reason: String,
    },
    /// The read itself failed.
    Failed {
        /// The error, redacted per SAFE-006 by the caller.
        error: String,
    },
}

/// The observation set behind one discovered property.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PropertyObservations {
    /// The observations, in arrival order; order carries no meaning.
    pub observations: Vec<Observation>,
}

/// MODEL-004's four confidence values — derived, never stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    /// Exactly one observed value (or absence), directly read.
    Authoritative,
    /// The only observed value came from a heuristic method.
    Inferred,
    /// Nothing was observed.
    Unavailable,
    /// Two or more observations with distinct canonical encodings.
    Conflicting,
}

impl PropertyObservations {
    /// Derive the confidence ADR-C4 defines over this set.
    ///
    /// One directly observed value is `authoritative`; a heuristic-only
    /// observation is `inferred`; zero observed is `unavailable`; two or
    /// more observed with distinct canonical encodings is `conflicting`.
    /// Observed absence participates as a value with its own encoding, so
    /// an absence and a presence conflict rather than collapsing.
    ///
    /// # Errors
    ///
    /// [`canonical::Error`] if an observed value cannot be canonically
    /// encoded — a programming error surfaced rather than panicked.
    pub fn derive_confidence(&self) -> Result<Confidence, canonical::Error> {
        let mut encodings: Vec<Vec<u8>> = Vec::new();
        let mut any_direct = false;
        for observation in &self.observations {
            let encoded = match &observation.outcome {
                Outcome::Observed { value } => canonical::encode(value)?,
                // Absence is a value: give it an encoding no observed
                // value can produce (the empty byte string — every real
                // encoding is at least one byte).
                Outcome::ObservedAbsent => Vec::new(),
                Outcome::Unavailable { .. } | Outcome::Failed { .. } => continue,
            };
            if observation.method == Method::Direct {
                any_direct = true;
            }
            if !encodings.contains(&encoded) {
                encodings.push(encoded);
            }
        }
        Ok(match encodings.len() {
            0 => Confidence::Unavailable,
            1 => {
                if any_direct {
                    Confidence::Authoritative
                } else {
                    Confidence::Inferred
                }
            }
            _ => Confidence::Conflicting,
        })
    }
}
