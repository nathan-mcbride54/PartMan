//! The WP-060 pure planner's chassis (increment 1).
//!
//! [`plan`] is PLAN-001's computation: deterministic and side-effect
//! free from a captured snapshot, the capability engine's answer, and a
//! typed request to an [`OperationPlan`] — or to a typed refusal that
//! carries the ground verbatim. Purity is structural: no clock (the
//! caller supplies creation time and the PLAN-007 window), no
//! randomness, no I/O, and determinism is held by test as byte-equal
//! plan bodies for equal inputs.
//!
//! The conditioning rule (ACC-009's planner half, CAP-007 both ways):
//!
//! - an `unsupported` or `blocked` capability answer refuses the request
//!   with **that answer carried verbatim** — reason and remediation
//!   travel, never re-derived and never paraphrased;
//! - `preview` permits planning — planning and simulation are exactly
//!   what CAP-003 defines the status to permit;
//! - `supported` is not a distinct planning state: it differs from
//!   `preview` at apply, and apply does not exist here;
//! - and no answer can admit a step the closure refuses — every step is
//!   [`PlanStep::mutating`] over the capture's authenticated facts, so a
//!   capability answer is advisory input to sequencing, never authority
//!   over construction.
//!
//! This increment plans single-operation requests whose step is the
//! operation's canonical effect-table entry. The step graph (increment
//! 2), the extent solver with request parameters (increment 3), and the
//! simulated final topology (increment 4) build on this chassis. The
//! register gates the assignment named — SI-15, SI-16, SI-17, SI-19,
//! SI-24 — have each resolved through a recorded decision; the unlock
//! increments implement those decisions' fixtures, starting with
//! ADR-0023's authored/inherited alignment rule in the solver
//! (increment 5).

use partman_capability::engine::{RuntimeFacts, TechnologyLimits, UnknownTarget, capability};
use partman_capability::{Capability, Status};
use partman_domain::model::capability::{Operation, OperationClass, canonical_ranges};
use partman_domain::model::naming::NodeId;
use partman_domain::model::plan::{OperationPlan, PlanError, ValidityWindow};
use partman_domain::model::snapshot::TopologySnapshot;
use partman_domain::model::step::{PlanStep, Severity, StepFlags, StepRefusal, StepRisk};

use crate::graph::{Dependency, GraphRefusal, execution_order};
use crate::simulate::{Effects, SimulateRefusal, simulate};
use crate::solve::{
    BoundaryPlacement, InheritedFact, SolveRefusal, SolvedCreate, SolvedGrow, SolvedShrink,
    StructuralEdge, grow_extension, place_create, shrink_reduction,
};
use partman_domain::model::protection::StepRanges;

pub mod graph;
pub mod simulate;
pub mod solve;

#[cfg(test)]
mod tests;

/// One planning request: an operation on an exact target. Parameters
/// (sizes, placements, labels) arrive with the extent solver's
/// increment, as decided vocabularies permit them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanRequest {
    /// The requested operation.
    pub operation: Operation,
    /// The exact target (CAP-001's grain).
    pub target: NodeId,
}

/// The caller-supplied identity and validity of the plan being built.
/// The planner is pure: creation time and the PLAN-007 window are
/// inputs, and the 24-hour default / 7-day maximum are the calling
/// surface's policy to apply before this boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanIdentity {
    /// The plan identifier bytes.
    pub plan_id: Vec<u8>,
    /// Creation timestamp, seconds since the epoch.
    pub created_at: u64,
    /// PLAN-007's validity window.
    pub validity: ValidityWindow,
}

