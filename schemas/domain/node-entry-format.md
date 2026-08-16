# The node-entry format inside PartMan hashed bodies

- Spec version: 11.1.0
- Requirement IDs: MODEL-002, MODEL-005, MODEL-006, SAFE-005
- Decided by: `docs/adr/0019-si27-node-naming.md` (naming and collision
  groups), `docs/adr/0018-si11-protection-closure.md` (the evidence facts a
  verdict reads), `docs/adr/0014-si35-table-state-axis.md` (the table-state
  stamp); delivered by WP-010 increment 3 slices 3a and 3f
- Underlying byte profile: `pce/1` (unchanged)
- Shared vectors: `schemas/domain/body-vectors.json`, `node_entries` section

This document records a delivered format. It decides nothing: a field
exists here because `crates/domain` encodes it, never because this document
says so, and the constructors and their tests are the authority wherever a
sentence here could be read two ways.

## 1. Where entries live, and what an entry is

A node entry is one element of a snapshot body's `nodes` set
(`schemas/domain/topology-snapshot-body.md`). It is a `pce/1` Map holding,
in this order of concern rather than byte order (map keys sort per
`pce/1`):

1. the **kind tag** and the kind's **naming fields** (§2, §3);
2. for a collision group, the **group fields** (§4); and
3. the node's **body-carried facts**, where established (§5).

Entries never carry a node identifier. Identifiers are derived positional
addresses recomputed from the naming fields (§6); a recorded identifier
would be a client-authored claim, which is the shape the whole model
refuses.

## 2. The kind tag

Every entry carries `kind`, a Text value from the closed list:

`physical-device`, `partition-table`, `partition`, `backing-signature`,
`file-system`, `encryption-layer`, `aggregate`, `volume`,
`backing-extent`, `multipath-node`, `conflicting-table-entry`.

An unknown kind refuses at the schema-validation pass. The generic `pce/1`
decoder accepts it — schema validity is the typed boundary's job, never
the codec's.

## 3. Per-kind naming fields

Identifier bytes are contract-source-verbatim: never re-encoded, never
trimmed, never case-folded. Absent optional fields are omitted, not
null-carried. Enum-tagged fields (`role`, `family`, `fs_kind`,
`technology`) encode a recognized value as its Text tag and an
unrecognized one as the reporting interface's raw discriminant **Bytes**
verbatim — the `pce/1` type discriminates recognition, so no tag string
can collide with raw bytes.

| `kind` | Fields |
| --- | --- |
| `physical-device` | `serial` Bytes?, `wwn` Bytes?, `total_bytes` Unsigned |
| `partition-table` | `parent` Bytes(32), `role` (§3a) |
| `partition` | `parent_table` Bytes(32), `start_offset` Unsigned |
| `backing-signature` | `host` Bytes(32), `family` (§3a), `primary_offset` Unsigned |
| `file-system` | `host` Bytes(32), `fs_kind` (§3a), `superblock_offset` Unsigned |
| `encryption-layer` | `backing_signature` Bytes(32) |
| `aggregate` | `technology` (§3a), `designator` Bytes? |
| `volume` | `producer` Bytes(32), `name` Bytes, `volume_role` Bytes? |
| `backing-extent` | `host` Bytes(32), then `path` Bytes **or** `range_start` Unsigned + `range_length` Unsigned |
| `multipath-node` | `lun_designator` Bytes |
| `conflicting-table-entry` | see `crates/domain/src/model/naming.rs`; the vectors carry the common kinds |

`Bytes(32)` is a referenced node's derived address (§6). `Bytes?` marks an
optional field, omitted when absent.

### 3a. Enum tags

- `role`: `gpt`, `mbr`, `apm`, `hybrid-mbr`
- `family`: `zfs`, `mdraid-0.90`, `mdraid-1.x`, `luks1`, `luks2`, `lvm2`,
  `storage-spaces`, `ldm`, `bitlocker`, `apfs-container`
- `fs_kind`: `ext2`, `ext3`, `ext4`, `btrfs`, `xfs`, `f2fs`, `fat12`,
  `fat16`, `fat32`, `exfat`, `ntfs`, `refs`, `hfsplus`, `apfs`, `udf`,
  `swap`
- `technology`: `lvm2`, `mdraid`, `storage-spaces`, `zfs`, `apfs`, `ldm`

## 4. Collision groups

Naming fields that derive equal addresses collapse before encoding into
one counted, flagged group entry (ADR-0019): the shared naming fields plus

- `collision_count` — Unsigned, the number of collapsed nodes (≥ 2);
- `duplicate_designator` — Bool, the committed duplicate-designator flag.

A group always encodes. Preserved plurality with non-silence is the
decided representation of the ambiguity SAFE-005 and ADR-0011 declare; a
forged count refuses at the typed boundary, held by the committed
regressions.

## 5. Body-carried facts (increment 3f)

Verdict inputs are body content (ADR-0016's logic extended to facts): what
the protection closure reads, the authorization hash commits to. Where the
helper's evidence contract established a fact for a node, the entry
carries it:

- `extent_host` Bytes(32) + `extent_start` Unsigned + `extent_length`
  Unsigned — the node's byte range, framed on the containment root the
  node's own name leads to (ADR-0037's rule; since ADR-0046 a body whose
  `extent_host` is any other address refuses at the typed boundary with
  both frames named, and a backing extent — outside every containment
  forest — is the one kind the rule does not reach);
- `transport` Text — one of `nvme-pcie`, `sata`, `sas-direct`, `usb`,
  `sd-mmc`, `paravirtual-local`, `recognized-remote`, `unrecognized`
  (ADR-0018's device-scope transport arm reads this);
- `member_count` Unsigned — the technology's self-reported member count
  (the Fusion arm reads this);
- `table_state` Map — ADR-C3's three-valued state as
  `{ state: "present", checksum: Bytes(32) }`, `{ state: "absent" }`, or
  `{ state: "indeterminate", cause: "ambiguous" | "unreadable" }`,
  stamped when the helper produces the snapshot (ADR-0014's stamp point).

An unestablished fact is omitted — absence of a key is "not established",
never a default. A fact on a kind that cannot carry it refuses at the
typed boundary, as does an extent that is not a range, one framed below
its containment root, one lying outside its containment parent, and a
containment edge that nests the node in a parent its name does not embed
(ADR-0041, ADR-0046).

## 6. Derived addresses

A node's address is `NodeId`: the SHA-256 over the canonical bytes of a
domain-separated preimage — the entry's `kind` and naming fields (§3
exactly, facts and group fields excluded) plus `schema:
"partman.node-id"` and `schema_version: 1`. Addresses appear in bodies
only as 32-byte references (`parent`, `host`, edge endpoints, fact hosts,
plan targets); every consumer recomputes and compares rather than
trusting recorded bytes (the decode-recompute rule).

## 7. Conformance

The shared vectors pin, for each entry kind exercised: the value tree,
its canonical bytes, and its digest. `crates/domain/tests/body_vectors.rs`
proves the constructors reproduce the recorded bytes and that every
node-entry vector appears verbatim in its snapshot's `nodes` set;
`packages/canonical/src/body-vectors.test.ts` proves the TypeScript codec
reproduces the same bytes and digests from the same trees.
