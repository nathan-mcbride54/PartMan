//! Increment 4b, third slice: the **held** standing of a whole device, read
//! from the kernel's `holders/` relation and reported as a state-layer
//! observation — the Linux input for the closure's consumed-member arm — and
//! nothing else.
//!
//! **What this is, and is not.** The Linux member-signature offset round
//! (`docs/reviews/LINUX_MEMBER_SIGNATURE_OFFSET_ROUND_2026-08-18.md`) found
//! that no client interface reports a signature's primary offset (DR14) and
//! that the family is client-readable per member only from the udev cache,
//! which the L4/L10 rows measured reporting exactly the stale signature on a
//! stale pair; a `BackingSignature`'s two naming fields are the helper's
//! byte layer's (ADR-0018; ADR-0019 `:252-256`). So this adapter builds **no
//! `BackingSignature`, no `Backing` edge and no `EncryptionLayer`** — they
//! enter the Linux inventory at HLP-002's re-discovery — and reports what the
//! client *does* read: whether a whole device is currently **held** by an
//! assembled node, from sysfs `holders/`, the source ADR-0018 names as the
//! Linux state-layer membership source ("the device-mapper/holders
//! topology", `:103-104`).
//!
//! **Every claim rests on DR15** (`docs/quality/observability.md`, the
//! held-standing cell, VMID 9473): `holders/` is a *live* fact from both
//! ends — positively empty on every member the moment its consumer is
//! stopped, naming the consumer again after re-assembly and after a
//! reboot — and it agrees with the assembled node's `slaves/` **by
//! identity** in every phase while the entry names moved under it (the two
//! LVs swapped `dm-0`/`dm-1` at re-assembly, the two arrays `md126`/`md127`
//! at the reboot). That is why a holder is reported by its own `md/uuid` or
//! `dm/uuid` and never by its entry name; the entry is a locator for the
//! read, and nothing else. DR15 also measured the unheld cases this module
//! reports as unheld: the PV of an active VG that no LV maps, a Btrfs
//! member, a plain disk, and — after a reboot with no opener — a live LUKS
//! disk, whose holder is exactly assembly and not what the disk carries.
//!
//! **What the standing does today: it is reported.** Assembly changes under
//! re-probe of unchanged hardware, so under MODEL-005's body-stability rule
//! a hold is envelope, never body — the shape increment 4a gave mounts —
//! and it enters no name and no address (a held member stays a
//! `PhysicalDevice` under its designated name: it is the host of what the
//! helper will find). The closure's consumed-member refusal that would
//! consume it is ADR-0018's forward obligation (`:391-398`, `:601-603`,
//! `:610`), filed as gitea#1008 on WP-010; until it lands the standing
//! changes no verdict, and this module says so rather than pretending
//! otherwise. The cached signature view (`ID_FS_TYPE`, `ID_FS_USAGE`,
//! `ID_FS_VERSION`) is read beside it (`devices::UDEV_SIGNATURE_KEYS`) as
//! `Heuristic`/`inferred` observations and is **consulted by nothing here**:
//! an unheld device stays unheld whatever the cache says, because demoting it
//! on a cached `linux_raid_member` would be, at the draft, the option
//! ADR-0018 rejected by name (a bench-tested disk unusable forever, `:523-529`).
//!
//! **Refusal, not guessing.** A `holders/` listing that did not answer
//! leaves the device's standing undetermined — reported as such, never as
//! unheld — the `partition` discipline again.

use std::path::Path;

use partman_domain::canonical::Value;
use partman_domain::model::provenance::{Observation, Outcome};

use crate::arrays::ARRAY_UUID_ATTRIBUTE;
use crate::contract::{
    AttributeRead, ContractSource, InterfaceAnswered, Listing, list_bounded, read_attribute,
};
use crate::devices::{BLOCK_CLASS, Device, DeviceKind};
use crate::observation::{Interface, observe_unavailable};
use crate::volumes::DM_UUID_ATTRIBUTE;

/// The kernel's holder listing, relative to a device's class directory
/// (DR4, DR15).
pub const HOLDERS_DIRECTORY: &str = "holders";

/// A holder's own identity — the key the standing is reported by (DR15:
/// entry names moved under the relation; identities held).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HolderIdentity {
    /// The holder is an mdraid array; its `md/uuid` as read.
    Mdraid(String),
    /// The holder is a device-mapper node; its `dm/uuid` as read.
    DeviceMapper(String),
    /// Neither identity attribute answered with a value. The device is
    /// still held — the listing named the holder — but the holder cannot be
    /// keyed, and it is **not** keyed by its entry name instead.
    Unidentified {
        /// Why.
        reason: String,
    },
}

/// One holder of a whole device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holder {
    /// The holder's block-class entry — a session-local locator, never a key.
    pub entry: String,
    /// The holder's identity, which is the key.
    pub identity: HolderIdentity,
}

/// A whole device's held standing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Standing {
    /// `holders/` positively names one or more assembled nodes.
    Held {
        /// The holders, in listing order.
        holders: Vec<Holder>,
    },
    /// `holders/` answered and is empty — positively unheld.
    Unheld,
    /// `holders/` did not answer. Not unheld: the fail-closed direction.
    Undetermined {
        /// Why.
        reason: String,
    },
}

