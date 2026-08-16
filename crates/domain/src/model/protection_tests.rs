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
    member: super::naming::NodeId,
    signature: super::naming::NodeId,
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
        member: member_id,
        signature: signature_id,
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
    mapper: NodeId,
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
        mapper: mapper_id,
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
//   ADR-0038's per-operation table, re-measured under ADR-0039's reach.
//   The four operations that destroy or release still refuse. The six
//   that destroy nothing refuse now too, and by a different route: they
//   declare no destroyed range, so nothing seeds the destruction
//   classes, and they reach the pool because a mutating step reaches
//   the content its target carries. ADR-0038 pinned those six as Clear
//   to keep the held half of issue #338 visible; the hold is over, and
//   the pin is inverted rather than deleted, so the change of answer is
//   itself the thing under test.
// Evidence: every_mutating_operation_reaches_the_content_its_target_carries
#[test]
fn every_mutating_operation_reaches_the_content_its_target_carries() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    let chain = luks_chain();
    let refuses = |op| {
        matches!(
            protection_gate(&chain.topology, &chain.facts, chain.part, op),
            ProtectionGate::Unsupported { .. } | ProtectionGate::Blocked { .. }
        )
    };
    for op in [
        Operation::Wipe,
        Operation::Encrypt,
        Operation::Shrink,
        Operation::Move,
        Operation::Grow,
        Operation::Create,
        Operation::Repair,
        Operation::Label,
        Operation::Uuid,
        Operation::Decrypt,
    ] {
        assert!(
            refuses(op),
            "{op:?} over a partition carrying a LUKS-wrapped ZFS vdev must reach the pool"
        );
    }

    // Source-class operations are never suppressed by a verdict, reach
    // or no reach (ADR-0018's operation classes).
    for op in [
        Operation::Detect,
        Operation::Read,
        Operation::Check,
        Operation::Copy,
    ] {
        assert_eq!(
            protection_gate(&chain.topology, &chain.facts, chain.part, op),
            ProtectionGate::Clear,
            "{op:?} is source class and must stay Clear"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   An extent on a kind the body format forbids one on must not steer
//   reach. `snapshot.rs` refuses an `extent_host` on an aggregate,
//   volume, encryption layer or multipath node, but `assemble` applies
//   no such rule, so an in-process caller — which is what the planner
//   and the capability engine are — can hold a snapshot that could never
//   round-trip. Here the mapper volume carries a device-framed extent it
//   may not have, while the ZFS member signature it hosts carries none.
//   Weigh the unlawful fact and the containment arm stops at the mapper,
//   leaving a live pool unconsulted; ignore it, as the closure and the
//   decode path now do through one shared predicate, and the descent
//   runs. The asymmetry itself is issue #349.
// Evidence: an_extent_the_format_forbids_never_steers_reach
#[test]
fn an_extent_the_format_forbids_never_steers_reach() {
    let chain = luks_chain();
    let mut facts = chain.facts.clone();
    facts.extents.insert(
        chain.mapper,
        HostRange {
            host: chain.sdb,
            start: 17 << 20,
            length: 400 << 20,
        },
    );
    let erase_the_header = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: chain.sdb,
            start: 1 << 20,
            length: 16 << 10,
        }],
    };
    let affected = affected_set(&chain.topology, &facts, chain.part, &erase_the_header);
    assert!(
        affected.contains(&chain.pool),
        "an extent the body format forbids must not stop the descent"
    );
    let refusal = step_constructs(&chain.topology, &facts, chain.part, &erase_the_header)
        .expect_err("erasing the LUKS header over a live pool must refuse");
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
}

// Requirements: MODEL-002, SAFE-005
//   Defect (b) of issue #338, at the closure. A shrink destroys a
//   sub-range of its target, and the ZFS label at the target's head lies
//   outside that sub-range: the label's bytes survive, the vdev they
//   describe does not. Before ADR-0039 the affected set was the target
//   and its device, the pool was unreached, and the step constructed
//   over 128 MiB of a live vdev.
// Evidence: a_partial_destruction_reaches_the_content_it_truncates
#[test]
fn a_partial_destruction_reaches_the_content_it_truncates() {
    let layout = root_on_zfs();
    let freed_tail = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: layout.sda,
            start: 640 << 20,
            length: 128 << 20,
        }],
    };
    let affected = affected_set(&layout.topology, &layout.facts, layout.member, &freed_tail);
    assert!(
        affected.contains(&layout.pool),
        "the pool is reached through the label its truncated member carries"
    );
    assert!(
        !affected.contains(&layout.esp),
        "the disjoint sibling is still not captured"
    );
    let refusal = step_constructs(&layout.topology, &layout.facts, layout.member, &freed_tail)
        .expect_err("a partial shrink over a live vdev must refuse");
    assert_eq!(refusal.node, layout.pool);
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
}

