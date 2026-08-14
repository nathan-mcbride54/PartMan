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
