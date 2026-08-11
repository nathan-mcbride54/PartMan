//! Tests for the engine core (WP-050 increment 2).

use partman_domain::canonical::{self, Value};
use partman_domain::model::capability::{Operation, OperationClass, canonical_ranges};
use partman_domain::model::identity::TableState;
use partman_domain::model::naming::{
    AggregateTechnology, FileSystemKind, NamingFields, NodeId, SignatureFamily, derive_id,
};
use partman_domain::model::protection::{Facts, HostRange, TransportClass, Verdict};
use partman_domain::model::snapshot::{SnapshotKind, TopologySnapshot};
use partman_domain::model::step::{PlanStep, Severity, StepFlags, StepRefusal, StepRisk};
use partman_domain::model::topology::{Edge, EdgeKind};

use super::engine::{
    PlatformFact, RuntimeFacts, TechnologyLimits, ToolFact, ToolState, capability,
};
use super::{Reason, Remediation, Status};

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
            checksum: canonical::hash(&Value::Text("engine fixture checksum".into()))
                .expect("hashable"),
        },
    );
}

/// The fixture's facts. `bare` deliberately gets no transport fact — its
/// device-scope arm cannot decide, the `MissingFact` indeterminacy — and
/// the signatures carry extents so their canonical destructive entries
/// have reach: an extent-less target's canonical entry is empty and the
/// gate clears at capability time (the plan step's declared ranges are
/// what refuse then), which the agreement enumeration confirms rather
/// than assumes.
fn fixture_facts(full: [NodeId; 3], bare_id: NodeId, signatures: [(NodeId, NodeId); 2]) -> Facts {
    let mut facts = Facts::default();
    for id in full {
        device_facts(&mut facts, id);
    }
    facts.extents.insert(
        bare_id,
        HostRange {
            host: bare_id,
            start: 0,
            length: 1 << 30,
        },
    );
    for (sig, host) in signatures {
        facts.extents.insert(
            sig,
            HostRange {
                host,
                start: 0,
                length: 1 << 16,
            },
        );
    }
    facts
}

/// The fixture: a clean permitted device; a ZFS signature consumed by a
/// ZFS pool (the non-goal refusal, inherited by the consumed member); an
/// orphan LUKS2 signature (the indeterminate arm); a device with no
/// transport fact (device-scope indeterminacy); and an XFS file system
/// hosted by containment on the clean device, for the technology-limit
/// arm.
fn fixture() -> (TopologySnapshot, Fixture) {
    let clean = device(b"ENG-CLEAN");
    let clean_id = derive_id(&clean).expect("derivable");
    let zfs_host = device(b"ENG-ZFS");
    let zfs_host_id = derive_id(&zfs_host).expect("derivable");
    let orphan_host = device(b"ENG-ORPHAN");
    let orphan_host_id = derive_id(&orphan_host).expect("derivable");
    let bare = device(b"ENG-BARE");
    let bare_id = derive_id(&bare).expect("derivable");

    let zfs_sig = NamingFields::BackingSignature {
        host: zfs_host_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let zfs_sig_id = derive_id(&zfs_sig).expect("derivable");
    let orphan_sig = NamingFields::BackingSignature {
        host: orphan_host_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let orphan_sig_id = derive_id(&orphan_sig).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: clean_id,
        kind: FileSystemKind::Xfs,
        superblock_offset: 0,
    };
    let fs_id = derive_id(&fs).expect("derivable");
    let zfs_pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"ENG-POOL".to_vec()),
    };
    let zfs_pool_id = derive_id(&zfs_pool).expect("derivable");

    let facts = fixture_facts(
        [clean_id, zfs_host_id, orphan_host_id],
        bare_id,
        [(zfs_sig_id, zfs_host_id), (orphan_sig_id, orphan_host_id)],
    );

    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![
            clean.clone(),
            zfs_host,
            orphan_host,
            bare,
            zfs_sig,
            orphan_sig,
            fs,
            zfs_pool,
        ],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: zfs_host_id,
                target: zfs_sig_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: orphan_host_id,
                target: orphan_sig_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: clean_id,
                target: fs_id,
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
    (
        snapshot,
        Fixture {
            clean: clean_id,
            zfs_sig: zfs_sig_id,
            orphan_sig: orphan_sig_id,
            bare: bare_id,
            fs: fs_id,
            zfs_pool: zfs_pool_id,
        },
    )
}

