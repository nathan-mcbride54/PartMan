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
    /// (device → table → partition; a host carrying a signature or file
    /// system). "Inside one byte space" is a claim about the *frame*, not
    /// about the parent's span: a table's own extent is its header
    /// bytes, and the partitions it describes lie beside them, so the two
    /// `partition-table`-sourced pairs carry no span claim (ADR-0041) and
    /// what a destroyed table releases is decided by the naming relation,
    /// not by this edge (ADR-0043). The other seven pairs are geometric.
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
    /// The naming sweep is **resolve-only** and deliberately so. It
    /// refuses a referent that resolves to nothing; it does not ask what
    /// *kind* the referent resolves to. Deriving that kind check from
    /// [`endpoint_pair_allowed`] is the right shape and is held behind
    /// issue #360: that table lists the pairs the *edge* validator needs
    /// and is not a complete catalogue of what a naming field may
    /// legitimately reference, so deriving a mandatory check from it
    /// today promotes its omissions into refusals — measured to refuse a
    /// GPT inside a LUKS volume, a partitioned mdraid array, and an xfs
    /// on a dm-multipath node, all of which build. **This is therefore a
    /// partial discharge of ADR-0037:146-150 and does not close #354**,
    /// whose stated harm is the forbidden *pairing*.
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
                if !kind_of.contains_key(&referent) {
                    return Err(TopologyError::UnresolvedNamingReferent {
                        node: entry.id(),
                        kind: fields.kind_name(),
                        field,
                        referent,
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
