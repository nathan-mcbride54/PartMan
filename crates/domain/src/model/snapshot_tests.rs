//! Tests for the snapshot body, envelope, and typed boundary
//! (WP-010 increment 3c).

use crate::canonical;

use super::naming::AggregateTechnology;
use super::naming::{NamingFields, SignatureFamily, TableRole, derive_id};
use super::protection::{Facts, HostRange, StepRanges, TransportClass, Verdict};
use super::provenance::{Confidence, Method, Observation, Outcome, PropertyObservations};
use super::snapshot::{SnapshotKind, SnapshotSchemaError, TopologySnapshot};
use super::topology::{Edge, EdgeKind};

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

fn small_capture() -> TopologySnapshot {
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, table, signature],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: signature_id,
            },
        ],
        Facts::default(),
    )
    .expect("assembles")
}

fn observation(method: Method, outcome: Outcome) -> Observation {
    Observation {
        adapter: "test-adapter".to_owned(),
        adapter_version: "0".to_owned(),
        method,
        outcome,
    }
}

// Requirements: MODEL-005
//   The body hash is a deterministic function of content, independent of
//   the order nodes and edges were observed in.
// Evidence: the_body_hash_is_independent_of_observation_order
#[test]
fn the_body_hash_is_independent_of_observation_order() {
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let edge = Edge {
        kind: EdgeKind::Containment,
        source: dev_id,
        target: table_id,
    };
    let forward = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev.clone(), table.clone()],
        vec![edge],
        Facts::default(),
    )
    .expect("assembles");
    let reversed = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![table, dev],
        vec![edge],
        Facts::default(),
    )
    .expect("assembles");
    assert_eq!(
        forward.body_hash().expect("hashable"),
        reversed.body_hash().expect("hashable")
    );
}

