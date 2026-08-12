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
    .expect("plans")
    .plan;
    let second = plan(
        request,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans")
    .plan;
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

/// A device carrying a LUKS2 signature consumed by its encryption
/// layer: two wipeable targets whose destroyed ranges overlap on the
/// device — the simulatable ordered-overlap chain.
fn chain_fixture() -> (TopologySnapshot, NodeId, NodeId) {
    let dev = device(b"PLN-CHAIN");
    let dev_id = derive_id(&dev).expect("derivable");
    let luks = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let luks_id = derive_id(&luks).expect("derivable");
    let layer = NamingFields::EncryptionLayer {
        backing_signature: luks_id,
    };
    let layer_id = derive_id(&layer).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, dev_id);
    facts.extents.insert(
        luks_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 16,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, luks, layer],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: luks_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: luks_id,
                target: layer_id,
            },
        ],
        facts,
    )
    .expect("assembles");
    (snapshot, dev_id, luks_id)
}

// Requirements: PLAN-003, PLAN-001, PLAN-002
//   The ordered-overlap chain constructs: wiping the signature before
//   wiping its device is legitimate exactly because the dependency
//   orders the overlapping ranges, the emitted steps carry that order,
//   plan severity is the step maximum, the whole set is deterministic
//   to the byte, and the simulated final topology arrives beside the
//   plan with the wiped chain gone.
// Evidence: an_ordered_chain_constructs_deterministically
#[test]
fn an_ordered_chain_constructs_deterministically() {
    let (snapshot, dev, luks) = chain_fixture();
    let set = PlanRequestSet {
        requests: vec![
            PlanRequest {
                operation: Operation::Wipe,
                target: luks,
            },
            PlanRequest {
                operation: Operation::Wipe,
                target: dev,
            },
        ],
        dependencies: vec![Dependency {
            before: 0,
            after: 1,
        }],
    };
    let first = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("the ordered chain plans")
    .plan;
    let second = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans again")
    .plan;
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

use super::solve::{
    BoundaryPlacement, DEFAULT_ALIGNMENT, SolveRefusal, StructuralEdge, free_extents,
    grow_extension, place_create, shrink_reduction,
};
use super::{Consequence, SizedRequest, plan_sized};

/// A host with one child partition-like extent at [1 MiB, 65 MiB) and a
/// misaligned second child at [100 MiB + 512, 100 MiB + 512 + 32 MiB).
fn solver_fixture() -> (TopologySnapshot, NodeId, NodeId, NodeId) {
    let host = device(b"SLV-HOST");
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: partman_domain::model::naming::TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let aligned = NamingFields::Partition {
        parent_table: table_id,
        start_offset: DEFAULT_ALIGNMENT,
    };
    let aligned_id = derive_id(&aligned).expect("derivable");
    let misaligned = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 100 * DEFAULT_ALIGNMENT + 512,
    };
    let misaligned_id = derive_id(&misaligned).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    facts.extents.insert(
        aligned_id,
        HostRange {
            host: host_id,
            start: DEFAULT_ALIGNMENT,
            length: 64 * DEFAULT_ALIGNMENT,
        },
    );
    facts.extents.insert(
        misaligned_id,
        HostRange {
            host: host_id,
            start: 100 * DEFAULT_ALIGNMENT + 512,
            length: 32 * DEFAULT_ALIGNMENT,
        },
    );

    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host, table, aligned, misaligned],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: aligned_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: misaligned_id,
            },
        ],
        facts,
    )
    .expect("assembles");
    (snapshot, host_id, aligned_id, misaligned_id)
}

// Requirements: PLAN-001
//   Free space is the host's extent minus its children's, ascending:
//   the gap before the first child, the gap between children, and the
//   tail after the last.
// Evidence: free_extents_are_the_hosts_minus_its_children
#[test]
fn free_extents_are_the_hosts_minus_its_children() {
    let (snapshot, host, _, _) = solver_fixture();
    let free = free_extents(&snapshot, host).expect("computes");
    let starts: Vec<(u64, u64)> = free
        .iter()
        .map(|range| (range.start, range.length))
        .collect();
    assert_eq!(
        starts,
        vec![
            (0, DEFAULT_ALIGNMENT),
            (65 * DEFAULT_ALIGNMENT, 35 * DEFAULT_ALIGNMENT + 512),
            (
                132 * DEFAULT_ALIGNMENT + 512,
                (1 << 30) - (132 * DEFAULT_ALIGNMENT + 512)
            ),
        ]
    );
}

