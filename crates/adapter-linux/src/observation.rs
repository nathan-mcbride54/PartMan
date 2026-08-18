//! MODEL-004 observations, in the domain's own vocabulary.
//!
//! This module defines **no** observation type. `partman-domain`'s
//! `provenance` module already carries MODEL-004's shape — a per-property set
//! of observations, each naming its source adapter, that adapter's version,
//! the method, and an outcome — together with ADR-C4's rule that a confidence
//! is derived from the set and never stored. A second vocabulary beside it
//! would be a parallel style, and the two would drift. What lives here is the
//! producer: the interfaces this contract reads, and the one mapping from a
//! bounded read to a domain observation.
//!
//! **The interface identity rides the adapter name.** MODEL-004 asks for the
//! source adapter, its version, and the method used. The domain's `Method` is
//! a closed two-valued enum that the confidence derivation reads, so it cannot
//! also carry which interface answered without widening another package's
//! type. An adapter that reads two interfaces is two sources for provenance
//! purposes, so each observation names itself `partman-adapter-linux/<interface>`
//! and the roster of those names is closed here.

use partman_domain::canonical::Value;
use partman_domain::model::provenance::{Method, Observation, Outcome};

use crate::VERSION;
use crate::contract::AttributeRead;

/// The interfaces the ordinary-client contract reads.
///
/// Closed, and closed on purpose: an interface this crate cannot name is an
/// interface it does not read. Increment 2's field lists are drawn from the
/// first two; increment 4a's state tables from the third, which entered the
/// way the first two did — by an observability row (the 2026-08-18
/// detection-rows sitting, DR1 and DR2), never by documentation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    /// Attribute files under the sysfs block class, read by this client.
    Sysfs,
    /// The udev database records under the udev root — values root's udevd
    /// computed at device-add time and cached, not observations this client
    /// made.
    UdevDatabase,
    /// The kernel's own state tables under the procfs root — the mount table
    /// (`self/mountinfo`) and the swap table (`swaps`), read by this client
    /// (increment 4a). State-layer facts under MODEL-005's body-stability
    /// rule, never topology and never body content.
    Procfs,
}

impl Interface {
    /// The interface's compile-time label. Never caller-supplied.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sysfs => "linux-sysfs",
            Self::UdevDatabase => "linux-udev-db",
            Self::Procfs => "linux-procfs",
        }
    }

    /// MODEL-004's method for this interface, and a decision rather than a
    /// transcription.
    ///
    /// A sysfs attribute is read directly from the platform interface the
    /// evidence contract names, so it is [`Method::Direct`] and a single such
    /// observation derives `authoritative`. A udev-database value is not:
    /// udevd computed it from its own rules at device-add time and this
    /// client reads the cache, which is the domain's [`Method::Heuristic`] —
    /// "computed or guessed from indirect evidence" — and derives `inferred`.
    ///
    /// The conservative direction is deliberate. Calling a cached third-party
    /// computation `authoritative` would let one stale record outrank nothing,
    /// and MODEL-004's confidence is derived from these methods rather than
    /// stated, so this mapping is the whole of what makes the derivation
    /// honest. It is revisited only if a udev value is ever re-read from the
    /// interface it describes rather than from the cache.
    ///
    /// A procfs table is the kernel reporting its own current state to the
    /// reader — no third party computed it and nothing cached it — so it is
    /// [`Method::Direct`] like a sysfs attribute. What it reports is a state
    /// fact, and a state fact's staleness is CONC-003's concern, not this
    /// mapping's.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::Sysfs | Self::Procfs => Method::Direct,
            Self::UdevDatabase => Method::Heuristic,
        }
    }

    /// The MODEL-004 source-adapter name for this interface.
    #[must_use]
    pub fn adapter(self) -> String {
        format!("partman-adapter-linux/{}", self.label())
    }
}

/// Turn one bounded read into one MODEL-004 observation.
///
/// The mapping is the whole of ADR-C4's separation at this boundary, and each
/// arm is a claim:
///
/// - a read value is an observed value;
/// - an attribute that exists and is empty, and one that is not present under
///   an interface that answered, are both **positively determined absences** —
///   values, not unavailabilities, because the interface was asked and
///   answered;
/// - an over-limit read, a non-UTF-8 read, and a failed read are all `failed`:
///   the read itself did not produce an answer, and none of them is evidence
///   about the device.
///
/// Nothing maps to `unavailable` here. That outcome belongs to the interface
/// layer — a contract that did not answer at all — and it is reached through
/// [`observe_unavailable`], whose callers hold the interface-level evidence
/// that no answer came, never through an attribute read.
#[must_use]
pub fn observe(interface: Interface, read: &AttributeRead) -> Observation {
    let outcome = match read {
        AttributeRead::Text(text) => Outcome::Observed {
            value: Value::Text(text.clone()),
        },
        AttributeRead::Empty | AttributeRead::NotPresent => Outcome::ObservedAbsent,
        AttributeRead::OverLimit { seen } => Outcome::Failed {
            error: format!(
                "the attribute is {seen} bytes, over the {} byte limit, and was not truncated",
                crate::contract::VALUE_LIMIT
            ),
        },
        AttributeRead::NotText => Outcome::Failed {
            error: "the attribute is not UTF-8 and was not lossily converted".to_owned(),
        },
        AttributeRead::Failed { error } => Outcome::Failed {
            error: error.clone(),
        },
    };
    Observation {
        adapter: interface.adapter(),
        adapter_version: VERSION.to_owned(),
        method: interface.method(),
        outcome,
    }
}

/// One observation recording that an interface did not answer.
///
/// This is the ADR-C4 arm an attribute read can never reach, and the
/// distinction it protects is the whole point: a key missing from a record
/// that exists is a positively determined **absence**, while every key of a
/// record that does not exist is **unavailable**, because calling those
/// absent would claim the interface answered and said nothing.
#[must_use]
pub fn observe_unavailable(interface: Interface, reason: &str) -> Observation {
    Observation {
        adapter: interface.adapter(),
        adapter_version: VERSION.to_owned(),
        method: interface.method(),
        outcome: Outcome::Unavailable {
            reason: reason.to_owned(),
        },
    }
}
