//! The extent solver (WP-060 increment 3), alignment-conservative,
//! carrying ADR-0023's authored/inherited distinction (increment 5).
//!
//! Free space is computed from the snapshot's authenticated extents and
//! nothing else: a host's free ranges are its own extent minus the
//! extents of the nodes the facts place on it. Where the facts carry a
//! table region as a child extent, placement avoids it; where they do
//! not, the solver does not invent one — the math is over what the body
//! authenticates, exactly like every other consumer of the facts.
//!
//! Placement is PART-009's default and only PART-009's default:
//! first-fit at the lowest 1 MiB-aligned start that fits. The two
//! permitted deviation causes — published geometry, explicit user
//! override — have no input vocabulary yet, so deviation is
//! inexpressible here rather than half-supported: each arrives with the
//! vocabulary that carries it, recorded in the plan as PART-009
//! requires, which is a body change under WP-010's grant.
//!
//! **A deviation is an act, not a state** (ADR-0023, resolving SI-15 in
//! spec 12.1.0). The solver judges only boundaries the plan authors: a
//! create's start and end, a grow's new end, a shrink's new end. A
//! pre-existing boundary the plan does not move — a legacy misaligned
//! start grown at its tail — is an **inherited fact**, byte-identical
//! before and after, reported as a typed [`InheritedFact`] for the
//! plan's consequence text and demanding no override. Every authored
//! boundary meets the default, is coincident with a pre-existing
//! structural edge (a neighbor's start, the host's end) and recorded as
//! such, or would need a deviation cause this vocabulary deliberately
//! cannot express — so it refuses typed instead. There is no fourth
//! state, held by test.

use partman_domain::model::naming::{NamingFields, NodeEntry, NodeId, TableRole};
use partman_domain::model::protection::HostRange;
use partman_domain::model::snapshot::TopologySnapshot;

/// PART-009's default alignment: 1 MiB.
pub const DEFAULT_ALIGNMENT: u64 = 1 << 20;

/// The pre-existing structural edge an authored boundary can lawfully
/// coincide with (ADR-0023's coincident-edge rule): placing exactly at
/// such an edge conforms to policy — aligning away from it instead
/// would mint an unusable sliver — and is recorded as coincident.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralEdge {
    /// A neighboring child's own start.
    NeighborStart {
        /// The neighbor whose start the boundary meets.
        neighbor: NodeId,
    },
    /// The host's own extent end.
    HostEnd,
    /// The low boundary of the region a partition table's own scheme
    /// claims at the host's tail: pre-existing, fixed by the scheme
    /// rather than authored by the plan.
    ReservedTableRegion {
        /// The table view whose scheme claims the region.
        table: NodeId,
    },
}

/// How an authored boundary satisfies PART-009: aligned to the default,
/// or coincident with a named pre-existing structural edge. The two
/// recorded deviation causes have no vocabulary here, and a boundary
/// that is neither refuses — so this type is the no-fourth-state
/// property, spelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryPlacement {
    /// The boundary sits on the 1 MiB default.
    Aligned,
    /// The boundary coincides with a pre-existing structural edge and
    /// is recorded as coincident (ADR-0023).
    Coincident {
        /// The edge coincided with.
        edge: StructuralEdge,
    },
}

/// A pre-existing off-policy boundary the plan does not move: an
/// inherited fact about the device, never a deviation and never a grant
/// by the user (ADR-0023). Carried out of the solver so the plan's
/// consequence text can state it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InheritedFact {
    /// The target's start predates the plan and is not on the 1 MiB
    /// default; the plan leaves it byte-identical.
    MisalignedStart {
        /// The target whose start is inherited.
        target: NodeId,
        /// The inherited start offset.
        start: u64,
    },
}

/// A solved create: the placed range plus how each authored boundary
/// satisfies policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolvedCreate {
    /// The placed range.
    pub placed: HostRange,
    /// The authored end's placement (the start is first-fit aligned by
    /// construction).
    pub end_placement: BoundaryPlacement,
}

/// A solved grow: the tail extension, the authored end's placement, and
/// the inherited start fact where the start predates policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolvedGrow {
    /// The extension range appended at the target's tail.
    pub extension: HostRange,
    /// The authored new end's placement.
    pub end_placement: BoundaryPlacement,
    /// The untouched misaligned start, where there is one.
    pub inherited_start: Option<InheritedFact>,
}

