//! ADR-0019 node addressing for the Linux contract, and the ADR-0034
//! designation it resolves.
//!
//! A node's address is derived from its naming fields and nothing else, so
//! everything hard about this module is upstream of `derive_id`: obtaining
//! the designated bytes, and refusing to invent them when the designation
//! does not reach this device.
//!
//! **Every naming input flows through [`read_naming_source`] and none
//! through `read_attribute`.** That is ADR-0034's verification clause, and it
//! holds here for the size input too. A sector count is a number rather than
//! an identifier, so the bytes-path rationale — verbatim identifier bytes,
//! non-UTF-8 legal, no newline stripping — does not obviously bite on it. It
//! is read through the naming path anyway: the clause is written without a
//! carve-out, satisfying it literally costs nothing, and doing the decimal
//! parse here rather than borrowing the text path's makes each of its
//! refusals this module's own and therefore testable.
//!
//! **What this module does not do.** It builds no partition-table node and
//! no partition node. `NamingFields::PartitionTable` requires a `TableRole`
//! — a scheme — and this contract reads no table bytes and no table-type
//! key; `NamingFields::Partition` requires a table node to hang from. The
//! choice ADR-0036 puts to this increment is recorded in the package
//! document, not decided in code.

use std::path::{Path, PathBuf};

use partman_domain::model::naming::{NamingError, NamingFields, NodeEntry, absorb};

use crate::contract::{ContractSource, InterfaceAnswered, NamingRead, read_naming_source};

/// The sysfs `size` unit, in bytes.
///
/// Measured rather than assumed, and measured twice at two scopes. The
/// 2026-08-13 readback (**R3**) established the 512-byte unit on a partition
/// node against a declared byte extent and explicitly declined to bridge the
/// gap to the whole-device node by convention; **FR5** then measured the
/// whole-device node itself — sysfs `size` `244457472` × 512 equalling
/// `blockdev --getsize64`'s `125162225664` exactly. `NamingFields::
/// PhysicalDevice` carries a required `total_bytes`, so this constant is a
/// prerequisite for addressing a device at all, and it now cites a row
/// instead of a convention.
pub const SECTOR_BYTES: u64 = 512;

/// How far up the ancestor chain the USB search walks before refusing.
///
/// The rule ADR-0034 designates is structural — "the nearest ancestor sysfs
/// node that is a USB device node" — not a fixed traversal, so the search
/// cannot be a hardcoded depth. It still needs a bound, for the reason every
/// other bound in this crate has one: an unbounded walk over a surface that
/// is behaving unexpectedly is a hang rather than an answer. The measured
/// instrument reached its ancestor in four steps; this leaves room for
/// deeper topologies without leaving the walk open.
pub const ANCESTOR_LIMIT: usize = 16;

/// The attributes whose joint presence marks a sysfs node as a USB **device**
/// node, as opposed to a USB interface node or any other ancestor.
///
/// **This predicate has no qualifying observability row, and the gap is
/// recorded rather than papered over.** ADR-0034's evidence obligation 1 —
/// capturing the resolved canonical path — is discharged by **FR4**, which
/// establishes that the measured traversal *reaches* a USB device node. No
/// row establishes what a client may read to *recognize* one, which is a
/// different claim and the one this predicate makes. Contrast ADR-0035's mmc
/// cell, whose structural rule **S5c** measured directly.
///
/// So the rule is written the way increment 2 wrote [`PARTITION_ATTRIBUTE`]'s
/// under the same shortfall: fail-closed, so the unmeasured direction is the
/// safe one. Recognition requires **both** markers to answer with a value;
/// an unreadable marker recognizes nothing, and an unrecognized ancestor
/// yields an absent serial and a weaker name rather than a guessed one. The
/// row is filed as an obligation on WP-035, which owns
/// `docs/quality/observability.md`.
///
/// [`PARTITION_ATTRIBUTE`]: crate::devices::PARTITION_ATTRIBUTE
pub const USB_DEVICE_MARKERS: [&str; 2] = ["idVendor", "idProduct"];

/// The designated serial source's own attribute name on a USB device node.
pub const USB_SERIAL_ATTRIBUTE: &str = "serial";

/// The link from a block device's class directory to its bus device node.
pub const DEVICE_LINK: &str = "device";

