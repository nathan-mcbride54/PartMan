//! Tests for edges and topology construction (WP-010 increment 3b,
//! ADR-0019; ADR-0018's edge-semantics handover).

use super::naming::{
    AggregateTechnology, ExtentLocator, NamingFields, SignatureFamily, TableRole, derive_id,
};
use super::topology::{
    Edge, EdgeKind, SemanticsClass, Topology, TopologyError, endpoint_pair_allowed,
};

fn device(serial: &[u8]) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: 1 << 30,
    }
}

/// One node of every kind, with resolvable internal parents where naming
/// requires them.
fn one_of_each() -> Vec<NamingFields> {
    let dev = device(b"D0");
    let dev_id = derive_id(&dev).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let partition = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 1 << 20,
    };
    let partition_id = derive_id(&partition).expect("derivable");
    let signature = NamingFields::BackingSignature {
        host: partition_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let file_system = NamingFields::FileSystem {
        host: partition_id,
        kind: super::naming::FileSystemKind::Ext4,
        superblock_offset: 0x438,
    };
    let encryption = NamingFields::EncryptionLayer {
        backing_signature: signature_id,
    };
    let encryption_id = derive_id(&encryption).expect("derivable");
    let aggregate = NamingFields::Aggregate {
        technology: AggregateTechnology::Lvm2,
        designator: Some(b"vg".to_vec()),
    };
    let volume = NamingFields::Volume {
        producer: encryption_id,
        name: b"mapper0".to_vec(),
        role: None,
    };
    let extent = NamingFields::BackingExtent {
        host: partition_id,
        locator: ExtentLocator::Path {
            bytes: b"/img/a.img".to_vec(),
        },
    };
    let multipath = NamingFields::MultipathNode {
        lun_designator: b"naa-bytes".to_vec(),
    };
    let conflicting = NamingFields::ConflictingTableEntry {
        table: table_id,
        view_role: TableRole::HybridMbr,
        entry_start: 1 << 20,
    };
    vec![
        dev,
        table,
        partition,
        signature,
        file_system,
        encryption,
        aggregate,
        volume,
        extent,
        multipath,
        conflicting,
    ]
}

fn id_of(fields: &NamingFields) -> super::naming::NodeId {
    derive_id(fields).expect("derivable")
}

// Requirements: MODEL-002
//   ADR-0019's edge kinds under ADR-0018's semantics-class handover:
//   each kind carries its class; platform-membership alone is
//   platform-asserted and bind-inert in v1.
// Evidence: semantics_classes_match_the_handover
#[test]
fn semantics_classes_match_the_handover() {
    for kind in EdgeKind::all() {
        if *kind == EdgeKind::PlatformMembership {
            assert_eq!(kind.semantics(), SemanticsClass::PlatformAsserted);
            assert!(!kind.traversed_by_bind_set());
        } else {
            assert_eq!(kind.semantics(), SemanticsClass::BytesWithinOrDerive);
            assert!(kind.traversed_by_bind_set());
        }
    }
}

// Requirements: MODEL-002, CONC-001
//   A representative chain — containment, backing, production,
//   host-backing, platform-membership — builds, deterministically.
// Evidence: a_valid_chain_builds_deterministically
#[test]
fn a_valid_chain_builds_deterministically() {
    let nodes = one_of_each();
    let dev = id_of(&nodes[0]);
    let table = id_of(&nodes[1]);
    let partition = id_of(&nodes[2]);
    let signature = id_of(&nodes[3]);
    let encryption = id_of(&nodes[5]);
    let volume = id_of(&nodes[7]);
    let extent = id_of(&nodes[8]);
    let multipath = id_of(&nodes[9]);
    let edges = vec![
        Edge {
            kind: EdgeKind::Containment,
            source: dev,
            target: table,
        },
        Edge {
            kind: EdgeKind::Containment,
            source: table,
            target: partition,
        },
        Edge {
            kind: EdgeKind::Containment,
            source: partition,
            target: signature,
        },
        Edge {
            kind: EdgeKind::Backing,
            source: signature,
            target: encryption,
        },
        Edge {
            kind: EdgeKind::Production,
            source: encryption,
            target: volume,
        },
        Edge {
            kind: EdgeKind::HostBacking,
            source: extent,
            target: volume,
        },
        Edge {
            kind: EdgeKind::PlatformMembership,
            source: multipath,
            target: dev,
        },
    ];
    let first = Topology::build(nodes.clone(), edges.clone()).expect("builds");
    let second = Topology::build(nodes, edges).expect("builds");
    assert_eq!(first, second);
    assert_eq!(first.edges().len(), 7);
}

