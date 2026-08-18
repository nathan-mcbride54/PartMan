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
//! - **Move** (sized, ADR-0052): the target and every node named within
//!   it re-derive their addresses at the destination — a moved partition
//!   renames (ADR-0019), so its content renames with it — with every
//!   extent framed on the host translated by the move's offset; the
//!   whole source range is destroyed for everything the move does not
//!   carry, release being destruction (ADR-0018). Nothing the
//!   destination overlaps survives that the solver did not already
//!   refuse.
//! - **Everything else** — unsized Move, Copy, Repair, Encrypt, Decrypt
//!   — refuses as not representable until its vocabulary arrives
//!   (encryption layers, copy destinations, and repair outcomes each
//!   need model surface this increment does not invent).

use partman_domain::model::naming::{NamingFields, NodeEntry, NodeId, derive_id};
use partman_domain::model::protection::{Facts, HostRange, names_within};
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
    /// A relocation of one target and everything named within it
    /// (ADR-0052). The moved set is exempt from `destroyed`, which for a
    /// move carries the whole source range.
    pub relocated: Option<Relocation>,
}

/// One relocation: the target moves from `source` to `destination` on
/// the same host, carrying every node named within it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relocation {
    /// The moved node.
    pub target: NodeId,
    /// Its whole pre-move extent.
    pub source: HostRange,
    /// Its whole post-move extent.
    pub destination: HostRange,
}

/// The nodes a relocation carries: the target and everything whose own
/// name positions it inside the target, at any depth — read off the
/// naming relation through the domain's own predicate, never the edge
/// set.
fn carried_by(capture: &TopologySnapshot, relocation: &Relocation) -> Vec<NodeId> {
    let topology = capture.topology();
    let mut carried = vec![relocation.target];
    for entry in topology.entries() {
        let id = entry.id();
        if id != relocation.target && names_within(topology, id, relocation.target) {
            carried.push(id);
        }
    }
    carried
}

/// A node's naming fields with every referent in `map` replaced — the
/// spelling of "renames with its host". Exhaustive over the variants that
/// carry a referent, so a new referent-bearing kind stops this compiling
/// rather than silently keeping a stale address.
fn retarget(fields: &NamingFields, map: &[(NodeId, NodeId)]) -> NamingFields {
    let swap = |id: &NodeId| {
        map.iter()
            .find(|(old, _)| old == id)
            .map_or(*id, |(_, new)| *new)
    };
    match fields {
        NamingFields::PhysicalDevice { .. }
        | NamingFields::Aggregate { .. }
        | NamingFields::MultipathNode { .. } => fields.clone(),
        NamingFields::PartitionTable { parent, role } => NamingFields::PartitionTable {
            parent: swap(parent),
            role: role.clone(),
        },
        NamingFields::Partition {
            parent_table,
            start_offset,
        } => NamingFields::Partition {
            parent_table: swap(parent_table),
            start_offset: *start_offset,
        },
        NamingFields::BackingSignature {
            host,
            family,
            primary_offset,
        } => NamingFields::BackingSignature {
            host: swap(host),
            family: family.clone(),
            primary_offset: *primary_offset,
        },
        NamingFields::FileSystem {
            host,
            kind,
            superblock_offset,
        } => NamingFields::FileSystem {
            host: swap(host),
            kind: kind.clone(),
            superblock_offset: *superblock_offset,
        },
        NamingFields::EncryptionLayer { backing_signature } => NamingFields::EncryptionLayer {
            backing_signature: swap(backing_signature),
        },
        NamingFields::Volume {
            producer,
            name,
            role,
        } => NamingFields::Volume {
            producer: swap(producer),
            name: name.clone(),
            role: role.clone(),
        },
        NamingFields::BackingExtent { host, locator } => NamingFields::BackingExtent {
            host: swap(host),
            locator: locator.clone(),
        },
        NamingFields::ConflictingTableEntry {
            table,
            view_role,
            entry_start,
        } => NamingFields::ConflictingTableEntry {
            table: swap(table),
            view_role: view_role.clone(),
            entry_start: *entry_start,
        },
    }
}

