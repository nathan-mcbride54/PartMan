//! The operation plan's body skeleton and its validation boundary
//! (WP-010 increment 3i; Section 6, PLAN-004, PLAN-006, PLAN-007,
//! MODEL-005; ADR-0012's hand-forged-artifact refusal).
//!
//! The body carries the Section 6 items whose vocabularies are decided
//! today: schema identity, plan id, creation timestamp, the source
//! snapshot's body hash **as bound at validation** (8.0.0's rule), the
//! validity window (body deliberately — enforced, not re-derived, per
//! ADR-C2), and the ordered step graph as a semantic array (MODEL-006:
//! steps are a dependency order, not a set). The remaining Section 6
//! items — outcome text, consequence text, privileges, environment
//! requirements, backup actions, cancellation, capability versions,
//! reversal — land as their owning vocabularies (WP-050/WP-060) arrive.
//!
//! [`OperationPlan::from_canonical_body`] takes the plan bytes **and the
//! snapshot they claim to bind**, and re-runs every step through
//! [`PlanStep::mutating`] — the same closure, the same acknowledgment
//! law. A hand-forged artifact that bypassed the type layer is refused
//! here by recomputation: a tampered affected set, a smuggled
//! acknowledgment, or a step whose reach the closure refuses never
//! parses into a plan. That is ADR-0012's second verification row at
//! the boundary this crate owns; the helper's independent re-discovery
//! (HLP-002) supplies the fresh snapshot at validation.

use std::collections::BTreeMap;
use std::fmt;

use crate::canonical::{self, Hash, Value};

use super::identity::{DeviceIdentity, identity_from_map};
use super::naming::{self, NodeId};
use super::protection::{HostRange, StepRanges};
use super::snapshot::{SnapshotKind, TopologySnapshot};
use super::step::{
    Acknowledgment, PlanStep, Precondition, Severity, StepClass, StepFlags, StepRefusal, StepRisk,
};

/// The plan body's schema identity (MODEL-003).
pub const SCHEMA: &str = "partman.plan";
/// The unlinked plan body schema version — the pre-ADR-0022 form with
/// no reversal linkage and no step preconditions. Still emitted by
/// [`OperationPlan::assemble`] and accepted at the boundary until the
/// planner's reversal increment migrates every emitter to the linked
/// form; its retirement is its own reviewed change (MODEL-003's
/// explicit-migration discipline).
pub const SCHEMA_VERSION: u64 = 1;
/// The linked plan body schema version (ADR-0022, spec 12.0.0, and
/// ADR-0024, spec 12.2.0): Section 6's reversal-linkage item is
/// required, every step carries its preconditions, and every step
/// carries its typed class. Version 2 — the linked form without the
/// class field — lived for exactly one change window, gained no
/// emitter outside it and no surviving artifact, and is refused at
/// decode with its retirement recorded in the changelog (MODEL-003's
/// explicit-migration discipline).
pub const LINKED_SCHEMA_VERSION: u64 = 3;

/// PLAN-008's per-step reversal-impossibility reason — closed, typed,
/// free of free text (the JRN-005 discipline).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpossibilityReason {
    /// The step destroys data; no reversal restores destroyed bytes.
    DataDestroyed,
    /// The model carries no prior value to restore (labels,
    /// identifiers).
    PriorValueNotCarried,
    /// The pre-state is preserved as a raw protection capture
    /// (ADR-0024's repair arm); putting it back is REC-001's
    /// identity-validated recovery plan, never a planner-emitted
    /// reversal.
    PreStatePreservedForRecovery,
}

impl ImpossibilityReason {
    const fn wire_name(self) -> &'static str {
        match self {
            Self::DataDestroyed => "data-destroyed",
            Self::PriorValueNotCarried => "prior-value-not-carried",
            Self::PreStatePreservedForRecovery => "pre-state-preserved-for-recovery",
        }
    }
}

/// One step's machine-readable reversal-impossibility statement
/// (PLAN-008's second arm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepImpossibility {
    /// The step's index in the plan body's step order.
    pub step: usize,
    /// Why that step's reversal is impossible.
    pub reason: ImpossibilityReason,
}

/// Section 6's reversal-linkage body item (ADR-0022, resolving SI-19).
/// The reference asymmetry is acyclic **by construction**: the forward
/// side names the emitted draft by plan ID *and body hash* — freezing
/// what was advertised at authorization time — while the draft side
/// names the forward plan by plan ID *only*, because a hash reference
/// in both directions is unconstructible (each body's hash would depend
/// on the other's). A mutual-hash spelling has no variant to inhabit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReversalLinkage {
    /// A truthful reversal draft was emitted; the forward body carries
    /// its identity and draft-body hash.
    Draft {
        /// The draft's plan ID.
        plan_id: Vec<u8>,
        /// The draft's body hash at emission.
        draft_hash: Hash,
    },
    /// No truthful reversal exists; every step says why. Statements
    /// cover exactly the plan's step indices, ascending.
    Impossible {
        /// The per-step statements.
        statements: Vec<StepImpossibility>,
    },
    /// The draft's own linkage: its reversal is re-application of the
    /// forward plan, named by plan ID — a reference, not a third plan,
    /// which is how the regress terminates.
    ReapplyForward {
        /// The forward plan's ID.
        forward_plan_id: Vec<u8>,
    },
}

/// The plan's validity window (PLAN-007): body content deliberately —
/// the helper enforces it rather than re-deriving it, so an
/// unauthenticated expiry could otherwise be extended without
/// invalidating the authorization bound to the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidityWindow {
    /// Seconds since the epoch after which the plan is expired.
    pub not_after: u64,
}

/// The operation plan's hashed body skeleton.
#[derive(Debug, PartialEq, Eq)]
pub struct OperationPlan {
    plan_id: Vec<u8>,
    created_at: u64,
    snapshot_hash: Hash,
    validity: ValidityWindow,
    identities: BTreeMap<String, DeviceIdentity>,
    steps: Vec<PlanStep>,
    /// `None` is the version-1 body; `Some` is the linked version-2
    /// body carrying Section 6's reversal item.
    reversal: Option<ReversalLinkage>,
}

/// The ADR-0022 severity rule, shared by both assembly paths and the
/// boundary: a Reversible claim stands only on an emitted reversal —
/// the draft linkage, or the draft's own reapply-forward statement. No
/// linkage (version 1) or an impossibility statement forbids it.
fn reversible_backed(steps: &[PlanStep], reversal: Option<&ReversalLinkage>) -> bool {
    let claims_reversible = steps
        .iter()
        .any(|step| step.risk().severity == Severity::Reversible);
    if !claims_reversible {
        return true;
    }
    matches!(
        reversal,
        Some(ReversalLinkage::Draft { .. } | ReversalLinkage::ReapplyForward { .. })
    )
}

impl OperationPlan {
    /// Assemble a version-1 (unlinked) plan over steps already
    /// constructed against the given snapshot. The snapshot's body hash
    /// is bound into the plan (PLAN-006's comparison object, as bound
    /// at validation per 8.0.0).
    ///
    /// # Errors
    ///
    /// [`PlanError::Snapshot`] if the snapshot cannot hash;
    /// [`PlanError::ReversibleWithoutReversal`] for a Reversible step —
    /// severity 1 stands only on an emitted reversal (ADR-0022), which
    /// this body form cannot carry; [`PlanError::UncarriedPreconditions`]
    /// for a step with preconditions, which this body form would
    /// silently drop.
    pub fn assemble(
        plan_id: Vec<u8>,
        created_at: u64,
        snapshot: &TopologySnapshot,
        validity: ValidityWindow,
        identities: BTreeMap<NodeId, DeviceIdentity>,
        steps: Vec<PlanStep>,
    ) -> Result<Self, PlanError> {
        if !reversible_backed(&steps, None) {
            return Err(PlanError::ReversibleWithoutReversal);
        }
        if steps.iter().any(|step| !step.preconditions().is_empty()) {
            return Err(PlanError::UncarriedPreconditions);
        }
        let snapshot_hash = snapshot.body_hash().map_err(|_| PlanError::Snapshot)?;
        Ok(Self {
            plan_id,
            created_at,
            snapshot_hash,
            validity,
            identities: identities
                .into_iter()
                .map(|(id, identity)| (id.to_string(), identity))
                .collect(),
            steps,
            reversal: None,
        })
    }

