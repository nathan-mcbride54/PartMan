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
/// — [`EdgeKind::Containment`](super::topology::EdgeKind::Containment)'s
/// "positional nesting inside one addressable byte space", and ADR-0037's
/// anchoring rule expressed in that forest's root address space.
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

    /// Whether `other` lies wholly within `self`, in the same frame.
    fn contains(&self, other: &Self) -> bool {
        self.host == other.host
            && other.start >= self.start
            && other.start.saturating_add(other.length) <= self.start.saturating_add(self.length)
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

/// Why a fact set is refused against its topology (issues #349 and
/// #356; ADR-0041). Every arm names the node it is about, so a refused
/// capture can be answered rather than merely rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactError {
    /// A fact is keyed by an address no absorbed entry carries. Such a
    /// fact would never enter the body bytes, so an in-process snapshot
    /// holding it and its own encoding would disagree about what facts
    /// exist.
    OrphanFact {
        /// The fact's key, as the body spells it.
        fact: &'static str,
        /// The address that resolves to nothing.
        node: NodeId,
    },
    /// A fact on a kind that does not carry it — a transport on a
    /// partition, a member count on a device, a table state on a
    /// volume, an extent on an aggregate. The predicate is the one the
    /// decode path reads, applied at construction so an in-process
    /// snapshot can never hold what its own bytes would refuse.
    MisplacedFact {
        /// The fact's key, as the body spells it.
        fact: &'static str,
        /// The node carrying it.
        node: NodeId,
        /// That node's kind name.
        kind: &'static str,
    },
    /// An extent's `host` is an address no absorbed entry carries. Edge
    /// endpoints and naming referents already had to resolve; a frame
    /// that resolves to nothing is a range in no address space.
    UnresolvedExtentHost {
        /// The node whose extent is framed on the missing host.
        node: NodeId,
        /// The address that resolves to nothing.
        host: NodeId,
    },
    /// An extent of zero bytes. An extent is a positional claim about
    /// bytes; a claim about no bytes is not one — `intersects` can never
    /// be true of it, so a signature declared this way is invisible to
    /// the byte scan. A structure whose bytes are unknown omits the fact
    /// (honest absence, failing closed) rather than declaring nothing.
    ZeroLengthExtent {
        /// The node.
        node: NodeId,
    },
    /// `start + length` exceeds `u64::MAX`; the declared range has no
    /// end and is not a range.
    ExtentOverflows {
        /// The node.
        node: NodeId,
    },
    /// A containment child's extent lies outside the extent of the parent
    /// the edge nests it in, in a frame the two can be compared in. The
    /// edge and the fact are both positional claims about the same bytes
    /// and they contradict; the body is refused rather than either being
    /// preferred.
    ExtentOutsideContainmentParent {
        /// The child.
        child: NodeId,
        /// The parent named by the containment edge.
        parent: NodeId,
    },
}

impl fmt::Display for FactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OrphanFact { fact, node } => {
                write!(formatter, "fact `{fact}` keyed by unknown address {node}")
            }
            Self::MisplacedFact { fact, node, kind } => {
                write!(
                    formatter,
                    "fact `{fact}` on {kind} {node}, which does not carry it"
                )
            }
            Self::UnresolvedExtentHost { node, host } => {
                write!(
                    formatter,
                    "extent of {node} is framed on unknown address {host}"
                )
            }
            Self::ZeroLengthExtent { node } => {
                write!(formatter, "extent of {node} declares zero bytes")
            }
            Self::ExtentOverflows { node } => {
                write!(formatter, "extent of {node} has no end below u64::MAX")
            }
            Self::ExtentOutsideContainmentParent { child, parent } => write!(
                formatter,
                "extent of {child} lies outside its containment parent {parent}"
            ),
        }
    }
}

impl std::error::Error for FactError {}

/// Whether a containment pair is *geometric* — the parent's extent is the
/// region its children lie in — as opposed to *structural*, where the
/// parent's extent is its own bytes and the children lie beside them.
///
/// A partition table's extent is the table structure — protective MBR,
/// header, entry array — not the region it governs: every committed GPT
/// fixture puts `p1` at `table.start + table.length` exactly, and a
/// BIOS-boot layout puts one entry *inside* the first MiB and the rest
/// beyond it. So `partition-table` → `partition` and
/// `partition-table` → `conflicting-table-entry` carry no span claim.
/// The other seven pairs do: a table, signature or file system inside a
/// device, and a signature or file system inside a partition or a
/// volume, all lie within their parent's bytes.
fn containment_pair_is_geometric(source_kind: &str) -> bool {
    !matches!(source_kind, "partition-table")
}

