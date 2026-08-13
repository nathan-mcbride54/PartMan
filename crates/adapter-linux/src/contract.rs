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
/// So the token is the evidence, and it is unforgeable: only
/// [`list_bounded`] returning [`Listing::Listed`] produces one.
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
