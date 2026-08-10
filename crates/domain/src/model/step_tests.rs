//! Tests for the mutating step's sole constructor (WP-010 increment 3h,
//! ADR-0012, ADR-0018's acknowledgment vocabulary).

use super::naming::{AggregateTechnology, NamingFields, SignatureFamily, derive_id};
use super::protection::{Facts, HostRange, StepRanges, TransportClass, Verdict};
use super::snapshot::{SnapshotKind, TopologySnapshot};
use super::step::{Acknowledgment, PlanStep, StepRefusal};
use super::topology::{Edge, EdgeKind};

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

fn device_facts(id: super::naming::NodeId) -> Facts {
    let mut facts = Facts::default();
    facts.transports.insert(id, TransportClass::Sata);
    facts.extents.insert(
        id,
        HostRange {
            host: id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts
}

fn wipe(host: super::naming::NodeId) -> StepRanges {
    StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host,
            start: 0,
            length: 1 << 30,
        }],
    }
}

/// A device carrying one signature; consumed decides whether a backing
/// edge to a pool exists.
fn signature_snapshot(
    consumed: bool,
) -> (
    TopologySnapshot,
    super::naming::NodeId,
    super::naming::NodeId,
) {
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Zfs,
        primary_offset: 512 << 20,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let mut nodes = vec![dev, signature];
    let mut edges = vec![Edge {
        kind: EdgeKind::Containment,
        source: dev_id,
        target: signature_id,
    }];
    if consumed {
        let pool = NamingFields::Aggregate {
            technology: AggregateTechnology::Zfs,
            designator: Some(b"tank".to_vec()),
        };
        let pool_id = derive_id(&pool).expect("derivable");
        nodes.push(pool);
        edges.push(Edge {
            kind: EdgeKind::Backing,
            source: signature_id,
            target: pool_id,
        });
    }
    let mut facts = device_facts(dev_id);
    facts.extents.insert(
        signature_id,
        HostRange {
            host: dev_id,
            start: 512 << 20,
            length: 1 << 20,
        },
    );
    let snapshot = TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges, facts)
        .expect("assembles");
    (snapshot, dev_id, signature_id)
}

// Requirements: SAFE-005, MODEL-002
//   ADR-0012's axis at the constructor: a step whose closure reaches a
//   consumed member's pool returns a typed refusal, not a value — and no
//   acknowledgment parameter can express permission for it.
// Evidence: a_refused_reach_is_unconstructible_even_acknowledged
#[test]
fn a_refused_reach_is_unconstructible_even_acknowledged() {
    let (snapshot, dev_id, signature_id) = signature_snapshot(true);
    let bare = PlanStep::mutating(&snapshot, dev_id, wipe(dev_id), vec![]);
    assert!(matches!(bare, Err(StepRefusal::Reached { .. })));

    // A release acknowledgment naming the consumed signature is itself
    // unlawful: its verdict is Refused, and Release covers only the
    // orphan arm.
    let acknowledged = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![Acknowledgment::Release {
            signature: signature_id,
        }],
    );
    assert!(matches!(
        acknowledged,
        Err(StepRefusal::UnlawfulAcknowledgment { .. })
    ));
}

// Requirements: SAFE-005
//   The orphan arm and its lawful escape: unacknowledged, the wipe
//   refuses on the indeterminate signature; with the release
//   acknowledgment naming exactly that node, it constructs and carries
//   the acknowledgment.
// Evidence: the_release_acknowledgment_converts_exactly_the_orphan
#[test]
fn the_release_acknowledgment_converts_exactly_the_orphan() {
    let (snapshot, dev_id, signature_id) = signature_snapshot(false);
    let bare = PlanStep::mutating(&snapshot, dev_id, wipe(dev_id), vec![]);
    match bare {
        Err(StepRefusal::Reached { node, verdict }) => {
            assert_eq!(node, signature_id);
            assert!(matches!(verdict, Verdict::Indeterminate { .. }));
        }
        other => panic!("unacknowledged orphan must refuse: {other:?}"),
    }

    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![Acknowledgment::Release {
            signature: signature_id,
        }],
    )
    .expect("acknowledged release constructs");
    assert!(step.affected().contains(&signature_id));
    assert_eq!(step.acknowledgments().len(), 1);
}

// Requirements: SAFE-005
//   An acknowledgment naming the wrong node covers nothing: the step
//   still refuses on the orphan it did not name.
// Evidence: an_acknowledgment_for_another_node_covers_nothing
#[test]
fn an_acknowledgment_for_another_node_covers_nothing() {
    let (snapshot, dev_id, _signature_id) = signature_snapshot(false);
    let stranger = derive_id(&device(b"STRANGER")).expect("derivable");
    let result = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![Acknowledgment::Release {
            signature: stranger,
        }],
    );
    assert!(
        matches!(result, Err(StepRefusal::UnlawfulAcknowledgment { .. })),
        "an acknowledgment for an absent node is unlawful: {result:?}"
    );
}

// Requirements: SAFE-005
//   The vocabulary is closed and the unmodelled kinds refuse at
//   construction rather than silently passing: an opaque-destruction
//   acknowledgment has no lawful object in this slice.
// Evidence: unmodelled_acknowledgment_kinds_refuse
#[test]
fn unmodelled_acknowledgment_kinds_refuse() {
    let (snapshot, dev_id, signature_id) = signature_snapshot(false);
    let result = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![Acknowledgment::OpaqueDestruction {
            layer: signature_id,
        }],
    );
    assert!(matches!(
        result,
        Err(StepRefusal::UnlawfulAcknowledgment { .. })
    ));
}

// Requirements: MODEL-002, SAFE-005
//   A clean permitted step constructs and reports its affected set.
// Evidence: a_permitted_step_constructs_with_its_affected_set
#[test]
fn a_permitted_step_constructs_with_its_affected_set() {
    let dev = device(b"CLEAN");
    let dev_id = derive_id(&dev).expect("derivable");
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev],
        vec![],
        device_facts(dev_id),
    )
    .expect("assembles");
    let step = PlanStep::mutating(&snapshot, dev_id, wipe(dev_id), vec![]).expect("constructs");
    assert_eq!(step.target(), dev_id);
    assert!(step.affected().contains(&dev_id));
}
