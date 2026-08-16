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

/// The four operations whose canonical entry declares a destroyed range
/// (ADR-0038's release set). The other six mutating operations write and
/// destroy nothing, which is why ADR-0048 moves these and not those.
fn destroying_operations() -> [super::capability::Operation; 4] {
    use super::capability::Operation;
    [
        Operation::Wipe,
        Operation::Encrypt,
        Operation::Move,
        Operation::Shrink,
    ]
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

/// The #360 chain (issue #360, ADR-0044): a whole-disk mdraid member
/// whose array produces `md0`, which carries its own GPT; `md0p1` hosts a
/// ZFS vdev. `disk → mdraid signature → array → volume → table → md0p1
/// → zfs signature → pool`. Every extent below the array is framed on the
/// volume, so no range declared on the disk ever intersects one: the pool
/// is reachable only if destruction propagates from the disk through the
/// array to the table, and the table then releases.
struct PartitionedMdraid {
    topology: Topology,
    facts: Facts,
    sda: NodeId,
    md_signature: NodeId,
    array: NodeId,
    md0: NodeId,
    table: NodeId,
    md0p1: NodeId,
    zfs_signature: NodeId,
    pool: NodeId,
}

#[allow(clippy::too_many_lines)]
fn partitioned_mdraid() -> PartitionedMdraid {
    let sda = device(b"MD-MEMBER");
    let sda_id = derive_id(&sda).expect("derivable");
    let md_signature = NamingFields::BackingSignature {
        host: sda_id,
        family: SignatureFamily::Mdraid1x,
        primary_offset: 4096,
    };
    let md_signature_id = derive_id(&md_signature).expect("derivable");
    let array = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"md0-uuid".to_vec()),
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
        start_offset: MIB,
    };
    let md0p1_id = derive_id(&md0p1).expect("derivable");
    let zfs_signature = NamingFields::BackingSignature {
        host: md0p1_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let zfs_signature_id = derive_id(&zfs_signature).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"mdpool".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![
            sda,
            md_signature,
            array,
            md0,
            table,
            md0p1,
            zfs_signature,
            pool,
        ],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: md_signature_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: md_signature_id,
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
            Edge {
                kind: EdgeKind::Containment,
                source: md0p1_id,
                target: zfs_signature_id,
            },
            Edge {
                kind: EdgeKind::Backing,
                source: zfs_signature_id,
                target: pool_id,
            },
        ],
    )
    .expect("a partitioned mdraid array builds (issue #360)");
    let mut extents = BTreeMap::new();
    let on_disk = |start, length| HostRange {
        host: sda_id,
        start,
        length,
    };
    let on_md0 = |start, length| HostRange {
        host: md0_id,
        start,
        length,
    };
    extents.insert(sda_id, on_disk(0, 1 << 30));
    extents.insert(md_signature_id, on_disk(4096, 4096));
    extents.insert(table_id, on_md0(0, MIB));
    extents.insert(md0p1_id, on_md0(MIB, 512 * MIB));
    extents.insert(zfs_signature_id, on_md0(MIB, MIB));
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    PartitionedMdraid {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        sda: sda_id,
        md_signature: md_signature_id,
        array: array_id,
        md0: md0_id,
        table: table_id,
        md0p1: md0p1_id,
        zfs_signature: zfs_signature_id,
        pool: pool_id,
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   Issue #360's chain, closed. A whole-disk mdraid member whose array
//   produces a volume carrying its own GPT, with a ZFS vdev on `md0p1`:
//   every extent below the array is framed on the volume, so no range on
//   the disk intersects one and reach alone stopped at the table (ADR-0043
//   measured it: four hops down and the partition survived). Destruction
//   now carries from the wiped member through the signature, the array
//   and the volume to the table, and the table releases: the pool is
//   reached, the step refuses, and the four release operations on the
//   member disk and on the array's own superblock go `Unsupported{Zfs}`
//   while the six that destroy nothing stay `Clear`. The table on the
//   volume, taken as a target, releases exactly as a table on a disk does.
// Evidence: destroying_a_partitioned_arrays_member_reaches_what_its_partitions_carry
#[test]
fn destroying_a_partitioned_arrays_member_reaches_what_its_partitions_carry() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    let m = partitioned_mdraid();
    let affected = affected_set(&m.topology, &m.facts, m.sda, &wipe_of(&m.facts, m.sda));
    for (name, node) in [
        ("md signature", m.md_signature),
        ("array", m.array),
        ("md0", m.md0),
        ("table", m.table),
        ("md0p1", m.md0p1),
        ("zfs signature", m.zfs_signature),
        ("pool", m.pool),
    ] {
        assert!(affected.contains(&node), "wiping the member reaches {name}");
    }
    let refusal = step_constructs(&m.topology, &m.facts, m.sda, &wipe_of(&m.facts, m.sda))
        .expect_err("the pool on the array's partition refuses the member wipe");
    assert!(matches!(refusal.verdict, Verdict::Refused { .. }));
    for target in [m.sda, m.md_signature, m.table] {
        for op in mutating_operations() {
            let gate = protection_gate(&m.topology, &m.facts, target, op);
            if matches!(
                op,
                Operation::Wipe | Operation::Shrink | Operation::Move | Operation::Encrypt
            ) {
                // The pool itself or the vdev signature that inherits its
                // refusal, whichever the closure visits first.
                assert!(
                    matches!(
                        gate,
                        ProtectionGate::Unsupported {
                            ground: RefusalGround::Zfs
                                | RefusalGround::InheritedFromConsumerOrProducer
                        }
                    ),
                    "{op:?} destroys its target and must refuse through the pool, got {gate:?}"
                );
            } else {
                assert_eq!(gate, ProtectionGate::Clear, "{op:?} destroys nothing");
            }
        }
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   What carries destruction, and what does not. Only the step's own
//   destruction of its target seeds it: the six operations that destroy
//   nothing reach the table on the volume (ADR-0039's carried content)
//   and release nothing — a table reached is not a table destroyed. A
//   range that merely touches a table never seeds it either: round 2's L1
//   and L2 guards hold unmoved and are not re-asserted here. ADR-0044's
//   named limit — an extentless target declaring no destroyed range, so
//   its own wipe could not be seen destroyed — is closed by ADR-0048
//   (issue #392): the volume and the array now refuse, and the rows that
//   pinned them assert the refusal instead. What this test still pins is
//   the distinction those rows were guarding: reach is not destruction.
// Evidence: destruction_carries_only_from_the_target_and_reach_never_releases
#[test]
fn destruction_carries_only_from_the_target_and_reach_never_releases() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    let m = partitioned_mdraid();
    let ranges = super::capability::canonical_ranges(Operation::Label, m.sda, &m.facts);
    let affected = affected_set(&m.topology, &m.facts, m.sda, &ranges);
    assert!(
        affected.contains(&m.table),
        "a label on the member reaches the table the array's volume carries"
    );
    assert!(
        !affected.contains(&m.md0p1) && !affected.contains(&m.pool),
        "and releases nothing: reach is not destruction"
    );
    // ADR-0044's named limit, closed by ADR-0048 (issue #392): an
    // extentless target's own wipe is seen destroyed, so the pool under
    // the table it carries is reached and every destroying operation
    // refuses. The volume reaches it because its children are framed on
    // it; the array because destruction now carries by identity into the
    // volume it produces.
    for target in [m.md0, m.array] {
        for op in destroying_operations() {
            assert_ne!(
                protection_gate(&m.topology, &m.facts, target, op),
                ProtectionGate::Clear,
                "{op:?} on an extentless target is seen destroyed (ADR-0048)"
            );
        }
        let ranges = super::capability::canonical_ranges(Operation::Wipe, target, &m.facts);
        let affected = affected_set(&m.topology, &m.facts, target, &ranges);
        assert!(
            affected.contains(&m.pool),
            "the pool under an extentless target's table is reached"
        );
    }

    // The distinction the closed limit was guarding, pinned in its own
    // right: a step that declares no destroyed range still only reaches.
    for target in [m.md0, m.array] {
        let affected = affected_set(&m.topology, &m.facts, target, &StepRanges::default());
        assert!(
            affected.contains(&m.table) && !affected.contains(&m.pool),
            "with no declared ranges, an extentless target reaches its table and releases nothing"
        );
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   The other population the row admits, and the false-refusal control.
//   A GPT inside a LUKS-mapped volume on the second partition of an
//   ordinary disk, a pool under the inner table's partition: wiping the
//   LUKS partition destroys it, destruction carries through its signature,
//   the layer and the mapper to the inner table, which releases, and the
//   pool refuses; wiping the ESP beside it stays 10/10 Clear — nothing on
//   the disk's own frame connects the two — and a label on the LUKS
//   partition reaches the inner table without releasing it. And the
//   control: a partitioned array whose partition carries a plain ext4
//   releases and reaches, both Permitted, and the member wipe constructs.
// Evidence: a_table_inside_a_mapped_volume_releases_and_a_plain_one_constructs
#[test]
#[allow(clippy::too_many_lines)]
fn a_table_inside_a_mapped_volume_releases_and_a_plain_one_constructs() {
    use super::capability::{Operation, ProtectionGate, protection_gate};
    use super::naming::FileSystemKind;

    // GPT inside LUKS.
    let disk = device(b"LUKS-GPT");
    let disk_id = derive_id(&disk).expect("derivable");
    let gpt = NamingFields::PartitionTable {
        parent: disk_id,
        role: TableRole::Gpt,
    };
    let gpt_id = derive_id(&gpt).expect("derivable");
    let esp = NamingFields::Partition {
        parent_table: gpt_id,
        start_offset: MIB,
    };
    let esp_id = derive_id(&esp).expect("derivable");
    let crypt = NamingFields::Partition {
        parent_table: gpt_id,
        start_offset: 512 * MIB,
    };
    let crypt_id = derive_id(&crypt).expect("derivable");
    let luks = NamingFields::BackingSignature {
        host: crypt_id,
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
        name: b"cryptdisk".to_vec(),
        role: None,
    };
    let mapper_id = derive_id(&mapper).expect("derivable");
    let inner = NamingFields::PartitionTable {
        parent: mapper_id,
        role: TableRole::Gpt,
    };
    let inner_id = derive_id(&inner).expect("derivable");
    let inner_p1 = NamingFields::Partition {
        parent_table: inner_id,
        start_offset: MIB,
    };
    let inner_p1_id = derive_id(&inner_p1).expect("derivable");
    let zfs = NamingFields::BackingSignature {
        host: inner_p1_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let zfs_id = derive_id(&zfs).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"cryptpool".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let containment = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    let topology = Topology::build(
        vec![
            disk, gpt, esp, crypt, luks, layer, mapper, inner, inner_p1, zfs, pool,
        ],
        vec![
            containment(disk_id, gpt_id),
            containment(gpt_id, esp_id),
            containment(gpt_id, crypt_id),
            containment(crypt_id, luks_id),
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
            containment(mapper_id, inner_id),
            containment(inner_id, inner_p1_id),
            containment(inner_p1_id, zfs_id),
            Edge {
                kind: EdgeKind::Backing,
                source: zfs_id,
                target: pool_id,
            },
        ],
    )
    .expect("a GPT inside a LUKS-mapped volume builds (issue #360)");
    let mut extents = BTreeMap::new();
    let on_disk = |start, length| HostRange {
        host: disk_id,
        start,
        length,
    };
    let on_mapper = |start, length| HostRange {
        host: mapper_id,
        start,
        length,
    };
    extents.insert(disk_id, on_disk(0, 1 << 30));
    extents.insert(gpt_id, on_disk(0, MIB));
    extents.insert(esp_id, on_disk(MIB, 256 * MIB));
    extents.insert(crypt_id, on_disk(512 * MIB, 512 * MIB));
    extents.insert(luks_id, on_disk(512 * MIB, 16 << 10));
    extents.insert(inner_id, on_mapper(0, MIB));
    extents.insert(inner_p1_id, on_mapper(MIB, 256 * MIB));
    extents.insert(zfs_id, on_mapper(MIB, MIB));
    let mut transports = BTreeMap::new();
    transports.insert(disk_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let affected = affected_set(&topology, &facts, crypt_id, &wipe_of(&facts, crypt_id));
    assert!(
        affected.contains(&inner_p1_id) && affected.contains(&pool_id),
        "wiping the LUKS partition releases the inner table's partition and reaches the pool"
    );
    assert!(step_constructs(&topology, &facts, crypt_id, &wipe_of(&facts, crypt_id)).is_err());
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&topology, &facts, esp_id, op),
            ProtectionGate::Clear,
            "{op:?} on the ESP beside the LUKS partition must not refuse"
        );
    }
    let label = super::capability::canonical_ranges(Operation::Label, crypt_id, &facts);
    let affected = affected_set(&topology, &facts, crypt_id, &label);
    assert!(
        affected.contains(&inner_id) && !affected.contains(&inner_p1_id),
        "a label on the LUKS partition reaches the inner table and releases nothing"
    );
    // A released partition is destroyed, not merely reached: wiping the
    // outer table releases the LUKS partition, whose destruction carries
    // to the inner table, which releases in turn.
    let affected = affected_set(&topology, &facts, gpt_id, &wipe_of(&facts, gpt_id));
    assert!(
        affected.contains(&inner_p1_id) && affected.contains(&pool_id),
        "the outer table's release carries through the mapped volume to the inner table's release"
    );

    // The control: a partitioned array carrying a plain ext4.
    let sda = device(b"MD-PLAIN");
    let sda_id = derive_id(&sda).expect("derivable");
    let md_signature = NamingFields::BackingSignature {
        host: sda_id,
        family: SignatureFamily::Mdraid1x,
        primary_offset: 4096,
    };
    let md_signature_id = derive_id(&md_signature).expect("derivable");
    let array = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"md1-uuid".to_vec()),
    };
    let array_id = derive_id(&array).expect("derivable");
    let md1 = NamingFields::Volume {
        producer: array_id,
        name: b"md1".to_vec(),
        role: None,
    };
    let md1_id = derive_id(&md1).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: md1_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let md1p1 = NamingFields::Partition {
        parent_table: table_id,
        start_offset: MIB,
    };
    let md1p1_id = derive_id(&md1p1).expect("derivable");
    let fs = NamingFields::FileSystem {
        host: md1p1_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let fs_id = derive_id(&fs).expect("derivable");
    let topology = Topology::build(
        vec![sda, md_signature, array, md1, table, md1p1, fs],
        vec![
            containment(sda_id, md_signature_id),
            Edge {
                kind: EdgeKind::Backing,
                source: md_signature_id,
                target: array_id,
            },
            Edge {
                kind: EdgeKind::Production,
                source: array_id,
                target: md1_id,
            },
            containment(md1_id, table_id),
            containment(table_id, md1p1_id),
            containment(md1p1_id, fs_id),
        ],
    )
    .expect("builds");
    let mut extents = BTreeMap::new();
    let on_disk = |start, length| HostRange {
        host: sda_id,
        start,
        length,
    };
    let on_md1 = |start, length| HostRange {
        host: md1_id,
        start,
        length,
    };
    extents.insert(sda_id, on_disk(0, 1 << 30));
    extents.insert(md_signature_id, on_disk(4096, 4096));
    extents.insert(table_id, on_md1(0, MIB));
    extents.insert(md1p1_id, on_md1(MIB, 512 * MIB));
    extents.insert(fs_id, on_md1(MIB, 512 * MIB));
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        member_counts: BTreeMap::new(),
        table_states: BTreeMap::new(),
    };
    let affected = step_constructs(&topology, &facts, sda_id, &wipe_of(&facts, sda_id))
        .expect("nothing released is protected, so the member wipe constructs");
    assert!(
        affected.contains(&md1p1_id) && affected.contains(&fs_id),
        "the release reaches the partition and its file system"
    );
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&topology, &facts, sda_id, op),
            ProtectionGate::Clear
        );
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   Content on a multipath node inherits its detection-only refusal
//   (ADR-0045; ADR-0011). Before the `multipath-node → …` containment rows
//   an xfs on `/dev/mapper/mpatha` could name the node in `host` and build,
//   but no edge could carry it, so its device-scope ascent found itself as
//   its own root and every one of its ten mutating gates was `Clear` over
//   a device §2.1 says never to mutate. With the row and the edge the node
//   is a containment root like a device, its own arm is inherited, and
//   the gate is `Unsupported` ten times over. The omitted-edge spelling is
//   pinned beside it as the named limit: it still gates `Clear`, because
//   device scope ascends the edge set and not the name — the escape ADR-0043
//   closed for release and this act leaves open for scope, filed.
// Evidence: content_on_a_multipath_node_inherits_its_detection_only_refusal
#[test]
fn content_on_a_multipath_node_inherits_its_detection_only_refusal() {
    use super::capability::{ProtectionGate, protection_gate};
    use super::naming::FileSystemKind;
    let mpatha = NamingFields::MultipathNode {
        lun_designator: b"naa.60014".to_vec(),
    };
    let mpatha_id = derive_id(&mpatha).expect("derivable");
    let xfs = NamingFields::FileSystem {
        host: mpatha_id,
        kind: FileSystemKind::Xfs,
        superblock_offset: 0,
    };
    let xfs_id = derive_id(&xfs).expect("derivable");
    let mut extents = BTreeMap::new();
    extents.insert(
        xfs_id,
        HostRange {
            host: mpatha_id,
            start: 0,
            length: 512 * MIB,
        },
    );
    let facts = Facts {
        extents,
        ..Facts::default()
    };

    let with_edge = Topology::build(
        vec![mpatha.clone(), xfs.clone()],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: mpatha_id,
            target: xfs_id,
        }],
    )
    .expect("an xfs on a multipath node builds (ADR-0045)");
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&with_edge, &facts, xfs_id, op),
            ProtectionGate::Unsupported {
                ground: RefusalGround::InheritedDeviceScope
            },
            "{op:?} on content of a multipath node inherits the node's refusal"
        );
    }
    assert!(matches!(
        node_verdict(&with_edge, &facts, xfs_id),
        Verdict::Refused {
            ground: RefusalGround::InheritedDeviceScope
        }
    ));

    // ADR-0045's named limit, closed by issue #397: with the edge
    // omitted the name alone still carries the scope, because the
    // hosting field is in the node's own hashed address.
    let without_edge = Topology::build(vec![mpatha, xfs], vec![]).expect("builds");
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&without_edge, &facts, xfs_id, op),
            ProtectionGate::Unsupported {
                ground: RefusalGround::InheritedDeviceScope
            },
            "{op:?}: device scope ascends names as well as edges (issue #397)"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   Issue #333's own measurement, made unrepresentable (ADR-0046). At the
