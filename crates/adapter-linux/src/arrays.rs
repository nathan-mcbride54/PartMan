//! Increment 4b: mdraid arrays as aggregates — named from **`md/uuid`**
//! since the second slice (ADR-0053), designator-absent where that source
//! is absent or unreadable — with their self-reported member count and
//! their kernel membership listing reported beside them.
//!
//! **The first slice named nothing** (2026-08-18, before ADR-0053): the
//! Linux host-assembled designation round had found no measured source it
//! could designate on the DR1–DR10 rows, so every array was an
//! `Aggregate { Mdraid, designator: None }` — indeterminate and not an
//! operand (ADR-0019; WP-010 slice 3q, gitea#1006), two of them absorbing
//! into a collision group. **The second slice names them.** DR11 measured
//! sysfs `md/uuid` present under each array's `md/`, client-readable,
//! byte-equal across re-assembly and a reboot, distinct per array, and
//! ADR-0053 designated it: the (Linux, Aggregate, mdraid) designator is the
//! `md/uuid` attribute, **bytes verbatim, trailing newline included**, read
//! through the bytes-preserving naming path and never through the
//! text-decoding one (ADR-0034's bytes-path requirement). The udev cache's
//! `MD_UUID` — the same bits spelled differently — is not read for naming.
//! An array whose `md/uuid` is positively absent or unreadable keeps the
//! designator-absent name and standing the first slice gave every array.
//!
//! **What rests on which row.** DR3: `md/` marks an mdraid array. DR5:
//! `md/raid_disks` reports the array's own member count through sysfs
//! (direct), agreeing with the database's `MD_DEVICES` — the self-reported
//! count ADR-C5 requires and never a count of members observed. DR4:
//! `slaves/` names the array's members as the kernel reports them, and is a
//! **per-mapping** relation, so it is reported as an observation and never
//! turned into an edge here — an edge needs member `BackingSignature`
//! nodes, whose family and offset the client has no measured source for
//! (`ID_FS_VERSION` and `md/metadata_version` were not read; the offset is
//! the helper's parser's fact), which is the next slice's question and
//! filed as such.
//!
//! **What this slice does not build.** No LVM2 aggregate (no VG identity is
//! client-readable, and one aggregate per LV would misrepresent one VG as
//! many); no `Volume` (a name needs a designated source); no
//! `EncryptionLayer` or `BackingSignature` (see above); no `BackingExtent`
//! for a loop (3b's host file-system node does not exist). Each stays a
//! withdrawn, reported device (increment 4a) until its cell or its round.

use std::path::Path;

use partman_domain::model::naming::{
    AggregateTechnology, NamingError, NamingFields, NodeEntry, absorb,
};

use crate::contract::{
    AttributeRead, ContractSource, InterfaceAnswered, Listing, NamingRead, list_bounded,
    read_attribute, read_naming_source,
};
use crate::devices::{BLOCK_CLASS, Device, DeviceKind, HostAssembledKind};

/// The array's self-reported member count, relative to its class directory
/// (DR5).
pub const RAID_DISKS_ATTRIBUTE: &str = "md/raid_disks";
/// ADR-0053's designated mdraid designator source, relative to the class
/// directory (DR11): the array's own UUID as sysfs reports it.
pub const ARRAY_UUID_ATTRIBUTE: &str = "md/uuid";
/// The kernel's membership listing, relative to the class directory (DR4).
pub const SLAVES_DIRECTORY: &str = "slaves";

/// The array's self-reported member count, as reported or not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberCount {
    /// The attribute answered with a decimal count.
    Reported(u64),
    /// The attribute is positively absent or empty — a measured absence.
    Absent,
    /// The attribute answered something that is not a count, or the read
    /// failed. Refused rather than guessed: a count is what ADR-C5's Fusion
    /// arm reads, and a wrong one would be a wrong verdict.
    Refused {
        /// What the attribute was.
        reason: String,
    },
}

/// The kernel's membership listing for one array — reported, never edged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Members {
    /// The `slaves/` entries, sorted, as listed.
    Listed(Vec<String>),
    /// The listing did not answer.
    Unavailable {
        /// Why.
        reason: String,
    },
}

/// How the designated source answered (ADR-0034's outcome rules, applied
/// to ADR-0053's mdraid cell).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignatorRead {
    /// `md/uuid` answered; the bytes are the designator, verbatim.
    Present,
    /// `md/uuid` is positively absent or empty — a measured absence. The
    /// designator is absent, the name weaker, and (for an aggregate) the
    /// node indeterminate and not an operand.
    Absent,
    /// The read failed or exceeded the bound — not absence. The same
    /// designator-absent name, the same standing, the reason recorded.
    Unreadable {
        /// Why the read did not produce an answer.
        reason: String,
    },
}