struct Fixture {
    clean: NodeId,
    zfs_sig: NodeId,
    orphan_sig: NodeId,
    bare: NodeId,
    fs: NodeId,
    zfs_pool: NodeId,
}

fn no_limits() -> TechnologyLimits {
    TechnologyLimits::default()
}

fn risk() -> StepRisk {
    StepRisk {
        severity: Severity::Destructive,
        flags: StepFlags::default(),
    }
}

// Requirements: CAP-005, CAP-001, CAP-003
//   The agreement CAP-005 requires, enumerated: over every operation and
//   every fixture target, with no limits and clean runtime facts, the
//   engine's protection-derived answer and the plan constructor agree —
//   what the engine calls plannable constructs, what it refuses for
//   protection refuses on the same ground, and source-class operations
//   are never suppressed by any verdict.
// Evidence: the_engine_and_the_plan_constructor_agree_on_every_pair
#[test]
fn the_engine_and_the_plan_constructor_agree_on_every_pair() {
    let (snapshot, fixture) = fixture();
    let targets = [
        fixture.clean,
        fixture.zfs_sig,
        fixture.orphan_sig,
        fixture.bare,
        fixture.fs,
        fixture.zfs_pool,
    ];
    let mut enumerated = 0;
    for operation in *Operation::all() {
        for target in targets {
            let answer = capability(
                operation,
                target,
                &snapshot,
                &no_limits(),
                &RuntimeFacts::clean(),
            )
            .expect("fixture targets resolve");
            enumerated += 1;

            if operation.class() == OperationClass::Source {
                assert!(
                    !matches!(
                        answer.reason(),
                        Reason::ProtectionRefused { .. } | Reason::ProtectionIndeterminate { .. }
                    ),
                    "a source operation must never take a protection answer: \
                     {operation:?} on {target:?}"
                );
                continue;
            }

            let constructed = PlanStep::mutating(
                &snapshot,
                target,
                canonical_ranges(operation, target, snapshot.facts()),
                vec![],
                risk(),
            );
            match (answer.status(), answer.reason(), constructed) {
                (Status::Preview, Reason::UnqualifiedPendingEvidence, Ok(_)) => {}
                (
                    Status::Unsupported,
                    Reason::ProtectionRefused { ground },
                    Err(StepRefusal::Reached {
                        verdict: Verdict::Refused { ground: reached },
                        ..
                    }),
                ) => {
                    assert_eq!(
                        ground,
                        reached.into(),
                        "the refusing ground must match: {operation:?} on {target:?}"
                    );
                }
                (
                    Status::Blocked,
                    Reason::ProtectionIndeterminate { cause },
                    Err(StepRefusal::Reached {
                        verdict: Verdict::Indeterminate { cause: reached },
                        ..
                    }),
                ) => {
                    assert_eq!(
                        cause,
                        reached.into(),
                        "the indeterminate cause must match: {operation:?} on {target:?}"
                    );
                }
                (status, reason, constructed) => panic!(
                    "engine and constructor disagree for {operation:?} on {target:?}: \
                     engine said {status:?}/{reason:?}, constructor said {constructed:?}"
                ),
            }
        }
    }
    assert_eq!(enumerated, 14 * 6, "every pair enumerated, none skipped");
}

