//! Tests for the plan body and its boundary (WP-010 increment 3i).

use crate::canonical::{self, Value};

use super::naming::{NamingFields, derive_id};
use super::plan::{OperationPlan, PlanSchemaError, ValidityWindow};
use super::protection::{Facts, HostRange, StepRanges, TransportClass};
use super::snapshot::{SnapshotKind, TopologySnapshot};
use super::step::{PlanStep, Severity, StepFlags, StepRisk};

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

fn clean_snapshot(serial: &[u8]) -> (TopologySnapshot, super::naming::NodeId) {
    let dev = device(serial);
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
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![dev], vec![], facts)
            .expect("assembles");
    (snapshot, dev_id)
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

fn risk(severity: Severity) -> StepRisk {
    StepRisk {
        severity,
        flags: StepFlags::default(),
    }
}

fn plan_over_steps(snapshot: &TopologySnapshot, steps: Vec<PlanStep>) -> OperationPlan {
    let statements = (0..steps.len())
        .map(|step| StepImpossibility {
            step,
            reason: ImpossibilityReason::DataDestroyed,
        })
        .collect();
    OperationPlan::assemble_linked(
        b"plan-1".to_vec(),
        1_700_000_000,
        snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        steps,
        ReversalLinkage::Impossible { statements },
    )
    .expect("assembles")
}

fn plan_over(snapshot: &TopologySnapshot, step: PlanStep) -> OperationPlan {
    plan_over_steps(snapshot, vec![step])
}

// Requirements: MODEL-003, MODEL-005, PLAN-006, PLAN-007
//   The plan body round-trips exactly against the snapshot it binds,
//   and its hash is what authorization would bind.
// Evidence: the_plan_body_round_trips_against_its_snapshot
#[test]
fn the_plan_body_round_trips_against_its_snapshot() {
    let (snapshot, dev_id) = clean_snapshot(b"D0");
    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    let plan = plan_over(&snapshot, step);
    let bytes = canonical::encode(&plan.body_value().expect("body")).expect("encodable");
    let rebuilt = OperationPlan::from_canonical_body(&bytes, &snapshot).expect("round-trips");
    assert_eq!(
        rebuilt.body_hash().expect("hashable"),
        plan.body_hash().expect("hashable")
    );
    assert_eq!(rebuilt.severity(), Severity::Destructive);
}

// Requirements: PLAN-006, SEC-002
//   A plan presented against a different snapshot than the one it
//   binds refuses — the ACC-007 stale-plan shape at the type layer.
// Evidence: a_plan_against_the_wrong_snapshot_refuses
#[test]
fn a_plan_against_the_wrong_snapshot_refuses() {
    let (snapshot, dev_id) = clean_snapshot(b"D0");
    let (other, _) = clean_snapshot(b"OTHER");
    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    let plan = plan_over(&snapshot, step);
    let bytes = canonical::encode(&plan.body_value().expect("body")).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&bytes, &other),
        Err(PlanSchemaError::SnapshotMismatch)
    );
}

// Requirements: SAFE-005, MODEL-005, HLP-002
//   ADR-0012's hand-forged-artifact refusal: bytes carrying a step the
//   closure refuses never parse into a plan, because every step is
//   re-run through the sole constructor.
// Evidence: a_hand_forged_step_is_refused_by_recomputation
#[test]
fn a_hand_forged_step_is_refused_by_recomputation() {
    use super::naming::{AggregateTechnology, SignatureFamily};
    use super::topology::{Edge, EdgeKind};

    // A snapshot whose device carries a consumed ZFS member.
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Zfs,
        primary_offset: 512 << 20,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
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
    facts.extents.insert(
        signature_id,
        HostRange {
            host: dev_id,
            start: 512 << 20,
            length: 1 << 20,
        },
    );
    let pool_id = derive_id(&pool).expect("derivable");
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, signature, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: dev_id,
                target: signature_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: signature_id,
                target: pool_id,
            },
        ],
        facts,
    )
    .expect("assembles");

    // Forge the bytes directly: a clean plan against a harmless
    // snapshot, retargeted at the pool-carrying device by editing the
    // decoded value tree — the type layer bypassed.
    let (clean, clean_dev) = clean_snapshot(b"CLEAN");
    let step = PlanStep::mutating(
        &clean,
        clean_dev,
        wipe(clean_dev),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    let plan = plan_over(&clean, step);
    let Value::Map(mut map) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    map.insert(
        "snapshot_hash".to_owned(),
        Value::Bytes(snapshot.body_hash().expect("hashable").as_bytes().to_vec()),
    );
    let Some(Value::Array(steps)) = map.get_mut("steps") else {
        panic!("steps present");
    };
    let Value::Map(step_map) = &mut steps[0] else {
        panic!("step is a map");
    };
    step_map.insert(
        "target".to_owned(),
        Value::Bytes(dev_id.as_bytes().to_vec()),
    );
    step_map.insert(
        "destroyed".to_owned(),
        Value::Array(vec![{
            let mut range = std::collections::BTreeMap::new();
            range.insert("host".to_owned(), Value::Bytes(dev_id.as_bytes().to_vec()));
            range.insert("start".to_owned(), Value::Unsigned(0));
            range.insert("length".to_owned(), Value::Unsigned(1 << 30));
            Value::Map(range)
        }]),
    );
    let forged = canonical::encode(&Value::Map(map)).expect("encodable");
    let result = OperationPlan::from_canonical_body(&forged, &snapshot);
    assert!(
        matches!(result, Err(PlanSchemaError::Step(_))),
        "the forged step must refuse through the sole constructor: {result:?}"
    );
}

// Requirements: PLAN-004
//   Plan severity is the maximum step severity.
// Evidence: plan_severity_is_the_maximum_step_severity
#[test]
fn plan_severity_is_the_maximum_step_severity() {
    let (snapshot, dev_id) = clean_snapshot(b"D0");
    // Informational deliberately: a Reversible claim in this unlinked
    // body form refuses (ADR-0022's rule, tested on its own below).
    let low = PlanStep::mutating(
        &snapshot,
        dev_id,
        StepRanges::default(),
        vec![],
        risk(Severity::Informational),
    )
    .expect("constructs");
    let high = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    let plan = plan_over_steps(&snapshot, vec![low, high]);
    assert_eq!(plan.severity(), Severity::Destructive);
}

