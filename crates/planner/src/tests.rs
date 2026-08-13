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

use super::EmittedReversal;
use partman_domain::model::plan::{
    BindRefusal, DraftPrecondition, DraftTarget, ImpossibilityReason, ReversalLinkage,
};
use partman_domain::model::step::Precondition;

/// The post-apply capture for the solver fixture's 10 MiB create at
/// 65 MiB: the created partition placed for real, optionally with a
/// filesystem landed inside it (the truth-decay world).
fn created_capture(with_fs: bool) -> TopologySnapshot {
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
    let created = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 65 * DEFAULT_ALIGNMENT,
    };
    let created_id = derive_id(&created).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: created_id,
        kind: partman_domain::model::naming::FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");

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
    facts.extents.insert(
        created_id,
        HostRange {
            host: host_id,
            start: 65 * DEFAULT_ALIGNMENT,
            length: 10 * DEFAULT_ALIGNMENT,
        },
    );
    let mut nodes = vec![host, table, aligned, misaligned, created];
    let mut edges = vec![
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
        Edge {
            kind: EdgeKind::Containment,
            source: table_id,
            target: created_id,
        },
    ];
    if with_fs {
        facts.extents.insert(
            fs_id,
            HostRange {
                host: created_id,
                start: 0,
                length: DEFAULT_ALIGNMENT,
            },
        );
        nodes.push(fs);
        edges.push(Edge {
            kind: EdgeKind::Containment,
            source: created_id,
            target: fs_id,
        });
    }
    TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges, facts)
        .expect("assembles")
}

// Requirements: PLAN-008, PLAN-001
//   PLAN-008's first arm end to end (ADR-0022): the sized create emits
//   a truthful reversal draft — byte-deterministic, its target spelled
//   as the creating step's output, its truthfulness the created node's
//   emptiness — the forward body carries the draft by ID and body hash,
//   the draft names the forward plan by ID alone, the forward step's
//   Reversible claim stands on the draft, and the linked body still
//   revalidates through the typed boundary.
// Evidence: the_create_reversal_draft_is_deterministic_and_linked
#[test]
fn the_create_reversal_draft_is_deterministic_and_linked() {
    let (snapshot, host, _, _) = solver_fixture();
    let request = SizedRequest::Create {
        host,
        size: 10 * DEFAULT_ALIGNMENT,
    };
    let plan_once = || {
        plan_sized(
            request,
            &snapshot,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
            &identity(),
        )
        .expect("plans")
    };
    let first = plan_once();
    let second = plan_once();
    let EmittedReversal::Draft(draft) = &first.reversal else {
        panic!("the create emits a draft: {:?}", first.reversal);
    };
    let EmittedReversal::Draft(second_draft) = &second.reversal else {
        panic!("plans again");
    };
    assert_eq!(
        partman_domain::canonical::encode(&draft.body_value()).expect("encodable"),
        partman_domain::canonical::encode(&second_draft.body_value()).expect("encodable"),
        "PLAN-001 holds over the draft: byte-equal drafts for equal inputs"
    );

    assert_eq!(
        first.plan.reversal(),
        Some(&ReversalLinkage::Draft {
            plan_id: b"pln-1/reversal".to_vec(),
            draft_hash: draft.body_hash().expect("hashable"),
        }),
        "the forward body freezes the advertisement by ID and hash"
    );
    assert_eq!(
        draft.forward_plan_id(),
        b"pln-1",
        "the draft answers by ID alone"
    );
    assert_eq!(
        draft.steps()[0].target,
        DraftTarget::StepOutput(0),
        "a created node is spelled as the creating step's output, never an address"
    );
    assert_eq!(
        draft.steps()[0].preconditions,
        vec![DraftPrecondition::StepOutputUnoccupied { step: 0 }]
    );
    assert_eq!(
        first.plan.severity(),
        Severity::Reversible,
        "the Reversible claim is made exactly where the truthful draft exists"
    );
    let rebuilt = OperationPlan::from_canonical_body(&body_bytes(&first.plan), &snapshot)
        .expect("the linked body revalidates");
    assert_eq!(
        rebuilt.body_hash().expect("hash"),
        first.plan.body_hash().expect("hash")
    );
}

