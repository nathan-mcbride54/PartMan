//! ADR-0018's protection layer as pure functions (WP-010 increment 3e).
//!
//! Everything here is a deterministic function over a validated
//! [`Topology`], the evidence-contract facts, and a step's declared range
//! sets. Nothing reads a device, and nothing here is body content yet —
//! carrying the facts in the snapshot body is a later slice; this one
//! lands the closure and verdicts with their committed regressions.
//!
//! Three ADR-0018 rules are load-bearing:
//!
//! - **The residual arm is `Indeterminate`, never `Permitted`** — round
//!   three's fail-open default inverted, property-tested.
//! - **Release is destruction**: the affected set closes over destroyed
//!   ranges by containment descent, upward backing, and downward
//!   production restricted to destroyed substrate.
//! - **A step whose affected set reaches a non-`Permitted` node refuses
//!   construction with a typed artifact** — ADR-0012's axis discharged
//!   here for the pure layer, and at the plan constructor in a later
//!   slice.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::identity::TableState;
use super::naming::{AggregateTechnology, NamingFields, NodeEntry, NodeId};
use super::topology::{EdgeKind, Topology};

/// A host-qualified byte range: one address space per containment root
/// (ADR-0018 2.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostRange {
    /// The node whose address space the range lives in.
    pub host: NodeId,
    /// First byte.
    pub start: u64,
    /// Length in bytes.
    pub length: u64,
}

impl HostRange {
    fn intersects(&self, other: &Self) -> bool {
        self.host == other.host
            && self.start < other.start.saturating_add(other.length)
            && other.start < self.start.saturating_add(self.length)
    }
}

/// The device-scope transport arm's closed positive local list
/// (ADR-0018 2.5). Everything not positively local fails closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportClass {
    /// `NVMe` over `PCIe`.
    NvmePcie,
    /// SATA.
    Sata,
    /// Directly attached SAS.
    SasDirect,
    /// USB mass storage.
    Usb,
    /// SD/MMC.
    SdMmc,
    /// A paravirtualized local class.
    ParavirtualLocal,
    /// A recognized remote class — the network-block-device non-goal.
    RecognizedRemote,
    /// A transport this build cannot positively classify.
    Unrecognized,
}

/// The evidence-contract facts the pure layer consumes.
///
/// Each map is keyed by address; absence of a fact is honest absence and
/// fails closed at the arm that needs it. The facts are supplied by the
/// contract's byte and state layers; nothing here reads a device.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    /// Each node's extent in its host's address space, for the
    /// extent-bearing kinds.
    pub extents: BTreeMap<NodeId, HostRange>,
    /// Each physical device's transport class.
    pub transports: BTreeMap<NodeId, TransportClass>,
    /// Each aggregate's self-reported member count (ADR-C5: never a count
    /// of members observed).
    pub member_counts: BTreeMap<NodeId, u64>,
    /// Each physical device's ADR-C3 table state — one of MODEL-005's two
    /// authored fields, stamped when the helper produces the snapshot at
    /// validation (ADR-0014); body content, so a plan identity claiming a
    /// different state diverges at the boundary.
    pub table_states: BTreeMap<NodeId, TableState>,
}

/// The three-valued protection verdict (ADR-0018 2.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Positively permitted by an enumerated arm.
    Permitted,
    /// A Section 2.1 non-goal or recognized-remote refusal.
    Refused {
        /// The refusing arm's stated ground.
        ground: RefusalGround,
    },
    /// Not positively determinable — the residual arm, and every
    /// missing-fact arm. `blocked` at the capability surface.
    Indeterminate {
        /// What could not be determined.
        cause: IndeterminateGround,
    },
}

/// Why a node refuses (closed, citing Section 2.1's entries).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalGround {
    /// ZFS: detect pools and members; never mutate.
    Zfs,
    /// Windows Storage Spaces pool or space structure.
    StorageSpaces,
    /// Windows dynamic disks (LDM).
    Ldm,
    /// Apple Fusion: an APFS container whose self-reported member count
    /// is two or more.
    Fusion,
    /// A recognized remote transport (network block devices).
    RemoteTransport,
    /// The node is a consumed member of a refused consumer, or a product
    /// of a refused producer.
    InheritedFromConsumerOrProducer,
    /// The node inherits its root device's device-scope refusal.
    InheritedDeviceScope,
}