// Requirements: MODEL-003
//   Unknown plan fields and unknown step fields are typed refusals.
// Evidence: unknown_plan_and_step_fields_are_refused
#[test]
fn unknown_plan_and_step_fields_are_refused() {
    let (snapshot, dev_id) = clean_snapshot(b"D0");
    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    let plan = plan_over(&snapshot, step);
    let Value::Map(map) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    let mut extra = map.clone();
    extra.insert("surprise".to_owned(), Value::Unsigned(1));
    let bytes = canonical::encode(&Value::Map(extra)).expect("encodable");
    assert!(matches!(
        OperationPlan::from_canonical_body(&bytes, &snapshot),
        Err(PlanSchemaError::UnknownField { .. })
    ));
}

use super::naming::TableRole;
use super::plan::{
    DraftPrecondition, DraftStep, DraftTarget, ImpossibilityReason, PlanError, ReversalDraft,
    ReversalLinkage, StepImpossibility,
};
use super::step::Precondition;
use super::topology::{Edge, EdgeKind};

/// The create-reversal worlds (ADR-0022): a device before the forward
/// apply, the simulated prediction of the created partition, the world
/// after a real apply, and the same world after data landed in the
/// created partition.
struct ReversalWorlds {
    pre: TopologySnapshot,
    proposal: TopologySnapshot,
    post: TopologySnapshot,
    post_with_data: TopologySnapshot,
    dev_id: super::naming::NodeId,
    created_range: HostRange,
}

fn reversal_worlds() -> ReversalWorlds {
    let dev = device(b"RV0");
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
    let created_range = HostRange {
        host: dev_id,
        start: 1 << 20,
        length: 10 << 20,
    };
    let fs = NamingFields::FileSystem {
        host: part_id,
        kind: super::naming::FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");

    let base_facts = |with_part: bool, with_fs: bool| {
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
        if with_part {
            facts.extents.insert(part_id, created_range);
        }
        if with_fs {
            // In the containment root's frame — the device's — like the
            // partition it names as host (ADR-0037, enforced by ADR-0046).
            facts.extents.insert(
                fs_id,
                HostRange {
                    host: dev_id,
                    start: 1 << 20,
                    length: 5 << 20,
                },
            );
        }
        facts
    };
    let containment = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    let assemble = |kind, nodes: Vec<NamingFields>, edges: Vec<Edge>, facts| {
        TopologySnapshot::assemble(kind, false, nodes, edges, facts).expect("assembles")
    };

    let pre = assemble(
        SnapshotKind::Captured,
        vec![dev.clone(), table.clone()],
        vec![containment(dev_id, table_id)],
        base_facts(false, false),
    );
    let proposal = assemble(
        SnapshotKind::Simulated,
        vec![dev.clone(), table.clone(), part.clone()],
        vec![
            containment(dev_id, table_id),
            containment(table_id, part_id),
        ],
        base_facts(true, false),
    );
    let post = assemble(
        SnapshotKind::Captured,
        vec![dev.clone(), table.clone(), part.clone()],
        vec![
            containment(dev_id, table_id),
            containment(table_id, part_id),
        ],
        base_facts(true, false),
    );
    let post_with_data = assemble(
        SnapshotKind::Captured,
        vec![dev, table, part, fs],
        vec![
            containment(dev_id, table_id),
            containment(table_id, part_id),
        ],
        base_facts(true, true),
    );
    ReversalWorlds {
        pre,
        proposal,
        post,
        post_with_data,
        dev_id,
        created_range,
    }
}

/// The forward create step, its emitted reversal draft, and the linked
/// forward plan carrying the draft's hash.
fn forward_and_draft(worlds: &ReversalWorlds) -> (OperationPlan, ReversalDraft) {
    let create = PlanStep::mutating(
        &worlds.pre,
        worlds.dev_id,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![worlds.created_range],
            destroyed: vec![],
        },
        vec![],
        risk(Severity::Disruptive),
    )
    .expect("constructs");
    let draft = ReversalDraft::compose(
        b"plan-fwd/reversal".to_vec(),
        1_700_000_000,
        &worlds.proposal,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        vec![DraftStep {
            target: DraftTarget::StepOutput(0),
            ranges: StepRanges {
                written_table_extents: vec![],
                consumed: vec![],
                destroyed: vec![worlds.created_range],
            },
            acknowledgments: vec![],
            risk: risk(Severity::Reversible),
            preconditions: vec![DraftPrecondition::StepOutputUnoccupied { step: 0 }],
        }],
        b"plan-fwd".to_vec(),
        std::slice::from_ref(&create),
    )
    .expect("the draft composes against the prediction");
    let forward = OperationPlan::assemble_linked(
        b"plan-fwd".to_vec(),
        1_700_000_000,
        &worlds.pre,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![create],
        ReversalLinkage::Draft {
            plan_id: draft.plan_id().to_vec(),
            draft_hash: draft.body_hash().expect("hashable"),
        },
    )
    .expect("the linked forward plan assembles");
    (forward, draft)
}

// Requirements: MODEL-003, MODEL-005, PLAN-006
//   The linked (version-2) body round-trips through the typed boundary
//   with its reversal linkage and per-step preconditions intact, and
//   the draft body round-trips its own boundary — the recompute rule
//   held for both artifacts of ADR-0022's architecture.
// Evidence: linked_bodies_and_drafts_round_trip
#[test]
fn linked_bodies_and_drafts_round_trip() {
    let worlds = reversal_worlds();
    let (forward, draft) = forward_and_draft(&worlds);
    let bytes = canonical::encode(&forward.body_value().expect("body")).expect("encodable");
    let rebuilt = OperationPlan::from_canonical_body(&bytes, &worlds.pre).expect("round-trips");
    assert_eq!(
        rebuilt.body_hash().expect("hashable"),
        forward.body_hash().expect("hashable")
    );
    assert!(matches!(
        rebuilt.reversal(),
        Some(ReversalLinkage::Draft { .. })
    ));

    let draft_bytes = canonical::encode(&draft.body_value()).expect("encodable");
    let draft_rebuilt =
        ReversalDraft::from_canonical_body(&draft_bytes).expect("the draft round-trips");
    assert_eq!(draft_rebuilt, draft);
    assert_eq!(draft_rebuilt.forward_plan_id(), b"plan-fwd");
}

