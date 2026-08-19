//! The WP-L100 Linux read-only inventory adapter (increment 5a).
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
//! ordinary client's — attribute files under the sysfs block class, the udev
//! database records, and (since increment 4a) the kernel's procfs mount and
//! swap tables — because those are the three **interfaces**
//! `docs/quality/observability.md` establishes as client-readable on a real
//! host (the third by the 2026-08-18 detection-rows sitting, DR1 and DR2).
//! All are file reads. No block device is opened and no subprocess is
//! launched, at any privilege.
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
//!   partition, file system, signature, or volume node either. Host-assembled
//!   block nodes — device-mapper, mdraid, loop — are **recognised and
//!   withdrawn** from the physical-device set (increment 4a, on DR3's
//!   markers): reported with their kind, named nothing, not operands, until
//!   a naming-designation round says which `NamingFields` kind each is and
//!   from which source it names. That round found no source it may
//!   designate on today's rows, so increment 4b's first slice (`arrays`)
//!   builds the one thing that needs no designation: each mdraid array as
//!   a **designator-absent** `Aggregate` — indeterminate, not an operand
//!   (ADR-0019; WP-010 slice 3q) — with its self-reported member count and
//!   its membership listing reported, not edged. The second slice names
//!   what ADR-0053 designates: an mdraid array from `md/uuid`, an LVM
//!   logical volume from `dm/name` under its designator-absent
//!   volume-group aggregate (`volumes`), each verbatim through the
//!   bytes-preserving path; a dm-crypt mapping and a loop device are
//!   reported and not named. **No `BackingSignature`, no `Backing` edge and
//!   no `EncryptionLayer` is built here, and none is waited for** (the
//!   member-signature offset round, 2026-08-18): no client interface
//!   reports a signature's offset (DR14) and the family is cache-only per
//!   member, so both naming fields are the helper's byte layer's and the
//!   nodes arrive at HLP-002's re-discovery. The third slice reports what
//!   the client does read instead (`held`): a whole device's **held**
//!   standing from sysfs `holders/`, keyed by the holder's own uuid (DR15:
//!   live from both ends; entry names moved, identities held), a
//!   state-layer observation that enters no name and changes no verdict
//!   (its consumer is the helper's capture, WP-L110); and the
//!   cached signature view (`ID_FS_TYPE`/`ID_FS_USAGE`/`ID_FS_VERSION`),
//!   reported as `Heuristic`/`inferred` and consulted by nothing.
//!   `NamingFields::PartitionTable` carries a `TableRole` — a scheme — and
//!   this contract reads no table bytes. ADR-0036's forward obligation put
//!   the choice to this increment in terms, and the package document records
//!   the branch taken and the measured grounds for it. The rest of the
//!   layered topology is increment 3b's, LIN-006's detection layer's
//!   topology half increment 4b's, and the CAP-004 runtime facts increment
//!   5's. The engine that judges any of it is WP-050's.
//! - **No mount node and no swap node.** The mount and swap tables are read
//!   (increment 4a, `state`) and reported as attributed state-layer
//!   observations keyed to admitted devices by `major:minor` — MODEL-005's
//!   body-stability rule and ADR-0005 Rule 2 place them in the envelope, so
//!   they are never topology and never body content (gitea#1004). The
//!   Section 5 `Mount` type is WP-010's and arrives with its first consumer.
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
//! - **No tool launched, and no tool needed.** The capability seam
//!   (`runtime`, increment 5a) produces WP-050's CAP-004 `RuntimeFacts`
//!   for the source-class operations this adapter serves — an empty tool
//!   roster for each, pinned by test, and the ACC-009 mapping from a
//!   caller-supplied probe to the engine's tool state — and answers a
//!   typed refusal for mutating operations, whose tools are WP-L110's to
//!   state. Probes come from the package that launches (WP-035's doctor);
//!   the Section 9 floor determination is increment 5b's.
//! - **No user-facing surface.** This is a library. The CLI is WP-035's and
//!   WP-080's, and the diagnostic bundle is WP-035's.

/// This adapter's version, as MODEL-004's observations report it.
///
/// Its own, deliberately: an attribution helper that borrowed a sibling
/// crate's constant would silently attribute observations to the wrong
/// package version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod arrays;
pub mod contract;
pub mod derivation;
pub mod devices;
pub mod held;
pub mod naming;
pub mod observation;
pub mod reach;
pub mod runtime;
pub mod state;
pub mod volumes;

#[cfg(test)]
mod tests;
