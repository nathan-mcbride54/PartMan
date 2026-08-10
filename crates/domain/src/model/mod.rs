//! Section 5 domain model (WP-010 increment 3).
//!
//! Increment 3a delivers node naming per ADR-0019: derived positional
//! addresses ([`naming::derive_id`]) and collision-group absorption
//! ([`naming::absorb`]). Node payloads, edges, snapshots, and the typed
//! decode/validate/hash boundary land in later slices.

pub mod naming;
pub mod topology;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod topology_tests;