/// A solved shrink: the freed tail, the authored end's placement, and
/// the inherited start fact where the start predates policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolvedShrink {
    /// The freed tail range (destroyed: bytes beyond the new end are
    /// gone).
    pub freed: HostRange,
    /// The authored new end's placement.
    pub end_placement: BoundaryPlacement,
    /// The untouched misaligned start, where there is one.
    pub inherited_start: Option<InheritedFact>,
}

/// Why the solver refused — each variant explaining itself with the
/// numbers it judged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolveRefusal {
    /// The named host carries no authenticated extent, so free space is
    /// not computable from the body's facts.
    HostHasNoExtent {
        /// The extent-less host.
        host: NodeId,
    },
    /// The named target carries no authenticated extent, so its resize
    /// geometry is not computable.
    TargetHasNoExtent {
        /// The extent-less target.
        target: NodeId,
    },
    /// No free range fits the aligned request.
    NoFitForSize {
        /// The size requested.
        requested: u64,
        /// The largest free range available, aligned — zero when none.
        largest_aligned_fit: u64,
    },
    /// The request authors a boundary that is neither on the 1 MiB
    /// default nor coincident with a pre-existing structural edge, and
    /// PART-009's two deviation causes have no vocabulary here — so
    /// there is no lawful spelling for it (ADR-0023's no-fourth-state
    /// rule). The refusal names the nearest conforming values so the
    /// caller can explain what would have succeeded.
    UnalignedAuthoredBoundary {
        /// The target whose boundary the request authors.
        target: NodeId,
        /// The offending authored offset.
        boundary: u64,
        /// The nearest aligned offset below the request, zero when none
        /// exists above the range's floor.
        nearest_aligned_below: u64,
        /// The coincident candidate — the structural edge bounding the
        /// range the boundary lives in — zero when none applies.
        coincident_candidate: u64,
    },
    /// Growth needs contiguous free space immediately after the
    /// target's extent, and it is not there.
    NoAdjacentFreeSpace {
        /// The target being grown.
        target: NodeId,
        /// The extension length needed.
        needed: u64,
        /// The contiguous free length actually available at the tail.
        available: u64,
    },
    /// A resize to a length that is not a resize: zero, or not smaller
    /// (shrink), or not larger (grow) than the current length.
    NotAResize {
        /// The target.
        target: NodeId,
        /// Its current length.
        current: u64,
        /// The requested length.
        requested: u64,
    },
    /// The host's own extent reaches past the device its own naming
    /// fields size, so the arithmetic's outer bound is not the device.
    HostExtentExceedsDevice {
        /// The host whose extent overruns it.
        host: NodeId,
        /// Where the host's extent ends.
        extent_end: u64,
        /// The total size the host's own hashed name declares.
        total_bytes: u64,
    },
    /// A node the facts place on this host carries a range that leaves
    /// the host's own extent, so the subtraction's cursor walk would
    /// report space the host does not have.
    ChildExtentOutsideHost {
        /// The host whose free space was asked for.
        host: NodeId,
        /// The node whose range leaves it.
        node: NodeId,
        /// The range's start.
        start: u64,
        /// The range's length.
        length: u64,
    },
    /// The host declares a table view in a scheme this build cannot
    /// name, so the regions that scheme claims are not derivable and
    /// free space is not computable.
    UnrecognizedTableScheme {
        /// The host whose free space was asked for.
        host: NodeId,
        /// The node declaring the scheme: a table view, or a
        /// conflicting entry recording one.
        view: NodeId,
        /// The reporting interface's raw discriminant bytes, verbatim.
        raw: Vec<u8>,
    },
    /// A partition the authenticated naming fields place on this host
    /// is not one the subtraction removes, so the bytes its own hashed
    /// name declares would be handed out as free.
    UnaccountedOccupant {
        /// The host whose free space was asked for.
        host: NodeId,
        /// The occupant nothing accounted for.
        occupant: NodeId,
        /// The start offset the occupant's own hashed name declares.
        declared_start: u64,
        /// What the facts carry instead.
        ground: OccupancyGround,
    },
}

