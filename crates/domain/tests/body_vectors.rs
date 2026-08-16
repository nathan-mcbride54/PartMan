//! Rust half of the MODEL-005 parity proof for the increment-3 body
//! schemas: the topology-snapshot body, the plan body, and the node-entry
//! map both build on.
//!
//! This reads `schemas/domain/body-vectors.json`, the same file the
//! TypeScript suite in `packages/canonical` reads, for the same reason the
//! profile vectors are shared: an implementation checked against a table it
//! also owns proves only self-consistency.
//!
//! The constructor tests below are the fixture's provenance: every vector
//! is rebuilt through the real constructors — `TopologySnapshot::assemble`,
//! `PlanStep::mutating`, `OperationPlan::assemble` — and must reproduce the
//! recorded bytes exactly. The fixture cannot drift from the constructors,
//! and a constructor change that moves canonical bytes fails here before it
//! ships. The documents in `schemas/domain/` describe these formats; a
//! field exists because a slice delivered it, never because a document says
//! so.

use std::collections::BTreeMap;
use std::path::PathBuf;

use partman_domain::canonical::{self, Value, encode, hash};
use partman_domain::model::identity::{DeviceIdentity, IndeterminateCause, TableState};
use partman_domain::model::naming::{
    AggregateTechnology, FileSystemKind, NamingFields, NodeId, SignatureFamily, TableRole,
    derive_id,
};
use partman_domain::model::plan::{
    DraftPrecondition, DraftStep, DraftTarget, ImpossibilityReason, OperationPlan, ReversalDraft,
    ReversalLinkage, StepImpossibility, ValidityWindow,
};
use partman_domain::model::protection::{Facts, HostRange, StepRanges, TransportClass};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::step::{
    Acknowledgment, PlanStep, Severity, StepClass, StepFlags, StepRisk,
};
use partman_domain::model::topology::{Edge, EdgeKind};
use serde_json::Value as Json;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas/domain/body-vectors.json")
}

fn fixture() -> Json {
    let text = std::fs::read_to_string(fixture_path()).expect("fixture file exists");
    let fixture: Json = serde_json::from_str(&text).expect("fixture parses");
    assert_eq!(
        fixture["profile"].as_str(),
        Some("pce/1"),
        "the body fixture declares the profile its bytes are encoded in"
    );
    fixture
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

/// Build a value from the fixture's representation — the same shape the
/// profile vectors use: integers as decimal strings, bytes as hex.
fn build(json: &Json) -> Value {
    let object = json.as_object().expect("value is an object");
    let (tag, payload) = object.iter().next().expect("value has one tag");
    match tag.as_str() {
        "uint" => Value::Unsigned(
            payload
                .as_str()
                .expect("uint is a decimal string")
                .parse()
                .expect("uint parses as u64"),
        ),
        "bytes" => {
            let text = payload.as_str().expect("bytes is hex");
            assert!(text.len().is_multiple_of(2), "hex needs an even length");
            Value::Bytes(
                (0..text.len())
                    .step_by(2)
                    .map(|index| {
                        u8::from_str_radix(&text[index..index + 2], 16).expect("valid hex")
                    })
                    .collect(),
            )
        }
        "text" => Value::Text(payload.as_str().expect("text is a string").to_owned()),
        "bool" => Value::Bool(payload.as_bool().expect("bool is a boolean")),
        "array" => Value::Array(
            payload
                .as_array()
                .expect("array is a list")
                .iter()
                .map(build)
                .collect(),
        ),
        "map" => Value::Map(
            payload
                .as_array()
                .expect("map is a list of pairs")
                .iter()
                .map(|pair| {
                    let pair = pair.as_array().expect("pair is a two-element list");
                    (
                        pair[0].as_str().expect("key is a string").to_owned(),
                        build(&pair[1]),
                    )
                })
                .collect::<BTreeMap<String, Value>>(),
        ),
        other => panic!("the body fixture does not use tag {other}"),
    }
}

fn vectors<'a>(fixture: &'a Json, section: &str) -> Vec<&'a Json> {
    fixture.as_object().expect("fixture is an object")[section]
        .as_array()
        .expect("section is a list")
        .iter()
        .collect()
}

