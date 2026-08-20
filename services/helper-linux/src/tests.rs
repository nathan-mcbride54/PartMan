use std::collections::BTreeMap;

use partman_domain::canonical::{self, Value};
use partman_rpc::Envelope;
use partman_transport_linux::{read_frame, write_frame};

use crate::{
    AuditEvent, AuditSink, Backend, EnumeratedDevice, LaunchRefusal, Operation, REQUEST_SCHEMA,
    Request, RequestRefusal, Response, SCHEMA_VERSION, TargetSpelling, ValidateWire, launch_rule,
    serve_connection,
};

mod increment2;
mod increment3;

struct FakeBackend;

impl Backend for FakeBackend {
    fn status(&self) -> Response {
        Response::Status {
            build: "0.0.0".to_owned(),
            authorizing_uid: 1000,
            served: vec![Operation::Status, Operation::Enumerate],
        }
    }
    fn enumerate(&self) -> Response {
        Response::Enumeration {
            proposal: true,
            outcome: "listed".to_owned(),
            devices: vec![EnumeratedDevice {
                selector: "device:0".to_owned(),
                kind: "plain".to_owned(),
                transport: "Unrecognized".to_owned(),
                properties: 9,
            }],
        }
    }
    fn validate_plan(&self, _request: &ValidateWire, audit: &mut dyn AuditSink) -> Response {
        if audit
            .record(AuditEvent::Captured {
                devices: 1,
                classified: 1,
            })
            .is_err()
        {
            return Response::ValidationRefused {
                arm: "audit".to_owned(),
                detail: "the audit log could not be written".to_owned(),
            };
        }
        Response::ValidationRefused {
            arm: "planner".to_owned(),
            detail: "the fake backend plans nothing".to_owned(),
        }
    }
}

/// Canned validate-plan arguments for wire tests.
fn canned_validate() -> ValidateWire {
    ValidateWire {
        target: TargetSpelling::Device {
            serial: None,
            wwn: None,
            total_bytes: 1 << 30,
        },
        requested: partman_domain::model::capability::Operation::Wipe,
        plan_id: b"wire-test".to_vec(),
        validity_seconds: 3600,
    }
}

/// A request with the arguments its operation requires.
fn wire_request(operation: Operation) -> Request {
    Request {
        operation,
        validate: (operation == Operation::ValidatePlan).then(canned_validate),
    }
}

/// Collects audit events; the flag makes every write refuse, so the
/// SEC-009 fail-closed arm is reachable in a test.
#[derive(Default)]
struct Collect(Vec<AuditEvent>, bool);

impl AuditSink for Collect {
    fn record(&mut self, event: AuditEvent) -> Result<(), crate::AuditRefused> {
        if self.1 {
            return Err(crate::AuditRefused);
        }
        self.0.push(event);
        Ok(())
    }
}

fn request_frame(body: &[u8]) -> Vec<u8> {
    let env = Envelope::request(body.to_vec()).unwrap();
    let mut wire = Vec::new();
    write_frame(&mut wire, &env.encode().unwrap()).unwrap();
    wire
}

fn decode_reply(wire: &[u8]) -> BTreeMap<String, Value> {
    let frame = read_frame(&mut std::io::Cursor::new(wire)).unwrap();
    let env = Envelope::decode(&frame).unwrap();
    match canonical::decode(env.body()).unwrap() {
        Value::Map(m) => m,
        other => panic!("not a map: {other:?}"),
    }
}

/// One connection over an in-memory duplex: the request bytes are the
/// input, the reply bytes are collected.
struct Duplex {
    input: std::io::Cursor<Vec<u8>>,
    output: Vec<u8>,
}

impl std::io::Read for Duplex {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.input.read(buf)
    }
}

