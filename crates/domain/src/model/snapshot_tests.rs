//! Tests for the snapshot body, envelope, and typed boundary
//! (WP-010 increment 3c).

use crate::canonical;

use super::naming::AggregateTechnology;
use super::naming::{NamingFields, SignatureFamily, TableRole, derive_id};
use super::protection::{FactError, Facts, HostRange, StepRanges, TransportClass, Verdict};
use super::provenance::{Confidence, Method, Observation, Outcome, PropertyObservations};
use super::snapshot::{SnapshotError, SnapshotKind, SnapshotSchemaError, TopologySnapshot};
use super::topology::{Edge, EdgeKind, TopologyError};

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

// Requirements: MODEL-005, MODEL-002
//   The decode-recompute rule for names (issue #354's kind half, ADR-0045):
//   a body whose partition names the physical device as its `parent_table`
//   — the issue's second probe, which decoded cleanly at `8e03e68` — refuses
//   at the rebuild with the constructor's own refusal, naming the pairing.
//   The forged body is otherwise byte-lawful (the field's value is a real
//   node's address, and the node's own address re-derives from it), so what
//   refuses is the pairing and nothing else. `SCHEMA_VERSION` is unchanged:
//   the refused population was never lawful under MODEL-002, only
//   unvalidated (MODEL-003's explicit-rejection limb, as #362 read it).
// Evidence: a_wrong_kind_naming_referent_refuses_at_decode
#[test]
fn a_wrong_kind_naming_referent_refuses_at_decode() {
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let partition = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let forged = NamingFields::Partition {
        parent_table: dev_id,
        start_offset: 1 << 20,
    };
    let forged_id = derive_id(&forged).expect("derivable");
    let in_process = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev.clone(), table.clone(), forged],
        vec![],
        Facts::default(),
    )
    .expect_err("assembly refuses the pairing");
    assert_eq!(
        in_process,
        SnapshotError::Topology(TopologyError::ForbiddenNamingReferent {
            node: forged_id,
            kind: "partition",
            field: "parent_table",
            referent: dev_id,
            referent_kind: "physical-device",
        })
    );

    let honest = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, table, partition],
        vec![],
        Facts::default(),
    )
    .expect("assembles");
    let body = honest.body_value().expect("body");
    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(nodes)) = map.get_mut("nodes") else {
        panic!("nodes present");
    };
    let entry = nodes
        .iter_mut()
        .find_map(|entry| match entry {
            canonical::Value::Map(entry)
                if entry.get("kind") == Some(&canonical::Value::Text("partition".to_owned())) =>
            {
                Some(entry)
            }
            _ => None,
        })
        .expect("the partition's entry");
    entry.insert(
        "parent_table".to_owned(),
        canonical::Value::Bytes(dev_id.as_bytes().to_vec()),
    );
    resort(nodes);
    let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    let at_boundary = TopologySnapshot::from_canonical_body(&bytes).expect_err("decode refuses");
    assert_eq!(
        at_boundary,
        SnapshotSchemaError::Rebuild(in_process),
        "the boundary's refusal is the constructor's refusal"
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
        Err(SnapshotSchemaError::Rebuild(SnapshotError::Facts(
            FactError::MisplacedFact {
                fact: "transport",
                ..
            }
        )))
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

// ---------------------------------------------------------------------
// Body validity: the facts are refused against the topology at assembly
// (issues #349 and #356; ADR-0041).
// ---------------------------------------------------------------------

/// A device carrying one partition table and one partition, the
/// partition's extent supplied by the caller. Returned unassembled so a
/// test can vary exactly one thing and ask whether the whole assembles.
fn one_partition(
    partition_extent: Option<HostRange>,
) -> (
    Vec<NamingFields>,
    Vec<Edge>,
    Facts,
    super::naming::NodeId,
    super::naming::NodeId,
) {
    let dev = device(b"V0");
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
    if let Some(extent) = partition_extent {
        facts.extents.insert(part_id, extent);
    }
    let edges = vec![
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
    ];
    (vec![dev, table, part], edges, facts, dev_id, part_id)
}

/// Re-sort a forged node set the way `sorted_set` does, so a forgery is
/// judged by the rule under test and not by MODEL-006's set order.
fn resort(entries: &mut [canonical::Value]) {
    entries.sort_by_cached_key(|entry| canonical::encode(entry).expect("encodable"));
}

fn assemble_result(
    nodes: Vec<NamingFields>,
    edges: Vec<Edge>,
    facts: Facts,
) -> Result<TopologySnapshot, SnapshotError> {
    TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges, facts)
}