    /// Assemble a linked (version-2) plan: Section 6's reversal item is
    /// required content (ADR-0022), and steps carry their preconditions
    /// in the body.
    ///
    /// # Errors
    ///
    /// As [`Self::assemble`], plus [`PlanError::MalformedLinkage`] when
    /// an impossibility statement set does not cover exactly the plan's
    /// step indices in ascending order, and
    /// [`PlanError::ReversibleWithoutReversal`] when a Reversible step
    /// rides an impossibility linkage.
    pub fn assemble_linked(
        plan_id: Vec<u8>,
        created_at: u64,
        snapshot: &TopologySnapshot,
        validity: ValidityWindow,
        identities: BTreeMap<NodeId, DeviceIdentity>,
        steps: Vec<PlanStep>,
        reversal: ReversalLinkage,
    ) -> Result<Self, PlanError> {
        if let ReversalLinkage::Impossible { statements } = &reversal {
            let covers_exactly = statements.len() == steps.len()
                && statements
                    .iter()
                    .enumerate()
                    .all(|(index, statement)| statement.step == index);
            if !covers_exactly {
                return Err(PlanError::MalformedLinkage);
            }
        }
        if !reversible_backed(&steps, Some(&reversal)) {
            return Err(PlanError::ReversibleWithoutReversal);
        }
        let snapshot_hash = snapshot.body_hash().map_err(|_| PlanError::Snapshot)?;
        Ok(Self {
            plan_id,
            created_at,
            snapshot_hash,
            validity,
            identities: identities
                .into_iter()
                .map(|(id, identity)| (id.to_string(), identity))
                .collect(),
            steps,
            reversal: Some(reversal),
        })
    }

    /// The plan's identifier bytes.
    #[must_use]
    pub fn plan_id(&self) -> &[u8] {
        &self.plan_id
    }

    /// The reversal linkage — `None` on a version-1 body.
    #[must_use]
    pub const fn reversal(&self) -> Option<&ReversalLinkage> {
        self.reversal.as_ref()
    }

    /// The bound identities, keyed by the target's address in hex
    /// (Section 6: complete bound device identities; strength is derived
    /// from each record, never stored).
    #[must_use]
    pub const fn identities(&self) -> &BTreeMap<String, DeviceIdentity> {
        &self.identities
    }

    /// The bound snapshot body hash (PLAN-006).
    #[must_use]
    pub const fn snapshot_hash(&self) -> &Hash {
        &self.snapshot_hash
    }

    /// The plan's steps, in dependency order.
    #[must_use]
    pub fn steps(&self) -> &[PlanStep] {
        &self.steps
    }

    /// PLAN-004: plan severity is the maximum step severity.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.steps
            .iter()
            .map(|step| step.risk().severity)
            .max()
            .unwrap_or(Severity::Informational)
    }

    /// The body as a canonical value.
    ///
    /// # Errors
    ///
    /// [`PlanError::Encoding`] on an unencodable element — unreachable
    /// for a plan this module assembled.
    pub fn body_value(&self) -> Result<Value, PlanError> {
        let linked = self.reversal.is_some();
        let mut body = BTreeMap::new();
        body.insert("schema".to_owned(), Value::Text(SCHEMA.to_owned()));
        body.insert(
            "schema_version".to_owned(),
            Value::Unsigned(if linked {
                LINKED_SCHEMA_VERSION
            } else {
                SCHEMA_VERSION
            }),
        );
        body.insert("plan_id".to_owned(), Value::Bytes(self.plan_id.clone()));
        body.insert("created_at".to_owned(), Value::Unsigned(self.created_at));
        body.insert(
            "snapshot_hash".to_owned(),
            Value::Bytes(self.snapshot_hash.as_bytes().to_vec()),
        );
        body.insert(
            "not_after".to_owned(),
            Value::Unsigned(self.validity.not_after),
        );
        body.insert(
            "identities".to_owned(),
            Value::Map(
                self.identities
                    .iter()
                    .map(|(key, identity)| (key.clone(), identity.body_value()))
                    .collect(),
            ),
        );
        body.insert(
            "steps".to_owned(),
            Value::Array(
                self.steps
                    .iter()
                    .map(|step| step_value(step, linked))
                    .collect(),
            ),
        );
        if let Some(reversal) = &self.reversal {
            body.insert("reversal".to_owned(), reversal_value(reversal));
        }
        Ok(Value::Map(body))
    }

    /// The plan body hash (MODEL-005): what HLP-003 binds authorization
    /// to and SEC-001 authorizes exactly.
    ///
    /// # Errors
    ///
    /// As [`Self::body_value`], plus encoding failure.
    pub fn body_hash(&self) -> Result<Hash, PlanError> {
        self.body_value()
            .and_then(|body| canonical::hash(&body).map_err(|_| PlanError::Encoding))
    }

    /// The validation boundary: rebuild a plan from body bytes against
    /// the snapshot it claims to bind, re-running every step through the
    /// sole constructor. A hand-forged artifact — tampered affected
    /// set, smuggled acknowledgment, or a reach the closure refuses —
    /// never parses.
    ///
    /// # Errors
    ///
    /// [`PlanSchemaError`] naming the first rule violated.
    pub fn from_canonical_body(
        bytes: &[u8],
        snapshot: &TopologySnapshot,
    ) -> Result<Self, PlanSchemaError> {
        // A prediction is not a capture, structurally: no plan body of
        // any version accepts a simulated snapshot as its binding base
        // (the 3c rule restated where a naive caller could otherwise
        // satisfy the hash equality with the prediction itself).
        if snapshot.kind() == SnapshotKind::Simulated {
            return Err(PlanSchemaError::PredictionNeverBinds);
        }
        let value = canonical::decode(bytes).map_err(PlanSchemaError::Codec)?;
        let Value::Map(map) = value else {
            return Err(PlanSchemaError::NotABodyMap);
        };
        match map.get("schema") {
            Some(Value::Text(text)) if text == SCHEMA => {}
            _ => return Err(PlanSchemaError::WrongSchema),
        }
        let linked = match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == SCHEMA_VERSION => false,
            Some(Value::Unsigned(version)) if *version == LINKED_SCHEMA_VERSION => true,
            _ => return Err(PlanSchemaError::WrongSchemaVersion),
        };
        for key in map.keys() {
            let known = matches!(
                key.as_str(),
                "schema"
                    | "schema_version"
                    | "plan_id"
                    | "created_at"
                    | "snapshot_hash"
                    | "not_after"
                    | "identities"
                    | "steps"
            ) || (linked && key.as_str() == "reversal");
            if !known {
                return Err(PlanSchemaError::UnknownField { key: key.clone() });
            }
        }
        let (plan_id, created_at, not_after) = parse_header_scalars(&map)?;
        let claimed_hash = match map.get("snapshot_hash") {
            Some(Value::Bytes(bytes)) => bytes.clone(),
            _ => {
                return Err(PlanSchemaError::MissingField {
                    key: "snapshot_hash",
                });
            }
        };
        let actual = snapshot
            .body_hash()
            .map_err(|_| PlanSchemaError::SnapshotUnhashable)?;
        if claimed_hash != actual.as_bytes() {
            return Err(PlanSchemaError::SnapshotMismatch);
        }
        let identities = parse_identities(&map, snapshot)?;
        let Some(Value::Array(step_values)) = map.get("steps") else {
            return Err(PlanSchemaError::MissingField { key: "steps" });
        };
        let mut steps = Vec::new();
        for step_value in step_values {
            steps.push(parse_step(step_value, snapshot, linked)?);
        }
        let reversal = if linked {
            match map.get("reversal") {
                Some(value) => Some(parse_reversal(value)?),
                None => return Err(PlanSchemaError::MissingField { key: "reversal" }),
            }
        } else {
            None
        };
        if let Some(ReversalLinkage::Impossible { statements }) = &reversal {
            let covers_exactly = statements.len() == steps.len()
                && statements
                    .iter()
                    .enumerate()
                    .all(|(index, statement)| statement.step == index);
            if !covers_exactly {
                return Err(PlanSchemaError::MalformedLinkage);
            }
        }
        if !reversible_backed(&steps, reversal.as_ref()) {
            return Err(PlanSchemaError::ReversibleWithoutReversal);
        }
        // The two-time truthfulness re-check (ADR-0022): a precondition
        // is body content, judged here against the snapshot the plan
        // binds — a claim that decayed refuses instead of silently
        // becoming a different plan.
        for step in &steps {
            for precondition in step.preconditions() {
                if let Some(node) = precondition.violated_by(snapshot) {
                    return Err(PlanSchemaError::PreconditionFailed { node });
                }
            }
        }
        let rebuilt = Self {
            plan_id,
            created_at,
            snapshot_hash: actual,
            validity: ValidityWindow { not_after },
            identities,
            steps,
            reversal,
        };
        let recomputed = rebuilt
            .body_value()
            .and_then(|body| canonical::encode(&body).map_err(|_| PlanError::Encoding))
            .map_err(|_| PlanSchemaError::SnapshotUnhashable)?;
        if recomputed != bytes {
            return Err(PlanSchemaError::RecomputationMismatch);
        }
        Ok(rebuilt)
    }
}

