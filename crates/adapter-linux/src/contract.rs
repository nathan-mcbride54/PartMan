//! The ordinary-client contract's read seam and its bounded-read discipline.
//!
//! Two primitives sit above an injected seam: one bounded directory listing
//! and one bounded attribute read. Both refuse rather than truncate, and both
//! keep ADR-C4's three answers apart — a value, a positively determined
//! absence, and a non-answer.
//!
//! **Everything that can be wrong is decided above the seam.** WP-035's
//! precedent enforces its per-value byte bound inside the production
//! implementation, where a Tier-1 fake cannot reach it, so the bound it
//! declares has no test. This seam therefore returns raw bytes and applies
//! every rule — the byte bound, the UTF-8 requirement, the trailing-newline
//! rule — in this module, where the fake drives each one. The variation is
//! deliberate and the tests are the reason for it.
//!
//! **Nothing here consults privilege.** There is no branch on user, group, or
//! `PermissionDenied`: a permission error travels the ordinary failure arm
//! like any other. A contract that widened with privilege would make the
//! published INV-003 reach a per-user statement, which INV-003 forbids.

use std::path::{Path, PathBuf};

/// Fail-closed bound on one directory listing.
///
/// Exceeding it refuses rather than truncating, so a partial listing is never
/// mistaken for a complete one. This is the device-count bound WP-L100's
/// assignment names, applied at the listing primitive because that is the
/// layer where a count exists: increment 2's whole-device enumeration inherits
/// it by listing the block-device class through this function.
pub const ENTRY_LIMIT: usize = 512;

/// Fail-closed bound on one attribute read, in bytes.
///
/// Identifier strings are short; anything larger is a surface behaving
/// unexpectedly and is refused. Refused, not truncated: a prefix is
/// byte-for-byte indistinguishable from a complete read of that length, so
/// truncation would hand a caller a partial answer wearing a whole answer's
/// shape.
pub const VALUE_LIMIT: usize = 4096;

/// The compiled sysfs root the production contract reads.
#[must_use]
pub fn sysfs_root() -> PathBuf {
    PathBuf::from("/sys")
}

/// The compiled udev-database root the production contract reads.
#[must_use]
pub fn udev_root() -> PathBuf {
    PathBuf::from("/run/udev/data")
}

/// The compiled procfs root the production contract reads (increment 4a).
///
/// The third interface, entered the way the first two were — by a row: the
/// DR1/DR2 cells of the 2026-08-18 detection-rows sitting establish that
/// `/proc/self/mountinfo` and `/proc/swaps` are client-readable in the
/// documented shape.
#[must_use]
pub fn procfs_root() -> PathBuf {
    PathBuf::from("/proc")
}

/// The compiled root of the OS-release interface the production contract
/// reads (increment 5b): the directory holding `os-release`.
///
/// The fourth interface, entered the way the first three were — by a row:
/// DR16 (jammy) and DR18 (the first Arch guest) of the 2026-08-19
/// floor-input sitting establish that `/etc/os-release` is a
/// client-readable file (a symlink to `/usr/lib/os-release` on both tiers)
/// in the documented `KEY=value` shape.
#[must_use]
pub fn os_release_root() -> PathBuf {
    PathBuf::from("/etc")
}

/// The injected read seam.
///
/// Shaped on WP-035's `DeviceSource` rather than a second idiom: object-safe,
/// `&self`, no generics, so callers take `&dyn`. It returns **bytes**, not
/// text, because every rule this crate declares about a value — its byte
/// bound, its encoding, its trailing newline — is then decided above the seam
/// and is reachable by a Tier-1 fake.
///
/// Both roots are parameters of the functions below rather than values this
/// trait knows, which is what lets a fake point at a synthesized tree without
/// any environment read. No implementation may consult the environment.
pub trait ContractSource {
    /// List one directory, sorted, so enumeration order is reproducible
    /// rather than filesystem-dependent.
    ///
    /// # Errors
    ///
    /// The directory is absent, or the listing itself failed. The caller keeps
    /// those apart: an absent interface is `unavailable`, a failed read is
    /// `failed`, and neither is ever rendered as an empty listing.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error>;

    /// Read one attribute file whole, as bytes.
    ///
    /// # Errors
    ///
    /// The file is absent — [`std::io::ErrorKind::NotFound`], which the caller
    /// reads as a positively determined absence — or the read itself failed.
    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error>;
}

/// The production contract: bounded file reads and one sorted directory
/// listing. It opens no device node and launches nothing.
pub struct SystemContractSource;

impl ContractSource for SystemContractSource {
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(path)? {
            names.push(entry?.file_name().to_string_lossy().into_owned());
        }
        names.sort();
        Ok(names)
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(path)
    }
}