fn named<'a>(fixture: &'a Json, section: &str, name: &str) -> &'a Json {
    vectors(fixture, section)
        .into_iter()
        .find(|entry| entry["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("fixture names {section}/{name}"))
}

fn assert_matches(entry: &Json, value: &Value) {
    let name = entry["name"].as_str().expect("vector is named");
    let bytes = encode(value).expect("encodable");
    assert_eq!(
        hex(&bytes),
        entry["canonical"].as_str().expect("canonical hex"),
        "canonical bytes must match for {name}"
    );
    assert_eq!(
        hex(hash(value).expect("hashable").as_bytes()),
        entry["sha256"].as_str().expect("sha256 hex"),
        "digest must match for {name}"
    );
}

// ---------------------------------------------------------------------------
// The constructions, shared by every vector below. These are the fixture's
// provenance: the generator that wrote the fixture ran exactly these.

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

fn minimal_captured() -> TopologySnapshot {
    TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![device(b"VEC-MIN")],
        vec![],
        Facts::default(),
    )
    .expect("assembles")
}

fn simulated_transitional() -> TopologySnapshot {
    TopologySnapshot::assemble(
        SnapshotKind::Simulated,
        true,
        vec![device(b"VEC-MIN")],
        vec![],
        Facts::default(),
    )
    .expect("assembles")
}

fn stamped_table_state() -> TableState {
    TableState::Present {
        checksum: canonical::hash(&Value::Text("body-vectors table checksum".into()))
            .expect("hashable"),
    }
}

fn plan_base() -> (TopologySnapshot, NodeId) {
    let dev = device(b"VEC-PLAN");
    let dev_id = derive_id(&dev).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(dev_id, TransportClass::Sata);
    facts.extents.insert(
        dev_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.table_states.insert(dev_id, stamped_table_state());
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![dev], vec![], facts)
            .expect("assembles");
    (snapshot, dev_id)
}

fn full_captured() -> TopologySnapshot {
    let dev = device(b"VEC-A");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let part_id = derive_id(&part).expect("derivable");
    let sig = NamingFields::BackingSignature {
        host: part_id,
        family: SignatureFamily::Mdraid1x,
        primary_offset: 4096,
    };
    let sig_id = derive_id(&sig).expect("derivable");
    let agg = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"VEC-MD-UUID".to_vec()),
    };
    let agg_id = derive_id(&agg).expect("derivable");
    let vol = NamingFields::Volume {
        producer: agg_id,
        name: b"vec-vol".to_vec(),
        role: None,
    };
    let vol_id = derive_id(&vol).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: vol_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");
    let dup = device(b"VEC-DUP");

    let mut facts = Facts::default();
    facts.transports.insert(dev_id, TransportClass::Sata);
    facts.table_states.insert(dev_id, stamped_table_state());
    // The signature's extent in the containment root's address space —
    // the device's, `start_offset + primary_offset` — as ADR-0037's rule
    // requires and ADR-0046 enforces; the vector was regenerated in that
    // act from its earlier partition-framed spelling.
    facts.extents.insert(
        sig_id,
        HostRange {
            host: dev_id,
            start: (1 << 20) + 4096,
            length: 1 << 16,
        },
    );
    facts.member_counts.insert(agg_id, 2);

    TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, table, part, sig, agg, vol, fs, dup.clone(), dup],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: part_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: part_id,
                target: sig_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: sig_id,
                target: agg_id,
            },
            Edge {
                kind: EdgeKind::Production,
                source: agg_id,
                target: vol_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: vol_id,
                target: fs_id,
            },
        ],
        facts,
    )
    .expect("assembles")
}

fn wipe_step(snapshot: &TopologySnapshot, target: NodeId) -> PlanStep {
    PlanStep::mutating(
        snapshot,
        target,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![],
            destroyed: vec![HostRange {
                host: target,
                start: 0,
                length: 1 << 30,
            }],
        },
        vec![],
        StepRisk {
            severity: Severity::Destructive,
            flags: StepFlags::default(),
        },
    )
    .expect("constructs")
}

