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
//   The ordered chain's two steps with the dependency removed refuse as
//   an unordered overlap naming both steps and the host: no order makes
//   concurrent effects on the same bytes deterministic, and the absent
//   dependency is exactly what would have explained them. The pair is
//   two wipes whose destroyed ranges truthfully overlap. It used to be a
//   wipe and an *unsized* create on one device, which overlapped only
//   because the create's canonical entry wrote the parent device
//   wholesale — the over-claim §2.1 forbids and issue #353 removes; an
//   unsized create's honest ground is the simulate refusal it already
//   gets, and the graph refusal is asserted on ranges that exist.
// Evidence: an_unordered_overlap_refuses_with_both_steps_named
#[test]
fn an_unordered_overlap_refuses_with_both_steps_named() {
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
        dependencies: vec![],
    };
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
                host: dev,
            }
        }
    );
}

/// A device with the given table state carrying one table view and one
/// partition at 1 MiB, the partition's extent framed on the device.
fn stateful_device_with_partition(
    serial: &[u8],
    state: TableState,
) -> (TopologySnapshot, NodeId, NodeId) {
    let dev = device(serial);
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: partman_domain::model::naming::TableRole::Gpt,
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
    facts.extents.insert(
        table_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 17_408,
        },
    );
    facts.extents.insert(
        part_id,
        HostRange {
            host: dev_id,
            start: 1 << 20,
            length: 256 << 20,
        },
    );
    facts.table_states.insert(dev_id, state);
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
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
    .expect("assembles");
    (snapshot, dev_id, part_id)
}

