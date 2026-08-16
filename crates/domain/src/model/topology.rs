//! Edges and topology construction per ADR-0019 (WP-010 increment 3b).
//!
//! An edge connects two addressed entries and carries one of the five edge
//! kinds MODEL-002 names, each with a **semantics class** — ADR-0018's
//! handover: the bind set traverses "the bytes of A live within or derive
//! from B" in reverse over semantics, not over names, and the
//! platform-membership class is bind-inert while multipath is
//! detection-only (ADR-0011).
//!
//! [`Topology::build`] is fail-closed with an artifact: every refusal is a
//! typed [`TopologyError`], never a panic and never an encoder failure. It
//! enforces, at construction, the endpoint-pair table below — which is
//! where the no-sibling-capture theorem's premise ("no backing or
//! production edge targets a physical device") lives as code rather than
//! review prose. The tests enumerate the table and the full complement of
//! forbidden triples exhaustively.

use std::collections::BTreeSet;
use std::fmt;

use super::naming::{NamingError, NamingFields, NodeEntry, NodeId, absorb};

/// The five edge kinds (MODEL-002, extended in 11.1.0 by ADR-0019).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeKind {
    /// Positional nesting inside one addressable byte space
    /// (device → table → partition; a host carrying a signature, a file
    /// system, or — for a volume — a partition table of its own, so a
    /// partitioned mdraid array or a GPT inside a mapped volume is
    /// `producer → volume → table → partition`, ADR-0044; a multipath
    /// node carries the same three, so content on `/dev/mapper/mpatha`
    /// inherits the node's detection-only refusal, ADR-0045). "Inside one
    /// byte space" is a claim about the *frame*, not about the parent's
    /// span: a table's own extent is its header bytes, and the partitions
    /// it describes lie beside them, so the two `partition-table`-sourced
    /// pairs carry no span claim (ADR-0041) and what a destroyed table
    /// releases is decided by the naming relation, not by this edge
    /// (ADR-0043). The other eleven pairs are geometric.
    Containment,
    /// Evidence to consumer: a backing signature backing its aggregate or
    /// encryption layer.
    Backing,
    /// Producer to product: an encryption layer or aggregate producing a
    /// virtual device or volume.
    Production,
    /// "The bytes of A live within B": a backing extent (file or byte
    /// range) carrying a host-backed virtual device.
    HostBacking,
    /// Platform-asserted composition: a platform-assembled multipath node
    /// and its member representation. Detection-only; closure- and
    /// bind-inert until the spec change ADR-0011 names.
    PlatformMembership,
}

/// The semantics class an edge kind carries (ADR-0018's handover).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticsClass {
    /// The bytes of the source live within, or derive from, the target's
    /// side of the relation; CONC-001's bind set traverses these in
    /// reverse.
    BytesWithinOrDerive,
    /// A platform's own composition assertion, carried without inference;
    /// not traversed by the v1 bind set.
    PlatformAsserted,
}

impl EdgeKind {
    /// The kind's semantics class.
    #[must_use]
    pub const fn semantics(self) -> SemanticsClass {
        match self {
            Self::Containment | Self::Backing | Self::Production | Self::HostBacking => {
                SemanticsClass::BytesWithinOrDerive
            }
            Self::PlatformMembership => SemanticsClass::PlatformAsserted,
        }
    }

    /// Whether the v1 bind set traverses this kind (CONC-001, ADR-0019).
    #[must_use]
    pub const fn traversed_by_bind_set(self) -> bool {
        matches!(self.semantics(), SemanticsClass::BytesWithinOrDerive)
    }

    const ALL: [Self; 5] = [
        Self::Containment,
        Self::Backing,
        Self::Production,
        Self::HostBacking,
        Self::PlatformMembership,
    ];

    /// Every edge kind, for exhaustive tests.
    #[must_use]
    pub const fn all() -> &'static [Self; 5] {
        &Self::ALL
    }
}