fn parse_header_scalars(
    map: &BTreeMap<String, Value>,
) -> Result<(Vec<u8>, u64, u64), PlanSchemaError> {
    let plan_id = match map.get("plan_id") {
        Some(Value::Bytes(bytes)) => bytes.clone(),
        _ => return Err(PlanSchemaError::MissingField { key: "plan_id" }),
    };
    let created_at = match map.get("created_at") {
        Some(Value::Unsigned(value)) => *value,
        _ => return Err(PlanSchemaError::MissingField { key: "created_at" }),
    };
    let not_after = match map.get("not_after") {
        Some(Value::Unsigned(value)) => *value,
        _ => return Err(PlanSchemaError::MissingField { key: "not_after" }),
    };
    Ok((plan_id, created_at, not_after))
}

fn parse_identities(
    map: &BTreeMap<String, Value>,
    snapshot: &TopologySnapshot,
) -> Result<BTreeMap<String, DeviceIdentity>, PlanSchemaError> {
    let Some(Value::Map(identity_values)) = map.get("identities") else {
        return Err(PlanSchemaError::MissingField { key: "identities" });
    };
    let mut identities = BTreeMap::new();
    for (key, value) in identity_values {
        let Value::Map(identity_map) = value else {
            return Err(PlanSchemaError::MalformedIdentity);
        };
        let identity =
            identity_from_map(identity_map).map_err(|_| PlanSchemaError::MalformedIdentity)?;
        // The authored-field rule (ADR-0014, MODEL-005's authoring
        // set): where the helper-produced snapshot carries a table
        // state for this device, a plan identity claiming a
        // different state never validates. The snapshot's fact is
        // the stamp; the plan's claim must agree or the plan is a
        // client-authored divergence.
        let id_bytes = hex_to_id(key).ok_or(PlanSchemaError::MalformedIdentity)?;
        if let Some(stamped) = snapshot.facts().table_states.get(&id_bytes)
            && *stamped != identity.table
        {
            return Err(PlanSchemaError::AuthoredFieldMismatch);
        }
        identities.insert(key.clone(), identity);
    }
    Ok(identities)
}

fn reversal_value(reversal: &ReversalLinkage) -> Value {
    let mut map = BTreeMap::new();
    match reversal {
        ReversalLinkage::Draft {
            plan_id,
            draft_hash,
        } => {
            map.insert("kind".to_owned(), Value::Text("draft".to_owned()));
            map.insert("plan_id".to_owned(), Value::Bytes(plan_id.clone()));
            map.insert(
                "hash".to_owned(),
                Value::Bytes(draft_hash.as_bytes().to_vec()),
            );
        }
        ReversalLinkage::Impossible { statements } => {
            map.insert("kind".to_owned(), Value::Text("impossible".to_owned()));
            map.insert(
                "statements".to_owned(),
                Value::Array(
                    statements
                        .iter()
                        .map(|statement| {
                            let mut entry = BTreeMap::new();
                            entry.insert(
                                "step".to_owned(),
                                Value::Unsigned(
                                    u64::try_from(statement.step)
                                        .expect("a step index fits in a u64"),
                                ),
                            );
                            entry.insert(
                                "reason".to_owned(),
                                Value::Text(statement.reason.wire_name().to_owned()),
                            );
                            Value::Map(entry)
                        })
                        .collect(),
                ),
            );
        }
        ReversalLinkage::ReapplyForward { forward_plan_id } => {
            map.insert("kind".to_owned(), Value::Text("reapply-forward".to_owned()));
            map.insert("plan_id".to_owned(), Value::Bytes(forward_plan_id.clone()));
        }
    }
    Value::Map(map)
}

fn parse_reversal(value: &Value) -> Result<ReversalLinkage, PlanSchemaError> {
    let Value::Map(map) = value else {
        return Err(PlanSchemaError::MalformedLinkage);
    };
    let Some(Value::Text(kind)) = map.get("kind") else {
        return Err(PlanSchemaError::MalformedLinkage);
    };
    let expect_keys = |allowed: &[&str]| -> Result<(), PlanSchemaError> {
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                return Err(PlanSchemaError::UnknownField { key: key.clone() });
            }
        }
        Ok(())
    };
    match kind.as_str() {
        "draft" => {
            expect_keys(&["kind", "plan_id", "hash"])?;
            let Some(Value::Bytes(plan_id)) = map.get("plan_id") else {
                return Err(PlanSchemaError::MalformedLinkage);
            };
            let Some(Value::Bytes(hash_bytes)) = map.get("hash") else {
                return Err(PlanSchemaError::MalformedLinkage);
            };
            let recorded: [u8; 32] = hash_bytes
                .as_slice()
                .try_into()
                .map_err(|_| PlanSchemaError::MalformedLinkage)?;
            let draft_hash = Hash::from_bytes(recorded);
            Ok(ReversalLinkage::Draft {
                plan_id: plan_id.clone(),
                draft_hash,
            })
        }
        "impossible" => {
            expect_keys(&["kind", "statements"])?;
            let Some(Value::Array(entries)) = map.get("statements") else {
                return Err(PlanSchemaError::MalformedLinkage);
            };
            let mut statements = Vec::with_capacity(entries.len());
            for entry in entries {
                let Value::Map(entry) = entry else {
                    return Err(PlanSchemaError::MalformedLinkage);
                };
                for key in entry.keys() {
                    if !matches!(key.as_str(), "step" | "reason") {
                        return Err(PlanSchemaError::UnknownField { key: key.clone() });
                    }
                }
                let Some(Value::Unsigned(step)) = entry.get("step") else {
                    return Err(PlanSchemaError::MalformedLinkage);
                };
                let reason = match entry.get("reason") {
                    Some(Value::Text(reason)) => match reason.as_str() {
                        "data-destroyed" => ImpossibilityReason::DataDestroyed,
                        "prior-value-not-carried" => ImpossibilityReason::PriorValueNotCarried,
                        "pre-state-preserved-for-recovery" => {
                            ImpossibilityReason::PreStatePreservedForRecovery
                        }
                        _ => return Err(PlanSchemaError::MalformedLinkage),
                    },
                    _ => return Err(PlanSchemaError::MalformedLinkage),
                };
                statements.push(StepImpossibility {
                    step: usize::try_from(*step).map_err(|_| PlanSchemaError::MalformedLinkage)?,
                    reason,
                });
            }
            Ok(ReversalLinkage::Impossible { statements })
        }
        "reapply-forward" => {
            expect_keys(&["kind", "plan_id"])?;
            let Some(Value::Bytes(plan_id)) = map.get("plan_id") else {
                return Err(PlanSchemaError::MalformedLinkage);
            };
            Ok(ReversalLinkage::ReapplyForward {
                forward_plan_id: plan_id.clone(),
            })
        }
        _ => Err(PlanSchemaError::MalformedLinkage),
    }
}

fn precondition_value(precondition: &Precondition) -> Value {
    let mut map = BTreeMap::new();
    match precondition {
        Precondition::RegionUnoccupied { region } => {
            map.insert(
                "kind".to_owned(),
                Value::Text("region-unoccupied".to_owned()),
            );
            map.insert(
                "host".to_owned(),
                Value::Bytes(region.host.as_bytes().to_vec()),
            );
            map.insert("start".to_owned(), Value::Unsigned(region.start));
            map.insert("length".to_owned(), Value::Unsigned(region.length));
        }
        Precondition::HostUnoccupied { host } => {
            map.insert("kind".to_owned(), Value::Text("host-unoccupied".to_owned()));
            map.insert("host".to_owned(), Value::Bytes(host.as_bytes().to_vec()));
        }
    }
    Value::Map(map)
}