// Requirements: PART-013, PLAN-001
//   A write on a partition touches the disk that carries it, for the
//   arms that read touched devices: a Label on a partition of a device
//   with a Present table carries that device's parse-backup obligation,
//   and the same request against Indeterminate media refuses on the
//   device. Both are derived from the step's declared ranges, whose host
//   is the disk — the partition's canonical entry, framed on its device.
//   Nothing pinned this before issue #353's act; the domain suite is
//   blind to it (dropping every write entry survives there), so the
//   consumer that depends on it is where it is held.
// Evidence: a_partition_write_still_touches_its_disk_for_the_protection_arms
#[test]
fn a_partition_write_still_touches_its_disk_for_the_protection_arms() {
    let (present, dev, part) = stateful_device_with_partition(
        b"PRT-PART",
        TableState::Present {
            checksum: canonical::hash(&Value::Text("present part".into())).expect("hashable"),
        },
    );
    let planned = plan(
        PlanRequest {
            operation: Operation::Label,
            target: part,
        },
        &present,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("a label on a partition of a Present-table disk plans");
    assert_eq!(
        planned.protection,
        vec![ProtectionObligation::ParseBackup { device: dev }],
        "the partition's write touches its disk, whose table is backed up first"
    );

    let (indeterminate, dev, part) = stateful_device_with_partition(
        b"PRT-PART-IND",
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
    );
    let refused = plan(
        PlanRequest {
            operation: Operation::Label,
            target: part,
        },
        &indeterminate,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("an ordinary write on a partition of Indeterminate media refuses");
    assert_eq!(
        refused,
        PlanRefusal::TableStateIndeterminate { device: dev },
        "and it refuses on the disk, not the partition"
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
    BoundaryPlacement, DEFAULT_ALIGNMENT, OccupancyGround, SolveRefusal, StructuralEdge,
    free_extents, grow_extension, place_create, reserved_regions, shrink_reduction,
};
use super::{Consequence, SizedRequest, plan_sized};
use partman_domain::model::naming::TableRole;

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
//   Free space is the host's extent minus its children's and minus the
//   regions its declared table schemes claim, ascending — so the head
//   region the scheme withholds is not offered, the gap between
//   children stands, and the tail stops at the reserved ceiling rather
//   than the host's own end.
// Evidence: free_extents_are_the_hosts_minus_its_children
#[test]
fn free_extents_are_the_hosts_minus_its_children() {
    let (snapshot, host, _, _) = solver_fixture();
    // The precondition this test silently rested on, now asserted: the
    // fixture's GPT table node carries a containment edge and no
    // extent, so nothing but its scheme accounts for the regions it
    // claims. Before ADR-0036 that made (0, 1 MiB) read as free, and
    // this test asserted it.
    let table_id = derive_id(&NamingFields::PartitionTable {
        parent: host,
        role: partman_domain::model::naming::TableRole::Gpt,
    })
    .expect("derivable");
    assert!(
        !snapshot.facts().extents.contains_key(&table_id),
        "the table node is extent-less by construction"
    );

    let free = free_extents(&snapshot, host).expect("computes");
    let starts: Vec<(u64, u64)> = free
        .iter()
        .map(|range| (range.start, range.length))
        .collect();
    assert_eq!(
        starts,
        vec![
            (65 * DEFAULT_ALIGNMENT, 35 * DEFAULT_ALIGNMENT + 512),
            (
                132 * DEFAULT_ALIGNMENT + 512,
                (1 << 30) - DEFAULT_ALIGNMENT - (132 * DEFAULT_ALIGNMENT + 512)
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
        // In the containment root's frame — the device's — like the
        // partition it names as host (ADR-0037's anchoring rule).
        facts.extents.insert(
            fs_id,
            HostRange {
                host: host_id,
                start: 65 * DEFAULT_ALIGNMENT,
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
//   A swept capture stays swept across a simulated rebuild, and this is
//   the theorem rather than a coincidence. `Topology::build` refuses a
//   naming referent that resolves to nothing (issue #354); the
//   destruction closure removes everything named relative to a removed
//   node. Both read `NamingFields::naming_referents`, so a survivor can
//   never name a casualty — and the rebuild the closure feeds cannot
//   refuse for that reason. The failure this pins is specific and was
//   measured: with the `Volume` arm dropped from the shared roster, the
//   whole planner suite stayed green while the domain sweep turned this
//   plan into a hard `SimulateRefusal::Assembly`. A closure gap does not
//   produce a slightly wrong prediction now; it refuses the plan
//   outright. The volume is reached only through the referent walk, its
//   kind being one of the four that may carry no extent.
// Evidence: a_produced_volume_is_removed_with_its_producer_and_the_rebuild_stands
#[test]
fn a_produced_volume_is_removed_with_its_producer_and_the_rebuild_stands() {
    let dev = device(b"PLN-VOL");
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
    let volume = NamingFields::Volume {
        producer: layer_id,
        name: b"cryptroot".to_vec(),
        role: None,
    };
    let volume_id = derive_id(&volume).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, dev_id);
    // Only the signature is placed. The layer and the volume carry no
    // extent — their kinds may not — so the closure is the only thing
    // that can reach them.
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
        vec![dev, luks, layer, volume],
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
            Edge {
                kind: EdgeKind::Production,
                source: layer_id,
                target: volume_id,
            },
        ],
        facts,
    )
    .expect("assembles");

    let Planned { simulated, .. } = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: dev_id,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("the rebuild must stand, not refuse: no survivor names a casualty");

    let survivors: Vec<NodeId> = simulated
        .topology()
        .entries()
        .iter()
        .map(super::NodeEntry::id)
        .collect();
    assert_eq!(
        survivors,
        vec![dev_id],
        "the wiped device remains; signature, layer and volume are all gone"
    );
    assert!(
        !survivors.contains(&volume_id),
        "the volume is reachable only through its producer referent"
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

// ---------------------------------------------------------------------
// ADR-0036: the scheme's own regions, and located occupancy (WP-060
// increment 10, on issue #319).
// ---------------------------------------------------------------------

const GIB: u64 = 1 << 30;

/// A device carrying exactly one table view in `role`, no partitions,
/// and a self-extent of `total`.
fn scheme_host(serial: &[u8], role: TableRole, total: u64) -> (TopologySnapshot, NodeId, NodeId) {
    let host = NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: total,
    };
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role,
    };
    let table_id = derive_id(&table).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(host_id, TransportClass::Sata);
    facts.extents.insert(
        host_id,
        HostRange {
            host: host_id,
            start: 0,
            length: total,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host, table],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: host_id,
            target: table_id,
        }],
        facts,
    )
    .expect("assembles");
    (snapshot, host_id, table_id)
}

fn free_pairs(snapshot: &TopologySnapshot, host: NodeId) -> Vec<(u64, u64)> {
    free_extents(snapshot, host)
        .expect("computes")
        .iter()
        .map(|range| (range.start, range.length))
        .collect()
}

/// A device with one table view and one partition, the partition's
/// extent supplied by the caller so each occupancy ground is built by
/// varying exactly one thing.
fn occupancy_host(
    declared_start: u64,
    placed: Option<HostRange>,
) -> (TopologySnapshot, NodeId, NodeId) {
    let host = device(b"OCCUPANT");
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: TableRole::Mbr,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: declared_start,
    };
    let part_id = derive_id(&part).expect("derivable");
    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    if let Some(range) = placed {
        facts.extents.insert(part_id, range);
    }
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host, table, part],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
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
    .expect("assembles");
    (snapshot, host_id, part_id)
}

// Requirements: PLAN-001, PART-009, INV-004
//   ADR-0036's reservation table, asserted per role on identical
//   geometry: every recognized scheme withholds its head, GPT and the
//   hybrid MBR view additionally withhold a tail, and MBR and APM
//   withhold none — a tail bound there would reserve bytes no structure
//   claims. The filed #319 defect is gone: no accepted placement starts
//   below the withheld head at any size that previously reached it.
// Evidence: the_schemes_own_regions_are_withheld_at_both_ends
#[test]
fn the_schemes_own_regions_are_withheld_at_both_ends() {
    for (role, ceiling) in [
        (TableRole::Gpt, GIB - DEFAULT_ALIGNMENT),
        (TableRole::HybridMbr, GIB - DEFAULT_ALIGNMENT),
        (TableRole::Mbr, GIB),
        (TableRole::Apm, GIB),
    ] {
        let (snapshot, host, _) = scheme_host(b"SCHEME", role.clone(), GIB);
        assert_eq!(
            free_pairs(&snapshot, host),
            vec![(DEFAULT_ALIGNMENT, ceiling - DEFAULT_ALIGNMENT)],
            "role {role:?} withholds the wrong regions"
        );
    }

    // The filed defect on the delivered fixture: a create of exactly
    // the default alignment used to land at offset 0, over the
    // protective MBR and the GPT header, recorded Aligned.
    let (snapshot, host, _, _) = solver_fixture();
    let solved = place_create(&snapshot, host, DEFAULT_ALIGNMENT).expect("places");
    assert!(
        solved.placed.start >= DEFAULT_ALIGNMENT,
        "a create was placed in the withheld head at {}",
        solved.placed.start
    );

    // Every size that could reach a sub-1 MiB start.
    for size in [1_u64, 512, 4096, DEFAULT_ALIGNMENT - 1, DEFAULT_ALIGNMENT] {
        if let Ok(solved) = place_create(&snapshot, host, size) {
            assert!(
                solved.placed.start >= DEFAULT_ALIGNMENT,
                "size {size} was placed at {}",
                solved.placed.start
            );
        }
    }

    // The legacy MBR host's 32,256-byte create at offset 0 — recorded
    // Coincident before ADR-0036, affirmatively conforming, on the boot
    // sector — now has no lawful placement, while the host's last byte
    // stays reachable because MBR withholds no tail.
    let (legacy, legacy_host, _) = legacy_fixture();
    assert!(
        place_create(&legacy, legacy_host, LEGACY_START).is_err(),
        "the boot-sector create must no longer conform"
    );
    assert_eq!(
        free_pairs(&legacy, legacy_host).last().map(|(s, l)| s + l),
        Some(GIB),
        "an MBR host keeps its last byte reachable"
    );
}

// Requirements: PLAN-001, INV-004
//   A scheme the build cannot name yields no derivable bound — its
//   metadata may sit anywhere — so the derivation is not presented at
//   all, whether or not the facts locate the table. Granting a located
//   unrecognized view full accounting instead reproduces the filed
//   defect exactly.
// Evidence: an_unrecognized_scheme_refuses_whether_or_not_it_is_located
#[test]
fn an_unrecognized_scheme_refuses_whether_or_not_it_is_located() {
    let raw = b"weird-scheme".to_vec();
    for locate in [false, true] {
        let (base, host, table) =
            scheme_host(b"UNREC", TableRole::Unrecognized { raw: raw.clone() }, GIB);
        let mut facts = base.facts().clone();
        if locate {
            facts.extents.insert(
                table,
                HostRange {
                    host,
                    start: 0,
                    length: DEFAULT_ALIGNMENT,
                },
            );
        }
        let snapshot = TopologySnapshot::assemble(
            SnapshotKind::Captured,
            false,
            vec![
                NamingFields::PhysicalDevice {
                    serial: Some(b"UNREC".to_vec()),
                    wwn: None,
                    total_bytes: GIB,
                },
                NamingFields::PartitionTable {
                    parent: host,
                    role: TableRole::Unrecognized { raw: raw.clone() },
                },
            ],
            vec![Edge {
                kind: EdgeKind::Containment,
                source: host,
                target: table,
            }],
            facts,
        )
        .expect("assembles");

        match free_extents(&snapshot, host) {
            Err(SolveRefusal::UnrecognizedTableScheme {
                host: refused,
                view,
                raw: carried,
            }) => {
                assert_eq!(refused, host);
                assert_eq!(view, table);
                assert_eq!(carried, raw, "the raw discriminant travels verbatim");
            }
            other => panic!("located={locate}: expected a scheme refusal, got {other:?}"),
        }
        assert!(
            place_create(&snapshot, host, DEFAULT_ALIGNMENT).is_err(),
            "located={locate}: no placement may exist under an unnamed scheme"
        );
    }
}

// Requirements: PLAN-001, INV-004
//   Located-ness is not key presence: the guard's notion of accounted
//   is the subtraction's own, so each ground names what the facts carry
//   beside the offset the occupant's own hashed name declares. The
//   ground is a function of the located range alone and every arm is
//   asserted there, empty range and other-host range included: the body
//   boundary refuses a zero-length extent before a snapshot can carry one
//   (ADR-0041) and, under ADR-0037's frame rule, a partition framed
//   anywhere but the root its own name leads to; the solver's reading of
//   a range must not depend on which shapes a snapshot lets through.
//   Through a snapshot, every ground a valid body can carry is then
//   asserted end to end.
// Evidence: an_unaccounted_occupant_refuses_naming_what_the_facts_carry_instead
#[test]
fn an_unaccounted_occupant_refuses_naming_what_the_facts_carry_instead() {
    let declared = 500 * DEFAULT_ALIGNMENT;
    let host_id = derive_id(&device(b"OCCUPANT")).expect("derivable");
    let elsewhere = derive_id(&device(b"ELSEWHERE")).expect("derivable");

    // The ground, read off the range alone.
    for (located, expected) in [
        (None, Some(OccupancyGround::NoRange)),
        (
            Some(HostRange {
                host: elsewhere,
                start: declared,
                length: DEFAULT_ALIGNMENT,
            }),
            Some(OccupancyGround::RangeOnAnotherHost { host: elsewhere }),
        ),
        (
            Some(HostRange {
                host: host_id,
                start: declared,
                length: 0,
            }),
            Some(OccupancyGround::RangeIsEmpty),
        ),
        (
            Some(HostRange {
                host: host_id,
                start: 400 << 20,
                length: DEFAULT_ALIGNMENT,
            }),
            Some(OccupancyGround::RangeStartsElsewhere { start: 400 << 20 }),
        ),
        (
            Some(HostRange {
                host: host_id,
                start: declared,
                length: DEFAULT_ALIGNMENT,
            }),
            None,
        ),
    ] {
        assert_eq!(
            crate::solve::occupancy_ground(located, host_id, declared),
            expected,
            "{located:?}"
        );
    }

    // Through a snapshot: the grounds a body framed as ADR-0037 requires
    // can carry.
    let cases = [
        (None, OccupancyGround::NoRange),
        (
            Some(HostRange {
                host: host_id,
                start: 400 << 20,
                length: DEFAULT_ALIGNMENT,
            }),
            OccupancyGround::RangeStartsElsewhere { start: 400 << 20 },
        ),
    ];

    for (index, (placed, expected)) in cases.into_iter().enumerate() {
        let (snapshot, host, part) = occupancy_host(declared, placed);
        match free_extents(&snapshot, host) {
            Err(SolveRefusal::UnaccountedOccupant {
                host: refused,
                occupant,
                declared_start,
                ground,
            }) => {
                assert_eq!(refused, host);
                assert_eq!(occupant, part);
                assert_eq!(declared_start, declared, "the hashed name's own offset");
                assert_eq!(ground, expected, "case {index}");
            }
            other => panic!("case {index}: expected an occupancy refusal, got {other:?}"),
        }
    }

    // The control: the same shape, correctly located, computes.
    let (snapshot, host, _) = occupancy_host(
        declared,
        Some(HostRange {
            host: host_id,
            start: declared,
            length: DEFAULT_ALIGNMENT,
        }),
    );
    assert!(free_extents(&snapshot, host).is_ok());
}

// Requirements: PLAN-001, INV-004
//   An occupant located on this host under a table view this host does
//   not carry: no scheme of this host's accounts for it. This is the
//   arm that closes the no-table-node hole positively, rather than by
//   refusing on absence. Asserted on the ground helper, where it can be
//   measured regardless of which shapes a snapshot lets through: under
//   ADR-0037's frame rule a partition's extent is framed on the root its
//   own table's name leads to, so a body carrying this shape is refused
//   at assembly once that rule is enforced, and the solver's own defence
//   must not depend on it. The other side of the arm is asserted too: a
//   partition under a foreign table that is located elsewhere, or
//   nowhere, is no occupant of this host at all.
// Evidence: an_occupant_under_a_table_this_host_does_not_carry_refuses
#[test]
fn an_occupant_under_a_table_this_host_does_not_carry_refuses() {
    let host_id = derive_id(&device(b"HOST-A")).expect("derivable");
    let other_id = derive_id(&device(b"HOST-B")).expect("derivable");
    let own_table = derive_id(&NamingFields::PartitionTable {
        parent: host_id,
        role: TableRole::Mbr,
    })
    .expect("derivable");
    let foreign_table = derive_id(&NamingFields::PartitionTable {
        parent: other_id,
        role: TableRole::Mbr,
    })
    .expect("derivable");
    let declared = 300 << 20;
    let on_this_host = HostRange {
        host: host_id,
        start: declared,
        length: DEFAULT_ALIGNMENT,
    };
    let elsewhere = HostRange {
        host: other_id,
        start: declared,
        length: DEFAULT_ALIGNMENT,
    };

    // Located on this host, under a table this host does not carry.
    assert_eq!(
        crate::solve::occupant_ground(
            Some(on_this_host),
            host_id,
            declared,
            foreign_table,
            &[own_table]
        ),
        Some(OccupancyGround::TableIsNotThisHosts {
            named_table: foreign_table
        })
    );
    // Under a foreign table but located elsewhere, or nowhere: another
    // host's matter, not this host's occupant.
    for located in [Some(elsewhere), None] {
        assert_eq!(
            crate::solve::occupant_ground(located, host_id, declared, foreign_table, &[own_table]),
            None,
            "{located:?}"
        );
    }
    // Under this host's own table, the arm defers to the range's ground.
    assert_eq!(
        crate::solve::occupant_ground(
            Some(on_this_host),
            host_id,
            declared,
            own_table,
            &[own_table]
        ),
        None
    );
    assert_eq!(
        crate::solve::occupant_ground(None, host_id, declared, own_table, &[own_table]),
        Some(OccupancyGround::NoRange)
    );
}

// Requirements: PLAN-001, INV-004
//   The roster is read from the authenticated naming fields, never from
//   containment edges: an edge rides in no node's address preimage, so
//   an edge-sourced roster would shrink silently when one is omitted.
//   Held as a property, not as a promise.
// Evidence: the_guard_stands_with_every_containment_edge_removed
#[test]
fn the_guard_stands_with_every_containment_edge_removed() {
    let (with_edges, host, _, _) = solver_fixture();
    let expected = free_pairs(&with_edges, host);

    let host_fields = device(b"SLV-HOST");
    let host_id = derive_id(&host_fields).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let aligned = NamingFields::Partition {
        parent_table: table_id,
        start_offset: DEFAULT_ALIGNMENT,
    };
    let misaligned = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 100 * DEFAULT_ALIGNMENT + 512,
    };
    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    facts.extents.insert(
        derive_id(&aligned).expect("derivable"),
        HostRange {
            host: host_id,
            start: DEFAULT_ALIGNMENT,
            length: 64 * DEFAULT_ALIGNMENT,
        },
    );
    facts.extents.insert(
        derive_id(&misaligned).expect("derivable"),
        HostRange {
            host: host_id,
            start: 100 * DEFAULT_ALIGNMENT + 512,
            length: 32 * DEFAULT_ALIGNMENT,
        },
    );
    let without_edges = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host_fields, table, aligned, misaligned],
        vec![],
        facts,
    )
    .expect("assembles");

    assert_eq!(
        free_pairs(&without_edges, host_id),
        expected,
        "the free list must not depend on containment edges"
    );
}