/// Evidence that an interface answered.
///
/// ADR-C4's separation is only decidable with this fact. A missing attribute
/// under an interface that answered is a positively determined absence; the
/// same missing attribute under an interface that never answered is an
/// unavailability, because absence there would claim the interface spoke.
///
/// So the token is the evidence, and it is unforgeable. Exactly two
/// operations produce one, and both are operations that *establish* an
/// interface answered: [`list_bounded`] returning [`Listing::Listed`], and
/// [`read_record`] returning [`RecordRead::Present`].
///
/// ```compile_fail
/// // The field is private and no function returns the type into existence,
/// // so a caller cannot assert that an interface answered.
/// let answered = partman_adapter_linux::contract::InterfaceAnswered(());
/// ```
pub struct InterfaceAnswered(());

/// The outcome of one bounded directory listing.
pub enum Listing {
    /// The interface answered, with its sorted entries and the evidence that
    /// it answered.
    Listed {
        /// The sorted entry names.
        entries: Vec<String>,
        /// Evidence for ADR-C4's absence arm.
        answered: InterfaceAnswered,
    },
    /// The listing exceeded [`ENTRY_LIMIT`] and was **not** truncated.
    OverLimit {
        /// How many entries were seen.
        seen: usize,
    },
    /// The platform did not expose the interface. Distinct from an empty
    /// listing: "there is no such interface here" is not "this machine has
    /// nothing", and rendering it as an empty listing is the fail-closed
    /// violation SAFE-005 exists to prevent.
    Unavailable {
        /// Why the interface could not answer.
        reason: String,
    },
    /// The listing itself failed.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
}

/// List one interface directory under [`ENTRY_LIMIT`].
#[must_use]
pub fn list_bounded(source: &dyn ContractSource, path: &Path) -> Listing {
    match source.list_dir(path) {
        Ok(entries) if entries.len() > ENTRY_LIMIT => Listing::OverLimit {
            seen: entries.len(),
        },
        Ok(entries) => Listing::Listed {
            entries,
            answered: InterfaceAnswered(()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Listing::Unavailable {
            reason: "the interface directory is not present on this host".to_owned(),
        },
        Err(error) => Listing::Failed {
            error: error.to_string(),
        },
    }
}

/// The outcome of one bounded attribute read.
///
/// The two absence arms are separate values rather than one, because they are
/// established differently and a reader may care which: an attribute that
/// exists and is empty was read, and one that is not present was looked for.
pub enum AttributeRead {
    /// The attribute was read: its bytes as text, with exactly one trailing
    /// newline stripped.
    Text(String),
    /// The attribute exists and is empty — a positively determined absence.
    Empty,
    /// The attribute is not present under an interface that answered — a
    /// positively determined absence.
    NotPresent,
    /// The read exceeded [`VALUE_LIMIT`] and was **not** truncated.
    OverLimit {
        /// How many bytes were seen.
        seen: usize,
    },
    /// The bytes are not UTF-8. A mangled identifier is not the value the
    /// interface reported, so this refuses rather than converting lossily.
    NotText,
    /// The read itself failed.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
}

/// Read one attribute under [`VALUE_LIMIT`].
///
/// The token argument is not consulted; it is required so that a caller
/// cannot reach [`AttributeRead::NotPresent`]'s absence reading without the
/// evidence that makes it an absence rather than an unavailability.
#[must_use]
pub fn read_attribute(
    source: &dyn ContractSource,
    path: &Path,
    _answered: &InterfaceAnswered,
) -> AttributeRead {
    let raw = match source.read_bytes(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return AttributeRead::NotPresent;
        }
        Err(error) => {
            return AttributeRead::Failed {
                error: error.to_string(),
            };
        }
    };
    if raw.len() > VALUE_LIMIT {
        return AttributeRead::OverLimit { seen: raw.len() };
    }
    let Ok(text) = String::from_utf8(raw) else {
        return AttributeRead::NotText;
    };
    // Exactly one trailing newline, and nothing else. Trimming all trailing
    // whitespace turns a padded SCSI vendor — `"ATA     "` — into an empty
    // string, which then reads as a positively determined absence of a
    // vendor. The attribute positively contained padding; that is an ADR-C4
    // violation, and WP-035 records having made it.
    let text = text.strip_suffix('\n').unwrap_or(&text).to_owned();
    if text.is_empty() {
        return AttributeRead::Empty;
    }
    AttributeRead::Text(text)
}

/// The outcome of one bounded, **bytes-preserving** read.
///
/// This is [`AttributeRead`] with its three transformations removed and one
/// arm consequently unreachable. [`read_attribute`] validates UTF-8, refuses
/// non-text as [`AttributeRead::NotText`], and strips one trailing newline;
/// each is correct for the text-shaped observation rows it serves and each is
/// unlawful for a name, because ADR-0019 takes identifier bytes
/// contract-source-verbatim and excludes the transformation class wholesale.
/// So there is no `NotText` here: non-UTF-8 bytes are a legal name.
///
/// The two absence arms stay separate for the reason they are separate above
/// — an attribute that exists and is empty was read, and one that is not
/// present was looked for — even though ADR-0034 gives both the same naming
/// consequence. Folding them here would destroy the distinction at the seam
/// rather than at the layer that decides it.
pub enum NamingRead {
    /// The source answered: its bytes, verbatim. The trailing newline is
    /// **included**, per ADR-0034 — stripping it is a transformation with an
    /// undecidable edge, since a value may legitimately end in `0x0a`.
    Bytes(Vec<u8>),
    /// The source exists and is empty — a positively determined absence.
    Empty,
    /// The source is not present under an interface that answered — a
    /// positively determined absence.
    NotPresent,
    /// The read exceeded [`VALUE_LIMIT`] and was **not** truncated.
    OverLimit {
        /// How many bytes were seen.
        seen: usize,
    },
    /// The read itself failed.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
}

/// Read one naming source under [`VALUE_LIMIT`], preserving its bytes.
///
/// ADR-0034: "The contract owes a bytes read seam before any naming input is
/// consumed; that is WP-L100 increment 3's first delivery obligation." This is
/// it. Every naming input flows through here and none through
/// [`read_attribute`], which remains correct for what it was built for.
///
/// The token argument is required for the same reason [`read_attribute`]
/// requires it, and the reason bites harder here: ADR-0034 gives a measured
/// absence and a failed read **different** naming outcomes — the first leaves
/// an operand with a weaker name, the second an indeterminate non-operand — so
/// a caller who could reach [`NamingRead::NotPresent`] without the evidence
/// that the interface answered could turn a failed read into an operand.
#[must_use]
pub fn read_naming_source(
    source: &dyn ContractSource,
    path: &Path,
    _answered: &InterfaceAnswered,
) -> NamingRead {
    let raw = match source.read_bytes(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return NamingRead::NotPresent;
        }
        Err(error) => {
            return NamingRead::Failed {
                error: error.to_string(),
            };
        }
    };
    if raw.len() > VALUE_LIMIT {
        return NamingRead::OverLimit { seen: raw.len() };
    }
    if raw.is_empty() {
        return NamingRead::Empty;
    }
    NamingRead::Bytes(raw)
}

