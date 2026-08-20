# ADR-0056: The Linux mutation toolset — the product's own table encoder; native tools, launched and floored, for file systems

- Status: Accepted
- Date: 2026-08-20. Made on the adversarially reviewed recommendation
  round of the same day
  (`docs/reviews/LINUX_MUTATION_TOOLSET_ROUND_2026-08-20.md`, a
  committed session record; option D and all four of its §5 decisions
  taken by the decision owner), under route (b) — the decision
  WP-L110's assignment claimed increment-gated before increment 4b and
  ADR-0054 explicitly handed over. Recorded before its first consumer
  is written — merging is not acceptance.
- Spec version: 20.0.0 (major under §0.1 — LIN-001's mutation sentence
  changes meaning; ADR-0054's precedent on the same sentence)
- Work packages blocked: none (the first consumer is WP-L110
  increment 4b, which stays gated on the launcher-home round, route c)
- Requirement IDs: LIN-001, LIN-002, SAFE-004, SAFE-005, SAFE-009,
  CAP-004, CAP-006, Section 9, HLP-002, ADR-0014, ADR-0021, ADR-0054
- Decision owners: Nate McBride

## Context

LIN-001 read, since 18.0.0: discovery through the measured
client-readable interfaces; *"UDisks2, libblockdev, or authoritative
native tools for authorization and mutations, behind the helper's own
recorded route decision (WP-L110; HLP-002, LIN-009)."* This is that
recorded decision, for the mutation half. The authorization *mechanism*
— how the helper asks polkit about its client — is the apply-ceremony
round's own follow-up on DR22–DR24's rows and is deliberately not
taken here.

The round was taken on measured substrate, per WP-L110's filed
tool-presence row (DR25, all three tiers, 2026-08-20): `sgdisk` and
`sfdisk` are present by default on **every** tier, at three distinct
util-linux feature generations (2.37.2 / 2.38.1 / 2.42.2); the `mkfs.*`
family is tier-gapped — Debian 12 ships only the ext family, and F2FS
and exFAT have **no maker on any default image**; libblockdev exists on
one tier of three (jammy's 2.26, riding in with udisks2, which every
acceptance guest purges); and a launched version query does not cover
the family — `mkfs.fat` answers no version spelling at all, and the
delivered probe semantics fail-close an unparsed banner to blocked.

Against the delivered types: the helper authorizes **a plan hash over
its own authored snapshot** (HLP-002; `AdmittedPlan`; the SEC-002 arms;
increment 4a's journal-borne apply). The read half of the table format
is already product code under ADR-0014's bounded, enumerating,
fuzz-obligated architecture; no encoder exists in product code; and the
repository's fixtures crate already authors GPT/MBR bytes as test code
and checks them against the real probers in CI — proof the encoding
knowledge is tractable and prober-verified in-tree, cited as precedent,
not linked as a component.

## The decision

1. **Partition-table writes are the product's own encoder.** The
   helper's first GPT/MBR table writer is product Rust
   (`#![forbid(unsafe_code)]`), the inverse of ADR-0014's parser
   architecture — bounded, reviewed, golden-vectored, and round-trip
   fuzzed against the delivered parser under Section 11.4's harness —
   emitting **exactly the bytes the admitted plan's steps resolve to**
   over the helper-authored snapshot, written under the journal and
   CONC-001's lock through a read-write sibling of the helper's byte
   layer, and proven on fixture images in disposable environments only
   (SAFE-001). The native tools' role on the table path is the one
   `cargo xtask probe-fixtures` already demonstrates: **independent
   verifiers of the product's bytes, never their author.** The
   plan-hash binding runs unbroken from validation to the device byte,
   and every refusal stays typed.
2. **File-system operations are authoritative native tools through the
   SAFE-004 launcher, per installed capability** (LIN-002): fixed
   absolute allow-list, structured argv, bounded output, a
   caller-stated deadline, sanitized environment; a tool's version
   floor enters the CAP-006 store **with its first invocation**, basis
   stated, per that store's own rule; absence fails closed (SAFE-005).
   Measured today that means: F2FS and exFAT advertise no capability on
   any default tier image, and Debian 12 advertises only the ext family,
   until their makers are installed — packaging (LIN-008) or the
   operator widens the matrix, never the helper assuming.
3. **The version-verification discipline, fixed.** SAFE-004's "verified
   executable identity/version" is read as: identity is the fixed
   absolute path plus a recorded content digest where the invocation
   demands one; the **version comes from the package manager's record
   and/or a recorded content digest — the launched version query is
   corroboration, never the sole source.** Ground: DR25 measured that
   `mkfs.fat` answers no version spelling, and the delivered
   `ToolProbe`/`tool_state` semantics fail-close an unparsed banner to
   `OutOfRange` — banner-only probing would block a correctly installed
   tool at every version, forever, while "verifying" the rest by
   trusting the tool's own banner.
4. **UDisks2 and libblockdev are not mutation routes.** The parked
   UDisks2 ≥ 2.9 floor (ADR-0054) gains no invoker on the taken route:
   it stays in that ADR's record and enters no store. Nothing here
   forecloses the LIN-003…007/LIN-010 rounds weighing a library for
   their own domains (the cryptsetup and LVM ecosystems are
   library-shaped in a way partitioning is not); those decisions belong
   to those rounds.
5. **Not decided here, each named with its owner:** the polkit
   authorization mechanism (the apply-ceremony round's follow-up); the
   launcher's home (route c, WP-035 in the room); the NTFS write stack
   (ADR-L1, unwritten — DR25 measures `mkfs.ntfs` present on jammy
   only); packaging dependencies (LIN-008); the first CAP-006
   qualification rows (WP-050's reviewed act).

## Options considered

### UDisks2 as the mutating actor (option A)

Rejected on rows and delivered types: the daemon is absent by default
on two of three tiers (DR18, DR19) and purged on the third's acceptance
guests; the plan-hash binding ends at the bus — udisksd derives and
acts on its own view, a structural TOCTOU no journal entry can witness;
SAFE-004's identity and allow-list discipline cannot see a
bus-activated daemon (ADR-0054's provoked-launch argument, applied to
the write path); and DR23 measured a root subject authorized for
`auth_admin` with no agent and no prompt, so the daemon's own polkit
layer is vacuous for a root helper-client — a second, empty
authorization behind ADR-0021's real one. Not rejected on licensing
(ADR-0006's "IPC is not linking" stands).

### libblockdev linked into the helper (option B)

Rejected: C FFI in the one privileged process, against an assignment
that says reviewed `unsafe` is permitted and none is planned (SAFE-009);
one default tier of three, at 2.26 while the current line is 3.x with a
breaking API change (archive knowledge, flagged in the round); and a
`dlopen` plugin model that re-asks SAFE-004's verified-identity question
one level down, where an allow-list of paths the loader does not consult
cannot answer it.

### Authoritative native tools for both halves (option C)

Rejected **for the table half only**: the executed artifact becomes a
translation of the validated plan into a tool dialect at three measured
feature generations; the tool then applies its own policy (re-reading
the device, recomputing CRCs, rewriting the protective MBR, relocating
the backup) rather than the plan's resolved bytes; refusals become
prose to parse per generation; and the write path's core acquires
per-tier version floors for authoring the very format the product
already parses bindingly. Kept, as decision 2, for the file-system
half — where the tools *are* the authority (an ext4 file system is what
e2fsprogs says it is) and where reimplementation would be the real
recklessness.

## Consequences

- **Positive:** the reviewed, fuzzed, hashed artifact is the artifact
  written; refusals stay typed end to end; table writes carry no tool
  floor and work on every default image; the CAP-006 store gains floors
  only for tools actually invoked; the native tools are retained
  exactly where they are authoritative, and as independent verifiers of
  the product's table bytes.
- **Negative, accepted knowingly:** the product owns GPT/MBR
  correctness end to end — backup placement, CRC discipline, the
  protective MBR, alignment, 4Kn — and an encoder defect writes a bad
  table. The mitigations are named and owed, not assumed: golden
  vectors, round-trip fuzz against the delivered parser, the
  real-prober CI acceptance, SAFE-001 disposable-only proving, review.
  A major version bump for one sentence. More code than shelling out.
- **Evidence obligations:** (1) the encoder round-trips against the
  delivered parser under fuzz before any Tier-2 write; (2) the written
  table is verified by an independent native prober in every Tier-2
  acceptance that writes one; (3) each launched tool's first invocation
  enters its floor with a stated basis; (4) the version-verification
  discipline is held by a test — a bannerless tool must still be
  verifiable, and a floor entered from a banner alone must be refused;
  (5) WP-L110's consequential objective edit lands as that package's
  own act.

## Verification

When increment 4b lands on this ADR: the written table's bytes re-parse
to the plan's resolved state (round-trip at Tier 1; prober-verified at
Tier 2 in disposable guests); every file-system operation reaches its
tool only through the launcher, from the fixed absolute allow-list; a
missing maker blocks exactly the capabilities that need it, with the
typed reason; and the mutations include: the encoder's CRC or backup
placement mutated (killed by the parser round-trip), a tool invoked
outside the launcher (killed by the structural guard), a version floor
satisfied from a launched banner alone (killed by the discipline's
test).

## Revisit conditions

- A native table writer that takes **bytes** rather than a dialect —
  option C's table half reopens with its translation cost collapsed.
- Review finding the encoder's maintenance burden outweighing the
  binding argument — option C's table half reopens with its dialect
  floors and prose-parsing accepted into CAP-006 from the start.
- The LIN-003…007/LIN-010 rounds — their library decisions are their
  own, and nothing here prejudges them.
