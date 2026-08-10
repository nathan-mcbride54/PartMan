//! Section 5 domain model (WP-010 increment 3).
//!
//! Increment 3a delivers node naming per ADR-0019: derived positional
//! addresses ([`naming::derive_id`]) and collision-group absorption
//! ([`naming::absorb`]). Increment 3b adds the five edge kinds with their
//! semantics classes and fail-closed topology construction
//! ([`topology::Topology::build`]). Increment 3c adds the snapshot body
//! and envelope with the typed decode/validate/hash boundary
//! ([`snapshot::TopologySnapshot::from_canonical_body`]) and MODEL-004
//! provenance ([`provenance::PropertyObservations`]). Identity records,
//! verdicts, plans, and their constructors land in later slices.

pub mod capability;
pub mod identity;
pub mod naming;
pub mod protection;
pub mod provenance;
pub mod snapshot;
pub mod step;
pub mod topology;

#[cfg(test)]
mod capability_tests;
#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod protection_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod step_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod topology_tests;