// Requirements: FS-007, CAP-003
//   ADR-0020's coupling, held: an immutable technology limit on the
//   target's own file-system kind answers `unsupported` with the limit
//   as its explicit reason and a remediation stating no remedy exists —
//   never `blocked`, which would invite remediation of the unremediable.
// Evidence: a_technology_limit_is_unsupported_with_no_remedy
#[test]
fn a_technology_limit_is_unsupported_with_no_remedy() {
    let (snapshot, fixture) = fixture();
    let limits = TechnologyLimits::new(vec![(FileSystemKind::Xfs, Operation::Shrink)]);

    let limited = capability(
        Operation::Shrink,
        fixture.fs,
        &snapshot,
        &limits,
        &RuntimeFacts::clean(),
    )
    .expect("resolves");
    assert_eq!(limited.status(), Status::Unsupported);
    assert_eq!(limited.reason(), Reason::TechnologyLimit);
    assert_eq!(*limited.remediation(), Remediation::NoneExists);

    let unlimited = capability(
        Operation::Grow,
        fixture.fs,
        &snapshot,
        &limits,
        &RuntimeFacts::clean(),
    )
    .expect("resolves");
    assert_eq!(
        unlimited.status(),
        Status::Preview,
        "a limit binds its own operation only"
    );
}

// Requirements: CAP-001, CAP-003
//   Refusal precedence is the assignment's order: protection beats the
//   floor, and a technology limit beats the floor, which beats tool
//   preconditions; within tools, missing beats out-of-range.
// Evidence: refusal_precedence_is_the_assignments_order
#[test]
fn refusal_precedence_is_the_assignments_order() {
    let (snapshot, fixture) = fixture();
    let below_floor = RuntimeFacts {
        tools: vec![ToolFact {
            tool: "mkfs.xfs".to_owned(),
            state: ToolState::Missing,
        }],
        platform: PlatformFact::BelowFloor,
    };

    // Protection first: the ZFS refusal answers even below floor.
    let protection = capability(
        Operation::Wipe,
        fixture.zfs_sig,
        &snapshot,
        &no_limits(),
        &below_floor,
    )
    .expect("resolves");
    assert!(matches!(
        protection.reason(),
        Reason::ProtectionRefused { .. }
    ));

    // Limit beats floor.
    let limits = TechnologyLimits::new(vec![(FileSystemKind::Xfs, Operation::Shrink)]);
    let limited = capability(
        Operation::Shrink,
        fixture.fs,
        &snapshot,
        &limits,
        &below_floor,
    )
    .expect("resolves");
    assert_eq!(limited.reason(), Reason::TechnologyLimit);

    // Floor beats tools.
    let floored = capability(
        Operation::Shrink,
        fixture.fs,
        &snapshot,
        &no_limits(),
        &below_floor,
    )
    .expect("resolves");
    assert_eq!(floored.reason(), Reason::PlatformFloor);
    assert_eq!(floored.status(), Status::Blocked);

    // Missing beats out-of-range.
    let two_tools = RuntimeFacts {
        tools: vec![
            ToolFact {
                tool: "xfs_growfs".to_owned(),
                state: ToolState::OutOfRange,
            },
            ToolFact {
                tool: "mkfs.xfs".to_owned(),
                state: ToolState::Missing,
            },
        ],
        platform: PlatformFact::MeetsFloor,
    };
    let missing = capability(
        Operation::Shrink,
        fixture.fs,
        &snapshot,
        &no_limits(),
        &two_tools,
    )
    .expect("resolves");
    assert_eq!(missing.reason(), Reason::ToolMissing);
    assert!(
        matches!(missing.remediation(), Remediation::Action(text) if text.contains("mkfs.xfs")),
        "the remediation names the missing tool"
    );
}