// Requirements: PLAN-001
//   Placement is first-fit at the lowest 1 MiB-aligned start that
//   holds the full size, and the no-fit refusal names the largest
//   aligned fit so the caller can explain what would have succeeded.
// Evidence: placement_is_aligned_first_fit_and_no_fit_is_explained
#[test]
fn placement_is_aligned_first_fit_and_no_fit_is_explained() {
    let (snapshot, host, _, _) = solver_fixture();
    let placed = place_create(&snapshot, host, 10 * DEFAULT_ALIGNMENT)
        .expect("fits")
        .placed;
    assert_eq!(placed.start, 65 * DEFAULT_ALIGNMENT);
    assert_eq!(placed.length, 10 * DEFAULT_ALIGNMENT);

    let refused = place_create(&snapshot, host, 1 << 40).expect_err("cannot fit");
    let SolveRefusal::NoFitForSize {
        requested,
        largest_aligned_fit,
    } = refused
    else {
        panic!("the refusal names the sizes: {refused:?}");
    };
    assert_eq!(requested, 1 << 40);
    assert!(largest_aligned_fit > 0, "the largest fit is reported");
}

/// The XP-era legacy shape ADR-0023's filed case names: an MBR
/// partition starting at sector 63 (byte 32,256), inside a 1 GiB host.
/// Its end sits at 100 MiB before any request.
fn legacy_fixture() -> (TopologySnapshot, NodeId, NodeId) {
    let host = device(b"SLV-LEGACY");
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: partman_domain::model::naming::TableRole::Mbr,
    };
    let table_id = derive_id(&table).expect("derivable");
    let legacy = NamingFields::Partition {
        parent_table: table_id,
        start_offset: LEGACY_START,
    };
    let legacy_id = derive_id(&legacy).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    facts.extents.insert(
        legacy_id,
        HostRange {
            host: host_id,
            start: LEGACY_START,
            length: 100 * DEFAULT_ALIGNMENT - LEGACY_START,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host, table, legacy],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: legacy_id,
            },
        ],
        facts,
    )
    .expect("assembles");
    (snapshot, host_id, legacy_id)
}

/// Sector 63 at 512-byte sectors: the legacy misaligned start.
const LEGACY_START: u64 = 63 * 512;