/// One admitted whole device's held report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeldReport {
    /// The device's session-local selector.
    pub selector: String,
    /// The standing.
    pub standing: Standing,
    /// The MODEL-004 observations on the sysfs interface: one per holder
    /// (its identity as the value, or a failure where the identity did not
    /// answer), one positively-absent observation for an unheld device, one
    /// unavailable or failed observation for an undetermined one.
    pub observations: Vec<Observation>,
}

/// Report the held standing of every admitted **plain** whole device.
///
/// Host-assembled nodes are not physical devices (increment 4a) and are not
/// reported here; an undetermined-kind device is not either. Nothing read
/// here enters a name.
#[must_use]
pub fn report_held(
    source: &dyn ContractSource,
    sysfs_root: &Path,
    devices: &[Device],
) -> Vec<HeldReport> {
    let class = sysfs_root.join(BLOCK_CLASS);
    let Listing::Listed { answered, .. } = list_bounded(source, &class) else {
        return Vec::new();
    };
    devices
        .iter()
        .filter(|device| device.kind == DeviceKind::Plain)
        .map(|device| report_one(source, &class, &answered, device))
        .collect()
}

fn report_one(
    source: &dyn ContractSource,
    class: &Path,
    answered: &InterfaceAnswered,
    device: &Device,
) -> HeldReport {
    let directory = class.join(&device.entry);
    let (standing, observations) = match list_bounded(source, &directory.join(HOLDERS_DIRECTORY)) {
        Listing::Listed { entries, .. } if entries.is_empty() => {
            (Standing::Unheld, vec![observation(Outcome::ObservedAbsent)])
        }
        Listing::Listed { entries, .. } => {
            let holders: Vec<Holder> = entries
                .into_iter()
                .map(|entry| {
                    let identity = identify(source, &class.join(&entry), answered);
                    Holder { entry, identity }
                })
                .collect();
            let observations = holders
                .iter()
                .map(|holder| match &holder.identity {
                    HolderIdentity::Mdraid(uuid) => observation(Outcome::Observed {
                        value: Value::Text(format!("held-by mdraid {ARRAY_UUID_ATTRIBUTE}={uuid}")),
                    }),
                    HolderIdentity::DeviceMapper(uuid) => observation(Outcome::Observed {
                        value: Value::Text(format!(
                            "held-by device-mapper {DM_UUID_ATTRIBUTE}={uuid}"
                        )),
                    }),
                    HolderIdentity::Unidentified { reason } => observation(Outcome::Failed {
                        error: format!("held by a node whose identity did not answer: {reason}"),
                    }),
                })
                .collect();
            (Standing::Held { holders }, observations)
        }
        Listing::OverLimit { seen } => {
            let reason = format!("`{HOLDERS_DIRECTORY}` lists {seen} entries, over the limit");
            (
                Standing::Undetermined {
                    reason: reason.clone(),
                },
                vec![observation(Outcome::Failed { error: reason })],
            )
        }
        Listing::Unavailable { reason } => (
            Standing::Undetermined {
                reason: reason.clone(),
            },
            vec![observe_unavailable(Interface::Sysfs, &reason)],
        ),
        Listing::Failed { error } => (
            Standing::Undetermined {
                reason: error.clone(),
            },
            vec![observation(Outcome::Failed { error })],
        ),
    };
    HeldReport {
        selector: device.selector.clone(),
        standing,
        observations,
    }
}

/// A holder's identity: `md/uuid` for an array, `dm/uuid` for a dm node.
///
/// Read through the text path: the identity keys a state observation and
/// enters no name, so ADR-0034's bytes-path clause does not reach it (the
/// naming of an array from `md/uuid` is `arrays.rs`'s, through the
/// bytes-preserving path).
fn identify(
    source: &dyn ContractSource,
    holder: &Path,
    answered: &InterfaceAnswered,
) -> HolderIdentity {
    match read_attribute(source, &holder.join(ARRAY_UUID_ATTRIBUTE), answered) {
        AttributeRead::Text(uuid) => return HolderIdentity::Mdraid(uuid),
        AttributeRead::NotPresent => {}
        other => {
            return HolderIdentity::Unidentified {
                reason: describe(ARRAY_UUID_ATTRIBUTE, &other),
            };
        }
    }
    match read_attribute(source, &holder.join(DM_UUID_ATTRIBUTE), answered) {
        AttributeRead::Text(uuid) => HolderIdentity::DeviceMapper(uuid),
        AttributeRead::NotPresent => HolderIdentity::Unidentified {
            reason: format!(
                "neither `{ARRAY_UUID_ATTRIBUTE}` nor `{DM_UUID_ATTRIBUTE}` is present"
            ),
        },
        other => HolderIdentity::Unidentified {
            reason: describe(DM_UUID_ATTRIBUTE, &other),
        },
    }
}

fn describe(attribute: &str, read: &AttributeRead) -> String {
    match read {
        AttributeRead::Text(_) | AttributeRead::NotPresent => {
            unreachable!("handled by the caller")
        }
        AttributeRead::Empty => format!("`{attribute}` is empty"),
        AttributeRead::OverLimit { seen } => {
            format!("`{attribute}` is {seen} bytes, over the limit")
        }
        AttributeRead::NotText => format!("`{attribute}` is not UTF-8"),
        AttributeRead::Failed { error } => format!("`{attribute}` could not be read: {error}"),
    }
}

fn observation(outcome: Outcome) -> Observation {
    Observation {
        adapter: Interface::Sysfs.adapter(),
        adapter_version: crate::VERSION.to_owned(),
        method: Interface::Sysfs.method(),
        outcome,
    }
}
