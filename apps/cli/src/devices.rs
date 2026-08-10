//! Unprivileged whole-device enumeration, and the seam it reads through.
//!
//! WP-035's charter governs every line here: *prints raw identifier strings
//! labelled by reporting interface; computes no strength, table state, hash,
//! verdict, or plan.* Nothing in this module interprets a value, elects a
//! canonical identifier, groups devices, or says what any medium contains.
//!
//! **What it reads, and why only this.** The Linux contract is the ordinary
//! client's: `/sys/class/block` attributes and the udev database under
//! `/run/udev/data`. Both are file reads. No block device is opened, no
//! subprocess is launched, and the crate's dependency closure stays empty.
//! The interfaces are the ones `docs/quality/observability.md` establishes as
//! client-readable on real hardware — nothing here reaches for a surface the
//! record did not measure.
//!
//! **Clamping is delivered here rather than deferred.** The adapter reads the
//! ordinary-client contract and has no privilege-conditional branch. The
//! record measures why that matters: adding a user to `disk` "grants both raw
//! reads and `blkid -p`", so a contract that widened with privilege would make
//! the published INV-003 reach declaration a per-user statement, which
//! INV-003 forbids — it is a property of the contract and the platform.
//! Running this as root produces the same answer as running it as anyone
//! else, and a Tier-1 test holds that.
//!
//! **What it deliberately does not read**, each because the record or the
//! register says so:
//!
//! - **No `ID_FS_*` signature fields.** The increment 6 matrix measured that
//!   projection giving a confident single wrong-ish answer — naming a live
//!   ext4 `linux_raid_member` on the stale-signature fixture. That is the
//!   verdict layer's material — SI-34's measured case, resolved by
//!   ADR-0016 into the helper-authored hashed body — not an inventory's.
//! - **No `ID_PART_ENTRY_*`, no `start`, no partition children.** Partition
//!   enumeration is INV-004, and partition-table state is helper-authored
//!   under ADR-0014 — never this client's to compute.
//! - **No `/dev` node is opened at any point**, which is what keeps
//!   `docs/quality/test-tiers.md`'s standing claim true: reading
//!   `/sys/class/block/sda/size` is a sysfs attribute file, not the device.
//! - **No device-mapper or loop interpretation**, and no cross-device
//!   grouping — grouping two interfaces' rows under one device would be a
//!   sameness inference ADR-0011 reaches.

use std::path::{Path, PathBuf};

use crate::inspect::{Attribution, Observation, ObservedValue, Outcome};

/// Fail-closed bound on how many devices one enumeration will report.
/// Exceeding it refuses rather than truncating, so a partial list is never
/// mistaken for a complete one.
pub const DEVICE_LIMIT: usize = 512;

/// Fail-closed bound on one attribute read, in bytes. Identifier strings are
/// short; anything larger is a surface behaving unexpectedly and is refused.
pub const VALUE_LIMIT: usize = 4096;

/// One `(interface, native property name, raw value)` triple, exactly as the
/// interface reported it.
///
/// No normalized field name exists here and nothing elects "the" serial: two
/// interfaces reporting a serial produce two rows under two native property
/// names, and the reader decides what to make of that. The register measured
/// four distinct identifier strings from a single unprivileged Windows class
/// with "nothing normative" saying which is stable, which is why this shape
/// refuses to choose.
pub struct RawField {
    /// The reporting interface, compile-time and never caller-supplied.
    pub interface: &'static str,
    /// The attribution method carried on the observation: how the value
    /// was obtained, including any in-band caveat (the udev rows carry
    /// theirs here). Compile-time, set by the adapter that read the field.
    pub method: &'static str,
    /// That interface's own property name, verbatim.
    pub property: String,
    /// How the value ended, in ADR-C4's vocabulary.
    pub outcome: Outcome,
}

/// One enumerated whole device, under a session-local selector.
pub struct Device {
    /// The session-local selector — `device:0`, `device:1`. **Unstable across
    /// runs by construction**: it is a position in this enumeration and
    /// nothing else. A stable device handle stays absent — ADR-0019's
    /// derived addresses are WP-010's landed types, unconsumed by this
    /// chassis — and this is the session-local index the boundary permits
    /// in its place.
    pub selector: String,
    /// The kernel's own name for the node, as the directory entry spelled it.
    /// Reported as a raw string like any other, never as an identity.
    pub kernel_name: String,
    /// Every raw field this contract read for the device, in a fixed order.
    pub fields: Vec<RawField>,
}

