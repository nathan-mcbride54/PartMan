//! HLP-002's re-discovery (increment 2): the helper's own capture of the
//! topology, independent of anything a client asserts, producing the
//! authoritative snapshot whose body hash an authorized plan binds
//! (ADR-0014, Section 6).
//!
//! **What the capture is.** The adapter's client contract run as root —
//! the same `enumerate`, the same naming designations, the same bounded
//! reads, nothing re-implemented — plus the one thing no layer below the
//! helper may author: the table state, computed by `crates/table-parser`
//! over head and tail windows read through a read-only device handle
//! ([`crate::bytes`]). Where a scheme is claimed, the capture authors the
//! partition-table node — the node WP-L100's increment 3b waited on
//! (ADR-0036's second branch). The protection verdict is not a stored
//! value: it is computed by the domain's closure over exactly the facts
//! this capture authors (ADR-0016's recompute rule), so authoring the
//! facts *is* authoring the verdict's inputs, and nothing else is.
//!
//! **What the capture refuses to invent, stated per arm:**
//!
//! - **Transport stays `Unrecognized`** (the adapter's own answer):
//!   ADR-0018's fabric-versus-local discrimination rows are outstanding,
//!   and privilege changes nothing about that — so on a real host every
//!   device's own protection arm is `Indeterminate` and every mutating
//!   validate-plan refuses at the capability gate, which is the honest
//!   fail-closed answer until the rows exist.
//! - **A collision group gets no facts.** Two devices whose designated
//!   sources derive one address (serial-less equal-size disks) absorb
//!   into ADR-0019's counted, flagged, indeterminate group; a table
//!   state keyed by an address that names two media would be a fact
//!   about neither, so none is authored and the withholding is recorded.
//! - **A refused window is a withheld state, never a guessed one.** A
//!   device whose geometry cannot be stated, whose node cannot be
//!   bracketed by device number, or whose read falls short gets no
//!   `table_states` entry — honest absence, which fails closed at the
//!   closure — with the refusal recorded in the envelope.
//! - **Host-assembled devices are withdrawn** exactly as the adapter
//!   withdraws them (dm, md, loop); their kinds are later increments'.
//! - **The held report is consumed, reading (b)** (ADR-0053, WP-L100's
//!   third slice): a held device stays a captured physical device — it
//!   is the host of what the helper will find — its standing recorded in
//!   the envelope; **no aggregate node and no member edge is emitted**
//!   until the increment that owns them, and that withholding is the
//!   capture's fail-closed answer to the assembled-state discriminant,
//!   said here rather than discovered later.
//!
//! Everything here is pure over the adapter's `ContractSource` and this
//! crate's `DeviceReader`, so the Tier-1 suite drives it with authored
//! trees and catalogue bytes on every platform; the real roots and the
//! real reader exist on Linux only.

use std::collections::BTreeMap;

use partman_adapter_linux::contract::{ContractSource, Listing, list_bounded};
use partman_adapter_linux::devices::{BLOCK_CLASS, Device, Enumeration, enumerate};
use partman_adapter_linux::held::{Standing, report_held};
use partman_adapter_linux::naming::{DeviceNaming, device_directory, name_device};
use partman_domain::canonical::{Hash, Value};
use partman_domain::model::identity::{IndeterminateCause, TableState};
use partman_domain::model::naming::{NamingFields, NodeId, TableRole, derive_id};
use partman_domain::model::protection::{Facts, HostRange};
use partman_domain::model::provenance::{Method, Observation, Outcome, PropertyObservations};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::topology::{Edge, EdgeKind};
use partman_table_parser::{Condition, Geometry, Scheme, TableState as ParsedState, classify};

use crate::bytes::{DeviceReader, WINDOW_BYTES};

/// The adapter's whole-device sector unit (its `size` attribute), which
/// `name_device` already multiplied into `total_bytes`.
const SIZE_UNIT: u64 = 512;

/// One captured whole device, in listing order — the capture's own record
/// of what it did and did not author, for the audit trail and the Tier-2
/// instrument. Selectors are session-local; no field carries a serial, a
/// path, a label or a username.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceCapture {
    /// Named, windowed, classified: the table state is authored, and the
    /// claimed scheme's table node exists.
    Authored {
        /// The session-local selector.
        selector: String,
        /// The device's derived address.
        node: NodeId,
        /// The authored state's name: `present`, `absent`,
        /// `indeterminate-ambiguous`, `indeterminate-unreadable`.
        state: &'static str,
        /// The claimed scheme's name, where one was claimed.
        scheme: Option<&'static str>,
        /// Whether the hybrid view node was authored beside the scheme's.
        hybrid: bool,
    },
    /// Named, but no table state was authored; the withholding is stated.
    NamedOnly {
        /// The session-local selector.
        selector: String,
        /// The device's derived address.
        node: NodeId,
        /// Why the state was withheld, in this crate's words.
        withheld: String,
    },
    /// Named into a collision group: the node exists as ADR-0019's
    /// counted, flagged group, and no fact is authored under the shared
    /// address.
    Grouped {
        /// The session-local selector.
        selector: String,
        /// The shared derived address.
        node: NodeId,
    },
    /// Not named: refused by the adapter's naming rules, or withdrawn as
    /// host-assembled. No node.
    NotNamed {
        /// The session-local selector.
        selector: String,
        /// Why, in the adapter's or this crate's words.
        why: String,
    },
}