/// The created range the v2 create/draft vectors share: 10 MiB at
/// 1 MiB on the plan-base device.
fn created_range(dev_id: NodeId) -> HostRange {
    HostRange {
        host: dev_id,
        start: 1 << 20,
        length: 10 << 20,
    }
}

/// The forward create step over the plan-base capture.
fn create_step(snapshot: &TopologySnapshot, dev_id: NodeId) -> PlanStep {
    PlanStep::mutating(
        snapshot,
        dev_id,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![created_range(dev_id)],
            destroyed: vec![],
        },
        vec![],
        StepRisk {
            severity: Severity::Disruptive,
            flags: StepFlags::default(),
        },
    )
    .expect("constructs")
}

/// The simulated prediction of the plan-base device after the create:
/// the minted partition placed at its range, under a GPT table view.
fn simulated_created() -> TopologySnapshot {
    let dev = device(b"VEC-PLAN");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let part_id = derive_id(&part).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(dev_id, TransportClass::Sata);
    facts.extents.insert(
        dev_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.extents.insert(part_id, created_range(dev_id));
    facts.table_states.insert(dev_id, stamped_table_state());
    TopologySnapshot::assemble(
        SnapshotKind::Simulated,
        false,
        vec![dev, table, part],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: part_id,
            },
        ],
        facts,
    )
    .expect("assembles")
}

/// The create-reversal draft (ADR-0022): one step destroying the
/// created range, target spelled as the forward step's output,
/// truthfulness carried as the created node's emptiness.
fn draft_create_reversal() -> ReversalDraft {
    let (snapshot, dev_id) = plan_base();
    let forward_step = create_step(&snapshot, dev_id);
    ReversalDraft::compose(
        b"vec-plan-fwd/reversal".to_vec(),
        1_700_000_000,
        &simulated_created(),
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        vec![DraftStep {
            target: DraftTarget::StepOutput(0),
            ranges: StepRanges {
                written_table_extents: vec![],
                consumed: vec![],
                destroyed: vec![created_range(dev_id)],
            },
            acknowledgments: vec![],
            risk: StepRisk {
                severity: Severity::Reversible,
                flags: StepFlags::default(),
            },
            preconditions: vec![DraftPrecondition::StepOutputUnoccupied { step: 0 }],
        }],
        b"vec-plan-fwd".to_vec(),
        std::slice::from_ref(&forward_step),
    )
    .expect("the draft composes against the prediction")
}

/// A device whose authored table state is `Indeterminate`, its damaged
/// table located as a child extent — the world ADR-0024's repair arm
/// exists for.
fn indeterminate_table_snapshot() -> (TopologySnapshot, NodeId, HostRange) {
    let dev = device(b"VEC-REPAIR");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let table_region = HostRange {
        host: dev_id,
        start: 0,
        length: 17_408,
    };
    let mut facts = Facts::default();
    facts.transports.insert(dev_id, TransportClass::Sata);
    facts.extents.insert(
        dev_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.extents.insert(table_id, table_region);
    facts.table_states.insert(
        dev_id,
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, table],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: dev_id,
            target: table_id,
        }],
        facts,
    )
    .expect("assembles");
    (snapshot, dev_id, table_region)
}

fn snapshot_for(name: &str) -> TopologySnapshot {
    match name {
        "snapshot-minimal-captured" => minimal_captured(),
        "snapshot-minimal-simulated-transitional" => simulated_transitional(),
        "snapshot-full-captured" => full_captured(),
        "snapshot-plan-base-captured" => plan_base().0,
        "snapshot-plan-base-simulated-created" => simulated_created(),
        "snapshot-plan-base-indeterminate-table" => indeterminate_table_snapshot().0,
        other => panic!("no construction for snapshot vector {other}"),
    }
}