// Requirements: MODEL-005, SAFE-005
//   An extent triple must be a range (issue #349): a zero-length extent
//   is a claim about no bytes and is invisible to the byte scan; one whose
//   `start + length` overflows has no end; one framed on an address no
//   entry carries lives in no address space. Each refuses at assembly
//   with the node named, and the honest triple assembles. Absence is not
//   refused: a partition with no extent still assembles, because absence
//   is honest and fails closed at the arm that needs it.
// Evidence: an_extent_that_is_not_a_range_refuses_at_assembly
#[test]
fn an_extent_that_is_not_a_range_refuses_at_assembly() {
    let ghost = derive_id(&device(b"GHOST")).expect("derivable");
    let (nodes, edges, facts, dev_id, part_id) = one_partition(None);
    assert!(
        assemble_result(nodes, edges, facts).is_ok(),
        "an absent extent is honest absence, never a refusal"
    );

    let honest = HostRange {
        host: dev_id,
        start: 1 << 20,
        length: 256 << 20,
    };
    let cases = [
        (
            HostRange {
                length: 0,
                ..honest
            },
            FactError::ZeroLengthExtent { node: part_id },
        ),
        (
            HostRange {
                start: u64::MAX - 1,
                length: 2,
                ..honest
            },
            FactError::ExtentOverflows { node: part_id },
        ),
        (
            HostRange {
                host: ghost,
                ..honest
            },
            FactError::UnresolvedExtentHost {
                node: part_id,
                host: ghost,
            },
        ),
    ];
    for (forged, expected) in cases {
        let (nodes, edges, facts, _, _) = one_partition(Some(forged));
        assert_eq!(
            assemble_result(nodes, edges, facts).err(),
            Some(SnapshotError::Facts(expected))
        );
    }
    let (nodes, edges, facts, _, _) = one_partition(Some(honest));
    assert!(assemble_result(nodes, edges, facts).is_ok());
    // The boundary case: an extent ending exactly at `u64::MAX` is a
    // range, not an overflow. The table→partition pair carries no span
    // claim, so nothing else refuses it either.
    let (nodes, edges, facts, _, _) = one_partition(Some(HostRange {
        start: u64::MAX - 1,
        length: 1,
        ..honest
    }));
    assert!(assemble_result(nodes, edges, facts).is_ok());
}

// Requirements: MODEL-005, SAFE-005
//   A fact keyed by an address no entry carries never enters the body
//   bytes, so an in-process snapshot holding one and its own encoding
//   would disagree about what facts exist. Every fact kind refuses at
//   assembly with the key and the address named.
// Evidence: an_orphan_fact_refuses_at_assembly
#[test]
fn an_orphan_fact_refuses_at_assembly() {
    let ghost = derive_id(&device(b"GHOST")).expect("derivable");
    let (nodes, edges, honest, dev_id, _) = one_partition(None);
    let dev_extent = honest.extents[&dev_id];

    let mut extents = honest.clone();
    extents.extents.insert(ghost, dev_extent);
    let mut transports = honest.clone();
    transports.transports.insert(ghost, TransportClass::Sata);
    let mut counts = honest.clone();
    counts.member_counts.insert(ghost, 2);
    let mut states = honest.clone();
    states
        .table_states
        .insert(ghost, super::identity::TableState::Absent);

    for (fact, facts) in [
        ("extent_host", extents),
        ("transport", transports),
        ("member_count", counts),
        ("table_state", states),
    ] {
        assert_eq!(
            assemble_result(nodes.clone(), edges.clone(), facts).err(),
            Some(SnapshotError::Facts(FactError::OrphanFact {
                fact,
                node: ghost
            })),
            "orphan {fact}"
        );
    }
}