// Requirements: MODEL-005
//   ADR-0019's derived-never-stored rule, unknown referents rejected:
//   an edge naming an address no entry carries is a typed refusal.
// Evidence: an_unknown_referent_is_a_typed_refusal
#[test]
fn an_unknown_referent_is_a_typed_refusal() {
    let nodes = vec![device(b"D0")];
    let dev = id_of(&nodes[0]);
    let ghost = id_of(&device(b"GHOST"));
    let result = Topology::build(
        nodes,
        vec![Edge {
            kind: EdgeKind::Containment,
            source: dev,
            target: ghost,
        }],
    );
    assert_eq!(result, Err(TopologyError::UnknownReferent { id: ghost }));
}

// Requirements: MODEL-002
//   Self-edges and duplicate edges are typed refusals; the edge set is a
//   set.
// Evidence: self_edges_and_duplicates_are_refused
#[test]
fn self_edges_and_duplicates_are_refused() {
    let nodes = one_of_each();
    let dev = id_of(&nodes[0]);
    let table = id_of(&nodes[1]);
    let self_edge = Topology::build(
        nodes.clone(),
        vec![Edge {
            kind: EdgeKind::Containment,
            source: dev,
            target: dev,
        }],
    );
    assert_eq!(self_edge, Err(TopologyError::SelfEdge { id: dev }));
    let duplicated = Edge {
        kind: EdgeKind::Containment,
        source: dev,
        target: table,
    };
    let duplicate = Topology::build(nodes, vec![duplicated, duplicated]);
    assert_eq!(
        duplicate,
        Err(TopologyError::DuplicateEdge { edge: duplicated })
    );
}

// Requirements: MODEL-002
//   The no-sibling-capture theorem's premise, ADR-0018's handover
//   discharged as an enumeration: no backing, production, or
//   host-backing pair in the endpoint table targets a physical device.
// Evidence: no_backing_production_or_host_backing_pair_targets_a_device
#[test]
fn no_backing_production_or_host_backing_pair_targets_a_device() {
    let kinds = [
        "physical-device",
        "partition-table",
        "partition",
        "backing-signature",
        "file-system",
        "encryption-layer",
        "aggregate",
        "volume",
        "backing-extent",
        "multipath-node",
        "conflicting-table-entry",
    ];
    for kind in [
        EdgeKind::Backing,
        EdgeKind::Production,
        EdgeKind::HostBacking,
    ] {
        for source in kinds {
            assert!(
                !endpoint_pair_allowed(kind, source, "physical-device"),
                "{kind:?} from {source} must not target a physical device"
            );
        }
    }
}

// Requirements: MODEL-002, SAFE-005
//   The no-sibling-capture premise, generalized past the name
//   `physical-device` and enumerated over the table. ADR-0039's closure
//   descends out of a destroyed node without a geometric bound on the
//   three propagating arms, which is safe exactly while none of their
//   pairs can target a node that declares bytes of its own: a node with
//   no extent has no siblings to be confused with. The extent-bearing
//   set is read off `NamingFields::may_carry_extent` — the same
//   predicate the decode path enforces — so this cannot drift from the
//   rule it depends on, and a pair added to the table without a matching
//   decision reds here rather than silently widening the closure.
// Evidence: no_propagating_pair_targets_a_kind_that_declares_bytes
#[test]
fn no_propagating_pair_targets_a_kind_that_declares_bytes() {
    let nodes = one_of_each();
    for kind in [
        EdgeKind::Backing,
        EdgeKind::Production,
        EdgeKind::HostBacking,
    ] {
        for source in &nodes {
            for target in &nodes {
                if endpoint_pair_allowed(kind, source.kind_name(), target.kind_name()) {
                    assert!(
                        !target.may_carry_extent(),
                        "{kind:?} propagates without a geometric bound, so it may not target                          {}, which may declare an extent",
                        target.kind_name()
                    );
                }
            }
        }
    }
}