/// Why the planner refused a request — each variant carrying its ground
/// verbatim, never a paraphrase.
#[derive(Debug, PartialEq, Eq)]
pub enum PlanRefusal {
    /// The capability engine's answer forbids planning this request:
    /// the full answer travels with the refusal (reason, remediation,
    /// status), exactly as the engine stated it.
    CapabilityRefused {
        /// The engine's verbatim answer.
        answer: Capability,
    },
    /// The request names a target the snapshot does not carry.
    UnknownTarget {
        /// The unresolvable address.
        target: NodeId,
    },
    /// The sole constructor refused the step — the closure's own
    /// refusal, which no capability answer can override (CAP-007).
    StepRefused {
        /// The constructor's verbatim refusal.
        refusal: StepRefusal,
    },
    /// The plan boundary refused assembly.
    PlanRefused {
        /// The boundary's verbatim error.
        error: PlanError,
    },
    /// A source-class operation is not plan material: plans mutate, and
    /// detection or reading needs no authorization artifact. The request
    /// belongs on the inspection surfaces, not here.
    NotAPlanningOperation {
        /// The source-class operation requested.
        operation: Operation,
    },
    /// The graph layer refused the request set — a cycle, a duplicate,
    /// or an unordered overlap, each explained by its variant
    /// (PLAN-003).
    GraphRefused {
        /// The graph's verbatim refusal.
        refusal: GraphRefusal,
    },
    /// The extent solver refused — no fit, missing extents, a
    /// non-resize, or an authored boundary with no lawful spelling
    /// (ADR-0023's no-fourth-state rule), each explained by its
    /// variant with the numbers it judged.
    SolveRefused {
        /// The solver's verbatim refusal.
        refusal: SolveRefusal,
    },
    /// Simulation refused, and PLAN-002 makes that the plan's refusal:
    /// every valid plan produces both topologies, so an effect this
    /// model cannot represent produces no valid plan at all.
    SimulateRefused {
        /// The simulation's verbatim refusal.
        refusal: SimulateRefusal,
    },
}

/// PLAN-002's complete product: the plan and the simulated final
/// topology it predicts, emitted together because a plan without its
/// simulation is not valid.
#[derive(Debug)]
pub struct Planned {
    /// The operation plan, bound to the capture.
    pub plan: OperationPlan,
    /// The simulated final topology, `SnapshotKind::Simulated` — the
    /// schema string that can never be a planning base or satisfy a
    /// PLAN-006 comparison.
    pub simulated: TopologySnapshot,
    /// The typed consequence facts the plan's consequence text must
    /// state (PART-009 via ADR-0023): inherited off-policy boundaries
    /// the plan leaves byte-identical, and authored boundaries recorded
    /// as coincident. Planner-layer carriage — the hashed consequence-
    /// text vocabulary is a later jointly-sequenced body change, and
    /// ADR-0023 rejected typed hashed carriage of these facts.
    pub consequences: Vec<Consequence>,
}

/// One typed consequence fact, with its user-facing sentence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consequence {
    /// The target's pre-existing start is off the 1 MiB default and the
    /// plan leaves it byte-identical: a fact about the device, never a
    /// grant by the user (ADR-0023).
    InheritedMisalignedStart {
        /// The target whose start is inherited.
        target: NodeId,
        /// The inherited start offset.
        start: u64,
    },
    /// An authored boundary was placed coincident with a pre-existing
    /// structural edge and is recorded as such (ADR-0023's
    /// coincident-edge rule).
    CoincidentBoundary {
        /// The target whose boundary was authored.
        target: NodeId,
        /// The authored offset.
        boundary: u64,
        /// The edge coincided with.
        edge: StructuralEdge,
    },
}

impl std::fmt::Display for Consequence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InheritedMisalignedStart { target, start } => write!(
                formatter,
                "{target} starts at byte {start}, off the 1 MiB default; the start predates this \
                 plan and is left byte-identical — an inherited fact about the device, not a \
                 change this plan makes"
            ),
            Self::CoincidentBoundary {
                target,
                boundary,
                edge,
            } => match edge {
                StructuralEdge::NeighborStart { neighbor } => write!(
                    formatter,
                    "{target}'s new boundary at byte {boundary} coincides with {neighbor}'s start \
                     and is recorded as coincident"
                ),
                StructuralEdge::HostEnd => write!(
                    formatter,
                    "{target}'s new boundary at byte {boundary} coincides with the end of its \
                     host and is recorded as coincident"
                ),
            },
        }
    }
}