/// A directed edge between two addressed entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    /// The edge kind.
    pub kind: EdgeKind,
    /// The source entry's address.
    pub source: NodeId,
    /// The target entry's address.
    pub target: NodeId,
}

/// A validated topology: absorbed entries plus edges whose referents
/// resolve and whose endpoint kinds satisfy the edge kind's pair table.
#[derive(Debug, PartialEq, Eq)]
pub struct Topology {
    entries: Vec<NodeEntry>,
    edges: Vec<Edge>,
}

impl Topology {
    /// The absorbed entries, sorted by address.
    #[must_use]
    pub fn entries(&self) -> &[NodeEntry] {
        &self.entries
    }

    /// The validated edges, sorted.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Build a topology from observed nodes and edges (ADR-0019).
    ///
    /// Nodes are absorbed per the collision rule; every address an
    /// absorbed node's own *name* embeds must then resolve to an absorbed
    /// entry (issue #354); edges are then checked: both endpoints must
    /// resolve to absorbed addresses (a decoder recomputes and rejects
    /// unknown referents — this is that rule at construction), self-edges
    /// and duplicates are refused, and each edge's endpoint kinds must
    /// appear in its kind's pair table. The result is a deterministic
    /// function of the observed sets.
    ///
    /// The naming sweep asks two things of every referent (issue #354,
    /// both halves; ADR-0045). It must **resolve** to an absorbed entry,
    /// and that entry's **kind** must be one the endpoint-pair table
    /// admits as the source of the relation the field names — read off
    /// [`endpoint_pair_allowed`] through [`naming_referent_rule`], the
    /// same table the edge check below reads, so there is no second
    /// authored list to drift from the first. A partition's `parent_table`
    /// must be a partition table; a table's `parent`, a signature's or
    /// file system's `host`, is a kind that may carry one — a device, a
    /// partition, a volume, a multipath node — exactly as the containment
    /// pairs say; a volume's `producer` is whatever `Production` or
    /// `HostBacking` admits. A backing extent's `host` is the one open
    /// field: no edge kind targets a backing extent, so the table has no
    /// opinion and the field must only resolve. Whether a *containment
    /// edge* agrees with the name is not asked here — that is ADR-0037's
    /// held enforcement (issue #333), which this sweep is the stated
    /// precondition of (ADR-0037:146-150, :217).
    ///
    /// # Errors
    ///
    /// A typed [`TopologyError`] naming the first rule violated; never a
    /// panic.
    pub fn build(nodes: Vec<NamingFields>, edges: Vec<Edge>) -> Result<Self, TopologyError> {
        let entries = absorb(nodes).map_err(TopologyError::Naming)?;
        let mut kind_of = std::collections::BTreeMap::new();
        for entry in &entries {
            let fields = match entry {
                NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
            };
            kind_of.insert(entry.id(), fields.kind_name());
        }
        // The naming sweep, before any edge is read: a name that points at
        // nothing is nonsense under every reading of every pair table, and
        // the edge set has no say in it either way.
        for entry in &entries {
            let fields = match entry {
                NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
            };
            for (field, referent) in fields.naming_referents() {
                let Some(referent_kind) = kind_of.get(&referent).copied() else {
                    return Err(TopologyError::UnresolvedNamingReferent {
                        node: entry.id(),
                        kind: fields.kind_name(),
                        field,
                        referent,
                    });
                };
                if !naming_referent_kind_allowed(fields.kind_name(), field, referent_kind) {
                    return Err(TopologyError::ForbiddenNamingReferent {
                        node: entry.id(),
                        kind: fields.kind_name(),
                        field,
                        referent,
                        referent_kind,
                    });
                }
            }
        }
        let mut seen = BTreeSet::new();
        let mut validated = edges;
        validated.sort_unstable();
        for edge in &validated {
            let source_kind = *kind_of
                .get(&edge.source)
                .ok_or(TopologyError::UnknownReferent { id: edge.source })?;
            let target_kind = *kind_of
                .get(&edge.target)
                .ok_or(TopologyError::UnknownReferent { id: edge.target })?;
            if edge.source == edge.target {
                return Err(TopologyError::SelfEdge { id: edge.source });
            }
            if !seen.insert(*edge) {
                return Err(TopologyError::DuplicateEdge { edge: *edge });
            }
            if !endpoint_pair_allowed(edge.kind, source_kind, target_kind) {
                return Err(TopologyError::ForbiddenEndpoint {
                    kind: edge.kind,
                    source_kind,
                    target_kind,
                });
            }
        }
        Ok(Self {
            entries,
            edges: validated,
        })
    }
}

