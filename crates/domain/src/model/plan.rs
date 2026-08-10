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
use super::snapshot::TopologySnapshot;
use super::step::{Acknowledgment, PlanStep, Severity, StepFlags, StepRefusal, StepRisk};

/// The plan body's schema identity (MODEL-003).
pub const SCHEMA: &str = "partman.plan";
/// The current plan body schema version.
pub const SCHEMA_VERSION: u64 = 1;

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
}

impl OperationPlan {
    /// Assemble a plan over steps already constructed against the given
    /// snapshot. The snapshot's body hash is bound into the plan
    /// (PLAN-006's comparison object, as bound at validation per 8.0.0).
    ///
    /// # Errors
    ///
    /// [`PlanError::Snapshot`] if the snapshot cannot hash.
    pub fn assemble(
        plan_id: Vec<u8>,
        created_at: u64,
        snapshot: &TopologySnapshot,
        validity: ValidityWindow,
        identities: BTreeMap<NodeId, DeviceIdentity>,
        steps: Vec<PlanStep>,
    ) -> Result<Self, PlanError> {
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
        })
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
        let mut body = BTreeMap::new();
        body.insert("schema".to_owned(), Value::Text(SCHEMA.to_owned()));
        body.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
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
            Value::Array(self.steps.iter().map(step_value).collect()),
        );
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
            ) {
                return Err(PlanSchemaError::UnknownField { key: key.clone() });
            }
        }
        match map.get("schema") {
            Some(Value::Text(text)) if text == SCHEMA => {}
            _ => return Err(PlanSchemaError::WrongSchema),
        }
        match map.get("schema_version") {
            Some(Value::Unsigned(version)) if *version == SCHEMA_VERSION => {}
            _ => return Err(PlanSchemaError::WrongSchemaVersion),
        }
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
        let Some(Value::Array(step_values)) = map.get("steps") else {
            return Err(PlanSchemaError::MissingField { key: "steps" });
        };
        let mut steps = Vec::new();
        for step_value in step_values {
            steps.push(parse_step(step_value, snapshot)?);
        }
        let rebuilt = Self {
            plan_id,
            created_at,
            snapshot_hash: actual,
            validity: ValidityWindow { not_after },
            identities,
            steps,
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

fn step_value(step: &PlanStep) -> Value {
    let mut map = BTreeMap::new();
    map.insert(
        "target".to_owned(),
        Value::Bytes(step.target().as_bytes().to_vec()),
    );
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
    map.insert(
        "severity".to_owned(),
        Value::Unsigned(match step.risk().severity {
            Severity::Informational => 0,
            Severity::Reversible => 1,
            Severity::Disruptive => 2,
            Severity::DataMoving => 3,
            Severity::Destructive => 4,
        }),
    );
    let flags = step.risk().flags;
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
    Value::Map(map)
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
    };
    map.insert("kind".to_owned(), Value::Text(kind.to_owned()));
    map.insert("node".to_owned(), Value::Bytes(node.as_bytes().to_vec()));
    Value::Map(map)
}

fn parse_step(value: &Value, snapshot: &TopologySnapshot) -> Result<PlanStep, PlanSchemaError> {
    let Value::Map(map) = value else {
        return Err(PlanSchemaError::MalformedStep);
    };
    for key in map.keys() {
        if !matches!(
            key.as_str(),
            "target"
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
    let target = parse_node(map.get("target"))?;
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
    // The recompute: the sole constructor runs the closure and the
    // acknowledgment law over the snapshot's authenticated facts. A
    // forged step never returns.
    PlanStep::mutating(
        snapshot,
        target,
        ranges,
        acknowledgments,
        StepRisk { severity, flags },
    )
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
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot => formatter.write_str("snapshot not hashable"),
            Self::Encoding => formatter.write_str("plan body not encodable"),
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
        }
    }
}

impl std::error::Error for PlanSchemaError {}