// Requirements: MODEL-005, MODEL-003, SAFE-005
//   `assemble` and `from_canonical_body` are one path (issue #349's fourth
//   defect): the decode boundary rebuilds through `assemble`, so every fact
//   the boundary refuses, the in-process constructor refuses with the
//   same typed error — and vice versa. Measured for all four misplaced
//   facts: the boundary's refusal carries the constructor's refusal as
//   its payload, equal by value. Under MODEL-003 this is the
//   explicit-rejection limb — bodies carrying a fact on a kind that
//   cannot carry it were never lawful, only unvalidated on one path.
// Evidence: assembly_and_decode_refuse_the_same_facts
#[test]
#[allow(clippy::too_many_lines)]
fn assembly_and_decode_refuse_the_same_facts() {
    type Case = (
        &'static str,
        Vec<(&'static str, canonical::Value)>,
        Facts,
        super::naming::NodeId,
    );
    let vg = NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: Some(b"vg".to_vec()),
    };
    let vg_id = derive_id(&vg).expect("derivable");
    let dev = device(b"V1");
    let dev_id = derive_id(&dev).expect("derivable");
    let honest = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![vg.clone(), dev.clone()],
        vec![],
        Facts::default(),
    )
    .expect("assembles");

    let mut extent_on_vg = Facts::default();
    extent_on_vg.extents.insert(
        vg_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 4096,
        },
    );
    let mut transport_on_vg = Facts::default();
    transport_on_vg
        .transports
        .insert(vg_id, TransportClass::Sata);
    let mut count_on_dev = Facts::default();
    count_on_dev.member_counts.insert(dev_id, 2);
    let mut state_on_vg = Facts::default();
    state_on_vg
        .table_states
        .insert(vg_id, super::identity::TableState::Absent);

    // (body key, the forged body fields, the same claim as in-process
    // facts, the node it lands on)
    let cases: [Case; 4] = [
        (
            "extent_host",
            vec![
                (
                    "extent_host",
                    canonical::Value::Bytes(dev_id.as_bytes().to_vec()),
                ),
                ("extent_start", canonical::Value::Unsigned(0)),
                ("extent_length", canonical::Value::Unsigned(4096)),
            ],
            extent_on_vg,
            vg_id,
        ),
        (
            "transport",
            vec![("transport", canonical::Value::Text("sata".to_owned()))],
            transport_on_vg,
            vg_id,
        ),
        (
            "member_count",
            vec![("member_count", canonical::Value::Unsigned(2))],
            count_on_dev,
            dev_id,
        ),
        (
            "table_state",
            vec![(
                "table_state",
                super::identity::table_value(&super::identity::TableState::Absent),
            )],
            state_on_vg,
            vg_id,
        ),
    ];

    for (fact, forged_fields, facts, node) in cases {
        let in_process = TopologySnapshot::assemble(
            SnapshotKind::Captured,
            false,
            vec![vg.clone(), dev.clone()],
            vec![],
            facts,
        )
        .expect_err("assembly refuses");
        let SnapshotError::Facts(FactError::MisplacedFact {
            fact: refused_fact,
            node: refused_node,
            ..
        }) = &in_process
        else {
            panic!("{fact}: expected a misplaced-fact refusal, got {in_process:?}");
        };
        assert_eq!((*refused_fact, *refused_node), (fact, node));

        let body = honest.body_value().expect("body");
        let canonical::Value::Map(mut map) = body else {
            panic!("body is a map");
        };
        let Some(canonical::Value::Array(nodes)) = map.get_mut("nodes") else {
            panic!("nodes present");
        };
        let target = nodes
            .iter_mut()
            .find_map(|entry| match entry {
                canonical::Value::Map(entry)
                    if super::naming::fields_from_map(entry)
                        .ok()
                        .and_then(|fields| derive_id(&fields).ok())
                        == Some(node) =>
                {
                    Some(entry)
                }
                _ => None,
            })
            .expect("the forged node's entry");
        for (key, value) in forged_fields {
            target.insert(key.to_owned(), value);
        }
        resort(nodes);
        let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
        let at_boundary =
            TopologySnapshot::from_canonical_body(&bytes).expect_err("decode refuses");
        assert_eq!(
            at_boundary,
            SnapshotSchemaError::Rebuild(in_process),
            "{fact}: the boundary's refusal is the constructor's refusal"
        );
    }
}