/// One mdraid array's report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayReport {
    /// The device's session-local selector.
    pub selector: String,
    /// The naming fields: `Aggregate { Mdraid, designator }`, the
    /// designator the `md/uuid` bytes verbatim where the source answered
    /// and absent otherwise (ADR-0053).
    pub fields: NamingFields,
    /// How the designated source answered.
    pub designator: DesignatorRead,
    /// The self-reported member count (DR5).
    pub member_count: MemberCount,
    /// The kernel's membership listing (DR4).
    pub members: Members,
}

/// Report every admitted mdraid array among the enumerated devices.
///
/// Only devices whose kind markers said `md/` are reported; a plain disk, a
/// dm node, a loop, or an undetermined device is not an array whatever its
/// entry name says.
#[must_use]
pub fn report_arrays(
    source: &dyn ContractSource,
    sysfs_root: &Path,
    devices: &[Device],
) -> Vec<ArrayReport> {
    let class = sysfs_root.join(BLOCK_CLASS);
    let Listing::Listed { answered, .. } = list_bounded(source, &class) else {
        // The class did not answer now; the devices were enumerated under a
        // listing that did, but no absence can be established without one.
        return Vec::new();
    };
    devices
        .iter()
        .filter(|device| device.kind == DeviceKind::HostAssembled(HostAssembledKind::Mdraid))
        .map(|device| report_array(source, &class.join(&device.entry), &answered, device))
        .collect()
}

fn report_array(
    source: &dyn ContractSource,
    directory: &Path,
    answered: &InterfaceAnswered,
    device: &Device,
) -> ArrayReport {
    let member_count = match read_attribute(source, &directory.join(RAID_DISKS_ATTRIBUTE), answered)
    {
        AttributeRead::Text(text) => match text.parse::<u64>() {
            Ok(count) => MemberCount::Reported(count),
            Err(_) => MemberCount::Refused {
                reason: format!("`{RAID_DISKS_ATTRIBUTE}` is not a decimal count: {text:?}"),
            },
        },
        AttributeRead::Empty | AttributeRead::NotPresent => MemberCount::Absent,
        AttributeRead::OverLimit { seen } => MemberCount::Refused {
            reason: format!("`{RAID_DISKS_ATTRIBUTE}` is {seen} bytes, over the limit"),
        },
        AttributeRead::NotText => MemberCount::Refused {
            reason: format!("`{RAID_DISKS_ATTRIBUTE}` is not UTF-8"),
        },
        AttributeRead::Failed { error } => MemberCount::Refused {
            reason: format!("`{RAID_DISKS_ATTRIBUTE}` could not be read: {error}"),
        },
    };
    let members = match list_bounded(source, &directory.join(SLAVES_DIRECTORY)) {
        Listing::Listed { entries, .. } => Members::Listed(entries),
        Listing::OverLimit { seen } => Members::Unavailable {
            reason: format!("`{SLAVES_DIRECTORY}` lists {seen} entries, over the limit"),
        },
        Listing::Unavailable { reason } => Members::Unavailable { reason },
        Listing::Failed { error } => Members::Unavailable { reason: error },
    };
    // The designator, through the bytes-preserving path (ADR-0034,
    // ADR-0053): the trailing newline is part of the name.
    let (designator, bytes) =
        match read_naming_source(source, &directory.join(ARRAY_UUID_ATTRIBUTE), answered) {
            NamingRead::Bytes(bytes) => (DesignatorRead::Present, Some(bytes)),
            NamingRead::Empty | NamingRead::NotPresent => (DesignatorRead::Absent, None),
            NamingRead::OverLimit { seen } => (
                DesignatorRead::Unreadable {
                    reason: format!("`{ARRAY_UUID_ATTRIBUTE}` is {seen} bytes, over the limit"),
                },
                None,
            ),
            NamingRead::Failed { error } => (
                DesignatorRead::Unreadable {
                    reason: format!("`{ARRAY_UUID_ATTRIBUTE}` could not be read: {error}"),
                },
                None,
            ),
        };
    ArrayReport {
        selector: device.selector.clone(),
        fields: NamingFields::Aggregate {
            technology: AggregateTechnology::Mdraid,
            designator: bytes,
        },
        designator,
        member_count,
        members,
    }
}

/// Absorb the reported arrays into ADR-0019's node set.
///
/// Named arrays absorb as distinct nodes; arrays whose `md/uuid` did not
/// answer carry the designator-absent name and, two or more, absorb into
/// one collision group — counted, flagged, indeterminate — or alone as an
/// indeterminate non-operand (slice 3q). Two arrays reporting equal
/// `md/uuid` bytes (a cloned array) absorb into a group flagged
/// `duplicate_designator`, ADR-0019's cloned-pool case.
///
/// # Errors
///
/// [`NamingError`] as the domain's absorption reports it.
pub fn absorb_arrays(reports: &[ArrayReport]) -> Result<Vec<NodeEntry>, NamingError> {
    absorb(reports.iter().map(|report| report.fields.clone()).collect())
}