// Requirements: MODEL-002, SAFE-005
//   The bound reads declared extents, and declared extents are authored
//   body content, so it must never be able to REMOVE reach. A moved
//   frame, a ghost host, a zero length and a saturating length all still
//   reach the pool. None of these values changes a node id — a
//   BackingSignature hashes its own `host` field, which is not the
//   extent's frame — so nothing upstream can tell these bodies apart.
// Evidence: a_forged_extent_can_never_shrink_the_closure
#[test]
fn a_forged_extent_can_never_shrink_the_closure() {
    let honest = root_on_zfs();
    let destroyed = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: honest.sda,
            start: 640 << 20,
            length: 128 << 20,
        }],
    };
    let label = *honest
        .facts
        .extents
        .get(&honest.signature)
        .expect("the fixture declares the label's extent");
    let forgeries = [
        (
            "framed on a sibling it does not sit in",
            HostRange {
                host: honest.esp,
                ..label
            },
        ),
        (
            "framed on a node that carries no bytes",
            HostRange {
                host: honest.pool,
                ..label
            },
        ),
        ("zero length", HostRange { length: 0, ..label }),
        (
            "saturating length",
            HostRange {
                length: u64::MAX,
                ..label
            },
        ),
    ];
    for (name, forged) in forgeries {
        let mut facts = honest.facts.clone();
        facts.extents.insert(honest.signature, forged);
        let affected = affected_set(&honest.topology, &facts, honest.member, &destroyed);
        assert!(
            affected.contains(&honest.pool),
            "a forged extent must not remove reach: {name}"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   The false-refusal controls for the widened reach, on a disk carrying
//   nothing protected. A device's extent is its own address space, so
//   every range on the disk lies inside it and descent out of one would
//   capture every sibling. Two shapes caught exactly that during the
//   round: an end-anchored stale mdraid superblock hosted by the device,
//   and a sibling partition carrying no extent fact at all. Deleting or
//   shrinking one partition must reach neither.
// Evidence: an_ordinary_disk_keeps_its_siblings_out_of_the_set
#[test]
#[allow(clippy::too_many_lines)]
fn an_ordinary_disk_keeps_its_siblings_out_of_the_set() {
    let sdz = device(b"SDZ");
    let sdz_id = derive_id(&sdz).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sdz_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let esp = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let esp_id = derive_id(&esp).expect("derivable");
    let data = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 512 << 20,
    };
    let data_id = derive_id(&data).expect("derivable");
    // An unassembled mdraid superblock at the very end of the disk,
    // hosted by the device itself: the shape `mdadm --zero-superblock`
    // exists for, and an orphan signature, so its arm is Indeterminate.
    let stale = NamingFields::BackingSignature {
        host: sdz_id,
        family: SignatureFamily::Mdraid09,
        primary_offset: 0,
    };
    let stale_id = derive_id(&stale).expect("derivable");
    let topology = Topology::build(
        vec![sdz, table, esp, data, stale],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sdz_id,
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
                target: data_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: sdz_id,
                target: stale_id,
            },
        ],
    )
    .expect("builds");
    let mut extents = BTreeMap::new();
    extents.insert(
        sdz_id,
        HostRange {
            host: sdz_id,
            start: 0,
            length: 1 << 30,
        },
    );
    extents.insert(
        table_id,
        HostRange {
            host: sdz_id,
            start: 0,
            length: 1 << 20,
        },
    );
    // The ESP carries no extent fact: the byte scan cannot judge it, and
    // the closure must not capture it on that account.
    extents.insert(
        data_id,
        HostRange {
            host: sdz_id,
            start: 512 << 20,
            length: 256 << 20,
        },
    );
    extents.insert(
        stale_id,
        HostRange {
            host: sdz_id,
            start: (1 << 30) - (64 << 10),
            length: 64 << 10,
        },
    );
    let mut transports = BTreeMap::new();
    transports.insert(sdz_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };

    let delete_data = StepRanges {
        written_table_extents: vec![HostRange {
            host: sdz_id,
            start: 0,
            length: 1 << 20,
        }],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: sdz_id,
            start: 512 << 20,
            length: 256 << 20,
        }],
    };
    let affected = step_constructs(&topology, &facts, table_id, &delete_data)
        .expect("deleting a partition on an ordinary disk constructs");
    assert!(
        !affected.contains(&stale_id),
        "a device's self-extent must not carry descent into a stale superblock at the far end"
    );
    assert!(
        !affected.contains(&esp_id),
        "a sibling that merely lacks an extent fact must not be captured"
    );

    let shrink_data = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: sdz_id,
            start: 640 << 20,
            length: 128 << 20,
        }],
    };
    let affected = step_constructs(&topology, &facts, data_id, &shrink_data)
        .expect("shrinking a partition on an ordinary disk constructs");
    assert!(!affected.contains(&stale_id));
    assert!(!affected.contains(&esp_id));
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

// Requirements: MODEL-002, SAFE-005
//   Issue #348's measured hole, and why ADR-0038's release entry is
//   load-bearing after ADR-0039 rather than superseded by it. On a
//   partition target carried-content reach alone refuses: the target
//   seeds the set and descent runs from it. On a whole-disk target it
//   does not — `descends_into` refuses a self-framed extent as a
//   descent source, which is what stops a disk's own extent capturing
//   its siblings — so reach there is entirely range-driven and the
//   release entry is the only thing refusing. Nothing committed said
//   so: deleting `Move`'s canonical entry left the whole domain suite
//   green while `gate(Move, sda)` fell from `Unsupported{Zfs}` to
//   `Clear` over a live pool, the false-`Clear` direction that
//   `a_release_over_an_unprotected_target_still_constructs` does not
//   cover. Both halves are asserted deliberately: the gate outcome
//   alone survives moving `Move` into the written-extents arm, which
//   changes the entry's class without changing any verdict, while the
//   range-class assertion pins `Move` as a release under ADR-0018's
//   own effect table — the classification issue #348 questioned.
// Evidence: a_release_over_a_whole_disk_reaches_the_aggregate_it_carries
#[test]
fn a_release_over_a_whole_disk_reaches_the_aggregate_it_carries() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    let layout = root_on_zfs();
    for op in [Operation::Move, Operation::Shrink] {
        let ranges = canonical_ranges(op, layout.sda, &layout.facts);
        assert!(
            ranges.destroyed.len() == 1 && ranges.written_table_extents.is_empty(),
            "{op:?} is a release under ADR-0018's effect table and must seed the \
             destroyed class, got {ranges:?}"
        );
        assert!(
            affected_set(&layout.topology, &layout.facts, layout.sda, &ranges)
                .contains(&layout.pool),
            "{op:?}'s canonical entry must reach the pool the whole disk carries"
        );
        let gate = protection_gate(&layout.topology, &layout.facts, layout.sda, op);
        assert!(
            matches!(
                gate,
                ProtectionGate::Unsupported {
                    ground: RefusalGround::Zfs
                }
            ),
            "{op:?} over a whole disk carrying a live pool must refuse, got {gate:?}"
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

/// A device whose serial is ground until its derived address sorts below
/// `ceiling` — the attacker's own move, since `derive_id` hashes the
/// serial and the edge order the walks followed is that address's order.
fn device_sorting_below(ceiling: NodeId) -> (NamingFields, NodeId) {
    for attempt in 0..200_000_u32 {
        let candidate = device(format!("GROUND-{attempt}").as_bytes());
        let id = derive_id(&candidate).expect("derivable");
        if id < ceiling {
            return (candidate, id);
        }
    }
    panic!("no ground serial sorted below the ceiling");
}

// Requirements: MODEL-002, SAFE-005
//   Issue #355's device-scope vector: a node's inherited device-scope
//   verdict is the worst over every containment root above it, never
//   whichever parent's derived address sorts first. A file system on a
//   recognized-remote device keeps that device's refusal when a second,
//   lawful containment edge from a local decoy device is added — the
//   edge the first-match walk would have followed, since the decoy's
//   serial is ground until its address sorts below the remote's.
// Evidence: a_decoy_containment_parent_never_displaces_a_device_refusal
#[test]
fn a_decoy_containment_parent_never_displaces_a_device_refusal() {
    let remote = device(b"REMOTE-HOST");
    let remote_id = derive_id(&remote).expect("derivable");
    let (local, local_id) = device_sorting_below(remote_id);
    assert!(local_id < remote_id, "the decoy wins the first-match walk");
    let file_system = NamingFields::FileSystem {
        host: remote_id,
        kind: super::naming::FileSystemKind::Ext4,
        superblock_offset: 0,
    };
    let file_system_id = derive_id(&file_system).expect("derivable");
    let mut transports = BTreeMap::new();
    transports.insert(remote_id, TransportClass::RecognizedRemote);
    transports.insert(local_id, TransportClass::Sata);
    let facts = Facts {
        extents: BTreeMap::new(),
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };

    let honest = Topology::build(
        vec![remote.clone(), local.clone(), file_system.clone()],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: remote_id,
            target: file_system_id,
        }],
    )
    .expect("builds");
    assert_eq!(
        node_verdict(&honest, &facts, file_system_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedDeviceScope
        },
        "the honest body inherits the remote device's refusal"
    );

    let with_decoy = Topology::build(
        vec![remote, local, file_system],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: remote_id,
                target: file_system_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: local_id,
                target: file_system_id,
            },
        ],
    )
    .expect("the two-parent body builds");
    assert_eq!(
        node_verdict(&with_decoy, &facts, file_system_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedDeviceScope
        },
        "an added parent may add refusal, never remove one"
    );
}

