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

fn plan_over(snapshot: &TopologySnapshot, step: PlanStep) -> OperationPlan {
    OperationPlan::assemble(
        b"plan-1".to_vec(),
        1_700_000_000,
        snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![step],
    )
    .expect("assembles")
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
    let low = PlanStep::mutating(
        &snapshot,
        dev_id,
        StepRanges::default(),
        vec![],
        risk(Severity::Reversible),
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
    let plan = OperationPlan::assemble(
        b"plan-2".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        std::collections::BTreeMap::new(),
        vec![low, high],
    )
    .expect("assembles");
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

// Requirements: SAFE-003, MODEL-005
//   The authored-field rule at the boundary (ADR-0014, MODEL-005's
//   authoring set): where the helper-produced snapshot stamps a table
//   state, a plan identity claiming a different state never validates;
//   an agreeing identity round-trips.
// Evidence: a_client_authored_table_state_never_validates
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
        risk(Severity::Reversible),
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
    let plan = OperationPlan::assemble(
        b"plan-3".to_vec(),
        1_700_000_000,
        &snapshot,
        ValidityWindow {
            not_after: 1_700_086_400,
        },
        agreeing,
        vec![step],
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