/// Why a declared occupant is not one the subtraction removes — the
/// value the facts carry, beside the name that declared it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OccupancyGround {
    /// The facts place no range for it at all.
    NoRange,
    /// A range exists, in another host's address space.
    RangeOnAnotherHost {
        /// The host the range names.
        host: NodeId,
    },
    /// A range exists on this host and is empty, so it removes nothing.
    RangeIsEmpty,
    /// A range exists on this host and does not begin where the
    /// occupant's own hashed name declares.
    RangeStartsElsewhere {
        /// Where the range actually begins.
        start: u64,
    },
    /// The occupant is located on this host under a table view this
    /// host does not carry, so no scheme of this host's accounts for it.
    TableIsNotThisHosts {
        /// The table address the occupant's own name declares.
        named_table: NodeId,
    },
}

/// The regions a partition-table scheme claims in its host's address
/// space: a **bound, never a measurement**. No sector size reaches this
/// module and a `PartitionTable` node carries no geometry, so each
/// figure is the smallest bound expressible in this module's only unit
/// that covers the scheme's structures at every sector size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchemeReservation {
    /// Bytes withheld at the host's low end.
    pub head: u64,
    /// Bytes withheld at the host's high end.
    pub tail: u64,
    /// The view that fixed the tail bound — least address among those
    /// claiming the maximum — or `None` where no view claims a tail.
    pub tail_claimed_by: Option<NodeId>,
}

fn fields_of(entry: &NodeEntry) -> &NamingFields {
    match entry {
        NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
    }
}

fn host_tables(snapshot: &TopologySnapshot, host: NodeId) -> Vec<NodeId> {
    snapshot
        .topology()
        .entries()
        .iter()
        .filter(|entry| {
            matches!(
                fields_of(entry),
                NamingFields::PartitionTable { parent, .. } if *parent == host
            )
        })
        .map(NodeEntry::id)
        .collect()
}

/// The union — per-end maximum — of the regions every table view the
/// host declares claims, read from the authenticated naming fields and
/// never from containment edges.
///
/// # Errors
///
/// [`SolveRefusal::UnrecognizedTableScheme`] where a view carries a
/// scheme this build cannot name, so no bound over it is derivable.
pub fn reserved_regions(
    snapshot: &TopologySnapshot,
    host: NodeId,
) -> Result<SchemeReservation, SolveRefusal> {
    let tables = host_tables(snapshot, host);
    let mut reservation = SchemeReservation {
        head: 0,
        tail: 0,
        tail_claimed_by: None,
    };
    for entry in snapshot.topology().entries() {
        let (view, role) = match fields_of(entry) {
            NamingFields::PartitionTable { parent, role } if *parent == host => (entry.id(), role),
            NamingFields::ConflictingTableEntry {
                table, view_role, ..
            } if tables.contains(table) => (entry.id(), view_role),
            _ => continue,
        };
        let (head, tail) = match role {
            TableRole::Gpt | TableRole::HybridMbr => (DEFAULT_ALIGNMENT, DEFAULT_ALIGNMENT),
            TableRole::Mbr | TableRole::Apm => (DEFAULT_ALIGNMENT, 0),
            TableRole::Unrecognized { raw } => {
                return Err(SolveRefusal::UnrecognizedTableScheme {
                    host,
                    view,
                    raw: raw.clone(),
                });
            }
        };
        reservation.head = reservation.head.max(head);
        if tail > reservation.tail {
            reservation.tail = tail;
            reservation.tail_claimed_by = Some(view);
        }
    }
    Ok(reservation)
}

/// The first occupant of `host` the subtraction does not remove, as the
/// refusal it produces.
fn unaccounted_occupant(
    snapshot: &TopologySnapshot,
    host: NodeId,
    tables: &[NodeId],
) -> Option<SolveRefusal> {
    for entry in snapshot.topology().entries() {
        let NamingFields::Partition {
            parent_table,
            start_offset,
        } = fields_of(entry)
        else {
            continue;
        };
        let located = snapshot.facts().extents.get(&entry.id()).copied();
        if let Some(ground) = occupant_ground(located, host, *start_offset, *parent_table, tables) {
            return Some(SolveRefusal::UnaccountedOccupant {
                host,
                occupant: entry.id(),
                declared_start: *start_offset,
                ground,
            });
        }
    }
    None
}