fn parse_precondition(value: &Value) -> Result<Precondition, PlanSchemaError> {
    let Value::Map(map) = value else {
        return Err(PlanSchemaError::MalformedStep);
    };
    let Some(Value::Text(kind)) = map.get("kind") else {
        return Err(PlanSchemaError::MalformedStep);
    };
    match kind.as_str() {
        "region-unoccupied" => {
            for key in map.keys() {
                if !matches!(key.as_str(), "kind" | "host" | "start" | "length") {
                    return Err(PlanSchemaError::UnknownField { key: key.clone() });
                }
            }
            let host = parse_node(map.get("host"))?;
            let start = match map.get("start") {
                Some(Value::Unsigned(value)) => *value,
                _ => return Err(PlanSchemaError::MalformedStep),
            };
            let length = match map.get("length") {
                Some(Value::Unsigned(value)) => *value,
                _ => return Err(PlanSchemaError::MalformedStep),
            };
            Ok(Precondition::RegionUnoccupied {
                region: HostRange {
                    host,
                    start,
                    length,
                },
            })
        }
        "host-unoccupied" => {
            for key in map.keys() {
                if !matches!(key.as_str(), "kind" | "host") {
                    return Err(PlanSchemaError::UnknownField { key: key.clone() });
                }
            }
            Ok(Precondition::HostUnoccupied {
                host: parse_node(map.get("host"))?,
            })
        }
        _ => Err(PlanSchemaError::MalformedStep),
    }
}

fn preconditions_value(preconditions: &[Precondition]) -> Value {
    Value::Array(preconditions.iter().map(precondition_value).collect())
}

fn draft_precondition_value(precondition: &DraftPrecondition) -> Value {
    match precondition {
        DraftPrecondition::Carried(carried) => precondition_value(carried),
        DraftPrecondition::StepOutputUnoccupied { step } => {
            let mut map = BTreeMap::new();
            map.insert(
                "kind".to_owned(),
                Value::Text("step-output-unoccupied".to_owned()),
            );
            map.insert(
                "step".to_owned(),
                Value::Unsigned(u64::try_from(*step).expect("a step index fits in a u64")),
            );
            Value::Map(map)
        }
    }
}

fn parse_draft_precondition(value: &Value) -> Result<DraftPrecondition, PlanSchemaError> {
    if let Value::Map(map) = value
        && let Some(Value::Text(kind)) = map.get("kind")
        && kind == "step-output-unoccupied"
    {
        for key in map.keys() {
            if !matches!(key.as_str(), "kind" | "step") {
                return Err(PlanSchemaError::UnknownField { key: key.clone() });
            }
        }
        let Some(Value::Unsigned(step)) = map.get("step") else {
            return Err(PlanSchemaError::MalformedStep);
        };
        return Ok(DraftPrecondition::StepOutputUnoccupied {
            step: usize::try_from(*step).map_err(|_| PlanSchemaError::MalformedStep)?,
        });
    }
    parse_precondition(value).map(DraftPrecondition::Carried)
}

const fn class_wire_name(class: StepClass) -> &'static str {
    match class {
        StepClass::Ordinary => "ordinary",
        StepClass::TableRepair => "table-repair",
    }
}

fn parse_class(map: &BTreeMap<String, Value>) -> Result<StepClass, PlanSchemaError> {
    match map.get("class") {
        Some(Value::Text(class)) => match class.as_str() {
            "ordinary" => Ok(StepClass::Ordinary),
            "table-repair" => Ok(StepClass::TableRepair),
            _ => Err(PlanSchemaError::MalformedStep),
        },
        _ => Err(PlanSchemaError::MalformedStep),
    }
}

fn step_value(step: &PlanStep, linked: bool) -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "target".to_owned(),
        Value::Bytes(step.target().as_bytes().to_vec()),
    );
    if linked {
        map.insert(
            "preconditions".to_owned(),
            preconditions_value(step.preconditions()),
        );
        map.insert(
            "class".to_owned(),
            Value::Text(class_wire_name(step.class()).to_owned()),
        );
    }
    map.insert(
        "written_table_extents".to_owned(),
        ranges_value(&step.ranges().written_table_extents),
    );
    map.insert("consumed".to_owned(), ranges_value(&step.ranges().consumed));
    map.insert(
        "destroyed".to_owned(),
        ranges_value(&step.ranges().destroyed),
    );
    map.insert(
        "acknowledgments".to_owned(),
        Value::Array(
            step.acknowledgments()
                .iter()
                .map(acknowledgment_value)
                .collect(),
        ),
    );
    insert_risk(&mut map, step.risk());
    Value::Map(map)
}

fn insert_risk(map: &mut BTreeMap<String, Value>, risk: StepRisk) {
    map.insert(
        "severity".to_owned(),
        Value::Unsigned(match risk.severity {
            Severity::Informational => 0,
            Severity::Reversible => 1,
            Severity::Disruptive => 2,
            Severity::DataMoving => 3,
            Severity::Destructive => 4,
        }),
    );
    let flags = risk.flags;
    map.insert(
        "flags".to_owned(),
        Value::Array(
            [
                ("security-sensitive", flags.security_sensitive),
                ("irreversible-after-start", flags.irreversible_after_start),
                ("requires-offline", flags.requires_offline),
                ("requires-reboot", flags.requires_reboot),
                ("requires-rescue", flags.requires_rescue),
            ]
            .iter()
            .filter(|(_, set)| *set)
            .map(|(name, _)| Value::Text((*name).to_owned()))
            .collect(),
        ),
    );
}

fn parse_risk(map: &BTreeMap<String, Value>) -> Result<StepRisk, PlanSchemaError> {
    let severity = match map.get("severity") {
        Some(Value::Unsigned(0)) => Severity::Informational,
        Some(Value::Unsigned(1)) => Severity::Reversible,
        Some(Value::Unsigned(2)) => Severity::Disruptive,
        Some(Value::Unsigned(3)) => Severity::DataMoving,
        Some(Value::Unsigned(4)) => Severity::Destructive,
        _ => return Err(PlanSchemaError::MalformedStep),
    };
    let mut flags = StepFlags::default();
    let Some(Value::Array(flag_values)) = map.get("flags") else {
        return Err(PlanSchemaError::MalformedStep);
    };
    for flag in flag_values {
        match flag {
            Value::Text(name) => match name.as_str() {
                "security-sensitive" => flags.security_sensitive = true,
                "irreversible-after-start" => flags.irreversible_after_start = true,
                "requires-offline" => flags.requires_offline = true,
                "requires-reboot" => flags.requires_reboot = true,
                "requires-rescue" => flags.requires_rescue = true,
                _ => return Err(PlanSchemaError::MalformedStep),
            },
            _ => return Err(PlanSchemaError::MalformedStep),
        }
    }
    Ok(StepRisk { severity, flags })
}

fn ranges_value(ranges: &[HostRange]) -> Value {
    Value::Array(
        ranges
            .iter()
            .map(|range| {
                let mut map = BTreeMap::new();
                map.insert(
                    "host".to_owned(),
                    Value::Bytes(range.host.as_bytes().to_vec()),
                );
                map.insert("start".to_owned(), Value::Unsigned(range.start));
                map.insert("length".to_owned(), Value::Unsigned(range.length));
                Value::Map(map)
            })
            .collect(),
    )
}

fn acknowledgment_value(acknowledgment: &Acknowledgment) -> Value {
    let mut map = BTreeMap::new();
    let (kind, node) = match acknowledgment {
        Acknowledgment::Release { signature } => ("release", signature),
        Acknowledgment::OpaqueDestruction { layer } => ("opaque-destruction", layer),
        Acknowledgment::IdentityBoundRestore { table } => ("identity-bound-restore", table),
        Acknowledgment::UncapturableRegions { table, regions } => {
            map.insert(
                "regions".to_owned(),
                Value::Array(
                    regions
                        .iter()
                        .map(|region| {
                            let mut entry = BTreeMap::new();
                            entry.insert("start".to_owned(), Value::Unsigned(region.start));
                            entry.insert("length".to_owned(), Value::Unsigned(region.length));
                            Value::Map(entry)
                        })
                        .collect(),
                ),
            );
            ("uncapturable-regions", table)
        }
    };
    map.insert("kind".to_owned(), Value::Text(kind.to_owned()));
    map.insert("node".to_owned(), Value::Bytes(node.as_bytes().to_vec()));
    Value::Map(map)
}