/// A topology-construction failure — a typed artifact, per the register's
/// governing finding.
#[derive(Debug, PartialEq, Eq)]
pub enum TopologyError {
    /// Node absorption failed (see [`NamingError`]).
    Naming(NamingError),
    /// An edge references an address no absorbed entry carries.
    UnknownReferent {
        /// The unresolved address.
        id: NodeId,
    },
    /// A node's own naming field references an address no absorbed entry
    /// carries (issue #354). Resolve-only: the referent's *kind* is not
    /// examined here — see [`Topology::build`].
    UnresolvedNamingReferent {
        /// The node whose name carries the dangling referent.
        node: NodeId,
        /// That node's kind name.
        kind: &'static str,
        /// The naming field carrying it.
        field: &'static str,
        /// The unresolved address.
        referent: NodeId,
    },
    /// A node's own naming field references an absorbed entry of a kind
    /// the endpoint-pair table does not admit as the source of the
    /// relation the field names (issue #354's kind half; ADR-0045): a
    /// partition whose `parent_table` is the physical device, a volume
    /// whose `producer` is a partition. The pairing the name asserts is
    /// one no edge could carry, so no frame may be derived from it
    /// (ADR-0037:146-150).
    ForbiddenNamingReferent {
        /// The node whose name carries the referent.
        node: NodeId,
        /// That node's kind name.
        kind: &'static str,
        /// The naming field carrying it.
        field: &'static str,
        /// The referent's address.
        referent: NodeId,
        /// The kind the referent resolved to.
        referent_kind: &'static str,
    },
    /// An edge's endpoints are one node.
    SelfEdge {
        /// The address on both ends.
        id: NodeId,
    },
    /// The same edge appears twice; the edge set is a set.
    DuplicateEdge {
        /// The repeated edge.
        edge: Edge,
    },
    /// The endpoint kinds are not in the edge kind's pair table.
    ForbiddenEndpoint {
        /// The edge kind.
        kind: EdgeKind,
        /// The source entry's kind name.
        source_kind: &'static str,
        /// The target entry's kind name.
        target_kind: &'static str,
    },
}

impl fmt::Display for TopologyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Naming(error) => write!(formatter, "node absorption failed: {error}"),
            Self::UnknownReferent { id } => {
                write!(formatter, "edge references unknown address {id}")
            }
            Self::UnresolvedNamingReferent {
                node,
                kind,
                field,
                referent,
            } => write!(
                formatter,
                "{kind} {node} names unknown address {referent} in `{field}`"
            ),
            Self::ForbiddenNamingReferent {
                node,
                kind,
                field,
                referent,
                referent_kind,
            } => write!(
                formatter,
                "{kind} {node} names {referent_kind} {referent} in `{field}`, which no relation admits"
            ),
            Self::SelfEdge { id } => write!(formatter, "self-edge at {id}"),
            Self::DuplicateEdge { edge } => write!(formatter, "duplicate edge {edge:?}"),
            Self::ForbiddenEndpoint {
                kind,
                source_kind,
                target_kind,
            } => write!(
                formatter,
                "{kind:?} edge may not run {source_kind} -> {target_kind}"
            ),
        }
    }
}

impl std::error::Error for TopologyError {}

