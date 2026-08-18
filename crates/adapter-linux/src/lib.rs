//! The WP-L100 Linux read-only inventory adapter (increment 3a).
//!
//! The ordinary Linux client contract, as a pure library: a bounded read seam
//! over sysfs attribute files and the udev database, the MODEL-004
//! observations those reads produce, and the INV-003 reach declaration this
//! contract publishes about itself. Every rule about a value is decided above
//! the seam, so a Tier-1 fake drives each one and no test needs Linux, a
//! device, or a privilege to run.
//!
//! The reach format is documented in `schemas/adapter-linux/reach.md`, in the
//! `schemas/domain` shape: the document records a delivered format and decides
//! nothing.
//!
//! **What this adapter reads, and why only this.** The contract is the
//! ordinary client's — attribute files under the sysfs block class and the
//! udev database records — because those are the two **interfaces**
//! `docs/quality/observability.md` establishes as client-readable on real
//! hardware. Both are file reads. No block device is opened and no subprocess
//! is launched, at any privilege.
//!
//! The claim is about the interfaces, not about every field read through
//! them, and the difference is load-bearing: a field can be read through a
//! measured interface at a path no row measures. Exactly one is in that
//! position today — `device/serial`, whose path the 2026-08-04 sitting never
//! read and which ADR-0034 does not designate, the serial that sitting
//! observed having come from the USB device node instead by parent
//! traversal. `schemas/adapter-linux/fields.md` states
//! per field which observability row supports it and which are read without
//! one, so the gap is a recorded decision rather than an implied warrant.
//!
//! **Clamping is delivered here rather than deferred.** There is no
//! privilege-conditional branch anywhere in this crate: running as root
//! produces the same answer as running as anyone else. A contract that widened
//! with privilege would make the published INV-003 reach a per-user statement,
//! which INV-003 forbids — the reach is a property of the contract and the
//! platform. A Tier-1 source-text test holds it.
//!
//! What this crate deliberately is not:
//!
//! - **No identity record.** SAFE-003's `DeviceIdentity` carries a required
//!   partition-table state, and every variant of that field is a
//!   determination about the medium that INV-003 forbids a client which read
//!   no table bytes to assert. The record binds at validation from the
//!   helper's own re-discovery; this crate reports the material it is built
//!   from, attributed per observation, and derives no identity strength.
//! - **No partition-table state and no protection verdict.** Both are authored
//!   by the privileged helper at validation (ADR-0014, ADR-0016). This crate
//!   emits neither on any path, so the closure fails closed at exactly the
//!   position an authored value occupies rather than reading a client's guess.
//! - **No layered topology.** Devices are addressed and absorbed (increment
//!   3a), but nothing here builds a partition-table node, and therefore no
//!   partition, file system, signature, or volume node either.
//!   `NamingFields::PartitionTable` carries a `TableRole` — a scheme — and
//!   this contract reads no table bytes. ADR-0036's forward obligation put
//!   the choice to this increment in terms, and the package document records
//!   the branch taken and the measured grounds for it. The rest of the
//!   layered topology is increment 3b's, LIN-006's detection layer increment
//!   4's, and the CAP-004 runtime facts increment 5's. The engine that judges
//!   any of it is WP-050's.
//! - **No free-extent derivation.** INV-004 forbids presenting it "where the
//!   host declares a table scheme the build cannot name", and this contract
//!   declares none. It is offered as an explicit refusal rather than omitted,
//!   because an absent surface and a refusing one are different things to a
//!   consumer.
//! - **No snapshot and no protection facts.** Nothing here builds a
//!   `protection::Facts` or a Section 6 snapshot body. Addressing devices is
//!   the part ADR-0019 governs and increment 3a delivers; assembling a client
//!   draft around them needs the node kinds above.
//! - **No transport this build can positively name.** ADR-0018's answer is
//!   `Unrecognized` for every device, because its own fabric-versus-local
//!   discrimination rows are outstanding on every platform. Classifying
//!   values are recorded on Linux now (`ID_BUS=usb`), but a value names no
//!   class until those rows say which classes are local. It resolves to
//!   `Indeterminate` at the closure — never `Permitted`.
//! - **No sameness inference.** Two interfaces reporting one identifier
//!   produce two attributed observations; nothing here elects one, groups two
//!   rows under one device, or infers cross-path sameness (ADR-0011).
//! - **No user-facing surface.** This is a library. The CLI is WP-035's and
//!   WP-080's, and the diagnostic bundle is WP-035's.

/// This adapter's version, as MODEL-004's observations report it.
///
/// Its own, deliberately: an attribution helper that borrowed a sibling
/// crate's constant would silently attribute observations to the wrong
/// package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod contract;
pub mod derivation;
pub mod devices;
pub mod naming;
pub mod observation;
pub mod reach;

#[cfg(test)]
mod tests;
