# Partition-table content checksums

- Spec version: 8.0.0
- Requirement IDs: SAFE-003, ADR-C3 (`Present {checksum}`), MODEL-005
  (body-stability)
- Normative for: every producer of ADR-C3's `Present` state — today the
  `crates/table-parser` classifier, eventually the privileged helper that
  consumes it (ADR-0014)

ADR-C3 defines `Present` as "read and hashed." This document fixes what
the hash covers, per scheme, so that two implementations can never agree
a table is present while disagreeing what its checksum is. The governing
principle, decided with the SI-35 resolution: **the checksum covers
copy-invariant content** — the facts every copy of the table asserts
identically — and never bytes that differ between copies by the format's
own design. "Both copies agree" and "the checksum" are thereby the same
fact, and a table carried by its backup copy alone (primary invalid,
`primary-invalid` condition) hashes identically to the same table carried
by both.

All checksums are SHA-256. Multi-byte integers are encoded exactly as
written below; no canonicalization beyond field selection is performed —
the inputs are on-disk bytes, not `pce/1` values.

## GPT

SHA-256 over the concatenation, in this order:

1. `DiskGUID` — 16 bytes, verbatim from the header.
2. `FirstUsableLBA` — 8 bytes, little-endian.
3. `LastUsableLBA` — 8 bytes, little-endian.
4. `NumberOfPartitionEntries` — 4 bytes, little-endian.
5. `SizeOfPartitionEntry` — 4 bytes, little-endian.
6. The partition entry array — `NumberOfPartitionEntries ×
   SizeOfPartitionEntry` bytes, verbatim.

Excluded, deliberately: `MyLBA`, `AlternateLBA`, `PartitionEntryLBA`,
both CRC fields, the revision, and the header size. The first three
differ between the primary and backup copies by design; the CRCs are
integrity plumbing over per-copy bytes; revision and header size are
container facts a content hash must not vary with. A checksum computed
over either raw header sector would make the two copies of one table
hash differently, which is the defect this document exists to rule out.

## MBR

SHA-256 over bytes 440..510 of LBA 0: the 4-byte disk signature, the
2-byte reserved field, and the four 16-byte partition entries. Boot code
(bytes 0..440) is excluded — it is executable content, not table
content — and the `0x55AA` signature is a constant. Applies to
standalone MBRs only; a protective or hybrid MBR is part of a GPT
medium's classification, not a `Present {checksum}` producer of its own.

## Apple Partition Map

SHA-256 over the partition-map sectors, verbatim: sectors 1 through
`MapEntries`, where `MapEntries` is the big-endian entry count every map
entry must agree on. APM keeps no redundant copy, so the map is the
content and copy-invariance is trivial.

## Executable vectors

`crates/table-parser/src/tests.rs` carries the executable form of this
document: `the_checksum_is_copy_invariant` proves the GPT recipe blind to
which copy carries the table, and the classification suite computes every
scheme's checksum over the deterministic fixture catalogue. A
cross-language vector file lands with the first second-language consumer
of these checksums; none exists today, and recording that here is what
keeps its absence a fact rather than an oversight.