/// A capture that produced an authoritative snapshot.
#[derive(Debug)]
pub struct CaptureOutcome {
    /// The snapshot — `Captured`, non-transitional, its facts exactly
    /// what this module authored.
    pub snapshot: TopologySnapshot,
    /// The snapshot's body hash: what a validated plan binds (PLAN-006).
    pub snapshot_hash: Hash,
    /// The per-device record, in listing order.
    pub devices: Vec<DeviceCapture>,
}

/// Why no snapshot exists. Typed; fail-closed; no identifier in any arm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureRefusal {
    /// The block-class listing did not answer as a listing.
    Enumeration {
        /// `over-limit`, `unavailable` or `failed`, with the interface's
        /// own words.
        outcome: String,
    },
    /// The snapshot could not be assembled — a constructor refusal, which
    /// for facts this module authored is a defect worth the typed arm.
    Assembly {
        /// The constructor's words.
        detail: String,
    },
    /// The body could not be hashed (unreachable for these shapes,
    /// reported rather than panicked).
    Unhashable,
}

/// Run HLP-002's capture: the adapter's contract through `source` at the
/// given roots, the byte layer through `reader`, the envelope stamped
/// `now`.
///
/// # Errors
///
/// [`CaptureRefusal`].
pub fn capture(
    source: &dyn ContractSource,
    sysfs_root: &std::path::Path,
    udev_root: &std::path::Path,
    reader: &dyn DeviceReader,
    now: u64,
) -> Result<CaptureOutcome, CaptureRefusal> {
    let listed = admitted(source, sysfs_root, udev_root)?;
    let class = sysfs_root.join(BLOCK_CLASS);
    let Listing::Listed { answered, .. } = list_bounded(source, &class) else {
        return Err(CaptureRefusal::Enumeration {
            outcome: "the interface stopped answering between passes".to_owned(),
        });
    };

    // Pass 1: name every admitted device; count address collisions.
    let (named, mut reports, counts) = name_pass(source, sysfs_root, &listed, &answered);

    // Pass 2: facts and table nodes for every uniquely-addressed device.
    let mut nodes: Vec<NamingFields> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut facts = Facts::default();
    let mut provenance: Vec<(String, PropertyObservations)> = Vec::new();
    for (index, entry) in named.iter().enumerate() {
        let Some((id, fields)) = entry else { continue };
        let device = &listed[index];
        nodes.push(fields.clone());
        if counts.get(id).copied().unwrap_or(0) > 1 {
            reports[index] = DeviceCapture::Grouped {
                selector: device.selector.clone(),
                node: *id,
            };
            record(
                &mut provenance,
                &device.selector,
                "table-window",
                Outcome::Unavailable {
                    reason: "the address names more than one device; no fact is authored under \
                             a shared address"
                        .to_owned(),
                },
            );
            continue;
        }
        let NamingFields::PhysicalDevice { total_bytes, .. } = fields else {
            continue;
        };
        facts.transports.insert(*id, device.transport);
        facts.extents.insert(
            *id,
            HostRange {
                host: *id,
                start: 0,
                length: *total_bytes,
            },
        );
        reports[index] = match window_and_classify(device, *total_bytes, reader) {
            Ok(classification) => author_one(
                *id,
                device,
                &classification,
                &mut facts,
                &mut nodes,
                &mut edges,
            ),
            Err(withheld) => {
                record(
                    &mut provenance,
                    &device.selector,
                    "table-window",
                    Outcome::Failed {
                        error: withheld.clone(),
                    },
                );
                DeviceCapture::NamedOnly {
                    selector: device.selector.clone(),
                    node: *id,
                    withheld,
                }
            }
        };
    }

    consume_held_and_attribute(source, sysfs_root, &listed, &mut provenance);

    let mut snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges, facts).map_err(
            |error| CaptureRefusal::Assembly {
                detail: format!("{error:?}"),
            },
        )?;
    snapshot.envelope.capture_timestamp = Some(now);
    snapshot.envelope.provenance = provenance;
    let snapshot_hash = snapshot
        .body_hash()
        .map_err(|_| CaptureRefusal::Unhashable)?;
    Ok(CaptureOutcome {
        snapshot,
        snapshot_hash,
        devices: reports,
    })
}