fn plan_for(name: &str) -> (OperationPlan, TopologySnapshot) {
    if name == "plan-v4-table-repair-acknowledged" {
        return table_repair_plan();
    }
    let (snapshot, dev_id) = plan_base();
    let step = wipe_step(&snapshot, dev_id);
    let plan = match name {
        // The identity-record coverage the retired version-1 vectors
        // carried (SAFE-003, plan-body.md §2), on the live version.
        "plan-v4-bound-identity-wipe" => {
            let mut identities = BTreeMap::new();
            identities.insert(
                dev_id,
                DeviceIdentity {
                    serial: Some(b"VEC-PLAN".to_vec()),
                    wwn: None,
                    os_instance_id: Some(b"vec-os-0".to_vec()),
                    connection_path: Some(b"pci-0000:00:1f.2-ata-1".to_vec()),
                    total_bytes: 1 << 30,
                    logical_sector_size: Some(512),
                    physical_sector_size: Some(4096),
                    table: stamped_table_state(),
                    witness: None,
                },
            );
            OperationPlan::assemble_linked(
                b"vec-plan-bound".to_vec(),
                1_700_000_000,
                &snapshot,
                ValidityWindow {
                    not_after: 1_700_086_400,
                },
                identities,
                vec![step],
                ReversalLinkage::Impossible {
                    statements: vec![StepImpossibility {
                        step: 0,
                        reason: ImpossibilityReason::DataDestroyed,
                    }],
                },
            )
            .expect("assembles")
        }
        "plan-v4-wipe-impossible" => OperationPlan::assemble_linked(
            b"vec-plan-v3".to_vec(),
            1_700_000_000,
            &snapshot,
            ValidityWindow {
                not_after: 1_700_086_400,
            },
            BTreeMap::new(),
            vec![step],
            ReversalLinkage::Impossible {
                statements: vec![StepImpossibility {
                    step: 0,
                    reason: ImpossibilityReason::DataDestroyed,
                }],
            },
        )
        .expect("assembles"),
        "plan-v4-forward-create-draft-linked" => {
            let draft = draft_create_reversal();
            OperationPlan::assemble_linked(
                b"vec-plan-fwd".to_vec(),
                1_700_000_000,
                &snapshot,
                ValidityWindow {
                    not_after: 1_700_086_400,
                },
                BTreeMap::new(),
                vec![create_step(&snapshot, dev_id)],
                ReversalLinkage::Draft {
                    plan_id: draft.plan_id().to_vec(),
                    draft_hash: draft.body_hash().expect("hashable"),
                },
            )
            .expect("assembles")
        }
        other => panic!("no construction for plan vector {other}"),
    };
    (plan, snapshot)
}

/// The table-repair plan over the indeterminate-table world: the typed
/// repair-family step carrying the capture-impossible acknowledgment
/// (ADR-0024), its reversal the pre-state-preserved statement.
fn table_repair_plan() -> (OperationPlan, TopologySnapshot) {
    let (snapshot, dev_id, table_region) = indeterminate_table_snapshot();
    let step = PlanStep::mutating_classed(
        &snapshot,
        dev_id,
        StepRanges {
            written_table_extents: vec![table_region],
            consumed: vec![],
            destroyed: vec![],
        },
        vec![Acknowledgment::UncapturableRegions {
            table: dev_id,
            regions: vec![table_region],
        }],
        StepRisk {
            severity: Severity::Disruptive,
            flags: StepFlags::default(),
        },
        StepClass::TableRepair,
    )
    .expect("the repair family constructs on the indeterminate table");
    let plan = OperationPlan::assemble_linked(
        b"vec-plan-repair".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::PreStatePreservedForRecovery,
            }],
        },
    )
    .expect("assembles");
    (plan, snapshot)
}

// ---------------------------------------------------------------------------