// Requirements: PLAN-008
//   The reference resolves against a post-apply capture and refuses
//   against the pre-apply one (ADR-0022's verification): binding
//   produces an ordinary plan bound to the capture's hash whose own
//   linkage is the reapply-forward statement — and the prediction
//   itself never binds.
// Evidence: the_draft_binds_after_apply_and_never_to_the_prediction
#[test]
fn the_draft_binds_after_apply_and_never_to_the_prediction() {
    let (snapshot, host, _, _) = solver_fixture();
    let planned = plan_sized(
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
    let EmittedReversal::Draft(draft) = &planned.reversal else {
        panic!("the create emits a draft");
    };

    let post = created_capture(false);
    let bound = draft
        .bind(&post, &planned.plan)
        .expect("the reference resolves against the post-apply capture");
    assert_eq!(
        bound.snapshot_hash(),
        &post.body_hash().expect("hashable"),
        "binding is a validation act"
    );
    assert_eq!(
        bound.reversal(),
        Some(&ReversalLinkage::ReapplyForward {
            forward_plan_id: b"pln-1".to_vec()
        }),
        "the regress terminates in a reference"
    );

    let refused = draft
        .bind(&snapshot, &planned.plan)
        .expect_err("the pre-apply world resolves nothing");
    assert_eq!(
        refused,
        BindRefusal::UnresolvedReference {
            step: 0,
            candidates: 0
        }
    );

    assert_eq!(
        draft.bind(&planned.simulated, &planned.plan),
        Err(BindRefusal::PredictionNeverBinds),
        "nobody ever applies a prediction"
    );
}

// Requirements: PLAN-008
//   Truthfulness is a two-time property (ADR-0022's named fixture): the
//   draft that was metadata-only at emission refuses by precondition
//   once anything lands in the created structure, instead of silently
//   becoming a destructive plan wearing a reversal's advertisement.
// Evidence: a_decayed_reversal_refuses_instead_of_destroying
#[test]
fn a_decayed_reversal_refuses_instead_of_destroying() {
    let (snapshot, host, _, _) = solver_fixture();
    let planned = plan_sized(
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
    let EmittedReversal::Draft(draft) = &planned.reversal else {
        panic!("the create emits a draft");
    };
    let decayed = created_capture(true);
    let refused = draft
        .bind(&decayed, &planned.plan)
        .expect_err("data landed in the created partition");
    assert!(
        matches!(refused, BindRefusal::PreconditionFailed { .. }),
        "the decay refuses by precondition: {refused:?}"
    );
}

// Requirements: PLAN-008
//   The grow's draft shrinks back while the reclaimed tail is clean:
//   an address-spelled target (the target pre-exists), the tail's
//   emptiness carried in the target's own address space, and the
//   forward severity deliberately conservative — a draft does not
//   compel the Reversible claim.
// Evidence: the_grow_draft_shrinks_back_while_the_tail_is_clean
#[test]
fn the_grow_draft_shrinks_back_while_the_tail_is_clean() {
    let (snapshot, _, aligned, _) = solver_fixture();
    let planned = plan_sized(
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
    let EmittedReversal::Draft(draft) = &planned.reversal else {
        panic!("the grow emits a draft: {:?}", planned.reversal);
    };
    assert_eq!(draft.steps()[0].target, DraftTarget::Address(aligned));
    assert_eq!(
        draft.steps()[0].preconditions,
        vec![DraftPrecondition::Carried(Precondition::RegionUnoccupied {
            region: HostRange {
                host: aligned,
                start: 64 * DEFAULT_ALIGNMENT,
                length: 6 * DEFAULT_ALIGNMENT,
            }
        })],
        "the reclaimed tail is judged in the target's own address space"
    );
    assert_eq!(
        planned.plan.severity(),
        Severity::Disruptive,
        "conservative-up, stated: the draft exists, the claim is not compelled"
    );
}

// Requirements: PLAN-008
//   PLAN-008's second arm: operations with no truthful reversal state
//   why, per step, machine-readably — the wipe's destroyed bytes, the
//   identity write's uncarried prior value — and the statements ride
//   the hashed body as the linkage.
// Evidence: unreversible_operations_state_why_per_step
#[test]
fn unreversible_operations_state_why_per_step() {
    let (snapshot, clean, _) = fixture();
    let wiped = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    let EmittedReversal::Impossible(statements) = &wiped.reversal else {
        panic!("a wipe has no truthful reversal");
    };
    assert_eq!(statements.len(), 1);
    assert_eq!(statements[0].reason, ImpossibilityReason::DataDestroyed);
    assert!(matches!(
        wiped.plan.reversal(),
        Some(ReversalLinkage::Impossible { statements }) if statements.len() == 1
    ));

    let (chain, dev, luks) = chain_fixture();
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
    let planned = plan_set(
        &set,
        &chain,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    let EmittedReversal::Impossible(statements) = &planned.reversal else {
        panic!("the wipe chain has no truthful reversal");
    };
    assert_eq!(
        statements
            .iter()
            .map(|statement| statement.step)
            .collect::<Vec<_>>(),
        vec![0, 1],
        "statements cover the emitted step order exactly"
    );
}

use super::{ProtectionObligation, RepairRequest, plan_repair};
use partman_domain::model::identity::IndeterminateCause;
use partman_domain::model::step::{Acknowledgment, StepClass};

/// A device with the given authored table state, its table located as
/// a child extent when asked — the worlds ADR-0024's arms select over.
fn stateful_device(
    serial: &[u8],
    state: TableState,
    with_table: bool,
) -> (TopologySnapshot, NodeId, HostRange) {
    let dev = device(serial);
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: partman_domain::model::naming::TableRole::Gpt,
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
    facts.table_states.insert(dev_id, state);
    let (nodes, edges) = if with_table {
        facts.extents.insert(table_id, table_region);
        (
            vec![dev, table],
            vec![Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: table_id,
            }],
        )
    } else {
        (vec![dev], vec![])
    };
    let snapshot = TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges, facts)
        .expect("assembles");
    (snapshot, dev_id, table_region)
}

// Requirements: PART-013, PLAN-001
//   ADR-0024's positively determined arms at the planner: a Present
//   table's plan carries the parse-backup obligation, and a blank
//   device's plan discharges as the journaled determination — a value,
//   not a skip, and no user acknowledgement is demanded or carried.
// Evidence: each_positively_determined_state_selects_its_protection_arm
#[test]
fn each_positively_determined_state_selects_its_protection_arm() {
    let (snapshot, clean, _) = fixture();
    let planned = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert_eq!(
        planned.protection,
        vec![ProtectionObligation::ParseBackup { device: clean }],
        "Present: the parse-level backup stands untouched"
    );

    let (blank, blank_dev, _) = stateful_device(b"PRT-BLANK", TableState::Absent, false);
    let planned = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: blank_dev,
        },
        &blank,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("a blank device plans");
    assert_eq!(
        planned.protection,
        vec![ProtectionObligation::JournaledDetermination { device: blank_dev }],
        "Absent: the record is the determination itself"
    );
    assert!(
        planned.plan.steps()[0].acknowledgments().is_empty(),
        "no user acknowledgement is demanded on a fact the user cannot inform"
    );
}

// Requirements: PART-013
//   ADR-0024's ordinary arm: an ordinary operation against
//   Indeterminate media refuses before any protection obligation is
//   computed — SAFE-005's planner half, with PART-013 never reached —
//   on the canonical path and the sized path alike.
// Evidence: an_ordinary_operation_on_indeterminate_media_refuses_before_protection
#[test]
fn an_ordinary_operation_on_indeterminate_media_refuses_before_protection() {
    let (snapshot, dev, _) = stateful_device(
        b"PRT-BAD",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
        true,
    );
    let refused = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: dev,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("SAFE-005 disables the write");
    assert_eq!(
        refused,
        PlanRefusal::TableStateIndeterminate { device: dev }
    );

    let refused = plan_sized(
        SizedRequest::Create {
            host: dev,
            size: 10 * DEFAULT_ALIGNMENT,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("the sized path refuses the same way");
    assert_eq!(
        refused,
        PlanRefusal::TableStateIndeterminate { device: dev }
    );
}

// Requirements: PART-013, PLAN-001
//   ADR-0024's repair arm: the typed table-repair family plans over
//   Indeterminate media, its write targets are exactly the located
//   table regions and the raw-capture obligation names exactly them,
//   the simulation drops the stamp (post-repair state unestablished
//   until a real capture), the reversal is the pre-state-preserved
//   statement, and the linked body revalidates. Fail-closed edges: no
//   located table refuses, and a positively determined state refuses —
//   the family exists for Indeterminate tables.
// Evidence: the_repair_family_captures_exactly_its_write_targets
#[test]
fn the_repair_family_captures_exactly_its_write_targets() {
    let (snapshot, dev, table_region) = stateful_device(
        b"PRT-RPR",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
        true,
    );
    let request = RepairRequest {
        target: dev,
        acknowledged_uncapturable: None,
    };
    let planned = plan_repair(
        &request,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("the repair family plans over the unsound source");
    assert_eq!(planned.plan.steps()[0].class(), StepClass::TableRepair);
    assert_eq!(
        planned.plan.steps()[0].ranges().written_table_extents,
        vec![table_region],
        "the write targets are exactly the located table regions"
    );
    assert_eq!(
        planned.protection,
        vec![ProtectionObligation::RawCapture {
            device: dev,
            regions: vec![table_region],
        }],
        "the raw capture preserves exactly what the plan will write"
    );
    assert!(
        !planned.simulated.facts().table_states.contains_key(&dev),
        "the stamp drops: the post-repair state is not established until a capture"
    );
    let EmittedReversal::Impossible(statements) = &planned.reversal else {
        panic!("the repair's reversal is a statement");
    };
    assert_eq!(
        statements[0].reason,
        ImpossibilityReason::PreStatePreservedForRecovery
    );
    let rebuilt = OperationPlan::from_canonical_body(&body_bytes(&planned.plan), &snapshot)
        .expect("the classed body revalidates");
    assert_eq!(
        rebuilt.body_hash().expect("hash"),
        planned.plan.body_hash().expect("hash")
    );

    let (tableless, tableless_dev, _) = stateful_device(
        b"PRT-LOST",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
        false,
    );
    let refused = plan_repair(
        &RepairRequest {
            target: tableless_dev,
            acknowledged_uncapturable: None,
        },
        &tableless,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("no located table, no invented regions");
    assert_eq!(
        refused,
        PlanRefusal::RepairWithoutLocatedTable {
            device: tableless_dev
        }
    );

    let (present, present_dev, _) = fixture_present_device();
    let refused = plan_repair(
        &RepairRequest {
            target: present_dev,
            acknowledged_uncapturable: None,
        },
        &present,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("the family exists for Indeterminate tables");
    assert_eq!(
        refused,
        PlanRefusal::RepairNeedsAnIndeterminateTable {
            device: present_dev
        }
    );
}

/// A Present-state device with a located table, for the repair family's
/// wrong-state refusal.
fn fixture_present_device() -> (TopologySnapshot, NodeId, HostRange) {
    stateful_device(
        b"PRT-OK",
        TableState::Present {
            checksum: canonical::hash(&Value::Text("present twin".into())).expect("hashable"),
        },
        true,
    )
}

// Requirements: PART-013
//   ADR-0024's capture-impossible arm: the plan proceeds only under the
//   plan-creation acknowledgement naming the exact uncapturable
//   regions; the acknowledgement rides the hashed body and the
//   obligation becomes acknowledged-unpreserved naming exactly those
//   regions — never a mid-flight prompt, never available outside the
//   typed family (the constructor law, held in the domain suite).
// Evidence: capture_impossible_proceeds_only_under_the_named_acknowledgement
#[test]
fn capture_impossible_proceeds_only_under_the_named_acknowledgement() {
    let (snapshot, dev, table_region) = stateful_device(
        b"PRT-ACK",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
        true,
    );
    let planned = plan_repair(
        &RepairRequest {
            target: dev,
            acknowledged_uncapturable: Some(vec![table_region]),
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("the acknowledged plan proceeds");
    assert_eq!(
        planned.protection,
        vec![ProtectionObligation::AcknowledgedUnpreserved {
            device: dev,
            regions: vec![table_region],
        }],
        "the obligation names exactly the acknowledged regions"
    );
    assert_eq!(
        planned.plan.steps()[0].acknowledgments(),
        &[Acknowledgment::UncapturableRegions {
            table: dev,
            regions: vec![table_region],
        }],
        "the acknowledgement rides the hashed body"
    );
    let rebuilt = OperationPlan::from_canonical_body(&body_bytes(&planned.plan), &snapshot)
        .expect("revalidates with the acknowledgement intact");
    assert_eq!(rebuilt.steps()[0].acknowledgments().len(), 1);
}

use super::{
    CancelClaim, ClaimRefusal, InterruptionProfile, cancellation_class, interruption_profile,
    irreversible_after_start, plan_flags,
};
use partman_domain::model::plan::{
    DraftPrecondition as DP, DraftStep, PlanError, ReversalDraft, StepImpossibility as Statement,
};
use partman_domain::model::protection::StepRanges;
use partman_domain::model::step::Cancellation;
use partman_domain::model::step::{PlanStep, StepFlags, StepRisk};

// Requirements: PLAN-004
//   ADR-0025's criterion partitions real step families (SI-17
//   resolved, spec 12.3.0): the PART-005 journaled chunk copy has
//   windows but no unrestorable intermediate — unflagged; the in-place
//   destructive and transforming families are flagged; entry-level
//   writes land entirely or not at all — unflagged. The flag is
//   derived from the criterion, never declared ad hoc.
// Evidence: the_criterion_partitions_step_families
#[test]
fn the_criterion_partitions_step_families() {
    assert_eq!(
        interruption_profile(Operation::Move),
        InterruptionProfile::RecoverableIntermediate,
        "the journaled chunk copy: windows, but always recoverable"
    );
    assert!(!irreversible_after_start(interruption_profile(
        Operation::Copy
    )));
    assert_eq!(
        interruption_profile(Operation::Wipe),
        InterruptionProfile::UnrestorableIntermediate,
        "in-place destruction: the first write forecloses unwinding"
    );
    assert!(irreversible_after_start(interruption_profile(
        Operation::Shrink
    )));
    assert_eq!(
        interruption_profile(Operation::Create),
        InterruptionProfile::LandsEntirelyOrNot
    );
    assert!(!irreversible_after_start(interruption_profile(
        Operation::Label
    )));

    // The derivation reaches the emitted plans: the wipe is flagged,
    // the create is not.
    let (snapshot, clean, _) = fixture();
    let wiped = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert!(
        wiped.plan.steps()[0].risk().flags.irreversible_after_start,
        "the wipe carries the flag its criterion derives"
    );
    let (solver, host, _, _) = solver_fixture();
    let created = plan_sized(
        SizedRequest::Create {
            host,
            size: 10 * DEFAULT_ALIGNMENT,
        },
        &solver,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert!(
        !created.plan.steps()[0]
            .risk()
            .flags
            .irreversible_after_start,
        "an entry-level write is unflagged"
    );
}

// Requirements: PLAN-005
//   The cancellation class partitions the operation vocabulary under
//   the WP-060 recorded cancellation-class decision (2026-08-12): the
//   journaled-chunk-copy family is checkpoint-cancellable on PART-005's
//   durable progress map and ACC-012's declared checkpoint — stated for
//   the family before the planner emits it — and every family the
//   planner emits today sits on the non-cancellable floor, each
//   earning more only through the decision's named revisit conditions.
//   The class is a stated declaration, never a derivation from the
//   interruption profile: cannot-stop and cannot-unwind are
//   independent facts in both directions (spec 12.3.0), and the
//   partition exhibits all the combinations that exist today.
// Evidence: the_cancellation_class_partitions_step_families
#[test]
fn the_cancellation_class_partitions_step_families() {
    assert_eq!(
        cancellation_class(Operation::Move),
        Cancellation::CheckpointCancellable,
        "the journaled chunk copy stops at its declared checkpoints"
    );
    assert_eq!(
        cancellation_class(Operation::Copy),
        Cancellation::CheckpointCancellable
    );
    for floor in [
        Operation::Wipe,
        Operation::Shrink,
        Operation::Create,
        Operation::Grow,
        Operation::Label,
        Operation::Uuid,
        Operation::Repair,
        Operation::Encrypt,
        Operation::Decrypt,
    ] {
        assert_eq!(
            cancellation_class(floor),
            Cancellation::NonCancellable,
            "no measured safe-stop story: {floor:?} sits on the floor"
        );
    }

    // The recorded independence, visible in the vocabulary: the entry
    // write cannot stop yet unwinds trivially (unflagged); the chunk
    // copy stops at checkpoints yet is unflagged; the wipe can neither
    // stop nor unwind. Neither axis derives the other.
    assert!(!irreversible_after_start(interruption_profile(
        Operation::Label
    )));
    assert!(!irreversible_after_start(interruption_profile(
        Operation::Move
    )));
    assert!(irreversible_after_start(interruption_profile(
        Operation::Wipe
    )));
}

// Requirements: PLAN-005, MODEL-005
//   The declaration reaches the emitted body end to end: every step of
//   an emitted plan carries its stated class in the hashed version-4
//   body — the wipe and the repair on the floor, spelled exactly as
//   PLAN-005 spells it — and the declaration survives the typed
//   boundary's recompute.
// Evidence: the_emitted_body_carries_the_cancellation_declaration
#[test]
fn the_emitted_body_carries_the_cancellation_declaration() {
    let (snapshot, clean, _) = fixture();
    let wiped = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert_eq!(
        wiped.plan.steps()[0].cancellation(),
        cancellation_class(Operation::Wipe),
        "the emitted step carries the stated class"
    );
    let Value::Map(body) = wiped.plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    let Some(Value::Array(steps)) = body.get("steps") else {
        panic!("steps present");
    };
    let Value::Map(step_map) = &steps[0] else {
        panic!("step is a map");
    };
    assert_eq!(
        step_map.get("cancellation"),
        Some(&Value::Text("non-cancellable".to_owned())),
        "the hashed body spells the floor exactly as PLAN-005 spells it"
    );

    let bytes = canonical::encode(&Value::Map(body)).expect("encodable");
    let rebuilt =
        partman_domain::model::plan::OperationPlan::from_canonical_body(&bytes, &snapshot)
            .expect("revalidates");
    assert_eq!(
        rebuilt.steps()[0].cancellation(),
        Cancellation::NonCancellable,
        "the declaration survives the boundary's recompute"
    );

    // The typed repair family declares through the same seam.
    let (repair_world, device, _) = stateful_device(
        b"PRT-CXL",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
        true,
    );
    let repaired = plan_repair(
        &RepairRequest {
            target: device,
            acknowledged_uncapturable: None,
        },
        &repair_world,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert_eq!(
        repaired.plan.steps()[0].cancellation(),
        Cancellation::NonCancellable,
        "the repair family sits on the floor"
    );
}

// Requirements: PLAN-004, PLAN-008
//   ADR-0025's contested combination constructs (the planner's named
//   withholding replaced by the decided behavior): severity 1 with
//   `irreversible-after-start` assembles through the sole constructors
//   exactly when its truthful reversal draft stands beside it —
//   endpoints fully undoable, mid-window roll-forward-only — and the
//   same construction with no draft still refuses (ADR-0022's rule,
//   unchanged by the flag).
// Evidence: the_combination_constructs_with_its_draft_and_refuses_without
#[test]
fn the_combination_constructs_with_its_draft_and_refuses_without() {
    let (snapshot, host, _, _) = solver_fixture();
    let placed = HostRange {
        host,
        start: 65 * DEFAULT_ALIGNMENT,
        length: 10 * DEFAULT_ALIGNMENT,
    };
    let combination = StepRisk {
        severity: Severity::Reversible,
        flags: StepFlags {
            irreversible_after_start: true,
            ..StepFlags::default()
        },
    };
    let step = PlanStep::mutating(
        &snapshot,
        host,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![placed],
            destroyed: vec![],
        },
        vec![],
        combination,
    )
    .expect("the combination is legal at the step layer");

    // The truthful draft, composed against the real prediction.
    let proposal = plan_sized(
        SizedRequest::Create {
            host,
            size: 10 * DEFAULT_ALIGNMENT,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans")
    .simulated;
    let draft = ReversalDraft::compose(
        b"combo/reversal".to_vec(),
        1_700_000_000,
        &proposal,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        vec![DraftStep {
            target: DraftTarget::StepOutput(0),
            ranges: StepRanges {
                written_table_extents: vec![],
                consumed: vec![],
                destroyed: vec![placed],
            },
            acknowledgments: vec![],
            risk: combination,
            preconditions: vec![DP::StepOutputUnoccupied { step: 0 }],
        }],
        b"combo".to_vec(),
        std::slice::from_ref(&step),
    )
    .expect("the draft composes");

    let with_draft = OperationPlan::assemble_linked(
        b"combo".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step.clone()],
        partman_domain::model::plan::ReversalLinkage::Draft {
            plan_id: draft.plan_id().to_vec(),
            draft_hash: draft.body_hash().expect("hashable"),
        },
    )
    .expect("severity 1 plus the flag assembles on its truthful draft");
    assert_eq!(with_draft.severity(), Severity::Reversible);
    assert!(plan_flags(&with_draft).irreversible_after_start);

    let without_draft = OperationPlan::assemble_linked(
        b"combo".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step],
        partman_domain::model::plan::ReversalLinkage::Impossible {
            statements: vec![Statement {
                step: 0,
                reason: ImpossibilityReason::DataDestroyed,
            }],
        },
    );
    assert_eq!(
        without_draft,
        Err(PlanError::ReversibleWithoutReversal),
        "no draft, no Reversible — the flag changes nothing there"
    );
}

// Requirements: PLAN-004, PLAN-005
//   ADR-0025's coupling rule, unconstructible rather than discouraged:
//   a flagged step's cancellation claims `no-writes` only before its
//   first write; after it, the honest outcomes are `partial` or
//   completion, and the dishonest claim has no constructor. Cannot-stop
//   and cannot-unwind stay independent — an unflagged step's post-write
//   `no-writes` (an unwound table write) remains representable.
// Evidence: a_flagged_cancellation_never_claims_no_writes_after_its_first_write
#[test]
fn a_flagged_cancellation_never_claims_no_writes_after_its_first_write() {
    let flagged = StepRisk {
        severity: Severity::Reversible,
        flags: StepFlags {
            irreversible_after_start: true,
            ..StepFlags::default()
        },
    };
    let unflagged = StepRisk {
        severity: Severity::Destructive,
        flags: StepFlags::default(),
    };
    assert_eq!(
        CancelClaim::no_writes(flagged, false),
        Ok(CancelClaim::NoWrites),
        "before the first write, no-writes is trivially honest"
    );
    assert_eq!(
        CancelClaim::no_writes(flagged, true),
        Err(ClaimRefusal::FlaggedAfterFirstWrite),
        "after it, the claim has no constructor"
    );
    assert_eq!(
        CancelClaim::no_writes(unflagged, true),
        Ok(CancelClaim::NoWrites),
        "independence: cannot-unwind is the flag's fact, not severity's"
    );
    assert_eq!(CancelClaim::partial(), CancelClaim::Partial);
}

// Requirements: PLAN-004
//   The ceremony's inputs on the flagged severity-1 plan (ADR-0025's
//   fixture 4, the planner's half): the plan-level flag union is
//   nonempty even at severity 1, which under ADR-0021's closed
//   flags-nonempty rule binds the interactive ceremony — the
//   combination can never be applied unattended. The tier's
//   computation and enforcement are the helper packages'
//   (partman-journal carries the vocabulary), recorded as a boundary
//   in the assignment.
// Evidence: the_flagged_severity_one_plan_carries_ceremony_binding_flags
#[test]
fn the_flagged_severity_one_plan_carries_ceremony_binding_flags() {
    let (snapshot, dev, _) = stateful_device(
        b"PRT-CEB",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
        true,
    );
    let repaired = plan_repair(
        &RepairRequest {
            target: dev,
            acknowledged_uncapturable: None,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    assert!(
        plan_flags(&repaired.plan).irreversible_after_start,
        "the in-place rewrite carries the flag into the plan union"
    );

    let (solver, host, _, _) = solver_fixture();
    let created = plan_sized(
        SizedRequest::Create {
            host,
            size: 10 * DEFAULT_ALIGNMENT,
        },
        &solver,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    let union = plan_flags(&created.plan);
    assert!(
        !(union.security_sensitive
            || union.irreversible_after_start
            || union.requires_offline
            || union.requires_reboot
            || union.requires_rescue),
        "an unflagged severity-1 plan's union stays empty — the ceremony binds on facts, not on reflex"
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