/// One designated source's outcome, in ADR-0034's own vocabulary.
///
/// The four arms are four different things, and collapsing any two of them
/// changes what a node is allowed to be. ADR-0034 closed two of ADR-0019's
/// gaps precisely because the delivered contract produced outcomes ADR-0019
/// had no rule for.
#[derive(Debug, PartialEq, Eq)]
pub enum DesignatedSource {
    /// The designated source answered, with its bytes verbatim.
    Present(Vec<u8>),
    /// A measured absence: the source was positively observed not to exist.
    /// The field is absent, the name is weaker, and the node **remains an
    /// operand** — "a stable truth about the hardware is a lawful weak name".
    Absent,
    /// A failed read: the source exists or may exist, and reading it failed.
    /// **Not absence.** The node derives its name from its remaining fields,
    /// is marked indeterminate, and is **not a plan operand**.
    Unreadable {
        /// Why the read did not produce an answer.
        reason: String,
    },
    /// The (platform, attachment class, identifier) cell is undesignated.
    /// The field is absent and **no read was attempted** against a source
    /// the designation does not name.
    Undesignated,
}

impl DesignatedSource {
    /// The bytes this outcome contributes to a naming field, if any.
    ///
    /// Three of the four arms contribute nothing, and they are still four
    /// arms: what they differ in is the node's standing, not its bytes.
    #[must_use]
    pub fn bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Present(bytes) => Some(bytes.clone()),
            Self::Absent | Self::Unreadable { .. } | Self::Undesignated => None,
        }
    }

    /// Whether this outcome leaves the node a plan operand (ADR-0034).
    #[must_use]
    pub const fn operand_eligible(&self) -> bool {
        !matches!(self, Self::Unreadable { .. })
    }
}

/// Resolve ADR-0034's designated serial source for one block device.
///
/// Walks the ancestor chain above the device's bus node, nearest first, and
/// reads [`USB_SERIAL_ATTRIBUTE`] from the first ancestor that answers as a
/// USB device node. The walk is expressed as parent components appended to
/// the bus-node path rather than as a resolved absolute path, because the
/// seam this contract closed at increment 1 offers a bounded listing and a
/// bounded read and no link resolution; each step is a path the platform
/// resolves for itself.
///
/// Every terminating outcome is fail-closed in the direction ADR-0034 gives
/// it. No USB ancestor within [`ANCESTOR_LIMIT`] is
/// [`DesignatedSource::Undesignated`] — the catch-all row of the designation
/// table, "every other attachment class" — and not a failure, because a
/// device that is not USB-attached has no designated serial source to fail
/// at.
#[must_use]
pub fn designated_serial(
    source: &dyn ContractSource,
    device_directory: &Path,
    answered: &InterfaceAnswered,
) -> DesignatedSource {
    let mut ancestor = device_directory.join(DEVICE_LINK);
    for _ in 0..ANCESTOR_LIMIT {
        ancestor = ancestor.join("..");
        match usb_device_node(source, &ancestor, answered) {
            Recognition::Yes => {
                return read_designated(source, &ancestor.join(USB_SERIAL_ATTRIBUTE), answered);
            }
            Recognition::No => {}
        }
    }
    DesignatedSource::Undesignated
}

/// Whether one ancestor answers as a USB device node.
///
/// Two-valued on purpose. An unreadable marker is not a third answer here:
/// it recognizes nothing, which continues the walk and ultimately yields
/// `Undesignated`. Promoting it to a failure would turn every host whose
/// ancestor chain contains one unreadable node into a host of
/// non-operands, which is fail-closed in the wrong direction — the
/// designation has nothing to say about a node it never identified.
enum Recognition {
    Yes,
    No,
}

fn usb_device_node(
    source: &dyn ContractSource,
    ancestor: &Path,
    answered: &InterfaceAnswered,
) -> Recognition {
    for marker in USB_DEVICE_MARKERS {
        if !matches!(
            read_naming_source(source, &ancestor.join(marker), answered),
            NamingRead::Bytes(_)
        ) {
            return Recognition::No;
        }
    }
    Recognition::Yes
}

/// Read one designated source and map it onto ADR-0034's outcome rules.
fn read_designated(
    source: &dyn ContractSource,
    path: &Path,
    answered: &InterfaceAnswered,
) -> DesignatedSource {
    match read_naming_source(source, path, answered) {
        NamingRead::Bytes(bytes) => DesignatedSource::Present(bytes),
        // Both absence arms are measured absences, and ADR-0034 gives them
        // one consequence: an operand with a weaker name.
        NamingRead::Empty | NamingRead::NotPresent => DesignatedSource::Absent,
        NamingRead::OverLimit { seen } => DesignatedSource::Unreadable {
            reason: format!("the designated source is {seen} bytes, over the limit"),
        },
        NamingRead::Failed { error } => DesignatedSource::Unreadable {
            reason: format!("the designated source could not be read: {error}"),
        },
    }
}

/// Why a device could not be addressed at all.
///
/// `NamingFields::PhysicalDevice` carries a **required** `total_bytes`, so a
/// device whose sector count does not answer has no naming field set and
/// therefore no address. It is reported rather than dropped: a device the
/// platform enumerated and this adapter silently omitted would be a
/// fail-open, and SAFE-005 puts absence on the refusing side.
#[derive(Debug, PartialEq, Eq)]
pub struct Unaddressable {
    /// The device's session-local selector, so the refusal names its subject.
    pub selector: String,
    /// Why no address could be derived.
    pub reason: String,
}