/// Why a partition naming `named_table` and `declared_start` is an
/// occupant of `host` the subtraction does not remove — or `None` where
/// it is accounted for, or is no occupant of this host at all: a
/// partition under a table this host does not carry, located elsewhere
/// or nowhere, is another host's matter. One located on this host under
/// such a table is refused as [`OccupancyGround::TableIsNotThisHosts`];
/// the rest is [`occupancy_ground`]. A function of the range and the
/// names alone, for the same reason that helper is: the body boundary
/// refuses some of these shapes before a snapshot can carry them
/// (ADR-0041; ADR-0037's frame rule once enforced makes this arm's shape
/// one of them), and the solver's own reading must not depend on which
/// shapes reach it.
pub(crate) fn occupant_ground(
    located: Option<HostRange>,
    host: NodeId,
    declared_start: u64,
    named_table: NodeId,
    host_tables: &[NodeId],
) -> Option<OccupancyGround> {
    if !host_tables.contains(&named_table) {
        return located
            .filter(|range| range.host == host)
            .map(|_| OccupancyGround::TableIsNotThisHosts { named_table });
    }
    occupancy_ground(located, host, declared_start)
}

/// Why a located range does not account for the occupant whose hashed
/// name declares `declared_start` on `host` — or `None` where it does.
/// A function of the range alone, so each ground is testable without a
/// snapshot: the body boundary refuses some of these shapes before a
/// snapshot can carry them (ADR-0041), and the solver's own reading of a
/// range must not depend on which shapes reach it.
pub(crate) fn occupancy_ground(
    located: Option<HostRange>,
    host: NodeId,
    declared_start: u64,
) -> Option<OccupancyGround> {
    match located {
        None => Some(OccupancyGround::NoRange),
        Some(range) if range.host != host => {
            Some(OccupancyGround::RangeOnAnotherHost { host: range.host })
        }
        Some(range) if range.length == 0 => Some(OccupancyGround::RangeIsEmpty),
        Some(range) if range.start != declared_start => {
            Some(OccupancyGround::RangeStartsElsewhere { start: range.start })
        }
        Some(_) => None,
    }
}

fn extent_of(snapshot: &TopologySnapshot, node: NodeId) -> Option<HostRange> {
    snapshot.facts().extents.get(&node).copied()
}

fn align_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

/// The structural edge at `offset` on `host`: a neighboring child whose
/// extent starts exactly there, or the host's own extent end. `None`
/// when nothing pre-existing sits at that offset.
fn structural_edge_at(
    snapshot: &TopologySnapshot,
    host: NodeId,
    offset: u64,
    ceiling: Option<(NodeId, u64)>,
) -> Option<StructuralEdge> {
    if let Some((neighbor, _)) = snapshot
        .facts()
        .extents
        .iter()
        .find(|(node, range)| **node != host && range.host == host && range.start == offset)
    {
        return Some(StructuralEdge::NeighborStart {
            neighbor: *neighbor,
        });
    }
    if let Some((table, at)) = ceiling
        && at == offset
    {
        return Some(StructuralEdge::ReservedTableRegion { table });
    }
    let own = extent_of(snapshot, host)?;
    (own.start + own.length == offset).then_some(StructuralEdge::HostEnd)
}

/// Judge one authored end against PART-009: on the default, coincident
/// with the structural edge bounding its free room, or refused typed —
/// no fourth state.
fn authored_end_placement(
    snapshot: &TopologySnapshot,
    host: NodeId,
    target: NodeId,
    end: u64,
    room_end: u64,
    ceiling: Option<(NodeId, u64)>,
) -> Result<BoundaryPlacement, SolveRefusal> {
    if end.is_multiple_of(DEFAULT_ALIGNMENT) {
        return Ok(BoundaryPlacement::Aligned);
    }
    if end == room_end
        && let Some(edge) = structural_edge_at(snapshot, host, end, ceiling)
    {
        return Ok(BoundaryPlacement::Coincident { edge });
    }
    Err(SolveRefusal::UnalignedAuthoredBoundary {
        target,
        boundary: end,
        nearest_aligned_below: (end / DEFAULT_ALIGNMENT) * DEFAULT_ALIGNMENT,
        coincident_candidate: room_end,
    })
}

/// A host's solved geometry: the regions its declared schemes claim,
/// and the free ranges left once those and its children are withheld.
#[derive(Clone, Debug, PartialEq, Eq)]
struct HostGeometry {
    reserved: SchemeReservation,
    ceiling: Option<(NodeId, u64)>,
    free: Vec<HostRange>,
}

