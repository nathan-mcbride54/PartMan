//! The simulated final topology (WP-060 increment 4): PLAN-002's second
//! half, built through the same fail-closed constructors as the capture
//! and stamped with the schema string that can never be a planning base.
//!
//! Simulation is **mandatory, not decorative**: PLAN-002 says every
//! valid plan produces both topologies, so an operation whose effects
//! this model cannot yet represent produces no valid plan at all —
//! [`SimulateRefusal::NotRepresentable`] refuses the request rather
//! than emitting a simulation that silently predicts "nothing changes"
//! about a change. What each operation honestly simulates today:
//!
//! - **Wipe**: every node the facts place on the target's bytes is
//!   gone, transitively with everything named relative to it, and the
//!   target's table-state stamp is dropped — the post-wipe state is
//!   not established until a real capture, and absence is the honest
//!   prediction (ADR-0014's stamp is the helper's; a `Simulated`
//!   snapshot predicts, and its schema string keeps the prediction
//!   from ever masquerading as establishment).
//! - **Create** (sized): a new partition minted under the host's one
//!   partition-table view, its extent the solver's placed range. A
//!   host with no table view, or with two (a hybrid), refuses typed —
//!   creating "somewhere" is not a prediction.
//! - **Grow / Shrink** (sized): the target's extent fact takes its new
//!   length.
//! - **Label / Uuid**: identity — this model carries no labels or
//!   mutable identifiers, so at this granularity the topology
//!   genuinely does not change, and identity is exact rather than
//!   lazy.
//! - **Everything else** — Move, Copy, Repair, Encrypt, Decrypt —
//!   refuses as not representable until its vocabulary arrives
//!   (encryption layers, copy destinations, and repair outcomes each
//!   need model surface this increment does not invent).

use partman_domain::model::naming::{NamingFields, NodeEntry, NodeId, derive_id};
use partman_domain::model::protection::{Facts, HostRange};
use partman_domain::model::snapshot::{SnapshotError, SnapshotKind, TopologySnapshot};
use partman_domain::model::topology::Edge;

/// Why simulation refused — and therefore why the plan is not valid
/// under PLAN-002's both-topologies rule.
#[derive(Debug, PartialEq, Eq)]
pub enum SimulateRefusal {
    /// The operation's effects are not representable in the current
    /// model, so no honest simulation exists and no valid plan does
    /// either. The vocabulary that represents it arrives under its own
    /// increment.
    NotRepresentable {
        /// A short name for the unrepresentable effect.
        effect: &'static str,
    },
    /// A sized create needs exactly one partition-table view under its
    /// host; none or two (a hybrid's second description) refuses,
    /// because creating "somewhere" is not a prediction.
    NoSingleTableView {
        /// The host.
        host: NodeId,
        /// How many table views the capture carries for it.
        views: usize,
    },
    /// Rebuilding the simulated snapshot refused — the same fail-closed
    /// construction the capture went through, verbatim.
    Assembly {
        /// The constructor's refusal, stringified (`SnapshotError` does
        /// not implement `PartialEq`).
        error: String,
    },
}

/// The changes one simulation applies, computed by the planning layer
/// that knows the request and solved geometry.
#[derive(Clone, Debug, Default)]
pub struct Effects {
    /// Nodes whose bytes are destroyed: everything the facts place on
    /// these ranges vanishes, transitively.
    pub destroyed: Vec<HostRange>,
    /// Targets whose table-state stamp drops (post-mutation state is
    /// unestablished until a real capture).
    pub stamp_dropped: Vec<NodeId>,
    /// A partition minted under the host's single table view.
    pub minted_partition: Option<HostRange>,
    /// Extent-length updates: (node, new length).
    pub resized: Vec<(NodeId, u64)>,
}

fn overlaps(range: &HostRange, other: &HostRange) -> bool {
    range.host == other.host
        && range.start < other.start + other.length
        && other.start < range.start + range.length
}

/// Destruction to a fixed point: everything the facts place on a
/// destroyed range, then everything named relative to a removed node —
/// read from `NamingFields::naming_referents`, the same roster
/// `Topology::build`'s naming sweep refuses a dangling referent from
/// (issue #354). Sharing the list is what makes "a swept capture stays
/// swept across a simulated rebuild" a theorem rather than a
/// coincidence: a referent kind this closure failed to follow would
/// survive here and then be refused by the rebuild it feeds. A node
/// whose own self-extent hosts the destroyed range survives —
/// wiping a device destroys its contents, not the device: the wiped
/// container remains, empty, which is exactly what the prediction
/// should say.
fn destroyed_closure(
    capture: &TopologySnapshot,
    effects: &Effects,
    nodes: &[(NodeId, NamingFields)],
) -> Vec<NodeId> {
    let mut removed: Vec<NodeId> = Vec::new();
    for (node, extent) in &capture.facts().extents {
        let is_self_extent_of_host = *node == extent.host;
        if !is_self_extent_of_host
            && effects
                .destroyed
                .iter()
                .any(|destroyed| overlaps(destroyed, extent))
            && !removed.contains(node)
        {
            removed.push(*node);
        }
    }
    loop {
        let before = removed.len();
        for (id, fields) in nodes {
            if !removed.contains(id)
                && fields
                    .naming_referents()
                    .iter()
                    .any(|(_, reference)| removed.contains(reference))
            {
                removed.push(*id);
            }
        }
        if removed.len() == before {
            return removed;
        }
    }
}