/// Why a node is indeterminate (closed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndeterminateGround {
    /// An unrecognized technology, kind, or discriminant.
    Unrecognized,
    /// A signature with no observed consumer — the orphan arm; the
    /// acknowledgment route is a plan-layer matter in a later slice.
    OrphanSignature,
    /// A collision group: no member is individually addressable.
    CollisionGroup,
    /// A fact the arm needs is absent (transport, member count).
    MissingFact,
    /// The node inherits its root device's device-scope indeterminacy.
    InheritedDeviceScope,
}

/// The effect-table entry a step declares (ADR-0018 2.3): the three range
/// sets over host-qualified extents. Release is destruction — a released
/// range belongs in `destroyed` even though no byte is overwritten.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StepRanges {
    /// The exact table extents written (never the parent device).
    pub written_table_extents: Vec<HostRange>,
    /// Free ranges consumed (verified free by the constructor).
    pub consumed: Vec<HostRange>,
    /// Ranges destroyed or released.
    pub destroyed: Vec<HostRange>,
}

/// A protection refusal: the typed artifact a non-permitted reach
/// produces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtectionRefusal {
    /// The node the closure reached.
    pub node: NodeId,
    /// Its verdict.
    pub verdict: Verdict,
}

impl fmt::Display for ProtectionRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "affected node {} is {:?}",
            self.node, self.verdict
        )
    }
}

/// A node's effective verdict: the worst of its own arm, its producer's
/// verdict where it is a produced node, and its root device's
/// device-scope verdict (node-local inheritance — never an edge
/// traversal to a sibling).
#[must_use]
pub fn node_verdict(topology: &Topology, facts: &Facts, id: NodeId) -> Verdict {
    let Some(entry) = topology.entries().iter().find(|entry| entry.id() == id) else {
        return Verdict::Indeterminate {
            cause: IndeterminateGround::Unrecognized,
        };
    };
    if matches!(entry, NodeEntry::Group { .. }) {
        return Verdict::Indeterminate {
            cause: IndeterminateGround::CollisionGroup,
        };
    }
    let fields = match entry {
        NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
    };
    let own = own_arm(topology, facts, id, fields);
    let with_producer = worst(own, producer_verdict(topology, facts, id));
    worst(with_producer, device_scope_verdict(topology, facts, id))
}