// Requirements: CAP-003, SAFE-005
//   Issue #355 at the surface the clients read: the decoy containment
//   parent leaves every mutating gate unsupported. The gate is where the
//   bypass paid out — ten Clear answers over a remote-transport host —
//   so the property is pinned at the gate and not only at the verdict.
// Evidence: a_decoy_containment_parent_never_clears_a_gate
#[test]
fn a_decoy_containment_parent_never_clears_a_gate() {
    use super::capability::{Operation, OperationClass, ProtectionGate, protection_gate};

    let remote = device(b"REMOTE-HOST");
    let remote_id = derive_id(&remote).expect("derivable");
    let (local, local_id) = device_sorting_below(remote_id);
    let file_system = NamingFields::FileSystem {
        host: remote_id,
        kind: super::naming::FileSystemKind::Ext4,
        superblock_offset: 0,
    };
    let file_system_id = derive_id(&file_system).expect("derivable");
    let mut transports = BTreeMap::new();
    transports.insert(remote_id, TransportClass::RecognizedRemote);
    transports.insert(local_id, TransportClass::Sata);
    let facts = Facts {
        extents: BTreeMap::new(),
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let with_decoy = Topology::build(
        vec![remote, local, file_system],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: remote_id,
                target: file_system_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: local_id,
                target: file_system_id,
            },
        ],
    )
    .expect("builds");
    for operation in Operation::all() {
        if operation.class() != OperationClass::Mutating {
            continue;
        }
        assert_eq!(
            protection_gate(&with_decoy, &facts, file_system_id, *operation),
            ProtectionGate::Unsupported {
                ground: RefusalGround::InheritedDeviceScope
            },
            "{operation:?} stays unsupported behind a decoy parent"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   Issue #355's producer vector: a produced node inherits the worst of
//   every producer the body declares. A volume produced by a live ZFS
//   pool keeps that refusal when a lawful Production edge from a benign
//   encryption layer is added — the pool's designator being ground so
//   the layer wins the first-match choice. This is the vector that
//   flipped all ten gates.
// Evidence: a_decoy_producer_never_displaces_a_pools_refusal
#[test]
#[allow(clippy::too_many_lines)]
fn a_decoy_producer_never_displaces_a_pools_refusal() {
    use super::capability::{Operation, OperationClass, ProtectionGate, protection_gate};

    let host = device(b"PRODUCER-HOST");
    let host_id = derive_id(&host).expect("derivable");
    let luks = NamingFields::BackingSignature {
        host: host_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let luks_id = derive_id(&luks).expect("derivable");
    let layer = NamingFields::EncryptionLayer {
        backing_signature: luks_id,
    };
    let layer_id = derive_id(&layer).expect("derivable");
    // Grind the pool's designator until it sorts after the benign layer,
    // so a first-match producer choice would follow the layer.
    let mut ground = None;
    for attempt in 0..200_000_u32 {
        let candidate = NamingFields::Aggregate {
            technology: AggregateTechnology::Zfs,
            designator: Some(format!("tank-{attempt}").into_bytes()),
        };
        let id = derive_id(&candidate).expect("derivable");
        if id > layer_id {
            ground = Some((candidate, id));
            break;
        }
    }
    let (pool, pool_id) = ground.expect("a pool sorting after the layer exists");
    let volume = NamingFields::Volume {
        producer: pool_id,
        name: b"vol0".to_vec(),
        role: None,
    };
    let volume_id = derive_id(&volume).expect("derivable");
    let mut transports = BTreeMap::new();
    transports.insert(host_id, TransportClass::Sata);
    let facts = Facts {
        extents: BTreeMap::new(),
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let edges = vec![
        Edge {
            kind: EdgeKind::Containment,
            source: host_id,
            target: luks_id,
        },
        Edge {
            kind: EdgeKind::Backing,
            source: luks_id,
            target: layer_id,
        },
        Edge {
            kind: EdgeKind::Production,
            source: pool_id,
            target: volume_id,
        },
    ];
    let honest = Topology::build(
        vec![
            host.clone(),
            luks.clone(),
            layer.clone(),
            pool.clone(),
            volume.clone(),
        ],
        edges.clone(),
    )
    .expect("builds");
    assert_eq!(
        node_verdict(&honest, &facts, volume_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedFromConsumerOrProducer
        },
        "the honest volume inherits the pool's refusal"
    );

    let mut decoyed = edges;
    decoyed.push(Edge {
        kind: EdgeKind::Production,
        source: layer_id,
        target: volume_id,
    });
    let with_decoy = Topology::build(vec![host, luks, layer, pool, volume], decoyed)
        .expect("the two-producer body builds");
    assert_eq!(
        node_verdict(&with_decoy, &facts, volume_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedFromConsumerOrProducer
        },
        "an added producer may add refusal, never remove one"
    );
    for operation in Operation::all() {
        if operation.class() != OperationClass::Mutating {
            continue;
        }
        assert_eq!(
            protection_gate(&with_decoy, &facts, volume_id, *operation),
            ProtectionGate::Unsupported {
                ground: RefusalGround::InheritedFromConsumerOrProducer
            },
            "{operation:?} stays unsupported behind a decoy producer"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   Issue #355's consumer vector, and the one the closure already
//   covered: a signature's own arm follows the worst of its consumers.
//   Membership carries unbounded in-degree (MODEL-002), so a second
//   consumer is lawful rather than adversarial by construction — a
//   signature backing both an aggregate and an encryption layer is a
//   representable observation, and the arm must not read only whichever
//   consumer sorts first.
// Evidence: a_signatures_arm_follows_the_worst_of_its_consumers
#[test]
fn a_signatures_arm_follows_the_worst_of_its_consumers() {
    let host = device(b"CONSUMER-HOST");
    let host_id = derive_id(&host).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: host_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let layer = NamingFields::EncryptionLayer {
        backing_signature: signature_id,
    };
    let layer_id = derive_id(&layer).expect("derivable");
    let mut ground = None;
    for attempt in 0..200_000_u32 {
        let candidate = NamingFields::Aggregate {
            technology: AggregateTechnology::Zfs,
            designator: Some(format!("pool-{attempt}").into_bytes()),
        };
        let id = derive_id(&candidate).expect("derivable");
        if id > layer_id {
            ground = Some((candidate, id));
            break;
        }
    }
    let (pool, pool_id) = ground.expect("a pool sorting after the layer exists");
    let mut transports = BTreeMap::new();
    transports.insert(host_id, TransportClass::Sata);
    let facts = Facts {
        extents: BTreeMap::new(),
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let topology = Topology::build(
        vec![host, signature, layer, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
                target: signature_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: signature_id,
                target: pool_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: signature_id,
                target: layer_id,
            },
        ],
    )
    .expect("the two-consumer body builds");
    assert_eq!(
        node_verdict(&topology, &facts, signature_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedFromConsumerOrProducer
        },
        "the ZFS consumer decides the arm, whichever consumer sorts first"
    );
}

// Requirements: MODEL-002
//   The multiplicity fold is conservative in one direction only: a body
//   presenting one ancestry, one producer and one consumer answers
//   exactly as it did before the fold, which is what keeps the change a
//   defect fix rather than a policy change. Held over the committed
//   root-on-ZFS layout, node by node.
// Evidence: single_ancestry_bodies_answer_exactly_as_before
#[test]
fn single_ancestry_bodies_answer_exactly_as_before() {
    let layout = root_on_zfs();
    for (node, expected) in [
        (layout.sda, Verdict::Permitted),
        (layout.table, Verdict::Permitted),
        (layout.esp, Verdict::Permitted),
        (layout.member, Verdict::Permitted),
        (
            layout.signature,
            Verdict::Refused {
                ground: RefusalGround::InheritedFromConsumerOrProducer,
            },
        ),
        (
            layout.pool,
            Verdict::Refused {
                ground: RefusalGround::Zfs,
            },
        ),
    ] {
        assert_eq!(
            node_verdict(&layout.topology, &layout.facts, node),
            expected,
            "single-ancestry verdicts are unmoved by the fold"
        );
    }
}

// ---------------------------------------------------------------------
// The overlapping-geometry fixture the issue-347 round-2 panel asked to
// be committed before any candidate in that family is measured again:
// a BIOS-booting GPT disk whose bios_grub entry sits at LBA 34, *inside*
// the first MiB the table's own extent declares. Every other committed
// fixture has `table.start + table.length == p1.start` exactly, so
// nothing in the population could see the sibling-capture shape that
// killed that round's candidate. Root on ZFS beside an ESP, as before.
// ---------------------------------------------------------------------

const MIB: u64 = 1 << 20;

struct BiosBootGpt {
    topology: Topology,
    facts: Facts,
    nodes: Vec<NamingFields>,
    edges: Vec<Edge>,
    sda: NodeId,
    table: NodeId,
    boot: NodeId,
    esp: NodeId,
    member: NodeId,
    signature: NodeId,
    pool: NodeId,
}

#[allow(clippy::too_many_lines)]
fn bios_boot_gpt() -> BiosBootGpt {
    let sda = device(b"SDA");
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    // Sectors 34..2047: [17408, 1 MiB).
    let boot = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 17408,
    };
    let boot_id = derive_id(&boot).expect("derivable");
    let esp = NamingFields::Partition {
        parent_table: table_id,
        start_offset: MIB,
    };
    let esp_id = derive_id(&esp).expect("derivable");
    let member = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 512 * MIB,
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
    let nodes = vec![sda, table, boot, esp, member, signature, pool];
    let edges = vec![
        Edge {
            kind: EdgeKind::Containment,
            source: sda_id,
            target: table_id,
        },
        Edge {
            kind: EdgeKind::Containment,
            source: table_id,
            target: boot_id,
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
    ];
    let topology = Topology::build(nodes.clone(), edges.clone()).expect("builds");
    let mut extents = BTreeMap::new();
    let host = |start, length| HostRange {
        host: sda_id,
        start,
        length,
    };
    extents.insert(sda_id, host(0, 1 << 30));
    extents.insert(table_id, host(0, MIB));
    extents.insert(boot_id, host(17408, MIB - 17408));
    extents.insert(esp_id, host(MIB, 256 * MIB));
    extents.insert(member_id, host(512 * MIB, 256 * MIB));
    extents.insert(signature_id, host(512 * MIB, MIB));
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    BiosBootGpt {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        nodes,
        edges,
        sda: sda_id,
        table: table_id,
        boot: boot_id,
        esp: esp_id,
        member: member_id,
        signature: signature_id,
        pool: pool_id,
    }
}

// Requirements: MODEL-002, MODEL-005, SAFE-005
//   The overlapping geometry is lawful under the body's validity rules
//   (ADR-0041): one entry inside the table's own first MiB and two beyond
//   it all assemble, because `partition-table` → `partition` is a
//   structural pair carrying no span claim. Had the containment check
//   been written as a blanket "child within parent", this honest disk —
//   and every committed GPT fixture with it — would refuse; this test is
//   what pins the pair-specific reading against that regression.
// Evidence: a_bios_boot_gpt_disk_assembles_under_the_validity_rules
#[test]
fn a_bios_boot_gpt_disk_assembles_under_the_validity_rules() {
    let f = bios_boot_gpt();
    let snapshot = super::snapshot::TopologySnapshot::assemble(
        super::snapshot::SnapshotKind::Captured,
        false,
        f.nodes.clone(),
        f.edges.clone(),
        f.facts.clone(),
    )
    .expect("the honest overlapping layout assembles");
    assert_eq!(snapshot.facts(), &f.facts);
    assert_eq!(snapshot.topology().entries().len(), 7);
    // The same fixture with its ZFS label pushed past its partition is the
    // issue-356 shape, and refuses.
    let mut forged = f.facts.clone();
    forged.extents.insert(
        f.signature,
        HostRange {
            host: f.sda,
            start: 900 * MIB,
            length: MIB,
        },
    );
    assert!(matches!(
        super::snapshot::TopologySnapshot::assemble(
            super::snapshot::SnapshotKind::Captured,
            false,
            f.nodes,
            f.edges,
            forged,
        ),
        Err(super::snapshot::SnapshotError::Facts(
            super::protection::FactError::ExtentOutsideContainmentParent { .. }
        ))
    ));
    let _ = (f.boot, f.esp, f.member, f.pool, f.table);
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   Deleting the bios_grub entry must reach neither the ESP nor the pool:
//   both are disjoint from the destroyed range. This is the property the
//   committed sibling guard states, on the one layout where the deleted
//   partition nests inside the table's own declared extent — the shape on
//   which the issue-347 round-2 candidate captured every sibling (its
//   panel's L1). Under ADR-0043 the release is decided by the step's
//   *target*: a partition-target step never releases its table, however
//   much of the table's declared region its range touches, so the honest
//   spelling of "delete bios_grub" — Wipe with the partition as target,
//   which is what the panel measured and what the planner emits — is
//   ten-for-ten Clear and reaches only the entry itself. The other
//   spelling is asserted beside it, as a priced limit rather
//   than a promise: a step whose target is the *table* and which destroys
//   any byte the body attributes to the table is read as destroying the
//   table, and releases. The closure has no non-authored way to tell one
//   GPT entry from the header — that is round 2's impossibility result —
//   and it reads the case fail-closed. Nothing delivered emits that
//   spelling; if something does, this is the row that says what it gets.
// Evidence: a_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table
#[test]
fn a_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    let f = bios_boot_gpt();
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&f.topology, &f.facts, f.boot, op),
            ProtectionGate::Clear,
            "{op:?} on bios_grub touches no sibling and must not refuse"
        );
    }
    let delete_bios_grub = canonical_ranges(Operation::Wipe, f.boot, &f.facts);
    let affected = affected_set(&f.topology, &f.facts, f.boot, &delete_bios_grub);
    assert!(
        affected.contains(&f.boot),
        "the destroyed entry itself is reached"
    );
    assert!(
        affected.contains(&f.table),
        "the table's declared region is intersected, so the table is in the set"
    );
    assert!(
        !affected.contains(&f.esp),
        "the ESP is disjoint from the destroyed range and must stay unreached"
    );
    assert!(!affected.contains(&f.member));
    assert!(
        !affected.contains(&f.pool),
        "the pool is disjoint from the destroyed range and must stay unreached"
    );

    // The priced limit: the same bytes destroyed by a step whose target
    // is the table release the table's partitions.
    let table_target = StepRanges {
        written_table_extents: vec![HostRange {
            host: f.sda,
            start: 0,
            length: MIB,
        }],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: f.sda,
            start: 17408,
            length: MIB - 17408,
        }],
    };
    let affected = affected_set(&f.topology, &f.facts, f.table, &table_target);
    assert!(
        affected.contains(&f.esp) && affected.contains(&f.pool),
        "a table-target step destroying bytes the body attributes to the table releases, fail-closed"
    );
    let _ = f.signature;
}

// Requirements: MODEL-002, SAFE-005
//   The destroyed range provably misses every GPT structure — LBA 0 is
//   [0, 512), the header LBA 1 is [512, 1024), the entry array LBA 2..33
//   is [1024, 17408) — and destroys [17408, 1 MiB) alone. Under ADR-0043
//   what that range releases is decided by whose step it is: with the
//   partition as target it releases nothing (the pool stays unreached, the
//   step constructs); with the table as target the closure reads the
//   table as destroyed and releases (the priced limit above). The pair is
//   the separation round 2 asked for — "the table was destroyed" from
//   "something inside the region the body attributes to the table was
//   destroyed" — drawn where the closure can draw it, on target identity,
//   never on the table's own authored geometry.
// Evidence: a_range_that_touches_no_gpt_structure_releases_only_from_a_table_target
#[test]
fn a_range_that_touches_no_gpt_structure_releases_only_from_a_table_target() {
    let f = bios_boot_gpt();
    let ranges = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: f.sda,
            start: 17408,
            length: MIB - 17408,
        }],
    };
    let affected = affected_set(&f.topology, &f.facts, f.boot, &ranges);
    assert!(
        !affected.contains(&f.pool),
        "from the partition's own step the range releases nothing"
    );
    assert!(step_constructs(&f.topology, &f.facts, f.boot, &ranges).is_ok());

    let affected = affected_set(&f.topology, &f.facts, f.table, &ranges);
    assert!(
        affected.contains(&f.pool),
        "from a table-target step the same range is read as destroying the table"
    );
    assert!(step_constructs(&f.topology, &f.facts, f.table, &ranges).is_err());
}