/// The consequence facts one solved request contributes: the coincident
/// record where the end placement is coincident, and the inherited
/// start where one exists.
fn solved_consequences(
    target: NodeId,
    end: u64,
    end_placement: BoundaryPlacement,
    inherited: Option<InheritedFact>,
) -> Vec<Consequence> {
    let mut consequences = Vec::new();
    if let BoundaryPlacement::Coincident { edge } = end_placement {
        consequences.push(Consequence::CoincidentBoundary {
            target,
            boundary: end,
            edge,
        });
    }
    if let Some(InheritedFact::MisalignedStart { target, start }) = inherited {
        consequences.push(Consequence::InheritedMisalignedStart { target, start });
    }
    consequences
}

/// The canonical-operation effects this model can honestly simulate.
fn canonical_effects(
    operation: Operation,
    target: NodeId,
    snapshot: &TopologySnapshot,
) -> Result<Effects, SimulateRefusal> {
    match operation {
        Operation::Wipe => Ok(Effects {
            destroyed: canonical_ranges(operation, target, snapshot.facts()).destroyed,
            stamp_dropped: vec![target],
            minted_partition: None,
            resized: vec![],
        }),
        // This model carries no labels or mutable identifiers, so at
        // this granularity the topology genuinely does not change:
        // identity is exact, not lazy.
        Operation::Label | Operation::Uuid => Ok(Effects::default()),
        Operation::Create => Err(SimulateRefusal::NotRepresentable {
            effect: "an unsized create has no placed range; use the sized request",
        }),
        Operation::Grow | Operation::Shrink => Err(SimulateRefusal::NotRepresentable {
            effect: "an unsized resize has no target length; use the sized request",
        }),
        Operation::Move | Operation::Copy => Err(SimulateRefusal::NotRepresentable {
            effect: "moves and copies need a destination vocabulary this model does not carry yet",
        }),
        Operation::Repair => Err(SimulateRefusal::NotRepresentable {
            effect: "repair outcomes are not predictable topology",
        }),
        Operation::Encrypt | Operation::Decrypt => Err(SimulateRefusal::NotRepresentable {
            effect: "encryption-layer minting needs vocabulary this increment does not invent",
        }),
        Operation::Detect | Operation::Read | Operation::Check => {
            Err(SimulateRefusal::NotRepresentable {
                effect: "source operations are refused before simulation",
            })
        }
    }
}

/// A sized request: the solver-backed operations, each carrying the
/// geometry the caller decided and the solver validates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizedRequest {
    /// Create a structure of `size` bytes in the host's free space,
    /// placed by the solver at PART-009's default alignment.
    Create {
        /// The host receiving the new structure.
        host: NodeId,
        /// The requested size in bytes.
        size: u64,
    },
    /// Grow the target to `new_length` at its tail.
    Grow {
        /// The target being grown.
        target: NodeId,
        /// The requested final length.
        new_length: u64,
    },
    /// Shrink the target to `new_length`; the start never moves.
    Shrink {
        /// The target being shrunk.
        target: NodeId,
        /// The requested final length.
        new_length: u64,
    },
}

/// A multi-request planning input: requests plus the dependency edges
/// that order them (PLAN-003's graph, explicit). An empty dependency
/// list is a set of independent steps — legal exactly when no two
/// declare effects on the same bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRequestSet {
    /// The requests, in caller order. Indices in `dependencies` refer
    /// to positions here.
    pub requests: Vec<PlanRequest>,
    /// The dependency edges: `before` must precede `after`.
    pub dependencies: Vec<Dependency>,
}

/// The risk this increment declares for a canonical single-operation
/// step: the operation's class decides the severity conservatively —
/// destructive for `Wipe`, data-moving for the content-moving family,
/// disruptive otherwise — and no flag is set, because every flag's
/// semantics either awaits SI-17 (the contested combination) or a
/// vocabulary a later increment owns. Conservative and stated is the
/// increment-1 posture; the risk model's full conditioning arrives with
/// the solver.
fn canonical_risk(operation: Operation) -> StepRisk {
    let severity = match operation {
        // Intentional destruction, PLAN-004's own definition.
        Operation::Wipe => Severity::Destructive,
        // Data is relocated or transformed; loss is possible on failure.
        Operation::Move
        | Operation::Copy
        | Operation::Shrink
        | Operation::Encrypt
        | Operation::Decrypt => Severity::DataMoving,
        // Everything else is at least disruptive here, deliberately:
        // severity 0 never fits a mutating step, and severity 1 claims a
        // "fully undoable via an emitted reversal plan" that PLAN-008
        // cannot emit until SI-19 decides its binding — a Reversible
        // claim without the reversal would be the assertion this
        // codebase refuses everywhere else. Conservative-up, stated.
        _ => Severity::Disruptive,
    };
    StepRisk {
        severity,
        flags: StepFlags::default(),
    }
}