// Requirements: PLAN-004, MODEL-005
//   ADR-0022's severity rule, structural at assembly and at the
//   boundary: a Reversible claim stands only on an emitted reversal —
//   an impossibility linkage refuses it, and a forged body claiming
//   severity 1 over an impossibility linkage never parses. (The
//   unlinked form's outright refusal is gone with the form itself:
//   since the version-1 retirement a plan without a linkage is
//   unconstructible, not refused.)
// Evidence: a_reversible_claim_stands_only_on_an_emitted_reversal
#[test]
fn a_reversible_claim_stands_only_on_an_emitted_reversal() {
    let (snapshot, dev_id) = clean_snapshot(b"D0");
    let reversible = PlanStep::mutating(
        &snapshot,
        dev_id,
        StepRanges::default(),
        vec![],
        risk(Severity::Reversible),
    )
    .expect("constructs");

    let impossible = OperationPlan::assemble_linked(
        b"plan-r".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![reversible],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::DataDestroyed,
            }],
        },
    );
    assert_eq!(impossible, Err(PlanError::ReversibleWithoutReversal));

    // The forged spelling: a legal severity-2 impossibility plan whose
    // severity byte is edited to 1 after assembly.
    let disruptive = PlanStep::mutating(
        &snapshot,
        dev_id,
        StepRanges::default(),
        vec![],
        risk(Severity::Disruptive),
    )
    .expect("constructs");
    let plan = OperationPlan::assemble_linked(
        b"plan-r".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![disruptive],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::DataDestroyed,
            }],
        },
    )
    .expect("assembles");
    let Value::Map(mut map) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    let Some(Value::Array(steps)) = map.get_mut("steps") else {
        panic!("steps present");
    };
    let Value::Map(step_map) = &mut steps[0] else {
        panic!("step is a map");
    };
    step_map.insert("severity".to_owned(), Value::Unsigned(1));
    let forged = canonical::encode(&Value::Map(map)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&forged, &snapshot),
        Err(PlanSchemaError::ReversibleWithoutReversal)
    );
}

// Requirements: MODEL-005, PLAN-006
//   A step-output reference resolves against a post-apply capture and
//   refuses against a pre-apply one; the bound plan is an ordinary
//   linked plan bound to the capture's hash, carrying the resolved
//   preconditions and the reapply-forward statement that terminates the
//   regress.
// Evidence: a_reference_resolves_after_apply_and_refuses_before
#[test]
fn a_reference_resolves_after_apply_and_refuses_before() {
    let worlds = reversal_worlds();
    let (forward, draft) = forward_and_draft(&worlds);

    let bound = draft
        .bind(&worlds.post, &forward)
        .expect("the reference resolves against the post-apply capture");
    assert_eq!(
        bound.snapshot_hash(),
        &worlds.post.body_hash().expect("hashable"),
        "binding is a validation act: the bound plan binds the capture"
    );
    assert_eq!(
        bound.reversal(),
        Some(&ReversalLinkage::ReapplyForward {
            forward_plan_id: b"plan-fwd".to_vec()
        })
    );
    assert_eq!(bound.severity(), Severity::Reversible);
    assert!(matches!(
        bound.steps()[0].preconditions(),
        [Precondition::HostUnoccupied { .. }]
    ));

    let refused = draft
        .bind(&worlds.pre, &forward)
        .expect_err("a pre-apply world resolves nothing");
    assert_eq!(
        refused,
        super::plan::BindRefusal::UnresolvedReference {
            step: 0,
            candidates: 0
        }
    );

    // A forward plan under a different ID: the draft refuses to bind
    // against a plan it does not reverse.
    let create = PlanStep::mutating(
        &worlds.pre,
        worlds.dev_id,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![worlds.created_range],
            destroyed: vec![],
        },
        vec![],
        risk(Severity::Disruptive),
    )
    .expect("constructs");
    let other_forward = OperationPlan::assemble_linked(
        b"plan-other".to_vec(),
        1_700_000_000,
        &worlds.pre,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![create],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::PriorValueNotCarried,
            }],
        },
    )
    .expect("assembles");
    assert_eq!(
        draft.bind(&worlds.post, &other_forward),
        Err(super::plan::BindRefusal::NotItsForwardPlan)
    );
}

// Requirements: MODEL-005
//   Truthfulness is a two-time property (ADR-0022's named fixture): the
//   reversal that was metadata-only at emission refuses by precondition
//   once data landed in the created structure — at the draft's binding
//   and at the plain boundary alike — instead of silently becoming a
//   destructive plan wearing a reversal's advertisement.
// Evidence: a_decayed_precondition_refuses_at_binding
#[test]
fn a_decayed_precondition_refuses_at_binding() {
    let worlds = reversal_worlds();
    let (forward, draft) = forward_and_draft(&worlds);
    let refused = draft
        .bind(&worlds.post_with_data, &forward)
        .expect_err("data landed; the reversal is no longer metadata-only");
    assert!(matches!(
        refused,
        super::plan::BindRefusal::PreconditionFailed { .. }
    ));

    // The same decay at the plain boundary: bind while clean, then
    // present the bound bytes against the data-carrying world.
    let bound = draft.bind(&worlds.post, &forward).expect("binds clean");
    let bytes = canonical::encode(&bound.body_value().expect("body")).expect("encodable");
    let result = OperationPlan::from_canonical_body(&bytes, &worlds.post_with_data);
    assert!(
        matches!(
            result,
            Err(PlanSchemaError::SnapshotMismatch | PlanSchemaError::PreconditionFailed { .. })
        ),
        "a decayed world refuses one way or the other: {result:?}"
    );
}