/// Issue #356's measured topology: a device, a partition at
/// `[0, 100 MiB)`, and a ZFS signature the containment edge nests in the
/// partition. The signature's extent is the caller's.
fn partition_carrying_signature(
    signature_extent: HostRange,
) -> (
    Vec<NamingFields>,
    Vec<Edge>,
    Facts,
    [super::naming::NodeId; 4],
) {
    const MIB: u64 = 1 << 20;
    let sda = device(b"SDA");
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 0,
    };
    let part_id = derive_id(&part).expect("derivable");
    let sig = NamingFields::BackingSignature {
        host: part_id,
        family: SignatureFamily::Zfs,
        primary_offset: MIB,
    };
    let sig_id = derive_id(&sig).expect("derivable");
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
        part_id,
        HostRange {
            host: sda_id,
            start: 0,
            length: 100 * MIB,
        },
    );
    facts.extents.insert(sig_id, signature_extent);
    let edges = vec![
        Edge {
            kind: EdgeKind::Containment,
            source: sda_id,
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
    ];
    (
        vec![sda, table, part, sig],
        edges,
        facts,
        [sda_id, table_id, part_id, sig_id],
    )
}

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   A containment edge and an extent fact are two positional claims about
//   the same bytes (issue #356). Where the pair is geometric and the frames
//   are comparable, a child lying outside its parent refuses at assembly
//   with both nodes named — the body is refused, neither claim preferred.
//   The measured escape: a signature the edge nests in `[0, 100 MiB)` and
//   the fact puts at 500 MiB. Honest spellings assemble: device-framed
//   inside the partition, partition-framed within its length, and the
//   exact-fit boundary. Two shapes are deliberately left alone, because
//   refusing them would decide what ADR-0037 holds: a child in a frame its
//   parent cannot be compared against, and a child whose parent declares
//   no extent (the golden vector's shape).
// Evidence: a_containment_child_outside_its_parent_refuses
#[test]
fn a_containment_child_outside_its_parent_refuses() {
    const MIB: u64 = 1 << 20;
    let sda_id = derive_id(&device(b"SDA")).expect("derivable");
    let (_, _, _, [_, _, part_id, sig_id]) = partition_carrying_signature(HostRange {
        host: sda_id,
        start: MIB,
        length: MIB,
    });
    let device_framed = |start: u64| HostRange {
        host: sda_id,
        start,
        length: MIB,
    };
    let partition_framed = |start: u64| HostRange {
        host: part_id,
        start,
        length: MIB,
    };
    let unrelated = derive_id(&device(b"OTHER")).expect("derivable");

    // The measured contradiction, and its partition-framed twin.
    for forged in [device_framed(500 * MIB), partition_framed(500 * MIB)] {
        let (nodes, edges, facts, _) = partition_carrying_signature(forged);
        assert_eq!(
            assemble_result(nodes, edges, facts).err(),
            Some(SnapshotError::Facts(
                FactError::ExtentOutsideContainmentParent {
                    child: sig_id,
                    parent: part_id,
                }
            )),
            "{forged:?}"
        );
    }
    // A child that starts inside and ends outside is outside.
    let (nodes, edges, facts, _) = partition_carrying_signature(HostRange {
        host: sda_id,
        start: 100 * MIB - 1,
        length: 2,
    });
    assert!(matches!(
        assemble_result(nodes, edges, facts),
        Err(SnapshotError::Facts(
            FactError::ExtentOutsideContainmentParent { .. }
        ))
    ));
    // Honest, in both lawful spellings; and the exact-fit boundary.
    for honest in [
        device_framed(MIB),
        partition_framed(MIB),
        device_framed(99 * MIB),
        partition_framed(99 * MIB),
    ] {
        let (nodes, edges, facts, _) = partition_carrying_signature(honest);
        assert!(assemble_result(nodes, edges, facts).is_ok(), "{honest:?}");
    }
    // Left alone: a frame the parent cannot be compared against. That the
    // frame resolves at all is required; where it lies is ADR-0037's.
    let (mut nodes, edges, mut facts, _) = partition_carrying_signature(HostRange {
        host: unrelated,
        start: 500 * MIB,
        length: MIB,
    });
    nodes.push(device(b"OTHER"));
    facts.transports.insert(unrelated, TransportClass::Sata);
    assert!(assemble_result(nodes, edges, facts).is_ok());
    // Left alone: the parent declares no extent.
    let (nodes, edges, mut facts, _) = partition_carrying_signature(device_framed(500 * MIB));
    facts.extents.remove(&part_id);
    assert!(assemble_result(nodes, edges, facts).is_ok());
}

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   A partition table's extent is the table structure's own bytes, not
//   the region it governs: every committed GPT fixture puts `p1` at
//   `table.start + table.length` exactly, so `partition-table` →
//   `partition` carries no span claim and a partition beyond the table's
//   first MiB assembles. The rule is per pair, read off the pair table's
//   source kind, and this test pins which side that pair is on — while
//   the table itself remains a geometric child of its device.
// Evidence: a_partition_beyond_its_tables_own_bytes_is_lawful
#[test]
fn a_partition_beyond_its_tables_own_bytes_is_lawful() {
    let dev_id = derive_id(&device(b"V0")).expect("derivable");
    let (nodes, edges, mut facts, _, part_id) = one_partition(Some(HostRange {
        host: dev_id,
        start: 1 << 20,
        length: 256 << 20,
    }));
    let table_id = edges[0].target;
    facts.extents.insert(
        table_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 20,
        },
    );
    let snapshot = assemble_result(nodes, edges, facts).expect("assembles");
    assert!(snapshot.facts().extents.contains_key(&part_id));

    let (nodes, edges, mut facts, _, _) = one_partition(None);
    facts.extents.insert(
        table_id,
        HostRange {
            host: dev_id,
            start: (1 << 30) - 4096,
            length: 8192,
        },
    );
    assert_eq!(
        assemble_result(nodes, edges, facts).err(),
        Some(SnapshotError::Facts(
            FactError::ExtentOutsideContainmentParent {
                child: table_id,
                parent: dev_id,
            }
        ))
    );
}

