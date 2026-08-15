# Issue #347 round 2 — the adversarial probe source, preserved 2026-08-14

Untracked session artifact promoted to the record, `docs/reviews` convention
(WP-000 owns `docs/reviews/**`).

**Why this file exists.** The round-2 panel
(`ISSUE-347_RELEASE_ROUND_2_ADVERSARIAL_2026-08-14.md`) rejected the candidate
but closed with an instruction that outlives it:

> Before any candidate in this family is measured again, **commit the
> overlapping-geometry shape to the fixture population**: `f11` and `f12` are
> already written as assertions and pass at HEAD.

Those assertions lived only in a workflow worktree
(`.claude/worktrees/wf_f2620bc0-76d-1`, branch `adv/347-lens1`), untracked and
wired in through a one-line `mod adversary_probe;` in `crates/domain/src/model/mod.rs`.
That worktree has been removed. The source is kept here verbatim so round 3 can
lift `f11`/`f12` into `crates/domain` as committed fixtures without re-deriving
the geometry.

**This is not delivered test code.** It is a probe: `//! Not for merge`, printing
probes mixed with hard assertions, no traceability annotations. Landing any of it
is WP-010 work with its own `Requirements:`/`Evidence:` blocks.

**The two that matter**

- `f11_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table`
- `f12_a_range_that_touches_no_gpt_structure_still_releases_the_disk`

Both **pass at HEAD** and **fail under the rejected candidate**, on a
`bios_boot_gpt` layout: root-on-ZFS plus a BIOS boot partition at LBA 34
(`[17408, 1 MiB)`) under a table declaring the conventional `[0, 1 MiB)` — the
overlap the entire committed fixture population lacks, since every committed
instance has `table.start + table.length == p1.start` exactly.

`probe_f1`–`probe_f13` are printing probes; they assert little and exist to
produce the capability-gate lines quoted in the round.

---

## Source, verbatim (`crates/domain/src/model/adversary_probe.rs` @ `6e1706b`)