// Requirements: PLAN-001, INV-004
//   The guard reads the table NODE, never the table STATE stamp.
//   WP-L100 increment 3 is chartered to emit a body with no
//   table_states entry on any path, and a stamp-conditioned guard would
//   refuse a chartered deliverable.
// Evidence: the_guard_is_blind_to_table_state
#[test]
fn the_guard_is_blind_to_table_state() {
    let (base, host, table) = scheme_host(b"BLIND", TableRole::Gpt, GIB);
    let expected = free_pairs(&base, host);

    for state in [
        None,
        Some(TableState::Present {
            checksum: canonical::hash(&Value::Text("blind".into())).expect("hashable"),
        }),
        Some(TableState::Absent),
    ] {
        let mut facts = base.facts().clone();
        facts.table_states.remove(&host);
        if let Some(state) = state.clone() {
            facts.table_states.insert(host, state);
        }
        let snapshot = TopologySnapshot::assemble(
            SnapshotKind::Captured,
            false,
            vec![
                NamingFields::PhysicalDevice {
                    serial: Some(b"BLIND".to_vec()),
                    wwn: None,
                    total_bytes: GIB,
                },
                NamingFields::PartitionTable {
                    parent: host,
                    role: TableRole::Gpt,
                },
            ],
            vec![Edge {
                kind: EdgeKind::Containment,
                source: host,
                target: table,
            }],
            facts,
        )
        .expect("assembles");
        assert_eq!(
            free_pairs(&snapshot, host),
            expected,
            "table state {state:?} changed the derivation"
        );
    }
}

// Requirements: PLAN-001
//   A host extent reaching past the size the host's own hashed name
//   declares makes the arithmetic's outer bound wrong. Unguarded at
//   HEAD: a 1 GiB device carrying a 2 GiB self-extent placed a 1.5 GiB
//   partition at offset 0, recorded Aligned — Section 11.2's "extents
//   remain inside the bound device", violated by the solver itself.
// Evidence: a_host_extent_past_its_own_declared_size_refuses
#[test]
fn a_host_extent_past_its_own_declared_size_refuses() {
    let host = device(b"OVERRUN");
    let host_id = derive_id(&host).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(host_id, TransportClass::Sata);
    facts.extents.insert(
        host_id,
        HostRange {
            host: host_id,
            start: 0,
            length: 2 * GIB,
        },
    );
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![host], vec![], facts)
            .expect("assembles");

    match free_extents(&snapshot, host_id) {
        Err(SolveRefusal::HostExtentExceedsDevice {
            host,
            extent_end,
            total_bytes,
        }) => {
            assert_eq!(host, host_id);
            assert_eq!(extent_end, 2 * GIB);
            assert_eq!(total_bytes, GIB);
        }
        other => panic!("expected a host-overrun refusal, got {other:?}"),
    }
    assert!(place_create(&snapshot, host_id, 1536 << 20).is_err());
}

// Requirements: PLAN-001
//   A child range leaving its host's extent is surfaced rather than
//   absorbed. Only the upper bound is checked: a lower bound would
//   refuse the partition-anchored shapes issue #333 leaves undecided.
// Evidence: a_child_extent_leaving_the_host_refuses
#[test]
fn a_child_extent_leaving_the_host_refuses() {
    let (snapshot, host, part) = occupancy_host(
        900 << 20,
        Some(HostRange {
            host: derive_id(&device(b"OCCUPANT")).expect("derivable"),
            start: 900 << 20,
            length: 200 << 20,
        }),
    );
    match free_extents(&snapshot, host) {
        Err(SolveRefusal::ChildExtentOutsideHost {
            host: refused,
            node,
            start,
            length,
        }) => {
            assert_eq!(refused, host);
            assert_eq!(node, part);
            assert_eq!(start, 900 << 20);
            assert_eq!(length, 200 << 20);
        }
        other => panic!("expected a child-overrun refusal, got {other:?}"),
    }
}

// Requirements: PLAN-001, INV-004
//   A host declaring no table view reserves nothing and does NOT refuse
//   on that ground — refusing on an absent node manufactures a refusal
//   from absence, the mirror of manufacturing free space from it. The
//   recorded residual, asserted so it cannot drift into a silent fix.
// Evidence: a_host_with_no_table_view_reserves_nothing
#[test]
fn a_host_with_no_table_view_reserves_nothing() {
    let host = device(b"NO-TABLE");
    let host_id = derive_id(&host).expect("derivable");
    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![host], vec![], facts)
            .expect("assembles");

    assert_eq!(free_pairs(&snapshot, host_id), vec![(0, GIB)]);
    let reserved = reserved_regions(&snapshot, host_id).expect("computes");
    assert_eq!((reserved.head, reserved.tail), (0, 0));
    let solved = place_create(&snapshot, host_id, DEFAULT_ALIGNMENT).expect("places");
    assert_eq!(solved.placed.start, 0);
}

// Requirements: PLAN-001, PART-009
//   ADR-0023's coincident-edge rule stays complete once a tail is
//   withheld. On a host whose extent is not a whole multiple of the
//   default, the reserved ceiling is unaligned — without the new edge a
//   create filling the last free range would refuse, or align down and
//   mint exactly the unusable sliver 12.1.0 rejected.
// Evidence: the_reserved_tail_is_a_coincident_edge_on_an_odd_sized_host
#[test]
fn the_reserved_tail_is_a_coincident_edge_on_an_odd_sized_host() {
    let odd = GIB + 512;
    let (snapshot, host, table) = scheme_host(b"ODD-GPT", TableRole::Gpt, odd);
    let free = free_pairs(&snapshot, host);
    assert_eq!(
        free,
        vec![(
            DEFAULT_ALIGNMENT,
            odd - DEFAULT_ALIGNMENT - DEFAULT_ALIGNMENT
        )],
        "the ceiling sits one reservation below an unaligned host end"
    );

    let (start, length) = free[0];
    let solved = place_create(&snapshot, host, length).expect("fills to the ceiling");
    assert_eq!(solved.placed.start, start);
    assert_eq!(
        solved.end_placement,
        BoundaryPlacement::Coincident {
            edge: StructuralEdge::ReservedTableRegion { table }
        },
        "filling to the reserved ceiling is coincident with the scheme's region"
    );
}