// Requirements: MODEL-003, MODEL-005, PLAN-006
//   Captured and simulated topologies carry two schema identifiers, so
//   identical content in the two worlds never hashes equal — a simulated
//   topology is structurally incapable of standing where a capture is
//   required.
// Evidence: captured_and_simulated_never_hash_equal
#[test]
fn captured_and_simulated_never_hash_equal() {
    let captured = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![device(b"D0")],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    let simulated = TopologySnapshot::assemble(
        SnapshotKind::Simulated,
        false,
        vec![device(b"D0")],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    assert_ne!(
        captured.body_hash().expect("hashable"),
        simulated.body_hash().expect("hashable")
    );
}

// Requirements: CONC-004, MODEL-005
//   The transitional marking is body content: a transitional snapshot can
//   never be hash-equal to a stable snapshot of the same topology.
// Evidence: a_transitional_snapshot_never_hashes_like_a_stable_one
#[test]
fn a_transitional_snapshot_never_hashes_like_a_stable_one() {
    let stable = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![device(b"D0")],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    let transitional = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        true,
        vec![device(b"D0")],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    assert_ne!(
        stable.body_hash().expect("hashable"),
        transitional.body_hash().expect("hashable")
    );
}

// Requirements: MODEL-004, MODEL-005, PLAN-006
//   The envelope is not in the bytes: editing the capture timestamp or the
//   provenance set moves no body hash, which is what keeps two probes of
//   unchanged hardware comparable.
// Evidence: envelope_edits_never_move_the_body_hash
#[test]
fn envelope_edits_never_move_the_body_hash() {
    let mut snapshot = small_capture();
    let before = snapshot.body_hash().expect("hashable");
    snapshot.envelope.capture_timestamp = Some(1_700_000_000);
    snapshot.envelope.provenance.push((
        "serial".to_owned(),
        PropertyObservations {
            observations: vec![observation(
                Method::Direct,
                Outcome::Observed {
                    value: canonical::Value::Bytes(b"D0".to_vec()),
                },
            )],
        },
    ));
    assert_eq!(snapshot.body_hash().expect("hashable"), before);
}

// Requirements: MODEL-003, MODEL-005, MODEL-006
//   The typed boundary round-trips: encoded body bytes decode, validate,
//   rebuild, and reproduce the exact bytes and hash.
// Evidence: the_typed_boundary_round_trips_exactly
#[test]
fn the_typed_boundary_round_trips_exactly() {
    let snapshot = small_capture();
    let body = snapshot.body_value().expect("body");
    let bytes = canonical::encode(&body).expect("encodable");
    let rebuilt = TopologySnapshot::from_canonical_body(&bytes).expect("round-trips");
    assert_eq!(rebuilt.kind(), snapshot.kind());
    assert_eq!(
        rebuilt.body_hash().expect("hashable"),
        snapshot.body_hash().expect("hashable")
    );
}

// Requirements: MODEL-005
//   The decode-recompute rule at the boundary: an edge forged to violate
//   the endpoint-pair table refuses at decode, because the rebuild runs
//   the same construction the encoder ran.
// Evidence: a_forged_forbidden_edge_refuses_at_decode
#[test]
fn a_forged_forbidden_edge_refuses_at_decode() {
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    // Assemble a VALID snapshot, then forge the edge kind in the value
    // tree: backing signature -> physical device is outside Backing's
    // pair table.
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, signature],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: dev_id,
            target: signature_id,
        }],
        Facts::default(),
    )
    .expect("assembles");
    let body = snapshot.body_value().expect("body");
    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(edges)) = map.get_mut("edges") else {
        panic!("edges present");
    };
    let canonical::Value::Map(edge) = &mut edges[0] else {
        panic!("edge is a map");
    };
    edge.insert(
        "kind".to_owned(),
        canonical::Value::Text("backing".to_owned()),
    );
    // Reverse source and target so the forged backing edge targets the
    // physical device.
    let source = edge.get("source").expect("source").clone();
    let target = edge.get("target").expect("target").clone();
    edge.insert("source".to_owned(), target);
    edge.insert("target".to_owned(), source);
    let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    let result = TopologySnapshot::from_canonical_body(&bytes);
    assert!(
        matches!(result, Err(SnapshotSchemaError::Rebuild(_))),
        "forged edge must refuse at the rebuild: {result:?}"
    );
}

// Requirements: MODEL-005, MODEL-006
//   A mis-sorted node set refuses at the boundary rather than being
//   repaired — the validation pass never sorts for the producer.
// Evidence: a_mis_sorted_set_is_refused_not_repaired
#[test]
fn a_mis_sorted_set_is_refused_not_repaired() {
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![device(b"A"), device(b"B")],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    let body = snapshot.body_value().expect("body");
    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(nodes)) = map.get_mut("nodes") else {
        panic!("nodes present");
    };
    nodes.reverse();
    let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    let result = TopologySnapshot::from_canonical_body(&bytes);
    assert!(
        matches!(result, Err(SnapshotSchemaError::SetOrder(_))),
        "mis-sorted set must refuse: {result:?}"
    );
}

// Requirements: MODEL-003
//   Unknown body fields, unknown schema strings, and unsupported schema
//   versions are typed refusals — the boundary is strict.
// Evidence: unknown_fields_schemas_and_versions_are_refused
#[test]
fn unknown_fields_schemas_and_versions_are_refused() {
    let snapshot = small_capture();
    let body = snapshot.body_value().expect("body");
    let bytes_of = |map: &std::collections::BTreeMap<String, canonical::Value>| {
        canonical::encode(&canonical::Value::Map(map.clone())).expect("encodable")
    };
    let canonical::Value::Map(map) = body else {
        panic!("body is a map");
    };

    let mut extra = map.clone();
    extra.insert("surprise".to_owned(), canonical::Value::Unsigned(1));
    assert!(matches!(
        TopologySnapshot::from_canonical_body(&bytes_of(&extra)),
        Err(SnapshotSchemaError::UnknownField { .. })
    ));

    let mut wrong_schema = map.clone();
    wrong_schema.insert(
        "schema".to_owned(),
        canonical::Value::Text("partman.plan".to_owned()),
    );
    assert!(matches!(
        TopologySnapshot::from_canonical_body(&bytes_of(&wrong_schema)),
        Err(SnapshotSchemaError::WrongSchema)
    ));

    let mut wrong_version = map;
    wrong_version.insert("schema_version".to_owned(), canonical::Value::Unsigned(2));
    assert!(matches!(
        TopologySnapshot::from_canonical_body(&bytes_of(&wrong_version)),
        Err(SnapshotSchemaError::WrongSchemaVersion)
    ));
}

