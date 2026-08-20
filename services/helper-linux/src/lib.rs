//! The WP-L110 Linux privileged helper, increment 1: the process and its
//! closed surface.
//!
//! **What this is.** The one privileged process of the Linux product
//! (SAFE-002 context 1), reached over the Linux transport (WP-040
//! increment 5, ADR-0055: a root-created `0711` directory, a `0600` node
//! owned by the authorizing user, the peer verified before any byte is
//! read), launched **per user** through `pkexec` under the package's
//! polkit action `org.partman.helper.serve` (the launch round,
//! `docs/reviews/LINUX_HELPER_LAUNCH_ROUND_2026-08-19.md`, L2 taken:
//! `allow_active`, nothing asked to start; the apply ceremony is a
//! separate ask per plan, HLP-003, increment 3). It accepts exactly
//! HLP-001's six operations — status, enumerate, validate-plan,
//! apply-plan, cancel, resume, journal-query — **and nothing else**: an
//! unknown operation, an unknown field, a path-shaped or command-shaped
//! payload is a typed refusal, and the operation set is closed by a test
//! that matches every variant (obligation 1). In this increment two
//! operations are *served* — status and enumerate, the latter answered
//! from the adapter's client contract run as root and labelled the
//! proposal it is (not HLP-002's re-discovery, which is increment 2) —
//! and the other four are *accepted and refused as not yet served*,
//! naming the increment that serves each: the vocabulary is complete,
//! the service is not, and the wire says which.
//!
//! **Launch rules (the round's L2).** `--serve <uid>` is refused unless
//! `PKEXEC_UID` is set by `pkexec` and equals `<uid>` ([`launch_rule`],
//! pure); the helper then ensures the directory, creates its node
//! through [`partman_transport_linux::linux::Endpoint::create`] (never
//! replacing an existing one — a second launch for the same uid finds a
//! node and exits with [`LaunchRefusal::AlreadyServed`] so the client
//! connects to the first), serves, and exits when idle (HLP-005).
//!
//! **Increment 2 — re-discovery and validate-plan.** `validate-plan` is
//! served: the helper takes its own HLP-002 capture ([`capture`]) — the
//! adapter's contract as root plus the byte layer over read-only device
//! handles ([`bytes`]), authoring the table state (ADR-0014) and the
//! facts the protection closure computes the verdict from (ADR-0016) —
//! and runs WP-060's `plan()` over it ([`validate`]); the client's own
//! draft or validation output is never an input (CAP-007). The wire
//! spells a target as its **naming fields**, never as a raw address
//! digest, so the helper derives every address itself (ADR-0019's
//! recompute-at-decode discipline), and an `Aggregate` target is not
//! even spellable — SI-13's structural interim, enforced again as a
//! typed arm in [`validate::validate_plan`]. SEC-002's admission arms
//! ([`validate::admit_presented_plan`]) are delivered and tested here;
//! the journal-backed act that feeds them is increment 3's. The helper's
//! own reach declaration is [`reach::REACH`], its host half resting on
//! the DR21 row.
//!
//! **Increment 3 — the authorization ladder.** The helper computes
//! HLP-003's required tier for itself, from its own recomputed severity
//! and flags ([`authorize::required_tier`]), and reports it on the
//! validate-plan response — the one site HLP-003 authorizes for it, and
//! UI-011's reason for wanting it. **No request field carries a tier**,
//! and none exists to add one to: that absence is where CAP-007's
//! "a client-assertable authorization is unrepresentable" lives. The
//! ladder itself ([`authorize::authorize`]) reads only an
//! [`validate::AdmittedPlan`], which exists only past SEC-002's arms, so
//! a forged, replayed, cross-user or expired body cannot reach the
//! computation. The floor act needs no polkit, no agent and no terminal
//! (ADR-0021's programmatic act, which keeps SAFE-003's unattended
//! population representable); the interactive ceremony is a seam whose
//! completion value is unconstructible in a shipped build, so every
//! interactive-tier plan is refused on one arm that names no route — the
//! decision taken on
//! `docs/reviews/LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md` (R8, with S2
//! as the decided two-phase shape for when a route lands, and both
//! no-retained-grant constraints bound). **`apply-plan` is not served by
//! this build**: it is answered `not-yet-served` naming increment 4,
//! because single-use cannot be held on a served path whose consumption
//! record does not exist yet.
//!
//! **Identity and logging.** HLP-007 is the transport's (no byte before
//! the verifier). HLP-006's audit log is a closed vocabulary of
//! [`AuditEvent`]s that carry uids, counts, operation names and this
//! crate's own words — no field can hold a serial, path, label or
//! username (SAFE-006 by construction), and a test holds the vocabulary.
//!
//! **What this is not (the assignment's boundary).** No apply and no
//! write (increment 4's; the only device access remains the byte layer's
//! bounded read-only windows); **no journal file** — the act is minted
//! here and appended by the increment that opens the journal; no tool
//! launched, no bus client, no polkit action shipped; no network; no
//! operation outside the six. The pure seams compile and test everywhere;
//! the endpoint and the real device reader exist on Linux only.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::BTreeMap;
use std::io::{Read, Write};