// ---------------------------------------------------------------------
// Issue #353: the canonical entry never writes a frame root wholesale,
// and a frame root that is the step's target reaches what it carries.
// ---------------------------------------------------------------------

/// A whole-disk ZFS vdev: no table; the label hosted by the device and
/// backing a live pool. The layout on which, before ADR-0042, every
/// refusal came from the over-claimed whole-device write alone.
struct WholeDiskVdev {
    topology: Topology,
    facts: Facts,
    sda: NodeId,
    signature: NodeId,
    pool: NodeId,
}

fn whole_disk_vdev() -> WholeDiskVdev {
    let sda = device(b"SDA");
    let sda_id = derive_id(&sda).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: sda_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![sda, signature, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
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
        signature_id,
        HostRange {
            host: sda_id,
            start: 0,
            length: MIB,
        },
    );
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    WholeDiskVdev {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        sda: sda_id,
        signature: signature_id,
        pool: pool_id,
    }
}

fn mutating_operations() -> [super::capability::Operation; 10] {
    use super::capability::Operation;
    [
        Operation::Create,
        Operation::Grow,
        Operation::Shrink,
        Operation::Move,
        Operation::Repair,
        Operation::Label,
        Operation::Uuid,
        Operation::Encrypt,
        Operation::Decrypt,
        Operation::Wipe,
    ]
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   Issue #353's regression, the one the issue said nothing committed
//   observed. On a whole-disk vdev every mutating gate refused before
//   this act, but six of them — Create, Grow, Repair, Label, Uuid,
//   Decrypt — refused only because the canonical entry wrote the parent
//   device wholesale, which §2.1 forbids in as many words; correct the
//   entry alone and six gates open over a live pool with the suite
//   green. Now the entry declares no written range for a frame root, and
//   the ten gates still refuse: the target seeds the set by identity and
//   descends into the label it carries. Both halves are asserted — the
//   entry's shape and the gate's outcome — so neither can quietly regress
//   into the other.
// Evidence: whole_disk_gates_hold_without_the_wholesale_write
#[test]
fn whole_disk_gates_hold_without_the_wholesale_write() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    let layout = whole_disk_vdev();
    for op in mutating_operations() {
        let ranges = canonical_ranges(op, layout.sda, &layout.facts);
        assert!(
            ranges.written_table_extents.is_empty(),
            "{op:?} must not write the parent device wholesale (§2.1), got {ranges:?}"
        );
        if matches!(
            op,
            Operation::Create
                | Operation::Grow
                | Operation::Repair
                | Operation::Label
                | Operation::Uuid
                | Operation::Decrypt
        ) {
            assert!(
                ranges.destroyed.is_empty() && ranges.consumed.is_empty(),
                "{op:?} destroys and consumes nothing at capability time, got {ranges:?}"
            );
        }
        let affected = affected_set(&layout.topology, &layout.facts, layout.sda, &ranges);
        assert!(
            affected.contains(&layout.signature) && affected.contains(&layout.pool),
            "{op:?} on the whole disk must reach the label it carries and the pool behind it"
        );
        let gate = protection_gate(&layout.topology, &layout.facts, layout.sda, op);
        assert!(
            matches!(
                gate,
                ProtectionGate::Unsupported {
                    ground: RefusalGround::Zfs
                }
            ),
            "{op:?} over a whole-disk vdev must refuse, got {gate:?}"
        );
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   The false-refusal control, and the exact reach of the new hop. On a
//   partitioned disk carrying a live pool member on sda2, the six
//   non-destroying operations on the disk itself refused before this
//   act only through the wholesale over-claim — creating a partition in
//   free space does not touch sda2 — and now clear; the four release
//   operations still destroy the whole extent and still refuse. The hop
//   reaches what the disk carries and no more: Label on sda brings the
//   table, not the ESP, not the member, not the pool.
// Evidence: a_frame_root_target_reaches_what_it_carries_and_no_more
#[test]
fn a_frame_root_target_reaches_what_it_carries_and_no_more() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    let layout = root_on_zfs();
    for op in mutating_operations() {
        let gate = protection_gate(&layout.topology, &layout.facts, layout.sda, op);
        let release = matches!(
            op,
            Operation::Shrink | Operation::Move | Operation::Encrypt | Operation::Wipe
        );
        if release {
            assert!(
                matches!(
                    gate,
                    ProtectionGate::Unsupported {
                        ground: RefusalGround::Zfs
                    }
                ),
                "{op:?} on the disk destroys its whole extent and must refuse, got {gate:?}"
            );
        } else {
            assert_eq!(
                gate,
                ProtectionGate::Clear,
                "{op:?} on the disk touches no protected partition and must not refuse"
            );
        }
    }
    // Below the frame root the entry is unchanged: a partition's Label
    // still declares the partition's own extent, framed on the disk,
    // which is what the plan layer's touched-device derivation reads.
    let member_label = canonical_ranges(Operation::Label, layout.member, &layout.facts);
    assert_eq!(
        member_label.written_table_extents,
        vec![layout.facts.extents[&layout.member]],
        "a partition target's entry is its own extent, unchanged by this act"
    );
    let label = canonical_ranges(Operation::Label, layout.sda, &layout.facts);
    let affected = affected_set(&layout.topology, &layout.facts, layout.sda, &label);
    assert!(affected.contains(&layout.sda));
    assert!(
        affected.contains(&layout.table),
        "the disk's table is content the disk carries"
    );
    assert!(
        !affected.contains(&layout.esp),
        "a partition is not carried by the disk"
    );
    assert!(!affected.contains(&layout.member));
    assert!(!affected.contains(&layout.pool));
}

// Requirements: MODEL-002, SAFE-005
//   The hop is for the target and for nothing else. A frame root that
//   enters the set because a range on it intersected its self-extent is
//   still never a descent source — that is ADR-0039's sibling-capture
//   guard, and it is asserted here on the very disk whose target case
//   now descends: deleting the member partition (target: the table)
//   range-destroys sda's self-extent, and the ESP stays out. Under
//   ADR-0039's clause with the target exemption removed, or with the
//   exemption widened to any node in the set, this reds.
// Evidence: a_frame_root_that_is_not_the_target_still_never_descends
#[test]
fn a_frame_root_that_is_not_the_target_still_never_descends() {
    let layout = root_on_zfs();
    let delete_member = StepRanges {
        written_table_extents: vec![HostRange {
            host: layout.sda,
            start: 0,
            length: MIB,
        }],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: layout.sda,
            start: 512 * MIB,
            length: 256 * MIB,
        }],
    };
    let affected = affected_set(
        &layout.topology,
        &layout.facts,
        layout.table,
        &delete_member,
    );
    assert!(
        affected.contains(&layout.sda),
        "the disk's self-extent intersects the destroyed range, so it is in the set"
    );
    assert!(
        !affected.contains(&layout.esp),
        "and yet the disk must not descend into its siblings: the ESP stays out"
    );
    assert!(
        affected.contains(&layout.pool),
        "the destroyed member's pool is reached"
    );
}