fn parse_step(
    value: &Value,
    snapshot: &TopologySnapshot,
    linked: bool,
) -> Result<PlanStep, PlanSchemaError> {
    let Value::Map(map) = value else {
        return Err(PlanSchemaError::MalformedStep);
    };
    // The step-output spelling is a draft's alone: a bound plan names
    // devices by address, and a reference that survived into this
    // boundary is a draft presented where a plan is required.
    if map.contains_key("target_step_output") {
        return Err(PlanSchemaError::DraftSpellingOutsideDraft);
    }
    for key in map.keys() {
        let known = matches!(
            key.as_str(),
            "target"
                | "written_table_extents"
                | "consumed"
                | "destroyed"
                | "acknowledgments"
                | "severity"
                | "flags"
        ) || (linked && matches!(key.as_str(), "preconditions" | "class"));
        if !known {
            return Err(PlanSchemaError::UnknownField { key: key.clone() });
        }
    }
    let target = parse_node(map.get("target"))?;
    let (preconditions, class) = if linked {
        let Some(Value::Array(entries)) = map.get("preconditions") else {
            return Err(PlanSchemaError::MalformedStep);
        };
        (
            entries
                .iter()
                .map(parse_precondition)
                .collect::<Result<Vec<_>, _>>()?,
            parse_class(map)?,
        )
    } else {
        (vec![], StepClass::Ordinary)
    };
    let ranges = StepRanges {
        written_table_extents: parse_ranges(map.get("written_table_extents"))?,
        consumed: parse_ranges(map.get("consumed"))?,
        destroyed: parse_ranges(map.get("destroyed"))?,
    };
    let Some(Value::Array(acknowledgment_values)) = map.get("acknowledgments") else {
        return Err(PlanSchemaError::MalformedStep);
    };
    let mut acknowledgments = Vec::new();
    for value in acknowledgment_values {
        acknowledgments.push(parse_acknowledgment(value)?);
    }
    let risk = parse_risk(map)?;
    // The recompute: the sole constructor runs the closure and the
    // acknowledgment law — the class-conditioned law included — over
    // the snapshot's authenticated facts. A forged step never returns.
    PlanStep::mutating_classed(snapshot, target, ranges, acknowledgments, risk, class)
        .map(|step| step.with_preconditions(preconditions))
        .map_err(PlanSchemaError::Step)
}

fn hex_to_id(text: &str) -> Option<NodeId> {
    if text.len() != 64 {
        return None;
    }
    let mut bytes = Vec::with_capacity(32);
    let mut chars = text.chars();
    while let (Some(high), Some(low)) = (chars.next(), chars.next()) {
        let value = u8::from_str_radix(&format!("{high}{low}"), 16).ok()?;
        bytes.push(value);
    }
    naming::id_from_bytes(&bytes)
}

fn parse_node(value: Option<&Value>) -> Result<NodeId, PlanSchemaError> {
    match value {
        Some(Value::Bytes(bytes)) => {
            naming::id_from_bytes(bytes).ok_or(PlanSchemaError::MalformedStep)
        }
        _ => Err(PlanSchemaError::MalformedStep),
    }
}

fn parse_ranges(value: Option<&Value>) -> Result<Vec<HostRange>, PlanSchemaError> {
    let Some(Value::Array(entries)) = value else {
        return Err(PlanSchemaError::MalformedStep);
    };
    let mut ranges = Vec::new();
    for entry in entries {
        let Value::Map(map) = entry else {
            return Err(PlanSchemaError::MalformedStep);
        };
        let host = parse_node(map.get("host"))?;
        let start = match map.get("start") {
            Some(Value::Unsigned(value)) => *value,
            _ => return Err(PlanSchemaError::MalformedStep),
        };
        let length = match map.get("length") {
            Some(Value::Unsigned(value)) => *value,
            _ => return Err(PlanSchemaError::MalformedStep),
        };
        ranges.push(HostRange {
            host,
            start,
            length,
        });
    }
    Ok(ranges)
}

fn parse_acknowledgment(value: &Value) -> Result<Acknowledgment, PlanSchemaError> {
    let Value::Map(map) = value else {
        return Err(PlanSchemaError::MalformedStep);
    };
    let node = parse_node(map.get("node"))?;
    match map.get("kind") {
        Some(Value::Text(kind)) => Ok(match kind.as_str() {
            "release" => Acknowledgment::Release { signature: node },
            "opaque-destruction" => Acknowledgment::OpaqueDestruction { layer: node },
            "identity-bound-restore" => Acknowledgment::IdentityBoundRestore { table: node },
            "uncapturable-regions" => {
                let Some(Value::Array(entries)) = map.get("regions") else {
                    return Err(PlanSchemaError::MalformedStep);
                };
                let mut regions = Vec::with_capacity(entries.len());
                for entry in entries {
                    let Value::Map(entry) = entry else {
                        return Err(PlanSchemaError::MalformedStep);
                    };
                    let start = match entry.get("start") {
                        Some(Value::Unsigned(value)) => *value,
                        _ => return Err(PlanSchemaError::MalformedStep),
                    };
                    let length = match entry.get("length") {
                        Some(Value::Unsigned(value)) => *value,
                        _ => return Err(PlanSchemaError::MalformedStep),
                    };
                    regions.push(HostRange {
                        host: node,
                        start,
                        length,
                    });
                }
                Acknowledgment::UncapturableRegions {
                    table: node,
                    regions,
                }
            }
            _ => return Err(PlanSchemaError::MalformedStep),
        }),
        _ => Err(PlanSchemaError::MalformedStep),
    }
}

/// A plan assembly or encoding failure.
#[derive(Debug, PartialEq, Eq)]
pub enum PlanError {
    /// The snapshot could not hash.
    Snapshot,
    /// The canonical encoder refused body content.
    Encoding,
    /// A Reversible step with no emitted reversal to stand on
    /// (ADR-0022: no draft, no Reversible).
    ReversibleWithoutReversal,
    /// A step carries preconditions the unlinked body form would
    /// silently drop; assemble the linked form instead.
    UncarriedPreconditions,
    /// An impossibility statement set that does not cover exactly the
    /// plan's step indices in ascending order.
    MalformedLinkage,
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot => formatter.write_str("snapshot not hashable"),
            Self::Encoding => formatter.write_str("plan body not encodable"),
            Self::ReversibleWithoutReversal => formatter
                .write_str("a Reversible step stands only on an emitted reversal (ADR-0022)"),
            Self::UncarriedPreconditions => formatter
                .write_str("preconditions need the linked body form; the unlinked form drops them"),
            Self::MalformedLinkage => formatter
                .write_str("impossibility statements must cover exactly the plan's steps in order"),
        }
    }
}

impl std::error::Error for PlanError {}

/// The plan boundary's error type, distinct from the codec's and the
/// snapshot's.
#[derive(Debug, PartialEq, Eq)]
pub enum PlanSchemaError {
    /// Not canonical `pce/1`.
    Codec(canonical::Error),
    /// Not a body map.
    NotABodyMap,
    /// An undeclared body or step field.
    UnknownField {
        /// The undeclared key.
        key: String,
    },
    /// Not this schema.
    WrongSchema,
    /// Not this schema version.
    WrongSchemaVersion,
    /// A required field is missing or mistyped.
    MissingField {
        /// The field's key.
        key: &'static str,
    },
    /// The bound snapshot hash does not match the supplied snapshot —
    /// the ACC-007 shape at the type layer.
    SnapshotMismatch,
    /// The supplied snapshot could not hash.
    SnapshotUnhashable,
    /// A step, range, acknowledgment, severity, or flag is malformed.
    MalformedStep,
    /// A bound identity does not parse, or its key is not an address.
    MalformedIdentity,
    /// A plan identity's authored field disagrees with the snapshot's
    /// stamp — the client-authored value that never validates
    /// (ADR-0014, MODEL-005's authoring set).
    AuthoredFieldMismatch,
    /// A step failed recomputation through the sole constructor — the
    /// hand-forged artifact, refused.
    Step(StepRefusal),
    /// The rebuilt body does not reproduce the input bytes.
    RecomputationMismatch,
    /// The supplied binding snapshot is a simulated topology — a
    /// prediction proposes and never binds (the 3c rule at this
    /// boundary).
    PredictionNeverBinds,
    /// The reversal linkage is malformed: an unknown kind, a mistyped
    /// field, or an impossibility set that does not cover the steps.
    MalformedLinkage,
    /// A Reversible step with no emitted reversal to stand on
    /// (ADR-0022: no draft, no Reversible).
    ReversibleWithoutReversal,
    /// A step precondition fails against the binding snapshot — the
    /// two-time truthfulness re-check refusing a decayed claim.
    PreconditionFailed {
        /// The node violating the precondition.
        node: NodeId,
    },
    /// A step spelled its target as a step-output reference, which only
    /// a reversal draft may carry; a bound plan names addresses.
    DraftSpellingOutsideDraft,
}