// Requirements: PLAN-001, PART-009
//   A GPT tail stops a grow short of the backup structures: filling to
//   the physical end of a GPT disk overwrites the backup header. The
//   identical device under MBR, which withholds no tail, still reaches
//   its last byte.
// Evidence: a_gpt_tail_stops_a_grow_short_of_the_backup
#[test]
fn a_gpt_tail_stops_a_grow_short_of_the_backup() {
    for (role, reaches_the_end) in [(TableRole::Gpt, false), (TableRole::Mbr, true)] {
        let host = NamingFields::PhysicalDevice {
            serial: Some(b"GROW-TAIL".to_vec()),
            wwn: None,
            total_bytes: GIB,
        };
        let host_id = derive_id(&host).expect("derivable");
        let table = NamingFields::PartitionTable {
            parent: host_id,
            role: role.clone(),
        };
        let table_id = derive_id(&table).expect("derivable");
        let part = NamingFields::Partition {
            parent_table: table_id,
            start_offset: DEFAULT_ALIGNMENT,
        };
        let part_id = derive_id(&part).expect("derivable");
        let mut facts = Facts::default();
        facts.transports.insert(host_id, TransportClass::Sata);
        facts.extents.insert(
            host_id,
            HostRange {
                host: host_id,
                start: 0,
                length: GIB,
            },
        );
        facts.extents.insert(
            part_id,
            HostRange {
                host: host_id,
                start: DEFAULT_ALIGNMENT,
                length: 64 * DEFAULT_ALIGNMENT,
            },
        );
        let snapshot = TopologySnapshot::assemble(
            SnapshotKind::Captured,
            false,
            vec![host, table, part],
            vec![
                Edge {
                    kind: EdgeKind::Containment,
                    source: host_id,
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
        .expect("assembles");

        // Grow to the device's physical end.
        let to_physical_end = GIB - DEFAULT_ALIGNMENT;
        let outcome = grow_extension(&snapshot, part_id, to_physical_end);
        if reaches_the_end {
            assert!(
                outcome.is_ok(),
                "{role:?} withholds no tail, so the last byte stays reachable"
            );
        } else {
            match outcome {
                Err(SolveRefusal::NoAdjacentFreeSpace {
                    target,
                    needed,
                    available,
                }) => {
                    assert_eq!(target, part_id);
                    assert_eq!(
                        needed - available,
                        DEFAULT_ALIGNMENT,
                        "the shortfall is exactly the withheld tail"
                    );
                }
                other => panic!("{role:?}: expected a tail-short refusal, got {other:?}"),
            }
            // Growing to the ceiling instead succeeds.
            assert!(
                grow_extension(&snapshot, part_id, to_physical_end - DEFAULT_ALIGNMENT).is_ok()
            );
        }
    }
}

// Requirements: PLAN-001, INV-004
//   A conflicting entry records a view of its table, so its role widens
//   the reservation — a hybrid device's GPT view withholds a tail its
//   MBR table alone would not. And it is never an occupant: it carries
//   no length, so no bound over it is computable, and ADR-0024's repair
//   family stays reachable on a repairable device.
// Evidence: a_conflicting_views_role_widens_the_reservation
#[test]
fn a_conflicting_views_role_widens_the_reservation() {
    let host = device(b"HYBRID");
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: TableRole::Mbr,
    };
    let table_id = derive_id(&table).expect("derivable");
    let conflicting = NamingFields::ConflictingTableEntry {
        table: table_id,
        view_role: TableRole::Gpt,
        entry_start: 2048,
    };
    let conflicting_id = derive_id(&conflicting).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host, table, conflicting],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: conflicting_id,
            },
        ],
        facts,
    )
    .expect("assembles");

    let reserved = reserved_regions(&snapshot, host_id).expect("computes");
    assert_eq!(
        (reserved.head, reserved.tail),
        (DEFAULT_ALIGNMENT, DEFAULT_ALIGNMENT),
        "the GPT view's tail widens an MBR table's reservation"
    );
    // The extent-less conflicting entry is not an occupant: the
    // derivation computes rather than refusing.
    assert_eq!(
        free_pairs(&snapshot, host_id),
        vec![(DEFAULT_ALIGNMENT, GIB - 2 * DEFAULT_ALIGNMENT)]
    );
}

// Requirements: PLAN-001, INV-004
//   The ordering rule, on a body carrying two live grounds at once: a
//   scheme this build cannot name AND an unlocated partition under it.
//   Without both present the ordering is unobservable.
// Evidence: the_refusal_is_the_scheme_before_the_occupant
#[test]
fn the_refusal_is_the_scheme_before_the_occupant() {
    let host = device(b"BOTH-GROUNDS");
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: TableRole::Unrecognized {
            raw: b"both".to_vec(),
        },
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 200 << 20,
    };
    let part_id = derive_id(&part).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    // The partition is deliberately left unlocated: both grounds live.
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![host, table, part],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
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
    .expect("assembles");

    assert!(
        matches!(
            free_extents(&snapshot, host_id),
            Err(SolveRefusal::UnrecognizedTableScheme { .. })
        ),
        "the scheme refusal must precede the occupancy refusal"
    );
}

// Requirements: PLAN-001, INV-004
//   Hosted layers are NOT required-located occupants, and the exclusion
//   is what keeps this decision independent of issue #333: the
//   delivered created_capture fixture anchors its file system on the
//   partition, and requiring hosted layers located refuses it under
//   #333's rival reading. Both a device-host call and a nested
//   partition-host call behave exactly as they did before ADR-0036.
// Evidence: a_partition_scoped_layer_is_not_the_devices_occupant
#[test]
fn a_partition_scoped_layer_is_not_the_devices_occupant() {
    let snapshot = created_capture(true);
    let device_id = derive_id(&device(b"SLV-HOST")).expect("derivable");
    // The device-scoped call computes: the partition-anchored file
    // system never enters the roster.
    let device_free = free_extents(&snapshot, device_id);
    assert!(
        device_free.is_ok(),
        "a partition-anchored layer must not refuse the device's derivation: {device_free:?}"
    );
}

// Requirements: PLAN-001, PLAN-008
//   Issue #341: `impossibility`'s `unreachable!` rests on the premise
//   that every operation reaching reversal emission is one the path can
//   plan. That premise was a property of `plan`'s statement order and
//   did not hold in `plan_set`, whose statement loop ran before its
//   simulatability check — so a single-request set carrying an unsized
//   create, on a target that clears the capability gate, aborted the
//   process instead of refusing. The refusal now matches the ground the
//   single-request path gives the same input.
// Evidence: an_unsized_create_in_a_set_refuses_with_the_single_paths_ground
#[test]
fn an_unsized_create_in_a_set_refuses_with_the_single_paths_ground() {
    let (snapshot, clean, _) = fixture();
    let set = PlanRequestSet {
        requests: vec![PlanRequest {
            operation: Operation::Create,
            target: clean,
        }],
        dependencies: vec![],
    };
    let from_set = plan_set(
        &set,
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("an unsized create has no geometry and cannot be planned");
    assert!(
        matches!(
            from_set,
            PlanRefusal::SimulateRefused {
                refusal: SimulateRefusal::NotRepresentable { .. }
            }
        ),
        "expected the simulation refusal, got {from_set:?}"
    );

    // The same request through the single-request path: the grounds
    // agree, which is the property the fix restores rather than a
    // coincidence of ordering.
    let from_single = plan(
        PlanRequest {
            operation: Operation::Create,
            target: clean,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("the single-request path refuses the same input");
    assert_eq!(
        format!("{from_set:?}"),
        format!("{from_single:?}"),
        "plan_set and plan must refuse an unsized create identically"
    );
}

// Requirements: PLAN-001, PLAN-008
//   The premise `impossibility`'s `unreachable!` rests on, pinned for
//   the whole operation vocabulary rather than for the one operation
//   that was measured to break it: no single-request set over any
//   operation reaches an unplannable statement. Every operation either
//   plans or refuses with an artifact — never aborts. This is the guard
//   that would have caught issue #341 before it was filed.
// Evidence: no_request_set_reaches_an_unplannable_statement
#[test]
fn no_request_set_reaches_an_unplannable_statement() {
    let (snapshot, clean, _) = fixture();
    for operation in [
        Operation::Detect,
        Operation::Read,
        Operation::Create,
        Operation::Grow,
        Operation::Shrink,
        Operation::Move,
        Operation::Copy,
        Operation::Check,
        Operation::Repair,
        Operation::Label,
        Operation::Uuid,
        Operation::Wipe,
        Operation::Encrypt,
        Operation::Decrypt,
    ] {
        let set = PlanRequestSet {
            requests: vec![PlanRequest {
                operation,
                target: clean,
            }],
            dependencies: vec![],
        };
        // The assertion is that this returns at all. A panic here is the
        // defect; either arm of the Result is a lawful outcome.
        let outcome = plan_set(
            &set,
            &snapshot,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
            &identity(),
        );
        assert!(
            outcome.is_ok() || outcome.is_err(),
            "{operation:?} must produce an artifact, never an abort"
        );
    }
}

/// A partitioned volume: an mdraid array over one member, the array
/// producing `md0`, and `md0` carrying a partition table with one
/// partition on it. `optional_pool` puts a live ZFS pool on that
/// partition. The array and the volume carry no extent — their kinds may
/// not — so everything below the volume is framed on the volume itself,
/// which is the population ADR-0048 moved and no planner test exercised.
struct PartitionedVolume {
    snapshot: TopologySnapshot,
    dev: NodeId,
    array: NodeId,
    md0: NodeId,
    table: NodeId,
    md0p1: NodeId,
    pool: Option<NodeId>,
}

#[allow(clippy::too_many_lines)]
fn partitioned_volume(with_pool: bool) -> PartitionedVolume {
    let dev = device(b"PLN-PVOL");
    let dev_id = derive_id(&dev).expect("derivable");
    let member = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Mdraid1x,
        primary_offset: 4096,
    };
    let member_id = derive_id(&member).expect("derivable");
    let array = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"md0".to_vec()),
    };
    let array_id = derive_id(&array).expect("derivable");
    let md0 = NamingFields::Volume {
        producer: array_id,
        name: b"md0".to_vec(),
        role: None,
    };
    let md0_id = derive_id(&md0).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: md0_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let md0p1 = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let md0p1_id = derive_id(&md0p1).expect("derivable");

    let mut entries = vec![dev.clone(), member, array, md0, table, md0p1];
    let mut edges = vec![
        Edge {
            kind: EdgeKind::Containment,
            source: dev_id,
            target: member_id,
        },
        Edge {
            kind: EdgeKind::Backing,
            source: member_id,
            target: array_id,
        },
        Edge {
            kind: EdgeKind::Production,
            source: array_id,
            target: md0_id,
        },
        Edge {
            kind: EdgeKind::Containment,
            source: md0_id,
            target: table_id,
        },
        Edge {
            kind: EdgeKind::Containment,
            source: table_id,
            target: md0p1_id,
        },
    ];

    let mut facts = Facts::default();
    device_facts(&mut facts, dev_id);
    facts.extents.insert(
        member_id,
        HostRange {
            host: dev_id,
            start: 4096,
            length: 4096,
        },
    );
    // Framed on the volume, which declares no extent of its own.
    facts.extents.insert(
        table_id,
        HostRange {
            host: md0_id,
            start: 0,
            length: 1 << 20,
        },
    );
    facts.extents.insert(
        md0p1_id,
        HostRange {
            host: md0_id,
            start: 1 << 20,
            length: 1 << 29,
        },
    );

    let mut pool_id = None;
    if with_pool {
        let zfs = NamingFields::BackingSignature {
            host: md0p1_id,
            family: SignatureFamily::Zfs,
            primary_offset: 0,
        };
        let zfs_id = derive_id(&zfs).expect("derivable");
        let pool = NamingFields::Aggregate {
            technology: AggregateTechnology::Zfs,
            designator: Some(b"tank".to_vec()),
        };
        let p_id = derive_id(&pool).expect("derivable");
        facts.extents.insert(
            zfs_id,
            HostRange {
                host: md0_id,
                start: 1 << 20,
                length: 1 << 20,
            },
        );
        entries.push(zfs);
        entries.push(pool);
        edges.push(Edge {
            kind: EdgeKind::Containment,
            source: md0p1_id,
            target: zfs_id,
        });
        edges.push(Edge {
            kind: EdgeKind::Backing,
            source: zfs_id,
            target: p_id,
        });
        pool_id = Some(p_id);
    }

    let snapshot = TopologySnapshot::assemble(SnapshotKind::Captured, false, entries, edges, facts)
        .expect("a partitioned volume assembles");
    PartitionedVolume {
        snapshot,
        dev: dev_id,
        array: array_id,
        md0: md0_id,
        table: table_id,
        md0p1: md0p1_id,
        pool: pool_id,
    }
}