// Requirements: MODEL-002
//   Every (kind, source, target) triple outside the pair table is refused
//   at construction — enumerated, not sampled.
// Evidence: every_triple_outside_the_pair_table_is_refused
#[test]
fn every_triple_outside_the_pair_table_is_refused() {
    let nodes = one_of_each();
    let ids: Vec<_> = nodes.iter().map(id_of).collect();
    let kind_names: Vec<&'static str> = nodes.iter().map(NamingFields::kind_name).collect();
    let mut refused = 0_u32;
    let mut allowed = 0_u32;
    for kind in EdgeKind::all() {
        for (source_index, source) in ids.iter().enumerate() {
            for (target_index, target) in ids.iter().enumerate() {
                if source_index == target_index {
                    continue;
                }
                let edge = Edge {
                    kind: *kind,
                    source: *source,
                    target: *target,
                };
                let result = Topology::build(nodes.clone(), vec![edge]);
                if endpoint_pair_allowed(*kind, kind_names[source_index], kind_names[target_index])
                {
                    assert!(result.is_ok(), "{kind:?} pair in table must build");
                    allowed += 1;
                } else {
                    assert_eq!(
                        result,
                        Err(TopologyError::ForbiddenEndpoint {
                            kind: *kind,
                            source_kind: kind_names[source_index],
                            target_kind: kind_names[target_index],
                        }),
                        "{kind:?} pair outside table must refuse"
                    );
                    refused += 1;
                }
            }
        }
    }
    assert!(allowed >= 14, "the table's pairs all built ({allowed})");
    assert!(refused > 500, "the complement was enumerated ({refused})");
}

// Requirements: MODEL-002
//   The membership edge may target the grouped member representation of
//   two equal-identity paths — the assembled-multipath shape.
// Evidence: membership_targets_the_grouped_member_entry
#[test]
fn membership_targets_the_grouped_member_entry() {
    let path = || device(b"SAME-LUN");
    let member_id = id_of(&path());
    let multipath = NamingFields::MultipathNode {
        lun_designator: b"naa-bytes".to_vec(),
    };
    let multipath_id = id_of(&multipath);
    let topology = Topology::build(
        vec![path(), path(), multipath],
        vec![Edge {
            kind: EdgeKind::PlatformMembership,
            source: multipath_id,
            target: member_id,
        }],
    )
    .expect("builds");
    assert_eq!(topology.entries().len(), 2, "two entries: group + node");
}

// Requirements: MODEL-002
//   Issue #354, and the capture-side half of ADR-0037:146-150's owed
//   sweep: a node's own naming field may not name an address no absorbed
//   entry carries. Enumerated over every referent in `one_of_each`
//   rather than sampled, so a referent-bearing field added later without
//   a sweep entry fails here.
// Evidence: every_naming_referent_must_resolve
#[test]
fn every_naming_referent_must_resolve() {
    let all = one_of_each();
    // Every distinct address some node's *name* embeds.
    let mut referents: Vec<super::naming::NodeId> = all
        .iter()
        .flat_map(super::naming::NamingFields::naming_referents)
        .map(|(_, referent)| referent)
        .collect();
    referents.sort_unstable();
    referents.dedup();
    assert_eq!(
        referents.len(),
        5,
        "device, table, partition, signature and encryption layer are named by others; \
         a change here means the basis moved and the coverage claim needs re-reading"
    );

    for missing in referents {
        // Drop exactly the node that address names, keep everything else.
        let kept: Vec<NamingFields> = all
            .iter()
            .filter(|fields| id_of(fields) != missing)
            .cloned()
            .collect();
        assert_eq!(kept.len(), all.len() - 1, "exactly one node removed");
        let result = Topology::build(kept, vec![]);
        let Err(TopologyError::UnresolvedNamingReferent { referent, .. }) = result else {
            panic!("dropping {missing} must refuse construction, got {result:?}");
        };
        assert_eq!(
            referent, missing,
            "the refusal must name the address that no longer resolves"
        );
    }
}