// Requirements: MODEL-005, MODEL-003, SAFE-005
//   The end-to-end shape of issue #356: an honest body encodes and
//   round-trips, and the same bytes with one extent moved 400 MiB past its
//   partition refuse at the decode boundary through the same rule, before
//   any closure runs over them. Under MODEL-003's explicit-rejection limb,
//   with the schema version unchanged: the refused body was never a lawful
//   capture, only an unvalidated one.
// Evidence: a_forged_extent_refuses_at_the_boundary_before_any_closure_runs
#[test]
fn a_forged_extent_refuses_at_the_boundary_before_any_closure_runs() {
    const MIB: u64 = 1 << 20;
    let sda_id = derive_id(&device(b"SDA")).expect("derivable");
    let (nodes, edges, facts, [_, _, part_id, sig_id]) = partition_carrying_signature(HostRange {
        host: sda_id,
        start: MIB,
        length: MIB,
    });
    let honest = assemble_result(nodes, edges, facts).expect("assembles");
    let bytes = canonical::encode(&honest.body_value().expect("body")).expect("encodable");
    TopologySnapshot::from_canonical_body(&bytes).expect("the honest body round-trips");

    let body = honest.body_value().expect("body");
    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(entries)) = map.get_mut("nodes") else {
        panic!("nodes present");
    };
    let mut moved = false;
    for entry in entries.iter_mut() {
        let canonical::Value::Map(entry) = entry else {
            continue;
        };
        let mut fields = entry.clone();
        for key in ["extent_host", "extent_start", "extent_length"] {
            fields.remove(key);
        }
        let is_signature = super::naming::fields_from_map(&fields)
            .ok()
            .and_then(|fields| derive_id(&fields).ok())
            == Some(sig_id);
        if is_signature {
            entry.insert(
                "extent_start".to_owned(),
                canonical::Value::Unsigned(500 * MIB),
            );
            moved = true;
        }
    }
    assert!(
        moved,
        "the signature's entry was found and its extent moved"
    );
    resort(entries);
    let forged = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    assert_ne!(forged, bytes);
    assert_eq!(
        TopologySnapshot::from_canonical_body(&forged),
        Err(SnapshotSchemaError::Rebuild(SnapshotError::Facts(
            FactError::ExtentOutsideContainmentParent {
                child: sig_id,
                parent: part_id,
            }
        )))
    );
}