impl fmt::Display for PlanSchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Codec(error) => write!(formatter, "not canonical pce/1: {error}"),
            Self::NotABodyMap => formatter.write_str("plan body is not a map"),
            Self::UnknownField { key } => write!(formatter, "undeclared field `{key}`"),
            Self::WrongSchema => formatter.write_str("unknown plan schema"),
            Self::WrongSchemaVersion => formatter.write_str("unsupported plan schema version"),
            Self::MissingField { key } => write!(formatter, "missing field `{key}`"),
            Self::SnapshotMismatch => {
                formatter.write_str("bound snapshot hash does not match the supplied snapshot")
            }
            Self::SnapshotUnhashable => formatter.write_str("supplied snapshot not hashable"),
            Self::MalformedStep => formatter.write_str("malformed step"),
            Self::MalformedIdentity => formatter.write_str("malformed bound identity"),
            Self::AuthoredFieldMismatch => formatter
                .write_str("a bound identity's authored field disagrees with the snapshot's stamp"),
            Self::Step(refusal) => write!(formatter, "step recomputation refused: {refusal}"),
            Self::RecomputationMismatch => {
                formatter.write_str("rebuilt plan does not reproduce the input bytes")
            }
            Self::PredictionNeverBinds => {
                formatter.write_str("a simulated topology proposes and never binds")
            }
            Self::MalformedLinkage => formatter.write_str("malformed reversal linkage"),
            Self::ReversibleWithoutReversal => formatter
                .write_str("a Reversible step stands only on an emitted reversal (ADR-0022)"),
            Self::PreconditionFailed { node } => {
                write!(
                    formatter,
                    "a step precondition fails: {node} occupies a region the plan requires empty"
                )
            }
            Self::DraftSpellingOutsideDraft => formatter
                .write_str("a step-output target reference is a reversal draft's spelling only"),
        }
    }
}

impl std::error::Error for PlanSchemaError {}

/// How a reversal-draft step spells its target (ADR-0022): an address
/// for a node that exists before the forward apply, or a typed
/// reference to the output of the forward step that creates it — never
/// an address for a created node, which is why the address spelling of
/// such a target is inexpressible in a draft that declares the
/// reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftTarget {
    /// A node that pre-exists the forward apply, by address.
    Address(NodeId),
    /// The output of the forward plan's step at this index — resolved
    /// to a derived address only at the reversal's validation, against
    /// the helper's own capture (ADR-0019's recompute-at-decode
    /// discipline).
    StepOutput(usize),
}

/// A draft's spelling of a precondition: carried directly where its
/// subject pre-exists, or by step-output reference where the subject is
/// a node the forward plan creates — rewritten to the resolved
/// [`Precondition`] at binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftPrecondition {
    /// A precondition whose subject exists before the forward apply.
    Carried(Precondition),
    /// The forward step's created output must host nothing — resolved
    /// to [`Precondition::HostUnoccupied`] at binding, when the created
    /// node has an address.
    StepOutputUnoccupied {
        /// The creating forward step's index.
        step: usize,
    },
}

/// One reversal-draft step: the draft's spelling of a step that becomes
/// a [`PlanStep`] at the draft's binding, when references resolve and
/// the closure runs against a real capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftStep {
    /// The target spelling.
    pub target: DraftTarget,
    /// The declared range sets.
    pub ranges: StepRanges,
    /// The acknowledgments the step will carry.
    pub acknowledgments: Vec<Acknowledgment>,
    /// The declared risk.
    pub risk: StepRisk,
    /// The truthfulness preconditions, re-checked at binding.
    pub preconditions: Vec<DraftPrecondition>,
}

/// PLAN-008's emitted reversal: an ordinary plan **draft**. Its
/// planning-time source-snapshot proposal is the forward plan's
/// simulated final topology — a prediction that proposes and never
/// binds; binding happens at [`ReversalDraft::bind`], after the forward
/// apply, against the helper's own capture. The draft's own reversal
/// linkage is the re-application statement naming the forward plan by
/// ID, which is how the regress terminates by construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReversalDraft {
    plan_id: Vec<u8>,
    created_at: u64,
    proposal_hash: Hash,
    validity: ValidityWindow,
    steps: Vec<DraftStep>,
    forward_plan_id: Vec<u8>,
}

/// Why a draft refused composition — emission-time truthfulness
/// (ADR-0022: "only where truthful" is judged at emission first).
#[derive(Debug, PartialEq, Eq)]
pub enum DraftRefusal {
    /// The proposal snapshot is not a simulated topology: a draft's
    /// proposal is definitionally the forward plan's prediction.
    ProposalMustBeSimulated,
    /// A step-output reference names a forward step that does not
    /// exist.
    ForwardStepOutOfRange {
        /// The referenced index.
        step: usize,
    },
    /// A step-output reference names a forward step that does not
    /// create exactly one structure (one consumed range).
    NotACreatingStep {
        /// The referenced index.
        step: usize,
    },
    /// The proposal does not place exactly one node at the creating
    /// step's range, so the reference does not resolve even in the
    /// prediction.
    UnresolvableInProposal {
        /// The referenced index.
        step: usize,
    },
    /// The closure refused the step against the proposal — a reversal
    /// that would reach a protected node is untruthful at emission.
    EmissionRefused(StepRefusal),
    /// A precondition already fails in the proposal: the draft would be
    /// untruthful the moment it was emitted.
    UntruthfulAtEmission {
        /// The node violating the precondition.
        node: NodeId,
    },
    /// The proposal could not hash.
    Snapshot,
}

impl fmt::Display for DraftRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProposalMustBeSimulated => {
                formatter.write_str("a draft's proposal is the forward plan's simulated topology")
            }
            Self::ForwardStepOutOfRange { step } => {
                write!(formatter, "no forward step at index {step}")
            }
            Self::NotACreatingStep { step } => {
                write!(
                    formatter,
                    "forward step {step} creates nothing referencable"
                )
            }
            Self::UnresolvableInProposal { step } => write!(
                formatter,
                "the proposal does not place exactly one node at forward step {step}'s output"
            ),
            Self::EmissionRefused(refusal) => {
                write!(formatter, "refused at emission: {refusal}")
            }
            Self::UntruthfulAtEmission { node } => write!(
                formatter,
                "untruthful at emission: {node} already occupies a region the reversal requires empty"
            ),
            Self::Snapshot => formatter.write_str("proposal not hashable"),
        }
    }
}

impl std::error::Error for DraftRefusal {}

/// Why a draft refused binding — the reversal's own validation against
/// a fresh capture, after the forward apply.
#[derive(Debug, PartialEq, Eq)]
pub enum BindRefusal {
    /// The binding snapshot is a prediction. Nobody ever applies a
    /// prediction: only a helper capture binds (ADR-0022's
    /// Simulated-never-binds rule extended to the reversal path).
    PredictionNeverBinds,
    /// The supplied forward plan is not the one this draft reverses.
    NotItsForwardPlan,
    /// A step-output reference names a forward step that does not
    /// exist.
    ForwardStepOutOfRange {
        /// The referenced index.
        step: usize,
    },
    /// A step-output reference names a forward step that does not
    /// create exactly one structure.
    NotACreatingStep {
        /// The referenced index.
        step: usize,
    },
    /// The capture does not place exactly one node at the creating
    /// step's range — an unresolvable reference refuses (a pre-apply
    /// world resolves nothing; an ambiguous world resolves nothing
    /// either).
    UnresolvedReference {
        /// The referenced index.
        step: usize,
        /// How many nodes the capture places at the range.
        candidates: usize,
    },
    /// The closure refused the resolved step against the capture.
    StepRefused(StepRefusal),
    /// A precondition fails against the capture: the reversal that was
    /// truthful at emission refuses now instead of silently becoming a
    /// destructive plan wearing a reversal's advertisement.
    PreconditionFailed {
        /// The node violating the precondition.
        node: NodeId,
    },
    /// Assembling the bound plan refused.
    Assembly(PlanError),
}