// Requirements: MODEL-005
//   The boundary's own precondition check: a linked plan assembled over
//   a world that already violates a step precondition never parses
//   against that world — assembly is permissive (the planner's honesty
//   is judged elsewhere), the boundary is not.
// Evidence: a_precondition_violated_in_the_bound_world_never_parses
#[test]
fn a_precondition_violated_in_the_bound_world_never_parses() {
    let worlds = reversal_worlds();
    let part_id = worlds
        .post_with_data
        .facts()
        .extents
        .iter()
        .find(|(_, extent)| **extent == worlds.created_range)
        .map(|(node, _)| *node)
        .expect("the created partition is placed");
    let step = PlanStep::mutating(
        &worlds.post_with_data,
        part_id,
        StepRanges::default(),
        vec![],
        risk(Severity::Disruptive),
    )
    .expect("constructs")
    .with_preconditions(vec![Precondition::HostUnoccupied { host: part_id }]);
    let plan = OperationPlan::assemble_linked(
        b"plan-x".to_vec(),
        1_700_000_000,
        &worlds.post_with_data,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::DataDestroyed,
            }],
        },
    )
    .expect("assembly is permissive");
    let bytes = canonical::encode(&plan.body_value().expect("body")).expect("encodable");
    assert!(matches!(
        OperationPlan::from_canonical_body(&bytes, &worlds.post_with_data),
        Err(PlanSchemaError::PreconditionFailed { .. })
    ));
}

use super::identity::IndeterminateCause;
use super::plan::{ImpossibilityReason as Reason3, StepImpossibility as Statement3};
use super::step::{Acknowledgment, Cancellation, StepClass, StepRefusal};

/// A device whose authored table state is `Indeterminate`, its damaged
/// table located as a child extent — ADR-0024's repair-arm world — and
/// a `Present`-state twin beside it.
fn repair_world() -> (TopologySnapshot, super::naming::NodeId, HostRange) {
    use super::identity::TableState;
    use super::naming::TableRole;

    let dev = device(b"RPR");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
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
    facts.extents.insert(table_id, table_region);
    facts.table_states.insert(
        dev_id,
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
    );
    let snapshot = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev, table],
        vec![super::topology::Edge {
            kind: super::topology::EdgeKind::Containment,
            source: dev_id,
            target: table_id,
        }],
        facts,
    )
    .expect("assembles");
    (snapshot, dev_id, table_region)
}

// Requirements: MODEL-005, PLAN-004
//   ADR-0024's acknowledgment arms are class-conditioned and
//   unconstructible outside the typed repair family: the
//   capture-impossible entry (naming exact, well-formed regions) and
//   the identity-bound-restore entry construct exactly on a
//   table-repair step over an Indeterminate-state device, and refuse on
//   an ordinary step, over a Present state, and with malformed regions.
// Evidence: the_capture_impossible_acknowledgment_attaches_only_to_the_repair_family
#[test]
fn the_capture_impossible_acknowledgment_attaches_only_to_the_repair_family() {
    let (snapshot, dev_id, table_region) = repair_world();
    let repair_ranges = || StepRanges {
        written_table_extents: vec![table_region],
        consumed: vec![],
        destroyed: vec![],
    };
    let acknowledged = |regions: Vec<HostRange>| Acknowledgment::UncapturableRegions {
        table: dev_id,
        regions,
    };

    PlanStep::mutating_classed(
        &snapshot,
        dev_id,
        repair_ranges(),
        vec![acknowledged(vec![table_region])],
        risk(Severity::Disruptive),
        StepClass::TableRepair,
    )
    .expect("the repair family constructs with the acknowledgment");
    PlanStep::mutating_classed(
        &snapshot,
        dev_id,
        repair_ranges(),
        vec![Acknowledgment::IdentityBoundRestore { table: dev_id }],
        risk(Severity::Disruptive),
        StepClass::TableRepair,
    )
    .expect("identity-bound-restore's arm exists on the repair family");

    // Outside the family: unconstructible — ADR-0024 fixture 3's last
    // clause, as the constructor law rather than discipline.
    let ordinary = PlanStep::mutating(
        &snapshot,
        dev_id,
        repair_ranges(),
        vec![acknowledged(vec![table_region])],
        risk(Severity::Disruptive),
    );
    assert!(matches!(
        ordinary,
        Err(StepRefusal::UnlawfulAcknowledgment { .. })
    ));
    let restore_ordinary = PlanStep::mutating(
        &snapshot,
        dev_id,
        repair_ranges(),
        vec![Acknowledgment::IdentityBoundRestore { table: dev_id }],
        risk(Severity::Disruptive),
    );
    assert!(matches!(
        restore_ordinary,
        Err(StepRefusal::UnlawfulAcknowledgment { .. })
    ));

    // On a positively determined table: unconstructible in the family
    // too — the arm is Indeterminate's alone, for both kinds.
    let (present, present_dev) = clean_snapshot_with_table(b"RPR-OK");
    let on_present = PlanStep::mutating_classed(
        &present,
        present_dev,
        StepRanges::default(),
        vec![Acknowledgment::UncapturableRegions {
            table: present_dev,
            regions: vec![HostRange {
                host: present_dev,
                start: 0,
                length: 512,
            }],
        }],
        risk(Severity::Disruptive),
        StepClass::TableRepair,
    );
    assert!(matches!(
        on_present,
        Err(StepRefusal::UnlawfulAcknowledgment { .. })
    ));
    let restore_on_present = PlanStep::mutating_classed(
        &present,
        present_dev,
        StepRanges::default(),
        vec![Acknowledgment::IdentityBoundRestore { table: present_dev }],
        risk(Severity::Disruptive),
        StepClass::TableRepair,
    );
    assert!(matches!(
        restore_on_present,
        Err(StepRefusal::UnlawfulAcknowledgment { .. })
    ));
}