// Requirements: PLAN-001, PLAN-002
//   ADR-0048's named obligation, discharged. `canonical_ranges` feeds
//   `Effects.destroyed`, so an extentless target's whole-frame entry
//   reaches `destroyed_closure` — and this is the population no planner
//   test exercised when that entry was introduced. Wiping a volume that
//   carries a partition table removes the table and the partition framed
//   on it from the simulated topology, and leaves the device and the
//   array that produces the volume standing: the volume's frame is the
//   volume's own address space and nothing else.
// Evidence: wiping_an_extentless_volume_removes_what_its_frame_holds
#[test]
fn wiping_an_extentless_volume_removes_what_its_frame_holds() {
    let v = partitioned_volume(false);

    let Planned { simulated, .. } = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: v.md0,
        },
        &v.snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("an unprotected partitioned volume plans");

    let survivors: Vec<NodeId> = simulated
        .topology()
        .entries()
        .iter()
        .map(super::NodeEntry::id)
        .collect();
    assert!(
        !survivors.contains(&v.table) && !survivors.contains(&v.md0p1),
        "everything framed on the wiped volume is gone from the prediction"
    );
    assert!(
        survivors.contains(&v.dev),
        "the device the array is built from is in another frame and survives"
    );
    assert!(
        survivors.contains(&v.md0),
        "a wipe empties the volume without removing it"
    );
}

// Requirements: PLAN-001, PLAN-002
//   The same population with a live pool on the partition: the plan
//   refuses rather than predicting. Before ADR-0048 this constructed —
//   the volume declared no destroyed range, so the closure never saw it
//   destroyed and the pool was never reached — and that is the defect
//   issue #392 recorded, now covered end to end through the planner
//   rather than at the domain gate alone.
// Evidence: wiping_a_volume_that_carries_a_live_pool_refuses_the_plan
#[test]
fn wiping_a_volume_that_carries_a_live_pool_refuses_the_plan() {
    let v = partitioned_volume(true);
    assert!(v.pool.is_some(), "the fixture carries a pool");

    for target in [v.md0, v.array] {
        let refused = plan(
            PlanRequest {
                operation: Operation::Wipe,
                target,
            },
            &v.snapshot,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
            &identity(),
        )
        .expect_err("a live pool below an extentless target refuses the plan");
        let PlanRefusal::CapabilityRefused { answer } = refused else {
            panic!("the refusal carries the engine's answer: {refused:?}");
        };
        assert_eq!(answer.status(), Status::Unsupported);
        assert!(
            matches!(answer.reason(), Reason::ProtectionRefused { .. }),
            "the refusal is the protection closure's, not a capability limit"
        );
    }
}

/// The host-backed chain ADR-0049 gave the closure an arm for: a device
/// carrying an ext4 file system, an image file on it, the loop volume
/// that image produces, and a live ZFS pool on the volume. `framed_on`
/// chooses where the image's extent is declared — the file system, which
/// is the honest form, or the device beyond the file system, which is the
/// authored frame ADR-0049 pins as an open limit under issue #365.
struct HostBacked {
    snapshot: TopologySnapshot,
    dev: NodeId,
}

#[allow(clippy::too_many_lines)]
fn host_backed() -> HostBacked {
    use partman_domain::model::naming::{ExtentLocator, FileSystemKind};

    let dev = device(b"PLN-LOOP");
    let dev_id = derive_id(&dev).expect("derivable");
    let host_fs = NamingFields::FileSystem {
        host: dev_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let host_fs_id = derive_id(&host_fs).expect("derivable");
    let image = NamingFields::BackingExtent {
        host: host_fs_id,
        locator: ExtentLocator::Path {
            bytes: b"/srv/images/vm.img".to_vec(),
        },
    };
    let image_id = derive_id(&image).expect("derivable");
    let loop0 = NamingFields::Volume {
        producer: image_id,
        name: b"loop0".to_vec(),
        role: None,
    };
    let loop0_id = derive_id(&loop0).expect("derivable");
    let sig = NamingFields::BackingSignature {
        host: loop0_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let sig_id = derive_id(&sig).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, dev_id);
    facts.extents.insert(
        host_fs_id,
        HostRange {
            host: dev_id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.extents.insert(
        image_id,
        HostRange {
            host: host_fs_id,
            start: 0,
            length: 1 << 29,
        },
    );
    facts.extents.insert(
        sig_id,
        HostRange {
            host: loop0_id,
            start: 0,
            length: 1 << 20,
        },
    );

    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, host_fs, image, loop0, sig, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: host_fs_id,
            },
            Edge {
                kind: EdgeKind::HostBacking,
                source: image_id,
                target: loop0_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: loop0_id,
                target: sig_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: sig_id,
                target: pool_id,
            },
        ],
        facts,
    )
    .expect("a host-backed body assembles");

    HostBacked {
        snapshot,
        dev: dev_id,
    }
}