use partman_domain::canonical::{self, Value};
use partman_domain::model::capability::Operation as CapabilityOperation;
use partman_domain::model::naming::{NamingError, NamingFields, NodeId, TableRole, derive_id};
use partman_rpc::{DecodeRefusal, Envelope};
use partman_transport_linux::{Refusal as TransportRefusal, read_frame, write_frame};

pub mod authorize;
pub mod bytes;
pub mod capture;
pub mod clock;
pub mod reach;
pub mod validate;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(test)]
mod tests;

/// The request schema identity (MODEL-003).
pub const REQUEST_SCHEMA: &str = "partman.helper.request";
/// The response schema identity (MODEL-003).
pub const RESPONSE_SCHEMA: &str = "partman.helper.response";
/// Version 3 of both: as version 2, plus the helper-computed
/// authorization tier on the validated response (HLP-003, UI-011).
/// Versions 1 and 2 are refused at decode with a remediation naming this
/// one (RPC-002's "never degrade silently") — the explicit-migration
/// discipline (MODEL-003), recorded in `schemas/helper/operations.md`.
/// The **request** vocabulary is unchanged: no field carries a tier, and
/// that absence is CAP-007's enforcement point.
pub const SCHEMA_VERSION: u64 = 3;
/// The polkit action the launch is authorized under (the round's L2).
pub const POLKIT_ACTION: &str = "org.partman.helper.serve";
/// The environment variable `pkexec` sets to the launching user's uid.
pub const PKEXEC_UID_VARIABLE: &str = "PKEXEC_UID";
/// The default runtime directory (ADR-0055 decision 3; `packaging/`'s
/// unit files may set it otherwise).
pub const DEFAULT_DIRECTORY: &str = "/run/partman";
/// The default idle exit, seconds without a connection (HLP-005).
pub const DEFAULT_IDLE_SECONDS: u64 = 120;

/// HLP-001's closed operation set. A new variant fails the closure test
/// before any prose can drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    /// Status: the helper's own state, no device touched.
    Status,
    /// Enumeration: the adapter's client contract as root — a proposal.
    Enumerate,
    /// Validate a plan (increment 2).
    ValidatePlan,
    /// Apply a plan by hash (increments 3–4).
    ApplyPlan,
    /// Cancel an execution (increment 4).
    Cancel,
    /// Resume an execution (increment 4).
    Resume,
    /// Query the journal (increment 4).
    JournalQuery,
}

impl Operation {
    /// The closed vocabulary, in HLP-001's order.
    pub const ALL: [Self; 7] = [
        Self::Status,
        Self::Enumerate,
        Self::ValidatePlan,
        Self::ApplyPlan,
        Self::Cancel,
        Self::Resume,
        Self::JournalQuery,
    ];

    /// The wire name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Enumerate => "enumerate",
            Self::ValidatePlan => "validate-plan",
            Self::ApplyPlan => "apply-plan",
            Self::Cancel => "cancel",
            Self::Resume => "resume",
            Self::JournalQuery => "journal-query",
        }
    }

    /// Parse a wire name; anything else is not an operation.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|op| op.name() == name)
    }

    /// The increment that serves this operation — `None` when it is
    /// served now.
    #[must_use]
    pub const fn served_in_increment(self) -> Option<u8> {
        match self {
            Self::Status | Self::Enumerate | Self::ValidatePlan => None,
            // Increment 4, not 3, and the correction is deliberate: the
            // ladder (increment 3) computes the tier and mints the act,
            // but ADR-0028's one-act-one-apply needs a consumption
            // record in a journal this build does not open. An operation
            // that authorized without being able to consume would be a
            // served path that cannot hold its own guarantee.
            Self::ApplyPlan | Self::Cancel | Self::Resume | Self::JournalQuery => Some(4),
        }
    }
}

/// A decoded request: the operation, and for `validate-plan` its
/// arguments — nothing else in version 2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// The operation.
    pub operation: Operation,
    /// Present exactly when the operation is `validate-plan` (the strict
    /// decode enforces both directions).
    pub validate: Option<ValidateWire>,
}

/// The validate-plan arguments as the wire spells them. The target is
/// spelled as **naming fields, never as a raw address digest**: the
/// helper derives the address itself (ADR-0019's recompute-at-decode
/// discipline), and a kind outside this vocabulary — an `Aggregate`
/// above all (SI-13) — has no spelling at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidateWire {
    /// The target's spelling.
    pub target: TargetSpelling,
    /// The requested CAP-002 operation.
    pub requested: CapabilityOperation,
    /// The client's plan identifier bytes (correlation, not authority;
    /// at most [`PLAN_ID_LIMIT`]).
    pub plan_id: Vec<u8>,
    /// The requested validity in seconds; `0` takes PLAN-007's default.
    pub validity_seconds: u64,
}