/// The block-class listing, or the typed refusal for each arm that is
/// not a listing.
fn admitted(
    source: &dyn ContractSource,
    sysfs_root: &std::path::Path,
    udev_root: &std::path::Path,
) -> Result<Vec<Device>, CaptureRefusal> {
    match enumerate(source, sysfs_root, udev_root) {
        Enumeration::Listed { devices } => Ok(devices),
        Enumeration::OverLimit { seen } => Err(CaptureRefusal::Enumeration {
            outcome: format!("over-limit: {seen} entries"),
        }),
        Enumeration::Unavailable { reason } => Err(CaptureRefusal::Enumeration {
            outcome: format!("unavailable: {reason}"),
        }),
        Enumeration::Failed { error } => Err(CaptureRefusal::Enumeration {
            outcome: format!("failed: {error}"),
        }),
    }
}

/// The held report, consumed reading (b) — standing recorded, no node
/// and no edge emitted from it — and the adapter's per-device
/// observation sets, attributed as the adapter attributed them.
fn consume_held_and_attribute(
    source: &dyn ContractSource,
    sysfs_root: &std::path::Path,
    listed: &[Device],
    provenance: &mut Vec<(String, PropertyObservations)>,
) {
    for report in report_held(source, sysfs_root, listed) {
        let standing = match report.standing {
            Standing::Held { ref holders } => format!("held by {} holder(s)", holders.len()),
            Standing::Unheld => "unheld".to_owned(),
            Standing::Undetermined { ref reason } => format!("undetermined: {reason}"),
        };
        record(
            provenance,
            &report.selector,
            "holders",
            Outcome::Observed {
                value: Value::Text(standing),
            },
        );
    }
    for device in listed {
        for (key, observations) in &device.properties {
            provenance.push((format!("{}:{key}", device.selector), observations.clone()));
        }
    }
}

/// Pass 1: name every admitted device through the adapter's rules,
/// deriving each address and counting collisions.
#[allow(clippy::type_complexity)]
fn name_pass(
    source: &dyn ContractSource,
    sysfs_root: &std::path::Path,
    listed: &[Device],
    answered: &partman_adapter_linux::contract::InterfaceAnswered,
) -> (
    Vec<Option<(NodeId, NamingFields)>>,
    Vec<DeviceCapture>,
    BTreeMap<NodeId, usize>,
) {
    let mut named: Vec<Option<(NodeId, NamingFields)>> = Vec::with_capacity(listed.len());
    let mut reports: Vec<DeviceCapture> = Vec::with_capacity(listed.len());
    let mut counts: BTreeMap<NodeId, usize> = BTreeMap::new();
    for device in listed {
        let directory = device_directory(sysfs_root, &device.entry);
        match name_device(source, &directory, answered, device.selector.clone()) {
            DeviceNaming::Addressed { fields, .. } => match derive_id(&fields) {
                Ok(id) => {
                    *counts.entry(id).or_insert(0) += 1;
                    named.push(Some((id, fields)));
                    reports.push(DeviceCapture::NamedOnly {
                        selector: device.selector.clone(),
                        node: id,
                        withheld: String::new(), // finalized in pass 2
                    });
                }
                Err(error) => {
                    named.push(None);
                    reports.push(DeviceCapture::NotNamed {
                        selector: device.selector.clone(),
                        why: format!("the address could not be derived: {error:?}"),
                    });
                }
            },
            DeviceNaming::Refused(refusal) => {
                named.push(None);
                reports.push(DeviceCapture::NotNamed {
                    selector: device.selector.clone(),
                    why: format!("naming refused: {}", refusal.reason),
                });
            }
            DeviceNaming::Withdrawn { kind, .. } => {
                named.push(None);
                reports.push(DeviceCapture::NotNamed {
                    selector: device.selector.clone(),
                    why: format!("withdrawn: host-assembled ({})", kind.label()),
                });
            }
        }
    }
    (named, reports, counts)
}

