//! Tests for the protection gate (WP-010 increment 3g, ADR-0018's
//! canonical-step rule).

use std::collections::BTreeMap;

use super::capability::{
    Operation, OperationClass, ProtectionGate, canonical_ranges, protection_gate,
};
use super::naming::{AggregateTechnology, NamingFields, SignatureFamily, TableRole, derive_id};
use super::protection::{Facts, HostRange, RefusalGround, TransportClass, step_constructs};
use super::topology::{Edge, EdgeKind, Topology};

struct Layout {
    topology: Topology,
    facts: Facts,
    sda: super::naming::NodeId,
    member: super::naming::NodeId,
    pool: super::naming::NodeId,
}

fn pool_layout() -> Layout {
    let sda = NamingFields::PhysicalDevice {
        serial: Some(b"SDA".to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    };
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
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
        designator: Some(b"tank".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![sda, table, member, signature, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: table_id,
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
    Layout {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
        },
        sda: sda_id,
        member: member_id,
        pool: pool_id,
    }
}

// Requirements: CAP-002
//   Every operation carries a class, and the partition is exactly
//   ADR-0018's: detect, read, check, and copy-as-source are source
//   class; the other ten are mutating.
// Evidence: operation_classes_match_the_decided_partition
#[test]
fn operation_classes_match_the_decided_partition() {
    let mut source = 0;
    let mut mutating = 0;
    for operation in Operation::all() {
        match operation.class() {
            OperationClass::Source => source += 1,
            OperationClass::Mutating => mutating += 1,
        }
    }
    assert_eq!(source, 4);
    assert_eq!(mutating, 10);
    assert_eq!(Operation::Copy.class(), OperationClass::Source);
    assert_eq!(Operation::Wipe.class(), OperationClass::Mutating);
}

// Requirements: CAP-001, CAP-005
//   The gate and the constructor run one closure: for every mutating
//   operation on every node in the layout, the gate is Clear exactly
//   when the canonical step constructs — agreement by construction,
//   enumerated.
// Evidence: the_gate_agrees_with_the_constructor_on_every_pair
#[test]
fn the_gate_agrees_with_the_constructor_on_every_pair() {
    let layout = pool_layout();
    for entry in layout.topology.entries() {
        let id = entry.id();
        for operation in Operation::all() {
            if operation.class() != OperationClass::Mutating {
                continue;
            }
            let gate = protection_gate(&layout.topology, &layout.facts, id, *operation);
            let ranges = canonical_ranges(*operation, id, &layout.facts);
            let constructs = step_constructs(&layout.topology, &layout.facts, id, &ranges).is_ok();
            assert_eq!(
                matches!(gate, ProtectionGate::Clear),
                constructs,
                "gate and constructor must agree for {operation:?} on {id}"
            );
        }
    }
}

// Requirements: CAP-002, SAFE-005
//   Source operations are never suppressed by a verdict: detect on the
//   pool itself is Clear, per WIN-003's detection duty and WIN-004's
//   copy-off escape.
// Evidence: source_operations_are_never_suppressed
#[test]
fn source_operations_are_never_suppressed() {
    let layout = pool_layout();
    for operation in [
        Operation::Detect,
        Operation::Read,
        Operation::Check,
        Operation::Copy,
    ] {
        assert_eq!(
            protection_gate(&layout.topology, &layout.facts, layout.pool, operation),
            ProtectionGate::Clear,
            "{operation:?} on the pool must stay available"
        );
    }
}

// Requirements: CAP-003, SAFE-005
//   The regime mapping: a mutating operation on the pool is unsupported
//   with the ZFS ground; wiping the member whose signature backs the
//   pool is unsupported through the reach; wiping the device is
//   unsupported through the reach.
// Evidence: mutating_operations_on_and_through_the_pool_are_unsupported
#[test]
fn mutating_operations_on_and_through_the_pool_are_unsupported() {
    let layout = pool_layout();
    assert!(matches!(
        protection_gate(
            &layout.topology,
            &layout.facts,
            layout.pool,
            Operation::Wipe
        ),
        ProtectionGate::Unsupported {
            ground: RefusalGround::Zfs
        }
    ));
    assert!(matches!(
        protection_gate(
            &layout.topology,
            &layout.facts,
            layout.member,
            Operation::Wipe
        ),
        ProtectionGate::Unsupported { .. }
    ));
    assert!(matches!(
        protection_gate(&layout.topology, &layout.facts, layout.sda, Operation::Wipe),
        ProtectionGate::Unsupported { .. }
    ));
}

// Requirements: CAP-003, SAFE-005
//   The indeterminate arm maps to blocked: a mutating operation on a
//   host carrying an orphan signature is Blocked, remediable — never
//   silently permitted and never unsupported-forever.
// Evidence: an_orphan_signature_blocks_rather_than_refuses
#[test]
fn an_orphan_signature_blocks_rather_than_refuses() {
    let host = NamingFields::PhysicalDevice {
        serial: Some(b"H".to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    };
    let host_id = derive_id(&host).expect("derivable");
    let orphan = NamingFields::BackingSignature {
        host: host_id,
        family: SignatureFamily::Mdraid09,
        primary_offset: 0,
    };
    let orphan_id = derive_id(&orphan).expect("derivable");
    let topology = Topology::build(
        vec![host, orphan],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: host_id,
            target: orphan_id,
        }],
    )
    .expect("builds");
    let mut facts = Facts::default();
    facts.transports.insert(host_id, TransportClass::Usb);
    facts.extents.insert(
        host_id,
        HostRange {
            host: host_id,
            start: 0,
            length: 1 << 30,
        },
    );
    facts.extents.insert(
        orphan_id,
        HostRange {
            host: host_id,
            start: (1 << 30) - (64 << 10),
            length: 64 << 10,
        },
    );
    assert!(matches!(
        protection_gate(&topology, &facts, host_id, Operation::Wipe),
        ProtectionGate::Blocked { .. }
    ));
}
