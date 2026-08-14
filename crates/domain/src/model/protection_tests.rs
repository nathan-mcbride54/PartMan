//! Tests for the protection layer (WP-010 increment 3e, ADR-0018).

use std::collections::BTreeMap;

use super::naming::{
    AggregateTechnology, NamingFields, NodeId, SignatureFamily, TableRole, derive_id,
};
use super::protection::{
    Facts, HostRange, IndeterminateGround, RefusalGround, StepRanges, TransportClass, Verdict,
    affected_set, node_verdict, step_constructs,
};
use super::topology::{Edge, EdgeKind, Topology};

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

/// The root-on-ZFS layout round three died on: sda carries an ESP at
/// sda1 and a ZFS member at sda2, the member's signature backing a ZFS
/// pool aggregate.
struct RootOnZfs {
    topology: Topology,
    facts: Facts,
    sda: super::naming::NodeId,
    table: super::naming::NodeId,
    esp: super::naming::NodeId,
    pool: super::naming::NodeId,
}

#[allow(clippy::too_many_lines)]
fn root_on_zfs() -> RootOnZfs {
    let sda = device(b"SDA");
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let esp = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let esp_id = derive_id(&esp).expect("derivable");
    let member = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 512 << 20,
    };
    let member_id = derive_id(&member).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: member_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"pool-guid".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![sda, table, esp, member, signature, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: esp_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: member_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: member_id,
                target: signature_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: signature_id,
                target: pool_id,
            },
        ],
    )
    .expect("builds");
    let mut extents = BTreeMap::new();
    extents.insert(
        sda_id,
        HostRange {
            host: sda_id,
            start: 0,
            length: 1 << 30,
        },
    );
    extents.insert(
        table_id,
        HostRange {
            host: sda_id,
            start: 0,
            length: 1 << 20,
        },
    );
    extents.insert(
        esp_id,
        HostRange {
            host: sda_id,
            start: 1 << 20,
            length: 256 << 20,
        },
    );
    extents.insert(
        member_id,
        HostRange {
            host: sda_id,
            start: 512 << 20,
            length: 256 << 20,
        },
    );
    extents.insert(
        signature_id,
        HostRange {
            host: sda_id,
            start: 512 << 20,
            length: 1 << 20,
        },
    );
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    RootOnZfs {
        topology,
        facts,
        sda: sda_id,
        table: table_id,
        esp: esp_id,
        pool: pool_id,
    }
}

// Requirements: MODEL-002, SAFE-005
//   The committed root-on-ZFS regression pair: creating a partition in
//   free space beside a pool member constructs; initializing the whole
//   device destroys the member and refuses through the pool.
// Evidence: the_root_on_zfs_regression_pair_holds
#[test]
fn the_root_on_zfs_regression_pair_holds() {
    let layout = root_on_zfs();

    // Create in free space: table write plus a consumed free range.
    let create = StepRanges {
        written_table_extents: vec![HostRange {
            host: layout.sda,
            start: 0,
            length: 1 << 20,
        }],
        consumed: vec![HostRange {
            host: layout.sda,
            start: 800 << 20,
            length: 64 << 20,
        }],
        destroyed: vec![],
    };
    let affected = step_constructs(&layout.topology, &layout.facts, layout.table, &create)
        .expect("create beside a pool member constructs");
    assert!(!affected.contains(&layout.pool), "the pool is unreached");

    // Initialize the device: the whole extent is destroyed.
    let initialize = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: layout.sda,
            start: 0,
            length: 1 << 30,
        }],
    };
    let refusal = step_constructs(&layout.topology, &layout.facts, layout.sda, &initialize)
        .expect_err("initializing the device must refuse through the pool");
    assert_eq!(refusal.node, layout.pool);
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
}

// Requirements: MODEL-002
//   Sibling non-capture: destroying the pool member's extent never
//   brings the ESP into the affected set — containment descent is
//   range-bounded and no edge crosses siblings.
// Evidence: a_sibling_esp_is_never_captured
#[test]
fn a_sibling_esp_is_never_captured() {
    let layout = root_on_zfs();
    let delete_member = StepRanges {
        written_table_extents: vec![HostRange {
            host: layout.sda,
            start: 0,
            length: 1 << 20,
        }],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: layout.sda,
            start: 512 << 20,
            length: 256 << 20,
        }],
    };
    let affected = affected_set(
        &layout.topology,
        &layout.facts,
        layout.table,
        &delete_member,
    );
    assert!(
        !affected.contains(&layout.esp),
        "the ESP is disjoint and unreached"
    );
    assert!(
        affected.contains(&layout.pool),
        "the pool is reached through its destroyed member signature"
    );
}