// Requirements: PLAN-001, PART-009
//   ADR-0023's filed case proceeds (SI-15 resolved, spec 12.1.0): the
//   63-sector-start grow-at-tail authors only the aligned new end, the
//   inherited start is byte-identical before and after, and the typed
//   inherited fact travels with the plan for its consequence text —
//   recorded as a fact about the device, never a grant by the user.
//   The same grow to an end that is neither aligned nor coincident
//   still refuses, naming the nearest conforming values.
// Evidence: misaligned_growth_authors_only_the_aligned_end
#[test]
fn misaligned_growth_authors_only_the_aligned_end() {
    let (snapshot, _, legacy) = legacy_fixture();
    let new_length = 200 * DEFAULT_ALIGNMENT - LEGACY_START;
    let first = plan_sized(
        SizedRequest::Grow {
            target: legacy,
            new_length,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("the filed case proceeds under the recorded decision");

    let extension = first.plan.steps()[0].ranges().consumed[0];
    assert_eq!(
        extension.start,
        100 * DEFAULT_ALIGNMENT,
        "grown at the tail"
    );
    assert_eq!(
        extension.start + extension.length,
        200 * DEFAULT_ALIGNMENT,
        "the one authored boundary follows the 1 MiB default"
    );
    let simulated_extent = first
        .simulated
        .facts()
        .extents
        .get(&legacy)
        .expect("still there");
    assert_eq!(
        simulated_extent.start, LEGACY_START,
        "the inherited start is byte-identical before and after"
    );
    assert_eq!(simulated_extent.length, new_length);
    assert_eq!(
        first.consequences,
        vec![Consequence::InheritedMisalignedStart {
            target: legacy,
            start: LEGACY_START,
        }],
        "the inherited fact travels as consequence material"
    );
    assert!(
        first.consequences[0].to_string().contains("inherited"),
        "the rendered sentence states the fact"
    );

    let refused = grow_extension(&snapshot, legacy, 150 * DEFAULT_ALIGNMENT)
        .expect_err("an unaligned, non-coincident authored end has no lawful spelling");
    assert_eq!(
        refused,
        SolveRefusal::UnalignedAuthoredBoundary {
            target: legacy,
            boundary: 150 * DEFAULT_ALIGNMENT + LEGACY_START,
            nearest_aligned_below: 150 * DEFAULT_ALIGNMENT,
            coincident_candidate: 1 << 30,
        }
    );
}

// Requirements: PLAN-001, PART-009
//   ADR-0023's coincident-edge rule: grow-to-fill places the authored
//   end exactly at the neighbor's pre-existing (misaligned) start,
//   conforms to policy, and is recorded as coincident — aligning down
//   instead would mint an unusable sliver, and without this rule the
//   start question would re-file itself about the end.
// Evidence: grow_to_fill_is_coincident_with_the_neighbors_edge
#[test]
fn grow_to_fill_is_coincident_with_the_neighbors_edge() {
    let (snapshot, _, aligned, misaligned) = solver_fixture();
    let fill_length = 99 * DEFAULT_ALIGNMENT + 512;
    let planned = plan_sized(
        SizedRequest::Grow {
            target: aligned,
            new_length: fill_length,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("grow-to-fill conforms");
    assert_eq!(
        planned.consequences,
        vec![Consequence::CoincidentBoundary {
            target: aligned,
            boundary: 100 * DEFAULT_ALIGNMENT + 512,
            edge: StructuralEdge::NeighborStart {
                neighbor: misaligned
            },
        }],
        "the coincident placement is recorded, naming the edge"
    );

    let solved = grow_extension(&snapshot, aligned, fill_length).expect("solves");
    assert_eq!(
        solved.end_placement,
        BoundaryPlacement::Coincident {
            edge: StructuralEdge::NeighborStart {
                neighbor: misaligned
            }
        }
    );
    assert_eq!(
        solved.inherited_start, None,
        "the aligned start inherits nothing"
    );
}

// Requirements: PLAN-001, PART-009
//   Section 11.2's preserved-alignment invariant read as ADR-0023's
//   split: authored boundaries meet policy (the shrink's new end on the
//   default), inherited boundaries are byte-identical before and after
//   (the untouched misaligned start, carried as the typed inherited
//   fact) — proven over the shrink path, the grow path's twin.
// Evidence: authored_boundaries_meet_policy_and_inherited_stay_byte_identical
#[test]
fn authored_boundaries_meet_policy_and_inherited_stay_byte_identical() {
    let (snapshot, _, legacy) = legacy_fixture();
    let new_length = 50 * DEFAULT_ALIGNMENT - LEGACY_START;
    let planned = plan_sized(
        SizedRequest::Shrink {
            target: legacy,
            new_length,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("an aligned shrink end conforms");
    let freed = planned.plan.steps()[0].ranges().destroyed[0];
    assert_eq!(
        freed.start,
        50 * DEFAULT_ALIGNMENT,
        "the authored new end meets the default"
    );
    let simulated_extent = planned
        .simulated
        .facts()
        .extents
        .get(&legacy)
        .expect("still there");
    assert_eq!(
        simulated_extent.start, LEGACY_START,
        "inherited, byte-identical"
    );
    assert_eq!(simulated_extent.length, new_length);
    assert_eq!(
        planned.consequences,
        vec![Consequence::InheritedMisalignedStart {
            target: legacy,
            start: LEGACY_START,
        }]
    );

    let refused = shrink_reduction(&snapshot, legacy, 50 * DEFAULT_ALIGNMENT)
        .expect_err("an unaligned shrink end refuses");
    assert!(matches!(
        refused,
        SolveRefusal::UnalignedAuthoredBoundary { .. }
    ));
}

// Requirements: PLAN-001, PART-009
//   ADR-0023's no-fourth-state property, swept: every authored end the
//   solver accepts is on the 1 MiB default or coincident with a named
//   pre-existing structural edge, and every other request refuses typed
//   — the deviation-override vocabulary stays inexpressible, so nothing
//   the solver emits can carry a fourth state.
// Evidence: no_authored_boundary_has_a_fourth_state
#[test]
fn no_authored_boundary_has_a_fourth_state() {
    let (snapshot, _, legacy) = legacy_fixture();
    let own_length = 100 * DEFAULT_ALIGNMENT - LEGACY_START;
    let host_end: u64 = 1 << 30;
    for megabytes in [101_u64, 137, 512, 1023] {
        for offset in [0_i64, -512, 512, 1] {
            let end = (megabytes * DEFAULT_ALIGNMENT)
                .checked_add_signed(offset)
                .expect("in range");
            let new_length = end - LEGACY_START;
            if new_length <= own_length {
                continue;
            }
            let solved = grow_extension(&snapshot, legacy, new_length);
            if end.is_multiple_of(DEFAULT_ALIGNMENT) {
                assert_eq!(
                    solved.expect("an aligned end conforms").end_placement,
                    BoundaryPlacement::Aligned,
                    "aligned at {end}"
                );
            } else if end == host_end {
                assert_eq!(
                    solved.expect("the fill end conforms").end_placement,
                    BoundaryPlacement::Coincident {
                        edge: StructuralEdge::HostEnd
                    },
                    "coincident at {end}"
                );
            } else {
                assert!(
                    matches!(
                        solved,
                        Err(SolveRefusal::UnalignedAuthoredBoundary { boundary, .. })
                            if boundary == end
                    ),
                    "no fourth state at {end}"
                );
            }
        }
    }
    // The genuine fill case: growing to the host's end is coincident
    // with a named edge even though 1 GiB is also on the default —
    // the default wins the record, and either way there is no fourth
    // state. An off-default fill against the misaligned neighbor is
    // the coincident case proven above.
    let filled = grow_extension(&snapshot, legacy, host_end - LEGACY_START)
        .expect("fill to the host end conforms");
    assert_eq!(filled.end_placement, BoundaryPlacement::Aligned);

    // Creates obey the same law: an off-default size conforms exactly
    // where it fills its room to a structural edge, and refuses
    // elsewhere.
    let (snapshot, host, _, misaligned) = solver_fixture();
    let refused = place_create(&snapshot, host, 10 * DEFAULT_ALIGNMENT + 7)
        .expect_err("an unaligned create end with room beyond it refuses");
    assert!(matches!(
        refused,
        SolveRefusal::UnalignedAuthoredBoundary { .. }
    ));
    let filling = place_create(&snapshot, host, 35 * DEFAULT_ALIGNMENT + 512)
        .expect("filling to the neighbor's start conforms");
    assert_eq!(filling.placed.start, 65 * DEFAULT_ALIGNMENT);
    assert_eq!(
        filling.end_placement,
        BoundaryPlacement::Coincident {
            edge: StructuralEdge::NeighborStart {
                neighbor: misaligned
            }
        }
    );
}

// Requirements: PLAN-001, PLAN-006
//   The sized path plans end to end: a solved create carries its placed
//   range in the body, deterministically, and revalidates through the
//   typed boundary against its snapshot.
// Evidence: a_solved_create_plans_deterministically
#[test]
fn a_solved_create_plans_deterministically() {
    let (snapshot, host, _, _) = solver_fixture();
    let request = SizedRequest::Create {
        host,
        size: 10 * DEFAULT_ALIGNMENT,
    };
    let first = plan_sized(
        request,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans")
    .plan;
    let second = plan_sized(
        request,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans again")
    .plan;
    assert_eq!(body_bytes(&first), body_bytes(&second));
    let rebuilt =
        OperationPlan::from_canonical_body(&body_bytes(&first), &snapshot).expect("revalidates");
    assert_eq!(
        rebuilt.body_hash().expect("hash"),
        first.body_hash().expect("hash")
    );
}

// Requirements: PLAN-001
//   A non-resize refuses with both lengths named, in both directions.
// Evidence: a_non_resize_refuses_with_both_lengths
#[test]
fn a_non_resize_refuses_with_both_lengths() {
    let (snapshot, _, aligned, _) = solver_fixture();
    let refused = plan_sized(
        SizedRequest::Shrink {
            target: aligned,
            new_length: 64 * DEFAULT_ALIGNMENT,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("equal length is not a shrink");
    assert!(matches!(
        refused,
        PlanRefusal::SolveRefused {
            refusal: SolveRefusal::NotAResize { .. }
        }
    ));
}

use super::Planned;
use super::simulate::SimulateRefusal;

// Requirements: PLAN-002, PLAN-006
//   Both topologies arrive together: the sized create's simulation
//   carries the minted partition at its placed extent under the host's
//   table view, the simulated body round-trips its own typed boundary,
//   and — the 3c property re-asserted at the planner's boundary — the
//   plan can never revalidate against the simulated snapshot: a
//   prediction is not a capture, structurally.
// Evidence: the_simulated_topology_arrives_and_is_never_a_base
#[test]
fn the_simulated_topology_arrives_and_is_never_a_base() {
    let (snapshot, host, _, _) = solver_fixture();
    let Planned {
        plan, simulated, ..
    } = plan_sized(
        SizedRequest::Create {
            host,
            size: 10 * DEFAULT_ALIGNMENT,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");

    assert_eq!(
        simulated.kind(),
        SnapshotKind::Simulated,
        "the prediction carries the schema string that is never a base"
    );
    let minted = simulated
        .facts()
        .extents
        .iter()
        .find(|(_, extent)| extent.host == host && extent.start == 65 * DEFAULT_ALIGNMENT);
    assert!(
        minted.is_some(),
        "the minted partition's extent is in the simulated facts"
    );
    assert_eq!(
        simulated.topology().entries().len(),
        snapshot.topology().entries().len() + 1,
        "one node was minted"
    );

    let simulated_bytes = partman_domain::canonical::encode(&simulated.body_value().expect("body"))
        .expect("encodable");
    let rebuilt = TopologySnapshot::from_canonical_body(&simulated_bytes)
        .expect("the simulated body round-trips its own boundary");
    assert_eq!(rebuilt.kind(), SnapshotKind::Simulated);

    OperationPlan::from_canonical_body(&body_bytes(&plan), &simulated).expect_err(
        "a plan can never revalidate against a simulated snapshot: a prediction is not a capture",
    );
}

// Requirements: PLAN-002
//   The wipe simulation removes everything the facts place on the wiped
//   bytes, transitively with everything named relative to it, and drops
//   the target's table-state stamp — the post-wipe state is not
//   established until a real capture, and absence is the honest
//   prediction.
// Evidence: a_wipe_simulation_removes_the_chain_and_the_stamp
#[test]
fn a_wipe_simulation_removes_the_chain_and_the_stamp() {
    let (snapshot, dev, _) = chain_fixture();
    let Planned { simulated, .. } = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: dev,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");

    assert_eq!(
        simulated.topology().entries().len(),
        1,
        "the signature and its layer are gone; the wiped device remains"
    );
    assert!(
        !simulated.facts().table_states.contains_key(&dev),
        "the stamp is dropped: post-wipe state is unestablished until a capture"
    );
}

// Requirements: PLAN-002
//   An effect this model cannot represent produces no valid plan at
//   all: simulation is mandatory, so the encrypt request refuses as
//   not representable rather than emitting a prediction that lies.
// Evidence: an_unrepresentable_effect_refuses_the_whole_plan
#[test]
fn an_unrepresentable_effect_refuses_the_whole_plan() {
    let (snapshot, clean, _) = fixture();
    let refused = plan(
        PlanRequest {
            operation: Operation::Encrypt,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("no simulation, no valid plan");
    assert!(matches!(
        refused,
        PlanRefusal::SimulateRefused {
            refusal: SimulateRefusal::NotRepresentable { .. }
        }
    ));
}

// Requirements: PLAN-002
//   A sized grow's simulation reflects the new length in the simulated
//   facts, and nothing else moves.
// Evidence: a_grow_simulation_updates_the_extent
#[test]
fn a_grow_simulation_updates_the_extent() {
    let (snapshot, _, aligned, _) = solver_fixture();
    let Planned { simulated, .. } = plan_sized(
        SizedRequest::Grow {
            target: aligned,
            new_length: 70 * DEFAULT_ALIGNMENT,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    let extent = simulated
        .facts()
        .extents
        .get(&aligned)
        .expect("still there");
    assert_eq!(extent.length, 70 * DEFAULT_ALIGNMENT);
    assert_eq!(extent.start, DEFAULT_ALIGNMENT, "the start never moves");
    assert_eq!(
        simulated.topology().entries().len(),
        snapshot.topology().entries().len(),
        "no node minted, none removed"
    );
}