// Requirements: MODEL-002
//   The roster itself, pinned per kind. `every_naming_referent_must_resolve`
//   enumerates the five distinct *addresses* `one_of_each` names, but two
//   fields name the table, so dropping one of those two from the roster
//   would survive that test alone. This pins all eleven kinds by field, so
//   no single arm can be weakened silently. It is the list the planner's
//   destruction closure walks as well, which is why an omission here is a
//   protection question and not a diagnostics one.
// Evidence: the_naming_referent_roster_is_pinned_per_kind
#[test]
fn the_naming_referent_roster_is_pinned_per_kind() {
    let all = one_of_each();
    let by_kind = |name: &str| -> Vec<&'static str> {
        all.iter()
            .find(|fields| fields.kind_name() == name)
            .unwrap_or_else(|| panic!("{name} must be in one_of_each"))
            .naming_referents()
            .into_iter()
            .map(|(field, _)| field)
            .collect()
    };
    let expected: Vec<(&str, Vec<&str>)> = vec![
        ("physical-device", vec![]),
        ("partition-table", vec!["parent"]),
        ("partition", vec!["parent_table"]),
        ("backing-signature", vec!["host"]),
        ("file-system", vec!["host"]),
        ("encryption-layer", vec!["backing_signature"]),
        ("aggregate", vec![]),
        ("volume", vec!["producer"]),
        ("backing-extent", vec!["host"]),
        ("multipath-node", vec![]),
        ("conflicting-table-entry", vec!["table"]),
    ];
    assert_eq!(
        expected.len(),
        all.len(),
        "one_of_each must still carry one node of every kind"
    );
    for (kind, fields) in expected {
        assert_eq!(by_kind(kind), fields, "{kind}'s naming referents");
    }
}

// Requirements: MODEL-002
//   The refusal is an artifact that locates itself: the node, its kind,
//   the field, and the address that did not resolve.
// Evidence: an_unresolved_naming_referent_names_its_field
#[test]
fn an_unresolved_naming_referent_names_its_field() {
    // The issue's first probe: a partition whose `parent_table` names a
    // derived-but-never-absorbed address — a ghost GPT view.
    let dev = device(b"D0");
    let dev_id = id_of(&dev);
    let real_table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Gpt,
    };
    let ghost_table = NamingFields::PartitionTable {
        parent: dev_id,
        role: TableRole::Mbr,
    };
    let ghost_id = id_of(&ghost_table);
    let partition = NamingFields::Partition {
        parent_table: ghost_id,
        start_offset: 1 << 20,
    };
    let partition_id = id_of(&partition);

    // Lawful containment edges under the *real* table, exactly as the
    // issue measured it: the edge set says one thing, the name another.
    let result = Topology::build(
        vec![dev, real_table.clone(), partition],
        vec![Edge {
            kind: EdgeKind::Containment,
            source: id_of(&real_table),
            target: partition_id,
        }],
    );
    assert_eq!(
        result,
        Err(TopologyError::UnresolvedNamingReferent {
            node: partition_id,
            kind: "partition",
            field: "parent_table",
            referent: ghost_id,
        })
    );
}

// Requirements: MODEL-002
//   The sweep is resolve-only by decision, and this pins the boundary.
//   A referent that resolves to the *wrong kind* still builds: deriving
//   that check from `endpoint_pair_allowed` is issue #354's held half.
//   It was held behind issue #360 while the table could not express a
//   partitioned mdraid array or a GPT inside a mapped volume; ADR-0044
//   landed that row, and what remains before the kind half is the
//   multipath population below. When #354 lands, this test is the one
//   that must be deliberately changed.
// Evidence: a_wrong_kind_referent_still_builds_and_that_is_the_held_half
#[test]
fn a_wrong_kind_referent_still_builds_and_that_is_the_held_half() {
    // The issue's second probe: `parent_table` names the physical device
    // — a real node, absorbed, of a kind no table view could be.
    let dev = device(b"D0");
    let dev_id = id_of(&dev);
    let partition = NamingFields::Partition {
        parent_table: dev_id,
        start_offset: 1 << 20,
    };
    Topology::build(vec![dev, partition], vec![])
        .expect("resolve-only admits a wrong-kind referent; #354 holds the kind half");
}