/// The relocation applied to the node set: the moved partition takes its
/// destination start, every carried node re-derives its address through
/// the renamed referents to a fixpoint, and every carried extent framed
/// on the host is translated by the move's offset. Returns the old→new
/// address map, or the derivation error.
fn relocate(
    capture: &TopologySnapshot,
    relocation: &Relocation,
    nodes: &mut [(NodeId, NamingFields)],
    facts: &mut Facts,
) -> Result<Vec<(NodeId, NodeId)>, SimulateRefusal> {
    let carried = carried_by(capture, relocation);
    let derive = |fields: &NamingFields| {
        derive_id(fields).map_err(|error| SimulateRefusal::Assembly {
            error: format!("relocated node must derive an address: {error:?}"),
        })
    };

    // The target itself: a partition renames by its start offset. Any
    // other kind reaching here has no positional field to move, which is
    // a refusal, not a silent no-op — the solver admits only extent-
    // bearing targets, and the only extent-bearing kind whose address is
    // its position is the partition.
    let mut map: Vec<(NodeId, NodeId)> = Vec::new();
    for (id, fields) in nodes.iter_mut() {
        if *id != relocation.target {
            continue;
        }
        let NamingFields::Partition { parent_table, .. } = fields else {
            return Err(SimulateRefusal::NotRepresentable {
                effect: "only a partition's address is its position; moving another kind renames nothing",
            });
        };
        let renamed = NamingFields::Partition {
            parent_table: *parent_table,
            start_offset: relocation.destination.start,
        };
        let new_id = derive(&renamed)?;
        map.push((*id, new_id));
        *fields = renamed;
        *id = new_id;
    }

    // Everything named within the target, to a fixpoint: a child renames
    // once its referent has, and its own dependents follow.
    loop {
        let before = map.len();
        for (id, fields) in nodes.iter_mut() {
            if !carried.contains(id) || map.iter().any(|(old, _)| old == id) {
                continue;
            }
            let renamed = retarget(fields, &map);
            if renamed == *fields {
                continue;
            }
            let new_id = derive(&renamed)?;
            map.push((*id, new_id));
            *fields = renamed;
            *id = new_id;
        }
        if map.len() == before {
            break;
        }
    }

    // Facts: carried extents framed on the host translate; the target's
    // own becomes the destination exactly. Everything else keyed by a
    // renamed node re-keys.
    let host = relocation.source.host;
    let mut translated = Facts::default();
    let rename = |id: &NodeId| {
        map.iter()
            .find(|(old, _)| old == id)
            .map_or(*id, |(_, new)| *new)
    };
    for (node, extent) in &facts.extents {
        let mut extent = *extent;
        if *node == relocation.target {
            extent = relocation.destination;
        } else if carried.contains(node) && extent.host == host {
            let end = extent.start + extent.length;
            if relocation.destination.start >= relocation.source.start {
                extent.start += relocation.destination.start - relocation.source.start;
            } else {
                extent.start -= relocation.source.start - relocation.destination.start;
            }
            debug_assert!(
                end >= relocation.source.start,
                "carried extent lies in the source"
            );
        }
        translated.extents.insert(rename(node), extent);
    }
    for (node, transport) in &facts.transports {
        translated.transports.insert(rename(node), *transport);
    }
    for (node, count) in &facts.member_counts {
        translated.member_counts.insert(rename(node), *count);
    }
    for (node, state) in &facts.table_states {
        translated.table_states.insert(rename(node), state.clone());
    }
    *facts = translated;
    Ok(map)
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
    // A relocation's carried set is exempt from the destroyed sweep: the
    // move declares its whole source destroyed (ADR-0052 D2, so the
    // closure reaches everything there), and what it carries is re-minted
    // at the destination rather than removed.
    let carried: Vec<NodeId> = effects
        .relocated
        .as_ref()
        .map_or_else(Vec::new, |relocation| carried_by(capture, relocation));
    let mut removed: Vec<NodeId> = Vec::new();
    for (node, extent) in &capture.facts().extents {
        let is_self_extent_of_host = *node == extent.host;
        if !is_self_extent_of_host
            && !carried.contains(node)
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
    let mut facts = surviving_facts(capture, effects, &removed);
    let mut surviving: Vec<(NodeId, NamingFields)> = nodes
        .iter()
        .filter(|(id, _)| !removed.contains(id))
        .cloned()
        .collect();
    let mut simulated_edges: Vec<Edge> = capture
        .topology()
        .edges()
        .iter()
        .filter(|edge| !removed.contains(&edge.source) && !removed.contains(&edge.target))
        .copied()
        .collect();

    // The relocation: renamed nodes, translated facts, re-pointed edges.
    if let Some(relocation) = &effects.relocated {
        let map = relocate(capture, relocation, &mut surviving, &mut facts)?;
        let rename = |id: NodeId| {
            map.iter()
                .find(|(old, _)| *old == id)
                .map_or(id, |(_, new)| *new)
        };
        for edge in &mut simulated_edges {
            edge.source = rename(edge.source);
            edge.target = rename(edge.target);
        }
    }
    let mut simulated_nodes: Vec<NamingFields> =
        surviving.into_iter().map(|(_, fields)| fields).collect();
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
