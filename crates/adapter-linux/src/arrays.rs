//! Increment 4b, first slice: mdraid arrays as **designator-absent**
//! aggregates, with their self-reported member count and their kernel
//! membership listing reported beside them.
//!
//! **What this slice builds, and why only this.** The Linux host-assembled
//! designation round (`docs/reviews/LINUX_HOST_ASSEMBLED_DESIGNATION_ROUND_2026-08-18.md`)
//! found that no measured Linux source may name a host-assembled kind under
//! ADR-0034's discipline: mdraid's only measured designator source is the
//! udev cache that ADR refuses while sysfs `md/uuid` is unmeasured; the LVM2
//! VG id is `not-client-readable` (L7); a volume's `dm/name` lacks its
//! stability cell. So this slice names **nothing** — every array is an
//! `Aggregate { technology: Mdraid, designator: None }`, which ADR-0019
//! decides is `Indeterminate` and not a plan operand, and which WP-010 slice
//! 3q made the closure enforce (gitea#1006). Two arrays derive the same
//! designator-absent address and absorb into a collision group, which is
//! the decided representation of "two aggregates this contract cannot tell
//! apart", not a defect: when a designation lands, each gains its field and
//! the group dissolves; nothing built here is withdrawn.
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
    AttributeRead, ContractSource, InterfaceAnswered, Listing, list_bounded, read_attribute,
};
use crate::devices::{BLOCK_CLASS, Device, DeviceKind, HostAssembledKind};

/// The array's self-reported member count, relative to its class directory
/// (DR5).
pub const RAID_DISKS_ATTRIBUTE: &str = "md/raid_disks";
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

/// One mdraid array's report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayReport {
    /// The device's session-local selector.
    pub selector: String,
    /// The naming fields — always `Aggregate { Mdraid, designator: None }`
    /// in this slice; the field is here so a designation, when one lands,
    /// changes one constructor and no consumer.
    pub fields: NamingFields,
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
    ArrayReport {
        selector: device.selector.clone(),
        fields: NamingFields::Aggregate {
            technology: AggregateTechnology::Mdraid,
            designator: None,
        },
        member_count,
        members,
    }
}

/// Absorb the reported arrays into ADR-0019's node set.
///
/// Every array in this slice carries the same designator-absent name, so
/// two or more absorb into one collision group — counted, flagged,
/// indeterminate — and one absorbs alone as an indeterminate non-operand
/// (slice 3q). Both are the decided fail-closed representation.
///
/// # Errors
///
/// [`NamingError`] as the domain's absorption reports it.
pub fn absorb_arrays(reports: &[ArrayReport]) -> Result<Vec<NodeEntry>, NamingError> {
    absorb(reports.iter().map(|report| report.fields.clone()).collect())
}