// Requirements: MODEL-002, SAFE-005
//   A frame root's own children are reached through the hop only where
//   geometry admits, exactly as any other descent: the disk's table is
//   inside the disk; a partition nested inside the table's own bytes is
//   reached through the table; a partition beyond them is not. Measured
//   on the BIOS-boot layout so the overlapping geometry is on record.
// Evidence: the_target_hop_is_bounded_by_the_same_geometry_as_every_other
#[test]
fn the_target_hop_is_bounded_by_the_same_geometry_as_every_other() {
    use super::capability::{Operation, canonical_ranges};
    let f = bios_boot_gpt();
    let create = canonical_ranges(Operation::Create, f.sda, &f.facts);
    assert!(create.written_table_extents.is_empty());
    let affected = affected_set(&f.topology, &f.facts, f.sda, &create);
    assert!(affected.contains(&f.table));
    assert!(
        affected.contains(&f.boot),
        "bios_grub lies inside the table's declared bytes and is reached through it"
    );
    assert!(!affected.contains(&f.esp));
    assert!(!affected.contains(&f.member));
    assert!(!affected.contains(&f.pool));
}

// ---------------------------------------------------------------------
// Issue #347 (ADR-0043): destroying a partition table releases the
// partitions it describes — decided structurally, by the step's target
// and the naming relation, never by the table's own authored geometry.
// ---------------------------------------------------------------------

