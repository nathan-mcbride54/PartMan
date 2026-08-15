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
