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
//! **Identity and logging.** HLP-007 is the transport's (no byte before
//! the verifier). HLP-006's audit log is a closed vocabulary of
//! [`AuditEvent`]s that carry uids, counts, operation names and this
//! crate's own words — no field can hold a serial, path, label or
//! username (SAFE-006 by construction), and a test holds the vocabulary.
//!
//! **What this is not (the assignment's boundary).** No authorization
//! (ADR-0021's act is increment 3's); no device opened (increment 2's);
//! no tool launched; no journal; no network; no operation outside the
//! six. The pure seams compile and test everywhere; the endpoint exists
//! on Linux only.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::BTreeMap;
use std::io::{Read, Write};

use partman_domain::canonical::{self, Value};
use partman_rpc::{DecodeRefusal, Envelope};
use partman_transport_linux::{Refusal as TransportRefusal, read_frame, write_frame};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(test)]
mod tests;

/// The request schema identity (MODEL-003).
pub const REQUEST_SCHEMA: &str = "partman.helper.request";
/// The response schema identity (MODEL-003).
pub const RESPONSE_SCHEMA: &str = "partman.helper.response";
/// Version 1 of both: the six operations, two served.
pub const SCHEMA_VERSION: u64 = 1;
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
            Self::Status | Self::Enumerate => None,
            Self::ValidatePlan => Some(2),
            Self::ApplyPlan => Some(3),
            Self::Cancel | Self::Resume | Self::JournalQuery => Some(4),
        }
    }
}

/// A decoded request: the operation and nothing else in version 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// The operation.
    pub operation: Operation,
}

/// Why a request body was refused. Typed; no peer-authored bytes in the
/// strings beyond a field name, which the strict decode bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestRefusal {
    /// Not canonical, or not a map.
    NotAMessage,
    /// The schema or version is not this one.
    WrongSchema,
    /// A field outside the version's vocabulary (RPC-003 strict).
    UnknownField {
        /// The field name, bounded.
        key: String,
    },
    /// The operation is not one of HLP-001's six.
    UnknownOperation,
    /// The operation field is missing or not text.
    MissingOperation,
}

impl Request {
    /// Encode a request to canonical bytes.
    ///
    /// # Errors
    ///
    /// [`DecodeRefusal::NotCanonical`] if encoding refuses — unreachable for
    /// this flat map, reported rather than panicked.
    pub fn encode(&self) -> Result<Vec<u8>, DecodeRefusal> {
        let mut map = BTreeMap::new();
        map.insert("schema".to_owned(), Value::Text(REQUEST_SCHEMA.to_owned()));
        map.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
        map.insert(
            "operation".to_owned(),
            Value::Text(self.operation.name().to_owned()),
        );
        canonical::encode(&Value::Map(map)).map_err(|_| DecodeRefusal::NotCanonical)
    }

    /// The strict decode: unknown fields and unknown operations refuse.
    ///
    /// # Errors
    ///
    /// [`RequestRefusal`], the first rule violated.
    pub fn decode(bytes: &[u8]) -> Result<Self, RequestRefusal> {
        let value = canonical::decode(bytes).map_err(|_| RequestRefusal::NotAMessage)?;
        let Value::Map(map) = value else {
            return Err(RequestRefusal::NotAMessage);
        };
        for key in map.keys() {
            if !matches!(key.as_str(), "schema" | "schema_version" | "operation") {
                return Err(RequestRefusal::UnknownField {
                    key: key.chars().take(64).collect(),
                });
            }
        }
        match (map.get("schema"), map.get("schema_version")) {
            (Some(Value::Text(s)), Some(Value::Unsigned(v)))
                if s == REQUEST_SCHEMA && *v == SCHEMA_VERSION => {}
            _ => return Err(RequestRefusal::WrongSchema),
        }
        let Some(Value::Text(name)) = map.get("operation") else {
            return Err(RequestRefusal::MissingOperation);
        };
        let operation = Operation::parse(name).ok_or(RequestRefusal::UnknownOperation)?;
        Ok(Self { operation })
    }
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
            } => {
                map.insert("outcome".to_owned(), Value::Text("enumeration".to_owned()));
                map.insert("proposal".to_owned(), Value::Bool(*proposal));
                map.insert("enumeration".to_owned(), Value::Text(outcome.clone()));
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

/// What answers operations: the helper's one seam for tests. The real
/// backend (`linux::SystemBackend`) enumerates through the adapter.
pub trait Backend {
    /// Status, never failing.
    fn status(&self) -> Response;
    /// Enumeration, never failing — the adapter's arms are values.
    fn enumerate(&self) -> Response;
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
        }
    }
}

/// Where audit lines go (HLP-006): the consumer appends; tests collect.
pub trait AuditSink {
    /// Append one event.
    fn record(&mut self, event: AuditEvent);
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
            audit.record(AuditEvent::Operation {
                operation: None,
                outcome: "refused",
            });
            return reply(stream, &response);
        }
    };
    let response = match Request::decode(envelope.body()) {
        Err(refusal) => {
            audit.record(AuditEvent::Operation {
                operation: None,
                outcome: "refused",
            });
            Response::Refused {
                reason: format!("request refused: {refusal:?}"),
            }
        }
        Ok(request) => {
            if let Some(increment) = request.operation.served_in_increment() {
                audit.record(AuditEvent::Operation {
                    operation: Some(request.operation),
                    outcome: "not-yet-served",
                });
                Response::NotYetServed {
                    operation: request.operation,
                    increment,
                }
            } else {
                audit.record(AuditEvent::Operation {
                    operation: Some(request.operation),
                    outcome: "served",
                });
                match request.operation {
                    Operation::Status => backend.status(),
                    Operation::Enumerate => backend.enumerate(),
                    _ => unreachable!("served_in_increment names every unserved operation"),
                }
            }
        }
    };
    reply(stream, &response)
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