#[test]
#[ignore = "generator: prints new fixture entries as JSON"]
fn print_new_vectors() {
    fn to_json(value: &Value) -> Json {
        match value {
            Value::Unsigned(number) => serde_json::json!({"uint": number.to_string()}),
            Value::Bytes(bytes) => serde_json::json!({"bytes": hex(bytes)}),
            Value::Text(text) => serde_json::json!({"text": text}),
            Value::Bool(flag) => serde_json::json!({"bool": flag}),
            Value::Array(items) => {
                serde_json::json!({"array": items.iter().map(to_json).collect::<Vec<_>>()})
            }
            Value::Map(map) => serde_json::json!({
                "map": map
                    .iter()
                    .map(|(key, entry)| serde_json::json!([key, to_json(entry)]))
                    .collect::<Vec<_>>()
            }),
            _ => panic!("tag not used by body vectors"),
        }
    }
    let entry = |name: &str, snapshot: Option<&str>, value: &Value| {
        let mut object = serde_json::Map::new();
        object.insert("name".into(), serde_json::json!(name));
        if let Some(snapshot) = snapshot {
            object.insert("snapshot".into(), serde_json::json!(snapshot));
        }
        object.insert("value".into(), to_json(value));
        object.insert(
            "canonical".into(),
            serde_json::json!(hex(&encode(value).expect("encodable"))),
        );
        object.insert(
            "sha256".into(),
            serde_json::json!(hex(hash(value).expect("hashable").as_bytes())),
        );
        println!(
            "{},",
            serde_json::to_string_pretty(&Json::Object(object)).expect("serializes")
        );
    };

    // The full capture and its node entries, regenerated by ADR-0046 when
    // the signature's extent moved into the containment root's frame.
    let full = full_captured().body_value().expect("body");
    entry("snapshot-full-captured", None, &full);
    if let Value::Map(map) = &full
        && let Some(Value::Array(nodes)) = map.get("nodes")
    {
        for (index, node) in nodes.iter().enumerate() {
            let kind = match node {
                Value::Map(fields) => match fields.get("kind") {
                    Some(Value::Text(kind)) => kind.clone(),
                    _ => panic!("node carries a kind"),
                },
                _ => panic!("node is a map"),
            };
            let group = match node {
                Value::Map(fields) if fields.contains_key("count") => "-group",
                _ => "",
            };
            entry(
                &format!("node-entry-{kind}-{index}{group}"),
                Some("snapshot-full-captured"),
                node,
            );
        }
    }
    let simulated = simulated_created();
    entry(
        "snapshot-plan-base-simulated-created",
        None,
        &simulated.body_value().expect("body"),
    );
    entry(
        "snapshot-plan-base-indeterminate-table",
        None,
        &indeterminate_table_snapshot().0.body_value().expect("body"),
    );
    for name in [
        "plan-v4-wipe-impossible",
        "plan-v4-forward-create-draft-linked",
        "plan-v4-bound-identity-wipe",
    ] {
        let (plan, _) = plan_for(name);
        entry(
            name,
            Some("snapshot-plan-base-captured"),
            &plan.body_value().expect("body"),
        );
    }
    let (repair, _) = table_repair_plan();
    entry(
        "plan-v4-table-repair-acknowledged",
        Some("snapshot-plan-base-indeterminate-table"),
        &repair.body_value().expect("body"),
    );
    let draft = draft_create_reversal();
    entry(
        "draft-create-reversal",
        Some("snapshot-plan-base-simulated-created"),
        &draft.body_value(),
    );
}

// Requirements: MODEL-005, MODEL-003, MODEL-006
//   Every recorded body vector's value tree encodes to its recorded
//   canonical bytes and digest through the generic codec — the table the
//   TypeScript suite must reproduce byte for byte.
// Evidence: every_fixture_vector_encodes_to_its_recorded_bytes
#[test]
fn every_fixture_vector_encodes_to_its_recorded_bytes() {
    let fixture = fixture();
    let mut checked = 0;
    for section in ["snapshots", "plans", "node_entries"] {
        for entry in vectors(&fixture, section) {
            assert_matches(entry, &build(&entry["value"]));
            checked += 1;
        }
    }
    assert!(checked >= 15, "the fixture must not quietly shrink");
}

