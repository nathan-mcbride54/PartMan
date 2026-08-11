//! Tests for the planner chassis (WP-060 increment 1).

use partman_capability::engine::{RuntimeFacts, TechnologyLimits};
use partman_capability::{Reason, Status};
use partman_domain::canonical::{self, Value};
use partman_domain::model::capability::Operation;
use partman_domain::model::identity::TableState;
use partman_domain::model::naming::{
    AggregateTechnology, NamingFields, NodeId, SignatureFamily, derive_id,
};
use partman_domain::model::plan::{OperationPlan, ValidityWindow};
use partman_domain::model::protection::{Facts, HostRange, TransportClass};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::step::Severity;
use partman_domain::model::topology::{Edge, EdgeKind};

use super::{PlanIdentity, PlanRefusal, PlanRequest, plan};

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

fn device_facts(facts: &mut Facts, id: NodeId) {
    facts.transports.insert(id, TransportClass::Sata);
    facts.extents.insert(
        id,
        HostRange {
            host: id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.table_states.insert(
        id,
        TableState::Present {
            checksum: canonical::hash(&Value::Text("planner fixture checksum".into()))
                .expect("hashable"),
        },
    );
}

/// A clean plannable device beside a refused ZFS chain.
fn fixture() -> (TopologySnapshot, NodeId, NodeId) {
    let clean = device(b"PLN-CLEAN");
    let clean_id = derive_id(&clean).expect("derivable");
    let zfs_host = device(b"PLN-ZFS");
    let zfs_host_id = derive_id(&zfs_host).expect("derivable");
    let zfs_sig = NamingFields::BackingSignature {
        host: zfs_host_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let zfs_sig_id = derive_id(&zfs_sig).expect("derivable");
    let zfs_pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"PLN-POOL".to_vec()),
    };
    let zfs_pool_id = derive_id(&zfs_pool).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, clean_id);
    device_facts(&mut facts, zfs_host_id);
    facts.extents.insert(
        zfs_sig_id,
        HostRange {
            host: zfs_host_id,
            start: 0,
            length: 1 << 16,
        },
    );

    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![clean, zfs_host, zfs_sig, zfs_pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: zfs_host_id,
                target: zfs_sig_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: zfs_sig_id,
                target: zfs_pool_id,
            },
        ],
        facts,
    )
    .expect("assembles");
    (snapshot, clean_id, zfs_sig_id)
}

fn identity() -> PlanIdentity {
    PlanIdentity {
        plan_id: b"pln-1".to_vec(),
        created_at: 1_700_000_000,
        validity: ValidityWindow {
            not_after: 1_700_086_400,
        },
    }
}

fn body_bytes(plan: &OperationPlan) -> Vec<u8> {
    canonical::encode(&plan.body_value().expect("body")).expect("encodable")
}

