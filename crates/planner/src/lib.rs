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
//! register gates the assignment names — SI-15, SI-16, SI-17, SI-19,
//! SI-24 — gate those later increments; nothing in this one touches
//! their questions.

use partman_capability::engine::{RuntimeFacts, TechnologyLimits, UnknownTarget, capability};
use partman_capability::{Capability, Status};
use partman_domain::model::capability::{Operation, OperationClass, canonical_ranges};
use partman_domain::model::naming::NodeId;
use partman_domain::model::plan::{OperationPlan, PlanError, ValidityWindow};
use partman_domain::model::snapshot::TopologySnapshot;
use partman_domain::model::step::{PlanStep, Severity, StepFlags, StepRefusal, StepRisk};

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
) -> Result<OperationPlan, PlanRefusal> {
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

    OperationPlan::assemble(
        identity.plan_id.clone(),
        identity.created_at,
        snapshot,
        identity.validity,
        std::collections::BTreeMap::new(),
        vec![step],
    )
    .map_err(|error| PlanRefusal::PlanRefused { error })
}
