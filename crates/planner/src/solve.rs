//! The extent solver (WP-060 increment 3), alignment-conservative.
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
//! **SI-15's case refuses by name.** A pre-existing partition whose
//! extent start is not 1 MiB-aligned, grown at its tail, matches
//! neither deviation cause — and realigning it would force a data move
//! the user did not request. The register holds that question
//! (`docs/spec-issues/README.md`, SI-15), so the solver refuses the
//! growth with a typed conflict naming the gate, until the register
//! decides. Refusing is the answer; guessing is what the register
//! exists to prevent.

use partman_domain::model::naming::NodeId;
use partman_domain::model::protection::HostRange;
use partman_domain::model::snapshot::TopologySnapshot;

/// PART-009's default alignment: 1 MiB.
pub const DEFAULT_ALIGNMENT: u64 = 1 << 20;

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
    /// SI-15's held case: the target's extent start is not 1 MiB-aligned,
    /// and growing it at its tail matches neither of PART-009's
    /// deviation causes. The register holds whether such growth is
    /// permitted, requires acknowledgment, or forces realignment; until
    /// it decides, the solver refuses rather than guesses.
    MisalignedLegacyGrowth {
        /// The misaligned target.
        target: NodeId,
        /// Its actual start offset.
        start: u64,
        /// The register gate holding the question.
        gate: &'static str,
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
}

fn extent_of(snapshot: &TopologySnapshot, node: NodeId) -> Option<HostRange> {
    snapshot.facts().extents.get(&node).copied()
}

fn align_up(value: u64, alignment: u64) -> u64 {
    value.div_ceil(alignment) * alignment
}

/// The host's free ranges: its own extent minus the extents the facts
/// place on it, ascending, coalesced by construction.
///
/// # Errors
///
/// [`SolveRefusal::HostHasNoExtent`] if the host carries no extent.
pub fn free_extents(
    snapshot: &TopologySnapshot,
    host: NodeId,
) -> Result<Vec<HostRange>, SolveRefusal> {
    let own = extent_of(snapshot, host).ok_or(SolveRefusal::HostHasNoExtent { host })?;
    let mut children: Vec<(u64, u64)> = snapshot
        .facts()
        .extents
        .iter()
        .filter(|(node, range)| **node != host && range.host == host)
        .map(|(_, range)| (range.start, range.length))
        .collect();
    children.sort_unstable();

    let mut free = Vec::new();
    let mut cursor = own.start;
    let end = own.start + own.length;
    for (start, length) in children {
        if start > cursor {
            free.push(HostRange {
                host,
                start: cursor,
                length: start - cursor,
            });
        }
        cursor = cursor.max(start + length);
    }
    if cursor < end {
        free.push(HostRange {
            host,
            start: cursor,
            length: end - cursor,
        });
    }
    Ok(free)
}

/// First-fit placement for a create: the lowest 1 MiB-aligned start
/// whose free range holds the full size.
///
/// # Errors
///
/// [`SolveRefusal`], the no-fit case naming the largest aligned fit so
/// the caller can explain what would have succeeded.
pub fn place_create(
    snapshot: &TopologySnapshot,
    host: NodeId,
    size: u64,
) -> Result<HostRange, SolveRefusal> {
    if size == 0 {
        return Err(SolveRefusal::NotAResize {
            target: host,
            current: 0,
            requested: 0,
        });
    }
    let free = free_extents(snapshot, host)?;
    let mut largest_aligned_fit = 0_u64;
    for range in &free {
        let aligned_start = align_up(range.start, DEFAULT_ALIGNMENT);
        let range_end = range.start + range.length;
        if aligned_start >= range_end {
            continue;
        }
        let usable = range_end - aligned_start;
        largest_aligned_fit = largest_aligned_fit.max(usable);
        if usable >= size {
            return Ok(HostRange {
                host,
                start: aligned_start,
                length: size,
            });
        }
    }
    Err(SolveRefusal::NoFitForSize {
        requested: size,
        largest_aligned_fit,
    })
}

/// The extension range for growing a target to `new_length` at its
/// tail. SI-15's held case — a misaligned start — refuses by name.
///
/// # Errors
///
/// [`SolveRefusal`], each variant carrying the numbers it judged.
pub fn grow_extension(
    snapshot: &TopologySnapshot,
    target: NodeId,
    new_length: u64,
) -> Result<HostRange, SolveRefusal> {
    let own = extent_of(snapshot, target).ok_or(SolveRefusal::TargetHasNoExtent { target })?;
    if new_length <= own.length {
        return Err(SolveRefusal::NotAResize {
            target,
            current: own.length,
            requested: new_length,
        });
    }
    if own.start % DEFAULT_ALIGNMENT != 0 {
        return Err(SolveRefusal::MisalignedLegacyGrowth {
            target,
            start: own.start,
            gate: "SI-15",
        });
    }
    let needed = new_length - own.length;
    let tail = own.start + own.length;
    let free = free_extents(snapshot, own.host)?;
    let available = free
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
    Ok(HostRange {
        host: own.host,
        start: tail,
        length: needed,
    })
}

/// The freed tail range for shrinking a target to `new_length`. The
/// start never moves — a start move is PART-005's journaled territory,
/// not a shrink.
///
/// # Errors
///
/// [`SolveRefusal`], the not-a-resize case carrying both lengths.
pub fn shrink_reduction(
    snapshot: &TopologySnapshot,
    target: NodeId,
    new_length: u64,
) -> Result<HostRange, SolveRefusal> {
    let own = extent_of(snapshot, target).ok_or(SolveRefusal::TargetHasNoExtent { target })?;
    if new_length == 0 || new_length >= own.length {
        return Err(SolveRefusal::NotAResize {
            target,
            current: own.length,
            requested: new_length,
        });
    }
    Ok(HostRange {
        host: own.host,
        start: own.start + new_length,
        length: own.length - new_length,
    })
}
