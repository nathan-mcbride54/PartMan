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
use super::topology::{EdgeKind, ReferentRule, Topology, naming_referent_rule};

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
    /// Whether the two ranges share a byte, in one frame. Ranges in
    /// different frames never intersect: nothing here translates.
    pub(crate) fn intersects(&self, other: &Self) -> bool {
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
    /// An extent's `host` is not the containment root the node's own name
    /// leads to (ADR-0037's anchoring rule, enforced by ADR-0046 on issue
    /// #333). The name and the extent are two claims about which address
    /// space the node's bytes are positioned in — the name through the
    /// containment relation it embeds, the extent directly — and they
    /// disagree. Both are named here, side by side, so a refused capture
    /// can be answered.
    ExtentFrameDisagreesWithName {
        /// The node.
        node: NodeId,
        /// The frame the extent declares.
        declared: NodeId,
        /// The containment root the name derives.
        derived: NodeId,
    },
    /// A containment edge nests a node inside one parent while the node's
    /// own name positions it inside another (ADR-0046, on the strength
    /// ADR-0045 held beside issue #333). The edge and the name are two
    /// claims about the same relation; the body is refused rather than
    /// either being preferred, both parents named.
    ContainmentEdgeDisagreesWithName {
        /// The child.
        child: NodeId,
        /// The parent the edge names.
        edge_parent: NodeId,
        /// The parent the child's own name embeds.
        named_parent: NodeId,
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
            Self::ExtentFrameDisagreesWithName {
                node,
                declared,
                derived,
            } => write!(
                formatter,
                "extent of {node} is framed on {declared}, but its name leads to containment root {derived}"
            ),
            Self::ContainmentEdgeDisagreesWithName {
                child,
                edge_parent,
                named_parent,
            } => write!(
                formatter,
                "containment edge nests {child} in {edge_parent}, but its name positions it in {named_parent}"
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
/// The other eleven pairs do: a table, signature or file system inside a
/// device, a signature or file system inside a partition, and a
/// signature, file system or table inside a volume (ADR-0044) or a
/// multipath node (ADR-0045), all lie within their parent's bytes — a
/// volume and a multipath node declare no extent of their own, so their
/// six pairs are geometric in kind and never compared.
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
/// arm that needs it. Since ADR-0046 (issue #333) the three positional
/// claims a body can make about a node — its name, the containment edge
/// that nests it, and its extent — are pairwise compared: the extent's
/// frame is the containment root the name leads to, the edge nests the
/// node where the name says, and the extent lies within the edge's
/// parent where the pair is geometric.
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
        // ADR-0037's anchoring rule, enforced (ADR-0046, issue #333): a
        // range in a containment forest is expressed in that forest's
        // root address space, and the root is derived from the node's own
        // name — never from the edge set, which a body may omit, and never
        // by replacing the declared host, which other consumers read.
        if let Some(derived) = frame_root(topology, *node)
            && extent.host != derived
        {
            return Err(FactError::ExtentFrameDisagreesWithName {
                node: *node,
                declared: extent.host,
                derived,
            });
        }
    }

    containment_agrees_with_names(topology)?;
    containment_agrees_with_extents(topology, facts)
}

/// A containment edge and the target's own name are two claims about the
/// same relation — which node the target's bytes lie inside. Where the
/// name embeds a containment source, the edge nests the node in that
/// source or the body is refused (ADR-0046). A node whose name embeds
/// none, or whose positioning field is the open one, is not asked.
fn containment_agrees_with_names(topology: &Topology) -> Result<(), FactError> {
    for edge in topology.edges() {
        if edge.kind != EdgeKind::Containment {
            continue;
        }
        let Some(fields) = kind_of(topology, edge.target) else {
            continue;
        };
        if let NamedPosition::Inside(named_parent) = named_position(fields)
            && named_parent != edge.source
        {
            return Err(FactError::ContainmentEdgeDisagreesWithName {
                child: edge.target,
                edge_parent: edge.source,
                named_parent,
            });
        }
    }
    Ok(())
}

/// Where a node's own name positions it, read off the naming relation
/// through [`naming_referent_rule`] — never off the edge set, which a
/// body may omit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NamedPosition {
    /// The name embeds a containment source: a partition's table, a
    /// table's carrier, a signature's or file system's host, a
    /// conflicting entry's table. The node's bytes lie inside that node's.
    Inside(NodeId),
    /// The name embeds no containment source: a physical device, a
    /// volume, a multipath node — and the kinds that carry no extent. The
    /// node is a containment root.
    Root,
    /// The name's positioning field is the one the pair table has no
    /// opinion about — a backing extent's `host`, which appears in no
    /// containment pair. Its range lives in its host's own address space
    /// (`ExtentLocator::Range`), outside every containment forest.
    Outside,
}