// Requirements: PLAN-001, PLAN-002
//   ADR-0049's named obligation, discharged. The act gave the protection
//   closure an arm into a backing extent over a relation no edge can
//   carry; this is the planner half of that population, which no test in
//   this crate built. On the honest body a wipe of the disk holding the
//   image refuses at the capability gate rather than planning — the
//   pool below the image is reached now, where before ADR-0049 the whole
//   chain was invisible and the plan constructed.
// Evidence: wiping_a_disk_that_holds_a_loop_image_refuses_the_plan
#[test]
fn wiping_a_disk_that_holds_a_loop_image_refuses_the_plan() {
    let b = host_backed();

    let refused = plan(
        PlanRequest {
            operation: Operation::Wipe,
            target: b.dev,
        },
        &b.snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect_err("a live pool below a loop image refuses the plan");
    let PlanRefusal::CapabilityRefused { answer } = refused else {
        panic!("the refusal carries the engine's answer: {refused:?}");
    };
    assert_eq!(answer.status(), Status::Unsupported);
    assert!(
        matches!(answer.reason(), Reason::ProtectionRefused { .. }),
        "the refusal is the protection closure's"
    );
}

// The two-layer disagreement this file recorded until ADR-0050 is gone,
// and deliberately not replaced by an equivalent. It was measured on a
// body whose backing extent declared a frame other than the host its own
// name carries — the only shape in which the capability gate permitted a
// wipe while `destroyed_closure` removed the image and the volume. That
// body is refused at assembly once a backing extent's frame is pinned to
// its named host, so the disagreement has no lawful witness left and a
// test asserting it could not be built.
//
// One observation from it is still true of `destroyed_closure` and is
// recorded here rather than lost: the walk propagates along naming
// referents, and `NamingFields::Aggregate` carries none, so no aggregate
// is ever reached by it. On the bodies that remain lawful the protection
// gate refuses before a prediction is produced, so nothing exhibits it —
// but it is ADR-0047's named limit in a second layer, and it belongs on
// issue #365 rather than in a test that cannot be constructed.

// ---------------------------------------------------------------------
// The move (ADR-0052): PART-005's destination vocabulary.
// ---------------------------------------------------------------------

use super::ReleasedContent;
use super::solve::move_relocation;

/// What sits on the device inside the created partition's range, named
/// within nothing but the device — the occupant a move must not carry
/// and must not overwrite, in the stale-content shape WP-020's captures
/// record.
#[derive(Clone, Copy)]
enum Stray {
    /// Nothing.
    None,
    /// A whole-disk XFS remnant: unprotected, `Clear` under the closure.
    Xfs,
    /// An mdraid member signature backing no aggregate: an orphan, and
    /// `Indeterminate` under the closure.
    OrphanSignature,
}

/// A device with a 32 MiB neighbour at [1 MiB, 33 MiB), the created
/// partition at [65 MiB, 75 MiB) with its ext4 landed at its head
/// (device-framed, as ADR-0046 requires of every containment child), free
/// space on both sides of it, and the chosen stray at [72 MiB, 73 MiB).
fn move_fixture(with_stray: Stray) -> (TopologySnapshot, NodeId, NodeId, NodeId, Option<NodeId>) {
    let host = device(b"SLV-HOST");
    let host_id = derive_id(&host).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: host_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let aligned = NamingFields::Partition {
        parent_table: table_id,
        start_offset: DEFAULT_ALIGNMENT,
    };
    let aligned_id = derive_id(&aligned).expect("derivable");
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
    let stray = match with_stray {
        Stray::None | Stray::Xfs => NamingFields::FileSystem {
            host: host_id,
            kind: partman_domain::model::naming::FileSystemKind::Xfs,
            superblock_offset: 72 * DEFAULT_ALIGNMENT,
        },
        Stray::OrphanSignature => NamingFields::BackingSignature {
            host: host_id,
            family: SignatureFamily::Mdraid1x,
            primary_offset: 72 * DEFAULT_ALIGNMENT,
        },
    };
    let stray_id = derive_id(&stray).expect("derivable");

    let mut facts = Facts::default();
    device_facts(&mut facts, host_id);
    facts.extents.insert(
        aligned_id,
        HostRange {
            host: host_id,
            start: DEFAULT_ALIGNMENT,
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
    facts.extents.insert(
        fs_id,
        HostRange {
            host: host_id,
            start: 65 * DEFAULT_ALIGNMENT,
            length: DEFAULT_ALIGNMENT,
        },
    );
    let contains = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    let mut nodes = vec![host, table, aligned, created, fs];
    let mut edges = vec![
        contains(host_id, table_id),
        contains(table_id, aligned_id),
        contains(table_id, created_id),
        contains(created_id, fs_id),
    ];
    if !matches!(with_stray, Stray::None) {
        facts.extents.insert(
            stray_id,
            HostRange {
                host: host_id,
                start: 72 * DEFAULT_ALIGNMENT,
                length: DEFAULT_ALIGNMENT,
            },
        );
        nodes.push(stray);
        edges.push(contains(host_id, stray_id));
    }
    let snapshot = TopologySnapshot::assemble(SnapshotKind::Captured, false, nodes, edges, facts)
        .expect("assembles");
    let stray_id = (!matches!(with_stray, Stray::None)).then_some(stray_id);
    (snapshot, host_id, created_id, fs_id, stray_id)
}

fn plan_move(
    snapshot: &TopologySnapshot,
    target: NodeId,
    new_start: u64,
) -> Result<Planned, PlanRefusal> {
    plan_sized(
        SizedRequest::Move { target, new_start },
        snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
}

// Requirements: PART-005, PLAN-002, PLAN-008
//   The disjoint move end to end (ADR-0052 D1, D2, D5): a partition with
//   its file system moves to free space. The step declares the whole
//   source destroyed and the whole destination consumed; the simulation
//   re-derives the partition's address at the destination and its file
//   system's with it — a moved partition renames (ADR-0019), so its
//   content renames — with the file system's device-framed extent
//   translated by the move's offset; the old addresses are gone; and the
//   reversal draft spells its target as the forward step's output, which
//   `consumed = [D]` resolves under the unchanged step-output contract,
//   with the released source as its truthfulness precondition. Nothing
//   is released, so nothing is enumerated. The family's ADR-0025 and
//   PLAN-005 declarations ride along: data-moving, not
//   irreversible-after-start, checkpoint-cancellable.
// Evidence: a_move_carries_the_target_and_what_it_names
#[test]
#[allow(clippy::too_many_lines)]
fn a_move_carries_the_target_and_what_it_names() {
    let (snapshot, host, created, fs, _) = move_fixture(Stray::None);
    let planned = plan_move(&snapshot, created, 80 * DEFAULT_ALIGNMENT).expect("plans");
    let step = &planned.plan.steps()[0];
    let source = HostRange {
        host,
        start: 65 * DEFAULT_ALIGNMENT,
        length: 10 * DEFAULT_ALIGNMENT,
    };
    let destination = HostRange {
        host,
        start: 80 * DEFAULT_ALIGNMENT,
        length: 10 * DEFAULT_ALIGNMENT,
    };
    assert_eq!(
        step.ranges().destroyed,
        vec![source],
        "D2: the whole source destroyed"
    );
    assert_eq!(
        step.ranges().consumed,
        vec![destination],
        "D2: the whole destination consumed"
    );
    assert_eq!(step.risk().severity, Severity::DataMoving);
    assert!(
        !step.risk().flags.irreversible_after_start,
        "ADR-0025's unflagged fixture"
    );
    assert_eq!(step.cancellation(), Cancellation::CheckpointCancellable);

    // The simulation: renamed, translated, old addresses gone.
    let table_id = derive_id(&NamingFields::PartitionTable {
        parent: host,
        role: TableRole::Gpt,
    })
    .expect("derivable");
    let moved = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 80 * DEFAULT_ALIGNMENT,
    };
    let moved_id = derive_id(&moved).expect("derivable");
    let moved_fs = NamingFields::FileSystem {
        host: moved_id,
        kind: partman_domain::model::naming::FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let moved_fs_id = derive_id(&moved_fs).expect("derivable");
    let after = planned.simulated.facts();
    assert_eq!(
        after.extents.get(&moved_id),
        Some(&destination),
        "the partition sits at D"
    );
    assert_eq!(
        after.extents.get(&moved_fs_id),
        Some(&HostRange {
            host,
            start: 80 * DEFAULT_ALIGNMENT,
            length: DEFAULT_ALIGNMENT,
        }),
        "the file system renamed with its host and its extent translated by the offset"
    );
    assert!(
        !after.extents.contains_key(&created),
        "the old partition address is gone"
    );
    assert!(
        !after.extents.contains_key(&fs),
        "the old file-system address is gone"
    );
    assert!(
        planned
            .simulated
            .topology()
            .entries()
            .iter()
            .any(|entry| entry.id() == moved_fs_id),
        "the renamed file system is a node of the simulated topology"
    );
    assert!(
        planned
            .simulated
            .topology()
            .edges()
            .iter()
            .any(|edge| edge.source == moved_id && edge.target == moved_fs_id),
        "its containment edge followed the rename"
    );

    // Nothing released, nothing enumerated; the end is on the default.
    assert!(
        planned.consequences.is_empty(),
        "{:?}",
        planned.consequences
    );

    // The reversal: move back, target spelled as the step's output.
    let EmittedReversal::Draft(draft) = &planned.reversal else {
        panic!("a move emits a draft, not an impossibility statement");
    };
    let back = &draft.steps()[0];
    assert_eq!(back.target, DraftTarget::StepOutput(0));
    assert_eq!(
        back.ranges.consumed,
        vec![source],
        "the reversal consumes S"
    );
    assert_eq!(
        back.ranges.destroyed,
        vec![destination],
        "the reversal destroys D"
    );
    assert_eq!(
        back.preconditions,
        vec![DraftPrecondition::Carried(Precondition::RegionUnoccupied {
            region: source
        })],
        "disjoint: the whole source must be empty for the move back to be truthful"
    );
    assert!(
        matches!(planned.plan.reversal(), Some(ReversalLinkage::Draft { .. })),
        "the body carries the draft by ID and hash"
    );
}

// Requirements: PART-005, PLAN-002, PLAN-008
//   The overlapping move — the mode PART-005 mandates as journaled chunk
//   copy — is the trilemma's case (ADR-0052 §Context). The declaration is
//   the same conservative shape, so `consumed = [D]` intersects the
//   target's own pre-move extent exactly as the amended ADR-0018 admits;
//   the copy mode is a derivation over the two ranges, stored nowhere;
//   the released range is one end of the source; and the reversal's
//   precondition asks only that end to be empty. Moving down into a
//   neighbour refuses as not free — the room is free space plus the
//   source, and nothing else.
// Evidence: an_overlapping_move_declares_the_whole_source_and_destination
#[test]
fn an_overlapping_move_declares_the_whole_source_and_destination() {
    let (snapshot, host, created, _, _) = move_fixture(Stray::None);
    let solved = move_relocation(&snapshot, created, 70 * DEFAULT_ALIGNMENT).expect("solves");
    assert!(solved.overlaps(), "D4: the mode is derived from the ranges");
    assert_eq!(
        solved.released(),
        HostRange {
            host,
            start: 65 * DEFAULT_ALIGNMENT,
            length: 5 * DEFAULT_ALIGNMENT,
        },
        "moving up releases the head of the source"
    );
    let disjoint = move_relocation(&snapshot, created, 80 * DEFAULT_ALIGNMENT).expect("solves");
    assert!(!disjoint.overlaps());
    assert_eq!(
        disjoint.released(),
        disjoint.source,
        "disjoint: the whole source is released"
    );

    let planned = plan_move(&snapshot, created, 70 * DEFAULT_ALIGNMENT).expect("plans");
    let step = &planned.plan.steps()[0];
    let source = solved.source;
    let destination = solved.destination;
    assert_eq!(step.ranges().destroyed, vec![source]);
    assert_eq!(step.ranges().consumed, vec![destination]);
    assert!(
        solved.overlaps(),
        "the consumed range intersects the target's own extent: ADR-0052's exception in use"
    );
    let EmittedReversal::Draft(draft) = &planned.reversal else {
        panic!("a move emits a draft");
    };
    assert_eq!(
        draft.steps()[0].preconditions,
        vec![DraftPrecondition::Carried(Precondition::RegionUnoccupied {
            region: solved.released()
        })],
        "only the released end need be empty"
    );

    // Down, overlapping, over the partition's own file system at its
    // head: the round's paradigm case. The literal "no other node" clause
    // refused this; the scoped clause admits it, and the tail is what is
    // released.
    let down = move_relocation(&snapshot, created, 60 * DEFAULT_ALIGNMENT).expect("solves");
    assert!(down.overlaps());
    assert_eq!(
        down.released(),
        HostRange {
            host,
            start: 70 * DEFAULT_ALIGNMENT,
            length: 5 * DEFAULT_ALIGNMENT,
        },
        "moving down releases the tail of the source"
    );
    let planned_down = plan_move(&snapshot, created, 60 * DEFAULT_ALIGNMENT)
        .expect("D3 scoped: the partition's own file system is not in the way");
    assert!(
        planned_down.consequences.is_empty(),
        "nothing not named within the target is released: {:?}",
        planned_down.consequences
    );

    // Down into the neighbour: the destination leaves the room.
    let refusal = plan_move(&snapshot, created, 30 * DEFAULT_ALIGNMENT).expect_err("refuses");
    assert!(
        matches!(
            refusal,
            PlanRefusal::SolveRefused {
                refusal: SolveRefusal::DestinationNotFree { target, .. }
            } if target == created
        ),
        "{refusal:?}"
    );
}

// Requirements: PART-005, PLAN-002
//   ADR-0052 D3's scoped clause and D6's enumeration, on the occupant
//   they exist for: a device-hosted signature inside the partition's
//   source range, named within nothing the move carries. Into the
//   overlap it refuses typed — the destination would overwrite content
//   the move does not carry. Out of the overlap, into free space, the
//   move plans, and the signature — released with the source range,
//   release being destruction (ADR-0018) — is enumerated as a typed
//   consequence whose sentence states the loss, and is gone from the
//   simulated topology. The literal "no other node" form of the clause
//   would have refused the partition's own file system too; the scoped
//   form admits it, and this is that admission measured.
// Evidence: a_destination_over_content_the_move_does_not_carry_refuses_and_a_release_is_stated
#[test]
fn a_destination_over_content_the_move_does_not_carry_refuses_and_a_release_is_stated() {
    let (snapshot, _, created, _, stray) = move_fixture(Stray::Xfs);
    let stray = stray.expect("the fixture carries the stray file system");

    // Into the overlap: [70, 80) covers the stray at [72, 73).
    let refusal = plan_move(&snapshot, created, 70 * DEFAULT_ALIGNMENT).expect_err("refuses");
    assert!(
        matches!(
            refusal,
            PlanRefusal::SolveRefused {
                refusal: SolveRefusal::DestinationOverlapsNode { target, node, .. }
            } if target == created && node == stray
        ),
        "{refusal:?}"
    );

    // Out of the overlap: the partition's own file system is carried, the
    // stray is released and stated.
    let planned = plan_move(&snapshot, created, 80 * DEFAULT_ALIGNMENT)
        .expect("plans past its own file system");
    let released: Vec<&Consequence> = planned
        .consequences
        .iter()
        .filter(|fact| matches!(fact, Consequence::RelocationReleases { .. }))
        .collect();
    assert_eq!(
        released,
        vec![&Consequence::RelocationReleases {
            target: created,
            node: stray,
            content: ReleasedContent::FileSystem(
                partman_domain::model::naming::FileSystemKind::Xfs
            ),
        }]
    );
    let sentence = released[0].to_string();
    assert!(
        sentence.contains("Xfs file system") && sentence.contains("released"),
        "{sentence}"
    );
    assert!(
        !planned.simulated.facts().extents.contains_key(&stray),
        "the released signature is gone from the prediction"
    );
}

// Requirements: PART-005, PLAN-002
//   The authenticated closure, not the solver, is what reaches content
//   inside the moved range (ADR-0052 D2, D6): with the whole source
//   declared destroyed, a step built directly through the domain's sole
//   constructor — the solver bypassed — reaches the partition's own file
//   system and the stray alike. Where the stray is an orphan signature,
//   Indeterminate under the closure, the conservative step refuses — and
//   the same orphan makes the planned move refuse at the capability gate
//   before any solver rule runs. The precise declaration, by contrast,
//   constructs and reaches the stray by no arm: trilemma (i), the ground
//   on which the conservative shape was chosen, measured.
// Evidence: the_closure_reaches_what_a_move_declares_without_the_solver
#[test]
fn the_closure_reaches_what_a_move_declares_without_the_solver() {
    let (snapshot, host, created, fs, stray) = move_fixture(Stray::Xfs);
    let stray = stray.expect("stray");
    let source = HostRange {
        host,
        start: 65 * DEFAULT_ALIGNMENT,
        length: 10 * DEFAULT_ALIGNMENT,
    };
    let destination = HostRange {
        host,
        start: 70 * DEFAULT_ALIGNMENT,
        length: 10 * DEFAULT_ALIGNMENT,
    };
    let step = PlanStep::mutating(
        &snapshot,
        created,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![destination],
            destroyed: vec![source],
        },
        vec![],
        StepRisk {
            severity: Severity::DataMoving,
            flags: StepFlags::default(),
        },
    )
    .expect("nothing here is protected");
    assert!(
        step.affected().contains(&fs),
        "the moved subtree is reached (ADR-0040)"
    );
    assert!(
        step.affected().contains(&stray),
        "the stray inside the source is reached"
    );

    // The orphan: reached, and refused — by the closure, and by the gate.
    let (orphaned, _, created_o, _, orphan) = move_fixture(Stray::OrphanSignature);
    let orphan = orphan.expect("orphan");
    let refusal = PlanStep::mutating(
        &orphaned,
        created_o,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![destination],
            destroyed: vec![source],
        },
        vec![],
        StepRisk {
            severity: Severity::DataMoving,
            flags: StepFlags::default(),
        },
    )
    .expect_err("an indeterminate node inside the declared source refuses the step");
    assert!(
        matches!(refusal, partman_domain::model::step::StepRefusal::Reached { node, .. } if node == orphan),
        "{refusal:?}"
    );
    assert!(
        matches!(
            plan_move(&orphaned, created_o, 80 * DEFAULT_ALIGNMENT).expect_err("refuses"),
            PlanRefusal::CapabilityRefused { .. }
        ),
        "the planned move refuses at the gate: the closure decides before the solver does"
    );

    // The precise declaration would not have reached the orphan at all.
    let precise = PlanStep::mutating(
        &orphaned,
        created_o,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![HostRange {
                host,
                start: 75 * DEFAULT_ALIGNMENT,
                length: 5 * DEFAULT_ALIGNMENT,
            }],
            destroyed: vec![HostRange {
                host,
                start: 65 * DEFAULT_ALIGNMENT,
                length: 5 * DEFAULT_ALIGNMENT,
            }],
        },
        vec![],
        StepRisk {
            severity: Severity::DataMoving,
            flags: StepFlags::default(),
        },
    )
    .expect(
        "constructs: the orphan lies in the overlap, which the precise declaration never names",
    );
    assert!(
        !precise.affected().contains(&orphan),
        "trilemma (i): under the precise declaration the overlap is in neither set and the \
         orphan inside it is reached by no arm"
    );
}

// Requirements: PART-005, PART-009
//   A move authors two boundaries. The start must sit on the default —
//   nothing pre-existing coincides with a moved start — and a start equal
//   to the current one is not a move; both refuse typed with the numbers
//   judged.
// Evidence: a_move_authors_an_aligned_start_and_is_a_move
#[test]
fn a_move_authors_an_aligned_start_and_is_a_move() {
    let (snapshot, _, created, _, _) = move_fixture(Stray::None);
    assert_eq!(
        move_relocation(&snapshot, created, 65 * DEFAULT_ALIGNMENT),
        Err(SolveRefusal::NotARelocation {
            target: created,
            start: 65 * DEFAULT_ALIGNMENT,
        })
    );
    assert_eq!(
        move_relocation(&snapshot, created, 80 * DEFAULT_ALIGNMENT + 512),
        Err(SolveRefusal::UnalignedAuthoredBoundary {
            target: created,
            boundary: 80 * DEFAULT_ALIGNMENT + 512,
            nearest_aligned_below: 80 * DEFAULT_ALIGNMENT,
            coincident_candidate: 0,
        })
    );
    // Off the end of the host's room: the tail region the scheme claims
    // is not free.
    let refusal =
        move_relocation(&snapshot, created, 1020 * DEFAULT_ALIGNMENT).expect_err("refuses");
    assert!(
        matches!(refusal, SolveRefusal::DestinationNotFree { .. }),
        "{refusal:?}"
    );
    // Exactly to the room's end: the destination may fill its room to
    // the boundary of the region the scheme claims, and its end — on the
    // default here — authors no consequence.
    let planned = plan_move(&snapshot, created, 1013 * DEFAULT_ALIGNMENT).expect("fills the room");
    assert!(
        planned.consequences.is_empty(),
        "{:?}",
        planned.consequences
    );
}

// Requirements: PART-005, PLAN-002
//   The tripwire's successor. `no_representable_request_relocates_bytes`
//   pinned that no sized request moved a pre-existing start and that no
//   consequence named a signature; both were true until the vocabulary
//   arrived, and ADR-0052 took the tripwire down in the same change as
//   its producer. What survives is the property in its positive form,
//   exhaustive over both vocabularies so a new request or consequence
//   kind stops this compiling: the move is the **only** sized request
//   that relocates a pre-existing start, and it relocates exactly the
//   nodes it names — every other request leaves every start and frame
//   byte-identical; and the release consequence is the **only** member
//   of the vocabulary that states a hosted-content outcome. The
//   silence of the other members is bounded here too: none of them says
//   anything about a signature, and this vocabulary carries no partition
//   type or role, so its silence is never a boot verdict.
// Evidence: only_a_move_relocates_a_pre_existing_start
#[test]
#[allow(clippy::too_many_lines)]
fn only_a_move_relocates_a_pre_existing_start() {
    let (snapshot, host, aligned, misaligned) = solver_fixture();
    let (move_snapshot, _, created, _, _) = move_fixture(Stray::None);
    let requests = [
        (
            &snapshot,
            SizedRequest::Create {
                host,
                size: 4 * DEFAULT_ALIGNMENT,
            },
        ),
        (
            &snapshot,
            SizedRequest::Grow {
                target: aligned,
                new_length: 80 * DEFAULT_ALIGNMENT,
            },
        ),
        (
            &snapshot,
            SizedRequest::Shrink {
                target: aligned,
                new_length: 32 * DEFAULT_ALIGNMENT,
            },
        ),
        (
            &move_snapshot,
            SizedRequest::Move {
                target: created,
                new_start: 80 * DEFAULT_ALIGNMENT,
            },
        ),
    ];
    for (base, request) in requests {
        // Exhaustive: a fifth `SizedRequest` fails to compile here.
        let relocates = match request {
            SizedRequest::Create { .. }
            | SizedRequest::Grow { .. }
            | SizedRequest::Shrink { .. } => None,
            SizedRequest::Move { target, new_start } => Some((target, new_start)),
        };
        let planned = plan_sized(
            request,
            base,
            &TechnologyLimits::default(),
            &RuntimeFacts::clean(),
            &identity(),
        )
        .expect("the solver backs all four");
        for (node, before) in &base.facts().extents {
            let after = planned.simulated.facts().extents.get(node);
            match relocates {
                Some((target, _))
                    if *node == target
                        || partman_domain::model::protection::names_within(
                            base.topology(),
                            *node,
                            target,
                        ) =>
                {
                    assert!(
                        after.is_none(),
                        "{request:?}: a relocated node renames — its old address is gone"
                    );
                }
                _ => {
                    let Some(after) = after else { continue };
                    assert_eq!(
                        before.start, after.start,
                        "{request:?} moved a pre-existing start it does not name"
                    );
                    assert_eq!(
                        before.host, after.host,
                        "{request:?} re-framed a pre-existing extent"
                    );
                }
            }
        }
    }

    // The vocabulary: exhaustive for the same reason.
    let vocabulary = [
        Consequence::InheritedMisalignedStart {
            target: misaligned,
            start: 100 * DEFAULT_ALIGNMENT + 512,
        },
        Consequence::CoincidentBoundary {
            target: aligned,
            boundary: 65 * DEFAULT_ALIGNMENT,
            edge: StructuralEdge::HostEnd,
        },
        Consequence::RelocationReleases {
            target: created,
            node: aligned,
            content: ReleasedContent::Other("partition"),
        },
    ];
    for fact in vocabulary {
        let states_hosted_content = match fact {
            Consequence::InheritedMisalignedStart { .. }
            | Consequence::CoincidentBoundary { .. } => false,
            Consequence::RelocationReleases { .. } => true,
        };
        assert_eq!(
            fact.to_string().contains("released"),
            states_hosted_content,
            "{fact:?}: only the release consequence states a hosted-content outcome"
        );
        assert!(
            !fact.to_string().contains("boot"),
            "{fact:?}: this vocabulary is not a boot-consequence verdict, and says nothing that reads as one"
        );
    }
}

// ---------------------------------------------------------------------
// Increment 12: the consequence text is stated into the body (slice 3p).
// ---------------------------------------------------------------------

/// The body's `consequences` set of a plan, read back through the
/// domain's own decoder — the same bytes authorization would bind.
fn body_consequences(planned: &Planned, world: &TopologySnapshot) -> Vec<String> {
    let bytes = body_bytes(&planned.plan);
    let rebuilt = OperationPlan::from_canonical_body(&bytes, world).expect("round-trips");
    rebuilt.consequences().to_vec()
}

// Requirements: PLAN-002, PART-009, PART-005
//   Section 6's consequence text is no longer planner-layer carriage:
//   the body's `consequences` set is exactly the `Display` sentences of
//   the typed facts the planner derived — measured on the release
//   fixture, where a disjoint move past a stray file system states one
//   release; on the create-to-a-structural-edge fixture, where a
//   coincident boundary is stated; and on the plain move, where the
//   vocabulary is silent and the body carries the empty set rather than
//   being silent about silence. The set is the domain's canonical one,
//   so the body hash is a function of which facts were stated and never
//   of the order the planner emitted them in; a stated fact moves it.
//   ADR-0052 D6's "delivered-in-planner, pending-in-body" is thereby
//   delivered, and ADR-0023's form holds: text, no typed carriage in
//   the hash — the typed facts stay beside the plan for the planner's
//   own consumers.
// Evidence: the_body_states_exactly_the_typed_consequences
#[test]
fn the_body_states_exactly_the_typed_consequences() {
    // A release: one typed fact, one sentence, verbatim.
    let (world, _, created, _, stray) = move_fixture(Stray::Xfs);
    let stray = stray.expect("stray");
    let planned = plan_move(&world, created, 80 * DEFAULT_ALIGNMENT).expect("plans");
    assert_eq!(planned.consequences.len(), 1, "{:?}", planned.consequences);
    let expected: Vec<String> = planned
        .consequences
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(body_consequences(&planned, &world), expected);
    assert!(
        expected[0].contains(&stray.to_string()),
        "the sentence names the released node"
    );

    // Silence: no typed fact, an empty set — present.
    let (quiet, _, created_q, _, _) = move_fixture(Stray::None);
    let silent = plan_move(&quiet, created_q, 80 * DEFAULT_ALIGNMENT).expect("plans");
    assert!(silent.consequences.is_empty());
    assert!(body_consequences(&silent, &quiet).is_empty());
    let Value::Map(map) = canonical::decode(&body_bytes(&silent.plan)).expect("decodes") else {
        panic!("body is a map");
    };
    assert_eq!(map.get("consequences"), Some(&Value::Array(vec![])));

    // A coincident boundary: the create that fills its room to the
    // scheme's reserved region states it, and the sentence in the body is
    // the typed fact's own.
    let (snapshot, host, _, _) = solver_fixture();
    let fill = plan_sized(
        SizedRequest::Create {
            host,
            size: 35 * DEFAULT_ALIGNMENT,
        },
        &snapshot,
        &TechnologyLimits::default(),
        &RuntimeFacts::clean(),
        &identity(),
    )
    .expect("plans");
    let stated = body_consequences(&fill, &snapshot);
    assert_eq!(
        stated,
        fill.consequences
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    // The hash follows the facts, not the planner's order: reordering the
    // typed facts changes nothing the body carries.
    let mut reordered = fill.consequences.clone();
    reordered.reverse();
    let mut sentences: Vec<String> = reordered.iter().map(ToString::to_string).collect();
    let mut expected_sorted = stated.clone();
    // The domain sorts by canonical bytes; whatever the emission order,
    // the body's set is the same value.
    sentences.sort_by_key(|s| canonical::encode(&Value::Text(s.clone())).expect("encodable"));
    expected_sorted.sort_by_key(|s| canonical::encode(&Value::Text(s.clone())).expect("encodable"));
    assert_eq!(sentences, expected_sorted);
}

// Requirements: PLAN-008, PLAN-002
//   The reversal draft's body carries the empty consequence set: a draft
//   is a prediction, its consequences are authored at its own planning
//   when it binds, and the emitted draft claims none — the slice-3p pin,
//   measured on the move's draft.
// Evidence: the_drafts_body_states_no_consequence
#[test]
fn the_drafts_body_states_no_consequence() {
    let (world, _, created, _, _) = move_fixture(Stray::None);
    let planned = plan_move(&world, created, 80 * DEFAULT_ALIGNMENT).expect("plans");
    let EmittedReversal::Draft(draft) = &planned.reversal else {
        panic!("a move emits a draft");
    };
    let Value::Map(map) = draft.body_value() else {
        panic!("draft body is a map");
    };
    assert_eq!(map.get("consequences"), Some(&Value::Array(vec![])));
}