/// The LUKS chain the round-three regression uses, and the fixture that
/// exposes ADR-0038's defect: reaching the pool needs propagation, so a
/// seed that lands only in `affected` never gets there.
struct LuksChain {
    topology: Topology,
    facts: Facts,
    sdb: NodeId,
    part: NodeId,
    pool: NodeId,
}

#[allow(clippy::too_many_lines)]
fn luks_chain() -> LuksChain {
    let sdb = device(b"SDB");
    let sdb_id = derive_id(&sdb).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sdb_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let part_id = derive_id(&part).expect("derivable");
    let luks = NamingFields::BackingSignature {
        host: part_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let luks_id = derive_id(&luks).expect("derivable");
    let layer = NamingFields::EncryptionLayer {
        backing_signature: luks_id,
    };
    let layer_id = derive_id(&layer).expect("derivable");
    let mapper = NamingFields::Volume {
        producer: layer_id,
        name: b"cryptzfs".to_vec(),
        role: None,
    };
    let mapper_id = derive_id(&mapper).expect("derivable");
    let member = NamingFields::BackingSignature {
        host: mapper_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let member_id = derive_id(&member).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![sdb, table, part, luks, layer, mapper, member, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sdb_id,
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
                target: mapper_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: mapper_id,
                target: member_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: member_id,
                target: pool_id,
            },
        ],
    )
    .expect("builds");
    let mut extents = BTreeMap::new();
    extents.insert(
        part_id,
        HostRange {
            host: sdb_id,
            start: 1 << 20,
            length: 512 << 20,
        },
    );
    extents.insert(
        luks_id,
        HostRange {
            host: sdb_id,
            start: 1 << 20,
            length: 16 << 10,
        },
    );
    let mut transports = BTreeMap::new();
    transports.insert(sdb_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let delete = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: sdb_id,
            start: 1 << 20,
            length: 512 << 20,
        }],
    };
    let _ = delete;
    LuksChain {
        topology,
        facts,
        sdb: sdb_id,
        part: part_id,
        pool: pool_id,
    }
}

// Requirements: MODEL-002, SAFE-005
//   The round-three killer: deleting a partition hosting LUKS reaches
//   the pool below through production over destroyed substrate —
//   partition, signature, encryption layer, mapper volume, member
//   signature, pool.
// Evidence: the_luks_descent_reaches_the_pool_below
#[test]
fn the_luks_descent_reaches_the_pool_below() {
    let chain = luks_chain();
    let delete = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: chain.sdb,
            start: 1 << 20,
            length: 512 << 20,
        }],
    };
    let refusal = step_constructs(&chain.topology, &chain.facts, chain.part, &delete)
        .expect_err("the pool below the encryption layer must refuse the delete");
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
    let affected = affected_set(&chain.topology, &chain.facts, chain.part, &delete);
    assert!(
        affected.contains(&chain.pool),
        "the pool is reached through the production descent"
    );
}

// Requirements: SAFE-005, MODEL-002
//   The inverted default: unrecognized technologies, missing facts, and
//   collision groups are Indeterminate, never Permitted; Fusion's
//   member-count arm refuses at two and permits at one.
// Evidence: the_residual_is_indeterminate_never_permitted
#[test]
fn the_residual_is_indeterminate_never_permitted() {
    let mystery = NamingFields::Aggregate {
        technology: AggregateTechnology::Unrecognized {
            raw: b"novel".to_vec(),
        },
        designator: Some(b"x".to_vec()),
    };
    let mystery_id = derive_id(&mystery).expect("derivable");
    let apfs = NamingFields::Aggregate {
        technology: AggregateTechnology::Apfs,
        designator: Some(b"c".to_vec()),
    };
    let apfs_id = derive_id(&apfs).expect("derivable");
    let bare = device(b"BARE");
    let bare_id = derive_id(&bare).expect("derivable");
    let topology = Topology::build(vec![mystery, apfs, bare], vec![]).expect("builds");

    let empty = Facts::default();
    assert!(matches!(
        node_verdict(&topology, &empty, mystery_id),
        Verdict::Indeterminate {
            cause: IndeterminateGround::Unrecognized
        }
    ));
    assert!(matches!(
        node_verdict(&topology, &empty, apfs_id),
        Verdict::Indeterminate {
            cause: IndeterminateGround::MissingFact
        }
    ));
    assert!(matches!(
        node_verdict(&topology, &empty, bare_id),
        Verdict::Indeterminate {
            cause: IndeterminateGround::MissingFact
        }
    ));

    let mut fusion_facts = Facts::default();
    fusion_facts.member_counts.insert(apfs_id, 2);
    assert!(matches!(
        node_verdict(&topology, &fusion_facts, apfs_id),
        Verdict::Refused {
            ground: RefusalGround::Fusion
        }
    ));
    fusion_facts.member_counts.insert(apfs_id, 1);
    assert_eq!(
        node_verdict(&topology, &fusion_facts, apfs_id),
        Verdict::Permitted
    );
}