fn named_position(fields: &NamingFields) -> NamedPosition {
    let mut position = NamedPosition::Root;
    for (field, referent) in fields.naming_referents() {
        match naming_referent_rule(fields.kind_name(), field) {
            ReferentRule::Sources(kinds) if kinds == [EdgeKind::Containment] => {
                position = NamedPosition::Inside(referent);
            }
            ReferentRule::Open => return NamedPosition::Outside,
            ReferentRule::Sources(_) => {}
        }
    }
    position
}

/// The nodes a node's own name positions it inside, nearest first, up
/// to and including its containment root; empty for a node outside every
/// containment forest, and for one whose walk cannot be completed on a
/// topology that [`Topology::build`] would not have built.
///
/// Every hop is a referent the build already resolved and kind-checked
/// (ADR-0045), so the walk climbs strictly — a signature to its
/// partition, the partition to its table, the table to its device — and
/// ends within the forest's depth; the visited guard is belt and braces.
fn named_ancestry(topology: &Topology, id: NodeId) -> Vec<NodeId> {
    let mut visited = BTreeSet::new();
    let mut ancestry = Vec::new();
    let mut current = id;
    while visited.insert(current) {
        let Some(fields) = kind_of(topology, current) else {
            return vec![];
        };
        match named_position(fields) {
            NamedPosition::Inside(referent) => {
                ancestry.push(referent);
                current = referent;
            }
            NamedPosition::Root => return ancestry,
            NamedPosition::Outside => return vec![],
        }
    }
    vec![]
}

/// The containment root a node's own name leads to: the address space
/// ADR-0037 says its extent is expressed in, enforced by ADR-0046. `None`
/// for a node outside every containment forest (a backing extent), which
/// the rule does not reach, and for an address no entry carries.
fn frame_root(topology: &Topology, id: NodeId) -> Option<NodeId> {
    let fields = kind_of(topology, id)?;
    match named_position(fields) {
        NamedPosition::Root => Some(id),
        NamedPosition::Inside(_) => named_ancestry(topology, id).last().copied(),
        NamedPosition::Outside => None,
    }
}