// Requirements: MODEL-005
//   A collision group survives the boundary with its count, and a forged
//   count below two refuses.
// Evidence: collision_groups_round_trip_and_forged_counts_refuse
#[test]
fn collision_groups_round_trip_and_forged_counts_refuse() {
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![device(b"SAME"), device(b"SAME")],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    let body = snapshot.body_value().expect("body");
    let bytes = canonical::encode(&body).expect("encodable");
    let rebuilt = TopologySnapshot::from_canonical_body(&bytes).expect("round-trips");
    assert_eq!(
        rebuilt.body_hash().expect("hashable"),
        snapshot.body_hash().expect("hashable")
    );

    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(nodes)) = map.get_mut("nodes") else {
        panic!("nodes present");
    };
    let canonical::Value::Map(entry) = &mut nodes[0] else {
        panic!("entry is a map");
    };
    entry.insert("collision_count".to_owned(), canonical::Value::Unsigned(1));
    let forged = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    assert!(matches!(
        TopologySnapshot::from_canonical_body(&forged),
        Err(SnapshotSchemaError::BadCollisionCount)
    ));
}

// Requirements: MODEL-004
//   ADR-C4's derivation: one direct observation is authoritative, a
//   heuristic-only one is inferred, none observed is unavailable, and two
//   distinct encodings conflict. Stored confidence has no constructor.
// Evidence: confidence_is_derived_exactly_as_adr_c4_defines
#[test]
fn confidence_is_derived_exactly_as_adr_c4_defines() {
    let observed = |bytes: &[u8]| Outcome::Observed {
        value: canonical::Value::Bytes(bytes.to_vec()),
    };
    let one_direct = PropertyObservations {
        observations: vec![observation(Method::Direct, observed(b"S1"))],
    };
    assert_eq!(
        one_direct.derive_confidence().expect("derivable"),
        Confidence::Authoritative
    );

    let heuristic_only = PropertyObservations {
        observations: vec![observation(Method::Heuristic, observed(b"S1"))],
    };
    assert_eq!(
        heuristic_only.derive_confidence().expect("derivable"),
        Confidence::Inferred
    );

    let nothing = PropertyObservations {
        observations: vec![observation(
            Method::Direct,
            Outcome::Unavailable {
                reason: "no interface".to_owned(),
            },
        )],
    };
    assert_eq!(
        nothing.derive_confidence().expect("derivable"),
        Confidence::Unavailable
    );

    let disagreeing = PropertyObservations {
        observations: vec![
            observation(Method::Direct, observed(b"S1")),
            observation(Method::Direct, observed(b"S2")),
        ],
    };
    assert_eq!(
        disagreeing.derive_confidence().expect("derivable"),
        Confidence::Conflicting
    );
}

// Requirements: MODEL-004
//   A positively observed absence is a value, not an unavailability: it
//   derives a determination alone and conflicts with a presence.
// Evidence: absence_is_a_value_not_an_unavailability
#[test]
fn absence_is_a_value_not_an_unavailability() {
    let absent_only = PropertyObservations {
        observations: vec![observation(Method::Direct, Outcome::ObservedAbsent)],
    };
    assert_eq!(
        absent_only.derive_confidence().expect("derivable"),
        Confidence::Authoritative
    );

    let absent_and_present = PropertyObservations {
        observations: vec![
            observation(Method::Direct, Outcome::ObservedAbsent),
            observation(
                Method::Direct,
                Outcome::Observed {
                    value: canonical::Value::Bytes(b"S1".to_vec()),
                },
            ),
        ],
    };
    assert_eq!(
        absent_and_present.derive_confidence().expect("derivable"),
        Confidence::Conflicting
    );
}