/// How one enumeration ended.
pub enum Enumeration {
    /// The contract answered. Devices in the order the interface listed them,
    /// after a sort that makes the answer reproducible.
    Listed(Vec<Device>),
    /// The platform did not expose the interface. Distinct from an empty
    /// list: "there is no such interface here" is not "this machine has no
    /// disks", and rendering it as an empty list is the fail-closed violation
    /// SAFE-005 exists to prevent.
    Unavailable {
        /// Why the platform could not expose it.
        reason: String,
    },
    /// The read itself errored.
    Failed {
        /// The error, as the operating system reported it.
        error: String,
    },
    /// More than [`DEVICE_LIMIT`] entries. Refused, never truncated.
    OverLimit {
        /// How many were seen before refusing.
        seen: usize,
    },
}

/// What enumeration needs from the operating system, as a seam.
///
/// Shaped on the doctor's [`crate::doctor::ToolLauncher`] rather than a second
/// idiom: object-safe, `&self`, no generics. Tier 1 injects a fake over a
/// synthesized tree, so no Tier-1 test reads the host's real `/sys` or
/// `/run/udev`.
pub trait DeviceSource {
    /// List the entries directly under this directory, sorted.
    ///
    /// # Errors
    ///
    /// The directory is absent, or the listing itself failed. The caller keeps
    /// those apart: an absent block class is `unavailable`, a failed read is
    /// `failed`, and neither is ever rendered as an empty device list.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error>;

    /// Read one attribute file whole, bounded by [`VALUE_LIMIT`].
    ///
    /// # Errors
    ///
    /// The attribute is absent, or the read failed. ADR-C4 keeps those
    /// distinct — a positively determined absence is a value, a failed read is
    /// not — so this returns the error kind rather than flattening either into
    /// an empty string.
    fn read_value(&self, path: &Path) -> Result<String, std::io::Error>;
}

/// The real source: bounded file reads and one directory listing. It opens no
/// device node and launches nothing.
pub struct SystemDeviceSource;

impl DeviceSource for SystemDeviceSource {
    fn list_dir(&self, path: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_owned());
            }
        }
        names.sort();
        Ok(names)
    }

    fn read_value(&self, path: &Path) -> Result<String, std::io::Error> {
        let raw = std::fs::read(path)?;
        // Refuse rather than truncate. An earlier version sliced at
        // VALUE_LIMIT and returned the prefix, which is byte-for-byte
        // indistinguishable in the output from a complete read of that
        // length — a partial answer mistaken for a whole one, which is what
        // the sibling DEVICE_LIMIT refuses in terms.
        if raw.len() > VALUE_LIMIT {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "attribute exceeds the value limit and was not truncated",
            ));
        }
        // Non-UTF-8 is refused for the same reason: `from_utf8_lossy` would
        // substitute U+FFFD silently, and `device/wwid` and `ID_SERIAL` are
        // not guaranteed UTF-8. A mangled identifier is not the raw string
        // the interface reported, which the charter requires.
        let text = String::from_utf8(raw).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "attribute is not UTF-8")
        })?;
        // Strip the single trailing newline sysfs appends, and nothing else.
        // Trimming all trailing whitespace turned a padded SCSI vendor —
        // `"ATA     "` — into an empty string, which `read_outcome` then
        // reported as a positively determined absence. That is an ADR-C4
        // violation: the attribute positively contained padding.
        Ok(text.strip_suffix('\n').unwrap_or(&text).to_owned())
    }
}

/// The sysfs attributes this contract reads, as `(property, relative path)`.
///
/// `size` is in 512-byte sectors regardless of the device's own block size —
/// a kernel convention, reported raw and uninterpreted like everything else.
const SYSFS_FIELDS: &[(&str, &str)] = &[
    ("size", "size"),
    ("ro", "ro"),
    ("removable", "removable"),
    ("logical_block_size", "queue/logical_block_size"),
    ("physical_block_size", "queue/physical_block_size"),
    ("device/vendor", "device/vendor"),
    ("device/model", "device/model"),
    ("device/serial", "device/serial"),
    ("device/wwid", "device/wwid"),
];