/// The affected set of a step (ADR-0018 2.3): a fixpoint over the
/// declared ranges — containment descent bounded by destroyed ranges,
/// upward backing from destroyed signatures, downward production from
/// destroyed substrate — seeded by the target and the declared table and
/// consumed extents.
#[must_use]
pub fn affected_set(
    topology: &Topology,
    facts: &Facts,
    target: NodeId,
    ranges: &StepRanges,
) -> BTreeSet<NodeId> {
    let mut affected: BTreeSet<NodeId> = BTreeSet::new();
    // Two destruction classes, deliberately. Range-destroyed nodes are
    // reached by the declared ranges themselves — containment reach IS
    // that intersection, and cascading a range-destroyed node into its
    // containment children would re-derive round two's sibling capture
    // through a device's own self-extent (this module's first draft did
    // exactly that, and the committed regression caught it).
    // Cascade-destroyed nodes — a consumer whose evidence died, a product
    // whose producer died — have no declared range in their own address
    // space, so containment descent applies to them alone.
    let mut range_destroyed: BTreeSet<NodeId> = BTreeSet::new();
    let mut cascade_destroyed: BTreeSet<NodeId> = BTreeSet::new();
    affected.insert(target);

    for entry in topology.entries() {
        let id = entry.id();
        if let Some(extent) = facts.extents.get(&id) {
            if ranges
                .destroyed
                .iter()
                .any(|range| range.intersects(extent))
            {
                range_destroyed.insert(id);
            }
            if ranges
                .written_table_extents
                .iter()
                .chain(ranges.consumed.iter())
                .any(|range| range.intersects(extent))
            {
                affected.insert(id);
            }
        }
    }

    loop {
        let mut changed = false;
        for edge in topology.edges() {
            let source_destroyed =
                range_destroyed.contains(&edge.source) || cascade_destroyed.contains(&edge.source);
            match edge.kind {
                // Containment descent only from cascade-destroyed nodes:
                // a destroyed product's hosted content dies with it.
                EdgeKind::Containment => {
                    if cascade_destroyed.contains(&edge.source)
                        && !range_destroyed.contains(&edge.target)
                        && cascade_destroyed.insert(edge.target)
                    {
                        changed = true;
                    }
                }
                // Upward backing, in ADR-0018's own two halves. Rule 3
                // is route-agnostic — "a BackingSignature IN THE SET
                // brings its consumer" — and the ADR contrasts it in
                // the same paragraph with rule 4's "in the set THROUGH
                // A DESTROYED RANGE". The delivered code gated both on
                // destruction; ADR-0038 frees the membership half and
                // leaves the substrate half gated, so a signature
                // reached by any route brings its consumer while only
                // a destroyed one takes its substrate down.
                EdgeKind::Backing => {
                    if (affected.contains(&edge.source) || source_destroyed)
                        && affected.insert(edge.target)
                    {
                        changed = true;
                    }
                    if source_destroyed
                        && !range_destroyed.contains(&edge.target)
                        && cascade_destroyed.insert(edge.target)
                    {
                        changed = true;
                    }
                }
                // Downward production restricted to destroyed substrate.
                EdgeKind::Production | EdgeKind::HostBacking => {
                    if source_destroyed
                        && !range_destroyed.contains(&edge.target)
                        && cascade_destroyed.insert(edge.target)
                    {
                        changed = true;
                    }
                }
                // Platform-membership is closure-inert in v1 (ADR-0019).
                EdgeKind::PlatformMembership => {}
            }
        }
        if !changed {
            break;
        }
    }

    affected.extend(range_destroyed.iter().copied());
    affected.extend(cascade_destroyed.iter().copied());
    affected
}

/// Whether a step constructs (ADR-0018 2.3): every affected node must be
/// `Permitted`. The first non-permitted node refuses with a typed
/// artifact. Acknowledgment-gated arms are a plan-layer matter in a later
/// slice; here they refuse.
///
/// # Errors
///
/// [`ProtectionRefusal`] naming the reached node and its verdict.
pub fn step_constructs(
    topology: &Topology,
    facts: &Facts,
    target: NodeId,
    ranges: &StepRanges,
) -> Result<BTreeSet<NodeId>, ProtectionRefusal> {
    let affected = affected_set(topology, facts, target, ranges);
    for id in &affected {
        let verdict = node_verdict(topology, facts, *id);
        if verdict != Verdict::Permitted {
            return Err(ProtectionRefusal { node: *id, verdict });
        }
    }
    Ok(affected)
}

