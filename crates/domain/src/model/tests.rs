//! Tests for node naming (WP-010 increment 3a, ADR-0019).

use super::naming::{
    AggregateTechnology, ExtentLocator, FileSystemKind, NamingFields, NodeEntry, SignatureFamily,
    TableRole, absorb, derive_id,
};

fn device(serial: Option<&[u8]>, wwn: Option<&[u8]>, total_bytes: u64) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: serial.map(<[u8]>::to_vec),
        wwn: wwn.map(<[u8]>::to_vec),
        total_bytes,
    }
}

// Requirements: MODEL-005
//   The derivation is a pure function of the fields: same input, same address.
// Evidence: derivation_is_deterministic
#[test]
fn derivation_is_deterministic() {
    let fields = device(Some(b"S1"), Some(b"W1"), 1 << 30);
    let first = derive_id(&fields).expect("derivable");
    let second = derive_id(&fields).expect("derivable");
    assert_eq!(first, second);
}

// Requirements: MODEL-005
//   Distinct contract-source identifier bytes derive distinct addresses.
// Evidence: distinct_serials_derive_distinct_addresses
#[test]
fn distinct_serials_derive_distinct_addresses() {
    let a = derive_id(&device(Some(b"S1"), None, 1 << 30)).expect("derivable");
    let b = derive_id(&device(Some(b"S2"), None, 1 << 30)).expect("derivable");
    assert_ne!(a, b);
}

// Requirements: MODEL-005, SAFE-005
//   ADR-0019's collision group on the measured L9 byte-identical pair:
//   byte-identical simultaneous devices absorb into one counted group; the
//   snapshot still exists and the limitation attaches to the pair.
// Evidence: byte_identical_devices_absorb_into_a_counted_group
#[test]
fn byte_identical_devices_absorb_into_a_counted_group() {
    let stick = || device(Some(b"0000"), None, 128 << 30);
    let entries = absorb(vec![stick(), stick()]).expect("absorbable");
    assert_eq!(entries.len(), 1);
    match &entries[0] {
        NodeEntry::Group {
            count,
            duplicate_designator,
            ..
        } => {
            assert_eq!(*count, 2);
            assert!(!duplicate_designator);
        }
        NodeEntry::Single { .. } => panic!("byte-identical pair must group"),
    }
}

// Requirements: MODEL-005
//   A node's address depends only on itself and its ancestors: adding an
//   unrelated device with its own subtree changes no existing address.
// Evidence: an_address_depends_only_on_the_node_and_its_ancestors
#[test]
fn an_address_depends_only_on_the_node_and_its_ancestors() {
    let sda = device(Some(b"SDA"), None, 1 << 40);
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let part = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let part_id = derive_id(&part).expect("derivable");

    // A second device arrives, carrying its own subtree.
    let sdb = device(Some(b"SDB"), None, 1 << 40);
    let second_device_id = derive_id(&sdb).expect("derivable");
    let sdb_table = NamingFields::PartitionTable {
        parent: second_device_id,
        role: TableRole::Gpt,
    };
    let entries = absorb(vec![
        sda.clone(),
        table.clone(),
        part.clone(),
        sdb,
        sdb_table,
    ])
    .expect("absorbable");

    // The original addresses are untouched by the arrival.
    assert_eq!(derive_id(&sda).expect("derivable"), sda_id);
    assert_eq!(derive_id(&table).expect("derivable"), table_id);
    assert_eq!(derive_id(&part).expect("derivable"), part_id);
    assert!(entries.iter().any(|entry| entry.id() == part_id));
}

// Requirements: MODEL-005
//   Absorption is a deterministic function of the observed multiset,
//   independent of enumeration order.
// Evidence: absorption_is_independent_of_input_order
#[test]
fn absorption_is_independent_of_input_order() {
    let a = device(Some(b"A"), None, 1);
    let b = device(Some(b"B"), None, 2);
    let c = device(Some(b"A"), None, 1);
    let forward = absorb(vec![a.clone(), b.clone(), c.clone()]).expect("absorbable");
    let reversed = absorb(vec![c, b, a]).expect("absorbable");
    assert_eq!(forward, reversed);
}

// Requirements: MODEL-005
//   Absorption is total and counts correctly: n equal nodes, one entry,
//   count n.
// Evidence: absorption_counts_every_colliding_member
#[test]
fn absorption_counts_every_colliding_member() {
    let clone = || device(None, None, 32 << 30);
    let entries = absorb(vec![clone(), clone(), clone()]).expect("absorbable");
    assert_eq!(entries.len(), 1);
    match &entries[0] {
        NodeEntry::Group { count, .. } => assert_eq!(*count, 3),
        NodeEntry::Single { .. } => panic!("three equal nodes must group"),
    }
}

// Requirements: MODEL-005, FS-008
//   A cloned aggregate pair groups flagged, and a child named from the
//   shared address keeps its address when the clone arrives — nothing
//   re-designates.
// Evidence: duplicate_designator_groups_flag_and_nothing_re_designates
#[test]
fn duplicate_designator_groups_flag_and_nothing_re_designates() {
    let vg = || NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: Some(b"vg-uuid-bytes".to_vec()),
    };
    let vg_id = derive_id(&vg()).expect("derivable");
    let lv = NamingFields::Volume {
        producer: vg_id,
        name: b"lv_home".to_vec(),
        role: None,
    };
    let lv_id_before = derive_id(&lv).expect("derivable");

    // The clone is attached: the pair groups, flagged.
    let entries = absorb(vec![vg(), vg()]).expect("absorbable");
    match &entries[0] {
        NodeEntry::Group {
            id,
            count,
            duplicate_designator,
            ..
        } => {
            assert_eq!(*id, vg_id, "the group carries the shared address");
            assert_eq!(*count, 2);
            assert!(*duplicate_designator);
        }
        NodeEntry::Single { .. } => panic!("duplicate designators must group"),
    }

    // The volume's address is derived from the shared address and is
    // unchanged by the clone's arrival.
    assert_eq!(derive_id(&lv).expect("derivable"), lv_id_before);
}

