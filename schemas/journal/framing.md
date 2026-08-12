# The journal frame profile

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` §4.7 (JRN-001,
  JRN-002, JRN-003, JRN-004)
- Owner: WP-070 (`docs/work-packages/WP-070.md`), increment 2; this
  document landed with increment 3's schema set
- Implementation: `crates/journal`'s frame layer, pinned byte-for-byte
  by the `appends_are_append_only_checksummed_and_monotonic` test
  against an independent transcription

This document records the delivered byte layout. It decides nothing.

## Frame layout

A journal is a byte log of frames, nothing else. Each frame,
little-endian:

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 8 | sequence number, unsigned, one-based, strictly monotonic |
| 8 | 4 | payload length `L`, unsigned, at most 1048576 (1 MiB) |
| 12 | `L` | payload — one `partman.journal.record` encoding (`records.md`) |
| 12 + `L` | 4 | CRC-32 (IEEE 802.3, polynomial `0xEDB88320`) over the preceding `12 + L` bytes |

Sequence numbers start at 1, advance by exactly 1 on append, and are
never reused or reset across rotation or compaction. `0` never appears
in a valid journal.

## Recovery semantics (JRN-001, JRN-004)

- **Torn tail:** a defect reaching the end of the bytes — an incomplete
  frame, or a complete final frame whose checksum fails — is an
  interrupted append; replay truncates it safely and reports the valid
  length.
- **Interior damage:** a checksum failure with bytes behind it, a
  complete over-bound frame, or a non-advancing sequence number refuses
  as corruption. Safe truncation is the tail's rule alone.
- **Gaps:** every sequence gap is classified three ways —
  compaction-covered proceeds (legitimized by a durable `compaction`
  record; the derivation is increment 4's), a torn tail truncates, and
  any other gap refuses as the named mid-chain-gap corruption case.

## Versioning under MODEL-003

The journal's schema version travels in every record's own encoding
(`records.md`): every frame payload carries `schema` and
`schema_version`, so a journal's content is versioned record by record.
The frame layout itself carries no version field; it is fixed by this
document, and bytes that do not parse under it refuse — MODEL-003's
explicit-rejection arm at the framing layer. A future framing change is
a new documented profile under its own reviewed decision, never a
silent variation.

## Durability (JRN-002)

Durability is a typed seam, not a platform claim: the frame layer
offers a `DurabilitySeam` exactly the not-yet-durable byte suffix, and
storage-write clearance for a record exists only behind the
seam-advanced watermark. Real fsync truth is the helper packages'
acceptance obligation.