/// Whether `node`'s own name positions it inside `host`, at any depth: a
/// file system naming a partition, a signature naming a partition whose
/// table names a device. Never true of a node for itself.
pub(crate) fn names_within(topology: &Topology, node: NodeId, host: NodeId) -> bool {
    named_ancestry(topology, node).contains(&host)
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
        // One frame, by construction: the frame rule puts both extents on
        // the root the child's name leads to, and the edge-name rule puts
        // the edge's parent on that same path, so the child and its parent
        // share a frame and the child's bytes lie within the parent's.
        // ADR-0041 also admitted a child expressed in the parent's own
        // address space and left a frame the parent could not be compared
        // against alone; both spellings are refused by the frame rule
        // before this rule is reached, and `contains` fails closed across
        // frames should a body ever arrive here in one.
        if !parent.contains(child) {
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
/// declared ranges — containment descent bounded by declared geometry,
/// upward backing from signatures in the set, downward production from
/// destroyed substrate — seeded by the target and the declared table and
/// consumed extents; and, inside it, the destroyed class proper, seeded
/// by the target when the step's own ranges reach it, carried along the
/// same arms, and releasing what its members' names describe
/// (ADR-0043, ADR-0044).
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
    // The destroyed class proper (ADR-0043, generalized by ADR-0044): what
    // the step's own destruction of its target takes down. Seeded by the
    // target alone, when the step's declared destroyed ranges reach it,
    // and carried from there along the same four arms under the same
    // geometric bound as reach — never seeded by a range that merely
    // touches some other node, which is round 2's sibling capture, and
    // never by reach alone, which is what the six non-destroying
    // operations carry. Every node that enters it releases what its
    // name-roster says it describes: a table's partitions, and nothing
    // for every other kind. A subset of `cascade_destroyed`.
    let mut destroyed: BTreeSet<NodeId> = BTreeSet::new();
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
    // Release (issue #347, ADR-0043; propagated by ADR-0044 on issue
    // #360). A step whose own destroyed ranges reach its target destroys
    // the target; destruction carries from there through the cascade;
    // and a destroyed partition table releases every partition whose
    // name says it describes it — ADR-0018's own definition of the
    // destroyed class, "content that ceases to be referenced". Decided by
    // the step's target, the cascade from it, and the naming relation,
    // and by nothing else: never by whether a range from some other
    // step's target happens to touch the table's declared extent (round
    // 2's sibling capture, from a partition-target step), never by
    // whether the destroyed ranges cover that extent (round 1's one-byte
    // fail-open), and never by the edge set (an omitted edge is not an
    // escape). The released partitions are destroyed too: they descend
    // into what they carry by the ordinary geometry, a signature they
    // carry brings its consumer, and a table below them releases in turn.
    if range_destroyed.contains(&target) {
        destroy(topology, target, &mut destroyed, &mut cascade_destroyed);
    }

    loop {
        let mut changed = false;
        for edge in topology.edges() {
            // ADR-0039's carried-content reach: every node in the set
            // propagates to the content it carries, not only the
            // destroyed ones. That is what gives the six operations
            // which destroy nothing a reach at all.
            let source_in_set = range_destroyed.contains(&edge.source)
                || cascade_destroyed.contains(&edge.source)
                || affected.contains(&edge.source);
            // Destruction carries only from a destroyed source: the
            // target when the step destroys it, and whatever that has
            // already taken down. A range-destroyed node that is not the
            // target is reached, and descends, but establishes no
            // destruction of its own — its extent is authored content a
            // sibling's range can be made to touch.
            let source_destroyed = destroyed.contains(&edge.source);
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
                    if source_in_set
                        && descends_into(
                            topology,
                            facts,
                            target,
                            edge.kind,
                            edge.source,
                            edge.target,
                        )
                        && carry(
                            topology,
                            edge.target,
                            source_destroyed,
                            &mut destroyed,
                            &mut cascade_destroyed,
                        )
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
                    if source_in_set && affected.insert(edge.target) {
                        changed = true;
                    }
                    if source_in_set
                        && descends_into(
                            topology,
                            facts,
                            target,
                            edge.kind,
                            edge.source,
                            edge.target,
                        )
                        && carry(
                            topology,
                            edge.target,
                            source_destroyed,
                            &mut destroyed,
                            &mut cascade_destroyed,
                        )
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

/// One hop of the cascade: `node` is reached, and — when its source is
/// destroyed — destroyed too. Returns whether either set grew.
fn carry(
    topology: &Topology,
    node: NodeId,
    source_destroyed: bool,
    destroyed: &mut BTreeSet<NodeId>,
    cascade_destroyed: &mut BTreeSet<NodeId>,
) -> bool {
    let reached = cascade_destroyed.insert(node);
    let taken = source_destroyed && destroy(topology, node, destroyed, cascade_destroyed);
    reached || taken
}

/// Mark `node` destroyed and release what its destruction releases
/// (issue #347, ADR-0043; issue #360, ADR-0044). Returns whether
/// anything new entered the destroyed class.
///
/// A destroyed partition table releases every partition whose own name
/// says the table describes it, and a released partition is destroyed in
/// turn. Read off the naming relation — `Partition.parent_table`, which a
/// partition cannot be represented without — never off a containment
/// edge, which a body may omit; and quantified over the roster by
/// [`NamingFields::released_by_table`], so a kind that comes to name a
/// table lands in one place. For every other kind the roster names
/// nothing and the node is simply destroyed.
fn destroy(
    topology: &Topology,
    node: NodeId,
    destroyed: &mut BTreeSet<NodeId>,
    cascade_destroyed: &mut BTreeSet<NodeId>,
) -> bool {
    if !destroyed.insert(node) {
        return false;
    }
    cascade_destroyed.insert(node);
    let released: Vec<NodeId> = topology
        .entries()
        .iter()
        .filter(|entry| {
            let fields = match entry {
                NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
            };
            fields.released_by_table() == Some(node)
        })
        .map(NodeEntry::id)
        .collect();
    for partition in released {
        destroy(topology, partition, destroyed, cascade_destroyed);
    }
    true
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
/// Only a containment root's own arm is inherited — a physical device's
/// device-scope arm, or a multipath node's detection-only refusal
/// (ADR-0045), so content on `/dev/mapper/mpatha` refuses with the node
/// — never a sibling's anything. The ascent is a graph walk rather than a line
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