// Requirements: MODEL-002
//   The three layouts the #354 panel measured as false-refused by the
//   pair-table-derived kind check, re-read under ADR-0044. Two of them
//   were the pair table's own omission and are now admitted rows: a GPT
//   inside a LUKS-mapped volume names a `Volume` in `PartitionTable.parent`
//   and builds *with* its `volume → partition-table` edge; a partitioned
//   mdraid array is `aggregate → volume → partition-table` over the
//   production hop, and builds with its edges too — the table's parent is
//   the array's produced volume, never the aggregate, which stays out of
//   the containment forest. The third — an xfs whose `host` names a
//   `MultipathNode` — no row admits, and it still builds under the
//   resolve-only sweep; that population is what #354's kind half must
//   decide. If any of these ever refuses, the kind half has leaked in.
// Evidence: honest_layouts_the_kind_check_would_have_refused_still_build
#[test]
fn honest_layouts_the_kind_check_would_have_refused_still_build() {
    let containment = |source, target| Edge {
        kind: EdgeKind::Containment,
        source,
        target,
    };
    // a. A GPT inside a LUKS volume: PartitionTable.parent names a Volume,
    //    and the row admits the edge.
    let dev = device(b"D0");
    let dev_id = id_of(&dev);
    let luks = NamingFields::BackingSignature {
        host: dev_id,
        family: SignatureFamily::Luks2,
        primary_offset: 0,
    };
    let luks_id = id_of(&luks);
    let layer = NamingFields::EncryptionLayer {
        backing_signature: luks_id,
    };
    let layer_id = id_of(&layer);
    let volume = NamingFields::Volume {
        producer: layer_id,
        name: b"cryptroot".to_vec(),
        role: None,
    };
    let volume_id = id_of(&volume);
    let inner_table = NamingFields::PartitionTable {
        parent: volume_id,
        role: TableRole::Gpt,
    };
    let inner_table_id = id_of(&inner_table);
    Topology::build(
        vec![dev, luks, layer, volume, inner_table],
        vec![
            containment(dev_id, luks_id),
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
            containment(volume_id, inner_table_id),
        ],
    )
    .expect("a GPT inside a LUKS volume must build, edges and all (ADR-0044)");

    // b. A partitioned mdraid array: the table's parent is the volume the
    //    array produces, and every edge is in the table.
    let array = NamingFields::Aggregate {
        technology: AggregateTechnology::Mdraid,
        designator: Some(b"md0".to_vec()),
    };
    let array_id = id_of(&array);
    let md0 = NamingFields::Volume {
        producer: array_id,
        name: b"md0".to_vec(),
        role: None,
    };
    let md0_id = id_of(&md0);
    let array_table = NamingFields::PartitionTable {
        parent: md0_id,
        role: TableRole::Gpt,
    };
    let array_table_id = id_of(&array_table);
    let md0p1 = NamingFields::Partition {
        parent_table: array_table_id,
        start_offset: 1 << 20,
    };
    let md0p1_id = id_of(&md0p1);
    Topology::build(
        vec![array, md0, array_table, md0p1],
        vec![
            Edge {
                kind: EdgeKind::Production,
                source: array_id,
                target: md0_id,
            },
            containment(md0_id, array_table_id),
            containment(array_table_id, md0p1_id),
        ],
    )
    .expect("a partitioned mdraid array must build, edges and all (ADR-0044)");
    assert!(
        !endpoint_pair_allowed(EdgeKind::Containment, "aggregate", "partition-table"),
        "an aggregate carries no table of its own; its produced volume does"
    );

    // c. An xfs on a dm-multipath node: FileSystem.host names a MultipathNode.
    let multipath = NamingFields::MultipathNode {
        lun_designator: b"naa.60014".to_vec(),
    };
    let multipath_id = id_of(&multipath);
    let xfs = NamingFields::FileSystem {
        host: multipath_id,
        kind: super::naming::FileSystemKind::Xfs,
        superblock_offset: 0,
    };
    Topology::build(vec![multipath, xfs], vec![])
        .expect("an xfs on a dm-multipath node must build");
}

// Requirements: MODEL-002
//   The sweep reads the absorbed set, not the input order: a node may be
//   named by an entry that precedes it. Absorption sorts by address, so
//   an order-sensitive sweep would refuse on an authored permutation —
//   input order is exactly the kind of authored field that must not be
//   able to move a refusal.
// Evidence: the_naming_sweep_does_not_depend_on_input_order
#[test]
fn the_naming_sweep_does_not_depend_on_input_order() {
    let all = one_of_each();
    let forward = Topology::build(all.clone(), vec![]).expect("builds");
    let mut reversed = all;
    reversed.reverse();
    let backward = Topology::build(reversed, vec![]).expect("builds in either order");
    assert_eq!(forward, backward, "construction is order-independent");
}