// Requirements: MODEL-005
//   The capture-impossible acknowledgment must name well-formed regions
//   — the journal's discipline at the constructor: empty sets,
//   zero-length regions, overlaps, and wrong-host regions all refuse.
// Evidence: malformed_capture_impossible_regions_refuse
#[test]
fn malformed_capture_impossible_regions_refuse() {
    let (snapshot, dev_id, table_region) = repair_world();
    let repair_ranges = || StepRanges {
        written_table_extents: vec![table_region],
        consumed: vec![],
        destroyed: vec![],
    };
    let acknowledged = |regions: Vec<HostRange>| Acknowledgment::UncapturableRegions {
        table: dev_id,
        regions,
    };
    // Malformed regions refuse: empty, zero-length, overlapping, and a
    // region on the wrong host.
    for regions in [
        vec![],
        vec![HostRange {
            host: dev_id,
            start: 0,
            length: 0,
        }],
        vec![
            HostRange {
                host: dev_id,
                start: 0,
                length: 1024,
            },
            HostRange {
                host: dev_id,
                start: 512,
                length: 1024,
            },
        ],
        vec![HostRange {
            host: derive_id(&device(b"OTHER")).expect("derivable"),
            start: 0,
            length: 512,
        }],
    ] {
        let malformed = PlanStep::mutating_classed(
            &snapshot,
            dev_id,
            repair_ranges(),
            vec![acknowledged(regions)],
            risk(Severity::Disruptive),
            StepClass::TableRepair,
        );
        assert!(matches!(
            malformed,
            Err(StepRefusal::UnlawfulAcknowledgment { .. })
        ));
    }
}

/// A clean device with a Present table state (the fixture above's
/// positively determined twin).
fn clean_snapshot_with_table(serial: &[u8]) -> (TopologySnapshot, super::naming::NodeId) {
    use super::identity::TableState;
    let dev = device(serial);
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
    facts.table_states.insert(
        dev_id,
        TableState::Present {
            checksum: canonical::hash(&Value::Text("repair twin checksum".into()))
                .expect("hashable"),
        },
    );
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![dev], vec![], facts)
            .expect("assembles");
    (snapshot, dev_id)
}

// Requirements: MODEL-003, MODEL-005
//   The classed (version-3) body round-trips with its class and
//   acknowledgment intact; a forged class flip never parses (the
//   acknowledgment law re-runs at the boundary); and the superseded
//   version 2 is refused at decode.
// Evidence: the_classed_body_round_trips_and_a_forged_class_never_parses
#[test]
fn the_classed_body_round_trips_and_a_forged_class_never_parses() {
    let (snapshot, dev_id, table_region) = repair_world();
    let step = PlanStep::mutating_classed(
        &snapshot,
        dev_id,
        StepRanges {
            written_table_extents: vec![table_region],
            consumed: vec![],
            destroyed: vec![],
        },
        vec![Acknowledgment::UncapturableRegions {
            table: dev_id,
            regions: vec![table_region],
        }],
        risk(Severity::Disruptive),
        StepClass::TableRepair,
    )
    .expect("constructs");
    let plan = OperationPlan::assemble_linked(
        b"plan-rpr".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: vec![Statement3 {
                step: 0,
                reason: Reason3::PreStatePreservedForRecovery,
            }],
        },
    )
    .expect("assembles");
    let bytes = canonical::encode(&plan.body_value().expect("body")).expect("encodable");
    let rebuilt = OperationPlan::from_canonical_body(&bytes, &snapshot).expect("round-trips");
    assert_eq!(rebuilt.steps()[0].class(), StepClass::TableRepair);

    // Forge the class to ordinary: the acknowledgment law re-runs at
    // the boundary and refuses the entry the ordinary class cannot
    // carry.
    let Value::Map(mut map) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    let Some(Value::Array(steps)) = map.get_mut("steps") else {
        panic!("steps present");
    };
    let Value::Map(step_map) = &mut steps[0] else {
        panic!("step is a map");
    };
    step_map.insert("class".to_owned(), Value::Text("ordinary".to_owned()));
    let forged = canonical::encode(&Value::Map(map)).expect("encodable");
    assert!(matches!(
        OperationPlan::from_canonical_body(&forged, &snapshot),
        Err(PlanSchemaError::Step(
            StepRefusal::UnlawfulAcknowledgment { .. }
        ))
    ));

    // The one-window version 2 is refused at decode.
    let Value::Map(mut downgraded) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    downgraded.insert("schema_version".to_owned(), Value::Unsigned(2));
    let stale = canonical::encode(&Value::Map(downgraded)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&stale, &snapshot),
        Err(PlanSchemaError::WrongSchemaVersion)
    );
}

// Requirements: PLAN-005, MODEL-003, MODEL-005
//   The version-4 body carries every step's cancellation declaration:
//   a declared value rides the hashed body and round-trips intact, the
//   vocabulary is closed at PLAN-005's three words (an unknown spelling
//   refuses), the field is required (a linked body without it
//   refuses), and the superseded version 3 — the linked form without
//   the field — is refused at decode, its retirement recorded.
// Evidence: the_cancellation_declaration_rides_the_body_and_the_vocabulary_is_closed
#[test]
fn the_cancellation_declaration_rides_the_body_and_the_vocabulary_is_closed() {
    let (snapshot, dev_id) = clean_snapshot(b"CANCEL");
    let step = PlanStep::mutating_declared(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
        StepClass::Ordinary,
        Cancellation::CheckpointCancellable,
    )
    .expect("constructs");
    let plan = OperationPlan::assemble_linked(
        b"plan-cxl".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: vec![Statement3 {
                step: 0,
                reason: Reason3::DataDestroyed,
            }],
        },
    )
    .expect("assembles");
    let bytes = canonical::encode(&plan.body_value().expect("body")).expect("encodable");
    let rebuilt = OperationPlan::from_canonical_body(&bytes, &snapshot).expect("round-trips");
    assert_eq!(
        rebuilt.steps()[0].cancellation(),
        Cancellation::CheckpointCancellable,
        "the declared value rides the body"
    );

    // The delegating constructors sit on the fail-closed floor.
    let defaulted = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    assert_eq!(defaulted.cancellation(), Cancellation::NonCancellable);

    let mutate_step = |edit: &dyn Fn(&mut std::collections::BTreeMap<String, Value>)| {
        let Value::Map(mut map) = plan.body_value().expect("body") else {
            panic!("body is a map");
        };
        {
            let Some(Value::Array(steps)) = map.get_mut("steps") else {
                panic!("steps present");
            };
            let Value::Map(step_map) = &mut steps[0] else {
                panic!("step is a map");
            };
            edit(step_map);
        }
        canonical::encode(&Value::Map(map)).expect("encodable")
    };

    // The vocabulary is closed: a fourth word never parses.
    let unknown = mutate_step(&|step_map| {
        step_map.insert(
            "cancellation".to_owned(),
            Value::Text("maybe-cancellable".to_owned()),
        );
    });
    assert_eq!(
        OperationPlan::from_canonical_body(&unknown, &snapshot),
        Err(PlanSchemaError::MalformedStep)
    );

    // The field is required content, not an option.
    let missing = mutate_step(&|step_map| {
        step_map.remove("cancellation");
    });
    assert_eq!(
        OperationPlan::from_canonical_body(&missing, &snapshot),
        Err(PlanSchemaError::MalformedStep)
    );

    // The one-window version 3 — the linked form without the
    // cancellation field — is refused at decode (MODEL-003's
    // explicit-migration discipline; the retirement this test records).
    let Value::Map(mut downgraded) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    downgraded.insert("schema_version".to_owned(), Value::Unsigned(3));
    let stale = canonical::encode(&Value::Map(downgraded)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&stale, &snapshot),
        Err(PlanSchemaError::WrongSchemaVersion)
    );
}