/// The udev-database keys this contract reads.
///
/// Every one of these is **a cached value that root's `udevd` computed at
/// device-add time, not something the client observed** — the record states
/// that directly, and the caveat travels in-band on every value rather than
/// living only in this comment.
const UDEV_KEYS: &[&str] = &[
    "ID_SERIAL",
    "ID_SERIAL_SHORT",
    "ID_WWN",
    "ID_WWN_WITH_EXTENSION",
    "ID_BUS",
    "ID_PATH",
];

/// The sysfs interface label carried on every attribute row.
const SYSFS: &str = "linux-sysfs";

/// The udev-database interface label.
const UDEV: &str = "linux-udev-db";

/// The caveat carried in-band on every udev-database value.
const UDEV_CAVEAT: &str =
    "cached value computed by root's udevd at device-add time, not observed by this client";

/// The method statement carried on every sysfs value.
const SYSFS_METHOD: &str = "bounded read of a sysfs attribute file";

/// This adapter's attribution, on every observation it makes.
fn attribution(interface: &'static str, method: &'static str) -> Attribution {
    Attribution {
        adapter: interface,
        version: crate::VERSION,
        method,
    }
}

/// Enumerate whole devices through the injected source.
///
/// Linux only. Every other platform answers `Unavailable`, which is the honest
/// shape: this package has no contract there yet, and an empty list would say
/// the machine has no disks.
pub fn enumerate(source: &dyn DeviceSource, sysfs_root: &Path, udev_root: &Path) -> Enumeration {
    if !cfg!(target_os = "linux") {
        return Enumeration::Unavailable {
            reason: "this package reads no device interface on this platform yet".to_owned(),
        };
    }

    let block = sysfs_root.join("class/block");
    let names = match source.list_dir(&block) {
        Ok(names) => names,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Enumeration::Unavailable {
                reason: "sysfs block class is not present".to_owned(),
            };
        }
        Err(error) => {
            return Enumeration::Failed {
                error: error.to_string(),
            };
        }
    };

    // Whole devices only. A node is whole when its `partition` attribute is
    // **positively absent** — `NotFound` and nothing else.
    //
    // An earlier version tested `.is_ok()`, which fails open: any read error
    // on `sda1/partition` — a masked `/sys` in a container, an LSM policy —
    // promoted the partition into the whole-device list, where its own sector
    // count would be reported as a device capacity. That is INV-004-shaped
    // output from a package that declares partition rows a non-goal, produced
    // by an unchecked error, and it is the one place in this module where the
    // ADR-C4 distinction decides what gets reported at all.
    let mut whole = Vec::new();
    for name in names {
        let dir = block.join(&name);
        // Only a positively determined absence admits a node. Everything
        // else is skipped: an attribute that exists means a partition, and a
        // read we could not make fails closed — a device omitted from an
        // inventory is recoverable, a partition presented as a device is not.
        let attribute = source.read_value(&dir.join("partition"));
        if matches!(&attribute, Err(error) if error.kind() == std::io::ErrorKind::NotFound) {
            whole.push(name);
        }
    }

    if whole.len() > DEVICE_LIMIT {
        return Enumeration::OverLimit { seen: whole.len() };
    }

    let devices = whole
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let dir = block.join(&name);
            let mut fields = Vec::new();

            for (property, relative) in SYSFS_FIELDS {
                fields.push(RawField {
                    interface: SYSFS,
                    method: SYSFS_METHOD,
                    property: (*property).to_owned(),
                    outcome: read_outcome(source, &dir.join(relative)),
                });
            }

            for (property, outcome) in udev_fields(source, udev_root, &dir) {
                fields.push(RawField {
                    interface: UDEV,
                    method: UDEV_CAVEAT,
                    property,
                    outcome,
                });
            }

            Device {
                selector: selector(index),
                kernel_name: name,
                fields,
            }
        })
        .collect();

    Enumeration::Listed(devices)
}

/// The session-local selector for one enumerated device.
#[must_use]
pub fn selector(index: usize) -> String {
    format!("device:{index}")
}