/// The `root_on_zfs` layout with the two `table -> partition` containment
/// edges omitted, every fact unchanged. A body may omit an edge; it cannot
/// represent a partition without naming its table.
fn root_on_zfs_without_table_edges() -> RootOnZfs {
    let l = root_on_zfs();
    let nodes: Vec<NamingFields> = l
        .topology
        .entries()
        .iter()
        .map(|entry| match entry {
            super::naming::NodeEntry::Single { fields, .. }
            | super::naming::NodeEntry::Group { fields, .. } => fields.clone(),
        })
        .collect();
    let edges: Vec<Edge> = l
        .topology
        .edges()
        .iter()
        .filter(|edge| !(edge.kind == EdgeKind::Containment && edge.source == l.table))
        .copied()
        .collect();
    assert_eq!(
        edges.len(),
        l.topology.edges().len() - 2,
        "both table edges dropped"
    );
    let topology = Topology::build(nodes, edges).expect("builds");
    RootOnZfs { topology, ..l }
}

fn wipe_of(facts: &Facts, node: NodeId) -> StepRanges {
    StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![facts.extents[&node]],
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   Issue #347 as filed: destroying the partition table of a disk carrying
//   a live ZFS vdev on sda2 releases the partitions it describes, and the
//   pool behind the member refuses. At HEAD the ten-operation gate on the
//   table was 10/10 Clear with the pool's Refused{Zfs} never consulted.
//   Now the four release operations refuse and the six that destroy
//   nothing stay Clear (Repair on a table is not a release — the property
//   a rejected #319 arm defeated). Both halves are asserted, the affected
//   set and the gate, so neither can regress into the other.
// Evidence: destroying_a_partition_table_releases_the_partitions_it_describes
#[test]
fn destroying_a_partition_table_releases_the_partitions_it_describes() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    let l = root_on_zfs();
    let affected = affected_set(&l.topology, &l.facts, l.table, &wipe_of(&l.facts, l.table));
    for (name, node) in [
        ("esp", l.esp),
        ("member", l.member),
        ("signature", l.signature),
        ("pool", l.pool),
    ] {
        assert!(
            affected.contains(&node),
            "destroying the table releases {name}"
        );
    }
    assert!(step_constructs(&l.topology, &l.facts, l.table, &wipe_of(&l.facts, l.table)).is_err());
    for op in mutating_operations() {
        let gate = protection_gate(&l.topology, &l.facts, l.table, op);
        if matches!(
            op,
            Operation::Wipe | Operation::Shrink | Operation::Move | Operation::Encrypt
        ) {
            assert!(
                matches!(
                    gate,
                    ProtectionGate::Unsupported {
                        ground: RefusalGround::Zfs
                    }
                ),
                "{op:?} destroys the table and must refuse through the pool, got {gate:?}"
            );
        } else {
            assert_eq!(gate, ProtectionGate::Clear, "{op:?} destroys nothing");
        }
    }
}

