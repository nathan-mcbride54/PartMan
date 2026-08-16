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
//   inside the partition, and the exact-fit boundary. Since ADR-0046
//   enforced ADR-0037's rule the partition-framed twin and the child in
//   an unrelated frame refuse before this rule is reached — the frame
//   itself disagrees with the name — and one shape is still left alone: a
//   child whose parent declares no extent.
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

    // The measured contradiction.
    let (nodes, edges, facts, _) = partition_carrying_signature(device_framed(500 * MIB));
    assert_eq!(
        assemble_result(nodes, edges, facts).err(),
        Some(SnapshotError::Facts(
            FactError::ExtentOutsideContainmentParent {
                child: sig_id,
                parent: part_id,
            }
        ))
    );
    // Its partition-framed twin, at 500 MiB or honestly within the
    // partition's length: the frame is refused first, both facts named.
    for framed in [partition_framed(500 * MIB), partition_framed(MIB)] {
        let (nodes, edges, facts, _) = partition_carrying_signature(framed);
        assert_eq!(
            assemble_result(nodes, edges, facts).err(),
            Some(SnapshotError::Facts(
                FactError::ExtentFrameDisagreesWithName {
                    node: sig_id,
                    declared: part_id,
                    derived: sda_id,
                }
            )),
            "{framed:?}"
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
    // Honest, in the one lawful spelling; and the exact-fit boundary.
    for honest in [device_framed(MIB), device_framed(99 * MIB)] {
        let (nodes, edges, facts, _) = partition_carrying_signature(honest);
        assert!(assemble_result(nodes, edges, facts).is_ok(), "{honest:?}");
    }
    // A frame the parent cannot be compared against — an unrelated
    // absorbed device — was left alone by this rule and is refused by the
    // frame rule: the signature's name leads to `sda`, not there.
    let (mut nodes, edges, mut facts, _) = partition_carrying_signature(HostRange {
        host: unrelated,
        start: 500 * MIB,
        length: MIB,
    });
    nodes.push(device(b"OTHER"));
    facts.transports.insert(unrelated, TransportClass::Sata);
    assert_eq!(
        assemble_result(nodes, edges, facts).err(),
        Some(SnapshotError::Facts(
            FactError::ExtentFrameDisagreesWithName {
                node: sig_id,
                declared: unrelated,
                derived: sda_id,
            }
        ))
    );
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

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   ADR-0037's anchoring rule, enforced (ADR-0046, issue #333): a range in
//   a containment forest is expressed in that forest's root address space,
//   and the root is derived from the node's own name — never from the
//   edge set, which a body may omit. A child extent framed on its
//   immediate host rather than the root refuses at assembly with the two
//   facts named side by side, in derive-and-compare form: the declared
//   host stays a fact and is refused, never replaced. The golden vector's
//   former shape (a signature framed on its partition), `plan_tests`'
//   former shape (a file system framed on its partition), and a table
//   framed on itself all refuse; the boundary's refusal is the
//   constructor's, equal by value; and with every containment edge
//   removed the refusal stands, because the frame is read off the name.
// Evidence: an_extent_framed_below_its_containment_root_refuses_at_both_boundaries
#[test]
#[allow(clippy::too_many_lines)]
fn an_extent_framed_below_its_containment_root_refuses_at_both_boundaries() {
    const MIB: u64 = 1 << 20;
    let dev = device(b"FRAME");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: MIB,
    };
    let part_id = derive_id(&part).expect("derivable");
    let sig = NamingFields::BackingSignature {
        host: part_id,
        family: SignatureFamily::Mdraid1x,
        primary_offset: 4096,
    };
    let sig_id = derive_id(&sig).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: part_id,
        kind: super::naming::FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");
    let nodes = || {
        vec![
            dev.clone(),
            table.clone(),
            part.clone(),
            sig.clone(),
            fs.clone(),
        ]
    };
    let containment = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    let edges = || {
        vec![
            containment(dev_id, table_id),
            containment(table_id, part_id),
            containment(part_id, sig_id),
            containment(part_id, fs_id),
        ]
    };
    let framed = |host, start, length| HostRange {
        host,
        start,
        length,
    };
    // The honest body: everything in the device's frame.
    let honest = || {
        let mut facts = Facts::default();
        facts.transports.insert(dev_id, TransportClass::Sata);
        facts.extents.insert(dev_id, framed(dev_id, 0, 1 << 30));
        facts.extents.insert(table_id, framed(dev_id, 0, MIB));
        facts
            .extents
            .insert(part_id, framed(dev_id, MIB, 256 * MIB));
        facts
            .extents
            .insert(sig_id, framed(dev_id, MIB + 4096, 1 << 16));
        facts
            .extents
            .insert(fs_id, framed(dev_id, 2 * MIB, 100 * MIB));
        facts
    };
    let honest_snapshot =
        assemble_result(nodes(), edges(), honest()).expect("the root-framed body assembles");

    // Three refused spellings, each the same bytes framed one hop down.
    let refused = [
        (sig_id, framed(part_id, 4096, 1 << 16), part_id),
        (fs_id, framed(part_id, MIB, 100 * MIB), part_id),
        (table_id, framed(table_id, 0, MIB), table_id),
    ];
    for (node, extent, declared) in refused {
        let mut facts = honest();
        facts.extents.insert(node, extent);
        let expected = || {
            SnapshotError::Facts(FactError::ExtentFrameDisagreesWithName {
                node,
                declared,
                derived: dev_id,
            })
        };
        assert_eq!(
            assemble_result(nodes(), edges(), facts.clone()).err(),
            Some(expected()),
            "{node}"
        );
        // With every containment edge removed the frame is still derived
        // and still refused: it is read off the name, not the edges.
        assert_eq!(
            assemble_result(nodes(), vec![], facts).err(),
            Some(expected()),
            "{node}, no edges"
        );

        // The same forgery at the boundary: rewrite the honest body's
        // entry for the node and decode. The boundary's refusal is the
        // constructor's, equal by value.
        let body = honest_snapshot.body_value().expect("body");
        let canonical::Value::Map(mut map) = body else {
            panic!("body is a map");
        };
        let Some(canonical::Value::Array(entries)) = map.get_mut("nodes") else {
            panic!("nodes present");
        };
        let mut forged = false;
        for entry in entries.iter_mut() {
            let canonical::Value::Map(fields) = entry else {
                panic!("entry is a map");
            };
            let mut named = fields.clone();
            for key in ["extent_host", "extent_start", "extent_length"] {
                named.remove(key);
            }
            let is_node = super::naming::fields_from_map(&named)
                .ok()
                .and_then(|fields| derive_id(&fields).ok())
                == Some(node);
            if is_node {
                fields.insert(
                    "extent_host".to_owned(),
                    canonical::Value::Bytes(extent.host.as_bytes().to_vec()),
                );
                fields.insert(
                    "extent_start".to_owned(),
                    canonical::Value::Unsigned(extent.start),
                );
                forged = true;
            }
        }
        assert!(forged, "the node's entry was found and re-framed");
        resort(entries);
        let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
        assert_eq!(
            TopologySnapshot::from_canonical_body(&bytes),
            Err(SnapshotSchemaError::Rebuild(expected())),
            "{node}, at the boundary"
        );
    }
}

/// One body holding every containment forest the pair table can root —
/// a device, a produced volume, a multipath node — with an extent-bearing
/// node at every depth of each, plus a backing extent, which appears in
/// no containment pair. Every extent is framed on the root its name leads
/// to, and the geometry is honest.
/// Nodes, edges, facts, and for every extent-bearing node the root its
/// name leads to (`None` outside every forest).
type EveryForest = (
    Vec<NamingFields>,
    Vec<Edge>,
    Facts,
    Vec<(super::naming::NodeId, Option<super::naming::NodeId>)>,
);

#[allow(clippy::too_many_lines)]
fn every_forest() -> EveryForest {
    use super::naming::{ExtentLocator, FileSystemKind};
    const MIB: u64 = 1 << 20;
    let id = |fields: &NamingFields| derive_id(fields).expect("derivable");
    let containment = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    let framed = |host, start, length| HostRange {
        host,
        start,
        length,
    };
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut facts = Facts::default();
    // (node, the root its name leads to; `None` outside every forest)
    let mut roots = Vec::new();

    // The device forest.
    let dev = device(b"EVERY");
    let dev_id = id(&dev);
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = id(&table);
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: MIB,
    };
    let part_id = id(&part);
    let sig_in_part = NamingFields::BackingSignature {
        host: part_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let sig_in_part_id = id(&sig_in_part);
    let fs_in_part = NamingFields::FileSystem {
        host: part_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_in_part_id = id(&fs_in_part);
    let entry = NamingFields::ConflictingTableEntry {
        table: table_id,
        view_role: TableRole::HybridMbr,
        entry_start: 300 * MIB,
    };
    let entry_id = id(&entry);
    let sig_on_dev = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Mdraid1x,
        primary_offset: 600 * MIB,
    };
    let sig_on_dev_id = id(&sig_on_dev);
    let fs_on_dev = NamingFields::FileSystem {
        host: dev_id,
        kind: FileSystemKind::Xfs,
        superblock_offset: 700 * MIB,
    };
    let fs_on_dev_id = id(&fs_on_dev);
    facts.transports.insert(dev_id, TransportClass::Sata);
    facts.extents.insert(dev_id, framed(dev_id, 0, 1 << 30));
    facts.extents.insert(table_id, framed(dev_id, 0, MIB));
    facts
        .extents
        .insert(part_id, framed(dev_id, MIB, 255 * MIB));
    facts
        .extents
        .insert(sig_in_part_id, framed(dev_id, MIB, 4096));
    facts
        .extents
        .insert(fs_in_part_id, framed(dev_id, 2 * MIB, 100 * MIB));
    facts
        .extents
        .insert(entry_id, framed(dev_id, 300 * MIB, MIB));
    facts
        .extents
        .insert(sig_on_dev_id, framed(dev_id, 600 * MIB, 4096));
    facts
        .extents
        .insert(fs_on_dev_id, framed(dev_id, 700 * MIB, 100 * MIB));
    nodes.extend([
        dev,
        table,
        part,
        sig_in_part,
        fs_in_part,
        entry,
        sig_on_dev,
        fs_on_dev,
    ]);
    edges.extend([
        containment(dev_id, table_id),
        containment(table_id, part_id),
        containment(part_id, sig_in_part_id),
        containment(part_id, fs_in_part_id),
        containment(table_id, entry_id),
        containment(dev_id, sig_on_dev_id),
        containment(dev_id, fs_on_dev_id),
    ]);
    roots.extend([
        (dev_id, Some(dev_id)),
        (table_id, Some(dev_id)),
        (part_id, Some(dev_id)),
        (sig_in_part_id, Some(dev_id)),
        (fs_in_part_id, Some(dev_id)),
        (entry_id, Some(dev_id)),
        (sig_on_dev_id, Some(dev_id)),
        (fs_on_dev_id, Some(dev_id)),
    ]);

    // The volume forest, produced by an aggregate the device signature
    // backs.
    let agg = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"every-md".to_vec()),
    };
    let agg_id = id(&agg);
    let vol = NamingFields::Volume {
        producer: agg_id,
        name: b"md0".to_vec(),
        role: None,
    };
    let vol_id = id(&vol);
    let vtable = NamingFields::PartitionTable {
        parent: vol_id,
        role: TableRole::Gpt,
    };
    let vtable_id = id(&vtable);
    let vpart = NamingFields::Partition {
        parent_table: vtable_id,
        start_offset: MIB,
    };
    let vpart_id = id(&vpart);
    let sig_below_vol = NamingFields::BackingSignature {
        host: vpart_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let sig_below_vol_id = id(&sig_below_vol);
    let fs_on_vol = NamingFields::FileSystem {
        host: vol_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 200 * MIB,
    };
    let fs_on_vol_id = id(&fs_on_vol);
    let sig_on_vol = NamingFields::BackingSignature {
        host: vol_id,
        family: SignatureFamily::Luks2,
        primary_offset: 300 * MIB,
    };
    let sig_on_vol_id = id(&sig_on_vol);
    facts.member_counts.insert(agg_id, 1);
    facts.extents.insert(vtable_id, framed(vol_id, 0, MIB));
    facts
        .extents
        .insert(vpart_id, framed(vol_id, MIB, 100 * MIB));
    facts
        .extents
        .insert(sig_below_vol_id, framed(vol_id, MIB, 4096));
    facts
        .extents
        .insert(fs_on_vol_id, framed(vol_id, 200 * MIB, 50 * MIB));
    facts
        .extents
        .insert(sig_on_vol_id, framed(vol_id, 300 * MIB, 4096));
    nodes.extend([
        agg,
        vol,
        vtable,
        vpart,
        sig_below_vol,
        fs_on_vol,
        sig_on_vol,
    ]);
    edges.extend([
        Edge {
            kind: EdgeKind::Backing,
            source: sig_on_dev_id,
            target: agg_id,
        },
        Edge {
            kind: EdgeKind::Production,
            source: agg_id,
            target: vol_id,
        },
        containment(vol_id, vtable_id),
        containment(vtable_id, vpart_id),
        containment(vpart_id, sig_below_vol_id),
        containment(vol_id, fs_on_vol_id),
        containment(vol_id, sig_on_vol_id),
    ]);
    roots.extend([
        (vtable_id, Some(vol_id)),
        (vpart_id, Some(vol_id)),
        (sig_below_vol_id, Some(vol_id)),
        (fs_on_vol_id, Some(vol_id)),
        (sig_on_vol_id, Some(vol_id)),
    ]);

    // The multipath forest.
    let mp = NamingFields::MultipathNode {
        lun_designator: b"every-lun".to_vec(),
    };
    let mp_id = id(&mp);
    let mtable = NamingFields::PartitionTable {
        parent: mp_id,
        role: TableRole::Mbr,
    };
    let mtable_id = id(&mtable);
    let mpart = NamingFields::Partition {
        parent_table: mtable_id,
        start_offset: MIB,
    };
    let mpart_id = id(&mpart);
    let fs_on_mp = NamingFields::FileSystem {
        host: mp_id,
        kind: FileSystemKind::Xfs,
        superblock_offset: 100 * MIB,
    };
    let fs_on_mp_id = id(&fs_on_mp);
    let sig_on_mp = NamingFields::BackingSignature {
        host: mp_id,
        family: SignatureFamily::Zfs,
        primary_offset: 200 * MIB,
    };
    let sig_on_mp_id = id(&sig_on_mp);
    facts.extents.insert(mtable_id, framed(mp_id, 0, MIB));
    facts.extents.insert(mpart_id, framed(mp_id, MIB, 50 * MIB));
    facts
        .extents
        .insert(fs_on_mp_id, framed(mp_id, 100 * MIB, 10 * MIB));
    facts
        .extents
        .insert(sig_on_mp_id, framed(mp_id, 200 * MIB, 4096));
    nodes.extend([mp, mtable, mpart, fs_on_mp, sig_on_mp]);
    edges.extend([
        containment(mp_id, mtable_id),
        containment(mtable_id, mpart_id),
        containment(mp_id, fs_on_mp_id),
        containment(mp_id, sig_on_mp_id),
    ]);
    roots.extend([
        (mtable_id, Some(mp_id)),
        (mpart_id, Some(mp_id)),
        (fs_on_mp_id, Some(mp_id)),
        (sig_on_mp_id, Some(mp_id)),
    ]);

    // Outside every forest: a backing extent, a byte range within the
    // device-hosted file system's own address space.
    let backing = NamingFields::BackingExtent {
        host: fs_on_dev_id,
        locator: ExtentLocator::Range {
            start: MIB,
            length: 8 * MIB,
        },
    };
    let backing_id = id(&backing);
    facts
        .extents
        .insert(backing_id, framed(fs_on_dev_id, MIB, 8 * MIB));
    nodes.push(backing);
    roots.push((backing_id, None));

    (nodes, edges, facts, roots)
}

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   The frame rule reaches every containment forest the pair table can
//   root and every depth of each (ADR-0046): a device's table, partition,
//   the signature and file system inside the partition, the conflicting
//   entry, and the signature and file system on the device itself; the
//   same below a produced volume and below a multipath node. Enumerated
//   rather than sampled: for every extent-bearing node in one honest body
//   and every absorbed node as a candidate frame, the body assembles
//   exactly when the frame is the root the node's own name leads to and
//   refuses, naming both, otherwise — the volume and multipath forests
//   included, whose roots carry no extent of their own. The backing
//   extent is the one node the rule does not reach: it appears in no
//   containment pair, its `host` is the one open naming field, and its
//   range lives in its host's own address space, so it assembles framed
//   on any absorbed node — a limit recorded, not a rule (issue #365).
// Evidence: the_frame_rule_reaches_every_forest_at_every_depth
#[test]
fn the_frame_rule_reaches_every_forest_at_every_depth() {
    let (nodes, edges, facts, roots) = every_forest();
    assert!(
        assemble_result(nodes.clone(), edges.clone(), facts.clone()).is_ok(),
        "the honest body assembles"
    );
    let candidates: Vec<super::naming::NodeId> = nodes
        .iter()
        .map(|fields| derive_id(fields).expect("derivable"))
        .collect();
    assert_eq!(
        candidates.len(),
        21,
        "the population is what this test says it is"
    );
    let mut refused = 0;
    let mut admitted = 0;
    for (node, root) in &roots {
        let honest = facts.extents[node];
        for candidate in &candidates {
            let mut mutated = facts.clone();
            mutated.extents.insert(
                *node,
                HostRange {
                    host: *candidate,
                    ..honest
                },
            );
            let result = assemble_result(nodes.clone(), edges.clone(), mutated);
            match root {
                Some(root) if candidate != root => {
                    assert_eq!(
                        result.err(),
                        Some(SnapshotError::Facts(
                            FactError::ExtentFrameDisagreesWithName {
                                node: *node,
                                declared: *candidate,
                                derived: *root,
                            }
                        )),
                        "{node} framed on {candidate}"
                    );
                    refused += 1;
                }
                _ => {
                    assert!(result.is_ok(), "{node} framed on {candidate}: {result:?}");
                    admitted += 1;
                }
            }
        }
    }
    // Seventeen forest nodes × twenty-one candidates, one lawful frame
    // each; the backing extent admits all twenty-one.
    assert_eq!((refused, admitted), (17 * 20, 17 + 21));
}

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   The third witness (ADR-0046, on the strength ADR-0045 held beside
//   issue #333): a containment edge and the target's own name are two
//   claims about which node the target's bytes lie inside, and a body
//   whose edge nests a node in one parent while its name positions it in
//   another is refused with both parents named — a signature edge-nested
//   under a sibling partition, a partition under another table, a table
//   under another device, at every forest. Enumerated: every containment
//   edge in the honest body, re-sourced onto every other absorbed node,
//   refuses exactly this way when the target names its parent, and only
//   the pair table's own refusal otherwise; and the boundary's refusal is
//   the constructor's.
// Evidence: a_containment_edge_that_disagrees_with_the_name_refuses
#[test]
#[allow(clippy::too_many_lines)]
fn a_containment_edge_that_disagrees_with_the_name_refuses() {
    let (nodes, edges, facts, _) = every_forest();
    let honest = assemble_result(nodes.clone(), edges.clone(), facts.clone())
        .expect("the honest body assembles");
    let candidates: Vec<super::naming::NodeId> = nodes
        .iter()
        .map(|fields| derive_id(fields).expect("derivable"))
        .collect();
    let kind_of = |id: super::naming::NodeId| {
        nodes
            .iter()
            .find(|fields| derive_id(fields).expect("derivable") == id)
            .map(NamingFields::kind_name)
            .expect("absorbed")
    };
    let mut refused = 0;
    let mut pair_refused = 0;
    for (index, edge) in edges.iter().enumerate() {
        if edge.kind != EdgeKind::Containment {
            continue;
        }
        for candidate in &candidates {
            if *candidate == edge.source || *candidate == edge.target {
                continue;
            }
            let mut moved = edges.clone();
            moved[index] = Edge {
                source: *candidate,
                ..*edge
            };
            let result = assemble_result(nodes.clone(), moved, facts.clone());
            if super::topology::endpoint_pair_allowed(
                EdgeKind::Containment,
                kind_of(*candidate),
                kind_of(edge.target),
            ) {
                assert_eq!(
                    result.err(),
                    Some(SnapshotError::Facts(
                        FactError::ContainmentEdgeDisagreesWithName {
                            child: edge.target,
                            edge_parent: *candidate,
                            named_parent: edge.source,
                        }
                    )),
                    "edge {index} re-sourced onto {candidate}"
                );
                refused += 1;
            } else {
                assert!(
                    matches!(
                        result,
                        Err(SnapshotError::Topology(
                            TopologyError::ForbiddenEndpoint { .. }
                        ))
                    ),
                    "edge {index} re-sourced onto {candidate}: {result:?}"
                );
                pair_refused += 1;
            }
        }
    }
    // Sixteen containment edges × nineteen other candidates: 59 land on a
    // kind the pair table admits as this target's parent and are refused
    // by the name; 245 are refused by the table itself first.
    assert_eq!(
        (refused, pair_refused),
        (59, 245),
        "the enumeration is what this test says it is"
    );

    // At the boundary: the honest body with one edge re-sourced onto the
    // sibling partition decodes to the constructor's refusal.
    let (dev_id, part_id, sig_id) = {
        let dev = derive_id(&nodes[0]).expect("derivable");
        let part = derive_id(&nodes[2]).expect("derivable");
        let sig = derive_id(&nodes[3]).expect("derivable");
        (dev, part, sig)
    };
    let body = honest.body_value().expect("body");
    let canonical::Value::Map(mut map) = body else {
        panic!("body is a map");
    };
    let Some(canonical::Value::Array(entries)) = map.get_mut("edges") else {
        panic!("edges present");
    };
    let mut moved = false;
    for entry in entries.iter_mut() {
        let canonical::Value::Map(fields) = entry else {
            panic!("edge is a map");
        };
        let is_sig_edge = fields.get("target")
            == Some(&canonical::Value::Bytes(sig_id.as_bytes().to_vec()))
            && fields.get("source") == Some(&canonical::Value::Bytes(part_id.as_bytes().to_vec()));
        if is_sig_edge {
            fields.insert(
                "source".to_owned(),
                canonical::Value::Bytes(dev_id.as_bytes().to_vec()),
            );
            moved = true;
        }
    }
    assert!(moved, "the signature's edge was found and re-sourced");
    resort(entries);
    let bytes = canonical::encode(&canonical::Value::Map(map)).expect("encodable");
    assert_eq!(
        TopologySnapshot::from_canonical_body(&bytes),
        Err(SnapshotSchemaError::Rebuild(SnapshotError::Facts(
            FactError::ContainmentEdgeDisagreesWithName {
                child: sig_id,
                edge_parent: dev_id,
                named_parent: part_id,
            }
        )))
    );
}