impl fmt::Display for BindRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PredictionNeverBinds => {
                formatter.write_str("a simulated topology proposes and never binds")
            }
            Self::NotItsForwardPlan => {
                formatter.write_str("the supplied forward plan is not the one this draft reverses")
            }
            Self::ForwardStepOutOfRange { step } => {
                write!(formatter, "no forward step at index {step}")
            }
            Self::NotACreatingStep { step } => {
                write!(
                    formatter,
                    "forward step {step} creates nothing referencable"
                )
            }
            Self::UnresolvedReference { step, candidates } => write!(
                formatter,
                "forward step {step}'s output resolves to {candidates} node(s) in the capture; \
                 exactly one is required"
            ),
            Self::StepRefused(refusal) => write!(formatter, "refused at binding: {refusal}"),
            Self::PreconditionFailed { node } => write!(
                formatter,
                "precondition failed: {node} occupies a region the reversal requires empty"
            ),
            Self::Assembly(error) => write!(formatter, "bound-plan assembly refused: {error}"),
        }
    }
}

impl std::error::Error for BindRefusal {}

/// Resolve a step-output reference in some world: the forward step's
/// single consumed range, and the one node the world places exactly
/// there. Returns the candidate count on failure so refusals can name
/// it.
fn resolve_step_output(
    forward_steps: &[PlanStep],
    world: &TopologySnapshot,
    step: usize,
) -> Result<NodeId, (bool, usize)> {
    // (out_of_range_or_not_creating, candidates): the caller maps the
    // tuple into its own refusal vocabulary.
    let Some(creating) = forward_steps.get(step) else {
        return Err((true, 0));
    };
    let [created_range] = creating.ranges().consumed.as_slice() else {
        return Err((true, 1));
    };
    let candidates: Vec<NodeId> = world
        .facts()
        .extents
        .iter()
        .filter(|(node, extent)| **node != created_range.host && *extent == created_range)
        .map(|(node, _)| *node)
        .collect();
    match candidates.as_slice() {
        [only] => Ok(*only),
        _ => Err((false, candidates.len())),
    }
}

impl ReversalDraft {
    /// Compose a draft at emission time: the proposal is the forward
    /// plan's simulated final topology, and truthfulness is judged here
    /// first — references must resolve in the proposal, the closure
    /// must permit every step against it, and every precondition must
    /// hold in it.
    ///
    /// # Errors
    ///
    /// [`DraftRefusal`], each variant naming what was untruthful.
    pub fn compose(
        plan_id: Vec<u8>,
        created_at: u64,
        proposal: &TopologySnapshot,
        validity: ValidityWindow,
        steps: Vec<DraftStep>,
        forward_plan_id: Vec<u8>,
        forward_steps: &[PlanStep],
    ) -> Result<Self, DraftRefusal> {
        if proposal.kind() != SnapshotKind::Simulated {
            return Err(DraftRefusal::ProposalMustBeSimulated);
        }
        let resolve = |index: usize| {
            resolve_step_output(forward_steps, proposal, index).map_err(|(structural, _)| {
                if structural {
                    if index >= forward_steps.len() {
                        DraftRefusal::ForwardStepOutOfRange { step: index }
                    } else {
                        DraftRefusal::NotACreatingStep { step: index }
                    }
                } else {
                    DraftRefusal::UnresolvableInProposal { step: index }
                }
            })
        };
        for draft_step in &steps {
            let target = match draft_step.target {
                DraftTarget::Address(node) => node,
                DraftTarget::StepOutput(index) => resolve(index)?,
            };
            PlanStep::mutating(
                proposal,
                target,
                draft_step.ranges.clone(),
                draft_step.acknowledgments.clone(),
                draft_step.risk,
            )
            .map_err(DraftRefusal::EmissionRefused)?;
            for precondition in &draft_step.preconditions {
                let resolved = match precondition {
                    DraftPrecondition::Carried(carried) => *carried,
                    DraftPrecondition::StepOutputUnoccupied { step } => {
                        Precondition::HostUnoccupied {
                            host: resolve(*step)?,
                        }
                    }
                };
                if let Some(node) = resolved.violated_by(proposal) {
                    return Err(DraftRefusal::UntruthfulAtEmission { node });
                }
            }
        }
        let proposal_hash = proposal.body_hash().map_err(|_| DraftRefusal::Snapshot)?;
        Ok(Self {
            plan_id,
            created_at,
            proposal_hash,
            validity,
            steps,
            forward_plan_id,
        })
    }

    /// The draft's plan ID.
    #[must_use]
    pub fn plan_id(&self) -> &[u8] {
        &self.plan_id
    }

    /// The forward plan this draft reverses, by ID — the only spelling
    /// the acyclicity rule permits on this side.
    #[must_use]
    pub fn forward_plan_id(&self) -> &[u8] {
        &self.forward_plan_id
    }

    /// The draft's steps.
    #[must_use]
    pub fn steps(&self) -> &[DraftStep] {
        &self.steps
    }

    /// The draft body as a canonical value: a linked plan body whose
    /// snapshot hash is the proposal's, whose reversal linkage is the
    /// re-application statement, and whose steps may spell step-output
    /// references.
    #[must_use]
    pub fn body_value(&self) -> Value {
        let mut body = BTreeMap::new();
        body.insert("schema".to_owned(), Value::Text(SCHEMA.to_owned()));
        body.insert(
            "schema_version".to_owned(),
            Value::Unsigned(LINKED_SCHEMA_VERSION),
        );
        body.insert("plan_id".to_owned(), Value::Bytes(self.plan_id.clone()));
        body.insert("created_at".to_owned(), Value::Unsigned(self.created_at));
        body.insert(
            "snapshot_hash".to_owned(),
            Value::Bytes(self.proposal_hash.as_bytes().to_vec()),
        );
        body.insert(
            "not_after".to_owned(),
            Value::Unsigned(self.validity.not_after),
        );
        body.insert("identities".to_owned(), Value::Map(BTreeMap::new()));
        body.insert(
            "steps".to_owned(),
            Value::Array(self.steps.iter().map(draft_step_value).collect()),
        );
        body.insert(
            "reversal".to_owned(),
            reversal_value(&ReversalLinkage::ReapplyForward {
                forward_plan_id: self.forward_plan_id.clone(),
            }),
        );
        Value::Map(body)
    }

    /// The draft body hash — what the forward plan's linkage freezes as
    /// the advertisement made at authorization time.
    ///
    /// # Errors
    ///
    /// [`PlanError::Encoding`] on an unencodable element — unreachable
    /// for a draft this module composed.
    pub fn body_hash(&self) -> Result<Hash, PlanError> {
        canonical::hash(&self.body_value()).map_err(|_| PlanError::Encoding)
    }

