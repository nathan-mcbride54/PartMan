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
use partman_domain::model::identity::{DeviceIdentity, TableState};
use partman_domain::model::naming::{
    AggregateTechnology, FileSystemKind, NamingFields, NodeId, SignatureFamily, TableRole,
    derive_id,
};
use partman_domain::model::plan::{OperationPlan, ValidityWindow};
use partman_domain::model::protection::{Facts, HostRange, StepRanges, TransportClass};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::step::{PlanStep, Severity, StepFlags, StepRisk};
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
    facts.extents.insert(
        sig_id,
        HostRange {
            host: part_id,
            start: 4096,
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

fn snapshot_for(name: &str) -> TopologySnapshot {
    match name {
        "snapshot-minimal-captured" => minimal_captured(),
        "snapshot-minimal-simulated-transitional" => simulated_transitional(),
        "snapshot-full-captured" => full_captured(),
        "snapshot-plan-base-captured" => plan_base().0,
        other => panic!("no construction for snapshot vector {other}"),
    }
}

fn plan_for(name: &str) -> (OperationPlan, TopologySnapshot) {
    let (snapshot, dev_id) = plan_base();
    let step = wipe_step(&snapshot, dev_id);
    let plan = match name {
        "plan-bare-wipe" => OperationPlan::assemble(
            b"vec-plan-bare".to_vec(),
            1_700_000_000,
            &snapshot,
            ValidityWindow {
                not_after: 1_700_086_400,
            },
            BTreeMap::new(),
            vec![step],
        )
        .expect("assembles"),
        "plan-bound-identity-wipe" => {
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
            OperationPlan::assemble(
                b"vec-plan-bound".to_vec(),
                1_700_000_000,
                &snapshot,
                ValidityWindow {
                    not_after: 1_700_086_400,
                },
                identities,
                vec![step],
            )
            .expect("assembles")
        }
        other => panic!("no construction for plan vector {other}"),
    };
    (plan, snapshot)
}

// ---------------------------------------------------------------------------

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
        let (plan, snapshot) = plan_for(name);
        let body = plan.body_value().expect("body");
        assert_matches(entry, &body);

        let bound = entry["snapshot"].as_str().expect("plans name a snapshot");
        let snapshot_entry = named(&fixture, "snapshots", bound);
        assert_eq!(
            hex(plan.snapshot_hash().as_bytes()),
            snapshot_entry["sha256"].as_str().expect("sha256 hex"),
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