impl std::io::Write for Duplex {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serve(body: &[u8]) -> (BTreeMap<String, Value>, Vec<AuditEvent>) {
    serve_through(body, &FakeBackend)
}

/// Serve one request through any backend — the child module's seam.
fn serve_through(body: &[u8], backend: &dyn Backend) -> (BTreeMap<String, Value>, Vec<AuditEvent>) {
    serve_through_sink(body, backend, Collect::default())
}

/// Serve through a caller-supplied sink, so a sink that refuses every
/// write is reachable from a test (SEC-009's fail-closed arm).
fn serve_through_sink(
    body: &[u8],
    backend: &dyn Backend,
    mut audit: Collect,
) -> (BTreeMap<String, Value>, Vec<AuditEvent>) {
    let mut d = Duplex {
        input: std::io::Cursor::new(request_frame(body)),
        output: Vec::new(),
    };
    serve_connection(&mut d, backend, &mut audit).unwrap();
    (decode_reply(&d.output), audit.0)
}

fn text(m: &BTreeMap<String, Value>, k: &str) -> String {
    match m.get(k) {
        Some(Value::Text(t)) => t.clone(),
        other => panic!("{k}: {other:?}"),
    }
}

// Requirements: HLP-001, RPC-005, CLI-004, RPC-003
//   The operation set is closed by construction: exactly HLP-001's six
//   operations in its order, each with a wire name that round-trips and
//   an increment that serves it (or none, served now); a new variant fails
//   the exhaustive match before any prose can drift. The request decode is
//   strict: an unknown field, a wrong schema or version, a missing
//   operation and an unknown operation — including a path-shaped and a
//   command-shaped one — each refuse typed, so no message carries a path
//   to execute, a command string, or dynamic code.
// Evidence: the_operation_set_is_closed_and_the_request_decode_is_strict
#[test]
#[allow(clippy::too_many_lines)]
fn the_operation_set_is_closed_and_the_request_decode_is_strict() {
    for op in Operation::ALL {
        match op {
            Operation::Status
            | Operation::Enumerate
            | Operation::ValidatePlan
            | Operation::ApplyPlan
            | Operation::Cancel
            | Operation::Resume
            | Operation::JournalQuery => {}
        }
        assert_eq!(Operation::parse(op.name()), Some(op));
        let bytes = wire_request(op).encode().unwrap();
        let decoded = Request::decode(&bytes).unwrap();
        assert_eq!(decoded.operation, op);
        assert_eq!(
            decoded.validate.is_some(),
            op == Operation::ValidatePlan,
            "arguments travel with exactly the one operation that takes them"
        );
    }
    assert_eq!(
        Operation::ALL.map(Operation::name),
        [
            "status",
            "enumerate",
            "validate-plan",
            "apply-plan",
            "cancel",
            "resume",
            "journal-query"
        ]
    );
    assert_eq!(
        Operation::ALL
            .iter()
            .filter(|op| op.served_in_increment().is_none())
            .count(),
        3,
        "this increment serves status, enumerate and validate-plan and names an increment for the rest"
    );
    for name in [
        "/bin/sh -c id",
        "rm -rf /",
        "../../etc/shadow",
        "exec",
        "shell",
        "",
    ] {
        let mut m = BTreeMap::new();
        m.insert("schema".to_owned(), Value::Text(REQUEST_SCHEMA.to_owned()));
        m.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
        m.insert("operation".to_owned(), Value::Text(name.to_owned()));
        let bytes = canonical::encode(&Value::Map(m)).unwrap();
        assert_eq!(
            Request::decode(&bytes),
            Err(RequestRefusal::UnknownOperation),
            "{name:?} is not an operation"
        );
    }
    let mut m = BTreeMap::new();
    m.insert("schema".to_owned(), Value::Text(REQUEST_SCHEMA.to_owned()));
    m.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
    m.insert("operation".to_owned(), Value::Text("status".to_owned()));
    m.insert("path".to_owned(), Value::Text("/dev/sda".to_owned()));
    let bytes = canonical::encode(&Value::Map(m.clone())).unwrap();
    assert_eq!(
        Request::decode(&bytes),
        Err(RequestRefusal::UnknownField {
            key: "path".to_owned()
        })
    );
    m.remove("path");
    m.insert(
        "schema_version".to_owned(),
        Value::Unsigned(SCHEMA_VERSION + 1),
    );
    // A future version is refused as a *version*, not as a schema, so the
    // reply can carry RPC-002's remediation instead of a debug rendering.
    assert_eq!(
        Request::decode(&canonical::encode(&Value::Map(m.clone())).unwrap()),
        Err(RequestRefusal::WrongVersion {
            spoken: SCHEMA_VERSION + 1
        })
    );
    m.insert(
        "schema".to_owned(),
        Value::Text("partman.helper.reqvest".to_owned()),
    );
    assert_eq!(
        Request::decode(&canonical::encode(&Value::Map(m.clone())).unwrap()),
        Err(RequestRefusal::WrongSchema),
        "a wrong schema identity is still a schema refusal"
    );
    m.insert("schema".to_owned(), Value::Text(REQUEST_SCHEMA.to_owned()));
    m.insert("schema_version".to_owned(), Value::Unsigned(SCHEMA_VERSION));
    m.remove("operation");
    assert_eq!(
        Request::decode(&canonical::encode(&Value::Map(m)).unwrap()),
        Err(RequestRefusal::MissingOperation)
    );
    assert_eq!(
        Request::decode(b"not canonical"),
        Err(RequestRefusal::NotAMessage)
    );
}

// Requirements: HLP-001, HLP-006, SAFE-005
//   Serving one connection: a served operation answers through the
//   backend (status names the served set and the uid; enumerate is
//   labelled a proposal and carries kinds, classes and counts, never
//   identifier bytes); an accepted-but-unserved operation answers
//   `not-yet-served` naming its increment — fail-closed, never a stub
//   success; a refused request answers `refused` and runs no backend; and
//   every outcome is audited through the closed event vocabulary.
// Evidence: a_connection_serves_refuses_or_names_the_increment_and_audits_each
#[test]
#[allow(clippy::too_many_lines)]
fn a_connection_serves_refuses_or_names_the_increment_and_audits_each() {
    let (reply, audit) = serve(&wire_request(Operation::Status).encode().unwrap());
    assert_eq!(text(&reply, "outcome"), "status");
    assert_eq!(reply.get("authorizing_uid"), Some(&Value::Unsigned(1000)));
    assert_eq!(
        audit,
        vec![AuditEvent::Operation {
            operation: Some(Operation::Status),
            outcome: "served"
        }]
    );

    let (reply, _) = serve(&wire_request(Operation::Enumerate).encode().unwrap());
    assert_eq!(text(&reply, "outcome"), "enumeration");
    assert_eq!(reply.get("proposal"), Some(&Value::Bool(true)));
    assert_eq!(text(&reply, "enumeration"), "listed");
    match reply.get("devices") {
        Some(Value::Array(devices)) => {
            assert_eq!(devices.len(), 1);
            let Value::Map(d) = &devices[0] else { panic!() };
            assert_eq!(
                d.keys().cloned().collect::<Vec<_>>(),
                ["kind", "properties", "selector", "transport"]
            );
        }
        other => panic!("devices: {other:?}"),
    }

    let (reply, audit) = serve(&wire_request(Operation::ValidatePlan).encode().unwrap());
    assert_eq!(text(&reply, "outcome"), "validation-refused");
    assert_eq!(text(&reply, "arm"), "planner");
    assert_eq!(
        audit,
        vec![
            AuditEvent::Operation {
                operation: Some(Operation::ValidatePlan),
                outcome: "served"
            },
            AuditEvent::Captured {
                devices: 1,
                classified: 1
            }
        ],
        "validate-plan is served in this increment, and the capture is audited"
    );

    for op in [
        Operation::ApplyPlan,
        Operation::Cancel,
        Operation::Resume,
        Operation::JournalQuery,
    ] {
        let (reply, audit) = serve(&wire_request(op).encode().unwrap());
        assert_eq!(text(&reply, "outcome"), "not-yet-served", "{op:?}");
        assert_eq!(text(&reply, "operation"), op.name());
        assert_eq!(
            reply.get("increment"),
            Some(&Value::Unsigned(u64::from(
                op.served_in_increment().unwrap()
            )))
        );
        assert_eq!(
            audit,
            vec![AuditEvent::Operation {
                operation: Some(op),
                outcome: "not-yet-served"
            }]
        );
    }

    // The envelope already refuses a non-canonical body (RPC-003, one
    // validator for both ends), so the wrong-shaped body here is canonical
    // text that is not a request map.
    let (reply, audit) = serve(&canonical::encode(&Value::Text("garbage".to_owned())).unwrap());
    assert_eq!(text(&reply, "outcome"), "refused");
    assert!(text(&reply, "reason").starts_with("request refused"));
    assert_eq!(
        audit,
        vec![AuditEvent::Operation {
            operation: None,
            outcome: "refused"
        }]
    );
}

// Requirements: HLP-006, SAFE-006, SEC-009
//   The audit vocabulary is closed, and no arm can carry an identifier:
//   every field is a uid, a pid, a count, an operation name, a fixed
//   outcome word, or a transport refusal rendered by this crate — a
//   serial, device path, label or username has no field to live in. The
//   closure is held by an exhaustive match; the rendering of every arm
//   is checked for the words it may carry.
// Evidence: the_audit_vocabulary_is_closed_and_carries_no_identifier
#[test]
fn the_audit_vocabulary_is_closed_and_carries_no_identifier() {
    let events = [
        AuditEvent::Started { uid: 1000 },
        AuditEvent::Admitted {
            uid: 1000,
            pid: 4242,
        },
        AuditEvent::ConnectionRefused {
            reason: partman_transport_linux::Refusal::FrameTruncated.to_string(),
        },
        AuditEvent::Operation {
            operation: Some(Operation::Enumerate),
            outcome: "served",
        },
        AuditEvent::Operation {
            operation: None,
            outcome: "refused",
        },
        AuditEvent::IdleExit { idle_seconds: 120 },
        AuditEvent::Captured {
            devices: 14,
            classified: 12,
        },
        AuditEvent::Authorization {
            tier: "interactive-ceremony",
            outcome: "computed",
        },
    ];
    for e in &events {
        match e {
            AuditEvent::Started { .. }
            | AuditEvent::Admitted { .. }
            | AuditEvent::ConnectionRefused { .. }
            | AuditEvent::Operation { .. }
            | AuditEvent::IdleExit { .. }
            | AuditEvent::Captured { .. }
            | AuditEvent::Authorization { .. } => {}
        }
        let line = e.to_string();
        assert!(line.starts_with("event="), "{line}");
        for forbidden in ["/dev/", "S3Z9NB0K", "A20036CA8695D921", "/home/", "muser"] {
            assert!(!line.contains(forbidden), "{line}");
        }
    }
    assert_eq!(events[0].to_string(), "event=started uid=1000");
    assert_eq!(
        events[3].to_string(),
        "event=operation name=enumerate outcome=served"
    );
}

// Requirements: HLP-007, SAFE-002, SAFE-005
//   The launch rule (the round's L2): the helper serves only the user
//   pkexec vouched for. No PKEXEC_UID — refused as not launched through
//   pkexec; an unparsable one refused; a requested uid other than the
//   vouched one refused naming both; the vouched uid itself admitted.
// Evidence: the_launch_rule_serves_only_the_user_pkexec_vouched_for
#[test]
fn the_launch_rule_serves_only_the_user_pkexec_vouched_for() {
    assert_eq!(
        launch_rule(1000, None),
        Err(LaunchRefusal::NotLaunchedThroughPkexec)
    );
    assert_eq!(
        launch_rule(1000, Some("nate")),
        Err(LaunchRefusal::PkexecUidUnparsable)
    );
    assert_eq!(
        launch_rule(1000, Some("0")),
        Err(LaunchRefusal::ServeForAnotherUser {
            requested: 1000,
            vouched: 0
        })
    );
    assert_eq!(
        launch_rule(0, Some("1000")),
        Err(LaunchRefusal::ServeForAnotherUser {
            requested: 0,
            vouched: 1000
        })
    );
    assert_eq!(launch_rule(1000, Some("1000\n")), Ok(1000));
    assert!(LaunchRefusal::AlreadyServed.to_string().contains("connect"));
}

// Requirements: HLP-001, HLP-007, RPC-001
//   The real loop on Linux, over the transport: a helper serving the
//   test's own uid in a temp directory answers status on a real
//   connection through pkexec's vouch (PKEXEC_UID injected) — the
//   endpoint the transport creates, the connection it admits, the
//   request framed and answered; and a second launch for the same uid is
//   AlreadyServed, not a second node. Off Linux the test asserts the
//   typed unsupported-platform refusal of the transport this helper
//   stands on.
// Evidence: a_helper_serves_its_user_over_the_real_transport_and_refuses_a_second_launch
#[test]
#[allow(clippy::too_many_lines)]
fn a_helper_serves_its_user_over_the_real_transport_and_refuses_a_second_launch() {
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(
            partman_transport_linux::platform_support(),
            Err(partman_transport_linux::Refusal::UnsupportedPlatform)
        );
    }
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use partman_rpc::Handshake;
        use partman_transport_linux::linux::{Endpoint, connect};
        use partman_transport_linux::{
            AuthorizingUser, SOCKET_DIRECTORY_MODE, Timeouts, node_name,
        };

