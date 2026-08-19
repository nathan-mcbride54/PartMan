//! Validate-plan (increment 2): the planner run over the helper's own
//! capture, and the SEC-002 admission arms a presented plan must pass.
//!
//! **The architecture, restated where it is enforced.** A client's draft,
//! its capability answers and its validation output are untrusted hints
//! (HLP-002, CAP-007): the helper does not check a client's plan — it
//! **re-plans**, running WP-060's `plan()` over the capture it just took,
//! so the plan that comes back binds the helper's snapshot hash
//! (ADR-0014: "as bound at validation"), carries the helper's own
//! severity and flags, and is refused on the engine's and the closure's
//! own grounds, verbatim. SI-13's structural interim is enforced before
//! the planner runs: a plan whose target is an `Aggregate` is refused
//! with the register issue named, until the round this package files
//! decides identity binding for pool and array targets.
//!
//! **The admission arms** ([`admit_presented_plan`]) are the checking
//! function increment 3's apply consumes: replayed, cross-user, altered,
//! stale, cross-device and expired plans each die on a typed arm, in that
//! order, against a **fresh** capture and the helper's clock. They are
//! pure and tested over authored inputs here; the journal-backed act that
//! feeds them one consumable record per apply is increment 3's (ADR-0021,
//! ADR-0028).

use partman_capability::engine::{RuntimeFacts, TechnologyLimits};
use partman_domain::canonical::{self, Hash};
use partman_domain::model::capability::Operation;
use partman_domain::model::naming::{NamingFields, NodeEntry, NodeId};
use partman_domain::model::plan::{OperationPlan, PlanSchemaError, ValidityWindow};
use partman_domain::model::snapshot::TopologySnapshot;
use partman_domain::model::step::{Severity, StepFlags};
use partman_planner::{PlanIdentity, PlanRefusal, PlanRequest, plan, plan_flags};

/// PLAN-007's default validity: 24 hours.
pub const VALIDITY_DEFAULT_SECONDS: u64 = 86_400;
/// PLAN-007's maximum validity: 7 days. A longer request refuses.
pub const VALIDITY_MAXIMUM_SECONDS: u64 = 604_800;

/// One validate-plan request, already decoded from the wire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidateRequest {
    /// The exact target's derived address (CAP-001's grain). Addresses
    /// are positional and derivable by any layer from the same facts, so
    /// a client can name a target without the helper trusting anything
    /// else it says.
    pub target: NodeId,
    /// The requested CAP-002 operation.
    pub operation: Operation,
    /// The plan identifier bytes the client chose (correlation, not
    /// authority; bounded at the wire).
    pub plan_id: Vec<u8>,
    /// The requested validity in seconds; `0` takes the default.
    pub validity_seconds: u64,
}

/// A plan the helper validated: the body, its hash, and what the client
/// needs to display and to apply by hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPlan {
    /// The plan body's canonical bytes.
    pub body_bytes: Vec<u8>,
    /// The body hash — what HLP-003's act will name, exactly.
    pub body_hash: Hash,
    /// The snapshot body hash the plan binds (the helper's own capture).
    pub snapshot_hash: Hash,
    /// The helper-computed plan severity (PLAN-004).
    pub severity: Severity,
    /// The helper-computed flag union (PLAN-004).
    pub flags: StepFlags,
    /// Section 6's user-facing consequence sentences, as the body carries
    /// them.
    pub consequences: Vec<String>,
    /// The window's end (PLAN-007).
    pub not_after: u64,
}

/// Why validation refused. Typed; the planner's grounds travel verbatim.
#[derive(Debug)]
pub enum ValidateRefusal {
    /// SI-13's structural interim: the target is an `Aggregate`, and no
    /// binding constructor for aggregates exists until the register's
    /// round decides one.
    AggregateTarget {
        /// The refused target.
        target: NodeId,
    },
    /// The requested validity exceeds PLAN-007's maximum.
    ValidityOverMaximum {
        /// The seconds requested.
        requested: u64,
    },
    /// The planner refused, its ground carried verbatim.
    Planner(PlanRefusal),
    /// The plan body could not be encoded (unreachable for planner
    /// output, reported rather than panicked).
    Encoding,
}

/// Validate one request over the helper's capture: SI-13's arm, the
/// window policy, then WP-060's `plan()` with the engine's facts.
///
/// # Errors
///
/// [`ValidateRefusal`].
pub fn validate_plan(
    capture: &TopologySnapshot,
    request: &ValidateRequest,
    now: u64,
    limits: &TechnologyLimits,
    runtime: &RuntimeFacts,
) -> Result<ValidatedPlan, ValidateRefusal> {
    if let Some(entry) = capture
        .topology()
        .entries()
        .iter()
        .find(|entry| entry.id() == request.target)
    {
        let fields = match entry {
            NodeEntry::Single { fields, .. } | NodeEntry::Group { fields, .. } => fields,
        };
        if matches!(fields, NamingFields::Aggregate { .. }) {
            return Err(ValidateRefusal::AggregateTarget {
                target: request.target,
            });
        }
    }
    let validity_seconds = if request.validity_seconds == 0 {
        VALIDITY_DEFAULT_SECONDS
    } else {
        request.validity_seconds
    };
    if validity_seconds > VALIDITY_MAXIMUM_SECONDS {
        return Err(ValidateRefusal::ValidityOverMaximum {
            requested: validity_seconds,
        });
    }
    let identity = PlanIdentity {
        plan_id: request.plan_id.clone(),
        created_at: now,
        validity: ValidityWindow {
            not_after: now.saturating_add(validity_seconds),
        },
    };
    let planned = plan(
        PlanRequest {
            operation: request.operation,
            target: request.target,
        },
        capture,
        limits,
        runtime,
        &identity,
    )
    .map_err(ValidateRefusal::Planner)?;
    let body_value = planned
        .plan
        .body_value()
        .map_err(|_| ValidateRefusal::Encoding)?;
    let body_bytes = canonical::encode(&body_value).map_err(|_| ValidateRefusal::Encoding)?;
    let body_hash = planned
        .plan
        .body_hash()
        .map_err(|_| ValidateRefusal::Encoding)?;
    Ok(ValidatedPlan {
        body_bytes,
        body_hash,
        snapshot_hash: *planned.plan.snapshot_hash(),
        severity: planned.plan.severity(),
        flags: plan_flags(&planned.plan),
        consequences: planned.plan.consequences().to_vec(),
        not_after: planned.plan.validity().not_after,
    })
}