//   filing, re-anchoring only the ZFS signature's extent into its member
//   partition's address space — every extent still present — left the
//   pool unreached and the whole-device wipe constructing, defeating the
//   flagship refusal without removing a fact. Under ADR-0037's rule as
//   enforced, that fact set is refused before any closure can read it:
//   the signature's name leads through the member and its table to `sda`,
//   and an extent framed anywhere else — the member, the table, the
//   sibling ESP — is refused with the declared frame and the derived root
//   named side by side. The refusal stands with both table edges removed,
//   because the root is read off the name; and every committed layout in
//   this file — root-on-ZFS with and without its table edges, the LUKS
//   chain, the BIOS-boot GPT, the whole-disk vdev, the partitioned mdraid
//   array with its volume-framed extents — is framed as the rule
//   requires.
// Evidence: the_flagship_defeat_is_unrepresentable_and_every_layout_is_lawful
#[test]
fn the_flagship_defeat_is_unrepresentable_and_every_layout_is_lawful() {
    use super::protection::{FactError, validate_facts};
    let layout = root_on_zfs();
    let signature = layout.facts.extents[&layout.signature];
    for declared in [layout.member, layout.table, layout.esp] {
        let mut facts = layout.facts.clone();
        facts.extents.insert(
            layout.signature,
            HostRange {
                host: declared,
                start: 0,
                ..signature
            },
        );
        let expected = Err(FactError::ExtentFrameDisagreesWithName {
            node: layout.signature,
            declared,
            derived: layout.sda,
        });
        assert_eq!(validate_facts(&layout.topology, &facts), expected);
        assert_eq!(
            validate_facts(&root_on_zfs_without_table_edges().topology, &facts),
            expected,
            "the frame is read off the name, not the edges"
        );
    }

    let without_edges = root_on_zfs_without_table_edges();
    let luks = luks_chain();
    let bios = bios_boot_gpt();
    let whole = whole_disk_vdev();
    let mdraid = partitioned_mdraid();
    let lawful: [(&str, &Topology, &Facts); 6] = [
        ("root_on_zfs", &layout.topology, &layout.facts),
        (
            "root_on_zfs_without_table_edges",
            &without_edges.topology,
            &without_edges.facts,
        ),
        ("luks_chain", &luks.topology, &luks.facts),
        ("bios_boot_gpt", &bios.topology, &bios.facts),
        ("whole_disk_vdev", &whole.topology, &whole.facts),
        ("partitioned_mdraid", &mdraid.topology, &mdraid.facts),
    ];
    for (name, topology, facts) in lawful {
        assert_eq!(validate_facts(topology, facts), Ok(()), "{name}");
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   Issue #397, ADR-0045's named limit closed. Device scope and the
//   producing relation both ascend the naming relation beside the edge
//   set. An edge is authored content a body may omit; a hosting or
//   producing field is in the node's own hashed name, so omitting it
//   changes the node's address rather than hiding its host. The three
//   arms: a multipath node's detection-only refusal, a device's
//   transport arm, and a producer's own refusal — each inherited with
//   the edge absent, on a body that builds and validates.
// Evidence: an_omitted_edge_is_not_an_escape_from_inheritance
#[test]
fn an_omitted_edge_is_not_an_escape_from_inheritance() {
    use super::capability::{ProtectionGate, protection_gate};
    use super::naming::FileSystemKind;
    use super::protection::validate_facts;

    // 1. A multipath node's detection-only refusal, the edge omitted.
    let mpatha = NamingFields::MultipathNode {
        lun_designator: b"naa.60015".to_vec(),
    };
    let mpatha_id = derive_id(&mpatha).expect("derivable");
    let xfs = NamingFields::FileSystem {
        host: mpatha_id,
        kind: FileSystemKind::Xfs,
        superblock_offset: 0,
    };
    let xfs_id = derive_id(&xfs).expect("derivable");
    let mut extents = BTreeMap::new();
    extents.insert(
        xfs_id,
        HostRange {
            host: mpatha_id,
            start: 0,
            length: 512 * MIB,
        },
    );
    let mp_facts = Facts {
        extents,
        ..Facts::default()
    };
    let mp = Topology::build(vec![mpatha, xfs], vec![]).expect("builds with no edge");
    assert_eq!(validate_facts(&mp, &mp_facts), Ok(()), "the body is lawful");
    for op in mutating_operations() {
        assert_eq!(
            protection_gate(&mp, &mp_facts, xfs_id, op),
            ProtectionGate::Unsupported {
                ground: RefusalGround::InheritedDeviceScope
            },
            "{op:?}: the name carries the multipath scope with no edge"
        );
    }

    // 2. A recognized-remote device's transport arm, the edge omitted:
    //    the network-block-device non-goal, no longer escaped by
    //    dropping one edge.
    let remote = NamingFields::PhysicalDevice {
        serial: Some(b"NBD0".to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    };
    let remote_id = derive_id(&remote).expect("derivable");
    let ext4 = NamingFields::FileSystem {
        host: remote_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let ext4_id = derive_id(&ext4).expect("derivable");
    let mut transports = BTreeMap::new();
    transports.insert(remote_id, TransportClass::RecognizedRemote);
    let remote_facts = Facts {
        transports,
        ..Facts::default()
    };
    let remote_topology = Topology::build(vec![remote, ext4], vec![]).expect("builds with no edge");
    for op in mutating_operations() {
        assert_ne!(
            protection_gate(&remote_topology, &remote_facts, ext4_id, op),
            ProtectionGate::Clear,
            "{op:?}: a file system naming a remote device inherits its scope"
        );
    }

    // 3. The producing relation: a volume naming a ZFS pool as its
    //    producer, the Production edge omitted, inherits the pool's own
    //    refusal rather than reading Permitted.
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let zvol = NamingFields::Volume {
        producer: pool_id,
        name: b"vol0".to_vec(),
        role: None,
    };
    let zvol_id = derive_id(&zvol).expect("derivable");
    let zfs = Topology::build(vec![pool, zvol], vec![]).expect("builds with no edge");
    let zfs_facts = Facts::default();
    assert!(
        matches!(
            node_verdict(&zfs, &zfs_facts, zvol_id),
            Verdict::Refused {
                ground: RefusalGround::InheritedFromConsumerOrProducer
            }
        ),
        "a volume naming its producer inherits the producer's refusal with no edge"
    );
    for op in mutating_operations() {
        assert_ne!(
            protection_gate(&zfs, &zfs_facts, zvol_id, op),
            ProtectionGate::Clear,
            "{op:?}: the producing name is not escaped by omitting the edge"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   The enumeration lens (ADR-0046's shape, issue #397): every committed
//   layout, every containment-bearing node in it, one edge removed at a
//   time — the node's verdict is never weakened by the removal. A
//   fixture is one shape; the lens is what catches a partial fix that
//   handles the kind it was written against and no other.
// Evidence: removing_any_one_containment_edge_never_weakens_a_verdict
#[test]
fn removing_any_one_containment_edge_never_weakens_a_verdict() {
    let root = root_on_zfs();
    let luks = luks_chain();
    let mdraid = partitioned_mdraid();
    let layouts: [(&str, &Topology, &Facts); 3] = [
        ("root_on_zfs", &root.topology, &root.facts),
        ("luks_chain", &luks.topology, &luks.facts),
        ("partitioned_mdraid", &mdraid.topology, &mdraid.facts),
    ];
    let mut checked = 0_usize;
    for (name, topology, facts) in layouts {
        let entries: Vec<NamingFields> = topology
            .entries()
            .iter()
            .map(|entry| match entry {
                super::naming::NodeEntry::Single { fields, .. }
                | super::naming::NodeEntry::Group { fields, .. } => fields.clone(),
            })
            .collect();
        let edges: Vec<Edge> = topology.edges().to_vec();
        for (index, dropped) in edges.iter().enumerate() {
            if dropped.kind != EdgeKind::Containment {
                continue;
            }
            let mut thinned = edges.clone();
            thinned.remove(index);
            let Ok(without) = Topology::build(entries.clone(), thinned) else {
                continue;
            };
            let rank = |verdict: &Verdict| match verdict {
                Verdict::Permitted => 0_u8,
                Verdict::Indeterminate { .. } => 1,
                Verdict::Refused { .. } => 2,
            };
            let before = node_verdict(topology, facts, dropped.target);
            let after = node_verdict(&without, facts, dropped.target);
            assert!(
                rank(&after) >= rank(&before),
                "{name}: dropping a containment edge weakened {:?} from {before:?} to {after:?}",
                dropped.target
            );
            checked += 1;
        }
    }
    assert!(checked >= 6, "the lens examined {checked} removals");
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   ADR-0048, issue #392: an extentless target is destroyed by identity.
//   Both halves are load-bearing and each is asserted on the arm only it
//   reaches. The whole-frame entry range-destroys what is framed on the
//   target, which is how the volume's table, partition, signature and
//   pool are reached; the identity seed destroys the target itself,
//   which is how destruction carries from an aggregate along production
//   to the volume it produces — nothing is framed on an aggregate, so
//   the entry alone leaves it Clear. The two controls that must not move
//   are pinned beside them: a whole-disk wipe, and the reach-only Label.
// Evidence: an_extentless_target_is_destroyed_by_identity
#[test]
fn an_extentless_target_is_destroyed_by_identity() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    let m = partitioned_mdraid();

    // 1. The volume: its children are framed on it, so the whole-frame
    //    entry reaches them and the pool below.
    let ranges = canonical_ranges(Operation::Wipe, m.md0, &m.facts);
    assert_eq!(
        ranges.destroyed,
        vec![HostRange {
            host: m.md0,
            start: 0,
            length: u64::MAX
        }],
        "an extentless target's destroyed entry is its whole frame"
    );
    let affected = affected_set(&m.topology, &m.facts, m.md0, &ranges);
    for reached in [m.table, m.md0p1, m.zfs_signature, m.pool] {
        assert!(
            affected.contains(&reached),
            "the volume's wipe reaches everything framed on it, and the pool below"
        );
    }

    // 2. The aggregate: nothing is framed on it, so only the identity
    //    seed carries destruction into the volume it produces.
    let array_ranges = canonical_ranges(Operation::Wipe, m.array, &m.facts);
    let array_affected = affected_set(&m.topology, &m.facts, m.array, &array_ranges);
    assert!(
        array_affected.contains(&m.pool),
        "destruction carries from an aggregate along production (the identity seed)"
    );

    // 3. Both refuse the four destroying operations over the live pool.
    //    The six that write declare no destroyed range and so still only
    //    reach — that is ADR-0039's distinction, unchanged here, and the
    //    reason this loop names four operations rather than ten.
    for target in [m.md0, m.array] {
        for op in destroying_operations() {
            assert_ne!(
                protection_gate(&m.topology, &m.facts, target, op),
                ProtectionGate::Clear,
                "{op:?} on an extentless target over a live pool"
            );
        }
        for op in [Operation::Label, Operation::Uuid] {
            assert_eq!(
                protection_gate(&m.topology, &m.facts, target, op),
                ProtectionGate::Clear,
                "{op:?} destroys nothing, so it reaches without releasing"
            );
        }
    }

    // 4. The controls, unmoved. A whole-disk wipe reaches what it always
    //    did, and a Label — which destroys nothing — still reaches the
    //    table without releasing the partition under it.
    let wipe_sda = canonical_ranges(Operation::Wipe, m.sda, &m.facts);
    let wiped = affected_set(&m.topology, &m.facts, m.sda, &wipe_sda);
    for reached in [m.md_signature, m.array, m.md0, m.table, m.md0p1, m.pool] {
        assert!(
            wiped.contains(&reached),
            "the whole-disk control is unmoved"
        );
    }
    let label_sda = canonical_ranges(Operation::Label, m.sda, &m.facts);
    let labelled = affected_set(&m.topology, &m.facts, m.sda, &label_sda);
    assert!(
        labelled.contains(&m.table) && !labelled.contains(&m.md0p1) && !labelled.contains(&m.pool),
        "reach is not destruction: Label reaches the table and releases nothing"
    );
}

// Requirements: MODEL-002, SAFE-005
//   The seed's second source reads the step's declared ranges and the
//   absence of an extent, so it is measured against issue #319's
//   population — a body whose extent-bearing node has no extent fact —
//   rather than argued about. `canonical_ranges` cannot tell "this kind
//   carries no extent" from "this extent is absent", so the whole-frame
//   entry lands on both, and the question is whether it can ever open a
//   gate. For every node ADR-0048's own arms read it cannot. The one
//   shape that does open gates is issue #319's third measured shape and
//   is pinned here as an open limit, not fixed: this act does not close
//   that issue and must not read as though it had.
// Evidence: the_identity_seed_never_weakens_a_gate_on_an_absent_extent
#[test]
fn the_identity_seed_never_weakens_a_gate_on_an_absent_extent() {
    use super::capability::{ProtectionGate, protection_gate};
    let m = partitioned_mdraid();

    let clear_count = |facts: &Facts, target: NodeId| {
        mutating_operations()
            .into_iter()
            .filter(|op| protection_gate(&m.topology, facts, target, *op) == ProtectionGate::Clear)
            .count()
    };

    // The nodes this act's arms read: the frame root, a partition framed
    // on the volume, and the table. Dropping any one never opens a gate.
    for victim in [m.sda, m.md0p1, m.table] {
        let mut thinned = m.facts.clone();
        thinned.extents.remove(&victim);
        for target in [m.sda, m.md0p1, m.md0, m.array] {
            let honest = clear_count(&m.facts, target);
            let absent = clear_count(&thinned, target);
            assert!(
                absent <= honest,
                "removing an extent fact opened {} gate(s) on {target:?}",
                absent.saturating_sub(honest)
            );
        }
    }

    // Issue #319's third measured shape, pinned as an OPEN limit. The ZFS
    // signature is reached only by the byte scan, so removing its extent
    // removes the one route to the pool and every gate opens — at HEAD
    // before this act and at HEAD after it. ADR-0048 neither causes this
    // nor repairs it; range-reach remains extent-only
    // (`protection.rs`, the `facts.extents.get(&id)` arm). Pinned so that
    // closing issue #319 is a deliberate change and never a drift.
    let mut no_signature = m.facts.clone();
    no_signature.extents.remove(&m.zfs_signature);
    for target in [m.sda, m.md0p1, m.md0, m.array] {
        assert_eq!(
            clear_count(&no_signature, target),
            10,
            "issue #319's open shape: an unlocated signature hides the pool from {target:?}"
        );
    }
}

// Requirements: MODEL-002, SAFE-005
//   The identity seed is frame-equal, not merely non-empty. `affected_set`
//   is public over caller-supplied `StepRanges`, and a plan step declares
//   its own ranges rather than the canonical ones, so a step destroying
//   bytes in some other frame must not destroy an extentless target that
//   happens to be in the same body. `canonical_ranges` always frames its
//   whole-frame entry on the target, so this distinction is invisible
//   through the gate and is pinned here at the closure instead.
// Evidence: the_identity_seed_is_frame_equal_not_merely_non_empty
#[test]
fn the_identity_seed_is_frame_equal_not_merely_non_empty() {
    let m = partitioned_mdraid();

    // A destroyed range on the DEVICE's frame, with the volume as target.
    let elsewhere = StepRanges {
        destroyed: vec![HostRange {
            host: m.sda,
            start: 0,
            length: 1 << 30,
        }],
        ..StepRanges::default()
    };
    let affected = affected_set(&m.topology, &m.facts, m.md0, &elsewhere);
    assert!(
        !affected.contains(&m.pool),
        "a range in another frame does not destroy an extentless target"
    );

    // The same range framed on the volume does destroy it.
    let here = StepRanges {
        destroyed: vec![HostRange {
            host: m.md0,
            start: 0,
            length: 1 << 30,
        }],
        ..StepRanges::default()
    };
    let affected = affected_set(&m.topology, &m.facts, m.md0, &here);
    assert!(
        affected.contains(&m.pool),
        "a range framed on the target destroys it by identity"
    );
}

/// The host-backed chain issue #409 measured: a device carrying an ext4
/// file system, an image file on that file system, the loop volume the
/// image produces, and a live ZFS pool on the volume. Every fact is
/// present and the body validates; the image's extent is framed on the
/// file system, where `ExtentLocator`'s own contract puts it.
struct LoopBacked {
    topology: Topology,
    facts: Facts,
    host: NodeId,
    host_fs: NodeId,
    image: NodeId,
    loop0: NodeId,
    sig: NodeId,
    pool: NodeId,
}

#[allow(clippy::too_many_lines)]
fn loop_backed() -> LoopBacked {
    use super::naming::{ExtentLocator, FileSystemKind};
    let host = device(b"LOOPHOST");
    let host_id = derive_id(&host).expect("derivable");
    let host_fs = NamingFields::FileSystem {
        host: host_id,
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

    let topology = Topology::build(
        vec![host, host_fs, image, loop0, sig, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
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
    )
    .expect("the loop-backed body builds");

    let mut extents = BTreeMap::new();
    extents.insert(
        host_id,
        HostRange {
            host: host_id,
            start: 0,
            length: 1 << 30,
        },
    );
    extents.insert(
        host_fs_id,
        HostRange {
            host: host_id,
            start: 0,
            length: 1 << 30,
        },
    );
    extents.insert(
        image_id,
        HostRange {
            host: host_fs_id,
            start: 0,
            length: 1 << 29,
        },
    );
    extents.insert(
        sig_id,
        HostRange {
            host: loop0_id,
            start: 0,
            length: MIB,
        },
    );
    let mut transports = BTreeMap::new();
    transports.insert(host_id, TransportClass::Sata);
    LoopBacked {
        topology,
        facts: Facts {
            extents,
            transports,
            ..Facts::default()
        },
        host: host_id,
        host_fs: host_fs_id,
        image: image_id,
        loop0: loop0_id,
        sig: sig_id,
        pool: pool_id,
    }
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   ADR-0049, issue #409: reach follows the hosting name. A backing
//   extent is the target of no edge kind — the pair table admits it only
//   as the source of `HostBacking` — so before this arm the whole
//   host-backed class had no upward reach and a wipe of the disk holding
//   a loop or VHD image gated Clear over the live pool on the volume that
//   image backed. Both targets the defect reaches are asserted: the
//   device, which the issue filed, and the file system, which it did not.
// Evidence: reach_follows_the_hosting_name_into_a_backing_extent
#[test]
fn reach_follows_the_hosting_name_into_a_backing_extent() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    use super::protection::validate_facts;
    let b = loop_backed();

    assert_eq!(
        validate_facts(&b.topology, &b.facts),
        Ok(()),
        "the image's extent is framed on the file system and the body is lawful"
    );

    for (name, target) in [("the device", b.host), ("its file system", b.host_fs)] {
        let ranges = canonical_ranges(Operation::Wipe, target, &b.facts);
        let affected = affected_set(&b.topology, &b.facts, target, &ranges);
        for reached in [b.image, b.loop0, b.sig, b.pool] {
            assert!(
                affected.contains(&reached),
                "wiping {name} reaches the image it holds, and the pool below it"
            );
        }
        for op in mutating_operations() {
            assert_ne!(
                protection_gate(&b.topology, &b.facts, target, op),
                ProtectionGate::Clear,
                "{op:?} on {name} refuses: a live pool hangs off the image it holds"
            );
        }
    }
}

// Requirements: MODEL-002, SAFE-005
//   The new arm is bounded exactly as containment is, and it adds reach
//   without adding a route back up. A wipe of the image reaches what the
//   image carries and never its host — the arm descends from host to
//   backing extent, not the reverse — which is what keeps a 512 MiB image
//   file from reading as destruction of the disk that holds it. That
//   asymmetry is the whole reason this route was taken over admitting a
//   containment pair, where the extent would have to be reframed onto the
//   device and the reverse reach followed.
// Evidence: the_hosting_arm_descends_only_and_stays_bounded
#[test]
fn the_hosting_arm_descends_only_and_stays_bounded() {
    use super::capability::{Operation, canonical_ranges};
    let b = loop_backed();

    let ranges = canonical_ranges(Operation::Wipe, b.image, &b.facts);
    let affected = affected_set(&b.topology, &b.facts, b.image, &ranges);
    assert!(
        affected.contains(&b.loop0) && affected.contains(&b.pool),
        "the image's own wipe still reaches what it backs"
    );
    assert!(
        !affected.contains(&b.host) && !affected.contains(&b.host_fs),
        "and never its host: the arm descends, so an image file is not its disk"
    );

    // The bound: an image whose declared geometry positively contradicts
    // containment in its host's frame is not descended into. The file
    // system spans the whole device; an image claiming bytes beyond it,
    // in that same frame, is refused the hop.
    let mut beyond = b.facts.clone();
    beyond.extents.insert(
        b.image,
        HostRange {
            host: b.host,
            start: 2 << 30,
            length: 1 << 29,
        },
    );
    let ranges = canonical_ranges(Operation::Wipe, b.host_fs, &beyond);
    let affected = affected_set(&b.topology, &beyond, b.host_fs, &ranges);
    assert!(
        !affected.contains(&b.image),
        "the hop is refused where the geometry positively contradicts containment"
    );
}

// Requirements: MODEL-002, SAFE-005
//   Reach is not destruction across the hosting arm either. A step that
//   destroys nothing reaches the image its host holds — ADR-0039's
//   carried content — but must not destroy it, because destroying a
//   backing extent carries down into the volume it backs and releases
//   the partitions a table there describes (ADR-0043, ADR-0044). The
//   distinction is invisible on a chain with no table, which is why this
//   fixture puts one on the loop volume: with it, a Label on the disk
//   that merely holds the image would otherwise release a partition
//   inside that image.
// Evidence: the_hosting_arm_reaches_without_destroying
#[test]
fn the_hosting_arm_reaches_without_destroying() {
    use super::capability::{Operation, canonical_ranges};
    use super::naming::{ExtentLocator, FileSystemKind, TableRole};

    let host = device(b"LOOPTBL");
    let host_id = derive_id(&host).expect("derivable");
    let host_fs = NamingFields::FileSystem {
        host: host_id,
        kind: FileSystemKind::Ext4,
        superblock_offset: 1024,
    };
    let host_fs_id = derive_id(&host_fs).expect("derivable");
    let image = NamingFields::BackingExtent {
        host: host_fs_id,
        locator: ExtentLocator::Path {
            bytes: b"/srv/images/disk.img".to_vec(),
        },
    };
    let image_id = derive_id(&image).expect("derivable");
    let loop0 = NamingFields::Volume {
        producer: image_id,
        name: b"loop0".to_vec(),
        role: None,
    };
    let loop0_id = derive_id(&loop0).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: loop0_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: MIB,
    };
    let part_id = derive_id(&part).expect("derivable");

    let topology = Topology::build(
        vec![host, host_fs, image, loop0, table, part],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: host_id,
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

    let mut extents = BTreeMap::new();
    for (node, host_of, start, length) in [
        (host_id, host_id, 0, 1 << 30),
        (host_fs_id, host_id, 0, 1 << 30),
        (image_id, host_fs_id, 0, 1 << 29),
        (table_id, loop0_id, 0, MIB),
        (part_id, loop0_id, MIB, 1 << 28),
    ] {
        extents.insert(
            node,
            HostRange {
                host: host_of,
                start,
                length,
            },
        );
    }
    let mut transports = BTreeMap::new();
    transports.insert(host_id, TransportClass::Sata);
    let facts = Facts {
        extents,
        transports,
        ..Facts::default()
    };

    // A wipe of the disk destroys the image, which carries down and
    // releases the partition the table inside it describes.
    let wipe = canonical_ranges(Operation::Wipe, host_id, &facts);
    let destroyed_reach = affected_set(&topology, &facts, host_id, &wipe);
    assert!(
        destroyed_reach.contains(&image_id) && destroyed_reach.contains(&part_id),
        "a wipe of the disk reaches the image and releases what its table describes"
    );

    // A label destroys nothing. It still reaches the image — carried
    // content — but must release nothing inside it.
    let label = canonical_ranges(Operation::Label, host_id, &facts);
    let reach_only = affected_set(&topology, &facts, host_id, &label);
    assert!(
        reach_only.contains(&image_id),
        "a label reaches the image the disk holds"
    );
    assert!(
        !reach_only.contains(&part_id),
        "and releases nothing inside it: reach is not destruction"
    );
}

// Requirements: MODEL-002, SAFE-005, CAP-003
//   The flagship population, and the shape the first draft of this act
//   was inert on. An `ExtentLocator::Path` image has no contiguous device
//   range — that is this ADR's own argument against the rejected route —
//   so the natural body declares no extent for it at all, and nothing
//   requires one. Routing the arm through containment's absent-child
//   carve-out made honest absence fail OPEN: the body validated and both
//   targets gated Clear on ten of ten over a live pool. Absence admits.
// Evidence: the_hosting_arm_admits_a_backing_extent_that_declares_no_extent
#[test]
fn the_hosting_arm_admits_a_backing_extent_that_declares_no_extent() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    use super::protection::validate_facts;
    let b = loop_backed();

    let mut unlocated = b.facts.clone();
    unlocated.extents.remove(&b.image);
    assert_eq!(
        validate_facts(&b.topology, &unlocated),
        Ok(()),
        "an image with no extent fact is a lawful body: nothing requires one"
    );

    for (name, target) in [("the device", b.host), ("its file system", b.host_fs)] {
        let ranges = canonical_ranges(Operation::Wipe, target, &unlocated);
        let affected = affected_set(&b.topology, &unlocated, target, &ranges);
        for reached in [b.image, b.loop0, b.sig, b.pool] {
            assert!(
                affected.contains(&reached),
                "wiping {name} reaches an unlocated image and the pool below it"
            );
        }
        for op in mutating_operations() {
            assert_ne!(
                protection_gate(&b.topology, &unlocated, target, op),
                ProtectionGate::Clear,
                "{op:?} on {name}: absence of an extent must not subtract reach"
            );
        }
    }
}

// Requirements: MODEL-002, SAFE-005
//   A measured OPEN LIMIT, pinned so that closing it is deliberate.
//   Nothing authenticates a backing extent's frame — its `host` naming
//   field is `ReferentRule::Open`, so `named_position` is `Outside`,
//   `frame_root` is `None`, ADR-0046's frame rule never runs on it, and
//   no edge may target it so the edge-versus-extent cross-check never
//   sees it either. An author may therefore frame the image's extent on
//   the host's own frame root, place it outside the host, and suppress
//   the hop on a body that validates. This is issue #365's undecided
//   question reaching into this arm. It is recorded here at its measured
//   cost rather than claimed absent, and it does not make protection
//   worse than it was before this act: the arm only ever adds reach.
// Evidence: an_authored_frame_can_still_suppress_the_hosting_arm
#[test]
fn an_authored_frame_can_still_suppress_the_hosting_arm() {
    use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
    use super::protection::validate_facts;
    let b = loop_backed();

    // Framed on the device — the file system's own frame root — and
    // placed beyond the file system's bytes.
    let mut authored = b.facts.clone();
    authored.extents.insert(
        b.image,
        HostRange {
            host: b.host,
            start: 2 << 30,
            length: 1 << 29,
        },
    );
    assert_eq!(
        validate_facts(&b.topology, &authored),
        Ok(()),
        "the body validates: nothing constrains a backing extent's frame"
    );

    for target in [b.host, b.host_fs] {
        let ranges = canonical_ranges(Operation::Wipe, target, &authored);
        let affected = affected_set(&b.topology, &authored, target, &ranges);
        assert!(
            !affected.contains(&b.pool),
            "the open limit: an authored frame suppresses the hop"
        );
        let clear = mutating_operations()
            .into_iter()
            .filter(|op| {
                protection_gate(&b.topology, &authored, target, *op) == ProtectionGate::Clear
            })
            .count();
        assert_eq!(
            clear, 10,
            "and its measured cost is ten of ten Clear, pinned under issue #365"
        );
    }
}