// Requirements: MODEL-003
//   The version-1 retirement (slice 3o): the unlinked form is refused
//   at decode like every other retired version, and the live version
//   is the only one the boundary accepts. Nothing emits version 1 —
//   its last emitters were this crate's own tests and vectors, and
//   `OperationPlan::assemble` is gone with it, so a plan without a
//   reversal linkage is unconstructible rather than refused.
// Evidence: the_retired_version_1_refuses_at_decode
#[test]
fn the_retired_version_1_refuses_at_decode() {
    let (snapshot, dev_id) = clean_snapshot(b"V1");
    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");
    let plan = plan_over(&snapshot, step);
    let Value::Map(mut downgraded) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    downgraded.insert("schema_version".to_owned(), Value::Unsigned(1));
    let stale = canonical::encode(&Value::Map(downgraded)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&stale, &snapshot),
        Err(PlanSchemaError::WrongSchemaVersion)
    );

    // The v1 spelling proper — no reversal, no preconditions, no class,
    // no cancellation — refuses the same way, not by a field error: the
    // version gate comes first.
    let Value::Map(mut unlinked) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    unlinked.insert("schema_version".to_owned(), Value::Unsigned(1));
    unlinked.remove("reversal");
    {
        let Some(Value::Array(steps)) = unlinked.get_mut("steps") else {
            panic!("steps present");
        };
        let Value::Map(step_map) = &mut steps[0] else {
            panic!("step is a map");
        };
        step_map.remove("preconditions");
        step_map.remove("class");
        step_map.remove("cancellation");
    }
    let v1_shaped = canonical::encode(&Value::Map(unlinked)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&v1_shaped, &snapshot),
        Err(PlanSchemaError::WrongSchemaVersion)
    );
}

// Requirements: MODEL-005, MODEL-003
//   A prediction proposes and never binds, structurally and everywhere:
//   composing a draft demands the simulated proposal, binding demands a
//   real capture, and the plain boundary refuses a simulated snapshot
//   as a binding base before reading a single field.
// Evidence: a_prediction_never_binds_anywhere
#[test]
fn a_prediction_never_binds_anywhere() {
    let worlds = reversal_worlds();
    let (forward, draft) = forward_and_draft(&worlds);

    assert_eq!(
        draft.bind(&worlds.proposal, &forward),
        Err(super::plan::BindRefusal::PredictionNeverBinds)
    );

    let bytes = canonical::encode(&forward.body_value().expect("body")).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&bytes, &worlds.proposal),
        Err(PlanSchemaError::PredictionNeverBinds)
    );

    let create = PlanStep::mutating(
        &worlds.pre,
        worlds.dev_id,
        StepRanges {
            written_table_extents: vec![],
            consumed: vec![worlds.created_range],
            destroyed: vec![],
        },
        vec![],
        risk(Severity::Disruptive),
    )
    .expect("constructs");
    assert_eq!(
        ReversalDraft::compose(
            b"r".to_vec(),
            1_700_000_000,
            &worlds.pre,
            ValidityWindow {
                not_after: 1_700_086_400,
            },
            vec![],
            b"plan-fwd".to_vec(),
            std::slice::from_ref(&create),
        ),
        Err(super::plan::DraftRefusal::ProposalMustBeSimulated)
    );
}

// Requirements: MODEL-003, MODEL-005
//   The linkage asymmetry is acyclic by construction: the forward side
//   carries a hash, the draft side carries an ID only, and a
//   reapply-forward linkage smuggling a hash key refuses as an unknown
//   field — the mutual-hash spelling has no encoding.
// Evidence: the_linkage_asymmetry_has_no_mutual_hash_spelling
#[test]
fn the_linkage_asymmetry_has_no_mutual_hash_spelling() {
    let worlds = reversal_worlds();
    let (forward, draft) = forward_and_draft(&worlds);

    let Value::Map(forward_body) = forward.body_value().expect("body") else {
        panic!("body is a map");
    };
    let Some(Value::Map(linkage)) = forward_body.get("reversal") else {
        panic!("the forward body carries the linkage");
    };
    assert!(linkage.contains_key("hash"), "forward side: by hash");

    let Value::Map(mut draft_body) = draft.body_value() else {
        panic!("draft body is a map");
    };
    let Some(Value::Map(draft_linkage)) = draft_body.get("reversal") else {
        panic!("the draft body carries the statement");
    };
    assert!(
        !draft_linkage.contains_key("hash"),
        "draft side: by ID only"
    );

    // Forge the mutual-hash spelling and watch it refuse.
    let Some(Value::Map(draft_linkage)) = draft_body.get_mut("reversal") else {
        panic!("present");
    };
    draft_linkage.insert("hash".to_owned(), Value::Bytes(vec![0; 32]));
    let forged = canonical::encode(&Value::Map(draft_body)).expect("encodable");
    assert!(matches!(
        ReversalDraft::from_canonical_body(&forged),
        Err(PlanSchemaError::UnknownField { .. })
    ));
}