// Requirements: MODEL-002, SAFE-005
//   The release is read off the naming relation and the step's target,
//   and off nothing the body can author to remove it (round 1 §11.4's
//   acceptance test, asked in the removal direction). Omit both
//   `table -> partition` edges: the release still fires — a partition
//   cannot be represented without naming its table. Deflate the table's
//   extent to its real 17 408 bytes, or inflate it by a byte: the release
//   still fires, because the trigger is the target's own membership in
//   the destroyed class, and a step that destroys its own target's extent
//   intersects it at every size. Under-declare the wipe to the protective
//   MBR's 512 bytes with the table as target: it still fires, fail-closed.
//   None of these is a coverage test, which round 2 measured to be
//   anti-monotone in the authored extent.
// Evidence: the_release_follows_the_naming_relation_not_the_edges_or_the_extent
#[test]
fn the_release_follows_the_naming_relation_not_the_edges_or_the_extent() {
    let l = root_on_zfs_without_table_edges();
    let affected = affected_set(&l.topology, &l.facts, l.table, &wipe_of(&l.facts, l.table));
    assert!(
        affected.contains(&l.pool),
        "with the table -> partition edges omitted the release still reaches the pool"
    );

    let l = root_on_zfs();
    for (label, length) in [
        ("deflated to 17408", 17408),
        ("inflated by a byte", MIB + 1),
    ] {
        let mut facts = l.facts.clone();
        facts.extents.insert(
            l.table,
            HostRange {
                host: l.sda,
                start: 0,
                length,
            },
        );
        let affected = affected_set(&l.topology, &facts, l.table, &wipe_of(&facts, l.table));
        assert!(
            affected.contains(&l.pool),
            "table extent {label}: the release still fires"
        );
    }

    let under_declared = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: l.sda,
            start: 0,
            length: 512,
        }],
    };
    let affected = affected_set(&l.topology, &l.facts, l.table, &under_declared);
    assert!(
        affected.contains(&l.pool),
        "a table-target step destroying 512 bytes of the table is read as destroying it"
    );
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   A table that is not the step's target never releases, however its
//   declared extent is touched. Round 2's L2: inflate the table's extent by
//   one byte so the ESP's own extent intersects it, then wipe the ESP —
//   the table enters the destroyed class by intersection, and released
//   nothing: the ESP's gate stays ten-for-ten Clear, the member and the
//   pool stay out. Under the rejected candidate this row went
//   Unsupported{Zfs} on one byte of over-declaration; here the byte does
//   nothing, because release is not decided by intersection.
// Evidence: a_table_that_is_not_the_target_never_releases
#[test]
fn a_table_that_is_not_the_target_never_releases() {
    use super::capability::{ProtectionGate, protection_gate};
    let l = root_on_zfs();
    let mut facts = l.facts.clone();
    facts.extents.insert(
        l.table,
        HostRange {
            host: l.sda,
            start: 0,
            length: MIB + 1,
        },
    );
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&l.topology, &facts, l.esp, op),
            ProtectionGate::Clear,
            "{op:?} on the ESP under a one-byte-inflated table must not refuse"
        );
    }
    let affected = affected_set(&l.topology, &facts, l.esp, &wipe_of(&facts, l.esp));
    assert!(
        affected.contains(&l.table),
        "the table's inflated extent is intersected, so it is in the set"
    );
    assert!(!affected.contains(&l.member), "and released nothing");
    assert!(!affected.contains(&l.pool));
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   The false-refusal controls. A plain ext4 disk: destroying its table
//   releases the partition and the file system it carries, both
//   Permitted, and the step constructs — the release reaches, it does not
//   refuse. A hybrid disk: the GPT describes the partitions and a hybrid
//   MBR view describes none, so wiping the MBR view releases nothing and
//   stays Clear while wiping the GPT refuses through the pool under sda2 —
//   round 2's M3 (a 512-byte MBR wipe refusing through a pool 512 MiB
//   away) cannot arise, because the release follows `parent_table`, and a
//   ConflictingTableEntry names a table without being released by it.
// Evidence: a_released_partition_refuses_only_for_what_it_carries
#[test]
#[allow(clippy::too_many_lines)]
fn a_released_partition_refuses_only_for_what_it_carries() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    use super::naming::FileSystemKind;

    // Plain ext4 disk.
    let plain = device(b"SDY");
    let plain_id = derive_id(&plain).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: plain_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let p1 = NamingFields::Partition {
        parent_table: table_id,
        start_offset: MIB,
    };
    let p1_id = derive_id(&p1).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: p1_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");
    let topology = Topology::build(
        vec![plain, table, p1, fs],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: plain_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: p1_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: p1_id,
                target: fs_id,
            },
        ],
    )
    .expect("builds");
    let mut extents = BTreeMap::new();
    let host = |start, length| HostRange {
        host: plain_id,
        start,
        length,
    };
    extents.insert(plain_id, host(0, 1 << 30));
    extents.insert(table_id, host(0, MIB));
    extents.insert(p1_id, host(MIB, 512 * MIB));
    extents.insert(fs_id, host(MIB, 512 * MIB));
    let mut transports = BTreeMap::new();
    transports.insert(plain_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let affected = affected_set(&topology, &facts, table_id, &wipe_of(&facts, table_id));
    assert!(affected.contains(&p1_id) && affected.contains(&fs_id));
    assert!(
        step_constructs(&topology, &facts, table_id, &wipe_of(&facts, table_id)).is_ok(),
        "nothing released is protected, so the table wipe constructs"
    );
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&topology, &facts, table_id, op),
            ProtectionGate::Clear
        );
    }

    // Hybrid disk: GPT + hybrid MBR view + one conflicting entry; the pool
    // is under the GPT's second partition.
    let sda = device(b"HYB");
    let sda_id = derive_id(&sda).expect("derivable");
    let gpt = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let gpt_id = derive_id(&gpt).expect("derivable");
    let mbr = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::HybridMbr,
    };
    let mbr_id = derive_id(&mbr).expect("derivable");
    let esp = NamingFields::Partition {
        parent_table: gpt_id,
        start_offset: MIB,
    };
    let esp_id = derive_id(&esp).expect("derivable");
    let member = NamingFields::Partition {
        parent_table: gpt_id,
        start_offset: 512 * MIB,
    };
    let member_id = derive_id(&member).expect("derivable");
    let sig = NamingFields::BackingSignature {
        host: member_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let sig_id = derive_id(&sig).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"hp".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let cte = NamingFields::ConflictingTableEntry {
        table: mbr_id,
        view_role: TableRole::HybridMbr,
        entry_start: MIB,
    };
    let cte_id = derive_id(&cte).expect("derivable");
    let topology = Topology::build(
        vec![sda, gpt, mbr, esp, member, sig, pool, cte],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: gpt_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: mbr_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: gpt_id,
                target: esp_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: gpt_id,
                target: member_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: member_id,
                target: sig_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: sig_id,
                target: pool_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: mbr_id,
                target: cte_id,
            },
        ],
    )
    .expect("builds");
    let mut extents = BTreeMap::new();
    let host = |start, length| HostRange {
        host: sda_id,
        start,
        length,
    };
    extents.insert(sda_id, host(0, 1 << 30));
    extents.insert(gpt_id, host(0, MIB));
    extents.insert(mbr_id, host(0, 512));
    extents.insert(esp_id, host(MIB, 256 * MIB));
    extents.insert(member_id, host(512 * MIB, 256 * MIB));
    extents.insert(sig_id, host(512 * MIB, MIB));
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&topology, &facts, mbr_id, op),
            ProtectionGate::Clear,
            "{op:?} on the hybrid MBR view describes no partition and releases nothing"
        );
    }
    let affected = affected_set(&topology, &facts, mbr_id, &wipe_of(&facts, mbr_id));
    assert!(!affected.contains(&esp_id) && !affected.contains(&pool_id));
    let _ = cte_id;
    assert!(matches!(
        protection_gate(&topology, &facts, gpt_id, Operation::Wipe),
        ProtectionGate::Unsupported {
            ground: RefusalGround::Zfs
        }
    ));
}

