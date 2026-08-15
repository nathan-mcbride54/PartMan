# SI-35 resolution round — 2026-08-09

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** ADR-0014 fixed the axis and deliberately left SI-35
Open; this round recommends the minimal honest package that moves it to
Resolved, for acceptance before any of it is built. Untracked session
artifact (`docs/reviews/**`, WP-000).

## What resolution requires, and the deadlock it must not create

SI-35's evidence clause has always gated acceptance of the chosen option
on "(3) a demonstration that the chosen option refuses rather than
proceeds on `gpt-conflicting-tables-512.img`" — and the register records
(3) as openable only now that an option exists. Meanwhile the ordering
runs: increment 3 needs SI-35 resolved; the privileged helper is built
*after* increment 3's domain model; so **a resolution that requires the
running helper deadlocks the entire schedule**. The clause must be
satisfiable by the option's *mechanism* — the raw-sector parser ADR-0014
named as the contract — exercised at Tier 1 over the fixtures the clause
itself names. This round says that in terms rather than discovering it
mid-build, and the adversarial round below attacks it as possible
laundering.

## Recommendation, in five parts

### 1. The parser: `crates/table-parser`, pure over caller-supplied windows

A new WP-010-owned crate (reservation first, per the governance
ordering). Pure functions over byte slices — no I/O, no process, no
device: `classify(head, tail, geometry) -> Classification`, where the
caller supplies the first/last windows of the medium and its sector
geometry. That is the exact shape M10 measured as separating (first and
last 64 KiB bracket both GPT copies in every fixture), it makes the
parser's promise independent of who reads the bytes — the helper later,
fixture slices at Tier 1 now — and it keeps the 4Kn case honest, since
`gpt-basic-4kn` exists to prove a 4Kn table "is not a 512-byte table
with different numbers." Scope: GPT complete (both copies, both CRCs,
entry arrays); MBR classification (standalone, protective, hybrid — the
protective/hybrid distinction requires reading MBR partition types
anyway); APM recognition (`apm-basic-512` exists and its big-endian
fields are the trap the fixture was built to set). Section 11.4
discipline throughout: `unsafe`-free, bounded, refusing on anything
outside its stated grammar, fuzz target landing in the same chain the
plist reader used (WP-010 target + WP-000 registration).

### 2. The classification, agreeing with the evidence layer's recorded commitments

WP-020's fixture-evidence layer already binds interpretations, landed
and mechanically checked; the resolution honors them rather than
re-litigating:

| Input shape | State | Named fixture |
| --- | --- | --- |
| Both copies valid, identical content | `Present {checksum}` | `gpt-basic-512`, `gpt-basic-4kn` |
| Both copies valid, different content | `Indeterminate` (ambiguous — no authority picks a side) | `gpt-conflicting-tables-512`, per its recorded claim |
| Invalid primary, valid backup | `Present {checksum over the valid copy's content}` **plus a `primary-invalid` condition** | `gpt-invalid-primary-valid-backup-512` — its recorded claim is "recoverable, and NOT ADR-C3 Indeterminate," and UEFI names the backup authoritative when the primary fails CRC |
| Valid primary, missing/invalid backup | `Present {checksum}` plus a `backup-missing` condition | `gpt-missing-backup-512`, by the same one-valid-authority logic |
| Neither copy valid | `Indeterminate` (unreadable) | *(no fixture yet; test obligation below)* |
| No table signatures at the locations where tables live | `Absent` — reading the defined locations and finding none is the positive observation | `blank-512`; also `ext4-with-stale-mdraid-090-512`, `luks2-whole-disk-512`, `lvm2-pv-orphan-512` — an absent **table** never asserts absent **data**, the 3.1.0 rejection made normative by SAFE-003, and the signature facts travel as their own FS-004 nodes |
| Valid GPT plus non-protective MBR | `Present {gpt checksum}` **plus a `hybrid-mbr` condition** | `hybrid-mbr-gpt-512` — hybrid is a detected condition (INV-003), not a fourth state; what a *plan* may do under it is PART-014/SI-11 material, deliberately not decided here |
| Standalone MBR / APM | `Present {checksum}` per that scheme's basis | `mbr-basic-512`, `apm-basic-512` |