/// Refuse a fact set that its topology cannot carry (issues #349, #356;
/// ADR-0041). Applied by [`TopologySnapshot::assemble`], which both the
/// in-process constructors and the decode boundary run through, so no
/// snapshot can exist whose facts would be refused on the other path.
///
/// Every rule refuses only what is positively unlawful. Absence of a fact
/// is never refused here — it is honest absence and fails closed at the
/// arm that needs it — and a child extent expressed in a frame its parent
/// cannot be compared against is left alone (that is ADR-0037's held
/// enforcement, issue #333, and not this function's to decide).
///
/// # Errors
///
/// [`FactError`] naming the first offending fact.
pub fn validate_facts(topology: &Topology, facts: &Facts) -> Result<(), FactError> {
    // Every fact key names an absorbed entry, and that entry's kind
    // carries the fact.
    placed(topology, "transport", facts.transports.keys(), |fields| {
        matches!(fields, NamingFields::PhysicalDevice { .. })
    })?;
    placed(
        topology,
        "member_count",
        facts.member_counts.keys(),
        |fields| matches!(fields, NamingFields::Aggregate { .. }),
    )?;
    placed(
        topology,
        "table_state",
        facts.table_states.keys(),
        |fields| matches!(fields, NamingFields::PhysicalDevice { .. }),
    )?;
    placed(
        topology,
        "extent_host",
        facts.extents.keys(),
        NamingFields::may_carry_extent,
    )?;

    // Every extent is a range: framed on an entry, of at least one byte,
    // with an end below `u64::MAX`.
    for (node, extent) in &facts.extents {
        if kind_of(topology, extent.host).is_none() {
            return Err(FactError::UnresolvedExtentHost {
                node: *node,
                host: extent.host,
            });
        }
        if extent.length == 0 {
            return Err(FactError::ZeroLengthExtent { node: *node });
        }
        if extent.start.checked_add(extent.length).is_none() {
            return Err(FactError::ExtentOverflows { node: *node });
        }
    }

    containment_agrees_with_extents(topology, facts)
}