fn host_extent_exceeds_device(
    snapshot: &TopologySnapshot,
    host: NodeId,
    own: HostRange,
) -> Option<SolveRefusal> {
    let entry = snapshot
        .topology()
        .entries()
        .iter()
        .find(|entry| entry.id() == host)?;
    let NamingFields::PhysicalDevice { total_bytes, .. } = fields_of(entry) else {
        return None;
    };
    let extent_end = own.start.saturating_add(own.length);
    (extent_end > *total_bytes).then_some(SolveRefusal::HostExtentExceedsDevice {
        host,
        extent_end,
        total_bytes: *total_bytes,
    })
}

fn child_extent_outside_host(
    snapshot: &TopologySnapshot,
    host: NodeId,
    own: HostRange,
) -> Option<SolveRefusal> {
    let end = own.start.saturating_add(own.length);
    snapshot
        .facts()
        .extents
        .iter()
        .find(|(node, range)| {
            **node != host && range.host == host && range.start.saturating_add(range.length) > end
        })
        .map(|(node, range)| SolveRefusal::ChildExtentOutsideHost {
            host,
            node: *node,
            start: range.start,
            length: range.length,
        })
}

fn host_geometry(snapshot: &TopologySnapshot, host: NodeId) -> Result<HostGeometry, SolveRefusal> {
    let own = extent_of(snapshot, host).ok_or(SolveRefusal::HostHasNoExtent { host })?;
    if let Some(refusal) = host_extent_exceeds_device(snapshot, host, own) {
        return Err(refusal);
    }
    let reserved = reserved_regions(snapshot, host)?;
    if let Some(refusal) = child_extent_outside_host(snapshot, host, own) {
        return Err(refusal);
    }
    let tables = host_tables(snapshot, host);
    if let Some(refusal) = unaccounted_occupant(snapshot, host, &tables) {
        return Err(refusal);
    }

    let end = own.start.saturating_add(own.length);
    let floor = own.start.saturating_add(reserved.head).min(end);
    let ceiling_at = end.saturating_sub(reserved.tail).max(own.start);
    let ceiling = reserved
        .tail_claimed_by
        .filter(|_| reserved.tail > 0)
        .map(|table| (table, ceiling_at));

    let mut children: Vec<(u64, u64)> = snapshot
        .facts()
        .extents
        .iter()
        .filter(|(node, range)| **node != host && range.host == host)
        .map(|(_, range)| (range.start, range.length))
        .collect();
    children.sort_unstable();

    let mut raw: Vec<(u64, u64)> = Vec::new();
    let mut cursor = own.start;
    for (start, length) in children {
        if start > cursor {
            raw.push((cursor, start - cursor));
        }
        cursor = cursor.max(start + length);
    }
    if cursor < end {
        raw.push((cursor, end - cursor));
    }

    let mut free = Vec::new();
    for (start, length) in raw {
        let clipped_start = start.max(floor);
        let clipped_end = (start + length).min(ceiling_at);
        if clipped_end > clipped_start {
            free.push(HostRange {
                host,
                start: clipped_start,
                length: clipped_end - clipped_start,
            });
        }
    }
    Ok(HostGeometry {
        reserved,
        ceiling,
        free,
    })
}

/// The host's free ranges: its own extent, minus the regions its
/// declared table schemes claim, minus the extents the facts place on
/// it, ascending, coalesced by construction.
///
/// # Errors
///
/// [`SolveRefusal::HostHasNoExtent`] if the host carries no extent, and
/// every refusal [`reserved_regions`] and the occupancy check produce.
pub fn free_extents(
    snapshot: &TopologySnapshot,
    host: NodeId,
) -> Result<Vec<HostRange>, SolveRefusal> {
    host_geometry(snapshot, host).map(|geometry| geometry.free)
}

