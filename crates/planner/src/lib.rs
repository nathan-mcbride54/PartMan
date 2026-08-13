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
use partman_domain::model::identity::TableState;
use partman_domain::model::naming::NodeId;
use partman_domain::model::naming::{NamingFields, NodeEntry};
use partman_domain::model::plan::{
    DraftPrecondition, DraftRefusal, DraftStep, DraftTarget, ImpossibilityReason, OperationPlan,
    PlanError, ReversalDraft, ReversalLinkage, StepImpossibility, ValidityWindow,
};
use partman_domain::model::snapshot::TopologySnapshot;
use partman_domain::model::step::{
    Acknowledgment, PlanStep, Precondition, Severity, StepClass, StepFlags, StepRefusal, StepRisk,
};

use crate::graph::{Dependency, GraphRefusal, execution_order};
use crate::simulate::{Effects, SimulateRefusal, simulate};
use crate::solve::{
    BoundaryPlacement, InheritedFact, SolveRefusal, SolvedCreate, SolvedGrow, SolvedShrink,
    StructuralEdge, grow_extension, place_create, shrink_reduction,
};
use partman_domain::model::protection::{HostRange, StepRanges};

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
    /// The reversal draft refused composition (PLAN-008): emission-time
    /// truthfulness failed, and a plan whose advertised reversal cannot
    /// honestly exist is refused rather than emitted with a false
    /// advertisement.
    ReversalRefused {
        /// The draft's verbatim refusal.
        refusal: DraftRefusal,
    },
    /// SAFE-005's planner half (ADR-0024's ordinary arm): a device this
    /// ordinary request would write carries an `Indeterminate` authored
    /// table state, so the affected write operation is disabled before
    /// PART-013's protection obligation is ever computed. The typed
    /// repair family is the one path that plans over such media, via
    /// [`plan_repair`].
    TableStateIndeterminate {
        /// The device whose table state disables the write.
        device: NodeId,
    },
    /// A repair needs a located table: the facts carry no
    /// partition-table child extent for the target, and the planner
    /// invents no regions (fail-closed — the write targets must be the
    /// authenticated table regions, exactly).
    RepairWithoutLocatedTable {
        /// The target with no located table region.
        device: NodeId,
    },
    /// The typed repair family exists for `Indeterminate` tables
    /// (REC-001, ADR-0024); a repair over a positively determined
    /// state is a future reviewed extension, not a default.
    RepairNeedsAnIndeterminateTable {
        /// The target whose state is positively determined or absent
        /// from the facts.
        device: NodeId,
    },
}

/// PART-013's planning-half output (ADR-0024): the protection
/// obligation each table-bearing device the plan touches will
/// discharge at Protecting — derived from the authored table states at
/// every computation, never stored (the journal's three-variant
/// protection record is the durable artifact, WP-070's). No arm is
/// silent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtectionObligation {
    /// `Present`: the parse-level backup, verified before the first
    /// table write; failure routes to Failed with no writes.
    ParseBackup {
        /// The device backed up.
        device: NodeId,
    },
    /// `Absent`: the obligation discharges as the helper's journaled
    /// fresh positive determination — a value, not a skip, and no user
    /// acknowledgement (ADR-C4 reaching the journal).
    JournaledDetermination {
        /// The device whose absence is the record.
        device: NodeId,
    },
    /// `Indeterminate`, the typed repair family: a raw capture of
    /// exactly the regions the plan will write, verified by re-read —
    /// the only truthful backup of an unsound source.
    RawCapture {
        /// The device captured.
        device: NodeId,
        /// Exactly the write-target regions.
        regions: Vec<HostRange>,
    },
    /// `Indeterminate`, capture impossible: the plan proceeds only
    /// under the plan-creation acknowledgement naming these exact
    /// regions (Section 12's separately supported recovery strategy);
    /// the pre-state is unpreserved by the user's recorded,
    /// region-naming choice.
    AcknowledgedUnpreserved {
        /// The device whose pre-state is acknowledged unpreservable.
        device: NodeId,
        /// The exact acknowledged regions.
        regions: Vec<HostRange>,
    },
}

