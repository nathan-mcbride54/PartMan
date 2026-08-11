//! The CAP-006 store's schema, as types (WP-050 increment 3).
//!
//! `docs/capabilities/` holds the qualification store and the per-tool
//! version floors (`docs/capabilities/format.md`). This module is the
//! schema those documents must satisfy, and the Tier-1 store test in
//! `store_tests.rs` is the CI gate that holds them to it — the
//! `shared_vectors` pattern: the repository data is validated by a test
//! that fails the build on a malformed or silently-qualified row.
//!
//! **What deliberately does not exist here: any evidence mint at all.**
//! The assignment's increment-3 sentence gives the evidence token "its
//! one constructor: loading a qualifying row from this store" — and this
//! increment delivers narrower than that grant: no constructor, because
//! both of its preconditions are vacuous. The store's advertised set is
//! empty (no row exists to qualify), and no consumer exists that could
//! possess a store document at runtime (the shipped engine never reads
//! the repository). A loading path today would be an API whose every
//! honest call fails; it arrives with the first consumer that embeds
//! qualification evidence at its own build boundary, under its own
//! grant, against rows that actually exist. Until then
//! [`QualificationEvidence`](super::QualificationEvidence) stays
//! unmintable everywhere — the increment-1 `compile_fail` proof
//! continues to hold verbatim, and the narrowing is strictly more
//! conservative than the grant, recorded in the CHANGELOG.

/// The qualification store's schema identity (MODEL-003).
pub const STORE_SCHEMA: &str = "partman.capability.qualification-store";
/// The current store schema version.
pub const STORE_SCHEMA_VERSION: u64 = 1;
/// The floors document's schema identity (MODEL-003).
pub const FLOORS_SCHEMA: &str = "partman.capability.tool-version-floors";
/// The current floors schema version.
pub const FLOORS_SCHEMA_VERSION: u64 = 1;

/// The CAP-002 operation names as the store spells them, kebab-case, in
/// the requirement's own order. The store test refuses a row naming
/// anything else.
pub const OPERATION_NAMES: &[&str; 14] = &[
    "detect", "read", "create", "grow", "shrink", "move", "copy", "check", "repair", "label",
    "uuid", "encrypt", "decrypt", "wipe",
];

/// The Section 9 platform labels as the store spells them, verbatim from
/// the floors table's platform column.
pub const PLATFORM_LABELS: &[&str; 5] =
    &["windows-11", "windows-10", "macos", "debian-ubuntu", "arch"];