/// The most plan-identifier bytes a request may carry.
pub const PLAN_ID_LIMIT: usize = 64;
/// The most serial or WWN bytes a target spelling may carry.
pub const TARGET_BYTES_LIMIT: usize = 256;

/// The wire's target vocabulary for increment 2: a whole device by its
/// naming triple, or one of its table views. Deliberately closed — the
/// kinds this increment's capture authors, and no other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetSpelling {
    /// A physical device by its ADR-0019 naming fields.
    Device {
        /// Serial bytes, where the designation carries one.
        serial: Option<Vec<u8>>,
        /// WWN bytes, where the designation carries one.
        wwn: Option<Vec<u8>>,
        /// Total size in bytes.
        total_bytes: u64,
    },
    /// A table view on such a device.
    Table {
        /// The carrying device's serial bytes, where designated.
        serial: Option<Vec<u8>>,
        /// The carrying device's WWN bytes, where designated.
        wwn: Option<Vec<u8>>,
        /// The carrying device's total size in bytes.
        total_bytes: u64,
        /// The view's role.
        role: TableRole,
    },
}

impl TargetSpelling {
    /// Derive the spelled target's address, exactly as the capture
    /// derives it from the same facts.
    ///
    /// # Errors
    ///
    /// The domain's naming error, verbatim, if the fields refuse.
    pub fn derive(&self) -> Result<NodeId, NamingError> {
        match self {
            Self::Device {
                serial,
                wwn,
                total_bytes,
            } => derive_id(&NamingFields::PhysicalDevice {
                serial: serial.clone(),
                wwn: wwn.clone(),
                total_bytes: *total_bytes,
            }),
            Self::Table {
                serial,
                wwn,
                total_bytes,
                role,
            } => {
                let parent = derive_id(&NamingFields::PhysicalDevice {
                    serial: serial.clone(),
                    wwn: wwn.clone(),
                    total_bytes: *total_bytes,
                })?;
                derive_id(&NamingFields::PartitionTable {
                    parent,
                    role: role.clone(),
                })
            }
        }
    }
}

/// Why a request body was refused. Typed; no peer-authored bytes in the
/// strings beyond a field name, which the strict decode bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestRefusal {
    /// Not canonical, or not a map.
    NotAMessage,
    /// The schema identity is not this crate's.
    WrongSchema,
    /// The schema identity is right and the version is not this one.
    /// Separated from [`Self::WrongSchema`] so the reply can carry
    /// RPC-002's remediation instead of a debug rendering.
    WrongVersion {
        /// The version the peer spoke.
        spoken: u64,
    },
    /// A field outside the version's vocabulary (RPC-003 strict).
    UnknownField {
        /// The field name, bounded.
        key: String,
    },
    /// The operation is not one of HLP-001's six.
    UnknownOperation,
    /// The operation field is missing or not text.
    MissingOperation,
    /// A field the operation requires is missing.
    MissingField {
        /// The field name.
        key: &'static str,
    },
    /// A field present on an operation whose vocabulary excludes it.
    FieldOutOfPlace {
        /// The field name.
        key: &'static str,
    },
    /// A field whose value violates its rule (type, bound, or closed
    /// vocabulary).
    BadField {
        /// The field name.
        key: &'static str,
    },
}

/// The version-2 field vocabulary: the header triple, and the
/// validate-plan arguments admitted exactly when the operation is
/// `validate-plan`.
const HEADER_FIELDS: [&str; 3] = ["schema", "schema_version", "operation"];
const VALIDATE_FIELDS: [&str; 8] = [
    "target_kind",
    "target_serial",
    "target_wwn",
    "target_total_bytes",
    "target_role",
    "requested_operation",
    "plan_id",
    "validity_seconds",
];

impl Request {
    /// Encode a request to canonical bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::NotCanonical`] if encoding refuses — unreachable for
    /// these flat maps, reported rather than panicked.
    pub fn encode(&self) -> Result<Vec<u8>, DecodeRefusal> {
        let mut map = BTreeMap::new();
        map.insert("schema".to_owned(), Value::Text(REQUEST_SCHEMA.to_owned()));
        map.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
        map.insert(
            "operation".to_owned(),
            Value::Text(self.operation.name().to_owned()),
        );
        if let Some(validate) = &self.validate {
            Self::encode_validate_fields(validate, &mut map)?;
        }
        canonical::encode(&Value::Map(map)).map_err(|_| DecodeRefusal::NotCanonical)
    }