```rust
//! ADVERSARIAL PROBE — lens 1 (over-reach / false refusal), issue #347.
//! Not for merge. Printing probes plus hard assertions.

#![allow(clippy::too_many_lines, clippy::similar_names, unused_imports)]

use std::collections::{BTreeMap, BTreeSet};

use super::capability::{Operation, ProtectionGate, canonical_ranges, protection_gate};
use super::naming::{
    AggregateTechnology, NamingFields, NodeId, SignatureFamily, TableRole, derive_id,
};
use super::protection::{
    Facts, HostRange, IndeterminateGround, RefusalGround, StepRanges, TransportClass, Verdict,
    affected_set, node_verdict, step_constructs,
};
use super::topology::{Edge, EdgeKind, Topology};

const MIB: u64 = 1 << 20;

fn dev(serial: &[u8], total: u64) -> NamingFields {
    NamingFields::PhysicalDevice {
        serial: Some(serial.to_vec()),
        wwn: None,
        total_bytes: total,
    }
}

fn mutating() -> [Operation; 10] {
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

fn gate_line(topology: &Topology, facts: &Facts, target: NodeId, label: &str) -> String {
    let mut out = format!("  gate[{label}]: ");
    for op in mutating() {
        let g = protection_gate(topology, facts, target, op);
        out.push_str(&format!(
            "{:?}={} ",
            op,
            match g {
                ProtectionGate::Clear => "Clear".to_owned(),
                ProtectionGate::Unsupported { ground } => format!("Unsupported{ground:?}"),
                ProtectionGate::Blocked { cause } => format!("Blocked{cause:?}"),
            }
        ));
    }
    out
}

// ---------------------------------------------------------------------
// F1: an honest BIOS-booting GPT disk. The conventional layout puts a
// bios_grub partition at LBA 34..2047 — inside the first MiB, which is
// exactly the extent the committed fixture gives the GPT node. Root on
// ZFS beside an ESP.
// ---------------------------------------------------------------------
struct BiosBoot {
    topology: Topology,
    facts: Facts,
    sda: NodeId,
    table: NodeId,
    boot: NodeId,
    esp: NodeId,
    member: NodeId,
    pool: NodeId,
}

fn bios_boot_gpt() -> BiosBoot {
    let sda = dev(b"SDA", 1 << 30);
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    // sectors 34..2047 -> [17408, 1 MiB)
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
    let topology = Topology::build(
        vec![sda, table, boot, esp, member, signature, pool],
        vec![
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
    extents.insert(table_id, host(0, MIB));
    extents.insert(boot_id, host(17408, MIB - 17408));
    extents.insert(esp_id, host(MIB, 256 * MIB));
    extents.insert(member_id, host(512 * MIB, 256 * MIB));
    extents.insert(signature_id, host(512 * MIB, MIB));
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    BiosBoot {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        sda: sda_id,
        table: table_id,
        boot: boot_id,
        esp: esp_id,
        member: member_id,
        pool: pool_id,
    }
}

#[test]
fn probe_f1_bios_boot_overlap() {
    let f = bios_boot_gpt();
    println!("=== F1: honest BIOS-boot GPT, bios_grub at [17408, 1 MiB), table [0, 1 MiB)");
    let ranges = canonical_ranges(Operation::Wipe, f.boot, &f.facts);
    println!("  canonical destroyed(Wipe, bios_grub) = {:?}", ranges.destroyed);
    let set = affected_set(&f.topology, &f.facts, f.boot, &ranges);
    println!(
        "  |affected| = {}  esp={} member={} pool={}",
        set.len(),
        set.contains(&f.esp),
        set.contains(&f.member),
        set.contains(&f.pool)
    );
    println!(
        "  step_constructs(Wipe, bios_grub) = {:?}",
        step_constructs(&f.topology, &f.facts, f.boot, &ranges).map(|s| s.len())
    );
    println!("{}", gate_line(&f.topology, &f.facts, f.boot, "bios_grub"));
    println!("{}", gate_line(&f.topology, &f.facts, f.esp, "esp"));
    println!("{}", gate_line(&f.topology, &f.facts, f.table, "table"));
    println!("{}", gate_line(&f.topology, &f.facts, f.sda, "sda"));
}

// ---------------------------------------------------------------------
// F2: hybrid GPT/MBR with a ConflictingTableEntry, root on ZFS.
// Two placements of the CTE extent are probed: the entry's own bytes in
// the MBR sector ([0,512), inside the table), and the region the entry
// describes ([1 MiB, ...), outside it).
// ---------------------------------------------------------------------
struct Hybrid {
    topology: Topology,
    facts: Facts,
    sda: NodeId,
    table: NodeId,
    cte: NodeId,
    esp: NodeId,
    member: NodeId,
    pool: NodeId,
}

fn hybrid(cte_extent: Option<(u64, u64)>) -> Hybrid {
    let sda = dev(b"SDAH", 1 << 30);
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let cte = NamingFields::ConflictingTableEntry {
        table: table_id,
        view_role: TableRole::HybridMbr,
        entry_start: MIB,
    };
    let cte_id = derive_id(&cte).expect("derivable");
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
        designator: Some(b"pool-guid-h".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![sda, table, cte, esp, member, signature, pool],
        vec![
            Edge {
                kind: EdgeKind::Containment,
                source: sda_id,
                target: table_id,
            },
            Edge {
                kind: EdgeKind::Containment,
                source: table_id,
                target: cte_id,
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
    let host = |start, length| HostRange {
        host: sda_id,
        start,
        length,
    };
    extents.insert(sda_id, host(0, 1 << 30));
    extents.insert(table_id, host(0, MIB));
    extents.insert(esp_id, host(MIB, 256 * MIB));
    extents.insert(member_id, host(512 * MIB, 256 * MIB));
    extents.insert(signature_id, host(512 * MIB, MIB));
    if let Some((start, length)) = cte_extent {
        extents.insert(cte_id, host(start, length));
    }
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    Hybrid {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        sda: sda_id,
        table: table_id,
        cte: cte_id,
        esp: esp_id,
        member: member_id,
        pool: pool_id,
    }
}

#[test]
fn probe_f2_hybrid_cte() {
    for (label, cte_extent) in [
        ("cte extentless", None),
        ("cte in the MBR sector [0,512)", Some((0, 512))),
        ("cte over the region it describes [1 MiB, 256 MiB)", Some((MIB, 256 * MIB))),
    ] {
        let f = hybrid(cte_extent);
        println!("=== F2 {label}");
        println!("{}", gate_line(&f.topology, &f.facts, f.table, "table"));
        println!("{}", gate_line(&f.topology, &f.facts, f.esp, "esp"));
        println!("{}", gate_line(&f.topology, &f.facts, f.cte, "cte"));
        println!("{}", gate_line(&f.topology, &f.facts, f.sda, "sda"));
        let ranges = canonical_ranges(Operation::Wipe, f.table, &f.facts);
        let set = affected_set(&f.topology, &f.facts, f.table, &ranges);
        println!(
            "  wipe-table: |affected|={} cte={} esp={} member={} pool={}",
            set.len(),
            set.contains(&f.cte),
            set.contains(&f.esp),
            set.contains(&f.member),
            set.contains(&f.pool)
        );
    }
}

// ---------------------------------------------------------------------
// F3: extent-less table (ADR-0036's shape) — control, must be identical
// on both sides.
// ---------------------------------------------------------------------
#[test]
fn probe_f3_extentless_table() {
    let mut f = bios_boot_gpt();
    f.facts.extents.remove(&f.table);
    println!("=== F3: extent-less table");
    println!("{}", gate_line(&f.topology, &f.facts, f.table, "table"));
    println!("{}", gate_line(&f.topology, &f.facts, f.boot, "bios_grub"));
    println!("{}", gate_line(&f.topology, &f.facts, f.esp, "esp"));
}

// ---------------------------------------------------------------------
// F4: two tables on one disk — a hybrid MBR beside the GPT it shadows.
// Both are `partition-table` nodes parented to the device; the MBR's
// bytes [0,512) lie inside the GPT node's declared [0,1 MiB).
// Wiping the hybrid MBR is a real, recommended remediation.
// ---------------------------------------------------------------------
struct TwoTables {
    topology: Topology,
    facts: Facts,
    sda: NodeId,
    gpt: NodeId,
    mbr: NodeId,
    esp: NodeId,
    member: NodeId,
    pool: NodeId,
}

fn two_tables() -> TwoTables {
    let sda = dev(b"SDA2T", 1 << 30);
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
    let signature = NamingFields::BackingSignature {
        host: member_id,
        family: SignatureFamily::Zfs,
        primary_offset: 0,
    };
    let signature_id = derive_id(&signature).expect("derivable");
    let pool = NamingFields::Aggregate {
        technology: AggregateTechnology::Zfs,
        designator: Some(b"pool-guid-2t".to_vec()),
    };
    let pool_id = derive_id(&pool).expect("derivable");
    let topology = Topology::build(
        vec![sda, gpt, mbr, esp, member, signature, pool],
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
    extents.insert(signature_id, host(512 * MIB, MIB));
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    TwoTables {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        sda: sda_id,
        gpt: gpt_id,
        mbr: mbr_id,
        esp: esp_id,
        member: member_id,
        pool: pool_id,
    }
}

#[test]
fn probe_f4_two_tables() {
    let f = two_tables();
    println!("=== F4: hybrid MBR [0,512) beside GPT [0,1 MiB), partitions on the GPT");
    println!("{}", gate_line(&f.topology, &f.facts, f.mbr, "hybrid-mbr"));
    println!("{}", gate_line(&f.topology, &f.facts, f.gpt, "gpt"));
    println!("{}", gate_line(&f.topology, &f.facts, f.esp, "esp"));
    let ranges = canonical_ranges(Operation::Wipe, f.mbr, &f.facts);
    println!("  destroyed(Wipe, hybrid-mbr) = {:?}", ranges.destroyed);
    let set = affected_set(&f.topology, &f.facts, f.mbr, &ranges);
    println!(
        "  |affected|={} gpt={} esp={} member={} pool={}",
        set.len(),
        set.contains(&f.gpt),
        set.contains(&f.esp),
        set.contains(&f.member),
        set.contains(&f.pool)
    );
    let _ = f.sda;
}

// ---------------------------------------------------------------------
// F5: the committed root_on_zfs geometry, but the *table* extent is the
// truthful GPT one — primary structures plus the backup header at the
// end of the disk, which a single HostRange can only express as the
// whole device span. Nothing dishonest is declared; the range is a
// superset of what the GPT occupies.
// ---------------------------------------------------------------------
#[test]
fn probe_f5_conservative_table_extent() {
    let mut f = bios_boot_gpt();
    f.facts.extents.insert(
        f.table,
        HostRange {
            host: f.sda,
            start: 0,
            length: 1 << 30,
        },
    );
    println!("=== F5: table extent declared over the whole disk (primary + backup GPT)");
    println!("{}", gate_line(&f.topology, &f.facts, f.esp, "esp"));
    println!("{}", gate_line(&f.topology, &f.facts, f.boot, "bios_grub"));
    println!("{}", gate_line(&f.topology, &f.facts, f.member, "member"));
    let ranges = canonical_ranges(Operation::Wipe, f.esp, &f.facts);
    let set = affected_set(&f.topology, &f.facts, f.esp, &ranges);
    println!(
        "  wipe-esp: |affected|={} member={} pool={}",
        set.len(),
        set.contains(&f.member),
        set.contains(&f.pool)
    );
}

// ---------------------------------------------------------------------
// F6: plan-layer spelling. `canonical_ranges` always destroys the
// target's own extent, so the gate can never see a destroyed range that
// misses. The plan constructor decodes `destroyed` verbatim. A one-byte
// destroyed range anywhere in the table's extent releases the disk.
// ---------------------------------------------------------------------
#[test]
fn probe_f6_one_byte() {
    let f = bios_boot_gpt();
    let one_byte = StepRanges {
        written_table_extents: vec![],
        consumed: vec![],
        destroyed: vec![HostRange {
            host: f.sda,
            start: 0,
            length: 1,
        }],
    };
    let set = affected_set(&f.topology, &f.facts, f.table, &one_byte);
    println!("=== F6: destroyed = exactly one byte at offset 0");
    println!(
        "  |affected|={} esp={} member={} pool={}  constructs={:?}",
        set.len(),
        set.contains(&f.esp),
        set.contains(&f.member),
        set.contains(&f.pool),
        step_constructs(&f.topology, &f.facts, f.table, &one_byte).is_ok()
    );
}

// ---------------------------------------------------------------------
// F7 / F8: no ZFS anywhere. An ordinary disk whose only non-Permitted
// node is (a) a hybrid ConflictingTableEntry, or (b) an orphan
// signature under a *sibling* partition. The step under test destroys
// the bios_grub partition, which nests inside the table's declared
// extent and touches nothing else.
// ---------------------------------------------------------------------
struct Plain {
    topology: Topology,
    facts: Facts,
    table: NodeId,
    boot: NodeId,
    data: NodeId,
    other: NodeId,
}

fn plain_disk(with_cte: bool, boot_start: u64) -> Plain {
    let sda = dev(if with_cte { b"PLAINC" } else { b"PLAINO" }, 1 << 30);
    let sda_id = derive_id(&sda).expect("derivable");
    let table = NamingFields::PartitionTable {
        parent: sda_id,
        role: TableRole::Gpt,
    };
    let table_id = derive_id(&table).expect("derivable");
    let boot = NamingFields::Partition {
        parent_table: table_id,
        start_offset: boot_start,
    };
    let boot_id = derive_id(&boot).expect("derivable");
    let data = NamingFields::Partition {
        parent_table: table_id,
        start_offset: 256 * MIB,
    };
    let data_id = derive_id(&data).expect("derivable");
    let mut nodes = vec![sda, table, boot, data];
    let mut edges = vec![
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
            target: data_id,
        },
    ];
    let other_id;
    if with_cte {
        let cte = NamingFields::ConflictingTableEntry {
            table: table_id,
            view_role: TableRole::HybridMbr,
            entry_start: 256 * MIB,
        };
        other_id = derive_id(&cte).expect("derivable");
        nodes.push(cte);
        edges.push(Edge {
            kind: EdgeKind::Containment,
            source: table_id,
            target: other_id,
        });
    } else {
        // An orphan signature under the *data* partition: no observed
        // consumer, so Indeterminate{OrphanSignature}.
        let sig = NamingFields::BackingSignature {
            host: data_id,
            family: SignatureFamily::Lvm2,
            primary_offset: 0,
        };
        other_id = derive_id(&sig).expect("derivable");
        nodes.push(sig);
        edges.push(Edge {
            kind: EdgeKind::Containment,
            source: data_id,
            target: other_id,
        });
    }
    let topology = Topology::build(nodes, edges).expect("builds");
    let host = |start, length| HostRange {
        host: sda_id,
        start,
        length,
    };
    let mut extents = BTreeMap::new();
    extents.insert(sda_id, host(0, 1 << 30));
    extents.insert(table_id, host(0, MIB));
    extents.insert(boot_id, host(boot_start, 64 * 1024));
    extents.insert(data_id, host(256 * MIB, 256 * MIB));
    if !with_cte {
        extents.insert(other_id, host(256 * MIB, MIB));
    }
    let mut transports = BTreeMap::new();
    transports.insert(sda_id, TransportClass::Sata);
    Plain {
        topology,
        facts: Facts {
            extents,
            transports,
            member_counts: BTreeMap::new(),
            table_states: BTreeMap::new(),
        },
        table: table_id,
        boot: boot_id,
        data: data_id,
        other: other_id,
    }
}

#[test]
fn probe_f7_plain_hybrid_cte_no_zfs() {
    for (label, boot_start) in [
        ("bios_grub nested in the table extent (start 17408)", 17408u64),
        ("bios_grub clear of the table extent (start 1 MiB)", MIB),
    ] {
        let f = plain_disk(true, boot_start);
        println!("=== F7 no-ZFS hybrid, CTE (extentless), {label}");
        println!("{}", gate_line(&f.topology, &f.facts, f.boot, "bios_grub"));
        println!("{}", gate_line(&f.topology, &f.facts, f.data, "data"));
        println!("{}", gate_line(&f.topology, &f.facts, f.table, "table"));
        let ranges = canonical_ranges(Operation::Wipe, f.boot, &f.facts);
        let set = affected_set(&f.topology, &f.facts, f.boot, &ranges);
        println!(
            "  wipe-bios_grub: |affected|={} cte={} data={}",
            set.len(),
            set.contains(&f.other),
            set.contains(&f.data)
        );
    }
}

#[test]
fn probe_f8_plain_orphan_signature_no_zfs() {
    for (label, boot_start) in [
        ("bios_grub nested in the table extent (start 17408)", 17408u64),
        ("bios_grub clear of the table extent (start 1 MiB)", MIB),
    ] {
        let f = plain_disk(false, boot_start);
        println!("=== F8 no-ZFS, orphan LVM2 signature under the *data* partition, {label}");
        println!("  orphan verdict = {:?}", node_verdict(&f.topology, &f.facts, f.other));
        println!("{}", gate_line(&f.topology, &f.facts, f.boot, "bios_grub"));
        println!("{}", gate_line(&f.topology, &f.facts, f.data, "data"));
        let ranges = canonical_ranges(Operation::Wipe, f.boot, &f.facts);
        let set = affected_set(&f.topology, &f.facts, f.boot, &ranges);
        println!(
            "  wipe-bios_grub: |affected|={} orphan={} data={}",
            set.len(),
            set.contains(&f.other),
            set.contains(&f.data)
        );
    }
}

// ---------------------------------------------------------------------
// F9: is a nested table (an EBR inside an extended partition)
// representable at all? Structural claim, read off the pair table.
// ---------------------------------------------------------------------
#[test]
fn probe_f9_nested_table_representable() {
    use super::topology::endpoint_pair_allowed;
    for (source, target) in [
        ("partition", "partition-table"),
        ("volume", "partition-table"),
        ("partition-table", "partition-table"),
        ("partition-table", "partition"),
        ("partition-table", "conflicting-table-entry"),
        ("physical-device", "partition-table"),
    ] {
        println!(
            "  containment({source} -> {target}) allowed = {}",
            endpoint_pair_allowed(EdgeKind::Containment, source, target)
        );
    }
}

// ---------------------------------------------------------------------
// F10: the "release strength" question the round-2 predicate dropped.
// Round 1's rejected predicate asked whether the destroyed bytes covered
// the table; round 2 asks only whether they touch it. Both spellings of
// "one byte inside the table's extent" are measured here against the
// committed root_on_zfs geometry to show the trigger surface.
// ---------------------------------------------------------------------
#[test]
fn probe_f10_trigger_surface() {
    let f = bios_boot_gpt();
    for (label, start, length) in [
        ("whole table extent", 0u64, MIB),
        ("one byte at 0", 0, 1),
        ("one byte at the last table byte", MIB - 1, 1),
        ("one byte just past the table", MIB, 1),
        ("the bios_grub partition only", 17408, MIB - 17408),
    ] {
        let ranges = StepRanges {
            written_table_extents: vec![],
            consumed: vec![],
            destroyed: vec![HostRange {
                host: f.sda,
                start,
                length,
            }],
        };
        let set = affected_set(&f.topology, &f.facts, f.table, &ranges);
        println!(
            "  destroyed={label:<34} |affected|={} esp={} pool={} constructs={}",
            set.len(),
            set.contains(&f.esp),
            set.contains(&f.pool),
            step_constructs(&f.topology, &f.facts, f.table, &ranges).is_ok()
        );
    }
}

// ---------------------------------------------------------------------
// F11: the committed guard `a_sibling_esp_is_never_captured`
// (protection_tests.rs:207-236) re-run verbatim in shape, with the
// deleted partition being the bios_grub one instead of the vdev member.
// Same step shape: the table's extent is *written*, the partition's
// extent is *destroyed*. This test asserts the property the committed
// guard asserts. It passes at HEAD and fails under the candidate.
// ---------------------------------------------------------------------
#[test]
fn f11_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table() {
    let f = bios_boot_gpt();
    let delete_bios_grub = StepRanges {
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
    let affected = affected_set(&f.topology, &f.facts, f.table, &delete_bios_grub);
    println!(
        "  F11 |affected|={} esp={} member={} pool={}",
        affected.len(),
        affected.contains(&f.esp),
        affected.contains(&f.member),
        affected.contains(&f.pool)
    );
    assert!(
        !affected.contains(&f.esp),
        "the ESP is disjoint from the destroyed range and must stay unreached"
    );
    assert!(
        !affected.contains(&f.pool),
        "the pool is disjoint from the destroyed range and must stay unreached"
    );
}

// ---------------------------------------------------------------------
// F12: the destroyed range provably misses every GPT structure. LBA0
// (protective MBR) is [0,512), the primary header LBA1 is [512,1024),
// the entry array LBA2..33 is [1024,17408). The step destroys
// [17408, 1 MiB) — not one byte of the GPT. The closure still treats the
// table as destroyed.
// ---------------------------------------------------------------------
#[test]
fn f12_a_range_that_touches_no_gpt_structure_still_releases_the_disk() {
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
    let affected = affected_set(&f.topology, &f.facts, f.table, &ranges);
    println!(
        "  F12 destroyed=[17408,1 MiB) (no GPT structure in it) |affected|={} pool={} constructs={}",
        affected.len(),
        affected.contains(&f.pool),
        step_constructs(&f.topology, &f.facts, f.table, &ranges).is_ok()
    );
    assert!(
        !affected.contains(&f.pool),
        "a range containing no GPT structure must not release the table's partitions"
    );
}

// ---------------------------------------------------------------------
// F13: one authored number on the COMMITTED fixture geometry. The table
// extent is the only thing varied; every node, edge and step is the
// committed root_on_zfs shape minus the bios_grub partition. The step is
// "wipe the ESP" — a partition that has nothing to do with the pool.
// Round 1 died because inflating this number REMOVED a refusal. Round 2
// calls the same monotonicity a virtue; measured here in the direction
// round 2 never tested.
// ---------------------------------------------------------------------
#[test]
fn probe_f13_one_authored_number_on_the_committed_geometry() {
    println!("=== F13: vary only extents[table]; step = wipe the ESP");
    for (label, length) in [
        ("[0, 1 MiB)      (the committed value)", MIB),
        ("[0, 2 MiB)      (still clear of every partition)", 2 * MIB),
        ("[0, 1 MiB + 1)  (round 1's one-byte inflation)", MIB + 1),
        ("[0, 2 MiB) + ESP nested? no", 2 * MIB),
        ("[0, 258 MiB)    (conservative: covers the ESP)", 258 * MIB),
    ] {
        let mut f = bios_boot_gpt();
        // Remove the bios_grub partition's influence: target the ESP.
        f.facts.extents.insert(
            f.table,
            HostRange {
                host: f.sda,
                start: 0,
                length,
            },
        );
        let ranges = canonical_ranges(Operation::Wipe, f.esp, &f.facts);
        let set = affected_set(&f.topology, &f.facts, f.esp, &ranges);
        println!(
            "  extents[table]={label:<48} |affected|={} member={} pool={} gate={:?}",
            set.len(),
            set.contains(&f.member),
            set.contains(&f.pool),
            protection_gate(&f.topology, &f.facts, f.esp, Operation::Wipe)
        );
    }
}
```