/// Author one classified device: the table state into the facts, the
/// claimed scheme's table node (and the hybrid's second view) into the
/// topology.
fn author_one(
    id: NodeId,
    device: &Device,
    classification: &partman_table_parser::Classification,
    facts: &mut Facts,
    nodes: &mut Vec<NamingFields>,
    edges: &mut Vec<Edge>,
) -> DeviceCapture {
    let (state, state_name) = authored_state(&classification.state);
    facts.table_states.insert(id, state);
    let hybrid = classification.conditions.contains(&Condition::HybridMbr);
    let Some((role, name)) = classification.scheme.map(scheme_role) else {
        return DeviceCapture::Authored {
            selector: device.selector.clone(),
            node: id,
            state: state_name,
            scheme: None,
            hybrid: false,
        };
    };
    let table = NamingFields::PartitionTable { parent: id, role };
    if let Ok(table_id) = derive_id(&table) {
        nodes.push(table);
        edges.push(Edge {
            kind: EdgeKind::Containment,
            source: id,
            target: table_id,
        });
    }
    if hybrid {
        let view = NamingFields::PartitionTable {
            parent: id,
            role: TableRole::HybridMbr,
        };
        if let Ok(view_id) = derive_id(&view) {
            nodes.push(view);
            edges.push(Edge {
                kind: EdgeKind::Containment,
                source: id,
                target: view_id,
            });
        }
    }
    DeviceCapture::Authored {
        selector: device.selector.clone(),
        node: id,
        state: state_name,
        scheme: Some(name),
        hybrid,
    }
}

/// Read the two windows and classify them, or say why not — every arm a
/// withheld state, never a guessed one.
fn window_and_classify(
    device: &Device,
    total_bytes: u64,
    reader: &dyn DeviceReader,
) -> Result<partman_table_parser::Classification, String> {
    let Some(number) = device.device_number.as_deref() else {
        return Err("the device number did not answer; the node cannot be bracketed".to_owned());
    };
    let sector_size = sector_size_of(device)?;
    if !total_bytes.is_multiple_of(u64::from(sector_size)) {
        return Err(format!(
            "the byte total is not a multiple of the stated sector size {sector_size}"
        ));
    }
    let geometry = Geometry {
        sector_size,
        total_sectors: total_bytes / u64::from(sector_size),
    };
    let windows = reader
        .windows(&device.entry, number, geometry)
        .map_err(|refusal| format!("the window read refused: {refusal}"))?;
    classify(&windows.head, &windows.tail, geometry)
        .map_err(|refusal| format!("classification refused: {}", refusal.detail()))
}

/// The stated sector size, from the adapter's own observation set.
fn sector_size_of(device: &Device) -> Result<u32, String> {
    let key = "linux-sysfs:logical_block_size";
    let text = device
        .properties
        .iter()
        .find(|(name, _)| name == key)
        .and_then(|(_, observations)| observations.observations.first())
        .and_then(|observation| match &observation.outcome {
            Outcome::Observed {
                value: Value::Text(text),
            } => Some(text.clone()),
            _ => None,
        })
        .ok_or_else(|| "the logical sector size did not answer".to_owned())?;
    text.trim().parse::<u32>().map_err(|_| {
        format!(
            "the logical sector size is not a number: {} bytes of text",
            text.len()
        )
    })
}

/// Map the parser's state into the domain's, with its reporting name.
fn authored_state(state: &ParsedState) -> (TableState, &'static str) {
    match state {
        ParsedState::Present { checksum } => (TableState::present(*checksum), "present"),
        ParsedState::Absent => (TableState::Absent, "absent"),
        ParsedState::Indeterminate { basis } => match basis {
            partman_table_parser::IndeterminateBasis::Ambiguous => (
                TableState::Indeterminate {
                    cause: IndeterminateCause::Ambiguous,
                },
                "indeterminate-ambiguous",
            ),
            partman_table_parser::IndeterminateBasis::Unreadable => (
                TableState::Indeterminate {
                    cause: IndeterminateCause::Unreadable,
                },
                "indeterminate-unreadable",
            ),
        },
    }
}

/// The claimed scheme's table role and reporting name.
fn scheme_role(scheme: Scheme) -> (TableRole, &'static str) {
    match scheme {
        Scheme::Gpt => (TableRole::Gpt, "gpt"),
        Scheme::Mbr => (TableRole::Mbr, "mbr"),
        Scheme::Apm => (TableRole::Apm, "apm"),
    }
}

/// One envelope observation, attributed to this crate.
fn record(
    provenance: &mut Vec<(String, PropertyObservations)>,
    selector: &str,
    property: &str,
    outcome: Outcome,
) {
    provenance.push((
        format!("{selector}:{property}"),
        PropertyObservations {
            observations: vec![Observation {
                adapter: "partman-helper-linux".to_owned(),
                adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
                method: Method::Direct,
                outcome,
            }],
        },
    ));
}

/// The capture's whole-device byte unit, exported for the instrument.
#[must_use]
pub const fn size_unit() -> u64 {
    SIZE_UNIT
}

/// The window size, re-exported where the instrument reports it.
#[must_use]
pub const fn window_bytes() -> u64 {
    WINDOW_BYTES
}