    fn encode_validate_fields(
        validate: &ValidateWire,
        map: &mut BTreeMap<String, Value>,
    ) -> Result<(), DecodeRefusal> {
        let (kind, serial, wwn, total_bytes, role) = match &validate.target {
            TargetSpelling::Device {
                serial,
                wwn,
                total_bytes,
            } => ("physical-device", serial, wwn, *total_bytes, None),
            TargetSpelling::Table {
                serial,
                wwn,
                total_bytes,
                role,
            } => ("partition-table", serial, wwn, *total_bytes, Some(role)),
        };
        map.insert("target_kind".to_owned(), Value::Text(kind.to_owned()));
        if let Some(serial) = serial {
            map.insert("target_serial".to_owned(), Value::Bytes(serial.clone()));
        }
        if let Some(wwn) = wwn {
            map.insert("target_wwn".to_owned(), Value::Bytes(wwn.clone()));
        }
        map.insert(
            "target_total_bytes".to_owned(),
            Value::Unsigned(total_bytes),
        );
        if let Some(role) = role {
            let name = match role {
                TableRole::Gpt => "gpt",
                TableRole::Mbr => "mbr",
                TableRole::Apm => "apm",
                TableRole::HybridMbr => "hybrid-mbr",
                TableRole::Unrecognized { .. } => {
                    return Err(DecodeRefusal::NotCanonical);
                }
            };
            map.insert("target_role".to_owned(), Value::Text(name.to_owned()));
        }
        map.insert(
            "requested_operation".to_owned(),
            Value::Text(operation_name(validate.requested).to_owned()),
        );
        map.insert("plan_id".to_owned(), Value::Bytes(validate.plan_id.clone()));
        map.insert(
            "validity_seconds".to_owned(),
            Value::Unsigned(validate.validity_seconds),
        );
        Ok(())
    }

    /// The strict decode: unknown fields, unknown operations, missing or
    /// out-of-place arguments, and violated bounds each refuse — the
    /// first rule violated, typed.
    ///
    /// # Errors
    ///
    /// [`RequestRefusal`].
    pub fn decode(bytes: &[u8]) -> Result<Self, RequestRefusal> {
        let value = canonical::decode(bytes).map_err(|_| RequestRefusal::NotAMessage)?;
        let Value::Map(map) = value else {
            return Err(RequestRefusal::NotAMessage);
        };
        for key in map.keys() {
            if !HEADER_FIELDS.contains(&key.as_str()) && !VALIDATE_FIELDS.contains(&key.as_str()) {
                return Err(RequestRefusal::UnknownField {
                    key: key.chars().take(64).collect(),
                });
            }
        }
        match (map.get("schema"), map.get("schema_version")) {
            (Some(Value::Text(s)), Some(Value::Unsigned(v)))
                if s == REQUEST_SCHEMA && *v == SCHEMA_VERSION => {}
            (Some(Value::Text(s)), Some(Value::Unsigned(spoken))) if s == REQUEST_SCHEMA => {
                return Err(RequestRefusal::WrongVersion { spoken: *spoken });
            }
            _ => return Err(RequestRefusal::WrongSchema),
        }
        let Some(Value::Text(name)) = map.get("operation") else {
            return Err(RequestRefusal::MissingOperation);
        };
        let operation = Operation::parse(name).ok_or(RequestRefusal::UnknownOperation)?;
        if operation != Operation::ValidatePlan {
            if let Some(key) = VALIDATE_FIELDS.iter().find(|key| map.contains_key(**key)) {
                return Err(RequestRefusal::FieldOutOfPlace { key });
            }
            return Ok(Self {
                operation,
                validate: None,
            });
        }
        let validate = decode_validate(&map)?;
        Ok(Self {
            operation,
            validate: Some(validate),
        })
    }
}

/// The CAP-002 operation's wire name (the store's kebab spelling).
#[must_use]
pub const fn operation_name(operation: CapabilityOperation) -> &'static str {
    match operation {
        CapabilityOperation::Detect => "detect",
        CapabilityOperation::Read => "read",
        CapabilityOperation::Create => "create",
        CapabilityOperation::Grow => "grow",
        CapabilityOperation::Shrink => "shrink",
        CapabilityOperation::Move => "move",
        CapabilityOperation::Copy => "copy",
        CapabilityOperation::Check => "check",
        CapabilityOperation::Repair => "repair",
        CapabilityOperation::Label => "label",
        CapabilityOperation::Uuid => "uuid",
        CapabilityOperation::Encrypt => "encrypt",
        CapabilityOperation::Decrypt => "decrypt",
        CapabilityOperation::Wipe => "wipe",
    }
}

fn bounded_bytes(
    map: &BTreeMap<String, Value>,
    key: &'static str,
    limit: usize,
) -> Result<Option<Vec<u8>>, RequestRefusal> {
    match map.get(key) {
        None => Ok(None),
        Some(Value::Bytes(bytes)) if bytes.len() <= limit => Ok(Some(bytes.clone())),
        Some(_) => Err(RequestRefusal::BadField { key }),
    }
}