fn own_arm(topology: &Topology, facts: &Facts, id: NodeId, fields: &NamingFields) -> Verdict {
    match fields {
        NamingFields::PhysicalDevice { .. } => match facts.transports.get(&id) {
            Some(
                TransportClass::NvmePcie
                | TransportClass::Sata
                | TransportClass::SasDirect
                | TransportClass::Usb
                | TransportClass::SdMmc
                | TransportClass::ParavirtualLocal,
            ) => Verdict::Permitted,
            Some(TransportClass::RecognizedRemote) => Verdict::Refused {
                ground: RefusalGround::RemoteTransport,
            },
            Some(TransportClass::Unrecognized) | None => Verdict::Indeterminate {
                cause: IndeterminateGround::MissingFact,
            },
        },
        NamingFields::Aggregate { technology, .. } => match technology {
            AggregateTechnology::Zfs => Verdict::Refused {
                ground: RefusalGround::Zfs,
            },
            AggregateTechnology::StorageSpaces => Verdict::Refused {
                ground: RefusalGround::StorageSpaces,
            },
            AggregateTechnology::Ldm => Verdict::Refused {
                ground: RefusalGround::Ldm,
            },
            AggregateTechnology::Apfs => match facts.member_counts.get(&id) {
                Some(count) if *count >= 2 => Verdict::Refused {
                    ground: RefusalGround::Fusion,
                },
                Some(_) => Verdict::Permitted,
                None => Verdict::Indeterminate {
                    cause: IndeterminateGround::MissingFact,
                },
            },
            AggregateTechnology::Lvm2 | AggregateTechnology::Mdraid => Verdict::Permitted,
            AggregateTechnology::Unrecognized { .. } => Verdict::Indeterminate {
                cause: IndeterminateGround::Unrecognized,
            },
        },
        NamingFields::BackingSignature { family, .. } => {
            if matches!(family, super::naming::SignatureFamily::Unrecognized { .. }) {
                return Verdict::Indeterminate {
                    cause: IndeterminateGround::Unrecognized,
                };
            }
            // A signature's own arm follows its observed consumer; no
            // consumer is the orphan arm (Indeterminate, remediable).
            let consumer = topology
                .edges()
                .iter()
                .find(|edge| edge.kind == EdgeKind::Backing && edge.source == id);
            match consumer {
                Some(edge) => match node_own_only(topology, facts, edge.target) {
                    Verdict::Refused { .. } => Verdict::Refused {
                        ground: RefusalGround::InheritedFromConsumerOrProducer,
                    },
                    Verdict::Indeterminate { cause } => Verdict::Indeterminate { cause },
                    Verdict::Permitted => Verdict::Permitted,
                },
                None => Verdict::Indeterminate {
                    cause: IndeterminateGround::OrphanSignature,
                },
            }
        }
        NamingFields::PartitionTable { .. }
        | NamingFields::Partition { .. }
        | NamingFields::FileSystem { .. }
        | NamingFields::EncryptionLayer { .. }
        | NamingFields::Volume { .. }
        | NamingFields::BackingExtent { .. }
        | NamingFields::ConflictingTableEntry { .. } => {
            if matches!(fields, NamingFields::ConflictingTableEntry { .. }) {
                Verdict::Indeterminate {
                    cause: IndeterminateGround::Unrecognized,
                }
            } else {
                Verdict::Permitted
            }
        }
        NamingFields::MultipathNode { .. } => Verdict::Refused {
            ground: RefusalGround::RemoteTransport,
        },
    }
}

fn node_own_only(topology: &Topology, facts: &Facts, id: NodeId) -> Verdict {
    let Some(entry) = topology.entries().iter().find(|entry| entry.id() == id) else {
        return Verdict::Indeterminate {
            cause: IndeterminateGround::Unrecognized,
        };
    };
    let fields = match entry {
        NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
    };
    own_arm(topology, facts, id, fields)
}

fn producer_verdict(topology: &Topology, facts: &Facts, id: NodeId) -> Verdict {
    let producer = topology.edges().iter().find(|edge| {
        matches!(edge.kind, EdgeKind::Production | EdgeKind::HostBacking) && edge.target == id
    });
    match producer {
        Some(edge) => match node_own_only(topology, facts, edge.source) {
            Verdict::Refused { .. } => Verdict::Refused {
                ground: RefusalGround::InheritedFromConsumerOrProducer,
            },
            other => other,
        },
        None => Verdict::Permitted,
    }
}

fn device_scope_verdict(topology: &Topology, facts: &Facts, id: NodeId) -> Verdict {
    // Walk reverse containment to the root; only a physical device's
    // device-scope arm is inherited (never a sibling's anything).
    let mut current = id;
    loop {
        let parent = topology
            .edges()
            .iter()
            .find(|edge| edge.kind == EdgeKind::Containment && edge.target == current)
            .map(|edge| edge.source);
        match parent {
            Some(parent) => current = parent,
            None => break,
        }
    }
    if current == id {
        return Verdict::Permitted;
    }
    match node_own_only(topology, facts, current) {
        Verdict::Refused { .. } => Verdict::Refused {
            ground: RefusalGround::InheritedDeviceScope,
        },
        Verdict::Indeterminate { .. } => Verdict::Indeterminate {
            cause: IndeterminateGround::InheritedDeviceScope,
        },
        Verdict::Permitted => Verdict::Permitted,
    }
}

fn worst(left: Verdict, right: Verdict) -> Verdict {
    match (&left, &right) {
        (Verdict::Refused { .. }, _) => left,
        (_, Verdict::Refused { .. }) => right,
        (Verdict::Indeterminate { .. }, _) => left,
        (_, Verdict::Indeterminate { .. }) => right,
        _ => Verdict::Permitted,
    }
}