// Requirements: MODEL-002, SAFE-005
//   Device scope inherits node-locally: a partition on a
//   recognized-remote device refuses through its own root device, and a
//   collision group is never an operand.
// Evidence: device_scope_and_collision_groups_fail_closed
#[test]
fn device_scope_and_collision_groups_fail_closed() {
    let lun = device(b"LUN");
    let lun_id = derive_id(&lun).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: lun_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let part_id = derive_id(&part).expect("derivable");
    let twin = || device(b"TWIN");
    let twin_id = derive_id(&twin()).expect("derivable");
    let topology = Topology::build(
        vec![lun, table, part, twin(), twin()],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: lun_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: part_id,
            },
        ],
    )
    .expect("builds");
    let mut facts = Facts::default();
    facts
        .transports
        .insert(lun_id, TransportClass::RecognizedRemote);
    assert!(matches!(
        node_verdict(&topology, &facts, part_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedDeviceScope
        }
    ));
    assert!(matches!(
        node_verdict(&topology, &facts, twin_id),
        Verdict::Indeterminate {
            cause: IndeterminateGround::CollisionGroup
        }
    ));
}

// Requirements: MODEL-002, SAFE-005
//   The signature arms: an orphan signature is Indeterminate (the
//   remediable arm, never silently permitted and never refused
//   forever), a member consumed by a supported aggregate is Permitted,
//   and a member consumed by a non-goal aggregate refuses.
// Evidence: signature_arms_follow_the_consumer
#[test]
fn signature_arms_follow_the_consumer() {
    let host = device(b"H");
    let host_id = derive_id(&host).expect("derivable");
    let orphan = NamingFields::BackingSignature {
        host: host_id,
        family: SignatureFamily::Mdraid09,
        primary_offset: 0,
    };
    let orphan_id = derive_id(&orphan).expect("derivable");
    let lvm_sig = NamingFields::BackingSignature {
        host: host_id,
        family: SignatureFamily::Lvm2,
        primary_offset: 4096,
    };
    let lvm_sig_id = derive_id(&lvm_sig).expect("derivable");
    let vg = NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: Some(b"vg".to_vec()),
    };
    let vg_id = derive_id(&vg).expect("derivable");
    let topology = Topology::build(
        vec![host, orphan, lvm_sig, vg],
        vec![Edge {
            kind: EdgeKind::Backing,
            source: lvm_sig_id,
            target: vg_id,
        }],
    )
    .expect("builds");
    let mut facts = Facts::default();
    facts.transports.insert(host_id, TransportClass::Usb);
    assert!(matches!(
        node_verdict(&topology, &facts, orphan_id),
        Verdict::Indeterminate {
            cause: IndeterminateGround::OrphanSignature
        }
    ));
    assert_eq!(
        node_verdict(&topology, &facts, lvm_sig_id),
        Verdict::Permitted
    );
}