fn decode_validate(map: &BTreeMap<String, Value>) -> Result<ValidateWire, RequestRefusal> {
    let Some(Value::Text(kind)) = map.get("target_kind") else {
        return Err(RequestRefusal::MissingField { key: "target_kind" });
    };
    let serial = bounded_bytes(map, "target_serial", TARGET_BYTES_LIMIT)?;
    let wwn = bounded_bytes(map, "target_wwn", TARGET_BYTES_LIMIT)?;
    let Some(Value::Unsigned(total_bytes)) = map.get("target_total_bytes") else {
        return Err(RequestRefusal::MissingField {
            key: "target_total_bytes",
        });
    };
    let target = match kind.as_str() {
        "physical-device" => {
            if map.contains_key("target_role") {
                return Err(RequestRefusal::FieldOutOfPlace { key: "target_role" });
            }
            TargetSpelling::Device {
                serial,
                wwn,
                total_bytes: *total_bytes,
            }
        }
        "partition-table" => {
            let Some(Value::Text(role)) = map.get("target_role") else {
                return Err(RequestRefusal::MissingField { key: "target_role" });
            };
            let role = match role.as_str() {
                "gpt" => TableRole::Gpt,
                "mbr" => TableRole::Mbr,
                "apm" => TableRole::Apm,
                "hybrid-mbr" => TableRole::HybridMbr,
                _ => return Err(RequestRefusal::BadField { key: "target_role" }),
            };
            TargetSpelling::Table {
                serial,
                wwn,
                total_bytes: *total_bytes,
                role,
            }
        }
        _ => return Err(RequestRefusal::BadField { key: "target_kind" }),
    };
    let Some(Value::Text(requested)) = map.get("requested_operation") else {
        return Err(RequestRefusal::MissingField {
            key: "requested_operation",
        });
    };
    let requested =
        crate::validate::parse_operation(requested).ok_or(RequestRefusal::BadField {
            key: "requested_operation",
        })?;
    let plan_id = bounded_bytes(map, "plan_id", PLAN_ID_LIMIT)?
        .ok_or(RequestRefusal::MissingField { key: "plan_id" })?;
    let Some(Value::Unsigned(validity_seconds)) = map.get("validity_seconds") else {
        return Err(RequestRefusal::MissingField {
            key: "validity_seconds",
        });
    };
    Ok(ValidateWire {
        target,
        requested,
        plan_id,
        validity_seconds: *validity_seconds,
    })
}

/// What the helper answers. Every arm is a closed shape; `Enumeration`
/// carries per-device facts of kind and class only — selectors, kinds,
/// transport classes and counts, never identifier bytes (those stay in
/// the adapter's observation set, for increment 2's snapshot body).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response {
    /// Status, served now.
    Status {
        /// The helper's build version (RPC-002's grammar).
        build: String,
        /// The uid this helper serves.
        authorizing_uid: u32,
        /// Which operations are served in this build.
        served: Vec<Operation>,
    },
    /// Enumeration, served now: the adapter's client contract as root —
    /// **a proposal**, not HLP-002's re-discovery.
    Enumeration {
        /// `true` always in this increment; stated on the wire.
        proposal: bool,
        /// The enumeration's outcome by name: listed, over-limit,
        /// unavailable, failed (the adapter's own arms).
        outcome: String,
        /// Per admitted device: selector, kind name, transport class
        /// name, property count.
        devices: Vec<EnumeratedDevice>,
    },
    /// Validate-plan, served now: the helper re-planned over its own
    /// HLP-002 capture, and this is the plan the client may display and
    /// later apply by hash. Nothing here is client-authored except the
    /// plan identifier bytes.
    Validated {
        /// The plan body's canonical bytes.
        plan: Vec<u8>,
        /// The body hash HLP-003's act will name.
        plan_hash: Vec<u8>,
        /// The bound snapshot's body hash — the helper's own capture.
        snapshot_hash: Vec<u8>,
        /// The helper-computed severity name (PLAN-004).
        severity: String,
        /// The helper-computed flag names (PLAN-004).
        flags: Vec<String>,
        /// The helper-computed authorization tier (HLP-003, ADR-0021):
        /// `floor-act` or `interactive-ceremony`. **Response data, never
        /// plan body** — MODEL-005's authoring set stays closed at two,
        /// and no client can assert this (CAP-007).
        tier: String,
        /// PLAN-007's window end.
        not_after: u64,
    },
    /// Validate-plan ran and refused, with the arm named and the ground
    /// carried in this crate's and the planner's own words — never a
    /// stub, never a guess.
    ValidationRefused {
        /// The refusing arm's name.
        arm: String,
        /// The ground, verbatim from the refusing layer.
        detail: String,
    },
    /// The operation exists and is not served by this build; the
    /// increment that serves it is named. Fail-closed, never a stub
    /// success.
    NotYetServed {
        /// The operation asked for.
        operation: Operation,
        /// The increment that serves it.
        increment: u8,
    },
    /// The request was refused before any operation ran.
    Refused {
        /// Why, as this crate's words.
        reason: String,
    },
}

/// One device as enumeration reports it: kind and class, no identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumeratedDevice {
    /// The session-local selector (`device:N`).
    pub selector: String,
    /// The adapter's kind name: `plain`, `host-assembled:<kind>`,
    /// `indeterminate`.
    pub kind: String,
    /// The transport class name (ADR-0018's closed list, `Unrecognized`
    /// included).
    pub transport: String,
    /// How many properties the adapter observed.
    pub properties: u64,
}