// Requirements: PLAN-001, PLAN-006
//   Determinism, held as bytes: equal snapshot, answers, and request
//   produce byte-equal plan bodies, and the produced plan revalidates
//   through the typed boundary against the same snapshot.
// Evidence: equal_inputs_produce_byte_equal_plans
#[test]
fn equal_inputs_produce_byte_equal_plans() {
    let (snapshot, clean, _) = fixture();
    let request = PlanRequest {
        operation: Operation::Wipe,
        target: clean,
    };
    let first = plan(
        request,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    let second = plan(
        request,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert_eq!(
        body_bytes(&first),
        body_bytes(&second),
        "PLAN-001: equal inputs, byte-equal bodies"
    );
    let rebuilt = OperationPlan::from_canonical_body(&body_bytes(&first), &snapshot)
        .expect("revalidates against the same snapshot");
    assert_eq!(
        rebuilt.body_hash().expect("hash"),
        first.body_hash().expect("hash")
    );
    assert_eq!(first.severity(), Severity::Destructive);
}

// Requirements: PLAN-001, CAP-005
//   The conditioning rule: a refused capability answer refuses the
//   request with the engine's answer carried verbatim — reason and
//   remediation travel, nothing is re-derived or paraphrased.
// Evidence: a_refused_capability_refuses_with_the_answer_verbatim
#[test]
fn a_refused_capability_refuses_with_the_answer_verbatim() {
    let (snapshot, _, zfs_sig) = fixture();
    let refused = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: zfs_sig,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("the refused chain must not plan");
    let PlanRefusal::CapabilityRefused { answer } = refused else {
        panic!("the refusal carries the engine's answer: {refused:?}");
    };
    assert_eq!(answer.status(), Status::Unsupported);
    assert!(matches!(answer.reason(), Reason::ProtectionRefused { .. }));
}

// Requirements: PLAN-001
//   A source-class operation is not plan material: plans mutate, and
//   the request refuses with the typed variant naming the operation.
// Evidence: a_source_operation_is_not_plan_material
#[test]
fn a_source_operation_is_not_plan_material() {
    let (snapshot, clean, _) = fixture();
    let refused = plan(
        PlanRequest {
            operation: Operation::Detect,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("a detect request must not plan");
    assert_eq!(
        refused,
        PlanRefusal::NotAPlanningOperation {
            operation: Operation::Detect
        }
    );
}

// Requirements: PLAN-001
//   An unknown target is the typed caller error, carried through from
//   the engine's own distinction between an answer and an error.
// Evidence: an_unknown_target_refuses_typed
#[test]
fn an_unknown_target_refuses_typed() {
    let (snapshot, _, _) = fixture();
    let stranger = derive_id(&device(b"PLN-STRANGER")).expect("derivable");
    let refused = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: stranger,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("must refuse");
    assert_eq!(refused, PlanRefusal::UnknownTarget { target: stranger });
}

use super::graph::{Dependency, GraphRefusal};
use super::{PlanRequestSet, plan_set};

fn wipe_and_create(target: NodeId) -> PlanRequestSet {
    PlanRequestSet {
        requests: vec![
            PlanRequest {
                operation: Operation::Wipe,
                target,
            },
            PlanRequest {
                operation: Operation::Create,
                target,
            },
        ],
        dependencies: vec![Dependency {
            before: 0,
            after: 1,
        }],
    }
}

// Requirements: PLAN-003, PLAN-001
//   The ordered-overlap chain constructs: a wipe followed by a create in
//   the freed space is legitimate exactly because the dependency orders
//   it, the emitted steps carry that order, plan severity is the step
//   maximum, and the whole set is deterministic to the byte.
// Evidence: an_ordered_chain_constructs_deterministically
#[test]
fn an_ordered_chain_constructs_deterministically() {
    let (snapshot, clean, _) = fixture();
    let set = wipe_and_create(clean);
    let first = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("the ordered chain plans");
    let second = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans again");
    assert_eq!(body_bytes(&first), body_bytes(&second));
    assert_eq!(first.steps().len(), 2);
    assert_eq!(first.severity(), Severity::Destructive);
}

// Requirements: PLAN-003
//   The same two steps with the dependency removed refuse as an
//   unordered overlap naming both steps and the host: no order makes
//   concurrent effects on the same bytes deterministic, and the absent
//   dependency is exactly what would have explained them.
// Evidence: an_unordered_overlap_refuses_with_both_steps_named
#[test]
fn an_unordered_overlap_refuses_with_both_steps_named() {
    let (snapshot, clean, _) = fixture();
    let mut set = wipe_and_create(clean);
    set.dependencies.clear();
    let refused = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("unordered overlap must refuse");
    assert_eq!(
        refused,
        PlanRefusal::GraphRefused {
            refusal: GraphRefusal::UnorderedOverlap {
                first: 0,
                second: 1,
                host: clean,
            }
        }
    );
}

// Requirements: PLAN-003
//   A dependency cycle refuses with its unorderable members named, and
//   a duplicate request refuses before ranges are even compared.
// Evidence: cycles_and_duplicates_refuse_with_explanations
#[test]
fn cycles_and_duplicates_refuse_with_explanations() {
    let (snapshot, clean, _) = fixture();
    let mut cyclic = wipe_and_create(clean);
    cyclic.dependencies = vec![
        Dependency {
            before: 0,
            after: 1,
        },
        Dependency {
            before: 1,
            after: 0,
        },
    ];
    let refused = plan_set(
        &cyclic,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("a cycle must refuse");
    assert_eq!(
        refused,
        PlanRefusal::GraphRefused {
            refusal: GraphRefusal::Cycle {
                members: vec![0, 1]
            }
        }
    );

    let duplicated = PlanRequestSet {
        requests: vec![
            PlanRequest {
                operation: Operation::Wipe,
                target: clean,
            },
            PlanRequest {
                operation: Operation::Wipe,
                target: clean,
            },
        ],
        dependencies: vec![],
    };
    let refused = plan_set(
        &duplicated,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("a duplicate must refuse");
    assert_eq!(
        refused,
        PlanRefusal::GraphRefused {
            refusal: GraphRefusal::DuplicateRequest {
                first: 0,
                second: 1
            }
        }
    );
}

// Requirements: PLAN-003
//   Malformed dependency edges refuse before anything else is judged:
//   an out-of-range index and a self-dependency each name themselves.
// Evidence: malformed_edges_refuse_by_name
#[test]
fn malformed_edges_refuse_by_name() {
    let (snapshot, clean, _) = fixture();
    let mut set = wipe_and_create(clean);
    set.dependencies = vec![Dependency {
        before: 0,
        after: 9,
    }];
    let refused = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("out of range must refuse");
    assert!(matches!(
        refused,
        PlanRefusal::GraphRefused {
            refusal: GraphRefusal::DependencyOutOfRange { .. }
        }
    ));

    set.dependencies = vec![Dependency {
        before: 1,
        after: 1,
    }];
    let refused = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("self-dependency must refuse");
    assert_eq!(
        refused,
        PlanRefusal::GraphRefused {
            refusal: GraphRefusal::SelfDependency { index: 1 }
        }
    );
}