/// The facts that survive: removed nodes drop everything, dropped
/// stamps drop their table state, and resizes take effect.
fn surviving_facts(capture: &TopologySnapshot, effects: &Effects, removed: &[NodeId]) -> Facts {
    let mut facts = Facts::default();
    for (node, extent) in &capture.facts().extents {
        if !removed.contains(node) {
            facts.extents.insert(*node, *extent);
        }
    }
    for (node, transport) in &capture.facts().transports {
        if !removed.contains(node) {
            facts.transports.insert(*node, *transport);
        }
    }
    for (node, count) in &capture.facts().member_counts {
        if !removed.contains(node) {
            facts.member_counts.insert(*node, *count);
        }
    }
    for (node, state) in &capture.facts().table_states {
        if !removed.contains(node) && !effects.stamp_dropped.contains(node) {
            facts.table_states.insert(*node, state.clone());
        }
    }
    for (node, new_length) in &effects.resized {
        if let Some(extent) = facts.extents.get_mut(node) {
            extent.length = *new_length;
        }
    }
    facts
}

/// Apply the effects to the capture and assemble the simulated final
/// topology (PLAN-002's second half) through the real constructors.
///
/// # Errors
///
/// [`SimulateRefusal`], each variant explaining itself.
pub fn simulate(
    capture: &TopologySnapshot,
    effects: &Effects,
) -> Result<TopologySnapshot, SimulateRefusal> {
    // The capture's entries, expanded: a collision group re-expands to
    // its member count so re-assembly re-absorbs it identically.
    let mut nodes: Vec<(NodeId, NamingFields)> = Vec::new();
    for entry in capture.topology().entries() {
        match entry {
            NodeEntry::Single { fields, .. } => nodes.push((entry.id(), fields.clone())),
            NodeEntry::Group { fields, count, .. } => {
                for _ in 0..*count {
                    nodes.push((entry.id(), fields.clone()));
                }
            }
        }
    }

    let removed = destroyed_closure(capture, effects, &nodes);

    // The minted partition, under the host's single table view.
    let mut minted: Option<(NamingFields, NodeId, HostRange)> = None;
    if let Some(placed) = &effects.minted_partition {
        let views: Vec<NodeId> = nodes
            .iter()
            .filter(|(id, fields)| {
                !removed.contains(id)
                    && matches!(fields, NamingFields::PartitionTable { parent, .. } if *parent == placed.host)
            })
            .map(|(id, _)| *id)
            .collect();
        if views.len() != 1 {
            return Err(SimulateRefusal::NoSingleTableView {
                host: placed.host,
                views: views.len(),
            });
        }
        let fields = NamingFields::Partition {
            parent_table: views[0],
            start_offset: placed.start,
        };
        minted = Some((fields, views[0], *placed));
    }

    // Rebuild the node, edge, and fact sets.
    let mut simulated_nodes: Vec<NamingFields> = nodes
        .iter()
        .filter(|(id, _)| !removed.contains(id))
        .map(|(_, fields)| fields.clone())
        .collect();
    let mut simulated_edges: Vec<Edge> = capture
        .topology()
        .edges()
        .iter()
        .filter(|edge| !removed.contains(&edge.source) && !removed.contains(&edge.target))
        .copied()
        .collect();

    let mut facts = surviving_facts(capture, effects, &removed);
    if let Some((fields, table, placed)) = minted {
        let new_id = derive_id(&fields).map_err(|error| SimulateRefusal::Assembly {
            error: format!("minted partition must derive an address: {error:?}"),
        })?;
        facts.extents.insert(new_id, placed);
        simulated_edges.push(Edge {
            kind: partman_domain::model::topology::EdgeKind::Containment,
            source: table,
            target: new_id,
        });
        simulated_nodes.push(fields);
    }

    TopologySnapshot::assemble(
        SnapshotKind::Simulated,
        false,
        simulated_nodes,
        simulated_edges,
        facts,
    )
    .map_err(|error: SnapshotError| SimulateRefusal::Assembly {
        error: format!("{error:?}"),
    })
}