impl Response {
    /// Encode to canonical bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::NotCanonical`] — unreachable for these shapes,
    /// reported rather than panicked.
    pub fn encode(&self) -> Result<Vec<u8>, DecodeRefusal> {
        let mut map = BTreeMap::new();
        map.insert("schema".to_owned(), Value::Text(RESPONSE_SCHEMA.to_owned()));
        map.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
        match self {
            Self::Status {
                build,
                authorizing_uid,
                served,
            } => {
                map.insert("outcome".to_owned(), Value::Text("status".to_owned()));
                map.insert("build".to_owned(), Value::Text(build.clone()));
                map.insert(
                    "authorizing_uid".to_owned(),
                    Value::Unsigned(u64::from(*authorizing_uid)),
                );
                map.insert(
                    "served".to_owned(),
                    Value::Array(
                        served
                            .iter()
                            .map(|op| Value::Text(op.name().to_owned()))
                            .collect(),
                    ),
                );
            }
            Self::Enumeration {
                proposal,
                outcome,
                devices,
            } => encode_enumeration(&mut map, *proposal, outcome, devices),
            Self::Validated {
                plan,
                plan_hash,
                snapshot_hash,
                severity,
                flags,
                tier,
                not_after,
            } => {
                map.insert("outcome".to_owned(), Value::Text("validated".to_owned()));
                map.insert("plan".to_owned(), Value::Bytes(plan.clone()));
                map.insert("plan_hash".to_owned(), Value::Bytes(plan_hash.clone()));
                map.insert(
                    "snapshot_hash".to_owned(),
                    Value::Bytes(snapshot_hash.clone()),
                );
                map.insert("severity".to_owned(), Value::Text(severity.clone()));
                map.insert(
                    "flags".to_owned(),
                    Value::Array(flags.iter().map(|f| Value::Text(f.clone())).collect()),
                );
                map.insert("tier".to_owned(), Value::Text(tier.clone()));
                map.insert("not_after".to_owned(), Value::Unsigned(*not_after));
            }
            Self::ValidationRefused { arm, detail } => {
                map.insert(
                    "outcome".to_owned(),
                    Value::Text("validation-refused".to_owned()),
                );
                map.insert("arm".to_owned(), Value::Text(arm.clone()));
                map.insert("detail".to_owned(), Value::Text(detail.clone()));
            }
            Self::NotYetServed {
                operation,
                increment,
            } => {
                map.insert(
                    "outcome".to_owned(),
                    Value::Text("not-yet-served".to_owned()),
                );
                map.insert(
                    "operation".to_owned(),
                    Value::Text(operation.name().to_owned()),
                );
                map.insert(
                    "increment".to_owned(),
                    Value::Unsigned(u64::from(*increment)),
                );
            }
            Self::Refused { reason } => {
                map.insert("outcome".to_owned(), Value::Text("refused".to_owned()));
                map.insert("reason".to_owned(), Value::Text(reason.clone()));
            }
        }
        canonical::encode(&Value::Map(map)).map_err(|_| DecodeRefusal::NotCanonical)
    }
}

/// The enumeration response's fields (split out for the line gate; the
/// vocabulary is Section 3 of `schemas/helper/operations.md`).
fn encode_enumeration(
    map: &mut BTreeMap<String, Value>,
    proposal: bool,
    outcome: &str,
    devices: &[EnumeratedDevice],
) {
    map.insert("outcome".to_owned(), Value::Text("enumeration".to_owned()));
    map.insert("proposal".to_owned(), Value::Bool(proposal));
    map.insert("enumeration".to_owned(), Value::Text(outcome.to_owned()));
    map.insert(
        "devices".to_owned(),
        Value::Array(
            devices
                .iter()
                .map(|d| {
                    let mut m = BTreeMap::new();
                    m.insert("selector".to_owned(), Value::Text(d.selector.clone()));
                    m.insert("kind".to_owned(), Value::Text(d.kind.clone()));
                    m.insert("transport".to_owned(), Value::Text(d.transport.clone()));
                    m.insert("properties".to_owned(), Value::Unsigned(d.properties));
                    Value::Map(m)
                })
                .collect(),
        ),
    );
}

/// What answers operations: the helper's one seam for tests. The real
/// backend (`linux::SystemBackend`) enumerates through the adapter.
pub trait Backend {
    /// Status, never failing.
    fn status(&self) -> Response;
    /// Enumeration, never failing — the adapter's arms are values.
    fn enumerate(&self) -> Response;
    /// Validate-plan: take an HLP-002 capture, re-plan the spelled
    /// request over it, and answer `Validated` or `ValidationRefused` —
    /// the arms are values, never errors. The audit sink receives the
    /// capture event (SEC-009).
    fn validate_plan(&self, request: &ValidateWire, audit: &mut dyn AuditSink) -> Response;
}