/// The outcome of reading one record file.
///
/// A record file is not an attribute: its absence means the interface holds
/// nothing for this subject, which is an unavailability, never a value's
/// absence. That is why [`RecordRead::NoRecord`] is a distinct variant from
/// [`AttributeRead::NotPresent`] and why this operation, rather than
/// consuming an [`InterfaceAnswered`], **produces** one.
pub enum RecordRead {
    /// The interface holds a record, with the evidence that it answered.
    Present {
        /// The record's text, with exactly one trailing newline stripped.
        text: String,
        /// Evidence for ADR-C4's absence arm over this record's keys.
        answered: InterfaceAnswered,
    },
    /// The interface holds no record for this subject. Every value the
    /// record would have carried is `unavailable`, never absent: absence
    /// would claim the interface answered and said nothing.
    NoRecord,
    /// The record exceeded [`RECORD_LIMIT`] and was **not** truncated.
    OverLimit {
        /// How many bytes were seen.
        seen: usize,
    },
    /// The bytes are not UTF-8.
    NotText,
    /// The read itself failed.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
}

/// Fail-closed bound on one record read, in bytes.
///
/// A record holds many keys where an attribute holds one, so it has its own
/// bound rather than borrowing [`VALUE_LIMIT`]. Exceeding it refuses: a
/// truncated record would drop keys silently, and a dropped key is
/// indistinguishable from one the interface never carried.
pub const RECORD_LIMIT: usize = 65_536;

/// Fail-closed bound on one state-table read, in bytes (increment 4a).
///
/// The mount table is a record with one line per mount rather than one
/// device's keys, and a container host can carry hundreds of lines, so it
/// has its own bound rather than borrowing [`RECORD_LIMIT`]. Exceeding it
/// refuses: a truncated table would drop mounts silently, and a dropped
/// mount is indistinguishable from an unmounted device — the fail-open
/// SAFE-005 exists to prevent.
pub const TABLE_LIMIT: usize = 1 << 20;

/// Read one record file under [`RECORD_LIMIT`].
#[must_use]
pub fn read_record(source: &dyn ContractSource, path: &Path) -> RecordRead {
    read_record_bounded(source, path, RECORD_LIMIT)
}

/// Read one state table under [`TABLE_LIMIT`]. The same three answers as
/// [`read_record`], because a table that is not present is an interface
/// that did not answer, never an empty table.
#[must_use]
pub fn read_table(source: &dyn ContractSource, path: &Path) -> RecordRead {
    read_record_bounded(source, path, TABLE_LIMIT)
}

fn read_record_bounded(source: &dyn ContractSource, path: &Path, limit: usize) -> RecordRead {
    let raw = match source.read_bytes(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return RecordRead::NoRecord;
        }
        Err(error) => {
            return RecordRead::Failed {
                error: error.to_string(),
            };
        }
    };
    if raw.len() > limit {
        return RecordRead::OverLimit { seen: raw.len() };
    }
    let Ok(text) = String::from_utf8(raw) else {
        return RecordRead::NotText;
    };
    RecordRead::Present {
        text: text.strip_suffix('\n').unwrap_or(&text).to_owned(),
        answered: InterfaceAnswered(()),
    }
}