    /// Rebuild a draft from body bytes: decode, parse strictly, and
    /// require the rebuilt body to reproduce the input exactly. No
    /// snapshot and no closure — a draft is a prediction's plan, and
    /// every safety judgment happens at [`Self::bind`].
    ///
    /// # Errors
    ///
    /// [`PlanSchemaError`] naming the first rule violated.
    pub fn from_canonical_body(bytes: &[u8]) -> Result<Self, PlanSchemaError> {
        let value = canonical::decode(bytes).map_err(PlanSchemaError::Codec)?;
        let Value::Map(map) = value else {
            return Err(PlanSchemaError::NotABodyMap);
        };
        for key in map.keys() {
            if !matches!(
                key.as_str(),
                "schema"
                    | "schema_version"
                    | "plan_id"
                    | "created_at"
                    | "snapshot_hash"
                    | "not_after"
                    | "identities"
                    | "steps"
                    | "reversal"
            ) {
                return Err(PlanSchemaError::UnknownField { key: key.clone() });
            }
        }
        match map.get("schema") {
            Some(Value::Text(text)) if text == SCHEMA => {}
            _ => return Err(PlanSchemaError::WrongSchema),
        }
        match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == LINKED_SCHEMA_VERSION => {}
            _ => return Err(PlanSchemaError::WrongSchemaVersion),
        }
        let (plan_id, created_at, not_after) = parse_header_scalars(&map)?;
        let proposal_hash = match map.get("snapshot_hash") {
            Some(Value::Bytes(bytes)) => {
                let recorded: [u8; 32] =
                    bytes
                        .as_slice()
                        .try_into()
                        .map_err(|_| PlanSchemaError::MissingField {
                            key: "snapshot_hash",
                        })?;
                Hash::from_bytes(recorded)
            }
            _ => {
                return Err(PlanSchemaError::MissingField {
                    key: "snapshot_hash",
                });
            }
        };
        match map.get("identities") {
            Some(Value::Map(identities)) if identities.is_empty() => {}
            // A draft binds identities at validation; carrying them in
            // the prediction would be a client-authored claim.
            _ => return Err(PlanSchemaError::MalformedIdentity),
        }
        let forward_plan_id = match map.get("reversal").map(parse_reversal) {
            Some(Ok(ReversalLinkage::ReapplyForward { forward_plan_id })) => forward_plan_id,
            Some(Ok(_)) => return Err(PlanSchemaError::MalformedLinkage),
            Some(Err(error)) => return Err(error),
            None => return Err(PlanSchemaError::MissingField { key: "reversal" }),
        };
        let Some(Value::Array(step_values)) = map.get("steps") else {
            return Err(PlanSchemaError::MissingField { key: "steps" });
        };
        let mut steps = Vec::new();
        for step_value in step_values {
            steps.push(parse_draft_step(step_value)?);
        }
        let rebuilt = Self {
            plan_id,
            created_at,
            proposal_hash,
            validity: ValidityWindow { not_after },
            steps,
            forward_plan_id,
        };
        let recomputed = canonical::encode(&rebuilt.body_value())
            .map_err(|_| PlanSchemaError::SnapshotUnhashable)?;
        if recomputed != bytes {
            return Err(PlanSchemaError::RecomputationMismatch);
        }
        Ok(rebuilt)
    }

    /// Bind the draft at its own validation, after the forward apply:
    /// references resolve against the helper's capture, every step
    /// re-runs the closure, every precondition is re-checked, and the
    /// result is an ordinary bound plan whose snapshot hash is the
    /// capture's (8.0.0's rule — binding is a validation act).
    ///
    /// # Errors
    ///
    /// [`BindRefusal`], each variant naming what refused.
    pub fn bind(
        &self,
        capture: &TopologySnapshot,
        forward: &OperationPlan,
    ) -> Result<OperationPlan, BindRefusal> {
        if capture.kind() != SnapshotKind::Captured {
            return Err(BindRefusal::PredictionNeverBinds);
        }
        if forward.plan_id() != self.forward_plan_id.as_slice() {
            return Err(BindRefusal::NotItsForwardPlan);
        }
        let resolve = |index: usize| {
            resolve_step_output(forward.steps(), capture, index).map_err(
                |(structural, candidates)| {
                    if structural {
                        if index >= forward.steps().len() {
                            BindRefusal::ForwardStepOutOfRange { step: index }
                        } else {
                            BindRefusal::NotACreatingStep { step: index }
                        }
                    } else {
                        BindRefusal::UnresolvedReference {
                            step: index,
                            candidates,
                        }
                    }
                },
            )
        };
        let mut steps = Vec::with_capacity(self.steps.len());
        for draft_step in &self.steps {
            let target = match draft_step.target {
                DraftTarget::Address(node) => node,
                DraftTarget::StepOutput(index) => resolve(index)?,
            };
            // References resolve into the bound spelling first —
            // ADR-0019's recompute-at-decode discipline: the address is
            // the helper's own derivation, and the bound plan carries
            // only evaluable preconditions.
            let mut resolved_preconditions = Vec::with_capacity(draft_step.preconditions.len());
            for precondition in &draft_step.preconditions {
                let resolved = match precondition {
                    DraftPrecondition::Carried(carried) => *carried,
                    DraftPrecondition::StepOutputUnoccupied { step } => {
                        Precondition::HostUnoccupied {
                            host: resolve(*step)?,
                        }
                    }
                };
                if let Some(node) = resolved.violated_by(capture) {
                    return Err(BindRefusal::PreconditionFailed { node });
                }
                resolved_preconditions.push(resolved);
            }
            let step = PlanStep::mutating(
                capture,
                target,
                draft_step.ranges.clone(),
                draft_step.acknowledgments.clone(),
                draft_step.risk,
            )
            .map_err(BindRefusal::StepRefused)?
            .with_preconditions(resolved_preconditions);
            steps.push(step);
        }
        OperationPlan::assemble_linked(
            self.plan_id.clone(),
            self.created_at,
            capture,
            self.validity,
            BTreeMap::new(),
            steps,
            ReversalLinkage::ReapplyForward {
                forward_plan_id: self.forward_plan_id.clone(),
            },
        )
        .map_err(BindRefusal::Assembly)
    }
}

fn draft_step_value(step: &DraftStep) -> Value {
    let mut map = BTreeMap::new();
    match step.target {
        DraftTarget::Address(node) => {
            map.insert("target".to_owned(), Value::Bytes(node.as_bytes().to_vec()));
        }
        DraftTarget::StepOutput(index) => {
            map.insert(
                "target_step_output".to_owned(),
                Value::Unsigned(u64::try_from(index).expect("a step index fits in a u64")),
            );
        }
    }
    map.insert(
        "preconditions".to_owned(),
        Value::Array(
            step.preconditions
                .iter()
                .map(draft_precondition_value)
                .collect(),
        ),
    );
    // A draft step is ordinary by construction today; a repair-family
    // draft is a future reviewed extension, so the class is pinned
    // rather than parameterized.
    map.insert(
        "class".to_owned(),
        Value::Text(class_wire_name(StepClass::Ordinary).to_owned()),
    );
    map.insert(
        "written_table_extents".to_owned(),
        ranges_value(&step.ranges.written_table_extents),
    );
    map.insert("consumed".to_owned(), ranges_value(&step.ranges.consumed));
    map.insert("destroyed".to_owned(), ranges_value(&step.ranges.destroyed));
    map.insert(
        "acknowledgments".to_owned(),
        Value::Array(
            step.acknowledgments
                .iter()
                .map(acknowledgment_value)
                .collect(),
        ),
    );
    insert_risk(&mut map, step.risk);
    Value::Map(map)
}

fn parse_draft_step(value: &Value) -> Result<DraftStep, PlanSchemaError> {
    let Value::Map(map) = value else {
        return Err(PlanSchemaError::MalformedStep);
    };
    for key in map.keys() {
        if !matches!(
            key.as_str(),
            "target"
                | "target_step_output"
                | "preconditions"
                | "class"
                | "written_table_extents"
                | "consumed"
                | "destroyed"
                | "acknowledgments"
                | "severity"
                | "flags"
        ) {
            return Err(PlanSchemaError::UnknownField { key: key.clone() });
        }
    }
    if parse_class(map)? != StepClass::Ordinary {
        // A repair-family draft step is a future reviewed extension.
        return Err(PlanSchemaError::MalformedStep);
    }
    let target = match (map.get("target"), map.get("target_step_output")) {
        (Some(value), None) => DraftTarget::Address(parse_node(Some(value))?),
        (None, Some(Value::Unsigned(index))) => DraftTarget::StepOutput(
            usize::try_from(*index).map_err(|_| PlanSchemaError::MalformedStep)?,
        ),
        // Exactly one spelling: both, neither, or a mistyped reference
        // all refuse.
        _ => return Err(PlanSchemaError::MalformedStep),
    };
    let Some(Value::Array(precondition_values)) = map.get("preconditions") else {
        return Err(PlanSchemaError::MalformedStep);
    };
    let preconditions = precondition_values
        .iter()
        .map(parse_draft_precondition)
        .collect::<Result<Vec<_>, _>>()?;
    let ranges = StepRanges {
        written_table_extents: parse_ranges(map.get("written_table_extents"))?,
        consumed: parse_ranges(map.get("consumed"))?,
        destroyed: parse_ranges(map.get("destroyed"))?,
    };
    let Some(Value::Array(acknowledgment_values)) = map.get("acknowledgments") else {
        return Err(PlanSchemaError::MalformedStep);
    };
    let acknowledgments = acknowledgment_values
        .iter()
        .map(parse_acknowledgment)
        .collect::<Result<Vec<_>, _>>()?;
    let risk = parse_risk(map)?;
    Ok(DraftStep {
        target,
        ranges,
        acknowledgments,
        risk,
        preconditions,
    })
}