/// HLP-006's audit vocabulary: closed, and every field a uid, a count,
/// an operation name or this crate's own words — no field can carry a
/// serial, path, label or username (SAFE-006 by construction).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditEvent {
    /// The helper started serving one uid.
    Started {
        /// The uid served.
        uid: u32,
    },
    /// A connection was admitted and handshaken.
    Admitted {
        /// The peer's uid (equals the served uid by construction).
        uid: u32,
        /// The peer's pid.
        pid: i32,
    },
    /// A connection was refused by the transport.
    ConnectionRefused {
        /// The transport's refusal, as its own display.
        reason: String,
    },
    /// An operation was served or refused.
    Operation {
        /// The operation, or `None` when the request itself was refused.
        operation: Option<Operation>,
        /// `served`, `not-yet-served` or `refused`.
        outcome: &'static str,
    },
    /// The helper exited idle.
    IdleExit {
        /// Seconds idle.
        idle_seconds: u64,
    },
    /// The helper computed an authorization tier for a validated plan
    /// (HLP-003, SEC-009). Fixed words only: the tier's own wire name and
    /// one outcome word from this crate's vocabulary. **No plan hash** —
    /// a 64-character digest in a log line is an identifier by any other
    /// name, and this vocabulary's stated property is that no field can
    /// hold one.
    Authorization {
        /// `floor-act` or `interactive-ceremony`.
        tier: &'static str,
        /// This build's only outcome word: `computed`.
        outcome: &'static str,
    },
    /// An HLP-002 capture was taken (the only device access the helper
    /// has before increment 4): counts only, no identifier.
    Captured {
        /// Whole devices the listing admitted.
        devices: u64,
        /// Devices whose table state was authored.
        classified: u64,
    },
}

impl fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Started { uid } => write!(f, "event=started uid={uid}"),
            Self::Admitted { uid, pid } => write!(f, "event=admitted uid={uid} pid={pid}"),
            Self::ConnectionRefused { reason } => {
                write!(f, "event=connection-refused reason={reason:?}")
            }
            Self::Operation { operation, outcome } => write!(
                f,
                "event=operation name={} outcome={outcome}",
                operation.map_or("-", Operation::name)
            ),
            Self::IdleExit { idle_seconds } => {
                write!(f, "event=idle-exit idle_seconds={idle_seconds}")
            }
            Self::Authorization { tier, outcome } => {
                write!(f, "event=authorization tier={tier} outcome={outcome}")
            }
            Self::Captured {
                devices,
                classified,
            } => {
                write!(
                    f,
                    "event=captured devices={devices} classified={classified}"
                )
            }
        }
    }
}

/// Why an audit line could not be recorded.
///
/// One variant: a caller may not act differently on the reason, only on
/// the fact. SEC-009 requires the record; an operation whose record could
/// not be written is refused rather than served silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditRefused;

impl fmt::Display for AuditRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the audit log could not be written")
    }
}

/// Where audit lines go (HLP-006): the consumer appends; tests collect.
///
/// **Fallible on purpose (increment 3).** Increments 1 and 2 discarded
/// the write's result (`let _ = writeln!`), so an operation could be
/// served with no record of it — SEC-009 asks for the log, and a log that
/// silently loses lines is not one. A sink that cannot record refuses,
/// and [`serve_connection`] turns that into a refusal of the operation.
pub trait AuditSink {
    /// Append one event.
    ///
    /// # Errors
    ///
    /// [`AuditRefused`] when the event could not be durably appended.
    fn record(&mut self, event: AuditEvent) -> Result<(), AuditRefused>;
}

/// Serve one admitted connection: read one request frame, decode it
/// strictly, answer through the backend, write one response frame. One
/// request per connection in this increment (the envelope's request
/// channel); the connection closes after. Returns the outcome recorded.
///
/// # Errors
///
/// A transport refusal on the frame; a decode refusal is **answered**,
/// not returned — the peer gets `Response::Refused`.
pub fn serve_connection<S: Read + Write>(
    stream: &mut S,
    backend: &dyn Backend,
    audit: &mut dyn AuditSink,
) -> Result<(), TransportRefusal> {
    let frame = read_frame(stream)?;
    let envelope = match Envelope::decode(&frame) {
        Ok(e) => e,
        Err(refusal) => {
            let response = Response::Refused {
                reason: format!("envelope refused: {refusal:?}"),
            };
            if audit
                .record(AuditEvent::Operation {
                    operation: None,
                    outcome: "refused",
                })
                .is_err()
            {
                return reply(stream, &unaudited());
            }
            return reply(stream, &response);
        }
    };
    let response = match Request::decode(envelope.body()) {
        Err(refusal) => {
            if audit
                .record(AuditEvent::Operation {
                    operation: None,
                    outcome: "refused",
                })
                .is_err()
            {
                return reply(stream, &unaudited());
            }
            Response::Refused {
                reason: refusal_reason(&refusal),
            }
        }
        Ok(request) => {
            if let Some(increment) = request.operation.served_in_increment() {
                if audit
                    .record(AuditEvent::Operation {
                        operation: Some(request.operation),
                        outcome: "not-yet-served",
                    })
                    .is_err()
                {
                    return reply(stream, &unaudited());
                }
                Response::NotYetServed {
                    operation: request.operation,
                    increment,
                }
            } else {
                if audit
                    .record(AuditEvent::Operation {
                        operation: Some(request.operation),
                        outcome: "served",
                    })
                    .is_err()
                {
                    return reply(stream, &unaudited());
                }
                match (request.operation, request.validate.as_ref()) {
                    (Operation::Status, _) => backend.status(),
                    (Operation::Enumerate, _) => backend.enumerate(),
                    (Operation::ValidatePlan, Some(arguments)) => {
                        backend.validate_plan(arguments, audit)
                    }
                    // The strict decode pairs the arguments with exactly
                    // the one operation that takes them; a served
                    // operation outside these arms is a defect answered
                    // as a refusal, never a panic in the privileged
                    // process.
                    _ => Response::Refused {
                        reason: "request shape does not match a served operation".to_owned(),
                    },
                }
            }
        }
    };
    reply(stream, &response)
}