/// One validation's record, as increment 3's journal-backed act will hold
/// it: the hash the act names, the user it was validated for, and whether
/// its one apply is spent (ADR-0028's one act, one apply).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationRecord {
    /// The validated plan's body hash.
    pub plan_hash: Hash,
    /// The RPC-001-authenticated user the validation answered.
    pub validated_for_uid: u32,
    /// Whether the record's one apply has been consumed.
    pub consumed: bool,
}

/// SEC-002's typed arms, in the order they are checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmissionRefusal {
    /// The record's one apply is spent: a replay.
    Replayed,
    /// Presented by a user other than the one it was validated for.
    CrossUser {
        /// The presenting peer's uid.
        presented_by: u32,
        /// The record's uid.
        validated_for: u32,
    },
    /// The presented bytes are not the validated plan: their hash is not
    /// the record's.
    HashMismatch,
    /// PLAN-006: the plan binds a snapshot other than the fresh capture —
    /// the topology moved, or the plan was cross-device from the start.
    Stale,
    /// A bound identity contradicts the fresh capture's authored facts
    /// (ADR-0014's stamp rule): a cross-device presentation.
    CrossDevice,
    /// The bytes fail the decode boundary any other way: altered.
    Altered {
        /// The boundary's refusal, as its debug name.
        boundary: String,
    },
    /// PLAN-007: the window has closed.
    Expired {
        /// The body's own end.
        not_after: u64,
        /// The helper's clock.
        now: u64,
    },
}

/// Admit one presented plan against a fresh capture, the helper's clock,
/// the presenting peer, and its validation record — or refuse on the
/// first SEC-002 arm that bites. On success the decoded plan is returned
/// for the apply that consumes the record (increment 3's).
///
/// # Errors
///
/// [`AdmissionRefusal`].
pub fn admit_presented_plan(
    bytes: &[u8],
    fresh_capture: &TopologySnapshot,
    now: u64,
    presented_by: u32,
    record: &ValidationRecord,
) -> Result<OperationPlan, AdmissionRefusal> {
    if record.consumed {
        return Err(AdmissionRefusal::Replayed);
    }
    if presented_by != record.validated_for_uid {
        return Err(AdmissionRefusal::CrossUser {
            presented_by,
            validated_for: record.validated_for_uid,
        });
    }
    let plan =
        OperationPlan::from_canonical_body(bytes, fresh_capture).map_err(
            |refusal| match refusal {
                PlanSchemaError::SnapshotMismatch => AdmissionRefusal::Stale,
                PlanSchemaError::AuthoredFieldMismatch | PlanSchemaError::MalformedIdentity => {
                    AdmissionRefusal::CrossDevice
                }
                other => AdmissionRefusal::Altered {
                    boundary: format!("{other:?}"),
                },
            },
        )?;
    if plan.body_hash().map_err(|_| AdmissionRefusal::Altered {
        boundary: "Unhashable".to_owned(),
    })? != record.plan_hash
    {
        return Err(AdmissionRefusal::HashMismatch);
    }
    if plan.validity().not_after < now {
        return Err(AdmissionRefusal::Expired {
            not_after: plan.validity().not_after,
            now,
        });
    }
    Ok(plan)
}

/// The CAP-002 wire names, closed by the same test that closes the
/// domain's list: parse one, or say it is not an operation.
#[must_use]
pub fn parse_operation(name: &str) -> Option<Operation> {
    Some(match name {
        "detect" => Operation::Detect,
        "read" => Operation::Read,
        "create" => Operation::Create,
        "grow" => Operation::Grow,
        "shrink" => Operation::Shrink,
        "move" => Operation::Move,
        "copy" => Operation::Copy,
        "check" => Operation::Check,
        "repair" => Operation::Repair,
        "label" => Operation::Label,
        "uuid" => Operation::Uuid,
        "encrypt" => Operation::Encrypt,
        "decrypt" => Operation::Decrypt,
        "wipe" => Operation::Wipe,
        _ => return None,
    })
}

/// The severity's wire name.
#[must_use]
pub const fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Informational => "informational",
        Severity::Reversible => "reversible",
        Severity::Disruptive => "disruptive",
        Severity::DataMoving => "data-moving",
        Severity::Destructive => "destructive",
    }
}

/// The flag union's wire names, in PLAN-004's order.
#[must_use]
pub fn flag_names(flags: &StepFlags) -> Vec<&'static str> {
    let mut names = Vec::new();
    if flags.security_sensitive {
        names.push("security-sensitive");
    }
    if flags.irreversible_after_start {
        names.push("irreversible-after-start");
    }
    if flags.requires_offline {
        names.push("requires-offline");
    }
    if flags.requires_reboot {
        names.push("requires-reboot");
    }
    if flags.requires_rescue {
        names.push("requires-rescue");
    }
    names
}