/// PLAN-008's emitted reversal output: a truthful draft, or the
/// per-step machine-readable impossibility statements — one of the two
/// for every plan, never neither.
#[derive(Debug)]
pub enum EmittedReversal {
    /// The truthful reversal draft, proposing the forward plan's
    /// simulated final topology and binding at its own validation after
    /// the forward apply (ADR-0022).
    Draft(ReversalDraft),
    /// Per-step statements of why reversal is impossible, in the plan
    /// body's step order.
    Impossible(Vec<StepImpossibility>),
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
    /// PLAN-008's output: the emitted reversal draft or the per-step
    /// impossibility statements. The plan body's reversal linkage
    /// carries the same fact in hashed form (the draft by ID and body
    /// hash); this field carries the draft itself for REC-010's
    /// advertisement and UI-005's display.
    pub reversal: EmittedReversal,
    /// The typed consequence facts the plan's consequence text must
    /// state (PART-009 via ADR-0023): inherited off-policy boundaries
    /// the plan leaves byte-identical, and authored boundaries recorded
    /// as coincident. Planner-layer carriage — the hashed consequence-
    /// text vocabulary is a later jointly-sequenced body change, and
    /// ADR-0023 rejected typed hashed carriage of these facts.
    pub consequences: Vec<Consequence>,
    /// PART-013's planning-half output (ADR-0024): the protection
    /// obligation per table-bearing device the plan touches, in device
    /// order — derived, never body content.
    pub protection: Vec<ProtectionObligation>,
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

/// The risk this planner declares for a canonical single-operation
/// step: the operation's class decides the severity conservatively —
/// destructive for `Wipe`, data-moving for the content-moving family,
/// disruptive otherwise — and no flag is set, because every flag's
/// semantics either awaits its unlock increment (ADR-0025's criterion)
/// or a vocabulary a later increment owns. The Reversible claim is
/// made exactly where a truthful draft is emitted — the sized create,
/// in [`plan_sized`] — per ADR-0022's rule: no draft, no Reversible,
/// and with the draft, the claim is honest rather than withheld.
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
        // Everything else is at least disruptive here — conservative-up,
        // stated; the sized create claims Reversible where its draft
        // exists, and the grow deliberately stays here until its
        // FS-grow reversibility story is measured.
        _ => Severity::Disruptive,
    };
    StepRisk {
        severity,
        flags: StepFlags::default(),
    }
}

/// The devices a step's declared ranges and target reach, ascending —
/// the population whose table states select PART-013's arms.
fn touched_devices(target: NodeId, ranges: &StepRanges) -> Vec<NodeId> {
    let mut devices: Vec<NodeId> = ranges
        .written_table_extents
        .iter()
        .chain(&ranges.consumed)
        .chain(&ranges.destroyed)
        .map(|range| range.host)
        .chain(std::iter::once(target))
        .collect();
    devices.sort_unstable();
    devices.dedup();
    devices
}

/// SAFE-005's planner half (ADR-0024's ordinary arm): an ordinary
/// request that would write a device whose authored table state is
/// `Indeterminate` refuses before any protection obligation is
/// computed. The typed repair family is exempt — it is the one path
/// that exists for such media.
fn indeterminate_table_guard(
    snapshot: &TopologySnapshot,
    target: NodeId,
    ranges: &StepRanges,
) -> Result<(), PlanRefusal> {
    for device in touched_devices(target, ranges) {
        if matches!(
            snapshot.facts().table_states.get(&device),
            Some(TableState::Indeterminate { .. })
        ) {
            return Err(PlanRefusal::TableStateIndeterminate { device });
        }
    }
    Ok(())
}