/// SEC-009 fail-closed: an operation whose audit line could not be
/// written is refused, not served silently. The peer is told the fact,
/// never the path or the cause — a refusal is not a channel for
/// reporting the host's filesystem.
fn unaudited() -> Response {
    Response::Refused {
        reason: "the audit log could not be written; no operation is served unrecorded".to_owned(),
    }
}

/// RPC-002: an incompatible version refuses with a remediation naming the
/// version this build speaks, never a debug rendering. The other arms
/// keep their typed rendering, which names a field or a rule and echoes
/// no client content.
fn refusal_reason(refusal: &RequestRefusal) -> String {
    match refusal {
        RequestRefusal::WrongVersion { spoken } => format!(
            "this helper speaks {REQUEST_SCHEMA} version {SCHEMA_VERSION}; the request              spoke version {spoken}. Send version {SCHEMA_VERSION}."
        ),
        other => format!("request refused: {other:?}"),
    }
}

fn reply<S: Write>(stream: &mut S, response: &Response) -> Result<(), TransportRefusal> {
    let body = response.encode().map_err(TransportRefusal::Decode)?;
    let envelope = Envelope::response(body).map_err(TransportRefusal::Decode)?;
    let bytes = envelope.encode().map_err(TransportRefusal::Decode)?;
    write_frame(stream, &bytes)
}

/// Why the launch refused (the round's L2 rules), before any endpoint
/// exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchRefusal {
    /// `PKEXEC_UID` is not set: the helper was not launched through
    /// `pkexec`, so no user vouched for.
    NotLaunchedThroughPkexec,
    /// `PKEXEC_UID` is not a uid.
    PkexecUidUnparsable,
    /// `--serve <uid>` names a user other than the one `pkexec` vouched
    /// for.
    ServeForAnotherUser {
        /// The requested uid.
        requested: u32,
        /// The vouched uid.
        vouched: u32,
    },
    /// A node for this uid already exists: another helper serves it; the
    /// client should connect to that one. Not a failure.
    AlreadyServed,
    /// The endpoint could not be created.
    Endpoint(TransportRefusal),
}

impl fmt::Display for LaunchRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLaunchedThroughPkexec => write!(
                f,
                "not launched through pkexec ({PKEXEC_UID_VARIABLE} unset); the helper serves \
                 only the user pkexec vouches for"
            ),
            Self::PkexecUidUnparsable => write!(f, "{PKEXEC_UID_VARIABLE} is not a uid"),
            Self::ServeForAnotherUser { requested, vouched } => write!(
                f,
                "refusing to serve uid {requested}: pkexec vouched for uid {vouched}"
            ),
            Self::AlreadyServed => {
                write!(f, "a helper already serves this user; connect to its node")
            }
            Self::Endpoint(r) => write!(f, "endpoint: {r}"),
        }
    }
}

impl std::error::Error for LaunchRefusal {}

/// HLP-005's exit decision, as a pure function: exit when idle **and not
/// serving**.
///
/// Lives here rather than in the Linux module so it compiles and is
/// tested on every platform: it is arithmetic over two numbers and a
/// flag, and a predicate that only exists on one target is a predicate
/// only tested on one target. A watchdog ignoring `serving` kills an
/// operation in flight (the delivered defect increment 3 fixes); one
/// ignoring `idle` never exits at all.
#[must_use]
pub const fn should_exit(idle_seconds: u64, bound: u64, serving: bool) -> bool {
    !serving && idle_seconds >= bound
}

/// The launch rule, pure: the requested uid must equal the uid `pkexec`
/// vouched for. Returns the uid to serve.
///
/// # Errors
///
/// [`LaunchRefusal`], the first rule violated.
pub fn launch_rule(requested: u32, pkexec_uid: Option<&str>) -> Result<u32, LaunchRefusal> {
    let vouched = pkexec_uid.ok_or(LaunchRefusal::NotLaunchedThroughPkexec)?;
    let vouched: u32 = vouched
        .trim()
        .parse()
        .map_err(|_| LaunchRefusal::PkexecUidUnparsable)?;
    if vouched != requested {
        return Err(LaunchRefusal::ServeForAnotherUser { requested, vouched });
    }
    Ok(vouched)
}
