//! Tests for the mutating step's sole constructor (WP-010 increment 3h,
//! ADR-0012, ADR-0018's acknowledgment vocabulary).

use super::naming::{AggregateTechnology, NamingFields, SignatureFamily, derive_id};
use super::protection::{Facts, HostRange, StepRanges, TransportClass, Verdict};
use super::snapshot::{SnapshotKind, TopologySnapshot};
use super::step::{Acknowledgment, PlanStep, Severity, StepFlags, StepRefusal, StepRisk};

fn destructive() -> StepRisk {
    StepRisk {
        severity: Severity::Destructive,
        flags: StepFlags::default(),
    }
}
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

// Requirements: SAFE-005, MODEL-002, MODEL-003
//   ADR-0039's headline, at the boundary that has no capability gate in
//   front of it. `parse_step` re-validates a recorded plan body through
//   this constructor, so what it accepts is what a signed body means.
//   Before ADR-0039 it accepted a declared partial shrink truncating
//   128 MiB off a live ZFS vdev: the freed tail misses the label's own
//   bytes, and the old closure could not reach past them. The
//   acknowledgment parameter cannot express permission for it either —
//   the pool is Refused, not Indeterminate.
// Evidence: a_declared_partial_shrink_over_a_live_vdev_is_unconstructible
#[test]
#[allow(clippy::too_many_lines)]
fn a_declared_partial_shrink_over_a_live_vdev_is_unconstructible() {
    let dev = device(b"D9");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: super::naming::TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let member = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 512 << 20,
    };
    let member_id = derive_id(&member).expect("derivable");
    let label = NamingFields::BackingSignature {
        host: member_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let label_id = derive_id(&label).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let mut facts = device_facts(dev_id);
    facts.extents.insert(
        table_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 20,
        },
    );
    facts.extents.insert(
        member_id,
        HostRange {
            host: dev_id,
            start: 512 << 20,
            length: 256 << 20,
        },
    );
    // The label sits at the member's head, outside the freed tail below.
    facts.extents.insert(
        label_id,
        HostRange {
            host: dev_id,
            start: 512 << 20,
            length: 1 << 20,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, table.clone(), member.clone(), label, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: member_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: member_id,
                target: label_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: label_id,
                target: pool_id,
            },
        ],
        facts,
    )
    .expect("assembles");

    // The solver's real freed tail for a 256 -> 128 MiB shrink.
    let freed_tail = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: dev_id,
            start: 640 << 20,
            length: 128 << 20,
        }],
    };
    let refusal = PlanStep::mutating(
        &snapshot,
        member_id,
        freed_tail.clone(),
        vec![],
        destructive(),
    )
    .expect_err("a declared partial shrink over a live vdev must not construct");
    assert!(matches!(
        refusal,
        StepRefusal::Reached {
            node,
            verdict: Verdict::Refused { .. }
        } if node == pool_id
    ));

    // The control: the same body over the same geometry with the pool
    // absent still constructs, so the refusal is the reach and not the
    // shape of the ranges.
    let mut unprotected_facts = device_facts(dev_id);
    unprotected_facts.extents.insert(
        member_id,
        HostRange {
            host: dev_id,
            start: 512 << 20,
            length: 256 << 20,
        },
    );
    let unprotected = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![device(b"D9"), table, member],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: member_id,
            },
        ],
        unprotected_facts,
    )
    .expect("assembles");
    PlanStep::mutating(&unprotected, member_id, freed_tail, vec![], destructive())
        .expect("the same shrink over an unprotected member still constructs");
}

// Requirements: SAFE-005, MODEL-002
//   ADR-0012's axis at the constructor: a step whose closure reaches a
//   consumed member's pool returns a typed refusal, not a value — and no
//   acknowledgment parameter can express permission for it.
// Evidence: a_refused_reach_is_unconstructible_even_acknowledged
#[test]
fn a_refused_reach_is_unconstructible_even_acknowledged() {
    let (snapshot, dev_id, signature_id) = signature_snapshot(true);
    let bare = PlanStep::mutating(&snapshot, dev_id, wipe(dev_id), vec![], destructive());
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
        destructive(),
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
    let bare = PlanStep::mutating(&snapshot, dev_id, wipe(dev_id), vec![], destructive());
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
        destructive(),
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
        destructive(),
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
        destructive(),
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
    let step = PlanStep::mutating(&snapshot, dev_id, wipe(dev_id), vec![], destructive())
        .expect("constructs");
    assert_eq!(step.target(), dev_id);
    assert!(step.affected().contains(&dev_id));
}