/// First-fit placement for a create: the lowest 1 MiB-aligned start
/// whose free range holds the full size, with the authored end judged
/// by the same policy — aligned, coincident with the range's bounding
/// edge, or refused.
///
/// # Errors
///
/// [`SolveRefusal`], the no-fit case naming the largest aligned fit so
/// the caller can explain what would have succeeded.
pub fn place_create(
    snapshot: &TopologySnapshot,
    host: NodeId,
    size: u64,
) -> Result<SolvedCreate, SolveRefusal> {
    if size == 0 {
        return Err(SolveRefusal::NotAResize {
            target: host,
            current: 0,
            requested: 0,
        });
    }
    let geometry = host_geometry(snapshot, host)?;
    let free = geometry.free;
    let mut largest_aligned_fit = 0_u64;
    let mut first_nonconforming: Option<SolveRefusal> = None;
    for range in &free {
        let aligned_start = align_up(range.start, DEFAULT_ALIGNMENT);
        let range_end = range.start + range.length;
        if aligned_start >= range_end {
            continue;
        }
        let usable = range_end - aligned_start;
        largest_aligned_fit = largest_aligned_fit.max(usable);
        if usable >= size {
            match authored_end_placement(
                snapshot,
                host,
                host,
                aligned_start + size,
                range_end,
                geometry.ceiling,
            ) {
                Ok(end_placement) => {
                    return Ok(SolvedCreate {
                        placed: HostRange {
                            host,
                            start: aligned_start,
                            length: size,
                        },
                        end_placement,
                    });
                }
                // A later range may still conform — an off-default size
                // is legal exactly where it fills its room to a
                // structural edge — so the scan continues and the first
                // refusal is only reported when no range conforms.
                Err(refusal) => {
                    first_nonconforming.get_or_insert(refusal);
                }
            }
        }
    }
    Err(first_nonconforming.unwrap_or(SolveRefusal::NoFitForSize {
        requested: size,
        largest_aligned_fit,
    }))
}

fn inherited_start(target: NodeId, start: u64) -> Option<InheritedFact> {
    (!start.is_multiple_of(DEFAULT_ALIGNMENT))
        .then_some(InheritedFact::MisalignedStart { target, start })
}

/// The extension range for growing a target to `new_length` at its
/// tail. The start is never touched: a misaligned start is an inherited
/// fact carried for the consequence text, not a deviation and not a
/// refusal (ADR-0023). The authored new end meets the default, is
/// coincident with the adjacent room's bounding edge (grow-to-fill), or
/// refuses.
///
/// # Errors
///
/// [`SolveRefusal`], each variant carrying the numbers it judged.
pub fn grow_extension(
    snapshot: &TopologySnapshot,
    target: NodeId,
    new_length: u64,
) -> Result<SolvedGrow, SolveRefusal> {
    let own = extent_of(snapshot, target).ok_or(SolveRefusal::TargetHasNoExtent { target })?;
    if new_length <= own.length {
        return Err(SolveRefusal::NotAResize {
            target,
            current: own.length,
            requested: new_length,
        });
    }
    let needed = new_length - own.length;
    let tail = own.start + own.length;
    let geometry = host_geometry(snapshot, own.host)?;
    let available = geometry
        .free
        .iter()
        .find(|range| range.start == tail)
        .map_or(0, |range| range.length);
    if available < needed {
        return Err(SolveRefusal::NoAdjacentFreeSpace {
            target,
            needed,
            available,
        });
    }
    let end_placement = authored_end_placement(
        snapshot,
        own.host,
        target,
        own.start + new_length,
        tail + available,
        geometry.ceiling,
    )?;
    Ok(SolvedGrow {
        extension: HostRange {
            host: own.host,
            start: tail,
            length: needed,
        },
        end_placement,
        inherited_start: inherited_start(target, own.start),
    })
}

/// The freed tail range for shrinking a target to `new_length`. The
/// start never moves — a start move is PART-005's journaled territory,
/// not a shrink — so a misaligned start is the same inherited fact a
/// grow carries. The authored new end sits inside the target's own
/// extent where no structural edge can pre-exist, so it meets the
/// default or refuses.
///
/// # Errors
///
/// [`SolveRefusal`], the not-a-resize case carrying both lengths.
pub fn shrink_reduction(
    snapshot: &TopologySnapshot,
    target: NodeId,
    new_length: u64,
) -> Result<SolvedShrink, SolveRefusal> {
    let own = extent_of(snapshot, target).ok_or(SolveRefusal::TargetHasNoExtent { target })?;
    if new_length == 0 || new_length >= own.length {
        return Err(SolveRefusal::NotAResize {
            target,
            current: own.length,
            requested: new_length,
        });
    }
    let end = own.start + new_length;
    if !end.is_multiple_of(DEFAULT_ALIGNMENT) {
        return Err(SolveRefusal::UnalignedAuthoredBoundary {
            target,
            boundary: end,
            nearest_aligned_below: (end / DEFAULT_ALIGNMENT) * DEFAULT_ALIGNMENT,
            coincident_candidate: 0,
        });
    }
    Ok(SolvedShrink {
        freed: HostRange {
            host: own.host,
            start: end,
            length: own.length - new_length,
        },
        end_placement: BoundaryPlacement::Aligned,
        inherited_start: inherited_start(target, own.start),
    })
}