// Requirements: MODEL-002, SAFE-005
//   The release roster, pinned per kind — the property test ADR-0018:210-217
//   demands, quantified over the naming roster rather than the edge set.
//   Exactly one kind is released by a table's destruction, `Partition`,
//   and the table that releases it is the one its own name declares in
//   `parent_table`. `ConflictingTableEntry` names a table in `table` and
//   is not released by it (ADR-0019 holds it verbatim as a record in the
//   table's own bytes; ADR-0036 decided it is not an occupant of the
//   region it names) — round 2's L3, which the candidate then had no
//   fixture to kill. Every other kind names no table. A kind added to the
//   roster that names a table lands here and must be classified.
// Evidence: the_release_roster_is_pinned_per_kind
#[test]
fn the_release_roster_is_pinned_per_kind() {
    use super::naming::{ExtentLocator, FileSystemKind};
    let dev = device(b"ROSTER");
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
    let sig = NamingFields::BackingSignature {
        host: part_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let sig_id = derive_id(&sig).expect("derivable");
    let layer = NamingFields::EncryptionLayer {
        backing_signature: sig_id,
    };
    let layer_id = derive_id(&layer).expect("derivable");
    let agg = NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: Some(b"vg".to_vec()),
    };
    let agg_id = derive_id(&agg).expect("derivable");
    let vol = NamingFields::Volume {
        producer: agg_id,
        name: b"lv".to_vec(),
        role: None,
    };
    let vol_id = derive_id(&vol).expect("derivable");
    let one_of_each: Vec<NamingFields> = vec![
        dev,
        table.clone(),
        part.clone(),
        NamingFields::ConflictingTableEntry {
            table: table_id,
            view_role: TableRole::HybridMbr,
            entry_start: MIB,
        },
        sig,
        NamingFields::FileSystem {
            host: vol_id,
            kind: FileSystemKind::Ext4,
            superblock_offset: 1024,
        },
        NamingFields::BackingExtent {
            host: vol_id,
            locator: ExtentLocator::Range {
                start: 0,
                length: MIB,
            },
        },
        layer.clone(),
        agg,
        vol,
        NamingFields::MultipathNode {
            lun_designator: b"mpatha".to_vec(),
        },
    ];
    let mut kinds: Vec<&'static str> = one_of_each.iter().map(NamingFields::kind_name).collect();
    kinds.sort_unstable();
    kinds.dedup();
    assert_eq!(kinds.len(), 11, "one of every kind is on the roster");

    let mut released_kinds = Vec::new();
    for fields in &one_of_each {
        let names_a_table = fields
            .naming_referents()
            .iter()
            .any(|(_, referent)| *referent == table_id);
        match (fields.kind_name(), fields.released_by_table()) {
            ("partition", Some(table)) => {
                assert_eq!(table, table_id, "released by the table its name declares");
                assert!(names_a_table);
                released_kinds.push("partition");
            }
            ("partition", None) => panic!("a partition is released by its table"),
            ("conflicting-table-entry", released) => {
                assert!(names_a_table, "a conflicting entry names its table");
                assert!(released.is_none(), "and is not released by it (round 2 L3)");
            }
            (kind, released) => {
                assert!(!names_a_table, "{kind} names no table");
                assert!(released.is_none(), "{kind} is released by nothing");
            }
        }
    }
    assert_eq!(released_kinds, vec!["partition"]);
    let _ = layer_id;
}