/// PART-013's planning-half derivation (ADR-0024): one obligation per
/// table-bearing touched device, arm-selected by the authored state.
/// The `Indeterminate` arm is reachable only from the typed repair
/// family (the guard refuses it everywhere else), where the obligation
/// is the raw capture of exactly the write-target regions — or the
/// acknowledged-unpreserved arm where the plan carries the
/// capture-impossible acknowledgement for that device.
fn protection_obligations(
    snapshot: &TopologySnapshot,
    steps: &[PlanStep],
) -> Vec<ProtectionObligation> {
    let mut obligations: Vec<ProtectionObligation> = Vec::new();
    let mut seen: Vec<NodeId> = Vec::new();
    for step in steps {
        for device in touched_devices(step.target(), step.ranges()) {
            if seen.contains(&device) {
                continue;
            }
            let Some(state) = snapshot.facts().table_states.get(&device) else {
                continue;
            };
            seen.push(device);
            obligations.push(match state {
                TableState::Present { .. } => ProtectionObligation::ParseBackup { device },
                TableState::Absent => ProtectionObligation::JournaledDetermination { device },
                TableState::Indeterminate { .. } => {
                    let acknowledged =
                        step.acknowledgments().iter().find_map(
                            |acknowledgment| match acknowledgment {
                                Acknowledgment::UncapturableRegions { table, regions }
                                    if *table == device =>
                                {
                                    Some(regions.clone())
                                }
                                _ => None,
                            },
                        );
                    match acknowledged {
                        Some(regions) => {
                            ProtectionObligation::AcknowledgedUnpreserved { device, regions }
                        }
                        None => ProtectionObligation::RawCapture {
                            device,
                            regions: step.ranges().written_table_extents.clone(),
                        },
                    }
                }
            });
        }
    }
    obligations
}