/// PLAN-001's computation: plan one request over a captured snapshot,
/// conditioning on the capability engine's answer and constructing
/// through the domain's sole constructors.
///
/// # Errors
///
/// [`PlanRefusal`], each variant carrying its ground verbatim.
pub fn plan(
    request: PlanRequest,
    snapshot: &TopologySnapshot,
    limits: &TechnologyLimits,
    runtime: &RuntimeFacts,
    identity: &PlanIdentity,
) -> Result<Planned, PlanRefusal> {
    if request.operation.class() == OperationClass::Source {
        return Err(PlanRefusal::NotAPlanningOperation {
            operation: request.operation,
        });
    }
    let answer = capability(request.operation, request.target, snapshot, limits, runtime)
        .map_err(|UnknownTarget { target }| PlanRefusal::UnknownTarget { target })?;
    match answer.status() {
        Status::Unsupported | Status::Blocked => {
            return Err(PlanRefusal::CapabilityRefused { answer });
        }
        Status::Preview | Status::Supported => {}
    }

    let step = PlanStep::mutating(
        snapshot,
        request.target,
        canonical_ranges(request.operation, request.target, snapshot.facts()),
        vec![],
        canonical_risk(request.operation),
    )
    .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;

    let effects = canonical_effects(request.operation, request.target, snapshot)
        .map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;
    let simulated =
        simulate(snapshot, &effects).map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;

    OperationPlan::assemble(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        vec![step],
    )
    .map(|plan| Planned {
        plan,
        simulated,
        consequences: vec![],
    })
    .map_err(|error| PlanRefusal::PlanRefused { error })
}

/// PLAN-001's computation for one sized request: the solver validates
/// the geometry, the capability engine conditions the pair, and the
/// step carries the solved ranges — a create consumes its placed range,
/// a grow consumes its tail extension, a shrink destroys its freed
/// tail, because bytes beyond the new end are gone and the risk model
/// says so.
///
/// # Errors
///
/// [`PlanRefusal`], each variant carrying its ground verbatim.
pub fn plan_sized(
    request: SizedRequest,
    snapshot: &TopologySnapshot,
    limits: &TechnologyLimits,
    runtime: &RuntimeFacts,
    identity: &PlanIdentity,
) -> Result<Planned, PlanRefusal> {
    let (operation, target, ranges, effects, consequences) = match request {
        SizedRequest::Create { host, size } => {
            let SolvedCreate {
                placed,
                end_placement,
            } = place_create(snapshot, host, size)
                .map_err(|refusal| PlanRefusal::SolveRefused { refusal })?;
            (
                Operation::Create,
                host,
                StepRanges {
                    written_table_extents: vec![],
                    consumed: vec![placed],
                    destroyed: vec![],
                },
                Effects {
                    minted_partition: Some(placed),
                    ..Effects::default()
                },
                solved_consequences(host, placed.start + placed.length, end_placement, None),
            )
        }
        SizedRequest::Grow { target, new_length } => {
            let SolvedGrow {
                extension,
                end_placement,
                inherited_start,
            } = grow_extension(snapshot, target, new_length)
                .map_err(|refusal| PlanRefusal::SolveRefused { refusal })?;
            (
                Operation::Grow,
                target,
                StepRanges {
                    written_table_extents: vec![],
                    consumed: vec![extension],
                    destroyed: vec![],
                },
                Effects {
                    resized: vec![(target, new_length)],
                    ..Effects::default()
                },
                solved_consequences(
                    target,
                    extension.start + extension.length,
                    end_placement,
                    inherited_start,
                ),
            )
        }
        SizedRequest::Shrink { target, new_length } => {
            let SolvedShrink {
                freed,
                end_placement,
                inherited_start,
            } = shrink_reduction(snapshot, target, new_length)
                .map_err(|refusal| PlanRefusal::SolveRefused { refusal })?;
            (
                Operation::Shrink,
                target,
                StepRanges {
                    written_table_extents: vec![],
                    consumed: vec![],
                    destroyed: vec![freed],
                },
                Effects {
                    resized: vec![(target, new_length)],
                    ..Effects::default()
                },
                solved_consequences(target, freed.start, end_placement, inherited_start),
            )
        }
    };

    let answer = capability(operation, target, snapshot, limits, runtime)
        .map_err(|UnknownTarget { target }| PlanRefusal::UnknownTarget { target })?;
    if matches!(answer.status(), Status::Unsupported | Status::Blocked) {
        return Err(PlanRefusal::CapabilityRefused { answer });
    }

    let step = PlanStep::mutating(snapshot, target, ranges, vec![], canonical_risk(operation))
        .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;

    let simulated =
        simulate(snapshot, &effects).map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;

    OperationPlan::assemble(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        vec![step],
    )
    .map(|plan| Planned {
        plan,
        simulated,
        consequences,
    })
    .map_err(|error| PlanRefusal::PlanRefused { error })
}