// Requirements: PLAN-005, MODEL-003
//   A draft step's cancellation is pinned to the non-cancellable floor
//   exactly as its class is pinned to ordinary: the emitted draft
//   carries the floor, and a draft body claiming a stronger class for
//   its step refuses at decode — a draft family off the floor is a
//   future reviewed extension of the recorded decision, not a
//   spelling.
// Evidence: a_draft_step_sits_on_the_cancellation_floor
#[test]
fn a_draft_step_sits_on_the_cancellation_floor() {
    let worlds = reversal_worlds();
    let (_, draft) = forward_and_draft(&worlds);

    let Value::Map(mut draft_body) = draft.body_value() else {
        panic!("draft body is a map");
    };
    {
        let Some(Value::Array(steps)) = draft_body.get("steps") else {
            panic!("steps present");
        };
        let Value::Map(step_map) = &steps[0] else {
            panic!("step is a map");
        };
        assert_eq!(
            step_map.get("cancellation"),
            Some(&Value::Text("non-cancellable".to_owned())),
            "the emitted draft carries the floor"
        );
    }

    let Some(Value::Array(steps)) = draft_body.get_mut("steps") else {
        panic!("steps present");
    };
    let Value::Map(step_map) = &mut steps[0] else {
        panic!("step is a map");
    };
    step_map.insert(
        "cancellation".to_owned(),
        Value::Text("cancellable".to_owned()),
    );
    let forged = canonical::encode(&Value::Map(draft_body)).expect("encodable");
    assert_eq!(
        ReversalDraft::from_canonical_body(&forged),
        Err(PlanSchemaError::MalformedStep)
    );
}

// Requirements: MODEL-003
//   PLAN-008's second arm is complete per step or refused: statements
//   must cover exactly the plan's step indices in order, at assembly
//   and at the boundary, and the draft's step-output spelling never
//   parses as a bound plan's step.
// Evidence: impossibility_coverage_and_draft_spellings_are_enforced
#[test]
fn impossibility_coverage_and_draft_spellings_are_enforced() {
    let (snapshot, dev_id) = clean_snapshot(b"D0");
    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        wipe(dev_id),
        vec![],
        risk(Severity::Destructive),
    )
    .expect("constructs");

    let uncovered = OperationPlan::assemble_linked(
        b"plan-i".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step.clone()],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 1,
                reason: ImpossibilityReason::DataDestroyed,
            }],
        },
    );
    assert_eq!(uncovered, Err(PlanError::MalformedLinkage));

    let plan = OperationPlan::assemble_linked(
        b"plan-i".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::DataDestroyed,
            }],
        },
    )
    .expect("assembles");

    // The draft-only target spelling refuses at the plain boundary.
    let Value::Map(mut map) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    let Some(Value::Array(steps)) = map.get_mut("steps") else {
        panic!("steps present");
    };
    let Value::Map(step_map) = &mut steps[0] else {
        panic!("step is a map");
    };
    step_map.remove("target");
    step_map.insert("target_step_output".to_owned(), Value::Unsigned(0));
    let forged = canonical::encode(&Value::Map(map)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&forged, &snapshot),
        Err(PlanSchemaError::DraftSpellingOutsideDraft)
    );
}
#[test]
fn a_client_authored_table_state_never_validates() {
    use super::identity::{DeviceIdentity, IndeterminateCause, TableState};

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
    // The helper's stamp: the parser found this device Indeterminate.
    facts.table_states.insert(
        dev_id,
        TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        },
    );
    let snapshot =
        TopologySnapshot::assemble(SnapshotKind::Captured, false, vec![dev], vec![], facts)
            .expect("assembles");

    let identity = |table: TableState| DeviceIdentity {
        serial: Some(b"D0".to_vec()),
        wwn: None,
        os_instance_id: None,
        connection_path: None,
        total_bytes: 1 << 30,
        logical_sector_size: Some(512),
        physical_sector_size: Some(512),
        table,
        witness: None,
    };

    let step = PlanStep::mutating(
        &snapshot,
        dev_id,
        StepRanges::default(),
        vec![],
        risk(Severity::Informational),
    )
    .expect("constructs");

    // The honest plan: identity agreeing with the stamp round-trips.
    let mut agreeing = std::collections::BTreeMap::new();
    agreeing.insert(
        dev_id,
        identity(TableState::Indeterminate {
            cause: IndeterminateCause::Ambiguous,
        }),
    );
    let plan = OperationPlan::assemble_linked(
        b"plan-3".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        agreeing,
        vec![step],
        ReversalLinkage::Impossible {
            statements: vec![StepImpossibility {
                step: 0,
                reason: ImpossibilityReason::PriorValueNotCarried,
            }],
        },
    )
    .expect("assembles");
    let bytes = canonical::encode(&plan.body_value().expect("body")).expect("encodable");
    let rebuilt = OperationPlan::from_canonical_body(&bytes, &snapshot).expect("round-trips");
    assert_eq!(rebuilt.identities().len(), 1);

    // The forged plan: the identity claims Present where the stamp says
    // Indeterminate — a client-authored value, refused.
    let Value::Map(mut map) = plan.body_value().expect("body") else {
        panic!("body is a map");
    };
    let Some(Value::Map(identities)) = map.get_mut("identities") else {
        panic!("identities present");
    };
    let key = identities.keys().next().expect("one identity").clone();
    let forged_identity = identity(TableState::Present {
        checksum: canonical::hash(&Value::Unsigned(0)).expect("hashable"),
    });
    identities.insert(key, forged_identity.body_value());
    let forged = canonical::encode(&Value::Map(map)).expect("encodable");
    assert_eq!(
        OperationPlan::from_canonical_body(&forged, &snapshot),
        Err(PlanSchemaError::AuthoredFieldMismatch)
    );
}