/// One device's naming outcome.
#[derive(Debug, PartialEq, Eq)]
pub enum DeviceNaming {
    /// The device can be addressed, with the fields its address derives from
    /// and the standing ADR-0034's serial outcome leaves it in.
    Addressed {
        /// ADR-0019's per-kind naming map for this device.
        fields: NamingFields,
        /// Whether the device is a plan operand, per ADR-0034's failed-read
        /// rule. False marks it indeterminate.
        operand_eligible: bool,
    },
    /// The device cannot be addressed.
    Refused(Unaddressable),
}

/// Parse a sysfs sector count into a byte total.
///
/// Refuses rather than guessing on every arm: a value that is not ASCII
/// decimal is not a sector count, and a count whose byte product overflows a
/// `u64` is not a device this build can address. Exactly one trailing
/// newline is tolerated because the platform's attribute files carry one;
/// that tolerance is a parse rule for a decimal number, not the text path's
/// strip applied to identifier bytes.
///
/// # Errors
///
/// A description of what the value was, for the refusal that carries it.
pub fn sector_count_to_bytes(raw: &[u8]) -> Result<u64, String> {
    let digits = raw.strip_suffix(b"\n").unwrap_or(raw);
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Err("the sector count is not an ASCII decimal value".to_owned());
    }
    // `digits` is ASCII by the check above, so the decode cannot fail.
    let text = String::from_utf8_lossy(digits);
    let sectors: u64 = text
        .parse()
        .map_err(|_| format!("the sector count `{text}` does not fit a 64-bit value"))?;
    sectors
        .checked_mul(SECTOR_BYTES)
        .ok_or_else(|| format!("{sectors} sectors of {SECTOR_BYTES} bytes overflows a byte total"))
}

/// Derive one device's naming fields from the designated sources.
///
/// WWN is [`DesignatedSource::Undesignated`] unconditionally: ADR-0034
/// leaves the cell undesignated on Linux for every attachment class, and its
/// verification clause requires that **no read is attempted** against an
/// undesignated source. That is why this takes no WWN path and reads none.
#[must_use]
pub fn name_device(
    source: &dyn ContractSource,
    device_directory: &Path,
    answered: &InterfaceAnswered,
    selector: String,
) -> DeviceNaming {
    let size = match read_naming_source(source, &device_directory.join(SIZE_ATTRIBUTE), answered) {
        NamingRead::Bytes(bytes) => bytes,
        NamingRead::Empty | NamingRead::NotPresent => {
            return refuse(selector, "the sector count is absent");
        }
        NamingRead::OverLimit { seen } => {
            return refuse(
                selector,
                &format!("the sector count is {seen} bytes, over the limit"),
            );
        }
        NamingRead::Failed { error } => {
            return refuse(selector, &format!("the sector count read failed: {error}"));
        }
    };
    let total_bytes = match sector_count_to_bytes(&size) {
        Ok(total) => total,
        Err(reason) => return refuse(selector, &reason),
    };

    let serial = designated_serial(source, device_directory, answered);
    DeviceNaming::Addressed {
        operand_eligible: serial.operand_eligible(),
        fields: NamingFields::PhysicalDevice {
            serial: serial.bytes(),
            wwn: None,
            total_bytes,
        },
    }
}

/// The attribute carrying a device's sector count.
pub const SIZE_ATTRIBUTE: &str = "size";

fn refuse(selector: String, reason: &str) -> DeviceNaming {
    DeviceNaming::Refused(Unaddressable {
        selector,
        reason: reason.to_owned(),
    })
}

/// Absorb the addressed devices into ADR-0019's node set.
///
/// The collision grouping is the domain's, not this adapter's: equal-address
/// same-kind nodes collapse into the counted, flagged, indeterminate group
/// before encoding, and re-implementing that here would be a second rule
/// beside the normative one.
///
/// # Errors
///
/// [`NamingError`] as the domain's absorption reports it.
pub fn absorb_devices(named: &[DeviceNaming]) -> Result<Vec<NodeEntry>, NamingError> {
    absorb(
        named
            .iter()
            .filter_map(|naming| match naming {
                DeviceNaming::Addressed { fields, .. } => Some(fields.clone()),
                DeviceNaming::Refused(_) => None,
            })
            .collect(),
    )
}

/// The path a device's class directory sits at, for callers assembling one.
#[must_use]
pub fn device_directory(sysfs_root: &Path, name: &str) -> PathBuf {
    sysfs_root.join(crate::devices::BLOCK_CLASS).join(name)
}