// Requirements: MODEL-005, MODEL-003
//   The snapshot vectors are the constructors' own output: each
//   construction reproduces its recorded bytes exactly, and the typed
//   boundary round-trips them. The fixture cannot drift from the code.
// Evidence: snapshot_constructions_reproduce_their_recorded_bytes
#[test]
fn snapshot_constructions_reproduce_their_recorded_bytes() {
    let fixture = fixture();
    for entry in vectors(&fixture, "snapshots") {
        let name = entry["name"].as_str().expect("named");
        let snapshot = snapshot_for(name);
        let body = snapshot.body_value().expect("body");
        assert_matches(entry, &body);
        let bytes = encode(&body).expect("encodable");
        let rebuilt = TopologySnapshot::from_canonical_body(&bytes).expect("round-trips");
        assert_eq!(
            rebuilt.body_hash().expect("hash"),
            snapshot.body_hash().expect("hash"),
            "typed boundary must reproduce {name}"
        );
    }
}

// Requirements: MODEL-005, PLAN-006, PLAN-007, SAFE-003
//   The plan vectors are the constructors' own output, their embedded
//   snapshot hash is byte-equal to the named snapshot vector's digest —
//   the PLAN-006 binding held across the fixture — and the typed boundary
//   revalidates them against that snapshot.
// Evidence: plan_constructions_reproduce_and_bind_their_snapshot
#[test]
fn plan_constructions_reproduce_and_bind_their_snapshot() {
    let fixture = fixture();
    for entry in vectors(&fixture, "plans") {
        let name = entry["name"].as_str().expect("named");
        let bound = entry["snapshot"].as_str().expect("plans name a snapshot");
        let snapshot_entry = named(&fixture, "snapshots", bound);
        let recorded_digest = snapshot_entry["sha256"].as_str().expect("sha256 hex");

        // A draft is a plan-shaped body whose snapshot hash is its
        // simulated proposal's; it round-trips its own boundary, never
        // the plain one (the step-output spelling refuses there).
        if name.starts_with("draft-") {
            let draft = match name {
                "draft-create-reversal" => draft_create_reversal(),
                other => panic!("no construction for draft vector {other}"),
            };
            let body = draft.body_value();
            assert_matches(entry, &body);
            let Value::Map(map) = &body else {
                panic!("draft body is a map")
            };
            let Some(Value::Bytes(proposal_hash)) = map.get("snapshot_hash") else {
                panic!("draft carries its proposal hash")
            };
            assert_eq!(
                hex(proposal_hash),
                recorded_digest,
                "{name} must carry the digest the fixture records for {bound}"
            );
            let bytes = encode(&body).expect("encodable");
            let rebuilt = ReversalDraft::from_canonical_body(&bytes).expect("round-trips");
            assert_eq!(rebuilt, draft, "typed draft boundary must reproduce {name}");
            continue;
        }

        let (plan, snapshot) = plan_for(name);
        let body = plan.body_value().expect("body");
        assert_matches(entry, &body);
        assert_eq!(
            hex(plan.snapshot_hash().as_bytes()),
            recorded_digest,
            "{name} must bind the digest the fixture records for {bound}"
        );

        let bytes = encode(&body).expect("encodable");
        let rebuilt = OperationPlan::from_canonical_body(&bytes, &snapshot).expect("revalidates");
        assert_eq!(
            rebuilt.body_hash().expect("hash"),
            plan.body_hash().expect("hash"),
            "typed boundary must reproduce {name}"
        );
    }
}

// Requirements: MODEL-005, MODEL-006
//   Every node-entry vector is byte-identical to the corresponding
//   element of its snapshot's `nodes` set, so the standalone table and
//   the in-body encoding can never disagree.
// Evidence: node_entries_match_their_snapshot_bodies
#[test]
fn node_entries_match_their_snapshot_bodies() {
    let fixture = fixture();
    for entry in vectors(&fixture, "node_entries") {
        let name = entry["name"].as_str().expect("named");
        let value = build(&entry["value"]);
        assert_matches(entry, &value);

        let owner = entry["snapshot"].as_str().expect("entries name a snapshot");
        let snapshot = snapshot_for(owner);
        let body = snapshot.body_value().expect("body");
        let Value::Map(map) = &body else {
            panic!("body is a map")
        };
        let Some(Value::Array(nodes)) = map.get("nodes") else {
            panic!("body carries nodes")
        };
        assert!(
            nodes.contains(&value),
            "{name} must appear verbatim in {owner}'s nodes set"
        );
    }
}
