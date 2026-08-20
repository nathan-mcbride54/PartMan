# The journal record schema, `partman.journal.record` version 2

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` §4.7 (JRN-005,
  JRN-006), Section 8, and MODEL-003/MODEL-005
- Owner: WP-070 (`docs/work-packages/WP-070.md`), increment 3; version 2
  (the recorded instant) is slice 3b, on the WP-L110 increment-4 shape
  round (`docs/reviews/WP-L110_INCREMENT_4_ROUND_2026-08-20.md` §9.3 —
  transition-only, as decided)
- Decided semantics carried: ADR-0021 and ADR-0028 (the authorization
  act), ADR-0024 (the three-variant protection record), ADR-0026 (the
  dry-run refusal class), ADR-0027 (the disposal linkage), ADR-0029
  (the compaction record and the per-apply budget), ADR-0030 (hash-only
  protection-artifact references)
- Implementation: `crates/journal`'s `records` module; the golden
  vectors below are held in agreement with that encoder by the
  `every_record_class_round_trips_and_matches_the_documented_vectors`
  test

This document records a delivered vocabulary. It decides nothing: a
field exists here because a recorded decision requires it and the crate
encodes it, never because this document says so.

## Encoding

Every record is one `pce/1` canonical Map (MODEL-005, WP-010's codec —
`schemas/canonical-encoding.md`), carried as one journal frame payload
(`schemas/journal/framing.md`). Domain separation is inside the value,
per that specification's §5:

| Field | Type | Value |
| --- | --- | --- |
| `schema` | Text | always `partman.journal.record` |
| `schema_version` | Unsigned | always `2` |
| `kind` | Text | one of the closed kind vocabulary below |

Any other version is refused at decode — MODEL-003's explicit-rejection
arm. **Version 1 is refused with nothing to migrate, honestly:** no
journal on-disk home existed while v1 was current, so no v1 byte ever
reached disk — the reason this version landed *before* the journal's
on-disk home (WP-L110 increment 4a) rather than after it. v2 differs
from v1 in exactly one way: the `transition` kind carries a required
`instant`. The decoder is strict in every direction: unknown fields,
unknown tags, mistyped positions, and wrong-length hashes refuse,
nothing is repaired, and no refusal echoes refused content.

## JRN-005, structurally

No record class has a free-text position. Every field is the pinned
schema constant, a member of a closed tag vocabulary, an unsigned
integer, or a 32-byte hash. Identifier-bearing values — device serials,
paths, labels, usernames, key material, file names (SEC-006's classes)
— have no field to occupy, and the redaction sweep in the crate's tests
plants each class in every position and proves the refusal. Bounded,
redacted embedded tool output (JRN-005's second clause) is a helper
surface; no field here can carry it.

## Record kinds

### `authorization-act` (ADR-0021, ADR-0028)

The floor act as a journal fact. One act authorizes one apply of one
plan; the journal is the act's only home.

| Field | Type | Meaning |
| --- | --- | --- |
| `plan` | Bytes(32) | the exact plan body hash (MODEL-005) |
| `tier` | Text | `floor-act` or `interactive-ceremony` — the helper-computed tier (ADR-0021), never a client claim |

### `transition` (Section 8, JRN-002)

One record per taken transition.

| Field | Type | Meaning |
| --- | --- | --- |
| `plan` | Bytes(32) | the plan whose apply took the transition |
| `transition` | Text | the row's tag, from the 23-member vocabulary below |
| `instant` | Unsigned | the instant the transition was recorded, in seconds since the Unix epoch — required on every transition record (v2). Authored by the caller's own fallible clock (the helper's clock seam refuses rather than defaults; the journal crate reads no clock), stored and returned unjudged. The journal's high-water instant — the backward-clock bound's input (WP-L110 increment 4a) — is the maximum such value; `validator-passes` is journaled at validation, so a transition-only instant covers the validation-to-presentation window the bound was written for |
| `effect` | Text, terminal rows only | `no-writes`, `partial`, or `complete` — required on every terminal row, forbidden elsewhere, and checked against the published per-row constraint at record-write time |
| `recovery_plan` | Bytes(32), optional | ADR-0027's disposal linkage: the recovery plan's body hash. Legal only on the `failure-accepted` row |

Transition tags, in the published table's row order:
`validator-passes`, `edit-or-invalidation`, `apply-submitted`,
`authorization-granted`, `declined-or-expired`, `revalidation-passes`,
`identity-mismatch`, `backups-verified`, `backup-failure`,
`final-step-complete`, `user-pauses`, `reboot-step-reached`,
`step-failure-or-interruption`, `cancel-honored`, `user-resumes`,
`cancel-while-paused`, `topology-changed-while-paused`,
`reboot-resume`, `resume-impossible`, `postconditions-pass`,
`postcondition-failure`, `roll-forward-selected`, `failure-accepted`.

### `checkpoint` (JRN-002)

| Field | Type | Meaning |
| --- | --- | --- |
| `plan` | Bytes(32) | the plan being executed |
| `step_index` | Unsigned | the completed step's index in the plan body's step order |

### `protection` (ADR-0024, ADR-0030)

The journaled outcome of the Protecting state — exactly one of three
arms, no arm silent.

| Field | Type | Meaning |
| --- | --- | --- |
| `plan` | Bytes(32) | the plan being protected for |
| `arm` | Text | `parse-backup-verified`, `absence-determined`, or `raw-capture-verified` |
| `artifact` | Bytes(32), backup arms only | the protection artifact's content hash (ADR-0030 Rule 2) |
| `store` | Text, backup arms only | `helper-protection-store` — the one helper-owned store ADR-0030 Rule 1 fixes; the per-OS location is helper deployment documentation, never journal content |
| `regions` | Array of Maps, raw-capture arm only | the captured write-target regions, each `{start: Unsigned, length: Unsigned}`, strictly ascending, non-overlapping, lengths nonzero |

The artifact's bytes have no field — "never its bytes" is held by the
schema's shape, and the crate's tests assert every byte-string position
in every encoded protection record is exactly 32 bytes.

### `compaction` (ADR-0029)

The durable declaration that legitimizes a reclaimed sequence range;
replay classifies a gap this record covers as policy, never corruption.

| Field | Type | Meaning |
| --- | --- | --- |
| `first` | Unsigned | the first reclaimed sequence number, ≥ 1 |
| `last` | Unsigned | the last reclaimed sequence number, ≥ `first` |
| `authority` | Text | `terminal-history-retention` — the only authority the liveness rule admits in v1 |

## The per-apply budget (ADR-0029)

`PER_APPLY_JOURNAL_BUDGET_BYTES = 268435456` (256 MiB of encoded
frames per apply), landing with this schema exactly as ADR-0029
requires. The magnitude is generous by design — over two hundred
maximum-size frames, millions of typical records — because the decided
part is the failure direction, not the number: exhaustion is a
journaled failure through Section 8's existing edges, never a
reclamation of live records. Enforcement is increment 4's.

## Response-data vocabulary carried for the helper surfaces

Two classes are defined beside the records because the ADRs assign the
record classes to this package while the enforcement is the helper
packages': the helper-computed authorization tier (ADR-0021: the
helper derives it from its own recomputed severity and flags), and the
dry-run refusal class (ADR-0026: `pending-qualification`,
distinguishable by type from every validation-failure class, so "the
combination is unqualified" can never read as "your plan is broken").

## Golden vectors

Each vector is the complete canonical encoding of one representative
record, hexadecimal, pinned by test against the crate's encoder. Plan
hash `0x11 × 32`, recovery plan hash `0x22 × 32`, artifact hash
`0x33 × 32`, instant `1700000000` (a plain epoch-seconds reading,
distinct from zero so a dropped instant cannot round-trip unnoticed).

`authorization-act` (tier `floor-act`):

```text
a5646b696e6471617574686f72697a6174696f6e2d61637464706c616e58201111111111111111111111111111111111111111111111111111111111111111647469657269666c6f6f722d61637466736368656d6176706172746d616e2e6a6f75726e616c2e7265636f72646e736368656d615f76657273696f6e02
```

`transition` (non-terminal, `validator-passes`, instant `1700000000`):

```text
a6646b696e646a7472616e736974696f6e64706c616e5820111111111111111111111111111111111111111111111111111111111111111166736368656d6176706172746d616e2e6a6f75726e616c2e7265636f726467696e7374616e741a6553f1006a7472616e736974696f6e7076616c696461746f722d7061737365736e736368656d615f76657273696f6e02
```

`transition` (terminal `failure-accepted`, effect `partial`, disposal
linkage, instant `1700000000`):

```text
a8646b696e646a7472616e736974696f6e64706c616e5820111111111111111111111111111111111111111111111111111111111111111166656666656374677061727469616c66736368656d6176706172746d616e2e6a6f75726e616c2e7265636f726467696e7374616e741a6553f1006a7472616e736974696f6e706661696c7572652d61636365707465646d7265636f766572795f706c616e582022222222222222222222222222222222222222222222222222222222222222226e736368656d615f76657273696f6e02
```

`checkpoint` (step 3):

```text
a5646b696e646a636865636b706f696e7464706c616e5820111111111111111111111111111111111111111111111111111111111111111166736368656d6176706172746d616e2e6a6f75726e616c2e7265636f72646a737465705f696e646578036e736368656d615f76657273696f6e02
```

`protection` (`parse-backup-verified`):

```text
a76361726d7570617273652d6261636b75702d7665726966696564646b696e646a70726f74656374696f6e64706c616e582011111111111111111111111111111111111111111111111111111111111111116573746f72657768656c7065722d70726f74656374696f6e2d73746f726566736368656d6176706172746d616e2e6a6f75726e616c2e7265636f7264686172746966616374582033333333333333333333333333333333333333333333333333333333333333336e736368656d615f76657273696f6e02
```

`protection` (`absence-determined`):

```text
a56361726d72616273656e63652d64657465726d696e6564646b696e646a70726f74656374696f6e64706c616e5820111111111111111111111111111111111111111111111111111111111111111166736368656d6176706172746d616e2e6a6f75726e616c2e7265636f72646e736368656d615f76657273696f6e02
```

`protection` (`raw-capture-verified`, regions `{512, 8}` and
`{4193792, 8}` — the 2j fixture's two signature ranges, chosen as a
familiar shape):

```text
a86361726d747261772d636170747572652d7665726966696564646b696e646a70726f74656374696f6e64706c616e582011111111111111111111111111111111111111111111111111111111111111116573746f72657768656c7065722d70726f74656374696f6e2d73746f726566736368656d6176706172746d616e2e6a6f75726e616c2e7265636f726467726567696f6e7382a2657374617274190200666c656e67746808a26573746172741a003ffe00666c656e67746808686172746966616374582033333333333333333333333333333333333333333333333333333333333333336e736368656d615f76657273696f6e02
```

`compaction` (range 1..=2, `terminal-history-retention`):

```text
a6646b696e646a636f6d70616374696f6e646c617374026566697273740166736368656d6176706172746d616e2e6a6f75726e616c2e7265636f726469617574686f72697479781a7465726d696e616c2d686973746f72792d726574656e74696f6e6e736368656d615f76657273696f6e02
```

## What this schema deliberately does not carry

- **No report bodies, no tool output.** JRN-005 bounds and redacts
  embedded output at the helper surface; v1 has no field for it.
- **No paths, no store locations.** JRN-004's documented per-OS
  location is deployment documentation.
- **No plan content.** Records reference plans by MODEL-005 body hash
  only — the WP-010 joint sequencing each ADR names, discharged
  hash-only, with no WP-010 body schema change required.
- **No retention state.** Liveness, the linkage closure, the budget's
  accounting, and `CoveredRanges` derivation from compaction records
  live in the crate's `retention` module (increment 4); this schema
  itself carries none of it — a compaction record declares a reclaimed
  range and its authority, nothing more.