// Requirements: MODEL-005, SAFE-005
//   Facts are body content: an extent, transport, or member count edit
//   moves the body hash, and the facts round-trip through the typed
//   boundary — the verdict's inputs are authenticated.
// Evidence: facts_are_authenticated_body_content
#[test]
fn facts_are_authenticated_body_content() {
    let dev = device(b"D0");
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
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev.clone()],
        vec![],
        facts.clone(),
    )
    .expect("assembles");
    let baseline = snapshot.body_hash().expect("hashable");

    let mut moved = facts.clone();
    moved.transports.insert(dev_id, TransportClass::Usb);
    let with_moved =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![dev], vec![], moved)
            .expect("assembles");
    assert_ne!(with_moved.body_hash().expect("hashable"), baseline);

    let bytes = canonical::encode(&snapshot.body_value().expect("body")).expect("encodable");
    let rebuilt = TopologySnapshot::from_canonical_body(&bytes).expect("round-trips");
    assert_eq!(rebuilt.facts(), snapshot.facts());
}

// Requirements: MODEL-005
//   A fact on a kind that does not carry it is a typed refusal: a
//   transport on an aggregate, a member count on a device.
// Evidence: misplaced_facts_are_typed_refusals
#[test]
fn misplaced_facts_are_typed_refusals() {
    let vg = NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: Some(b"vg".to_vec()),
    };
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![vg],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    let body = snapshot.body_value().expect("body");
    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(nodes)) = map.get_mut("nodes") else {
        panic!("nodes present");
    };
    let canonical::Value::Map(entry) = &mut nodes[0] else {
        panic!("entry is a map");
    };
    entry.insert(
        "transport".to_owned(),
        canonical::Value::Text("sata".to_owned()),
    );
    let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    assert!(matches!(
        TopologySnapshot::from_canonical_body(&bytes),
        Err(SnapshotSchemaError::MisplacedFact { key: "transport" })
    ));
}

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   The full-stack regression: a decoded body's own authenticated facts
//   drive the closure, and initializing the device refuses through the
//   pool — encode, decode, refuse, with no out-of-band input.
// Evidence: a_decoded_body_refuses_the_pool_end_to_end
#[test]
fn a_decoded_body_refuses_the_pool_end_to_end() {
    let sda = device(b"SDA");
    let sda_id = derive_id(&sda).expect("derivable");
    let member = NamingFields::BackingSignature {
        host: sda_id,
        family: SignatureFamily::Zfs,
        primary_offset: 512 << 20,
    };
    let member_id = derive_id(&member).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(sda_id, TransportClass::Sata);
    facts.extents.insert(
        sda_id,
        HostRange {
            host: sda_id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.extents.insert(
        member_id,
        HostRange {
            host: sda_id,
            start: 512 << 20,
            length: 1 << 20,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![sda, member, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: member_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: member_id,
                target: pool_id,
            },
        ],
        facts,
    )
    .expect("assembles");
    let bytes = canonical::encode(&snapshot.body_value().expect("body")).expect("encodable");
    let rebuilt = TopologySnapshot::from_canonical_body(&bytes).expect("round-trips");
    let initialize = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: sda_id,
            start: 0,
            length: 1 << 30,
        }],
    };
    let refusal = rebuilt
        .step_constructs(sda_id, &initialize)
        .expect_err("the decoded body refuses through the pool");
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
    let affected =
        super::protection::affected_set(rebuilt.topology(), rebuilt.facts(), sda_id, &initialize);
    assert!(
        affected.contains(&pool_id),
        "the pool is reached from the decoded body's own facts"
    );
}