// Requirements: MODEL-005, PLAN-008
//   ADR-0022's truthfulness precondition reads occupancy as bytes, not as
//   frame names (issue #401, ADR-0046). Under ADR-0037's rule a
//   partition is never a frame, so "an extent framed on the host" finds
//   nothing on any lawful capture and a decayed reversal would bind;
//   occupancy is therefore also read geometrically — a node whose extent
//   lies on the host's bytes, compared in the frame the host's own extent
//   is expressed in — and, for the whole-host form, by name — a node
//   whose own name positions it inside the host, extent or none. The
//   host's frame ancestors are not its occupants whatever their bytes
//   overlap; a disjoint sibling is not; and the region form is
//   byte-exact: the reclaimed tail is translated through the host's
//   extent and a file system ending before it does not violate. A host
//   whose own extent is absent has bytes that cannot be located, and
//   where nothing else is found it is returned itself: honest absence
//   fails closed here as at every other arm that needs a fact.
// Evidence: occupancy_is_read_by_geometry_and_by_name
#[test]
#[allow(clippy::too_many_lines)]
fn occupancy_is_read_by_geometry_and_by_name() {
    const MIB: u64 = 1 << 20;
    let dev = device(b"OCC");
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
    let sibling = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 20 * MIB,
    };
    let sibling_id = derive_id(&sibling).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: part_id,
        kind: super::naming::FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");
    let unlocated = NamingFields::FileSystem {
        host: part_id,
        kind: super::naming::FileSystemKind::Xfs,
        superblock_offset: 0,
    };
    let unlocated_id = derive_id(&unlocated).expect("derivable");
    let sibling_sig = NamingFields::BackingSignature {
        host: sibling_id,
        family: super::naming::SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let sibling_sig_id = derive_id(&sibling_sig).expect("derivable");
    let framed = |start, length| HostRange {
        host: dev_id,
        start,
        length,
    };
    let containment = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    // (extra nodes beyond the device, table and two partitions; whether
    // the partition under test carries an extent; extra extents)
    let world = |extra: Vec<NamingFields>,
                 part_located: bool,
                 extents: Vec<(super::naming::NodeId, HostRange)>| {
        let mut facts = Facts::default();
        facts.transports.insert(dev_id, TransportClass::Sata);
        facts.extents.insert(dev_id, framed(0, 1 << 30));
        facts.extents.insert(table_id, framed(0, MIB));
        if part_located {
            facts.extents.insert(part_id, framed(MIB, 10 * MIB));
        }
        facts.extents.insert(sibling_id, framed(20 * MIB, 10 * MIB));
        facts.extents.extend(extents);
        let mut nodes = vec![dev.clone(), table.clone(), part.clone(), sibling.clone()];
        nodes.extend(extra);
        // Deliberately no partition→content edges: the reading must not
        // depend on them.
        TopologySnapshot::assemble(
            SnapshotKind::Captured,
            false,
            nodes,
            vec![
                containment(dev_id, table_id),
                containment(table_id, part_id),
                containment(table_id, sibling_id),
            ],
            facts,
        )
        .expect("assembles")
    };
    let whole = Precondition::HostUnoccupied { host: part_id };
    let tail = |start, length| Precondition::RegionUnoccupied {
        region: HostRange {
            host: part_id,
            start,
            length,
        },
    };

    // Empty: the device's self-extent and the table are the partition's
    // frame ancestors, the sibling and its signature are disjoint.
    let empty = world(
        vec![sibling_sig.clone()],
        true,
        vec![(sibling_sig_id, framed(20 * MIB, 4096))],
    );
    assert_eq!(whole.violated_by(&empty), None);
    assert_eq!(tail(4 * MIB, 6 * MIB).violated_by(&empty), None);

    // A file system on the partition's bytes, framed on the device as the
    // partition is: found by geometry, whole-host and region alike, and
    // the region is byte-exact.
    let with_fs = world(vec![fs.clone()], true, vec![(fs_id, framed(MIB, 5 * MIB))]);
    assert_eq!(whole.violated_by(&with_fs), Some(fs_id));
    assert_eq!(tail(4 * MIB, 2 * MIB).violated_by(&with_fs), Some(fs_id));
    assert_eq!(
        tail(5 * MIB, 2 * MIB).violated_by(&with_fs),
        None,
        "the file system ends at 6 MiB, before the reclaimed tail begins"
    );

    // A file system that names the partition but declares no extent:
    // found by name for the whole host; a region cannot be located by a
    // name and is not.
    let unlocated_world = world(vec![unlocated.clone()], true, vec![]);
    assert_eq!(whole.violated_by(&unlocated_world), Some(unlocated_id));
    assert_eq!(tail(4 * MIB, 2 * MIB).violated_by(&unlocated_world), None);

    // The partition itself carries no extent: geometry cannot be read, the
    // name still can; and where nothing is found the host itself is
    // returned, because bytes that cannot be located are not known empty.
    let host_unlocated = world(vec![fs.clone()], false, vec![(fs_id, framed(MIB, 5 * MIB))]);
    assert_eq!(whole.violated_by(&host_unlocated), Some(fs_id));
    assert_eq!(
        tail(4 * MIB, 2 * MIB).violated_by(&host_unlocated),
        Some(part_id)
    );
    let host_unlocated_and_bare = world(vec![], false, vec![]);
    assert_eq!(whole.violated_by(&host_unlocated_and_bare), Some(part_id));
    assert_eq!(
        tail(4 * MIB, 2 * MIB).violated_by(&host_unlocated_and_bare),
        Some(part_id)
    );

    // ADR-0022's original reading is kept and still answers alone in one
    // corner: a range framed on the host itself, beyond the host's own
    // extent — bytes the host's declared extent understates. A backing
    // extent is the one kind the frame rule lets be framed anywhere; one
    // hosted elsewhere and framed on a bare device past its self-extent
    // names nothing inside the device and lies on none of its declared
    // bytes, and is an occupant of it all the same.
    let other = device(b"OCC-OTHER");
    let other_id = derive_id(&other).expect("derivable");
    let backing = NamingFields::BackingExtent {
        host: other_id,
        locator: super::naming::ExtentLocator::Range {
            start: 0,
            length: 4096,
        },
    };
    let backing_id = derive_id(&backing).expect("derivable");
    let mut facts = Facts::default();
    facts.transports.insert(dev_id, TransportClass::Sata);
    facts.extents.insert(dev_id, framed(0, 1 << 30));
    facts.extents.insert(backing_id, framed(2 << 30, 4096));
    let beyond = TopologySnapshot::assemble(
        SnapshotKind::Captured,
        false,
        vec![dev.clone(), other, backing],
        vec![],
        facts,
    )
    .expect("assembles");
    assert_eq!(
        Precondition::HostUnoccupied { host: dev_id }.violated_by(&beyond),
        Some(backing_id),
        "found by frame alone"
    );
}