/// The position of an operation in CAP-002's list — the discriminant
/// the duplicate-request check keys on.
fn operation_index(operation: Operation) -> u8 {
    let position = Operation::all()
        .iter()
        .position(|candidate| *candidate == operation)
        .expect("every operation is in CAP-002's list");
    u8::try_from(position).expect("fourteen operations fit in a byte")
}

/// PLAN-003's computation: plan a request set over a captured snapshot.
/// Every request passes the same conditioning as [`plan`]; the graph
/// layer then refuses cycles, duplicates, and dependency-unordered
/// overlaps with typed explanations, and the plan's steps carry the
/// deterministic execution order (Kahn's, smallest ready index first).
///
/// # Errors
///
/// [`PlanRefusal`], each variant carrying its ground verbatim. Requests
/// are conditioned in caller order, so the first refusing request
/// decides the error deterministically.
pub fn plan_set(
    set: &PlanRequestSet,
    snapshot: &TopologySnapshot,
    limits: &TechnologyLimits,
    runtime: &RuntimeFacts,
    identity: &PlanIdentity,
) -> Result<Planned, PlanRefusal> {
    let mut keys = Vec::with_capacity(set.requests.len());
    let mut ranges = Vec::with_capacity(set.requests.len());
    for request in &set.requests {
        if request.operation.class() == OperationClass::Source {
            return Err(PlanRefusal::NotAPlanningOperation {
                operation: request.operation,
            });
        }
        let answer = capability(request.operation, request.target, snapshot, limits, runtime)
            .map_err(|UnknownTarget { target }| PlanRefusal::UnknownTarget { target })?;
        if matches!(answer.status(), Status::Unsupported | Status::Blocked) {
            return Err(PlanRefusal::CapabilityRefused { answer });
        }
        keys.push((operation_index(request.operation), request.target));
        ranges.push(canonical_ranges(
            request.operation,
            request.target,
            snapshot.facts(),
        ));
    }

    let order = execution_order(&keys, &ranges, &set.dependencies)
        .map_err(|refusal| PlanRefusal::GraphRefused { refusal })?;

    let mut steps = Vec::with_capacity(set.requests.len());
    for index in order.order {
        let request = set.requests[index];
        let step = PlanStep::mutating(
            snapshot,
            request.target,
            canonical_ranges(request.operation, request.target, snapshot.facts()),
            vec![],
            canonical_risk(request.operation),
        )
        .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;
        steps.push(step);
    }

    let mut combined = Effects::default();
    for request in &set.requests {
        let effects = canonical_effects(request.operation, request.target, snapshot)
            .map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;
        combined.destroyed.extend(effects.destroyed);
        combined.stamp_dropped.extend(effects.stamp_dropped);
        combined.resized.extend(effects.resized);
    }
    let simulated = simulate(snapshot, &combined)
        .map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;

    OperationPlan::assemble(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        steps,
    )
    .map(|plan| Planned {
        plan,
        simulated,
        consequences: vec![],
    })
    .map_err(|error| PlanRefusal::PlanRefused { error })
}