/// The endpoint-pair table: which (source kind, target kind) pairs each
/// edge kind admits.
///
/// The no-sibling-capture theorem's premise is a property of this table —
/// no `Backing`, `Production`, or `HostBacking` pair targets
/// `physical-device` — and the tests enumerate the table to prove it
/// rather than trusting this comment.
#[must_use]
pub fn endpoint_pair_allowed(
    kind: EdgeKind,
    source_kind: &'static str,
    target_kind: &'static str,
) -> bool {
    let pairs: &[(&str, &str)] = match kind {
        EdgeKind::Containment => &[
            ("physical-device", "partition-table"),
            ("partition-table", "partition"),
            ("partition-table", "conflicting-table-entry"),
            ("physical-device", "backing-signature"),
            ("physical-device", "file-system"),
            ("partition", "backing-signature"),
            ("partition", "file-system"),
            ("volume", "backing-signature"),
            ("volume", "file-system"),
            ("volume", "partition-table"),
            ("multipath-node", "backing-signature"),
            ("multipath-node", "file-system"),
            ("multipath-node", "partition-table"),
        ],
        EdgeKind::Backing => &[
            ("backing-signature", "aggregate"),
            ("backing-signature", "encryption-layer"),
        ],
        EdgeKind::Production => &[("encryption-layer", "volume"), ("aggregate", "volume")],
        EdgeKind::HostBacking => &[("backing-extent", "volume")],
        EdgeKind::PlatformMembership => &[("multipath-node", "physical-device")],
    };
    pairs.contains(&(source_kind, target_kind))
}

/// What a naming field's referent may be, read off the endpoint-pair table
/// (issue #354's kind half; ADR-0045).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferentRule {
    /// The referent names the *source* of an incoming edge of one of these
    /// kinds, with the field's owner as target; the admissible referent
    /// kinds are exactly the sources [`endpoint_pair_allowed`] pairs with
    /// the owner's kind under those edge kinds.
    Sources(&'static [EdgeKind]),
    /// No edge kind targets the owner from its referent, so the table has
    /// no opinion: the referent must resolve, and nothing more is asked.
    Open,
}

/// The rule for one naming field, keyed by the owner's kind name and the
/// field name [`NamingFields::naming_referents`] reports.
///
/// This is a map from field to *relation*, never a list of kinds: the
/// kinds come from the pair table at the moment of the check, so a row
/// added there admits the naming here in the same act, and a row absent
/// there refuses it. Every field the naming roster carries is classified;
/// the roster is a closed enum and
/// `the_naming_referent_rule_is_pinned_per_field` reds if a field arrives
/// unclassified — and an unclassified field admits **nothing** rather than
/// everything, so the failure is a refusal in the suite, not a silent gap.
#[must_use]
pub fn naming_referent_rule(owner_kind: &str, field: &str) -> ReferentRule {
    const CONTAINMENT: &[EdgeKind] = &[EdgeKind::Containment];
    const BACKING: &[EdgeKind] = &[EdgeKind::Backing];
    const PRODUCING: &[EdgeKind] = &[EdgeKind::Production, EdgeKind::HostBacking];
    const NONE: &[EdgeKind] = &[];
    match (owner_kind, field) {
        ("backing-extent", "host") => ReferentRule::Open,
        ("partition-table", "parent")
        | ("partition", "parent_table")
        | ("backing-signature" | "file-system", "host")
        | ("conflicting-table-entry", "table") => ReferentRule::Sources(CONTAINMENT),
        ("encryption-layer", "backing_signature") => ReferentRule::Sources(BACKING),
        ("volume", "producer") => ReferentRule::Sources(PRODUCING),
        _ => ReferentRule::Sources(NONE),
    }
}

/// Whether a naming field may reference a node of `referent_kind`: the
/// pair table's answer for the relation the field names.
#[must_use]
pub fn naming_referent_kind_allowed(
    owner_kind: &'static str,
    field: &'static str,
    referent_kind: &'static str,
) -> bool {
    match naming_referent_rule(owner_kind, field) {
        ReferentRule::Open => true,
        ReferentRule::Sources(kinds) => kinds
            .iter()
            .any(|kind| endpoint_pair_allowed(*kind, referent_kind, owner_kind)),
    }
}