/// Read one attribute into ADR-C4's outcome vocabulary.
///
/// The three-way distinction is the point: a file that is absent is a
/// positively determined absence and therefore a **value**; a file that exists
/// but will not read is `failed`; and neither is ever rendered as the other.
fn read_outcome(source: &dyn DeviceSource, path: &Path) -> Outcome {
    match source.read_value(path) {
        Ok(value) if value.is_empty() => Outcome::Observed(ObservedValue::Absent {
            reason: "the attribute exists and is empty".to_owned(),
        }),
        Ok(value) => Outcome::Observed(ObservedValue::Decimal(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Outcome::Observed(ObservedValue::Absent {
                reason: "the attribute is not present for this device".to_owned(),
            })
        }
        Err(error) => Outcome::Failed {
            error: error.to_string(),
        },
    }
}

/// Read the udev database record for one device, if the interface exposes it.
///
/// The record is keyed by `b<major>:<minor>`, which comes from sysfs's `dev`
/// attribute. If that attribute is missing the whole udev half is
/// `unavailable` for the device — not absent, because absence here would
/// claim the interface answered.
fn udev_fields(
    source: &dyn DeviceSource,
    udev_root: &Path,
    device_dir: &Path,
) -> Vec<(String, Outcome)> {
    let dev = match source.read_value(&device_dir.join("dev")) {
        Ok(value) if !value.is_empty() => value,
        _ => {
            return UDEV_KEYS
                .iter()
                .map(|key| {
                    (
                        (*key).to_owned(),
                        Outcome::Unavailable {
                            reason: "no device number, so the udev record cannot be located"
                                .to_owned(),
                        },
                    )
                })
                .collect();
        }
    };

    let record = udev_root.join(format!("b{dev}"));
    let text = match source.read_value(&record) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return UDEV_KEYS
                .iter()
                .map(|key| {
                    (
                        (*key).to_owned(),
                        Outcome::Unavailable {
                            reason: "no udev database record for this device".to_owned(),
                        },
                    )
                })
                .collect();
        }
        Err(error) => {
            return UDEV_KEYS
                .iter()
                .map(|key| {
                    (
                        (*key).to_owned(),
                        Outcome::Failed {
                            error: error.to_string(),
                        },
                    )
                })
                .collect();
        }
    };

    UDEV_KEYS
        .iter()
        .map(|key| {
            let found = text.lines().find_map(|line| {
                // udev database property lines are `E:KEY=value`. Anything
                // else in the file is another record class and is not read.
                let rest = line.strip_prefix("E:")?;
                let (name, value) = rest.split_once('=')?;
                (name == *key).then(|| value.to_owned())
            });
            let outcome = match found {
                Some(value) => Outcome::Observed(ObservedValue::Decimal(value)),
                None => Outcome::Observed(ObservedValue::Absent {
                    reason: "the key is not present in this device's udev record".to_owned(),
                }),
            };
            ((*key).to_owned(), outcome)
        })
        .collect()
}

/// The compiled sysfs root the production adapter reads.
#[must_use]
pub fn sysfs_root() -> PathBuf {
    PathBuf::from("/sys")
}

/// The compiled udev-database root the production adapter reads.
#[must_use]
pub fn udev_root() -> PathBuf {
    PathBuf::from("/run/udev/data")
}

/// Render one device's raw fields as observations, so the enumeration answer
/// shares the inspector's existing shape rather than inventing a second one.
#[must_use]
pub fn observations(device: &Device) -> Vec<Observation> {
    device
        .fields
        .iter()
        .map(|field| Observation {
            subject: format!("{}:{}", field.interface, field.property),
            attribution: attribution(field.interface, field.method),
            outcome: clone_outcome(&field.outcome),
        })
        .collect()
}

/// Outcome is not `Clone` — it belongs to `inspect` and this module does not
/// widen another module's public shape to save a few lines here.
fn clone_outcome(outcome: &Outcome) -> Outcome {
    match outcome {
        Outcome::Observed(ObservedValue::Bytes(value)) => {
            Outcome::Observed(ObservedValue::Bytes(value.clone()))
        }
        Outcome::Observed(ObservedValue::Decimal(value)) => {
            Outcome::Observed(ObservedValue::Decimal(value.clone()))
        }
        Outcome::Observed(ObservedValue::Absent { reason }) => {
            Outcome::Observed(ObservedValue::Absent {
                reason: reason.clone(),
            })
        }
        Outcome::Unavailable { reason } => Outcome::Unavailable {
            reason: reason.clone(),
        },
        Outcome::Failed { error } => Outcome::Failed {
            error: error.clone(),
        },
    }
}
