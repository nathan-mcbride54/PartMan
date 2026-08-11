//! The step graph (WP-060 increment 2): PLAN-003's acyclic dependency
//! machinery, with conflicts refused as typed values that explain
//! themselves — which two steps, which resource, which rule — never as
//! a boolean.
//!
//! The conflict rule this increment commits: **overlap between
//! dependency-unordered steps is the conflict.** Two steps whose
//! declared effect ranges touch the same bytes of the same host are
//! legitimate when a dependency path orders them — a wipe followed by a
//! create in the freed space is a chain, and the dependency is its
//! explanation — but with no path in either direction, no execution
//! order makes concurrent effects on the same bytes deterministic, so
//! the pair refuses with both steps and the host named. Duplicate
//! requests (same operation, same target) refuse before ranges are
//! even compared: a plan that says one thing twice is a request error,
//! not a sequencing problem.
//!
//! Ordering is deterministic (PLAN-001): Kahn's algorithm with the
//! smallest ready index first, so equal request sets produce byte-equal
//! step arrays, and the emitted order is stable under everything but
//! the dependencies themselves.

use partman_domain::model::naming::NodeId;
use partman_domain::model::protection::StepRanges;

/// One dependency edge: the request at `before` must precede the
/// request at `after`. Indices are positions in the request list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dependency {
    /// The prerequisite request's index.
    pub before: usize,
    /// The dependent request's index.
    pub after: usize,
}

/// Why the graph layer refused a request set — each variant naming the
/// steps and resource it explains (PLAN-003's "explain the conflict").
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphRefusal {
    /// A dependency edge names a request index that does not exist.
    DependencyOutOfRange {
        /// The offending edge.
        edge: Dependency,
        /// How many requests the set actually holds.
        requests: usize,
    },
    /// A dependency edge from a request to itself.
    SelfDependency {
        /// The self-referential index.
        index: usize,
    },
    /// The dependencies contain a cycle: no execution order satisfies
    /// them. The members are every request left unordered when the
    /// sort exhausted its ready set — the cycle and everything
    /// downstream of it, named so the caller sees exactly what cannot
    /// be sequenced.
    Cycle {
        /// The unorderable request indices, ascending.
        members: Vec<usize>,
    },
    /// Two requests are the same operation on the same target: a plan
    /// that says one thing twice is a request error.
    DuplicateRequest {
        /// The first request's index.
        first: usize,
        /// The duplicate's index.
        second: usize,
    },
    /// Two dependency-unordered steps declare effects on the same bytes
    /// of the same host: no order makes them deterministic, and the
    /// dependency that would explain the overlap is absent.
    UnorderedOverlap {
        /// The lower request index.
        first: usize,
        /// The higher request index.
        second: usize,
        /// The host whose bytes both steps touch.
        host: NodeId,
    },
}

/// A validated execution order: request indices in the deterministic
/// topological order the plan body will carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOrder {
    /// Request indices, dependency-consistent, smallest-ready-first.
    pub order: Vec<usize>,
}

fn ranges_overlap(left: &StepRanges, right: &StepRanges) -> Option<NodeId> {
    let all_left = left
        .written_table_extents
        .iter()
        .chain(&left.consumed)
        .chain(&left.destroyed);
    for a in all_left {
        let all_right = right
            .written_table_extents
            .iter()
            .chain(&right.consumed)
            .chain(&right.destroyed);
        for b in all_right {
            if a.host == b.host && a.start < b.start + b.length && b.start < a.start + a.length {
                return Some(a.host);
            }
        }
    }
    None
}

/// Whether `from` reaches `to` through the dependency edges.
fn reaches(edges: &[Dependency], from: usize, to: usize) -> bool {
    let mut stack = vec![from];
    let mut seen = vec![from];
    while let Some(current) = stack.pop() {
        if current == to {
            return true;
        }
        for edge in edges {
            if edge.before == current && !seen.contains(&edge.after) {
                seen.push(edge.after);
                stack.push(edge.after);
            }
        }
    }
    false
}

/// Validate the dependency graph over the requests' declared effect
/// ranges and produce the deterministic execution order.
///
/// `keys` are the per-request (operation-discriminant, target) pairs
/// used for duplicate detection; `ranges` are each request's declared
/// effect ranges in request order.
///
/// # Errors
///
/// [`GraphRefusal`], each variant explaining its conflict.
pub fn execution_order(
    keys: &[(u8, NodeId)],
    ranges: &[StepRanges],
    dependencies: &[Dependency],
) -> Result<ExecutionOrder, GraphRefusal> {
    let count = keys.len();
    for edge in dependencies {
        if edge.before >= count || edge.after >= count {
            return Err(GraphRefusal::DependencyOutOfRange {
                edge: *edge,
                requests: count,
            });
        }
        if edge.before == edge.after {
            return Err(GraphRefusal::SelfDependency { index: edge.before });
        }
    }
    for first in 0..count {
        for second in (first + 1)..count {
            if keys[first] == keys[second] {
                return Err(GraphRefusal::DuplicateRequest { first, second });
            }
        }
    }
    for first in 0..count {
        for second in (first + 1)..count {
            let ordered =
                reaches(dependencies, first, second) || reaches(dependencies, second, first);
            if !ordered && let Some(host) = ranges_overlap(&ranges[first], &ranges[second]) {
                return Err(GraphRefusal::UnorderedOverlap {
                    first,
                    second,
                    host,
                });
            }
        }
    }

    // Kahn's algorithm, smallest ready index first: deterministic under
    // PLAN-001, and the emitted order is the body's semantic order.
    let mut incoming: Vec<usize> = vec![0; count];
    for edge in dependencies {
        incoming[edge.after] += 1;
    }
    let mut order = Vec::with_capacity(count);
    let mut done: Vec<bool> = vec![false; count];
    while order.len() < count {
        let Some(next) = (0..count).find(|index| !done[*index] && incoming[*index] == 0) else {
            let members: Vec<usize> = (0..count).filter(|index| !done[*index]).collect();
            return Err(GraphRefusal::Cycle { members });
        };
        done[next] = true;
        order.push(next);
        for edge in dependencies {
            if edge.before == next {
                incoming[edge.after] -= 1;
            }
        }
    }
    Ok(ExecutionOrder { order })
}