/// Every key in `nodes` names an absorbed entry whose kind `carries` the
/// fact called `fact`.
fn placed<'a>(
    topology: &Topology,
    fact: &'static str,
    nodes: impl Iterator<Item = &'a NodeId>,
    carries: impl Fn(&NamingFields) -> bool,
) -> Result<(), FactError> {
    for node in nodes {
        match kind_of(topology, *node) {
            None => return Err(FactError::OrphanFact { fact, node: *node }),
            Some(fields) if !carries(fields) => {
                return Err(FactError::MisplacedFact {
                    fact,
                    node: *node,
                    kind: fields.kind_name(),
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// A containment edge and the two extents at its ends are three claims
/// about the same bytes. Where the pair is geometric and the frames are
/// comparable, the child lies within the parent or the body is refused.
fn containment_agrees_with_extents(topology: &Topology, facts: &Facts) -> Result<(), FactError> {
    for edge in topology.edges() {
        if edge.kind != EdgeKind::Containment {
            continue;
        }
        let Some(source_kind) = kind_of(topology, edge.source).map(NamingFields::kind_name) else {
            continue;
        };
        if !containment_pair_is_geometric(source_kind) {
            continue;
        }
        let (Some(parent), Some(child)) = (
            facts.extents.get(&edge.source),
            facts.extents.get(&edge.target),
        ) else {
            continue;
        };
        let inside = if child.host == parent.host {
            // Both in one frame: the child's bytes lie within the parent's.
            parent.contains(child)
        } else if child.host == edge.source {
            // The child is expressed in the parent's own address space:
            // its end lies within the parent's length. (Checked arithmetic
            // holds by the rule above; saturating is belt and braces.)
            child.start.saturating_add(child.length) <= parent.length
        } else {
            // A frame this parent cannot be compared against — ADR-0037's
            // held enforcement, not a contradiction this rule can see.
            true
        };
        if !inside {
            return Err(FactError::ExtentOutsideContainmentParent {
                child: edge.target,
                parent: edge.source,
            });
        }
    }
    Ok(())
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
    // Two destruction classes, still — a node reached by a declared range
    // and a node whose evidence or producer died answer differently at
    // the arms that ask about substrate. What separated them until
    // ADR-0039 was descent: only the cascade class descended, because
    // cascading a range-destroyed node into its containment children
    // would re-derive round two's sibling capture through a device's own
    // self-extent (this module's first draft did exactly that, and the
    // committed regression caught it). Both classes descend now, and it
    // is `descends_into` that refuses that self-extent hop — which is
    // what let the containment bound become a statement about geometry
    // rather than about which class a node landed in.
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
            // ADR-0039's carried-content reach: every node in the set
            // propagates to the content it carries, not only the
            // destroyed ones. That is what gives the six operations
            // which destroy nothing a reach at all.
            let source_destroyed = range_destroyed.contains(&edge.source)
                || cascade_destroyed.contains(&edge.source)
                || affected.contains(&edge.source);
            match edge.kind {
                // Containment descent into carried content, and downward
                // production into a destroyed producer's products: one
                // arm since ADR-0039, because the difference between them
                // is now a clause of `descends_into` rather than a
                // different rule. Production and host-backing targets are
                // products, which carry no extent of their own; a
                // containment target may, and is compared against its
                // source's.
                EdgeKind::Containment | EdgeKind::Production | EdgeKind::HostBacking => {
                    if source_destroyed
                        && descends_into(
                            topology,
                            facts,
                            target,
                            edge.kind,
                            edge.source,
                            edge.target,
                        )
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
                        && descends_into(
                            topology,
                            facts,
                            target,
                            edge.kind,
                            edge.source,
                            edge.target,
                        )
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

fn kind_of(topology: &Topology, id: NodeId) -> Option<&NamingFields> {
    topology
        .entries()
        .iter()
        .find(|entry| entry.id() == id)
        .map(|entry| match entry {
            NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
        })
}

/// Whether descent may cross this edge (issue #338's held half).
///
/// The bound refuses a hop only where the declared geometry positively
/// contradicts containment. Every absence, mismatch or ambiguity admits,
/// so the closure can never reach less than the committed one does — the
/// extents it reads are authored body content, and a predicate that can
/// subtract reach hands that content a lever.
fn descends_into(
    topology: &Topology,
    facts: &Facts,
    step_target: NodeId,
    kind: EdgeKind,
    source: NodeId,
    target: NodeId,
) -> bool {
    // An extent on a kind the body format forbids one on is not evidence
    // of anything: the closure reads the same predicate the decode path
    // does, so an unlawful fact can never steer reach.
    let parent = facts
        .extents
        .get(&source)
        .filter(|_| kind_of(topology, source).is_none_or(NamingFields::may_carry_extent));
    // A node whose extent is expressed in its own address space declares a
    // frame, not a claim to have been destroyed: every range on the disk
    // lies inside a device's self-extent, so descending out of one would
    // re-derive round two's sibling capture. The committed code says the
    // same thing by never descending from a range-destroyed node at all.
    //
    // Unless the frame root is the step's own target (issue #353). The
    // target is in the set by identity, not because a range intersected
    // its self-extent, and ADR-0039's rule is that a step reaches the
    // content its target carries: for a disk, the table and whatever is
    // hosted directly on the device. Without this hop a whole-disk
    // layout's protection came entirely from the over-claimed
    // whole-device write this issue removes, and six gates opened over a
    // live pool once it was removed. Descent from the target's children
    // onward is still bounded by geometry below.
    if parent.is_some_and(|extent| extent.host == source) && source != step_target {
        return false;
    }
    match (parent, facts.extents.get(&target)) {
        // The source declares no bytes at all. Every such node is a
        // product — the decode rule forbids an extent on an aggregate,
        // volume, encryption layer or multipath node — and the committed
        // closure descends out of them unconditionally. So does this one.
        (None, _) => true,
        // Both sides declare bytes. Descend into content that lies within
        // the source, into content framed by the source, and into anything
        // expressed in a frame this one cannot be compared against.
        (Some(extent), Some(child)) => {
            child.host == source || child.host != extent.host || extent.contains(child)
        }
        // The source declares bytes and the child does not. On the
        // propagating arms that is the ordinary shape — a product carries
        // no extent by construction — and the committed closure descends.
        // Under containment it is a node positioned inside a known frame
        // whose position is unstated, and the committed closure never
        // descends out of an extent-bearing containment source at all;
        // admitting it here would capture a sibling that merely lacks a
        // fact.
        (Some(_), None) => kind != EdgeKind::Containment,
    }
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
            // A signature's own arm follows its observed consumers; no
            // consumer is the orphan arm (Indeterminate, remediable).
            // Membership carries unbounded in-degree (MODEL-002), so the
            // arm folds `worst` over every consumer rather than taking
            // whichever edge sorts first: the sort key is a derived
            // address over hashed naming fields, so a first-match choice
            // is one an author selects (issue #355).
            let mut consumers = topology
                .edges()
                .iter()
                .filter(|edge| edge.kind == EdgeKind::Backing && edge.source == id)
                .peekable();
            if consumers.peek().is_none() {
                return Verdict::Indeterminate {
                    cause: IndeterminateGround::OrphanSignature,
                };
            }
            consumers.fold(Verdict::Permitted, |carried, edge| {
                worst(
                    carried,
                    match node_own_only(topology, facts, edge.target) {
                        Verdict::Refused { .. } => Verdict::Refused {
                            ground: RefusalGround::InheritedFromConsumerOrProducer,
                        },
                        other => other,
                    },
                )
            })
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

/// A produced node's inherited verdict: the worst of every producer the
/// body declares.
///
/// Nothing bounds how many producers a node presents — the endpoint-pair
/// table admits `Production` from both an encryption layer and an
/// aggregate, `Topology::build` enforces no in-degree rule, and a node's
/// naming fields name at most one producer while the edges are authored
/// separately. Taking whichever edge sorts first therefore lets an
/// author choose the inherited verdict, because the sort key is a
/// derived address over hashed fields (issue #355). Folding admits no
/// such choice: with one producer it is that producer's verdict, and
/// with none it is `Permitted`, exactly as before.
fn producer_verdict(topology: &Topology, facts: &Facts, id: NodeId) -> Verdict {
    topology
        .edges()
        .iter()
        .filter(|edge| {
            matches!(edge.kind, EdgeKind::Production | EdgeKind::HostBacking) && edge.target == id
        })
        .fold(Verdict::Permitted, |carried, edge| {
            worst(
                carried,
                match node_own_only(topology, facts, edge.source) {
                    Verdict::Refused { .. } => Verdict::Refused {
                        ground: RefusalGround::InheritedFromConsumerOrProducer,
                    },
                    other => other,
                },
            )
        })
}

/// A node's inherited device-scope verdict: the worst over every
/// containment root above it.
///
/// Only a physical device's device-scope arm is inherited (never a
/// sibling's anything). The ascent is a graph walk rather than a line
/// because nothing bounds a node's containment in-degree: the pair
/// table admits a `BackingSignature` or `FileSystem` under both a
/// physical device and a partition, and `Topology::build` enforces no
/// cardinality rule. Following whichever parent sorted first let an
/// author move a node under a decoy device and inherit that device's
/// arm instead of its real host's — a refusal turned into `Permitted`
/// by one added edge (issue #355). Every root is visited and folded
/// with `worst`, so an added parent can only ever add refusal, and a
/// node with a single ancestry answers exactly as before. Termination
/// rests on the visited set, not on the pair table's acyclicity.
fn device_scope_verdict(topology: &Topology, facts: &Facts, id: NodeId) -> Verdict {
    let mut roots = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut pending = vec![id];
    while let Some(current) = pending.pop() {
        if !seen.insert(current) {
            continue;
        }
        let mut ascended = false;
        for edge in topology.edges() {
            if edge.kind == EdgeKind::Containment && edge.target == current {
                ascended = true;
                pending.push(edge.source);
            }
        }
        if !ascended {
            roots.insert(current);
        }
    }
    roots
        .into_iter()
        .filter(|root| *root != id)
        .fold(Verdict::Permitted, |carried, root| {
            worst(
                carried,
                match node_own_only(topology, facts, root) {
                    Verdict::Refused { .. } => Verdict::Refused {
                        ground: RefusalGround::InheritedDeviceScope,
                    },
                    Verdict::Indeterminate { .. } => Verdict::Indeterminate {
                        cause: IndeterminateGround::InheritedDeviceScope,
                    },
                    Verdict::Permitted => Verdict::Permitted,
                },
            )
        })
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