        use crate::linux::{SystemBackend, already_served, ensure_directory};

        let me = rustix::process::geteuid().as_raw();
        let dir = std::env::temp_dir().join(format!("partman-helper-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        ensure_directory(&dir).unwrap();
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o7777,
            SOCKET_DIRECTORY_MODE
        );
        let timeouts = Timeouts {
            request_ms: 30_000,
            handshake_ms: 30_000,
        };
        assert!(
            !already_served(&dir, me),
            "no node yet: a launch would serve"
        );
        let endpoint = Endpoint::create(&dir, AuthorizingUser(me), timeouts).unwrap();
        let path = endpoint.path().to_path_buf();
        assert!(fs::symlink_metadata(dir.join(node_name(AuthorizingUser(me)))).is_ok());
        assert!(
            already_served(&dir, me),
            "a second launch for this uid finds the node and is AlreadyServed, not a second endpoint"
        );
        let backend = SystemBackend::new(me, "0.0.0");
        let helper = Handshake::local("0.0.0");
        let t = std::thread::spawn(move || {
            let mut audit = Collect::default();
            for _ in 0..2 {
                let mut conn = endpoint.accept(&helper).unwrap();
                serve_connection(conn.stream(), &backend, &mut audit).unwrap();
            }
            audit.0
        });
        let ask = |op: Operation| -> BTreeMap<String, Value> {
            let mut client = connect(&path, &Handshake::local("0.0.1"), timeouts).unwrap();
            let body = wire_request(op).encode().unwrap();
            let env = Envelope::request(body).unwrap();
            write_frame(client.stream(), &env.encode().unwrap()).unwrap();
            let frame = read_frame(client.stream()).unwrap();
            let reply = Envelope::decode(&frame).unwrap();
            let Value::Map(m) = canonical::decode(reply.body()).unwrap() else {
                panic!()
            };
            m
        };
        let m = ask(Operation::Status);
        assert_eq!(text(&m, "outcome"), "status");
        assert_eq!(
            m.get("authorizing_uid"),
            Some(&Value::Unsigned(u64::from(me)))
        );
        assert_eq!(
            m.get("served"),
            Some(&Value::Array(vec![
                Value::Text("status".to_owned()),
                Value::Text("enumerate".to_owned()),
                Value::Text("validate-plan".to_owned())
            ])),
            "status names exactly the operations this build serves"
        );
        // The system backend over this host's real contract: a proposal,
        // with one of the adapter's four arms; the devices' shape is
        // structural (no identifier bytes), whatever the host has.
        let m = ask(Operation::Enumerate);
        assert_eq!(text(&m, "outcome"), "enumeration");
        assert_eq!(
            m.get("proposal"),
            Some(&Value::Bool(true)),
            "enumeration is labelled the proposal it is"
        );
        assert!(
            ["listed", "over-limit", "unavailable", "failed"]
                .contains(&text(&m, "enumeration").as_str())
        );
        if let Some(Value::Array(devices)) = m.get("devices") {
            for d in devices {
                let Value::Map(d) = d else { panic!() };
                assert_eq!(
                    d.keys().cloned().collect::<Vec<_>>(),
                    ["kind", "properties", "selector", "transport"]
                );
            }
        }
        let audit = t.join().unwrap();
        assert_eq!(audit.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }
}