**Conditions are body facts beside the state, not new states.** ADR-C3's
vocabulary stays three-valued; SAFE-005 hooks conditions ("corrupt
metadata" disables the affected writes) without collapsing a recoverable
medium into `Indeterminate` — which would have forbidden the REC-001
repair its fixture exists to represent.

### 3. `Present {checksum}` is computed over copy-invariant content

The open question Part 6's precondition 1 has carried since round one:
what the checksum covers. Recommendation: SHA-256 over the scheme's
**copy-invariant content** — for GPT: disk GUID, usable range, entry
count and size, and the partition entry array bytes; never the raw
header, whose `MyLBA`/`AlternateLBA`/`PartitionEntryLBA`/CRC fields
differ between copies *by design*. This makes "both copies agree" and
"checksum" the same fact, keeps MODEL-005's body-stability (re-probe of
unchanged hardware reproduces it byte-for-byte), and gives the
invalid-primary case a well-defined value (the valid copy's content).
MBR and APM each get their stated equivalent. The exact byte recipe is
canonical-encoding material and lands in `schemas/` with the spec
change, cross-language golden tests included — a checksum two
implementations compute differently is a body-hash split waiting to
happen.

### 4. What "the demonstration" honestly is, pre-helper

Three pieces, each mutation-verified, landing with the parser: the named
fixture classifies `Indeterminate` with the ambiguity basis recorded;
the classification type itself carries no proceed-enabling reading (no
`is_safe`, no default arm — a `match` on the state must handle
`Indeterminate` explicitly, pinned the compile-fail way the interlock
pins non-cloneability); and spec 8.0.0's PART-001 categorical invariant
plus SAFE-005's existing rule make "proceeds" normatively impossible for
every writer that will ever exist. What this is **not** — and the
register entry must say so — is an end-to-end refusal by a running write
path, which cannot exist before increment 3 and the helper. The claim
recorded at resolution: *the mechanism classifies correctly and the
specification forbids proceeding on that classification*; the end-to-end
re-demonstration is a named obligation on the first write-capable
increment, in the register's own text.

### 5. Spec 8.0.0 and the register move

One major change carrying: ADR-0014's four enumerated amendments
(PART-001's categorical invariant — the major; the ADR-C2 authoring
verb; Section 6's bound-at-validation wording; the client
emits-no-table-state prohibition); ADR-C3's `Present {checksum}` basis
per part 3; SI-35 → Resolved with the part-4 caveat and the end-to-end
obligation in its banner; the gate count dropping to eight items, five
direct. Sequenced after the parser and demonstration merge — the
register's evidence clause is satisfied by things that exist, never by
things scheduled.

## The adversarial round

**Attack 1 — "part 4 launders the evidence clause: 'refuses rather than
proceeds' means a refusal, and you are shipping a classification."**
The round's hardest attack, and it is *right* that the difference must
be stated rather than blurred — that is why part 4's register text
names what was and was not demonstrated, and binds the end-to-end
re-demonstration to the first write-capable increment as register text,
not a review memory. What the attack cannot supply is an alternative
that isn't the deadlock: the clause predates the ordering insight that
the helper postdates increment 3. Read as "running write path refuses,"
the clause makes SI-35 unresolvable before the thing SI-35 blocks.
Fail-closed resolution of *that* conflict is part 4's shape: demonstrate
everything that can exist, forbid the rest normatively, record the
residue as an obligation. Sustained as a caveat, not a blocker.

**Attack 2 — "Present-plus-condition for a damaged primary is
write-enabling spin; SAFE-005 says corrupt metadata disables writes."**
Refuted by reading both halves: the condition *is* the SAFE-005 hook —
`primary-invalid` disables the affected writes exactly as "corrupt
metadata" demands — while the state stays `Present` because the table
content is determinable from one valid authority, which is the fixture
evidence layer's own recorded, mechanically-checked claim ("recoverable,
and NOT ADR-C3 Indeterminate"). Collapsing it to `Indeterminate` would
misreport a determinable table as undetermined — ADR-C4's conflation in
the other direction — and would make REC-001's repair flow start from a
state that denies what the helper just read.

**Attack 3 — "the checksum basis belongs in an ADR, not smuggled through
a spec row."** Considered, and the mechanics are deliberate: the basis
is ADR-0014's axis *applied* — copy-invariant content is the only basis
compatible with body-stability and the both-copies-agree rule already
decided — so it lands as the ADR-C3 amendment ADR-0014's Consequences
enumerated, cited to this round and to Nate's acceptance of it. If
review at draft time finds genuine open alternatives (it should name
one), an ADR-0016 reservation costs one governance PR; the round found
none: every alternative either hashes per-copy fields that differ by
design or hashes raw sectors whose backup-pointer bytes make equal
content hash unequally.

**Attack 4 — "hybrid-as-condition quietly decides PART-014's hybrid
policy."** Refuted by scope: the condition records that the medium is
hybrid — INV-003's detection duty, nothing more. What a plan may *do* on
hybrid media (refuse, degrade, one-scheme-wins) is PART-014
classification inside SI-11's round, untouched; the row says so in the
table itself.

**Attack 5 — "a new crate before increment 3 is increment 3 by another
name."** Refuted by contents: the parser exports a classification of
bytes, not Section 5 domain types — no node, no snapshot, no identity
record, no hash of anything but table content. It is upstream evidence
machinery, the same class as the fixtures crate it will take as a dev
dependency. The one type it shares with the future model (`TableState`)
is ADR-C3's closed three-value vocabulary plus conditions — text that
has been normative since 3.1.0. Increment 3 consumes it; nothing here
preempts what increment 3 decides about everything else.

**Attack 6 — "both-copies-invalid has no fixture, so that row is
untested prose."** Sustained and converted: the parser PR's obligations
include a `gpt-both-copies-invalid-512` fixture (WP-020's generator,
its own evidence claim) or an explicit recorded reason it cannot be
deterministic. No silent gap.

## Mechanics, if accepted

PR chain, each on the pattern this week established: (1) governance —
reserve `crates/table-parser/**` and the fixture addition's evidence
row under their owners; (2) WP-010 — the parser, the classification
tests over every catalogue fixture, the demonstration trio, the new
fixture; (3) the fuzz chain — WP-010 target (`classify` never panics,
bounds hold), WP-000 registration (four targets, workflow arithmetic);
(4) WP-010 — spec 8.0.0, ADR-C3 basis, register move to Resolved with
the part-4 caveat, gate count to five direct. Each lands green before
the next; the spec change goes last so the register never cites
scheduled work as evidence.