/// PLAN-008's per-operation impossibility reason for the operations the
/// planner can plan today but cannot truthfully reverse. Total over
/// exactly the operations that reach reversal emission; the
/// draft-backed pair and the never-planned rest are structurally
/// upstream.
fn impossibility(operation: Operation) -> ImpossibilityReason {
    match operation {
        // Destroyed bytes are not restorable — the wipe entirely, the
        // shrink's freed tail.
        Operation::Wipe | Operation::Shrink => ImpossibilityReason::DataDestroyed,
        // The model carries no prior value to restore.
        Operation::Label | Operation::Uuid => ImpossibilityReason::PriorValueNotCarried,
        // Create and Grow emit drafts instead of statements, and every
        // other operation refuses before reversal emission (source
        // class, or unrepresentable simulation). A new operation
        // reaching here is a review point, not a default.
        _ => unreachable!("refused before reversal emission"),
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

    let ranges = canonical_ranges(request.operation, request.target, snapshot.facts());
    indeterminate_table_guard(snapshot, request.target, &ranges)?;
    let step = PlanStep::mutating(
        snapshot,
        request.target,
        ranges,
        vec![],
        canonical_risk(request.operation),
    )
    .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;

    let effects = canonical_effects(request.operation, request.target, snapshot)
        .map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;
    let simulated =
        simulate(snapshot, &effects).map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;

    let statements = vec![StepImpossibility {
        step: 0,
        reason: impossibility(request.operation),
    }];
    let protection = protection_obligations(snapshot, std::slice::from_ref(&step));
    OperationPlan::assemble_linked(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: statements.clone(),
        },
    )
    .map(|plan| Planned {
        plan,
        simulated,
        reversal: EmittedReversal::Impossible(statements),
        consequences: vec![],
        protection,
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
    let SolvedRequest {
        operation,
        target,
        ranges,
        effects,
        consequences,
        material,
    } = solve_sized(request, snapshot)?;

    let answer = capability(operation, target, snapshot, limits, runtime)
        .map_err(|UnknownTarget { target }| PlanRefusal::UnknownTarget { target })?;
    if matches!(answer.status(), Status::Unsupported | Status::Blocked) {
        return Err(PlanRefusal::CapabilityRefused { answer });
    }

    // The Reversible claim is made exactly where the truthful draft is
    // emitted (ADR-0022): the sized create is metadata-only and its
    // draft deletes the empty created structure. The grow keeps the
    // conservative severity while its draft still exists — the rule is
    // one-directional: no draft, no Reversible, but a draft does not
    // compel the claim.
    let risk = if matches!(material, ReversalMaterial::CreateDraft { .. }) {
        StepRisk {
            severity: Severity::Reversible,
            flags: StepFlags::default(),
        }
    } else {
        canonical_risk(operation)
    };
    indeterminate_table_guard(snapshot, target, &ranges)?;
    let step = PlanStep::mutating(snapshot, target, ranges, vec![], risk)
        .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;

    let simulated =
        simulate(snapshot, &effects).map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;

    let reversal = emit_reversal(material, &simulated, identity, &step)?;
    let linkage = match &reversal {
        EmittedReversal::Draft(draft) => ReversalLinkage::Draft {
            plan_id: draft.plan_id().to_vec(),
            draft_hash: draft
                .body_hash()
                .map_err(|error| PlanRefusal::PlanRefused { error })?,
        },
        EmittedReversal::Impossible(statements) => ReversalLinkage::Impossible {
            statements: statements.clone(),
        },
    };

    let protection = protection_obligations(snapshot, std::slice::from_ref(&step));
    OperationPlan::assemble_linked(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        vec![step],
        linkage,
    )
    .map(|plan| Planned {
        plan,
        simulated,
        reversal,
        consequences,
        protection,
    })
    .map_err(|error| PlanRefusal::PlanRefused { error })
}

/// One solved sized request: everything [`plan_sized`] needs downstream
/// of the solver.
struct SolvedRequest {
    operation: Operation,
    target: NodeId,
    ranges: StepRanges,
    effects: Effects,
    consequences: Vec<Consequence>,
    material: ReversalMaterial,
}

/// Solve one sized request through the extent solver, carrying the
/// consequence facts and the reversal material each shape decides.
fn solve_sized(
    request: SizedRequest,
    snapshot: &TopologySnapshot,
) -> Result<SolvedRequest, PlanRefusal> {
    match request {
        SizedRequest::Create { host, size } => {
            let SolvedCreate {
                placed,
                end_placement,
            } = place_create(snapshot, host, size)
                .map_err(|refusal| PlanRefusal::SolveRefused { refusal })?;
            Ok(SolvedRequest {
                operation: Operation::Create,
                target: host,
                ranges: StepRanges {
                    written_table_extents: vec![],
                    consumed: vec![placed],
                    destroyed: vec![],
                },
                effects: Effects {
                    minted_partition: Some(placed),
                    ..Effects::default()
                },
                consequences: solved_consequences(
                    host,
                    placed.start + placed.length,
                    end_placement,
                    None,
                ),
                material: ReversalMaterial::CreateDraft { placed },
            })
        }
        SizedRequest::Grow { target, new_length } => {
            let SolvedGrow {
                extension,
                end_placement,
                inherited_start,
            } = grow_extension(snapshot, target, new_length)
                .map_err(|refusal| PlanRefusal::SolveRefused { refusal })?;
            let own_start = snapshot
                .facts()
                .extents
                .get(&target)
                .map_or(0, |extent| extent.start);
            Ok(SolvedRequest {
                operation: Operation::Grow,
                target,
                ranges: StepRanges {
                    written_table_extents: vec![],
                    consumed: vec![extension],
                    destroyed: vec![],
                },
                effects: Effects {
                    resized: vec![(target, new_length)],
                    ..Effects::default()
                },
                consequences: solved_consequences(
                    target,
                    extension.start + extension.length,
                    end_placement,
                    inherited_start,
                ),
                material: ReversalMaterial::GrowDraft {
                    extension,
                    // The reclaimed tail in the target's own address
                    // space: nothing may sit on it for the shrink-back
                    // to stay metadata-only.
                    reclaimed: HostRange {
                        host: target,
                        start: extension.start - own_start,
                        length: extension.length,
                    },
                },
            })
        }
        SizedRequest::Shrink { target, new_length } => {
            let SolvedShrink {
                freed,
                end_placement,
                inherited_start,
            } = shrink_reduction(snapshot, target, new_length)
                .map_err(|refusal| PlanRefusal::SolveRefused { refusal })?;
            Ok(SolvedRequest {
                operation: Operation::Shrink,
                target,
                ranges: StepRanges {
                    written_table_extents: vec![],
                    consumed: vec![],
                    destroyed: vec![freed],
                },
                effects: Effects {
                    resized: vec![(target, new_length)],
                    ..Effects::default()
                },
                consequences: solved_consequences(
                    target,
                    freed.start,
                    end_placement,
                    inherited_start,
                ),
                material: ReversalMaterial::Impossible(ImpossibilityReason::DataDestroyed),
            })
        }
    }
}

/// What [`plan_sized`] decided to emit as PLAN-008's output, before the
/// draft is composed.
#[derive(Clone, Copy)]
enum ReversalMaterial {
    /// The create's reversal: delete the created structure while it
    /// holds nothing, its target spelled as the creating step's output.
    CreateDraft {
        /// The placed range the forward step consumes.
        placed: HostRange,
    },
    /// The grow's reversal: shrink back to the old end while nothing
    /// sits on the reclaimed tail.
    GrowDraft {
        /// The tail extension in the host's address space.
        extension: HostRange,
        /// The same tail in the target's own address space.
        reclaimed: HostRange,
    },
    /// No truthful reversal exists; say why.
    Impossible(ImpossibilityReason),
}

/// Compose the emitted reversal (PLAN-008): the draft's plan ID is
/// derived from the forward plan's deterministically, its proposal is
/// the simulated final topology, and its truthfulness is judged at
/// emission by [`ReversalDraft::compose`].
fn emit_reversal(
    material: ReversalMaterial,
    simulated: &TopologySnapshot,
    identity: &PlanIdentity,
    forward_step: &PlanStep,
) -> Result<EmittedReversal, PlanRefusal> {
    let draft_steps = match material {
        ReversalMaterial::Impossible(reason) => {
            return Ok(EmittedReversal::Impossible(vec![StepImpossibility {
                step: 0,
                reason,
            }]));
        }
        ReversalMaterial::CreateDraft { placed } => vec![DraftStep {
            target: DraftTarget::StepOutput(0),
            ranges: StepRanges {
                written_table_extents: vec![],
                consumed: vec![],
                destroyed: vec![placed],
            },
            acknowledgments: vec![],
            risk: StepRisk {
                severity: Severity::Reversible,
                flags: StepFlags::default(),
            },
            preconditions: vec![DraftPrecondition::StepOutputUnoccupied { step: 0 }],
        }],
        ReversalMaterial::GrowDraft {
            extension,
            reclaimed,
        } => vec![DraftStep {
            target: DraftTarget::Address(reclaimed.host),
            ranges: StepRanges {
                written_table_extents: vec![],
                consumed: vec![],
                destroyed: vec![extension],
            },
            acknowledgments: vec![],
            risk: StepRisk {
                severity: Severity::Reversible,
                flags: StepFlags::default(),
            },
            preconditions: vec![DraftPrecondition::Carried(Precondition::RegionUnoccupied {
                region: reclaimed,
            })],
        }],
    };
    let mut draft_plan_id = identity.plan_id.clone();
    draft_plan_id.extend_from_slice(b"/reversal");
    ReversalDraft::compose(
        draft_plan_id,
        identity.created_at,
        simulated,
        identity.validity,
        draft_steps,
        identity.plan_id.clone(),
        std::slice::from_ref(forward_step),
    )
    .map(EmittedReversal::Draft)
    .map_err(|refusal| PlanRefusal::ReversalRefused { refusal })
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
        let request_ranges = canonical_ranges(request.operation, request.target, snapshot.facts());
        indeterminate_table_guard(snapshot, request.target, &request_ranges)?;
        ranges.push(request_ranges);
    }

    let order = execution_order(&keys, &ranges, &set.dependencies)
        .map_err(|refusal| PlanRefusal::GraphRefused { refusal })?;

    let mut steps = Vec::with_capacity(set.requests.len());
    let mut statements = Vec::with_capacity(set.requests.len());
    for (position, index) in order.order.iter().enumerate() {
        let request = set.requests[*index];
        let step = PlanStep::mutating(
            snapshot,
            request.target,
            canonical_ranges(request.operation, request.target, snapshot.facts()),
            vec![],
            canonical_risk(request.operation),
        )
        .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;
        steps.push(step);
        // PLAN-008's statements follow the emitted step order: every
        // canonical operation this path plans is a wipe or an identity
        // write, none truthfully reversible.
        statements.push(StepImpossibility {
            step: position,
            reason: impossibility(request.operation),
        });
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

    let protection = protection_obligations(snapshot, &steps);
    OperationPlan::assemble_linked(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        steps,
        ReversalLinkage::Impossible {
            statements: statements.clone(),
        },
    )
    .map(|plan| Planned {
        plan,
        simulated,
        reversal: EmittedReversal::Impossible(statements),
        consequences: vec![],
        protection,
    })
    .map_err(|error| PlanRefusal::PlanRefused { error })
}

/// One typed repair-family request (ADR-0024, REC-001's class): repair
/// the named device's located table. The capture-impossible
/// acknowledgement, where the user chose Section 12's separately
/// supported recovery strategy at plan creation, names the exact
/// uncapturable regions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepairRequest {
    /// The device whose table is repaired.
    pub target: NodeId,
    /// The plan-creation acknowledgement's regions, where capture is
    /// known impossible — `None` plans the ordinary raw-capture arm.
    pub acknowledged_uncapturable: Option<Vec<HostRange>>,
}

/// The located table regions for a repair: the authenticated extents of
/// the target's partition-table children — never invented (fail-closed
/// where the facts locate no table).
fn located_table_regions(snapshot: &TopologySnapshot, target: NodeId) -> Vec<HostRange> {
    snapshot
        .topology()
        .entries()
        .iter()
        .filter_map(|entry| {
            let fields = match entry {
                NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
            };
            matches!(fields, NamingFields::PartitionTable { parent, .. } if *parent == target)
                .then(|| snapshot.facts().extents.get(&entry.id()).copied())
                .flatten()
        })
        .collect()
}

/// PLAN-001's computation for the typed repair family (ADR-0024): the
/// step is `table-repair` class over the located table regions —
/// exactly the regions the plan will write, which is exactly what the
/// raw-capture obligation preserves — the simulation drops the target's
/// stamp (the post-repair state is unestablished until a real capture,
/// the wipe precedent), and the reversal is the pre-state-preserved
/// statement: the capture is the substrate, and restoring it is
/// REC-001's recovery plan.
///
/// # Errors
///
/// [`PlanRefusal`], each variant carrying its ground verbatim.
pub fn plan_repair(
    request: &RepairRequest,
    snapshot: &TopologySnapshot,
    limits: &TechnologyLimits,
    runtime: &RuntimeFacts,
    identity: &PlanIdentity,
) -> Result<Planned, PlanRefusal> {
    let answer = capability(Operation::Repair, request.target, snapshot, limits, runtime)
        .map_err(|UnknownTarget { target }| PlanRefusal::UnknownTarget { target })?;
    if matches!(answer.status(), Status::Unsupported | Status::Blocked) {
        return Err(PlanRefusal::CapabilityRefused { answer });
    }
    if !matches!(
        snapshot.facts().table_states.get(&request.target),
        Some(TableState::Indeterminate { .. })
    ) {
        return Err(PlanRefusal::RepairNeedsAnIndeterminateTable {
            device: request.target,
        });
    }
    let regions = located_table_regions(snapshot, request.target);
    if regions.is_empty() {
        return Err(PlanRefusal::RepairWithoutLocatedTable {
            device: request.target,
        });
    }
    let acknowledgments = match &request.acknowledged_uncapturable {
        Some(acknowledged) => vec![Acknowledgment::UncapturableRegions {
            table: request.target,
            regions: acknowledged.clone(),
        }],
        None => vec![],
    };
    let step = PlanStep::mutating_classed(
        snapshot,
        request.target,
        StepRanges {
            written_table_extents: regions,
            consumed: vec![],
            destroyed: vec![],
        },
        acknowledgments,
        StepRisk {
            // Table metadata is rewritten in place over an unsound
            // source; loss is possible on failure — conservative-up,
            // stated.
            severity: Severity::DataMoving,
            flags: StepFlags::default(),
        },
        StepClass::TableRepair,
    )
    .map_err(|refusal| PlanRefusal::StepRefused { refusal })?;

    let simulated = simulate(
        snapshot,
        &Effects {
            stamp_dropped: vec![request.target],
            ..Effects::default()
        },
    )
    .map_err(|refusal| PlanRefusal::SimulateRefused { refusal })?;

    let statements = vec![StepImpossibility {
        step: 0,
        reason: ImpossibilityReason::PreStatePreservedForRecovery,
    }];
    let protection = protection_obligations(snapshot, std::slice::from_ref(&step));
    OperationPlan::assemble_linked(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: statements.clone(),
        },
    )
    .map(|plan| Planned {
        plan,
        simulated,
        reversal: EmittedReversal::Impossible(statements),
        consequences: vec![],
        protection,
    })
    .map_err(|error| PlanRefusal::PlanRefused { error })
}