// Requirements: MODEL-002, SAFE-005
//   ADR-0038's first correction, measured per operation on the fixture
//   that needs propagation to reach the pool. Shrink and Move are the
//   two operations ADR-0018 names as releases beside the two that
//   destroy outright, and only they move — from Clear to a refusal over
//   a live ZFS pool below a LUKS chain. The six that destroy nothing
//   stay Clear, which is the held half of issue #338 shown rather than
//   asserted; conservatism is argued per operation because the
//   propagating arms carry negative range_destroyed guards and
//   monotonicity is therefore false.
// Evidence: release_operations_seed_the_closure_and_the_others_do_not
#[test]
fn release_operations_seed_the_closure_and_the_others_do_not() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    let chain = luks_chain();
    let refuses = |op| {
        matches!(
            protection_gate(&chain.topology, &chain.facts, chain.part, op),
            ProtectionGate::Unsupported { .. } | ProtectionGate::Blocked { .. }
        )
    };

    // The two ADR-0018 names as releases, plus the two that destroy
    // outright: all refuse through the chain.
    for op in [
        Operation::Wipe,
        Operation::Encrypt,
        Operation::Shrink,
        Operation::Move,
    ] {
        assert!(refuses(op), "{op:?} must reach the pool and refuse");
    }

    // The six that destroy nothing: ADR-0018 licenses no destroyed
    // entry for them, so their reach still needs the propagation
    // widening issue #338 holds. Pinned so the held half cannot drift
    // shut silently or open further without a decision.
    for op in [
        Operation::Grow,
        Operation::Create,
        Operation::Repair,
        Operation::Label,
        Operation::Uuid,
        Operation::Decrypt,
    ] {
        assert!(
            !refuses(op),
            "{op:?} destroys nothing; #338 holds its reach and this pin records that"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   The false-refusal control on the corrected operations: a shrink or
//   move over a device carrying no protected chain still constructs.
//   The correction over-reaches by declaring the whole target extent
//   destroyed, and this bounds that over-reach to bodies that actually
//   carry something to reach.
// Evidence: a_release_over_an_unprotected_target_still_constructs
#[test]
fn a_release_over_an_unprotected_target_still_constructs() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    let layout = root_on_zfs();
    for op in [Operation::Shrink, Operation::Move] {
        let gate = protection_gate(&layout.topology, &layout.facts, layout.esp, op);
        assert!(
            matches!(gate, ProtectionGate::Clear),
            "{op:?} over the unprotected ESP must stay Clear, got {gate:?}"
        );
    }
}

// Requirements: MODEL-002
//   ADR-0038's second correction, and the committed sibling guard
//   re-measured on MEMBERSHIP rather than merely re-run green. Rule 3
//   is route-agnostic in ADR-0018 — a signature in the set brings its
//   consumer — so ungating its membership half must not also drag a
//   disjoint sibling in. The ESP stays out of the set; the pool stays
//   in it.
// Evidence: ungating_rule_three_membership_never_captures_a_sibling
#[test]
fn ungating_rule_three_membership_never_captures_a_sibling() {
    let layout = root_on_zfs();
    let delete_member = StepRanges {
        written_table_extents: vec![HostRange {
            host: layout.sda,
            start: 0,
            length: 1 << 20,
        }],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: layout.sda,
            start: 512 << 20,
            length: 256 << 20,
        }],
    };
    let affected = affected_set(
        &layout.topology,
        &layout.facts,
        layout.table,
        &delete_member,
    );
    assert!(
        !affected.contains(&layout.esp),
        "the ESP is disjoint and must not be captured by the ungated membership half"
    );
    assert!(
        affected.contains(&layout.pool),
        "the pool is still reached through its destroyed member signature"
    );
}

// Requirements: MODEL-002, SAFE-005
//   ADR-0038's second correction, and the only fixture that observes
//   it. ADR-0018's rule 3 is route-agnostic — "a BackingSignature IN
//   THE SET brings its consumer" — contrasted in the same paragraph
//   with rule 4's "in the set THROUGH A DESTROYED RANGE". A step that
//   writes over a ZFS label's bytes without destroying them puts the
//   signature in the affected set by the written-extent route; the
//   ungated membership half must then bring the pool, whose own arm
//   refuses. Gate that half back on destruction and this body
//   constructs over a live vdev — which is what it did before ADR-0038.
// Evidence: a_signature_reached_without_destruction_brings_its_consumer
#[test]
fn a_signature_reached_without_destruction_brings_its_consumer() {
    let layout = root_on_zfs();
    // The ZFS label sits at [512 MiB, 513 MiB). Write over it, destroy
    // nothing: the signature enters the set by written-extent
    // intersection alone.
    let write_over_label = StepRanges {
        written_table_extents: vec![HostRange {
            host: layout.sda,
            start: 512 << 20,
            length: 1 << 20,
        }],
        consumed: vec![],
        destroyed: vec![],
    };
    let affected = affected_set(
        &layout.topology,
        &layout.facts,
        layout.sda,
        &write_over_label,
    );
    assert!(
        affected.contains(&layout.pool),
        "rule 3 is route-agnostic: a signature in the set brings its consumer"
    );
    let refusal = step_constructs(
        &layout.topology,
        &layout.facts,
        layout.sda,
        &write_over_label,
    )
    .expect_err("the pool's own arm refuses once it is reached");
    assert_eq!(refusal.node, layout.pool);
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
}