// Requirements: CAP-003
//   ACC-009's shape: a missing or out-of-range tool makes the dependent
//   capability blocked with a remediation message naming the tool, and a
//   clean toolset answers preview pending CAP-006 evidence.
// Evidence: tool_preconditions_block_with_named_remediation
#[test]
fn tool_preconditions_block_with_named_remediation() {
    let (snapshot, fixture) = fixture();
    let out_of_range = RuntimeFacts {
        tools: vec![ToolFact {
            tool: "e2fsck".to_owned(),
            state: ToolState::OutOfRange,
        }],
        platform: PlatformFact::MeetsFloor,
    };
    let blocked = capability(
        Operation::Check,
        fixture.clean,
        &snapshot,
        &no_limits(),
        &out_of_range,
    )
    .expect("resolves");
    assert_eq!(blocked.status(), Status::Blocked);
    assert_eq!(blocked.reason(), Reason::ToolVersionOutOfRange);
    assert!(matches!(blocked.remediation(), Remediation::Action(text) if text.contains("e2fsck")));

    let clean = capability(
        Operation::Check,
        fixture.clean,
        &snapshot,
        &no_limits(),
        &RuntimeFacts::clean(),
    )
    .expect("resolves");
    assert_eq!(clean.status(), Status::Preview);
    assert_eq!(clean.reason(), Reason::UnqualifiedPendingEvidence);
}

// Requirements: CAP-001
//   An address the snapshot does not carry is a caller error, not a
//   capability answer: CAP-001 is per exact target, and an answer about
//   nobody would be an invention.
// Evidence: an_unknown_target_is_a_typed_error_not_an_answer
#[test]
fn an_unknown_target_is_a_typed_error_not_an_answer() {
    let (snapshot, _) = fixture();
    let stranger = derive_id(&device(b"ENG-STRANGER")).expect("derivable");
    let refused = capability(
        Operation::Detect,
        stranger,
        &snapshot,
        &no_limits(),
        &RuntimeFacts::clean(),
    );
    assert_eq!(
        refused.expect_err("must refuse").target,
        stranger,
        "the error names the unresolvable address"
    );
}

// Requirements: CAP-003, CAP-005
//   ADR-0011's detection-only rule (LIN-006), the arm increment 4's
//   coverage requirement caught missing: a mutating operation on a
//   multipath node or on a recognized member answers `unsupported` with
//   the multipath reason and no remedy — v1 policy, not a precondition —
//   while source classes pass untouched, because detection-only means
//   detection works.
// Evidence: multipath_mutation_is_unsupported_detection_passes
#[test]
fn multipath_mutation_is_unsupported_detection_passes() {
    let first_member = device(b"ENG-MP-A");
    let first_id = derive_id(&first_member).expect("derivable");
    let second_member = device(b"ENG-MP-B");
    let second_id = derive_id(&second_member).expect("derivable");
    let mp = NamingFields::MultipathNode {
        lun_designator: b"ENG-LUN".to_vec(),
    };
    let mp_id = derive_id(&mp).expect("derivable");
    let mut facts = Facts::default();
    device_facts(&mut facts, first_id);
    device_facts(&mut facts, second_id);
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![first_member, second_member, mp],
        vec![
            Edge {
                kind: EdgeKind::PlatformMembership,
                source: mp_id,
                target: first_id,
            },
            Edge {
                kind: EdgeKind::PlatformMembership,
                source: mp_id,
                target: second_id,
            },
        ],
        facts,
    )
    .expect("assembles");

    for target in [mp_id, first_id, second_id] {
        let refused = capability(
            Operation::Wipe,
            target,
            &snapshot,
            &no_limits(),
            &RuntimeFacts::clean(),
        )
        .expect("resolves");
        assert_eq!(refused.status(), Status::Unsupported);
        assert_eq!(refused.reason(), Reason::MultipathDetectionOnly);
        assert_eq!(*refused.remediation(), Remediation::NoneExists);

        let detected = capability(
            Operation::Detect,
            target,
            &snapshot,
            &no_limits(),
            &RuntimeFacts::clean(),
        )
        .expect("resolves");
        assert_eq!(
            detected.status(),
            Status::Preview,
            "detection-only means detection works"
        );
    }
}