// Requirements: MODEL-002, INV-008
//   The stale pair is two addresses on one host: a live ext4 file system at
//   its superblock offset and an end-anchored mdraid 0.90 signature are
//   distinct nodes — what round three's device projection could not
//   represent.
// Evidence: the_stale_pair_is_two_addresses
#[test]
fn the_stale_pair_is_two_addresses() {
    let host = derive_id(&device(Some(b"HOST"), None, 4 << 20)).expect("derivable");
    let live_ext4 = NamingFields::FileSystem {
        host,
        kind: FileSystemKind::Ext4,
        superblock_offset: 0x438,
    };
    let stale_raid = NamingFields::BackingSignature {
        host,
        family: SignatureFamily::Mdraid09,
        primary_offset: 0x3f_0000,
    };
    let a = derive_id(&live_ext4).expect("derivable");
    let b = derive_id(&stale_raid).expect("derivable");
    assert_ne!(a, b);
    let entries = absorb(vec![live_ext4, stale_raid]).expect("absorbable");
    assert_eq!(entries.len(), 2);
}

// Requirements: MODEL-002
//   A hybrid table's aliased extent is two addresses under two view roles —
//   parent-plus-offset injectivity restored by re-parenting onto the table.
// Evidence: a_hybrid_aliased_extent_is_two_addresses_under_two_views
#[test]
fn a_hybrid_aliased_extent_is_two_addresses_under_two_views() {
    let dev = derive_id(&device(Some(b"D"), None, 1 << 30)).expect("derivable");
    let gpt_view = derive_id(&NamingFields::PartitionTable {
        parent: dev,
        role: TableRole::Gpt,
    })
    .expect("derivable");
    let hybrid_view = derive_id(&NamingFields::PartitionTable {
        parent: dev,
        role: TableRole::HybridMbr,
    })
    .expect("derivable");
    let esp = NamingFields::Partition {
        parent_table: gpt_view,
        start_offset: 1 << 20,
    };
    let alias = NamingFields::Partition {
        parent_table: hybrid_view,
        start_offset: 1 << 20,
    };
    assert_ne!(
        derive_id(&esp).expect("derivable"),
        derive_id(&alias).expect("derivable")
    );
}

// Requirements: MODEL-002
//   Unrecognized discriminants carry their raw bytes, so two distinct
//   unknown values never share an address, and a crafted raw value never
//   collides with a known tag.
// Evidence: unrecognized_discriminants_are_distinct_by_raw_bytes
#[test]
fn unrecognized_discriminants_are_distinct_by_raw_bytes() {
    let host = derive_id(&device(Some(b"H"), None, 1 << 30)).expect("derivable");
    let sig = |raw: &[u8]| NamingFields::BackingSignature {
        host,
        family: SignatureFamily::Unrecognized { raw: raw.to_vec() },
        primary_offset: 0,
    };
    let known = NamingFields::BackingSignature {
        host,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let a = derive_id(&sig(b"vendorA")).expect("derivable");
    let b = derive_id(&sig(b"vendorB")).expect("derivable");
    let k = derive_id(&known).expect("derivable");
    let crafted = derive_id(&sig(b"zfs")).expect("derivable");
    assert_ne!(a, b);
    assert_ne!(a, k);
    assert_ne!(
        crafted, k,
        "raw bytes and a known tag are different domains"
    );
}

// Requirements: MODEL-005
//   Designator-less aggregates of one technology collide with each other
//   and with nothing else; the group is representable, not an error.
// Evidence: designator_absent_aggregates_group_among_themselves
#[test]
fn designator_absent_aggregates_group_among_themselves() {
    let orphan = || NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: None,
    };
    let named = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"pool-guid".to_vec()),
    };
    let entries = absorb(vec![orphan(), orphan(), named]).expect("absorbable");
    assert_eq!(entries.len(), 2);
    let groups: Vec<_> = entries
        .iter()
        .filter(|entry| matches!(entry, NodeEntry::Group { .. }))
        .collect();
    assert_eq!(groups.len(), 1);
    match groups[0] {
        NodeEntry::Group {
            count,
            duplicate_designator,
            ..
        } => {
            assert_eq!(*count, 2);
            assert!(
                !duplicate_designator,
                "absent designators are not the duplicate-designator case"
            );
        }
        NodeEntry::Single { .. } => unreachable!(),
    }
}

// Requirements: MODEL-005
//   A backing extent names from its host and locator; two loop devices
//   backed by distinct files are distinct addresses — round three's
//   equal-size fixture collision is gone.
// Evidence: distinct_backing_files_are_distinct_addresses
#[test]
fn distinct_backing_files_are_distinct_addresses() {
    let fs = derive_id(&device(Some(b"FS-HOST"), None, 1 << 40)).expect("derivable");
    let file = |path: &[u8]| NamingFields::BackingExtent {
        host: fs,
        locator: ExtentLocator::Path {
            bytes: path.to_vec(),
        },
    };
    let a = derive_id(&file(b"/img/a.img")).expect("derivable");
    let b = derive_id(&file(b"/img/b.img")).expect("derivable");
    assert_ne!(a, b);
}