// Requirements: CAP-003, CAP-005, CAP-006
//   The all-statuses/all-reasons coverage the assignment's increment 4
//   names, over integration-shaped fixture topologies (constructed to
//   mirror the WP-020 catalogue's shapes — byte-level derivation arrives
//   with the platform adapters, and this narrowing is recorded in the
//   CHANGELOG): every reachable Reason variant is produced by a real
//   scenario, every reachable status appears, and the two unreachable
//   members — `supported` and its evidence-built reason — are asserted
//   unreachable, their proof being increment 1's compile_fail doctest
//   rather than an exercised path.
// Evidence: every_reachable_status_and_reason_is_exercised
#[test]
fn every_reachable_status_and_reason_is_exercised() {
    let (snapshot, fixture) = fixture();
    let no_limits = no_limits();
    let clean = RuntimeFacts::clean();
    let limits = TechnologyLimits::new(vec![(FileSystemKind::Xfs, Operation::Shrink)]);
    let below_floor = RuntimeFacts {
        tools: vec![],
        platform: PlatformFact::BelowFloor,
    };
    let missing_tool = RuntimeFacts {
        tools: vec![ToolFact {
            tool: "mkfs.xfs".to_owned(),
            state: ToolState::Missing,
        }],
        platform: PlatformFact::MeetsFloor,
    };
    let stale_tool = RuntimeFacts {
        tools: vec![ToolFact {
            tool: "e2fsck".to_owned(),
            state: ToolState::OutOfRange,
        }],
        platform: PlatformFact::MeetsFloor,
    };

    let scenarios: [(Operation, NodeId, &TechnologyLimits, &RuntimeFacts); 7] = [
        // ProtectionRefused, via the consumed ZFS chain.
        (Operation::Wipe, fixture.zfs_sig, &no_limits, &clean),
        // ProtectionIndeterminate, via the orphan signature.
        (Operation::Wipe, fixture.orphan_sig, &no_limits, &clean),
        // TechnologyLimit (ADR-0020's unsupported).
        (Operation::Shrink, fixture.fs, &limits, &clean),
        // PlatformFloor.
        (Operation::Grow, fixture.fs, &no_limits, &below_floor),
        // ToolMissing.
        (Operation::Grow, fixture.fs, &no_limits, &missing_tool),
        // ToolVersionOutOfRange.
        (Operation::Grow, fixture.fs, &no_limits, &stale_tool),
        // UnqualifiedPendingEvidence (preview, the unrefused answer).
        (Operation::Grow, fixture.fs, &no_limits, &clean),
    ];
    let answers = scenarios.map(|(operation, target, limit, runtime)| {
        capability(operation, target, &snapshot, limit, runtime)
    });
    let mut reasons: Vec<Reason> = Vec::new();
    let mut statuses: Vec<Status> = Vec::new();
    for answer in answers {
        let answer = answer.expect("resolves");
        reasons.push(answer.reason());
        statuses.push(answer.status());
    }
    // MultipathDetectionOnly is exercised by its own test over the
    // multipath fixture; count it here so the roster below is complete.
    reasons.push(Reason::MultipathDetectionOnly);
    statuses.push(Status::Unsupported);

    for reason in Reason::all_variants() {
        let covered = match reason {
            Reason::ProtectionRefused { .. } => reasons
                .iter()
                .any(|seen| matches!(seen, Reason::ProtectionRefused { .. })),
            Reason::ProtectionIndeterminate { .. } => reasons
                .iter()
                .any(|seen| matches!(seen, Reason::ProtectionIndeterminate { .. })),
            Reason::QualifiedByEvidence => {
                // Unreachable by proof, not by omission: the evidence
                // token has no constructor (increment 1's compile_fail
                // doctest, still holding), so no scenario can produce
                // this reason and none pretends to.
                !reasons.contains(&Reason::QualifiedByEvidence)
            }
            other => reasons.contains(other),
        };
        assert!(
            covered,
            "reason {reason:?} must be covered or proven unreachable"
        );
    }
    for status in [Status::Preview, Status::Unsupported, Status::Blocked] {
        assert!(statuses.contains(&status), "status {status:?} must appear");
    }
    assert!(
        !statuses.contains(&Status::Supported),
        "supported stays unreachable until CAP-006 evidence exists"
    );
}
