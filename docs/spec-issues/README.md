# Specification issues

Section 1.11 and Section 0.2 of `AGENT_BUILD_SPEC.md` require that when two
requirements conflict, work stops and the conflict is filed, rather than an
implementer silently picking a side. This directory is where those filings live.

Each entry states the requirements that disagree, why the disagreement cannot be
settled by reading harder, the options, and what the choice costs. **None of
them proposes an answer as though it were decided.**

## How these were found

Seven parallel readers extracted the Section 5 domain-type requirements from the
specification during WP-010 increment 3, and three adversarial reviewers checked
the resulting design for completeness, safety, and encoding fidelity. Thirty
conflict reports and twenty-five design findings were deduplicated into the first
twenty-six issues below.

SI-27 was found by the second attempt at the protection model, and SI-28 through
SI-32 by the third, which put one design to five adversarial lenses. That is the
pattern to expect: each attempt at a hard decision surfaces conflicts the previous
one could not see, because it had not yet reached the layer they live in.

## Why this blocked increment 3

Most of the blocking ones are **hash-visible**: the choice changes the canonical
bytes of a plan or topology snapshot. Under MODEL-005 and ADR-C1, changing a
hashed artifact's shape later is not a refactor — it invalidates every hash the
product has ever issued, and there is no migration for an authorization token
that no longer matches. Guessing now and correcting later is the one option with
no cheap exit.

## Legend

- **Blocks 3** — must be decided before WP-010 increment 3 writes the type.
- **Hash-visible** — the decision changes canonical bytes.
- **Later** — decidable before the work package named, not before increment 3.
- **Editorial** — a defect in normative text with no decision to make, so it
  blocks nothing and is fixed by the next spec change.

---

# Part 1 — Blocking WP-010 increment 3

## Status of every issue

**This table is the only authoritative status.** Nothing else in this repository
should restate a count or a blocker list. A progress review on 2026-07-28 found
the previous hand-written summary saying "five remain" directly above seven
names, and this document disagreeing with `docs/work-packages/WP-010.md` about
SI-31 — which made the register unusable as the dependency gate Section 1.11
requires it to be. Counts drift; a table that must be edited to add a row does
not.

Every issue appears exactly once. **Nothing gates increment 3.** SI-28 —
the register's last direct blocker — was reclassified off the gate on
2026-08-09 by the decision owner, the SI-37 pattern: it stays
**Mitigated-open**, its interim conservative floor in force and unchanged,
its relaxation route staying ADR-0017's named revisit condition; only its
class moved, because the floor refuses the affected population from
decided, contract-readable facts (transport class, removability,
identifier presence — no undecided hashed field is an input to it), and a
refused population can hold no issued authorization for a later
discriminating mechanism to invalidate. The priced cost, accepted
knowingly: if that mechanism ever adds an identity-record field, it pays a
MODEL-003 schema major after implementation exists — accepted because the
alternative was gating the entire domain model on a mechanism with no
measurement route.
Two former transitive blockers are resolved: SI-12
in spec 4.3.0 by ADR-0011, and SI-38 in spec 6.0.0 by ADR-0013. Six direct
blockers are resolved: SI-39 in spec 7.0.0 by ADR-0015, SI-35 in spec 8.0.0
by ADR-0014's axis carried to its instrument — the register's first
measurement-born direct blocker to close end to end — SI-34 in spec 9.0.0
by ADR-0016, the placement question closed by the architecture the SI-35
resolution built, SI-33 in spec 10.0.0 by ADR-0017, the witness
designed inside its own measured limits, SI-11 — the register's
longest-running direct blocker, four rounds — in spec 11.0.0 by ADR-0018,
the protection closure computed, total, and fail-closed, with SI-29 and
SI-30 resolved within its decision and SI-37 reclassified (open, off this
gate, its dual-path matrix now the acceptance evidence for relaxing the
populations the closure blocks rather than a precondition on the type),
and SI-27 in spec 11.1.0 by ADR-0019 — node names as derived positional
addresses whose collisions produce counted, indeterminate, always-encodable
groups, the two new edge kinds typed, the theorem re-proof handed to
increment 3 as a property test. SI-36 is withdrawn and gates nothing.

| Class | Meaning | Issues |
| --- | --- | --- |
| **Resolved** | An ADR and spec change landed the decision | SI-01, SI-02, SI-03, SI-04, SI-05, SI-06, SI-07, SI-08, SI-09, SI-10, SI-11, SI-12, SI-15, SI-16, SI-17, SI-18, SI-19, SI-20, SI-21, SI-22, SI-23, SI-24, SI-27, SI-29, SI-30, SI-31, SI-32, SI-33, SI-34, SI-35, SI-38, SI-39, SI-40 (by ADR-0020 with no spec change — the decision amends no normative text, recorded in its banner so the absent spec change reads as deliberate, not forgotten) |
| **Direct blocker** | Must be decided before increment 3 writes a type | *(none — SI-28 reclassified below, 2026-08-09)* |
| **Transitive blocker** | A separately sequenced prerequisite decision that must resolve before a direct blocker can be decided | *(none)* |
| **Input** | A subquestion or evidence case resolved within the consuming direct blocker's decision | *(none — SI-29 and SI-30 resolved within SI-11's decision; SI-37 reclassified below)* |
| **Later** | Decidable before the named work package, not before increment 3 | SI-13, SI-14, SI-25, SI-26, SI-37 (before the spec change that first moves a closure-blocked multipath-capable population to `Permitted`; ADR-0018), SI-28 (**Mitigated-open**, floor in force; before the round that either relaxes the floor under ADR-0017's revisit condition or lands a discriminating mechanism; reclassified off the increment-3 gate 2026-08-09) |
| **Withdrawn** | Retained as history after the filing was shown not to be a conflict | SI-36 |

No direct blockers remain. SI-28's Mitigated-open state — the interim
conservative floor of Part 7's aftermath, in force and unchanged; SI-33's
witness landed as its refusal input; relaxation parked behind ADR-0017's
named revisit condition — is recorded in its entry and its Later-class row
above, with the reclassification banner in the entry carrying the decision
and its priced cost.

**SI-31 is resolved in spec 4.1.0 by ADR-C6.** Its answer is plain unsigned
bytewise ordering over each element's full canonical encoding, for
schema-declared sets only. Semantic arrays retain order and `pce/1` is unchanged.
The Rust and TypeScript set boundaries now encode sort keys at the element's
actual enclosing depth, so the standalone-encoding reset the issue recorded is
closed rather than merely documented. The shared fixture is intentionally
unsorted and includes both the comparator disagreement and exact depth boundary,
so it exercises the decision instead of preserving a prearranged answer.

Round one is recorded in Part 4, round two in Part 5, round three in Part 6, and
SI-28's fourth round in Part 7. SI-11's fourth round — the accepted one — is
recorded in ADR-0018, its session round document being an untracked artifact.

**SI-28's interim posture is decided, and it is a mitigation rather than a
resolution.** The decision owner chose the conservative floor: destructive
whole-device operations on removable media behind a bridge that exposes no
medium-attributable identifier are refused, and the continuity witness (SI-33)
is the route by which that refusal may later be relaxed. That order is
deliberate — a blunt rule can be narrowed once a discriminating mechanism is
proven, whereas a permissive rule cannot be tightened after plans have been
issued against it. **SI-28 stays open**, because the floor does not discriminate
two media and Part 7's warning against false closure applies to it as much as to
any other proposal.

> **Dated history — retained as filed.** The five paragraphs below, through
> "A second instance has since been measured, in a different layer.", record
> the state as of the SI-34 filing. Both questions they hold open have since
> closed: SI-35 resolved in spec 8.0.0 by ADR-0014, and SI-34 resolved in
> spec 9.0.0 by ADR-0016 — the placement question closed by the architecture
> the SI-35 resolution built, so the closing advice below is addressed to a
> decision that no longer exists. Current status lives in the authoritative
> table above and the SI-34 and SI-35 entries below.

**Half the approach is settled; the other half is reopened.** Protection is
proven by computation and never accepted as a client declaration — that has
survived every round and is not in question.

Whether the derived verdict is **frozen into the hashed body** is open again.
Round two justified it by concluding that every client/helper asymmetry is a
roster-identity fact, so the two sides would always compute the same graph. **A
fixture measured in WP-020 refutes that universal premise within one named
finite projection:** on bytes carrying a live ext4 superblock and a stale
mdraid 0.90 superblock, the retained single-answer interface reports only the
stale signature while the retained enumerating interface reports both. That
observation difference is not roster identity.

Be precise about what the fixture does and does not prove. It refutes the
*universal roster-identity premise* for that named projection. It does **not**
establish an actual cross-privilege difference between the complete client and
helper graphs, or that the two sides reach different *verdicts* — both remain
untested, and `docs/quality/observability.md` states the distinction. Neither
"S1 is disproved" nor "the fixture changes the verdict" is a correct reading.

What remains is therefore mechanism **and** one reopened decision, filed as
SI-34.

**A second instance has since been measured, in a different layer.** SI-35
records that `libblkid` produces byte-identical output for a healthy GPT and for
one whose two valid tables disagree — so ADR-C3's `Indeterminate` state is not
observable through the interface an unprivileged client reads, in the
*partition-table* layer rather than the signature layer. Two independent
instances retire the reading that client/helper asymmetry is a peculiarity of
signature probing, and any answer proposed for SI-34 should be tested against
SI-35's case before it is accepted.

**Read SI-28 first.** It is the only defect found so far that destroys data
without requiring a bug in anything being designed, and it lands on an
already-accepted decision. One attempt to resolve it has already failed, and the
reason is in Part 7: it is not a classification problem, so reclassifying the
record does not close it.

## SI-01 Identity strength is not computable at discovery time

> **Resolved in spec 3.1.0 by ADR-C3** — strength is a property of one record; matching is a separate helper-side verdict.

**Requirements:** SAFE-003, INV-002, Section 6 · **Blocks 3, hash-visible**

SAFE-003 defines strength as a comparison outcome — a stable hardware identifier
"plus size, sector geometry, and partition-table checksum all **match**". But it
also requires that "each identity record MUST be classified", INV-002 requires
reporting strength at discovery, and Section 6 requires plans to carry bound
identities with strength. In all three there is no counterpart to match against.

Either strength at discovery means "a stable hardware identifier and the other
fields are **present**", and the matching clause describes only the helper's
re-probe; or the field is meaningless before re-probe. The two readings produce
different plan bytes for the same device.

**Options:** (a) presence-based at discovery, match-based at re-probe, with the
two named distinctly; (b) strength is re-probe-only and absent from discovery
and plan bytes — which contradicts INV-002 and Section 6 as written.

## SI-02 Blank and table-less media can never be Strong

> **Resolved in spec 3.1.0 by ADR-C3** — option (b): partition-table state is three-valued, so a positively observed absence is determined and a blank device can be Strong, while an unreadable table cannot. The proposal to also exempt blank-media initialization from severity 4 was **rejected**; see the ADR.

**Requirements:** SAFE-003, INV-003, PART-001 · **Blocks 3**

Strong requires a partition-table checksum match. INV-003 requires representing
missing tables and PART-001 requires initializing blank media, where no checksum
exists. Read literally, a factory-blank NVMe exposing both serial and WWN is
Weak, so every first-initialization takes the weak-identity path: typed
device-name confirmation (UI-009) and refusal of unattended apply.

**Options:** (a) an absent table is a vacuous match, so blank media can be
Strong; (b) it is a distinct third case with its own policy; (c) the literal
reading stands and first-initialization is deliberately high-friction.

This decides whether `PartitionTable.checksum == None` is identity-degrading.

## SI-03 Is provenance inside the hashed identity bytes?

> **Resolved in spec 3.0.0 by ADR-C2** (`docs/adr/0002-hashed-artifact-body-and-envelope.md`).

**Requirements:** MODEL-004, MODEL-005, SAFE-003, HLP-003 · **Blocks 3, hash-visible**

MODEL-004 requires *every* discovered property to record source adapter and
confidence. Identity fields are discovered properties, and they are covered by
the plan hash. The specification never says whether the provenance metadata is
inside the hashed bytes.

If it is, a plan authorized while one adapter reported the device rehashes
differently after an adapter or version change on unchanged hardware, breaking
HLP-003's binding. If it is not, a `conflicting` identity observation sits
outside the authorization boundary that SAFE-003 and SEC-001/002 establish.

**Must be decided before the identity encoding is frozen.** Changing it later
invalidates every hash previously issued.

## SI-04 MODEL-004 cannot express its own `conflicting` value

> **Resolved in spec 3.1.0 by ADR-C4** — option (a): provenance is a set of observations held in the envelope, with the four confidence values derived rather than stored. The proposal to also collapse disputed body values to a single resolution bit was **rejected**; see the ADR.

**Requirements:** MODEL-004, INV-007, SAFE-005, UI-010 · **Blocks 3, hash-visible**

MODEL-004 requires each discovered property to record "its source adapter"
(singular) while permitting the confidence value `conflicting`, which by
definition means two or more adapters disagreed. A single-source envelope cannot
say which sources disagreed or what each reported. SAFE-005 requires ambiguous
identity to disable the affected write, which needs to know what disagreed, and
INV-007 requires raw evidence to be inspectable.

**Options:** (a) the provenance record holds multiple `(adapter, value)`
observations, changing the shape MODEL-004 states; (b) `conflicting` is an
unexplained flag and losing observations live only in the diagnostic bundle,
outside the hash.

## SI-05 A plan cannot contain its own hash

> **Resolved in spec 3.0.0 by ADR-C2** (`docs/adr/0002-hashed-artifact-body-and-envelope.md`).

**Requirements:** Section 6, MODEL-005, ADR-C1, HLP-001, SEC-001 · **Blocks 3, hash-visible**

Section 6 requires `OperationPlan` to contain the "Cryptographic plan hash",
while MODEL-005 defines that hash as SHA-256 over the plan's canonical bytes. A
field cannot simultaneously be inside the bytes and be the hash of those bytes.

The model needs an envelope/body split — hash outside, everything else inside —
that neither Section 6 nor ADR-C1 states. **Which Section 6 items belong to the
body versus the envelope is undecided and load-bearing**, because HLP-001
applies by plan hash and SEC-001 authorizes only exact hashes.

## SI-06 What is inside a TopologySnapshot's hash?

> **Resolved in spec 3.0.0 by ADR-C2** (`docs/adr/0002-hashed-artifact-body-and-envelope.md`).

**Requirements:** MODEL-005, PLAN-006, HLP-004, CONC-004, MODEL-004 · **Blocks 3, hash-visible**

PLAN-006 requires the helper to re-discover topology and reject a mismatch,
which implies comparing a fresh capture's digest against the recorded one. But
MODEL-005 hashes the whole artifact's canonical bytes, and that artifact
includes CONC-004's transitional marking, MODEL-004 provenance, and any capture
timestamp.

If those are inside the hash, two captures of a physically identical topology
never produce equal digests and the freshness check can never pass. If they are
outside it, the hash does not cover the whole artifact and MODEL-005's
single-canonical-encoding guarantee weakens.

The specification does not separate topological content from capture metadata.

## SI-07 StorageContainer, StoragePool, and RaidSet have no boundary

> **Resolved in spec 4.0.0 by ADR-C5** — one `Aggregate` node with a closed technology discriminant, membership as a `Backs` edge of unbounded in-degree, and a self-reported member count so that detect-only is a function of kind *and* membership.

**Requirements:** Section 5, MODEL-002, MAC-003, MAC-010 · **Blocks 3, hash-visible**

Section 5 lists all three and defines none. MODEL-002 lumps "Storage Spaces,
LVM, RAID, APFS containers, and Btrfs multi-device file systems" together as
non-linear relationships. Nothing says whether an LVM volume group, an mdraid
array, or a Storage Spaces pool is a container, a pool, or a RAID set — so the
same membership edges can be modelled twice, and the choice is hash-visible.

MAC-003 additionally requires APFS physical stores (plural, many-to-one for
Fusion per MAC-010), which a one-to-one `StorageContainer` cannot express.

## SI-08 Btrfs multi-device: container, or file system with many backings?

> **Resolved in spec 4.0.0 by ADR-C5** — a file system with an ordered set of n ≥ 1 backings, single-device being the cardinality-1 instance of the same shape, so `btrfs device add` changes the member set and not the node shape.

**Requirements:** MODEL-002, FS-003, LIN-006, MAC-003 · **Blocks 3, hash-visible**

MODEL-002 places file system strictly above volume, yet requires representing
Btrfs multi-device file systems, where one file system spans several devices and
performs the aggregation role a container performs for APFS. APFS gets an
explicit container type; Btrfs gets none.

## SI-09 FS-004 detects things that are not file systems

> **Resolved in spec 4.0.0 by ADR-C5** — signatures materialize as `BackingSignature` nodes with an optional consumer, `FileSystemKind` stays purely file systems, and the routing rule for an unrecognized signature is stated in the ADR rather than left to an adapter.

**Requirements:** FS-004, MODEL-002 · **Blocks 3, hash-visible**

FS-004 requires detecting "LVM PV, Linux RAID, LUKS, BitLocker, ZFS pool
members, Storage Spaces, LDM metadata" under file-system operations, while
MODEL-002 places encryption and containers on layers distinct from file system.

Either `FileSystemKind` enumerates non-file-system signatures, breaking the
mandated layering, or FS-004 results are materialized as
`StorageContainer`/`EncryptionLayer` nodes. The answer changes both the schema
and every snapshot hash.

## SI-10 The `Snapshot` type has no defined scope

> **Resolved in spec 4.0.0 by ADR-C5** — renamed `StorageSnapshot`, covering APFS, LVM2, Apple signed system, VSS, and Btrfs. The node is envelope content, and MAC-009's signed system snapshots reach protection through a flag on the file system that carries them, which is what makes this answerable independently of SI-27.

**Requirements:** Section 5, Section 20, MAC-003, LIN-004, PART-015, FS-003 · **Blocks 3**

Section 5 requires a `Snapshot` type, but Section 20 defines only "Snapshot
(topology)", which is `TopologySnapshot`. The storage-level type is named and
never specified. Explicit requirements exist for APFS (MAC-003), LVM2 (LIN-004),
and Apple signed system snapshots. Windows VSS is implied by PART-015 naming a
VSS store as a shrink-limit cause; Btrfs snapshots are implied by nothing,
despite FS-003 and LIN-006 requiring Btrfs support.

Whether `SnapshotKind` must cover VSS and Btrfs is a specification question.

## SI-11 Is non-goal protection a type-level impossibility or a runtime guard?

> **Resolved 2026-08-09 in spec 11.0.0 by ADR-0018, on the fourth
> round.** The closure exists and is computed, total, and fail-closed:
> per-node verdicts are three-valued with an `Indeterminate` residual —
> round three's fail-open arm inverted and property-tested — computed
> from a named two-layer helper evidence contract (own enumerating
> byte-layer parsers generalizing ADR-0014's architecture, named
> state-layer APIs, a protective join), which discharges ADR-0016's
> named-contract hard input. A mutating step's affected set closes over
> destroyed substrate — downward containment range-bounded, upward
> backing, downward production — with release counted as destruction,
> so round three's root-on-ZFS-over-LUKS destruction path refuses while
> the no-sibling-capture theorem is a committed property test and
> creating a partition beside a pool member constructs. Device scope
> inverts to a closed positive local-transport list; capability status
> is computed from canonical steps by the same closure (CAP-005
> agreement by construction); source classes are never suppressed;
> PART-014 classification is exhaustive, Regime B, outside the body. A
> closed three-entry acknowledgment vocabulary (release,
> opaque-destruction, identity-bound-restore) replaces both silent
> permission and forever-refusal, with the consumed-member case
> deliberately unrepresentable. SI-29 and SI-30 are resolved within
> this decision (their entries below); SI-37 is reclassified — open,
> off the increment-3 gate, its matrix now relaxation evidence.
> **What is not demonstrated, because no write path exists, is named as
> obligations on the first write-capable increment beside the
> SI-33/SI-34/SI-35 banner obligations**: the consumed-member refusal,
> the release-acknowledgment path on the stale-tail shrink, the
> locked-layer acknowledgment path, and the
> `gpt-conflicting-tables-512` restore-only rule, each
> mutation-verified. **One residual is stated rather than rounded
> away**: a non-goal hidden inside a locked encryption layer is
> destroyed blind if the user records the opaque-destruction
> acknowledgment — opacity is physical, the guarantee's scope is
> observable topology, and the design makes the blind spot a typed,
> confirmed act rather than a silent default. The filing and round
> history below are retained as the record.

> **Axis decided 2026-08-02 in spec 4.4.0 by ADR-0012; the issue stays open.**
> A mutating step naming a client-visible Section 2.1 non-goal node is
> unrepresentable in the plan type, with the helper's recomputation retained.
> The three rejected rounds are history, not three instances of one closure
> failure: round one failed on PART-014/MAC-009 status mapping, round two on
> sibling capture plus SI-27's naming gap, and round three on the six defects
> listed in Part 6. SI-11's surviving work includes the protection construction
> those reviews exposed: total fail-closed verdicts, downward reach without sibling
> capture, device-scope inheritance, per-operation status, step-level
> construction and decode checks over structural effects, and exhaustive
> PART-014 classification. Part 6 retains the full affected-set, bind-set,
> table-extent, and host-qualified-extent review checklist. SI-29 and SI-30
> retain the Storage Spaces and sealed-volume classification cases; SI-37
> supplies the unequal-identifier multipath coverage case. SI-27 retains naming
> and edge typing; SI-34 retains verdict body placement; later recovery and
> ruleset interaction remains with SI-20 through SI-22.

**Requirements:** Section 2.1, Section 6, SAFE-005, PART-014, CAP-001, CAP-002,
CAP-003, CAP-005, HLP-002, MODEL-005, PLAN-004, CONC-001, CONC-005, PART-009,
PART-012, Section 20, Section 0.2 · **Resolved** (was: blocks 3, hash-visible)

Section 2.1 says the product MUST NOT mutate ZFS, Storage Spaces, LDM, or
Fusion — absolute. The mechanism it supplies, PART-014 protected objects, is
defined in the glossary as refusal "without an explicit supported plan", which is
bypassable by construction; and PART-014's enumerated list does not include pool
members, ZFS, Storage Spaces, or LDM at all. Section 0.2 grants override
authority to Section 3, and Section 2.1 is not in Section 3.

ADR-0012 decided that the client-visible case is **unrepresentable** rather than
merely rejected at runtime. Part 6's next-attempt list is the recorded review
checklist a next design must address in the remaining closure, construction,
residual, and status work; it did not absorb the separately owned naming,
placement, recovery, or ruleset questions.

## Filed by round three

## SI-28 A card reader's serial identifies the transport, not the medium

> **Reclassified 2026-08-09 by the decision owner: off the increment-3
> gate, the SI-37 pattern.** SI-28 stays **Mitigated-open** — not
> Resolved, and Part 7's warning against false closure stands in full.
> The interim conservative floor is in force and unchanged: destructive
> whole-device operations on removable media behind a bridge exposing no
> medium-attributable identifier are refused. SI-33's continuity witness
> (10.0.0) is landed as the refusal input on qualified apparatus, and
> the floor's relaxation route stays ADR-0017's named revisit condition,
> requiring apparatus-qualification evidence and its own round. What
> moves is only the class: the floor is computable from decided,
> contract-readable facts — transport class, removability, identifier
> presence — so no undecided hashed field is an input to it, and the
> refused population can hold no issued authorization for a later
> discriminating mechanism to invalidate. **The priced cost, accepted
> knowingly**: a future mechanism that resolves this issue by adding an
> identity-record field pays a MODEL-003 schema major after
> implementation exists — the class of cost the register's preamble
> warns has no cheap exit — accepted because the alternative was gating
> the entire domain model on a mechanism nobody can currently measure.
> This reclassification resolves nothing, relaxes nothing, and licenses
> no closure; the filing and Part 7 remain the record.

> **Confirmed on hardware, 2026-07-28.** Not a hypothesis. A USB SD reader
> enumerates two LUNs — one holding a card, one **empty** — and both report the
> same disk serial. A slot containing no medium cannot be reporting the medium's
> identity. The two LUNs differ only by the trailing `&0`/`&1` of the PnP instance
> path, which Section 16 forbids as identity, and Windows exposes no card
> register (no CID, PSN, manufacturer or OEM id) to fall back on. Measurements in
> `docs/quality/observability.md`. One round of resolution has already failed;
> see Part 7.

**Requirements:** SAFE-003, ADR-C3, ACC-014, UI-009, SEC-002 ·
**Mitigated-open, Later** (was: direct blocker, blocks 3; reclassified off
the gate 2026-08-09; hash-visible pending its own resolution)

SAFE-003 anticipates that a USB bridge or SD reader may expose *no* stable
hardware identifier, and classifies the record Weak when that happens. It does
not anticipate the inverse, which is at least as common: **a reader that exposes
its own serial.**

Many USB SD and CF readers report the reader's serial number over the mass
storage class; the card's own CID serial is not exposed. So two blank cards of
the same capacity, read through one reader, produce records that are byte-equal
in serial, WWN, total bytes, both sector sizes, connection path, and — both being
blank — partition-table state. Under ADR-C3 that record is **Strong**, because it
carries a stable hardware identifier and a positively determined table state.

Three protections then do not apply, all of which were written for exactly this
device class:

- ACC-014 and SAFE-003's weak-identity policy: typed device-name confirmation
  (UI-009) and refusal of unattended apply.
- SAFE-003's helper re-check: `IdentityMatch` **succeeds**, because every field
  the reader reports is unchanged.
- SEC-002's cross-device rejection: the two cards are one device by every
  recorded field.

A plan bound to card A therefore executes destructively against card B. No
collision check can catch it — the two cards are never simultaneously present, so
there is nothing to compare. This is not a naming problem; it is a claim about
what a serial identifies.

**Options:** (a) a stable hardware identifier counts toward Strong only when it
is attributable to the medium rather than the enclosure, which needs a normative,
per-platform rule for when that attribution is knowable; (b) removable media
behind a bridge transport are Weak regardless of reported identifiers, which is
blunt and re-imposes the friction ADR-C3 removed on genuinely strong removables;
(c) Strong requires a medium-attributable identifier *or* a positively determined
non-blank table state, so a blank card in a reader is Weak but a partitioned one
is not.

**This is a defect in an accepted decision.** ADR-C3 shipped in 3.1.0 and its
Strong definition assumes a stable hardware identifier identifies the medium.
Nothing is implemented against it yet, so the correction is still free.

**A second gap the same measurement exposed.** Two USB flash drives of one model
and identical capacity each offer *two different identifier strings from two
layers* — a storage-layer serial and a USB descriptor serial, not equal for the
same device. A plan binding one and a re-probe reading the other would not match.
So the canonicalization item in Part 6 is understated: the question is not only
how to normalize a serial, but **which** serial is the bound one, per platform
and per transport.

## SI-29 Does Storage Spaces protection cover content inside a space?

> **Resolved 2026-08-09 in spec 11.0.0, within SI-11's decision
> (ADR-0018).** The narrow reading, with a geometry line: the protected
> objects are the pool, the spaces as structural objects, and the
> member-disk substrates — not the file systems inside a space. An NTFS
> resize strictly within a space's already-provisioned block interface,
> through the platform's own documented API, is an ordinary target;
> anything changing the space's own geometry or membership is
> pool/space mutation and refuses. Two gates travel with the
> permission: mutation inside a space is `blocked` while the pool is
> degraded or a thin space's allocation headroom cannot be verified,
> and the write path is the documented API only. The broad reading was
> rejected on its measured cost — every Storage Spaces user loses NTFS
> resize for a protection Section 2.1's own words ("pools/spaces") do
> not claim. This is the narrowing that makes 11.0.0 major. The filing
> below is retained as the record.

**Requirements:** Section 2.1, WIN-003, PART-014 · **Resolved** (was: blocks 3, hash-visible)

Section 2.1 says "detect, represent, and protect pools/spaces; no pool or space
mutation". WIN-003 says Storage Spaces are "detected, represented, and protected
only". Neither says whether a *file system inside* a space is a protected object
or an ordinary target that happens to sit on unusual backing.

The narrow reading protects the pool and the space objects while permitting an
NTFS resize inside a space; the broad reading protects everything above the pool.
The two produce different capability answers for the same volume on the same
machine, and the choice feeds the protection verdict, so it is hash-visible and
not migratable later.

## SI-30 Does "never modified" for the sealed system volume cover deletion?

> **Resolved 2026-08-09 in spec 11.0.0, within SI-11's decision
> (ADR-0018).** Deletion-by-containing-erase is severed from
> modification of the sealed object. The sealed volume and signed
> system snapshots, as direct targets, are refused for every mutating
> class, in every environment, with no acknowledgment route — "never
> modified," absolute. A whole-container or whole-device destructive
> step that reaches them only through substrate destruction is governed
> by Section 2.1's documented-supported-paths clause and MAC-009's
> Recovery rule, through a named, closed step family that is **empty in
> v1** — today every such erase refuses through the closure like any
> other reached non-goal and reports `unsupported` as any unimplemented
> operation does, and implementing Apple's documented path someday is a
> Regime C matter, not a closure amendment. This resolves the axis
> round one and round three each froze by accident, in opposite
> directions: neither `unsupported`-everywhere (round one's error,
> which MAC-009's `blocked` text contradicts) nor an acknowledgment
> route (the sealed object has none). The filing below is retained as
> the record.

**Requirements:** Section 2.1, MAC-009, PART-014, CAP-003 · **Resolved** (was: blocks 3)

Section 2.1 requires that Apple sealed system volumes and signed system snapshots
are "never modified" and limits boot-volume work to documented supported paths.
MAC-009 makes them protected objects and requires `blocked` with a stated reason
for operations macOS permits only in Recovery.

Whole-object deletion is neither, and the requirements point opposite ways.
Erasing `Macintosh HD` is not a modification *of* the sealed volume, which argues
for treating it as an ordinary destructive operation gated by MAC-009's Recovery
rule (`blocked` + a reason). But Section 2.1's non-goal argues that the product
never does it at all (`unsupported`). Round three's design routed it to
`unsupported` in two places and recommended `blocked` in a third, and its
verification plan hardcoded the first — freezing the answer to MAC-009's
most-cited target by accident.

This is the same axis round one was rejected on, so it is filed rather than
picked.

## SI-31 `pce/1` specifies no ordering for array and set elements

> **Resolved in spec 4.1.0 by ADR-C6.** The rule is schema-level, sets only:
> unsigned lexicographic comparison of each element's full canonical bytes,
> with strict duplicate rejection and inherited enclosing depth. Semantic
> arrays and the `pce/1` profile are unchanged.

**Requirements:** MODEL-005, MODEL-006, ADR-C1, ADR-C6,
`schemas/domain/canonical-collections.md` · **Resolved; formerly Blocks 3,
hash-visible**

Filed against `schemas/canonical-encoding.md` rather than against two conflicting
requirements, because the defect is an omission in a normative document.

§3 fixes map key ordering as length-first then bytewise, and notes that this is
"equivalently, and identically in result" a plain bytewise comparison of the
fully encoded key. That equivalence holds for **text keys**, whose heads are
monotonic in length. It does not generalize, and the profile never states an
ordering for array elements.

Part 3 of this document already agreed that set ordering must be over each
element's fully canonical encoding. Every set-valued field in the domain model —
partitions, free extents, backings, observations — therefore depends on a
comparator the profile does not define, and the two candidates disagree on
ordinary values:

```
{len: 5, off: 300}  -> a2636c656e05636f666619012c   (13 bytes)
{len: 6, off:   0}  -> a2636c656e06636f666600       (11 bytes)
```

Length-first orders the second first; plain bytewise orders the first first.
Rust's `Vec<u8>` ordering is plain bytewise and the existing `compare_keys` /
`compareKeys` helpers are length-first over text, so the two implementations
would each reach for a different one and MODEL-005 parity would fail on the first
extent set — silently, since both produce valid canonical encodings of the same
logical value.

**Recommended answer, on a corrected ground: plain bytewise over each element's
fully canonical encoded bytes.** The answer survived adversarial review; the
reasoning first offered for it did not, and the scope, the document, and the
proposed evidence were all wrong. Recorded in full because the corrections are
what make this safe to land.

**The original derivation was false.** It claimed §3 *is already* plain bytewise
over encoded bytes, so a general rule would merely restate it. Two comparators
restrict to §3 on text keys, not one — plain bytewise over encoded, **and**
length-first over encoded, since total encoded length is strictly increasing in
payload length. §3 cannot select between them. Worse, §3 states its own
provenance as RFC 8949 §4.2.3 length-first, "not the bytewise ordering of
§4.2.1", and says "the choice is deliberate". Plain bytewise for elements is
§4.2.1, so this **creates** a second convention rather than avoiding one. An
experiment showing length-first-over-string agrees with bytewise-over-encoded on
3,721 text pairs demonstrated that one candidate matches §3, not that only one
does.

**The ground that does hold.** A set element's order carries no semantic content,
so the tiebreak is implementability. Rust's `<[u8] as Ord>::cmp` is plain
unsigned bytewise, so `elements.sort()` is correct by default while length-first
would be a silent wrong answer in the language most likely to reach for the
default. Concretely: the committed `nested containers` vector's array elements
are `a16179f6` and `f5`, already bytewise-ascending, so **no committed digest
moves under plain bytewise — one would move under length-first.**

**Scope: sets only, never arrays.** The filed title said "array and set
elements", and that is too broad. Constraining arrays narrows §1's Array range
("any sequence of values"), which §7 makes a new profile version rather than an
in-place clarification — §7's clarification clause covers only input the profile
already declares invalid, and `decode` **accepts** a descending two-element array
today in both languages. It would also break two committed fuzz targets:
`roundtrip_value.rs` asserts `encode` refuses only with `DepthLimitExceeded` or
`NegativeOutOfRange`, and asserts `decode(encode(v)) == v`.

**Document: not this one.** If the rule binds sets only — which is right — it
cannot be a `pce/1` rule, because **`pce/1` has no Set kind**. The descending
bytes decode to a plain `Array` with no discriminant and no ordering check, and
no variant of the codec error type applies. It belongs to the per-schema
validation pass that Part 6 item 7 already names as the sole decode boundary,
with its own error type.

**The proposed evidence would not have caught this.** Both fixture loaders build
arrays in file order and never sort, so a golden vector asserts only "these bytes
encode this given sequence" — which both comparators reproduce. SI-31 would have
been closed with evidence blind to SI-31. The test has to exercise the sort.

**Two prerequisite defects in the encoder, found by the same review.**

1. `encode` was not injective in TypeScript: `TextEncoder` substitutes U+FFFD for
   an unpaired surrogate, so two distinct values produced identical bytes and the
   encoder could emit a map declaring two byte-identical keys — bytes its own
   decoder rejects, violating §6.1. Reproduced, then fixed; Rust was unaffected
   because `String` is validated UTF-8, which means the two implementations had
   disagreed about what was *encodable*.
2. **Encoding an element resets the depth budget.** The only exported byte
   producer in either language starts at depth 0, so a 100-deep element encodes
   standalone in both while the spliced 200-deep result is rejected by both
   decoders. Same §6.1 class, unfixed, and it becomes reachable exactly when
   set-valued fields exist. Whatever lands this rule must state how per-element
   encoding accounts for depth.

## SI-32 The glossary's weak-identity definition contradicts SAFE-003

> **Corrected in spec 4.0.0.** The glossary now defines weak identity as any record that is not Strong under SAFE-003, which covers an indeterminate table state as well as an absent identifier. ACC-014 gained a note in the same pass, recording that it exercises only the *absent*-identifier case.

**Requirements:** Section 20, SAFE-003, ADR-C3 · **Editorial, does not block**

Section 20 defines weak identity as "device identity lacking any stable hardware
identifier (SAFE-003)". Since 3.1.0, SAFE-003 also classifies as Weak a record
whose partition-table state could not be determined, **even when a serial or WWN
is present** — that distinction is the whole point of ADR-C3's three-valued table
state, and the ADR calls it a deliberate tightening.

Read as written, the glossary says a device with a serial and an unreadable GPT
is not weak-identity, which is precisely the case ADR-C3 added.

The amendment edited the requirement and not the definition that restates it.
Recorded rather than fixed in place because it is normative text: the fix is one
line and belongs in the next spec change, whose version bump is already open (see
ADR-C5). Round four found that ACC-014 and ACC-007 need amendment in the same
pass, so fold all three into one spec change rather than three.

## SI-34 Should the derived protection verdict be frozen into the hashed body?

> **Resolved 2026-08-09 in spec 9.0.0 by ADR-0016.** Yes — and the
> premise that made yes dangerous is gone. The verdict is hashed-body
> content, **helper-authored at validation** from a named evidence
> contract, recomputed at revalidation and before first write, any
> within-target divergence rejecting under SAFE-003/PLAN-006's existing
> rules. The filed options all bridged a two-observer world ADR-0014
> removed: no client authors any bindable artifact, so option (c)'s
> freshness projection and monotone floor — and its two open
> dependencies, projection membership and the monotonicity proof —
> dissolve with the second author, while (c)'s point survives by
> construction: a client cannot weaken the safety decision, because no
> client claim is representable. The evidence clause is discharged at
> its honest scope: the both-views stale-signature comparison is
> measured on real Linux (L10, both directions, double-capture-stable)
> and on macOS (M7 client-blind; M10 helper-distinct — the currency
> note below predated M10's taking and its "cannot be measured until
> M10 exists" is corrected by that taking); the client-permitted-loses
> contest dissolves unrepresentable. **What is not demonstrated,
> because no write path exists to demonstrate it on, is named here as
> obligations on the first write-capable increment, beside SI-35's:** a
> helper-only fact that changes protection rejects before the first
> write with a structured divergence, and out-of-target evidence blocks
> nothing — both on the stale-signature fixture family,
> mutation-verified. The verdict's internal shape stays SI-11's, with
> ADR-0016's named-contract requirement as a hard input; naming stays
> SI-27's. The filing and its history below are retained as the record.

**Requirements:** MODEL-005, PLAN-006, HLP-002, CAP-007, SAFE-005, Section 0.2 · **Resolved** (was: blocks 3, hash-visible)

Filed after a project review, and after the measurement that reopened it. This
issue exists because the answer was previously treated as settled using a
universal justification that one named finite projection has since refuted.

**What is not in question.** Protection is derived from discovered evidence and
the graph, and the helper recomputes it independently. A client cannot declare an
object safe. HLP-002 and CAP-007 already require this and no round has disputed
it.

**What is.** Round two justified freezing the helper's exact derived verdict into
the hashed body by concluding that every client/helper asymmetry is a
roster-identity fact. Within the retained regular-file projection, WP-020's
stale-signature fixture refutes that universal premise: the single-answer
interface reports only the stale mdraid signature while the enumerating
interface reports both it and the live ext4 signature. That finite observation
asymmetry is not roster identity. The fixture does not by itself establish that
a real client and helper produce different complete graphs or final protection
verdicts. If a qualified implementation later establishes such a graph
difference while the verdict is body content, unchanged hardware can produce
different body hashes — the PLAN-006 unsatisfiability ADR-C2 exists to prevent.

**Options:**

(a) **Keep freezing.** Then the projection must be clamped so both sides compute
identical node sets, which means the helper derives a safety verdict from a
deliberately impoverished view while its own probe sees more. A verdict that
ignores evidence it holds.

(b) **Drop it.** The helper computes protection from the best evidence available
and it is not body content. Divergence resolves silently in the helper's favour,
and the user may have authorized a plan computed under a different view than the
one executed.

(c) **Freeze a projection and a floor.** Authenticate a normative
cross-privilege *freshness projection* — only facts both sides are proven able to
reproduce — plus a coarse monotone safety floor ordered
`permitted < indeterminate < refused`. The helper recomputes the exact verdict
from all live evidence and may only keep or tighten the floor, never loosen it.
Any tightening that changes permission, affected objects, risk, or consequence
text rejects before the first write and requires a new reviewed plan; extra
evidence that changes none of those is journaled and execution continues.

**Option (c) is the recommendation of the project review** and is the only one
that keeps the useful half of freezing — a client cannot weaken the safety
decision — without either blinding the helper or tolerating silent divergence in
what was authorized.

**Do not record (c) as decided.** It has had no adversarial round, and the two
things it depends on are unresolved: which facts belong in the freshness
projection is exactly the unfinished observability work, and the monotonicity
claim needs a proof that extra evidence can never make a verdict *less*
restrictive.

**It does not dissolve SI-27.** ADR-C5 puts technology, membership, and signature
facts in the body independently of any verdict, so those edges still need stable
document-local node identifiers. Moving the verdict out reduces SI-27's
protection-specific burden and no more. An earlier claim in this project that
SI-27 "largely dissolves" under (c) was too strong.

**Evidence required before any option is accepted:** the stale-signature fixture
must show the freshness projection comparing equal across both views while the
helper retains both signatures; a helper-only fact that changes protection must
reject before the first write with a structured divergence; a helper-only fact
that changes nothing must proceed and be journaled; a client claiming
`permitted` must lose to the helper's `refused`; and Windows, real partitioned
Linux hardware, and macOS observability must all be established first.

> **Status of the observability element, 2026-08-05: satisfied. Every other
> element of the clause above remains unsatisfied.** All three named platforms
> now have records — Windows, real-partitioned Linux from the 2026-08-04
> matrix, and macOS from the 2026-08-05 increment 6 matrix, valid on its
> second sitting.
>
> **The reading this rests on, stated so it can be attacked.** Round four's
> precondition 1 defines the thing being demanded as "a per-platform
> observability record, established empirically and **non-elevated**". The
> privileged comparison leg therefore falls outside what "observability
> established" asks for, and macOS's untaken M10 does not hold this element
> open. **The narrower reading — that "macOS observability" means the whole
> matrix including its privileged leg — is recorded as rejected rather than
> ignored**, because it is not unreasonable: it would leave this element open
> until a disposable macOS VM exists. If that reading is preferred, this
> element reverts to unsatisfied and nothing else in this entry changes.
>
> **What is missing on macOS regardless of which reading wins.** M10 is
> `not yet taken` for want of a disposable macOS VM, so the macOS record has
> **no privileged comparison leg at all**. The first element of the clause
> above — the stale-signature fixture showing the freshness projection
> comparing equal **across both views** while the helper retains both
> signatures — is therefore **unmeasured on macOS**, and cannot be measured
> there until M10 exists. What macOS did establish is one side of it: the
> client sees nothing whatever for that fixture, its projection being
> byte-identical to a blank disk. M9 is `not established` because Apple
> Silicon has no Fusion Drive. The macOS second-reader readback was
> discharged 2026-08-08 by an independent reader session, every digest
> matching; the discharge and the custody caveats it carries are recorded
> in `docs/quality/observability.md`.
>
> This note records currency. It does not resolve SI-34, discharge its
> evidence clause, decide or rank an option, or move its state.

## SI-35 The measured client projections do not separate ADR-C3's three table states

> **Resolved 2026-08-09 in spec 8.0.0, by ADR-0014's axis carried to its
> instrument.** The privileged helper is the sole author of ADR-C3's
> partition-table state, from its own raw-sector parser
> (`crates/table-parser`) — the only contract the completed campaign
> found separating; the client emits no table state on any platform,
> which resolves the `Present` face SI-39 parked here by construction
> (no report, no forbidden report); the state lives in the hashed body,
> helper-stamped at validation, ADR-C4's guard satisfied unamended; and
> `Present`'s checksum is fixed over copy-invariant content per
> `schemas/table-checksum.md`, closing the basis question open since
> round one. The evidence clause's refusal demonstration is discharged
> **at its honest scope**: the parser classifies the decisive
> `gpt-conflicting-tables-512` as `Indeterminate`-ambiguous and
> `gpt-both-copies-invalid-512` as `Indeterminate`-unreadable, both
> mutation-verified, with claimed-never-`Absent` a searched fuzz
> property — and what is **not** demonstrated, because it cannot exist
> before increment 3, is an end-to-end refusal by a running write path.
> **That re-demonstration is a named obligation on the first
> write-capable increment**, in this banner rather than a review memory:
> the increment that first wires a write path must show
> `gpt-conflicting-tables-512` refused through SAFE-005 and PART-001's
> categorical invariant, end to end, before any destructive capability
> is represented as working. The filing and its evidence history below
> are retained as the record.

**Requirements:** ADR-C3, MODEL-005, PLAN-006, INV-003, SAFE-005, HLP-002 ·
**Resolved** (was: blocks 3, hash-visible)

> **Evidence status 2026-08-04: two of three acceptance categories are
> satisfied; the third is blocked on a decision.** The loop category is
> discharged by the descriptor-bound non-WSL run of 2026-08-03, valid on its
> third sitting after two void sittings and their recorded instrument
> amendments; it confirms the historical WSL2 negative on qualifying ground,
> and issue #94's non-qualification no longer applies. The Windows category is
> discharged by the completion rerun of 2026-08-04, which repaired the original
> procedure's omitted layout rows and discarded status surfaces, made all three
> declared refutation conditions evaluable, refuted them, and answered the
> hybrid question. A refusal proof for the chosen option still cannot exist
> until an option and implementation exist, and no option has been chosen.
> The finite retained projections did not separate the decisive pair; that does
> not prove that no client-readable interface can do so.
>
> **Axis update, 2026-08-08: the option is chosen.** ADR-0014 fixes the
> axis — helper as sole author of table state, from its own raw-sector
> parser, stamped into the hashed body at validation; the client emits no
> state — and the refusal-demonstration category is therefore open rather
> than blocked: it closes when the parser lands and demonstrates
> `Indeterminate`-and-refuse on `gpt-conflicting-tables-512`. The issue
> stays Open until then, per the ADR-0012 axis-decided shape.

Filed 2026-07-28 from a measurement, not from a reading. Part 6's precondition 1
already requires an ADR-C3 amendment fixing **what `Present { checksum }` is
computed over**. The measurements constrain that choice without deciding it.

**What the retained measurements establish.** On regular files under
libblkid 2.41, `gpt-basic-512.img` and
`gpt-conflicting-tables-512.img` produced byte-identical output from both
`blkid -p -o udev` and `wipefs -n`. The first has two agreeing tables; the
second has two independently valid tables describing different partitions.
That finite projection therefore did not encode ADR-C3's distinction between
`Present` and `Indeterminate`. `ID_FS_AMBIVALENT` did not fire. The full table
and the later runs' validity limits are in `docs/quality/observability.md`.

The same file probe labelled a damaged-primary image as `gpt`; only the retained
`wipefs` offsets exposed the missing primary signature. That does not establish
silent backup recovery: the record cannot distinguish use of the valid backup
from parsing primary bytes without validating the CRC. Windows likewise
retained the same two partition rows as the healthy fixture, but no fact
identifying which copy it used or whether it validated the primary CRC. The
hybrid image's regular-file projection was also plain `gpt`.

**The conflict.** ADR-C3 requires three partition-table states, INV-003 requires
detecting inconsistent and hybrid tables, and the future design must say which
observer computes which state from which contract. The named regular-file
projection collapses a decisive pair. The **2026-08-02** Windows run did not
produce a decision-complete counterexample because its wrapper skipped
reachable layout queries for two enumeration-gap fixtures and discarded queried
`MSFT_PhysicalDisk` properties, and the historical loop projection could not
repair either gap because its attach was not descriptor-bound and its
normalizer was changed after the first result. Both defects have since been
repaired by successor runs — the descriptor-bound non-WSL loop sitting of
2026-08-03 and the Windows completion rerun of 2026-08-04 — and **neither
successor separates the decisive pair either**. That is what makes the
conflict above a design question rather than a measurement gap: the
enumerated projections are now completely retained — which is what the two
reruns fixed — and they still do not encode ADR-C3's distinction. So the
future design must say which observer computes which state from which
contract, and what it does when the contract is silent. "Completely
retained" is not "exhaustive": it means each run kept every surface it
queried, not that every client-readable interface was queried.

Raw-sector computation also has a privilege boundary: the retained
non-elevated Windows and Linux environments denied direct device reads. That is
evidence about those environments, not a universal statement that every
client-readable interface lacks a separating fact. Consequently neither raw
sectors nor an unspecified "kernel view" can be selected without naming the
platform contract and the fail-closed behavior when it is absent.

**Options, none decided:**

(a) **Privilege-tagged state.** `Present`/`Absent`/`Indeterminate` gains a
recorded observation basis, so "indeterminate *from here*" is distinct from
"indeterminate". Costs: the basis is body content and hash-visible, and two
observers at different privilege may still produce different bodies — the
PLAN-006 problem ADR-C2 exists to prevent, and the same placement problem SI-34
is open on.

(b) **Clamp to a named reproducible client projection.** Both sides compute
over a precisely specified projection a non-elevated client can reproduce, and
`Indeterminate` is reachable only through facts that projection carries. This
requires positive evidence that the selected contract separates every state in
scope, plus a fail-closed answer where it does not. Equality in one finite
libblkid projection neither supplies that contract nor refutes the existence of
another client-readable fact.

(c) **`Indeterminate` becomes helper-only.** The client never claims the state;
the helper, which can read sectors, computes it and may only tighten. This is
SI-34's option (c) applied to a second field and inherits SI-34's unproven
monotonicity and placement obligations.

**Bearing on SI-34.** SI-34 was filed on a signature-layer asymmetry. The
partition-table measurements provide a separate finite projection in which
decisive bytes are not reflected in the retained client view. That is enough to
reject an assumed universal client/helper symmetry; it is not enough to claim
that every partition-table interface is asymmetric or that the resulting
protection verdicts differ.

**What this does not establish.** The regular-file probe is not a kernel parse.
The 2026-08-02 WSL2 loop run is historical and non-qualifying, so it
establishes neither a positive separating contract nor a universal negative,
and the 2026-08-02 Windows procedure ran on attached fixtures but was
incomplete for its pre-registered hypotheses. Both reruns have since been
taken and are valid — and **neither separates the decisive pair either**.
What is still not established is any positive separating contract: the
completed runs are negatives over their enumerated projections, and a
negative over an enumerated projection is not a proof that no
client-readable interface separates these states.

**Evidence required before any option is accepted:** (1) the descriptor-bound,
non-WSL loop-device measurement, so the file-probing limitation is not mistaken
for a kernel limitation; (2) the decision-complete Windows equivalent,
including every reachable layout and retained status surface; and (3) a
demonstration that the chosen option refuses rather than proceeds on
`gpt-conflicting-tables-512.img`.

**(1) and (2) are satisfied as of 2026-08-04. (3) is not, and cannot be until
an option is chosen.** (1) is discharged by the 2026-08-03 descriptor-bound
non-WSL sitting, valid on its third attempt. (2) is discharged by the
2026-08-04 Windows completion rerun, whose three added gates cover this
clause's two named requirements directly: total retention of every queried
property value, and a mandatory index-fallback layout probe at every
`Win32_DiskDrive`-supplied index for any fixture without an `MSFT_Disk` row.
One declared limit survives it — W-H2's which-GPT-copy question remains
unmeasured, because both copies of that fixture describe identical partitions
and the discriminating fixture variant belongs to WP-020's catalogue. That
limit bounds the finding; it does not reopen the category.

## SI-33 A continuity witness for media that cannot be told apart

> **Resolved 2026-08-09 in spec 10.0.0 by ADR-0017.** The witness exists
> and is a **refusal input, never an assurance**: an epoch-token/counter
> field of SAFE-003's identity record — client-readable,
> helper-verified, deliberately not an authoring-set entry — scoped to
> exchange-capable targets on qualified apparatus (one today, the
> measured Windows counter with its PDO epoch token, per the reach
> pattern). Comparable only within an unchanged epoch and never on a
> decrease; movement or incomparability rejects covered targets under
> the existing identity-change rule; `no-exchange-observed` — the
> liveness ceiling's own words — relaxes nothing, so staleness on
> unmeasured hardware costs only assurance that was never claimed, the
> fail-closed inversion of the evaluable-but-stale trap this filing
> named. The S4-measured vector — swap between plan and apply on media
> whose every identifier is identical — becomes a refusal where the
> apparatus is qualified. **SI-28's floor and Mitigated-open state are
> untouched**; the relaxation route this issue was filed hoping for is
> ADR-0017's named revisit condition, requiring apparatus-qualification
> evidence and its own round. Placement resolved to the body with the
> record it protects, making this issue hash-visible after all — its
> row is corrected with this resolution. Write-path demonstrations are
> named obligations on the first write-capable increment, beside
> SI-35's and SI-34's. The filing and its liveness record below are
> retained as the record.

**Requirements:** SAFE-003, PLAN-006, HLP-004, ADR-C2, ADR-C3 ·
**Resolved** (was: blocks 3; hash-visible via the placement this
resolution decided)

Filed by round four of SI-28, which established that SI-28 cannot be resolved by
classification alone (Part 7). This is the only mechanism anyone has proposed
that discriminates two media whose recorded identity fields are equal.

The idea: bind the plan not only to *what the target reports* but to a witness
that **the medium was never exchanged** between plan creation and apply. It names
nothing and identifies nothing; it witnesses non-interruption. Windows exposes a
media-change counter through `IOCTL_STORAGE_CHECK_VERIFY2`, and comparable
signals exist elsewhere.

**Do not file this as solved by the counter's existence.** The variant reachable
on a zero-access non-elevated handle may return a value the class driver already
holds, and a witness that is *evaluable but stale* fails open in precisely the
vector it exists for — plan, swap, apply, seconds apart, within one attach
session — while the plan carries a field implying the check was made. That is
worse than no witness, because it converts an admitted gap into a false
assurance.

**Liveness is a precondition on any design, not a detail.** Read the counter,
exchange the medium with no intervening I/O, re-read immediately, and assert the
value moved; repeat with a sixty-second idle gap to detect poll-driven behaviour;
then close and reopen the handle and assert it survives. Until that passes on
real hardware, this is a hypothesis.

> **The liveness precondition is discharged, decided 2026-08-05.** It passed on
> real hardware in the 2026-08-04 sittings: the immediate re-read and the
> sixty-second idle gap both moved, and the close-before-event/reopen arm
> survived in three of three trials across true no-handle windows.
> **SI-33 is therefore no longer a hypothesis, and stays Open.** What the pass
> discharges is the precondition, not the issue.
>
> **Three limits the protocol declared before any data existed, recorded here
> because a reader of the sentence above will otherwise assume they are gone.**
>
> - **The positive cannot be attributed to exchange-synchronous detection.**
>   Prompt movement is equally consistent with a background poll. The strongest
>   recordable positive is *"no staleness observed under these conditions"*.
> - **It is bounded.** Slot-exchange family only, on one reader, one bridge,
>   one build. It generalizes to nothing else by itself.
> - **The exposed reading is not globally monotone.** One run read a value
>   *below* one an earlier run had already passed, across a boundary containing
>   a timestamped PnP arrival. That measured decrease makes an **equality-only
>   witness unsafe**, so a design must characterize the counter's epoch rather
>   than compare values — or use another witness entirely.
>
> **What this does not do.** It decides no axis, no design, and no
> body-versus-apply placement — the placement question below stays open. And it
> **does not relax SI-28's interim conservative floor**: SI-33 is the route by
> which that floor may later be relaxed, and the route is the design, not the
> liveness pass.

Placement is also open: a witness is compared rather than re-derived, so ADR-C2's
rule argues for the body, but a witness that changes on every attach makes
PLAN-006 unsatisfiable if hashed naively.

## SI-37 The multipath refusal has no fail-closed rule for unassembled paths with unequal identifiers

> **Reclassified 2026-08-09 in spec 11.0.0 by ADR-0018: open, no longer
> gating increment 3.** The fail-closed design home this filing asked
> of SI-11's round exists: the device-scope transport arm is a closed
> positive local-transport list (everything else `Indeterminate` or
> refused as remote), and the positive-local population carries
> per-transport path-multiplicity contracts — NVMe's own
> shared-capability report and subsystem grouping, SAS/SCSI
> device-reported WWN equality firing ADR-0011's existing ambiguity
> rule, SATA/USB/SD point-to-point by transport construction — with an
> `unavailable` answer, or reported multi-path capability without a
> platform-assembled node, `blocked`. The filed population is therefore
> typed and fail-closed, which is what lets increment 3 write the type.
> **SI-37 is deliberately not resolved**: its own evidence clause
> requires the per-platform dual-path matrix and negative controls
> before any option is accepted, and no such measurement exists. The
> matrix becomes the acceptance evidence for any future arm moving a
> closure-blocked multipath-capable population to `Permitted` — the
> SI-28-floor pattern applied to multipath. Its class moves to Later,
> pinned to the spec change that would first relax those populations.
> The filing below is retained as the record, and its evidence clause
> is unchanged.

**Requirements:** Section 2.1, ADR-0011, ADR-0012, SAFE-003, SAFE-005,
MODEL-003, MODEL-004, MODEL-005, PLAN-006, INV-001, INV-008, CAP-001,
CAP-003, HLP-002, LIN-006 · **Open, Later** (was: input to SI-11, gating 3
through SI-11; reclassified in 11.0.0, hash-visible pending its own
resolution)

Filed 2026-08-02 by the post-acceptance integrity review of ADR-0011.

> **This does not reopen SI-12 or withdraw ADR-0011.** ADR-0011's accepted
> answers remain in force: a platform-recognized multipath node and its
> recognized members are detection-only and `unsupported` as write targets;
> an unassembled pair presenting byte-equal stable identifiers is SAFE-005
> ambiguity and `blocked`; the product does not infer cross-path sameness of
> its own; and supported multipath mutation remains deferred. This filing gives
> the residual that ADR-0011 recorded, but did not assign to an open register
> item, a fail-closed design home.

**What is established, and what is not.** The observability record establishes
that one physical device can expose different identifier strings through
different layers and that bridges can synthesize identifiers. It therefore
refutes an assumption that byte equality across every relevant observation
layer is automatic. No retained measurement establishes that one real
multipathed LUN presents unequal identifiers on two simultaneous paths. The
population below is an admissible, safety-relevant counterexample already named
by ADR-0011, not a measured prevalence claim. Its existence on supported hosts
must be measured; its absence must not be assumed.

**The exact conflict.** Section 2.1 requires the product to detect, correctly
represent, and protect every multipathed attachment and never mutate it.
ADR-0011 specifies two enforceable cases:

1. the platform assembles a multipath node and reports its member paths; or
2. no node is assembled, but two devices present byte-equal stable identifiers,
   making both SAFE-005 ambiguity.

Neither rule covers this permitted observation:

- the host exposes nodes `P` and `Q` for two paths to one LUN;
- no platform multipath framework supplies an assembled node or membership
  relation;
- the stable-identifier bytes selected for `P` and `Q` differ because the
  paths, HBAs, bridges, layers, or representations transform them differently;
- each record independently carries a stable identifier, size, both sector
  sizes, and a positively determined partition-table state.

Under SAFE-003, strength is a property of one record, so both records can be
`Strong`. Their identifier bytes are unequal, so ADR-0011's equal-identifier
ambiguity rule does not fire. No platform membership relation exists, and
ADR-0011 correctly forbids the product from inventing a same-device claim.
SAFE-005 supplies the required result once identity is known to be ambiguous,
but no requirement supplies the predicate that makes this pair ambiguous.

The client can therefore classify `P` and `Q` as ordinary mutable disks. The
ADR-0012 constructor then has no protected node to make unrepresentable.
HLP-002 does not close the gap by itself: independent rediscovery can reproduce
the same classification without a bug, and no normative helper-only rule
supplies the missing membership fact. A mutating capability may consequently
remain `supported` while the physical target belongs to Section 2.1's absolute
non-goal.

This is a coverage failure in the protection classifier, not a reversal of the
detection-only policy. Treating unequal identifier bytes as proof that the
devices are distinct would turn missing evidence into permission. Treating
similar model, size, or path text as proof of sameness would violate ADR-0011.
Neither is an available default.

**Why this gates increment 3 and is conservatively hash-visible.** SI-11 owns
the closure that decides which nodes Section 2.1 reaches; this case must be one
of its explicit inputs. Option (c), or any answer placing a durable membership
fact in the topology, would change the snapshot body and canonical bytes under
MODEL-005. Options that keep the fact helper-derived may avoid that change;
choosing among those placements before increment 3 is why the issue is marked
hash-visible rather than silently settled by implementing no field.

**Options, none decided:**

(a) **Require a platform-authoritative membership or single-path assertion.**
For each supported platform and transport, name an API and value whose contract
either reports platform-owned membership or positively rules out another path
to the same target. Unavailable, conflicting, or non-authoritative answers are
`blocked`. Cost: the supported write population is bounded by what each
platform can positively attest.

(b) **Fail closed over an explicitly bounded host/transport population.** Where
multiple paths are possible and no authoritative framework proves membership
or single-path status, every affected mutating capability is `blocked` with a
remediation such as enabling the platform multipath framework or removing the
redundant path. Cost: potentially broad refusal, including legitimate distinct
disks. The population must be defined from observable properties; "SAN-like"
or "might be multipath" is not an implementable predicate.

(c) **Represent membership uncertainty explicitly.** Add a closed state such as
`recognized`, `ruled-out`, or `indeterminate`, backed by typed observations;
only `ruled-out` permits ordinary mutation, while `recognized` is
`unsupported` and `indeterminate` is `blocked`. Cost: likely Section 5 and
canonical-schema additions, a MODEL-003 versioned surface, a body-versus-
envelope decision under MODEL-005/HLP-002, and coordination with SI-27 if a new
node or relation must be named. `Indeterminate` records uncertainty; it must not
assert that two paths are the same device.

Not an option: allowing mutation merely because identifiers differ, relying on
an unspecified helper check, or deduplicating by model, capacity, connection
text, or another heuristic.

**Evidence required before any option is accepted:**

1. A per-platform matrix using one LUN exposed through at least two real or
   faithfully virtualized paths, measured with the native multipath framework
   both assembled and deliberately absent or disabled. Record raw identifier
   bytes by source API and layer, the platform membership relation, and the
   privilege needed to read each fact.
2. Distinct-LUN negative controls matched as closely as practical in model,
   capacity, sector geometry, controller, and transport, so the availability
   cost of a conservative rule is measured rather than hidden.
3. Repeated probes across path addition and removal, enumeration-order changes,
   framework restart, and host reboot. Any proposed body field must be stable on
   unchanged hardware and satisfy PLAN-006.
4. Separate unprivileged-client and privileged-helper projections. If only the
   helper sees the decisive fact, the design must state how the client remains
   conservative and how the helper tightens without claiming type-level
   enforcement of an invisible fact.
5. A documented positive contract for any proposed single-path assertion.
   Absence of an assembled node, a second enumerated path, or equal identifier
   bytes is not negative proof.
6. If a new field or state is selected, exact body/envelope placement,
   provenance, canonicalization, shared Rust/TypeScript vectors, and schema
   versioning evidence.
7. Fixture-backed tests proving that a client-visible decisive fact yields
   neither a mutating `supported` capability nor a constructible mutating step;
   that a helper-only decisive fact causes every client-constructed or
   hand-forged artifact to be refused before the first write; and that distinct-
   LUN controls retain exactly the availability the accepted option promises.

**Dependencies.** SI-11 consumes this issue; its round-four closure may resolve
SI-37 only by naming this case explicitly and carrying the required evidence.
SI-27 does not decide sameness: its equal-identifier collision and membership-
edge naming work remain, and it consumes a future SI-37 node or relation only
after that mechanism is accepted. SI-12 remains Resolved; SI-37 concerns proof
that a target belongs to its protected population, not whether recognized
multipath should become writable.

## SI-38 INV-003 requires the unprivileged discovery layer to detect what it measurably cannot

**Requirements:** INV-003, SAFE-002, HLP-002, HLP-005, SAFE-005, ADR-C3,
MODEL-005, Section 0.2 · **Resolved in spec 6.0.0 by ADR-0013**

> **Resolved 2026-08-05 in spec 6.0.0 by ADR-0013.** INV-003's detection duty
> is scoped by privilege: unprivileged discovery detects every state its
> platform contract can distinguish and may not report one it cannot reach;
> the privileged path owes the full set; and the unprivileged layer MUST
> publish the reach of its platform contract, per-platform and independently
> of any device. A consumer may not read an unprivileged inventory as evidence
> that an undeclared state is absent — the privileged re-discovery HLP-002
> already requires before the first write determines it, and the unprivileged
> layer neither refuses on the ground of its own blindness nor represents that
> blindness as a determination.
>
> **SAFE-002 is untouched.** Qualifying it was rejected on precedence: it is a
> Section 3 constraint and bending it to satisfy a Section 7 functional
> requirement inverts Section 0.2's ordering. The obvious alternative — detect
> what you can and report the rest as undetermined — was rejected as
> unimplementable, because the client cannot identify the remainder.
>
> This narrows an existing MUST, which is why it is a major bump. **SI-35 is
> unblocked and remains a direct blocker.**

Filed 2026-08-04 from a measurement and an adversarial review, not from a
reading. Section 0.2 requires this filing rather than permitting it: "If two
requirements in this spec conflict, agents MUST stop, file a spec issue
describing the conflict, and not silently pick a side."

**The two requirements, quoted.**

> **INV-003:** Detect GPT, MBR, Apple Partition Map, missing tables,
> hybrid/inconsistent tables, and corrupt metadata.

> **SAFE-002:** The GUI, CLI, discovery layer, and default test suites MUST run
> without elevation.

INV-003 lives in Section 7.1, Inventory and topology, beside INV-001, INV-002
and INV-004 — the discovery layer's own duties. SAFE-002 places that layer at
no elevation.

**What makes this concrete rather than theoretical.** Two decision-complete
measurements, recorded in `docs/quality/observability.md`:

- **Linux, 2026-08-03.** A descriptor-bound loop device in a disposable non-WSL
  VM with partitions materialized. The frozen client projection was
  byte-identical between `gpt-basic-512` and `gpt-conflicting-tables-512` in
  every valid trial.
- **Windows, 2026-08-04.** The completion rerun, under gates requiring total
  retention of every queried property value and a mandatory layout probe at
  every `Win32_DiskDrive` index for a fixture absent from `MSFT_Disk`. All
  three declared refutation conditions became evaluable and all three were
  refuted. W-Q4 additionally found that no scheme is reported for the hybrid
  fixture or its MBR control, and **nothing flags the aliasing**.

So both of INV-003's clauses that this evidence reaches — inconsistent tables
and hybrid tables — are unsatisfied at the unprivileged layer on both measured
platforms. **macOS was measured 2026-08-05 and answers the same way**: the
client projections are byte-identical across the decisive pair there too, so
all three supported platforms agree. The privileged leg taken the same day
(M10) located the separating fact in the backup table, behind a read SAFE-002
places outside the discovery layer — which is why the resolution scopes the
duty by privilege rather than hunting for a client interface that does not
appear to exist.

**The escape that does not work.** The natural reading that dissolves the
conflict is that the privileged helper detects these states, so the *product*
satisfies INV-003. It fails on timing:

> **HLP-002:** **Before the first write**, the helper independently
> re-discovers topology and recomputes capability and validation results.

HLP-002 is plan-time. INV-003 is an inventory requirement. At inventory there
is no privileged observer in the loop at all, so the unprivileged layer is the
sole observer and the requirement lands on it alone.

**Why this is not SI-35.** SI-35 asks which observer computes ADR-C3's table
states and from what contract. This issue asks whether a Section 7 MUST is
satisfiable at the layer Section 3 assigns it to. They meet — any SI-35 answer
that makes the decisive discrimination helper-only leaves INV-003's clauses
unsatisfied at inventory — but they are not the same question, and an ADR
cannot resolve this one. Section 0.2 item 4: ADRs "refine this spec but MUST
NOT weaken any MUST." Every resolution below is a normative amendment, so it
needs a spec change and not an ADR.

**Options, none decided and none recommended:**

(a) **Scope INV-003 by privilege.** The requirement gains an explicit split:
what the unprivileged discovery layer MUST detect, and what only the
privileged path detects. Cost: the unprivileged inventory becomes explicitly
incomplete on a safety-relevant case, and every consumer of an inventory must
learn which half it holds.

(b) **Add a fail-closed clause to INV-003.** The discovery layer MUST detect
the states it can and MUST report the remainder as undetermined rather than
absent, tying INV-003 to SAFE-005's existing fail-closed rule. Cost: needs a
definition of "undetermined" at the inventory layer that does not simply
restate ADR-C3's `Indeterminate`, or it pre-empts SI-35.

(c) **Qualify SAFE-002 for this detection.** Discovery gains a narrow
privileged leg for table-state probing. Cost: SAFE-002 is a Section 3 safety
constraint with Section 0.2 precedence over everything, and weakening it to
satisfy a Section 7 functional requirement inverts that order. Recorded for
completeness; it is the option the precedence rules argue hardest against.

(d) **Establish that some client-readable interface does separate these
states**, dissolving the conflict empirically rather than normatively. No
candidate interface has been named. Two complete projections have failed to
supply one, and neither failure proves none exists.

**Evidence required before any option is accepted:** for (d), a named
client-readable interface measured to separate the decisive pair on every
supported platform, under the custody rules the existing protocols use. For
(a), (b), and (c), a statement of what an inventory consumer may rely on when
the undetected case is present, tested against `gpt-conflicting-tables-512`
and `hybrid-mbr-gpt-512`.

**Dependencies, as they stood at filing.** SI-35 could not be decided before
this resolved: an SI-35 axis making `Indeterminate` helper-only would have
been an implicit choice of option (a), the silent side-picking Section 0.2
forbids. It was classified a **transitive blocker** rather than an input
because an input is "resolved within the consuming direct blocker's decision"
and this one could not be — an ADR may not amend a MUST, so it had to be
sequenced separately. **All of that is now history: this issue resolved in
spec 6.0.0 and the gate on SI-35 is lifted.** `INV-004`'s adjacent clauses
were untouched by this filing and remain so.

## SI-39 SAFE-003 says a blank device can be Strong; INV-003 forbids the client saying so

> **Resolved 2026-08-08 in spec 7.0.0 by ADR-0015.** SAFE-003's
> blank-can-be-Strong derivation is scoped to the observing contract; the
> strength rule itself is untouched, so Strong means the same thing on
> every platform and only the attainable population varies. On a platform
> whose client contract does not separate the absent case — macOS, by the
> increment 6 measurement this filing rests on — client-derived records
> for blank media are Weak by the rule's own terms, PART-001 routes
> through the weak-identity path whose pre-apply re-probe is the
> separating observation (M10), and the plan's claim is "initialize this
> device, which the client could not distinguish from occupied," never
> "this medium is blank." Rejected and recorded in the ADR: reach-relative
> strength (weakens the guarantee, not the population), reportable-`Absent`
> under caveat (the recorded data-loss path), a split client/helper
> strength vocabulary, and option (d) retained as a self-executing revisit
> condition rather than a resolution. The `Present` face of INV-003's
> sentence stays deliberately with SI-35, exactly as filed. The filing
> below is retained as history.

**Requirements:** SAFE-003, INV-003, ADR-C3, ADR-C4, MODEL-005, Section 0.2 ·
**Resolved** (was: direct blocker, hash-visible)

Filed 2026-08-05 from a measurement and an adversarial review. Section 0.2
requires this filing rather than permitting it: "If two requirements in this
spec conflict, agents MUST stop, file a spec issue describing the conflict, and
not silently pick a side."

**This repository created the conflict, hours before finding it.** INV-003's
governing sentence was added by ADR-0013 in spec 6.0.0 on 2026-08-05, and that
ADR's adversarial round did not reach SAFE-003. Recording the provenance is not
self-flagellation: a register that files a conflict as though it were
discovered in the specification, when it was introduced into it, misleads the
next round about where to look for others.

**The two requirements, quoted.**

> **SAFE-003:** Every plan that writes storage MUST bind each target to an
> immutable identity record containing all available identifiers: … Partition-
> table type and state, which MUST distinguish `Present` (read and hashed),
> `Absent` (positively observed to have none), and `Indeterminate` (unreadable
> or ambiguous). Only the first two are positively determined. **A blank device
> can therefore be Strong**; a device whose table failed to parse cannot.

> **INV-003:** Unprivileged discovery MUST detect every state its platform
> contract can distinguish, and **MUST NOT report a state its contract cannot
> reach**. Reporting a table as consistent, **or a medium as positively without
> a table**, is such a report where the contract does not separate that case.

**What makes it concrete.** The increment 6 macOS matrix (2026-08-05, valid on
its second sitting) established that `blank-512` and media carrying a live
ext4 with a stale mdraid superblock, an mdraid member, a LUKS2 container, and
an LVM2 orphan all produce **byte-identical** unprivileged projections. The
macOS client contract therefore does not separate the `Absent` case.

So on macOS: INV-003 forbids the client reporting a blank medium as positively
without a table; the state is therefore not positively determined; and by
SAFE-003's own classification the device is Weak — where SAFE-003 says in terms
that a blank device can be Strong. Both are requirements, both are quotable,
and they cannot both hold on that platform.

**Why this is not SI-38.** SI-38 was INV-003 against SAFE-002 — a Section 7
detection duty assigned to a layer Section 3 places at no elevation. It
resolved by scoping the duty. This is INV-003 against **SAFE-003**, about what
an identity record may contain and what strength follows, and SI-38's
resolution is what created it.

**Why this is not SI-35, and the boundary is deliberate.** INV-003's same
sentence also forbids "reporting a table as consistent" where the contract does
not separate that case — and no measured client contract on any of the three
platforms separates a healthy GPT from one whose two tables disagree. Whether
reporting `Present` on such a medium is itself a forbidden report is a live
question that reaches all three platforms rather than macOS alone. **It is
recorded here and deliberately not decided**, because it overlaps SI-35's open
axis question, and settling it inside a filing about a different conflict would
be the silent side-picking Section 0.2 forbids.

**Options, none decided and none recommended:**

(a) **Amend SAFE-003's strength classification** so a positively determined
table state is not required for Strong where the platform contract cannot
reach it — for example by making the requirement relative to the published
INV-003 reach. Cost: strength stops being one notion across platforms, and
ADR-C3 chose the current rule deliberately; SI-02's resolution rests on it.

(b) **Amend INV-003** so a medium the contract cannot distinguish from blank
is reportable as `Absent` under a stated caveat. Cost: reintroduces exactly
what ADR-0013 was written to end, since a macOS client would report `Absent`
for a disk holding a LUKS2 container — and PART-001 initializes blank media.
This is the option with a recorded data-loss path.

(c) **Accept the consequence**: blank media are Weak on platforms whose
contract cannot separate `Absent`, and SAFE-003's "a blank device can
therefore be Strong" gains a platform qualifier. Cost: SAFE-003's
weak-identity policy — pre-apply re-probe and the unattended-apply refusal —
applies to ordinary blank media on macOS, and ADR-C3's recorded consequence
that "a strong-identity blank removable now qualifies for SAFE-003's replug
path-change allowance" is narrowed.

(d) **Establish that some client-readable macOS interface separates the case**,
dissolving it empirically. No candidate is named; the matrix measured the two
interfaces the contract reads, and M10 located the separating fact behind a
privileged read.

**Evidence required before any option is accepted:** for (d), a named
client-readable macOS interface measured to separate a blank medium from an
occupied one, under the custody rules the existing protocols use. For (a),
(b) and (c), a statement of what an inventory consumer and a plan may rely on
for a medium in the unseparated case, tested against `blank-512` and
`luks2-whole-disk-512`.

**Dependencies.** The macOS rows this rests on carried an outstanding
second-reader readback when this was filed; **that readback was discharged
2026-08-08** by an independent reader session, every digest matching (the
discharge and its custody caveats are in `docs/quality/observability.md`),
so the measurement half of this filing no longer waits on custody. Whether SI-39 must resolve before SI-35 is
deliberately not asserted: the two interact through INV-003's single sentence,
but no ordering between them is established here.

---

# Part 2 — Blocking later work packages

## SI-12 Multipath devices and the single connection path

> **Resolved 2026-08-02 in spec 4.3.0 by ADR-0011.** v1 represents multipath
> detection-only, with a platform-neutral Section 2.1 non-goal entry as the
> normative home: the inventory carries the platform's own multipath node
> and its member paths connected by the kernel-reported membership relation
> — the edge kind deliberately left to SI-27's naming round, per round
> three's own finding that signatureless host-assembled devices need a new
> edge kind — the product infers no cross-path sameness of its own, and
> mutation reports CAP-003 `unsupported` with a multipath reason. Equal
> stable identifiers with no platform-assembled multipath node are SAFE-005
> ambiguity, `blocked`. The retained bridge-synthesis and two-layer-serial
> measurements establish only that identifier equality cannot be assumed
> across bridges or observation layers; no retained run measured one LUN on
> two simultaneous paths with unequal identifiers. That unassembled-and-
> unequal population remains an unmeasured, uncovered residual filed as SI-37,
> not as covered. The path-set
> encoding — including its body-versus-envelope placement, itself part of
> what this issue left undecided — is deferred behind a MODEL-003 version
> bump to the spec change that first makes multipath a supported target,
> gated on multipath observability rows that do not yet exist. The
> transitive block on SI-27 lifts; the equal-identifier simultaneous-pair
> collision family is assigned to SI-27's scope. The filing below is
> retained as history.

> **Reclassified by round three (block lifted by the resolution above): this blocked SI-27, and therefore increment 3.** Two paths to one LUN must deduplicate to a single device node with the path set in the envelope, or any node-naming scheme is wrong on the first SAN it meets — a multipath pair is one device seen twice, which is the opposite of two ambiguous devices and needs the opposite treatment. *(The resolution takes the deduplication from the kernel's own assembly rather than performing it, and defers the path-set placement it directed.)*

**Requirements:** SAFE-003, LIN-006, INV-001 · **Was: Blocks 3 (earlier: Later, WP-L100), hash-visible · Resolved 4.3.0**

SAFE-003 models one connection path per identity record and makes a path change
the special case for removable replug. LIN-006 requires detecting multipath and
device mapper, and INV-001 requires hardware RAID LUNs — devices that
legitimately present several concurrent paths. Whether the bound record holds
one canonical path, an ordered set, or an unordered set is unstated, and a
differing path count or order would make an unchanged device compare unequal at
re-probe.

## SI-13 Identity binding for pool and array write targets

**Requirements:** SAFE-003, LIN-004, LIN-005, UI-009, ACC-014 · **Later (WP-L110)**

SAFE-003 enumerates device-level identity fields only. LIN-004 and LIN-005 make
pools and arrays direct write targets possessing none of them. Whether such a
plan binds the union of member identities, the pool UUID, or both — and how
strength is classified for an aggregate whose members differ — governs whether
the weak-identity path applies to an mdraid grow.

## SI-14 Derived properties have no confidence rule

**Requirements:** MODEL-004, INV-004 · **Later (WP-050)**

INV-004 lists free extents and alignment among properties to detect, bringing
them under MODEL-004's provenance requirement, but both are computed from other
provenanced properties. None of the four confidence values describes a derived
value, and no rule composes a derived property's confidence from its inputs.

## SI-15 Pre-existing misaligned partitions

> **Resolved 2026-08-11 in spec 12.1.0 by ADR-0023.** A PART-009
> deviation is **authored, not inherited** — an act the plan performs,
> never a state it finds. An authored boundary (one whose byte offset
> the plan sets) meets the 1 MiB default, is placed coincident with a
> pre-existing structural edge (conformant, recorded as coincident — the
> adversarial round's sharpest finding: without this rule the same issue
> re-files about the grown end), or carries one of PART-009's two
> existing deviation causes; there is no fourth state. A boundary
> byte-identical before and after the plan is an inherited fact:
> no override, no block, recorded in consequence text as a fact about
> the device rather than a grant by the user. The filed case proceeds —
> growing a legacy misaligned MBR partition at its tail authors only the
> aligned new end — and realignment stays available only as an explicit
> PART-005 move at severity 3, so a grow is never silently a move in
> either direction. Section 11.2's preserved-alignment invariant reads
> as the split implies, with no text change. Rejected and recorded in
> the ADR: the strict reading (safety theater that fixes no alignment
> while locking the legacy population out of maintenance), auto-realign
> (severity laundering, the silent-consequence shape this register has
> refused every time), permanent refusal (fail-closed posture spent
> where no failure exists), and typed alignment-fact carriage (retained
> as a revisit condition). The solver's named refusal case unlocks
> without the deviation-override vocabulary, which stays deliberately
> inexpressible; the code change rides WP-060's next Rust increment.
> The filing below is retained as history.

**Requirements:** PART-009, PART-004, PART-005, Section 11.2 ·
**Resolved** (was: Later (WP-060))

PART-009 permits alignment deviation only when published geometry requires it or
the user explicitly overrides. A legacy MBR partition at a non-1 MiB offset
grown at its tail matches neither cause, yet realigning it forces a data move the
user did not request.

## SI-16 Backup-before-first-write on blank or corrupt media

> **Resolved 2026-08-11 in spec 12.2.0 by ADR-0024.** PART-013
> discharges by the helper's authored table state — each of the filing's
> three options is right somewhere, and the error was choosing one for
> all cases. `Present`: the parse-level backup stands untouched,
> verified, failure → Failed. `Absent` (the helper's fresh positive
> determination, the same one PART-001 requires): the obligation
> discharges as a journaled determination — the backup record is the
> positively determined absence, a value not a skip (ADR-C4 reaching
> the journal), with no user acknowledgement, which could only train
> the rubber stamp on a fact it cannot inform. `Indeterminate`:
> ordinary operations stay SAFE-005-disabled before PART-013 is
> reached, while the typed REC-001 repair family — a step class, never
> an intent flag — backs up a verified raw capture of exactly the
> regions it will write, the only truthful backup of an unsound source;
> capture-impossible refuses per Section 8's existing row, with the one
> exit Section 12's MUST-NOT clause already carved formalized as a
> plan-creation journaled acknowledgement naming the uncapturable
> regions. A blank device and an unreadable one never take the same
> arm, and no arm is silent. Rejected and recorded in the ADR: uniform
> vacuous satisfaction (fail-open on corrupt media), uniform
> acknowledgement (ceremony where it cannot inform), uniform block (the
> filing's own reductio — PART-001 unrunnable, the repair family
> fail-closed against itself). The protection record's journal encoding
> lands with JRN-006 under WP-070, jointly sequenced; REC-011's
> corrupt-encryption-header twin stays WP-R100's under this shape when
> designed. The filing below is retained as history.

**Requirements:** PART-013, PART-001, REC-001, INV-003, SAFE-005 ·
**Resolved** (was: Later (WP-060))

PART-013 requires backing up table metadata before the first table write, and
Section 8 routes backup failure to Failed. On blank media there is nothing to
back up; when restoring a damaged table the backup source is precisely what is
unsound. Whether an absent or corrupt prior table satisfies PART-013 vacuously,
requires a journaled acknowledgement, or blocks decides whether the operations
intended to repair a table are fail-closed against themselves.

## SI-17 Severity 1 versus the `irreversible-after-start` flag

> **Resolved 2026-08-11 in spec 12.3.0 by ADR-0025.**
> `irreversible-after-start` is defined temporally, for the first time:
> a step carries it when a reachable interrupted state exists from which
> the pre-step state cannot be restored by unwinding — once the first
> write lands, stopping cannot go back, and interruption recovery is
> roll-forward per the journal, never unwind. The criterion is a
> reachable unrestorable intermediate, not the existence of a write
> (the journaled PART-005-shape copy is unflagged; the in-place
> multi-sector rewrite is flagged). The flag therefore claims the
> mid-execution window while severity claims endpoints — "fully
> undoable before or after apply" quantifies over before-first-write
> and after-completion, ADR-0022's completed-apply boundary — so **the
> combination is legal** and PLAN-004's declared orthogonality becomes
> true rather than aspirational. One coupling rule: a flagged step's
> cancellation claims `no-writes` only before its first write, its
> post-write outcomes `partial` or completion — Section 8's existing
> effect values, selected, not extended. Cannot-stop (PLAN-005's
> `non-cancellable`) and cannot-unwind are independent facts in both
> directions. No new guard was needed: any flag binds the interactive
> ceremony (ADR-0021), the severity-1 reversal draft stands (ADR-0022),
> and UI-005 displays both facts — an inflated severity would repeat
> the 2.0.0 conflation and lie about the completed effect. Rejected
> and recorded in the ADR: permanent illegality (severity inflation or
> flag suppression), endpoint-irreversibility as the definition
> (redundant, contradictory by construction), and dropping the flag
> (deletes the interruption-window warning). The planner's named
> refusal unlocks, riding the crate's next Rust increment. The filing
> below is retained as history.

**Requirements:** PLAN-004, PLAN-005, UI-009, HLP-003 ·
**Resolved** (was: Later (WP-060))

PLAN-004 declares the flags orthogonal to severity, but severity 1 is "fully
undoable ... via an emitted reversal plan", which `irreversible-after-start`
directly negates. The flag is never defined, nor is its relationship to
PLAN-005's `non-cancellable` class — cannot-stop and cannot-undo may or may not
be the same thing. Since UI-009 and HLP-003 key off severity plus flags, the
model cannot silently decide whether the combination is legal.

## SI-18 Does a severity-1 plan need fresh authorization?

> **Resolved 2026-08-11 in spec 11.2.0 by ADR-0021.** Authorization is a
> two-tier ladder and SAFE-002 is untouched — the SI-38 precedence shape,
> a Section 3 constraint never bent to fit a lower section. Every apply
> at every severity requires a floor authorization: a fresh, explicit act
> by the RPC-001-authenticated user naming the exact plan hash,
> single-use, PLAN-007-windowed, journaled, never cached, and satisfiable
> programmatically — which keeps SAFE-003's unattended/scripted-apply
> population live. The interactive ceremony HLP-003 already required at
> severity ≥ Disruptive stands verbatim and additionally binds any plan
> carrying a step flag — the severity-plus-flags participation PLAN-004
> promised and HLP-003 never stated; the concrete gap was a LUKS keyslot
> addition, fully reversible yet `security-sensitive`, which a
> severity-only ladder would have given the lightest authorization in
> the product. The enforced tier derives from the helper's own
> recomputed severity and flags (HLP-002), never from client claims.
> **The entry's named question is answered: no authorization-requirement
> field enters the plan** — the requirement is a total function of body
> content already present, a stored copy would add only an agreement
> obligation (ADR-0016's lesson reached with no field at all), and
> WP-040's authorization vocabulary unlocks with no jointly-sequenced
> WP-010 schema change. Rejected and recorded in the ADR: reading
> SAFE-002 through HLP-003's silence, the ceremony everywhere, and the
> helper-authored plan-carried field. The filing below is retained as
> history.

**Requirements:** SAFE-002, HLP-003, Section 0.2 ·
**Resolved** (was: Later (WP-040))

SAFE-002 confines privileged behavior to a helper "executing a validated plan
after fresh, explicit user authorization", which reads as every privileged
execution. HLP-003 requires fresh interactive authorization only for severity
≥ 2. A severity-1 plan still writes storage and still needs privilege. Section
0.2 gives Section 3 precedence, pointing to SAFE-002, but the two are written in
contradiction, and the answer decides whether the plan carries an
authorization-requirement field distinct from severity.

## SI-19 A reversal plan has no snapshot to bind to

> **Resolved 2026-08-11 in spec 12.0.0 by ADR-0022.** The reversal is an
> ordinary `OperationPlan` draft, linked by reference — **`OperationPlan`
> is not recursive**, the entry's named question answered. The filing
> predated 8.0.0, which dissolved its core: binding is a validation act
> for every plan, so a reversal emitted at planning time is exactly as
> unbound as every other draft; its proposal is the simulated final
> topology and its binding is its own validate-plan after the forward
> apply, so nobody ever applies a prediction and the delivered
> Simulated-never-binds rule stands untouched. Section 6's body item
> becomes reversal linkage — the draft's plan ID and body hash, acyclic
> by construction (forward→hash, reversal→ID) — and round three's
> created-node residue gets its only possible spelling: typed
> step-output references, resolved to derived addresses at the
> reversal's validation per ADR-0019, refusing when unresolvable.
> Truthfulness is a two-time property re-checked as body-content
> preconditions (the volume-that-gained-data case refuses rather than
> silently becoming destructive); a reversal apply takes its own
> ADR-0021 authorization; a stale or refused draft re-plans under
> PLAN-007. Rejected and recorded in the ADR: binding the simulated
> topology, exemption from binding, lazy re-planning with no emission
> (surviving as the staleness fallback), and recursive embedding. SI-15,
> SI-16, SI-17, SI-20, SI-24 and every REC-* behavior stay open; the
> linkage encoding lands as the jointly-sequenced WP-060/WP-010 schema
> change when implemented. The filing below is retained as history.

> **Amended by round three.** Not orthogonal to naming, as previously assumed, but strictly harder. Round three showed that most nodes a plan creates *can* be named from position relative to already-named nodes — a new encryption layer from its backing partition, a new file system from the mapper device. The residue that cannot is small and enumerable: volumes minted inside an existing container (`newfs_apfs`) and LVM snapshots, which have no position to be named from until they exist. That residue is this issue's, not SI-27's.

**Requirements:** PLAN-008, PLAN-002, PLAN-006, HLP-004, Section 6 ·
**Resolved** (was: Later (WP-060))

PLAN-008 requires the planner to emit a reversal plan at planning time. Section 6
requires every plan to carry a source topology snapshot hash, and PLAN-006
requires the helper to reject a mismatch — but the topology a reversal plan runs
against does not exist yet and has only a simulated snapshot. Whether a reversal
plan binds the simulated final topology, is emitted unbound and re-planned after
apply, or is exempt decides whether `OperationPlan` is recursive.

## SI-20 RecoveryRequired has no exit in the transition table

> **Resolved 2026-08-12 in spec 12.5.0 by ADR-0027.** The transition
> table's two RecoveryRequired exits are the two arms of REC-009's own
> disjunction, and the table is complete under the reading that splits
> recovery actions the way the architecture splits plans. A
> roll-forward action continues the *original* plan — same hash, same
> journal, resuming from the last durable checkpoint through the
> existing → Executing edge, re-verification inherited from JRN-003 —
> and is the one recovery act that is not its own plan, stated as a
> scoping of the prose sentence whose every other instance remains
> true. Any distinct recovery action is its own `OperationPlan`, and
> selecting it is the acceptance the → Failed trigger names: the
> original terminates with its honest effect summary, the full report,
> and a journaled linkage naming the recovery plan — one user act, two
> records, the disposal durable before the recovery plan may apply
> (JRN-002's shape, HLP-005-structural on shared device sets, so the
> filed torn state is unreachable). No state, edge, or trigger is
> added; the rows, terminal list, and "No other transitions exist"
> stand verbatim, and no → Cancelled edge exists because unwind
> semantics belong to the Executing era. Rejected and recorded in the
> ADR: recovery-executing-as-the-original (breaks plan-hash binding),
> new exits or a `Superseded` terminal (couples lifecycles or renames
> a fact the linkage carries), rewording the Failed row (retexts a
> machine-readable row at major for what prose does at minor). SI-21's
> authorization-reuse question is untouched on both edges. No
> re-attribution follows: no WP-070 assignment exists, and the ADR
> records the verification obligations so that assignment's creation
> cannot omit them. The filing below is retained as history.

**Requirements:** Section 8, REC-009 ·
**Resolved** (was: Later (WP-070))

Section 8 states recovery actions "are themselves plans under this same
contract", but the table moves the *original* plan directly
`RecoveryRequired → Executing`. A recovery action that is its own plan has its
own lifecycle and hash; if a second plan executes, the original stays in
RecoveryRequired, for which the table provides no exit — there is no
`RecoveryRequired → Completed` or `→ Cancelled`.

## SI-21 Resume and roll-forward reuse an authorization HLP-003 forbids

> **Resolved 2026-08-12 in spec 12.6.0 by ADR-0028.** No reuse occurs,
> because nothing is used twice: an authorization act authorizes one
> **apply**, and an apply is a journal-continuous execution lifecycle —
> from its act to a terminal state, identified by the plan hash and an
> unbroken JRN-001 chain — that interruption suspends and only
> `Completed`, `Failed`, or `Cancelled` ends. The three re-entry edges
> continue the *same* apply under the *same* journaled, hash-bound,
> single-use act, consumed once at the apply's start. The helper-exit
> worry dissolves because the authorization is a journal fact, never
> process state (JRN-003 reconstructs; HLP-005's idle exit discards
> nothing meant to persist in a process); the caching prohibition
> forbids approvals outliving their apply, not applies outliving
> interruptions. Freshness has its boundary in PLAN-007's existing
> machinery: every re-entry is bounded by the validity window, a
> re-entry past expiry is rejected per HLP-004 and readmitted only
> through re-approval against a fresh snapshot — a fresh act for the
> same continuing apply, one-act-one-apply being a ceiling on an act's
> reach, never a floor on their count. Each edge keeps its named
> verification, and WIN-009 reads as same-apply continuity, not a
> retained grant. Rejected and recorded in the ADR: re-prompting on
> every resume (rubber-stamp training plus new table edges), retained
> helper state (contradicts HLP-003 outright), severity-scaled resume
> prompting (a second encoding of the ladder's dimension). Fed forward
> to SI-22, undecided: the authorization record is recovery-critical.
> No re-attribution follows — no WP-070 assignment exists, and the ADR
> records the verification obligations so its creation cannot omit
> them. The filing below is retained as history.

**Requirements:** HLP-003, HLP-005, Section 8, WIN-009 ·
**Resolved** (was: Later (WP-070))

The table reaches `Executing` from `RecoveryRequired`, and `Protecting` from
`Revalidating` after `RebootPending`, without passing `AwaitingAuthorization`.
So a roll-forward or post-reboot resume writes storage under an authorization
granted before the interruption — possibly after the helper exited, which
HLP-005 permits. WIN-009 suggests reuse is intended; that is exactly the
retained grant HLP-003 forbids.

## SI-22 Journal retention can delete what recovery depends on

> **Resolved 2026-08-12 in spec 12.7.0 by ADR-0029.** Liveness-scoped
> retention: bounded and unbounded stop colliding when they stop
> sharing a population. Retention MAY reclaim only records of terminal
> applies — a non-terminal apply's records, the authorization act's
> included (ADR-0028's fed-forward fact, absorbed), are exempt until
> their apply terminates, with the exemption closing over ADR-0027's
> linkage graph (finite, because chains are). JRN-004's bound stays
> true universally: terminal history bounded by SEC-009's retention
> controls, the live segment by a per-apply journal budget whose
> exhaustion is a journaled failure through Section 8's existing
> edges — fail-closed toward the writer, never the recoverer, the
> round's sharpest finding turned into an enforced property.
> Reclamation writes a durable compaction record, so replay classifies
> every gap: policy, torn tail (JRN-001's rule, governing the tail
> while compaction governs the head), or corruption, which refuses.
> Sequence numbers are never reused or reset across rotation or
> compaction. The exemption is the enforced correctness floor;
> audit-log retention beyond it stays SEC-009's user-controlled
> domain. ADR-0028's revisit condition is discharged by this
> reconciliation. Rejected and recorded in the ADR: retention-wins
> (the filed trap ratified), recovery-wins-transitively-forever
> (unbounded journal), time-capped exemption (re-creates the hazard on
> exactly the unbounded state). The budget's magnitude and the
> compaction record's encoding land with JRN-006 under WP-070, jointly
> sequenced; no re-attribution follows — no WP-070 assignment exists,
> and the ADR records the verification obligations so its creation
> cannot omit them. The filing below is retained as history.

**Requirements:** JRN-001, JRN-003, JRN-004, SEC-009, Section 8, SAFE-005 ·
**Resolved** (was: Later (WP-070))

JRN-004 requires bounded journals with retention controls; JRN-003 requires
recovery state to derive solely from the journal; Section 8 requires
RecoveryRequired to persist until the user acts, which is unbounded in time.
Nothing exempts records belonging to a non-terminal plan, so retention can delete
the records recovery needs and SAFE-005 then fails closed on a plan the product
itself is holding open. How rotation preserves JRN-001's monotonic sequence and
torn-tail semantics is also unstated.

## SI-23 The encryption-metadata backup artifact has no protection owner

> **Resolved 2026-08-12 in spec 12.8.0 by ADR-0030.** The REC-011
> backup is a first-class **protection artifact** with four rules.
> Home: a dedicated helper-owned store inheriting JRN-004's
> admin-protected documented-location clause, sibling to and never
> inside the journal — JRN-005's bounds stand, ADR-0029's budget is
> not bloated, replay never drags key-slot material past every
> consumer, and ADR-0029's named fork is answered in terms: the
> lifecycle does not route through the journal. Reference by
> identity: the journal, the plan, and every SAFE-006 surface carry
> the artifact's content hash and store identity only — SAFE-006's
> list stands verbatim, a hash is not the material, only the helper
> reads the store (SAFE-008), and a restore is an identity-validated
> plan (REC-001) at its own authorization tier. Retention: ADR-0029's
> liveness rule, adopted in REC-011's own text — exempt while the
> creating apply or its referencing linkage closure is non-terminal,
> the filing's "RecoveryAction must reach it" made structural. End of
> life: explicit user-controlled retention in SEC-009's shape, never
> silent in either direction — the deciding surface states that
> retention preserves revoked-passphrase slots and deletion forfeits
> the only disaster-recovery copy, with displayed changeable defaults
> permitted and silence forbidden. Rejected and recorded in the ADR:
> journal embedding, arbitrary user-chosen location, and auto-delete
> together with its silent-retention mirror. ADR-0024's corrupt-source
> discharge stays WP-R100's, untouched. No re-attribution follows —
> neither assignment exists, and the ADR records the verification
> obligations so their creation cannot omit them. The filing below is
> retained as history.

**Requirements:** REC-011, SAFE-006, JRN-004, JRN-005, REC-001 ·
**Resolved** (was: Later (WP-R100))

REC-011 requires backing up encryption-layer metadata — explicitly the LUKS
header, which contains key slots — before mutating that layer. SAFE-006 and
JRN-005 forbid key material in logs, plans, journals, and UI state, but nothing
says where this artifact lives, how it is protected, or whether it inherits
JRN-004's admin-protected location. It cannot be discarded, because RecoveryAction
must reach it.

## SI-24 CAP-003 `preview` versus PLAN-009 dry-run parity

> **Resolved 2026-08-12 in spec 12.4.0 by ADR-0026.** CAP-003's
> "simulation" is the planner's prediction — PLAN-001 planning and
> PLAN-002's simulated final topology, the pure surface `preview`
> licenses — while a PLAN-009 dry run is an apply rehearsal belonging
> to the surface `preview` refuses. The conflict turned on one
> undefined word the spec's own vocabulary had already split: PLAN-002
> names its output the simulated final topology, PLAN-009 never calls
> the dry run a simulation, and the glossary defines Preview with no
> dry-run mention. A dry run of a preview-backed plan **runs** — not
> refused upfront from the client's advisory view, which would invert
> CAP-007 in the refusing direction — and terminates at the helper's
> own recomputed capability gate with a typed refusal naming the
> qualification gap and its CAP-006 remediation, distinguishable by
> type from every validation-failure class. Such a dry run is never
> successful, so PLAN-009's guarantee stands absolute with no
> success-with-caveat outcome representable. The pipeline's internal
> gate order is deliberately not decided: parity is the property,
> sameness of the dry-run/apply refusal pair is what verification
> asserts, and the order is WP-070's. Rejected and recorded in the
> ADR: success-with-carried-caveat (the asterisk that eats the one
> crisp guarantee), the partial pipeline (the second pipeline PLAN-009
> forbids), narrowing `preview` (amputation), upfront client-side
> refusal (CAP-007's inversion). Decided before the pipeline exists,
> deliberately: no evidence clause names an unbuilt artifact, and the
> decision constrains the implementation rather than reading it — the
> ADR-0022 class. WP-060's last register gate clears. The filing below
> is retained as history.

**Requirements:** CAP-003, PLAN-009, HLP-002, CAP-007 ·
**Resolved** (was: Later (WP-050))

CAP-003 says preview permits "planning and simulation" while apply is refused.
PLAN-009 says a dry run traverses the identical pipeline including helper
revalidation, and that success means only physical outcomes remain uncertain.
A dry run of a preview-backed plan must therefore either fail (contradicting
"simulation permitted") or succeed while apply is still guaranteed to be refused
for a non-physical reason (contradicting PLAN-009).

## SI-25 CAP-002's operation list does not span the operation surface

**Requirements:** CAP-002, DIA-004, DIA-005, PART-007, PART-010, PART-011 · **Later (WP-050)**

CAP-002 enumerates fourteen operations including a single `wipe`, but DIA-005
requires overwrite, crypto-erase, sanitize, format, discard, and file deletion to
be distinguished and "never call them equivalent". Modelling them as one `wipe`
violates DIA-005; adding variants exceeds CAP-002's list. Separately PART-007 and
PART-010 map to no CAP-002 operation at all. Whether the list is a closed
enumeration or a required minimum is unstated.

## SI-26 `Stable` is not a capability status

**Requirements:** Section 16, CAP-003 · **Later (WP-050)**

Section 16 forbids marking a capability "Stable" without matrix fixture and
acceptance evidence, but `Stable` is not one of CAP-003's four values
(`supported`, `preview`, `unsupported`, `blocked`) and appears nowhere else.
Either it is a stale synonym for `supported`, in which case Section 16's evidence
rule attaches there, or `Capability` needs a maturity axis orthogonal to status.

## SI-40 FS-007's "blocked reasons" versus CAP-003's `blocked` definition

> **Resolved 2026-08-10 by ADR-0020, no spec change — deliberately.**
> Reading (a) of the options below, decided by the decision owner the
> same day the filing landed: FS-007's "blocked reasons" is the generic
> noun phrase for the capability reason vocabulary, and an immutable
> technology limit's status follows CAP-003's definitions —
> `unsupported`, carrying the limit as its explicit reason and a
> remediation stating no remedy exists. No normative sentence moves;
> the ADR records why the no-amendment resolution is not an omission:
> `blocked` keeps meaning remediable, so a permanent impossibility
> never invites remediation of the unremediable. WP-050 increment 2's
> technology-limit composition is unblocked; nothing else waited.

**Requirements:** FS-007, CAP-003 · **Later (WP-050, before increment 2
composes technology limits)** *(filed class, retained; see banner)*

FS-007: "Surface immutable technical limits, such as XFS not shrinking, as
explicit blocked reasons." CAP-003's definitions: "`blocked` — implemented,
but a runtime precondition fails (missing tool, version, state)";
"`unsupported` — the product does not implement the operation for this
target." An immutable technology limit is not an implemented operation with
a failing runtime precondition, so giving it CAP-003 `blocked` contradicts
`blocked`'s own definition, while giving it `unsupported` contradicts
FS-007's word "blocked". One case, two statuses, both texts normative — and
the answer is product-visible on every capability surface, because CAP-005
serves them all from the one engine.

Surfaced 2026-08-10 while WP-050 increment 1 built the CAP-003 vocabulary.
That increment left the `TechnologyLimit` reason's status coupling
deliberately unasserted (`crates/capability/src/lib.rs`) rather than decide
this in a constructor; WP-050 increment 2's technology-limit composition
waits on the resolution, and its other arms — whose couplings are decided
texts — do not.

Options, none decided: (a) read FS-007's "blocked reasons" as the generic
noun phrase for the reason vocabulary — the reading this repository's prose
already uses ("the blocked-reason capability surface is WP-050's") — with
the status following CAP-003's definitions: `unsupported`, carrying the
limit as its explicit reason and a remediation stating no remedy exists;
(b) read FS-007 as mandating the literal `blocked` status, and amend
CAP-003's definition list so `blocked` admits immutable limits;
(c) any distinct shape a resolution round proposes. As classification
rather than recommendation: (a) amends no normative text; (b) retexts
CAP-003's definitions.

---

# Part 3 — Design findings, not specification conflicts

These are decisions within the implementer's authority. They are recorded so
they are not rediscovered, and will be resolved when increment 3 is written.
They do **not** require a specification change.

- **Set ordering must be over encoded bytes, not a discriminant.** A
  discriminant is a total order only if it injectively determines the element,
  which is false for several proposed sets. Sorting each element's fully
  canonical encoding and rejecting a non-increasing successor is total by
  construction. *(MODEL-005)*
- **Node identifiers must be a pure function of stable content**, not of
  discovery arrival order, or a re-probe of unchanged topology produces
  different bytes and every Draft plan spuriously invalidates. *(MODEL-005,
  CONC-003)*
- **A label that is valid UTF-8 must have exactly one representation.** A
  `Utf8 | Raw` pair where both are legal for the same bytes lets two adapters
  disagree on an unchanged label. *(MODEL-005)*
- **Normalization belongs to adapters, never to the decoder.** A decoder that
  NFC-normalizes repairs rather than rejects, which is the malleability class
  ADR-C1 forbids. *(`schemas/canonical-encoding.md` §6)*
- **Version expressions must not be free text**, and need a fixed arity, or
  `2.9` and `2.9.0` become one version with two encodings inside the plan hash.
  *(MODEL-005, Section 6)*
- **Preserved unknown structure needs its own depth and size budget**, well
  below the profile limit, since it is the one field with externally controlled
  depth — and must not become a path for key material to enter a hashed
  artifact. *(INV-008, SAFE-006)*
- **No anonymous tuples in hashed types**, and no `Vec` where a set is meant.
  *(MODEL-005)*
- **`Null` must be either banned everywhere or permitted explicitly**; the two
  rules cannot both hold, and Rust and TypeScript will resolve it differently.
  *(MODEL-005)*
- **A secret must be structurally unable to reach a plan.** A reference type
  that cannot carry material, and cannot print it, is the only shape that makes
  SAFE-006 a property rather than a review item. *(SAFE-006, LIN-003, WIN-005,
  MAC-004)*
- **Detect-only must propagate to members, not only upward.** Marking a ZFS pool
  while leaving its member partitions mutable protects nothing. *(PART-014,
  Section 2.1)*
- **Copy-as-source is not mutation.** A binary per-node policy cannot express
  LDM's "migration off dynamic disks only via copy to basic disks". *(WIN-004,
  Section 2.1)*

---

# Part 4 — Analysed, not yet decided

Both remaining decisions were analysed in round one and both proposals were
rejected by adversarial review. The *direction* of each survives; the mechanism
does not. Recorded so the next attempt starts from the objection rather than from
scratch.

*(When this was written, four decisions remained. ADR-C3 and ADR-C4 have since
resolved two of them.)*

## SI-07 to SI-10, the aggregation vocabulary

**Direction that survived.** One aggregation node type carrying a technology
discriminant, rather than three disjoint types. Btrfs multi-device is a file
system with several backings, not a container. FS-004's non-file-system
signatures are materialized as container and encryption nodes rather than
enumerated into `FileSystemKind`, which preserves MODEL-002's layering.

**Why it was not accepted.** The proposal's load-bearing justification was that
detect-only status would be a total function over the closed kind enum. It is
not, on the first non-goal it maps: an Apple Fusion container and an ordinary
mutable APFS container carry the same kind and differ only by member-set shape
(MAC-010). Detect-only is therefore a function over kind *and* members — the
coarse-kind failure the proposal listed as a future risk, occurring immediately
against a Section 2.1 MUST NOT. Splitting the kind by instance shape contradicts
the proposal's own rule that the discriminant is the technology.

**What the next attempt needs.** Either a detect-only predicate defined over kind
and membership rather than kind alone, or a reason why membership-derived
protection is safe to compute outside the hashed body.

## SI-11, protection strength for non-goals

**Direction that survived.** Detect-only should be enforced structurally rather
than only at runtime, and it must propagate to members, not merely upward:
marking a ZFS pool while leaving its member partitions mutable protects nothing.
Read-as-source must be distinguishable from mutation, so Section 2.1's permitted
copy-based migration off dynamic disks (WIN-004) stays possible.

**Why it was not accepted.** The proposal removed Apple sealed volumes from
PART-014 and required mutating operations on a detect-only target to report
`unsupported` and never `blocked`. That contradicts MAC-009, which makes sealed
volumes protected objects and requires exactly `blocked`, with a stated reason,
for operations macOS permits only in Recovery. The proposal amended neither, so
merging it would have shipped a fresh Section 1.11 conflict on day one. The
reasoning underneath was also wrong: `blocked` means a runtime precondition
failed and the user has a next step, and "boot into Recovery" is precisely that.

**What the next attempt needs.** A status mapping that keeps MAC-009 intact,
distinguishing "this product will never do this" from "this platform will not
permit it right now".

---

# Part 5 — Protection model, round two

The decision owner chose the approach: **protection is proven by computation,
not asserted**, with the derived verdict frozen into the hashed body so it is
authenticated and a client/helper disagreement fails closed.

A second design attempt under that constraint **validated the approach and
failed on mechanism**. Both are recorded.

## What the round established, and it is the important part

The open worry was that an unprivileged client and a privileged helper might see
membership differently, making a frozen verdict diverge on unchanged hardware —
the same structural failure that hashing capture metadata caused for PLAN-006.

Per-platform investigation (including empirical, non-elevated testing on
Windows) found the asymmetry is real but **lands where it does not matter**:

> Every asymmetric fact is a *roster-identity* fact — which pool, which group,
> which volume group. No protection verdict in this product needs roster
> identity.

- **Windows LDM.** The group GUID and member roster are Administrator-only. But
  Section 2.1 forbids in-place LDM editing *uniformly*, so the group is not a
  verdict input. The per-disk GPT type that decides it reads unprivileged.
- **Windows Storage Spaces.** Pool membership is readable unprivileged through
  `MSFT_StoragePoolToPhysicalDisk` — verified on this machine from a
  non-elevated session, contradicting the widespread claim that it needs
  Administrator. A hardened host can revoke it by namespace ACL, but the member
  disk also carries its own GPT type, which is not policy-gated.
- **Linux ZFS and LVM.** An exported pool's vdev tree is root-only, and a VG
  roster with no active LV is invisible. Neither is a verdict input: ZFS is
  uniformly detect-only, LVM is uniformly supported. `ID_FS_TYPE=zfs_member` is
  in the world-readable udev database.
- **macOS Fusion.** The *only* case where member-set shape decides a verdict —
  and macOS is the one platform where shape is fully readable unprivileged.

The Fusion counterexample that killed round one and the asymmetry that
threatened round two are **disjoint**. Shape is needed exactly where it is
symmetric. That makes the chosen approach viable, and it is a fact about the
platforms rather than a design choice, so it holds regardless of what is built
on top.

## Why round two was still rejected

**1. Protection propagated through containment into unrelated siblings.** The
closure treated "disk contains partition" and "aggregate has member" as one
relation. On the standard root-on-ZFS layout — `zpool create tank /dev/sda2` —
protection flowed from `sda2` up to `sda` and back down to `sda1`, making the
**EFI System Partition immutable on every such machine**, which kills LIN-007
bootloader repair and REC-006 for that entire population. The same composition
fires on macOS: a Fusion store at `disk0s2` freezes `disk0s1`.

Containment is not membership. Protecting a ZFS member must not protect its
sibling ESP. The propagation rule has to distinguish the two edge kinds.

**2. The hashed body would contain a graph, and nothing names its nodes.** This
design is the first to freeze cross-references — backing edges, containment
edges, divergence reports — into the body. Every obvious naming scheme breaks
ADR-C2's requirement that identical hardware produce equal digests:

- Path-derived names are unstable across reboot, replug, and enumeration race.
- Positional indices are worse under INV-009 device churn.
- Names derived from the identity record inherit SAFE-003's *connection path*,
  so a replug changes the name — breaking the one case SAFE-003 explicitly
  blesses.
- Names derived from a path-free subset **collide** for exactly the population
  ADR-C3 calls Weak: two identical blank USB sticks behind bridges with no
  serial. Two nodes, one name, and the edge relation silently merges them.

That is a new blocking gap, filed below as SI-27.

## SI-27 The hashed body has no node-naming rule

> **Resolved 2026-08-09 in spec 11.1.0 by ADR-0019, on its round
> four.** Node identifiers are derived, kind-discriminated positional
> addresses — the round-three decomposition kept: an address, never a
> device identity — computed from fields ADR-0018's evidence contract
> reads, canonicalized by the contract's one named source per platform
> verbatim (no transformation, so the divergence-worse-than-collision
> hazard is structurally absent), recomputed at every decode by the
> schema-validation pass. Equal derived addresses collapse, before
> encoding, into counted, flagged, indeterminate **collision groups**
> whose operands are `blocked` pairwise — the representation of the
> ambiguity Section 2.1/ADR-0011 already declare, preserving two-ness
> and never silent, and the answer to this filing's
> indistinguishable-devices demand: the snapshot always encodes, the
> limitation attaches to exactly the colliding targets, and
> individually distinct addresses were established to require an
> excluded input. The ancestor-only address property is a committed
> property test; a duplicate-designator clone re-designates nothing.
> The four round-three collision families each have a mechanism:
> multipath via the **platform-membership** edge (typed here, its
> path-set encoding untouched and deferred per ADR-0011), virtual
> devices via `BackingExtent` and the **host-backing** edge (closing
> CONC-001's empty loop-device bind set and the own-fixtures-collide
> defect), stale signatures via offset-qualified addresses (the
> stale-pair fixture is the committed two-address regression), and
> table entries via role-discriminated views with partitions
> re-parented onto the table and `ConflictingTableEntry` evidence
> nodes scoped by ADR-0018's closure. Preconditions 2–4: the Linux
> designator rows exist; the Windows and macOS rows are named evidence
> obligations with designator-less aggregates `Indeterminate`
> meanwhile; `probe_tag` is discharged by the evidence contract; the
> preserved-unknown budgets are fixed (depth 4, 32 KiB, normative
> truncation-with-digest, versioned redaction). The theorem re-proof
> under the extended edge set lands with increment 3 as a property
> test. The filing and round history below are retained as the record.

> **Round-four handover, 2026-08-09 (ADR-0018).** SI-11's resolution
> states two requirements on this round's edge work, recorded here so
> the naming round designs against its consumer. First: the
> no-sibling-capture theorem's premise — no backing or production edge
> targets a physical device — is quantified over edge kinds, so any
> kind this round adds (the multipath membership edge ADR-0011
> deferred here, the host-backed-virtual-device edge round three's
> record requires) must preserve that premise or the theorem must be
> re-proved under the new edge set before acceptance, as a property
> test, not an argument. Second: every edge kind carries a semantics
> class, because ADR-0018's bind set traverses "the bytes of A live
> within or derive from B" in reverse over semantics, not over names —
> a new kind with the right class is traversed with no restatement,
> and that is what closes CONC-001's currently-empty loop-device bind
> set when the host-backing edge lands.

**Requirements:** Section 5, MODEL-002, MODEL-005, ADR-C2, SAFE-003, ADR-C3,
LIN-006, ADR-0011 · **Resolved** (was: blocks 3, hash-visible)

Until now the body was a bag of values with no internal references, so node
identity never had to be canonical. Any protection closure, and any faithful
representation of MODEL-002's layering, puts edges in the body and therefore
needs a `NodeId` that is stable across re-probes of unchanged hardware and
collision-free across simultaneously present devices.

Section 5 lists no such type and neither ADR-C2 nor ADR-C4 addresses naming.
The four obvious schemes each fail, as above. A resolution must state the
derivation, its stability guarantee, and its behaviour when two Weak-identity
devices are indistinguishable. That behavior must still produce a snapshot: it
may neither silently merge the devices nor refuse discovery for the whole host,
and any fail-closed limitation must attach to the affected target or targets.

ADR-0011 removes product deduplication from this issue; it does not remove the
collision family. SI-27 must define stable, collision-safe `NodeId` behavior for
simultaneously present records whose identity bytes are equal, whether their
individual strength is Weak or Strong, without silently merging two devices.
It must also type and name the platform-reported membership edge between a
recognized multipath node and its materialized members. The separate question
of how to fail closed when unassembled paths to one LUN present unequal
identifiers is SI-37, an input to SI-11; SI-27 must not infer sameness to close
that gap.

---

# Part 6 — Aggregation, protection, and naming, round three

Round three answered all six remaining blockers in one design. **Four resolved,
proposed as ADR-C5.** Two did not. Five adversarial lenses raised fifty-eight
objections; the author refuted none of them, and the final review upheld the
fatal findings while correcting four objections that were wrong in their
reasoning or their worked example. Recorded so round four starts from the
objection.

The most durable finding of the round is an attack result rather than a design
result, and it governs both open issues:

> **Fail-closed-by-unencodability is not fail-closed.** Every collision in the
> submitted naming scheme surfaced as an encoder error, and an encoder error
> produces no artifact — nothing to display, nothing to journal, no
> `Indeterminate` verdict, and no SAFE-005 refusal, because a refusal must be
> encodable to be issued. On a whole-host snapshot that turns a two-device
> ambiguity into a denial of discovery for every device on the machine,
> reachable by an unprivileged party with a cheap USB stick, and it defeats
> INV-008 in the strongest possible way. **Any scheme whose collision behaviour
> is "the codec refuses" is wrong by construction.**

## SI-11, protection strength for non-goals (round three)

**Direction that survived.**

- **The three-regime mapping**, which is the first attempt to satisfy what Part 4
  asked for. Regime A (Section 2.1 non-goal) → `unsupported` with an
  out-of-scope reason; Regime A′ (SAFE-005 ambiguity) → `blocked`; Regime B
  (PART-014) → **status unchanged**, reason attached; Regime C (Recovery-only,
  missing tool, dirty volume) → `blocked`. MAC-009 stays intact and nothing is
  removed from PART-014, so round one's error is not repeated: on Apple Silicon,
  operations macOS permits only in Recovery report `blocked` with that reason,
  which is MAC-009's literal requirement.
- **The rejection of Regime B → `blocked`.** CAP-001 computes capability per
  exact target with no plan in scope, and CAP-005 requires GUI, CLI, and planner
  served from one engine, so a status that flips on whether a plan happens to
  exist is not a property of the target. The alternative reports `blocked` for
  shrinking `C:` on every Windows machine and fails ACC-001.
- **No fifth CAP-003 value.** CAP-003 already mandates "plus a reason and
  remediation", so a closed reason enum is the specified home. A fifth status
  would touch CAP-005, CAP-007, ACC-009, PLAN-009, FS-007, SAFE-004, IMG-011, and
  MAC-009.
- **A three-valued verdict** — permitted, refused, indeterminate — carrying
  ADR-C3's three-valued table state and ADR-C4's positively-observed-absence rule
  one layer up. A binary verdict encodes an EIO reading a ZFS uberblock as "not a
  member", on the failing-disk population the product exists for.
- **Three edge kinds, not one**, which is the categorical fix for round two's
  sibling-capture failure rather than a special case: containment (positional
  nesting in one addressable byte space), backing (evidence → consumer), and
  production (producer → product). The failures below are in the closure over
  these edges, not in the edge taxonomy.
- **Copy-as-source as a shape property rather than a per-node policy**, which
  generalizes the WIN-004 allowance beyond LDM.

**Why the mechanism was not accepted.** Six defects, two of which destroy data
with both defence layers passing.

1. **The closure has no downward dependency rule.** On the documented
   root-on-ZFS-over-LUKS layout — `luksFormat /dev/sdb1`, `cryptsetup open`,
   `zpool create tank /dev/mapper/cryptzfs` — deleting `sdb1` reaches the LUKS
   signature and the encryption layer (permitted under LIN-003) and stops. With
   no downward production rule the pool below is unreachable, the worst verdict
   is permitted, the plan constructs, **the helper recomputes the identical
   verdict, and the body hashes match.** A Section 2.1 MUST NOT is violated with
   no override, no confirmation, and no mention of the pool in PLAN-004
   consequence text or UI-005. The absorption lemma is the statement of the bug:
   "every closure path terminates at a producer-root" is the claim that the model
   never inspects what a producer produces. Two further instances: `vgremove` on
   a VG holding a ZFS-backed LV, and `mdadm --zero-superblock` on an array
   carrying a pool.
2. **The residual arm defaulted to permitted, which is fail-open.** An unreadable
   GPT yields an indeterminate table state with nothing enumerable beneath it and
   no matching arm, so initializing a live vdev member computes permitted. That
   reproduces, one layer up, the exact collapse ADR-C3 and ADR-C4 were written to
   forbid — a known-unknown treated as a negative — inside the lattice built to
   prevent it. An unrecognized file system falls to the same wildcard,
   contradicting SAFE-005's first clause verbatim, and Section 0.2 makes SAFE-005
   override this design. The transport enum listed only iSCSI, NBD, and Ceph RBD
   while Section 2.1 says "etc.", so an NVMe-over-TCP namespace — an ordinary
   `/dev/nvme1n1` on any kernel since 5.0 — was fully mutable.
3. **Round two's sibling capture was re-derived**, found independently by three
   lenses, through the clause putting a created node's parent in the directly
   affected set. Creating a partition on the design's own root-on-ZFS example
   puts the whole device in that set and descends to the pool member, so ACC-003
   is `unsupported` on every root-on-ZFS machine, every Fusion Mac with free
   space, and every GPT dynamic disk. The stated non-interference theorem is
   false as proved: it discharges the containment case with "a step that
   byte-covers or destroy-targets the whole device", which does not cover the
   parent-of-created clause written three lines above it. Worked example 3
   asserts an affected set its own rule contradicts.
4. **A device-scope refusal is unreachable from below.** With no upward
   containment rule, `mkfs.ext4 /dev/sdb1` on an iSCSI LUN never reaches the
   device's transport, so Section 2.1's network-block-device prohibition binds
   only when the whole device is the step target.
5. **The regime mapping is missing a CAP-002 dimension.** Capability is computed
   with no plan in scope, so a refused node makes `copy`, `read`, and `detect`
   report `unsupported` — advertising WIN-004's only sanctioned escape from a
   dynamic disk as not implemented, and reporting `unsupported` for `detect` on a
   Storage Spaces pool against WIN-003 and Section 2.1's preamble. Planner and
   capability engine then disagree on one target/operation pair, which CAP-005
   forbids. Separately, an ambiguous-identity refusal routed to Regime A, so two
   indistinguishable USB sticks report `unsupported` with a reason naming no
   technology and citing no Section 2.1 clause.
6. **The type-level guarantee does not cover the closure.** The proposed
   constructor receives neither the written ranges nor the structural effects, so
   it can consult only the target's own verdict — and the design states that
   verdict is permitted for the ZFS member's host partition. For the case SI-11
   is actually about, a permitted target whose closure reaches a refused node, the
   submitted answer is "rejected at runtime", which is the option SI-11 says does
   not survive a bug in the guard.

Also rejected: an unconditionally refused orphan signature. ZFS writes label
pairs at both ends of a vdev and ordinary repurposing clears only the leading
pair, so a bench-tested disk pulled from a pool would be `unsupported` for
initialization, DIA-004 sanitize, and ACC-010 forever, with the only stated
escape being an unsupervised `zpool labelclear` outside the product — the exact
hazard the product exists to prevent. And a locked encryption layer was permitted
in every state, so a BitLocker- or LUKS-locked container holding a pool is
destroyed at `supported`, severity 4, while FS-010 and REC-011 fire and imply the
contents were considered.

**What the next attempt needs.**

1. **A closure that reaches a Section 2.1 object *below* the node being written,
   with a proof it does not re-derive sibling capture.** The candidate to start
   from — hand-checked by the final review against every layout in the submission
   and deliberately *not* accepted — is a second fixpoint over substrate
   destruction, closed under downward containment, upward backing, and a new
   downward production rule restricted to substrate destruction. Non-interference
   appears to hold because that rule's products are virtual devices or volumes
   and never a physical device, so `lvresize lv_home` cannot reach `lv_root` (a
   volume is not a producer) and Btrfs `delete(sdc1)` cannot reach `sdb1` (a file
   system is not a producer). **This must be proved as a property test alongside
   the sibling theorem, not asserted.** The open parameter is which effect classes
   count as substrate destruction; that single choice decides over- versus
   under-refusal and is currently undefined.
2. **An inverted default.** Enumerate permitted explicitly per kind and variant
   and make the residual arm indeterminate. Add arms for an indeterminate
   partition table, an unrecognized file system, and a locked encryption layer,
   and read the consumer edge in the ZFS, Storage Spaces, LDM, and CoreStorage
   arms as the APFS arm already does, so an orphan is indeterminate (`blocked`,
   remediable) rather than refused (`unsupported`, no next step). Then settle in
   the ADR, not in an adapter, which table-repair operations are permitted
   against an indeterminate table and on what acknowledgment — REC-001 and
   REC-004 exist for exactly that disk.
3. **A directly-affected set that targets table writes at the table node.** Give
   the partition table explicit extents (primary GPT, backup GPT, MBR sector,
   each EBR) and replace "the parent of each created node" with "the partition
   table of that parent, plus the free extents consumed". Re-prove the theorem
   against a create-in-free-space step and a delete step, with a property test
   generating steps that write only the table region. Regressions in both
   directions on the root-on-ZFS fixture: creating a BIOS boot partition on `sda`
   must construct; initializing `sda` must not.
4. **Node-local inheritance for device-scope refusals, not a closure rule.** A
   node's verdict is the worst of its own predicate and its root device's
   device-scope refusal. This covers transport, device read-only, and whole-device
   refusals, and cannot re-derive sibling capture, because a partition inherits
   only its own device's device-scope refusal — an ESP on an iSCSI LUN is
   correctly refused while an ESP on a SATA disk carrying a ZFS member is not,
   since that refusal lives on the backing signature.
5. **A CAP-002 operation-class dimension.** A refused verdict suppresses only the
   mutating operations and never `detect`, `read`, or `copy`-as-source. Add a
   source-access predicate with an enforced read-only mode the helper verifies: a
   refused or indeterminate node may be a source operand only if **every** step
   touching it is a source step — FS-005's health check on a dirty-log NTFS
   inside an LDM partition otherwise either writes to an LDM volume or kills
   WIN-004. Route ambiguous identity to Regime A′.
6. **The guarantee at the step level.** The only constructor for a mutating plan
   step takes the snapshot body plus written ranges, effect class, target, and
   structural effects, runs the closure, and returns a result. State that decoding
   re-runs the closure, since the promised negative-deserialization test depends
   on it, and cost the TypeScript mirror explicitly, because the GUI decodes plans
   and MODEL-005 requires byte-identical agreement. The alternative — dropping the
   affected set from the body and re-deriving it helper-side — is cheaper and
   costs the authentication of that set; it must be argued on its own terms rather
   than assumed away.
7. **The verdict's place in the body argued as a named exception, not a rule
   change.** See ADR-C5's rejection of the MODEL-005 amendment. The genuine
   tension is resolved the way ADR-C2 resolved CONC-004's transitional marking: a
   narrow, named over-inclusion defended on ADR-C2's own terms.
8. **A CONC-001 bind set defined independently**, as the transitive closure of the
   affected set under reverse backing, reverse production, **and ancestor
   containment**, down to and including every device node. As written the bind set
   traverses no containment edges, so shrinking `sda1` binds no physical device
   and two plans rewrite one GPT concurrently — a lost-update path CONC-005's
   exactly-one-wins never sees.
9. **The extent accessor made total and normative.** Restrict its domain to the
   extent-bearing kinds and state that the extent clause is vacuous elsewhere;
   qualify every range with its host so a multi-device step such as `btrfs
   balance` is unambiguous; declare one address space per containment-forest root
   so an MBR logical's offset is not device-absolute to one adapter and
   parent-relative to another. Write the GPT reserved-region, minimum-gap, and
   alignment rules down, or remove free extents from the hashed body entirely —
   they carry no verdict input, and PART-009 and PART-012 compute placement from
   the planner's own policy.
10. **The `RecoveryRequired` interaction answered, not deferred.** Section 8 makes
    that state unbounded, so a ruleset change can strand a machine mid-move with
    its PLAN-008 reversal plan invalidated and REC-010's advertised rollback
    evaporated. **The cost is intrinsic to freezing derived verdicts, not to the
    ruleset-version field** — the body hash of unchanged hardware moves whether or
    not a version field exists, so relocating the field to the envelope buys
    nothing. Define an update-time re-derivation route coordinated with SI-20,
    SI-21, and SI-22, and do not evaluate an in-flight plan under the ruleset it
    declares — that is the CAP-007 downgrade-by-assertion hazard the design itself
    cites elsewhere. HLP-002, CAP-007, and SAFE-008 already forbid it; restate the
    prohibition at the point of implementation pressure, with the negative test.
11. **The PART-014 class function written out**, exhaustively over all nine
    enumerated kinds, sourced from live helper-side discovery including mount
    path, partition flags, and label. Regime B never enters a verdict, so the
    class need not be body content — which is also what removes mounts and the
    active-swap flag from the hashed body. A class list that cannot name each
    PART-014 item one-for-one is not a resolution of SI-11; the submitted one
    cannot identify Debian's `/boot`, which carries the generic Linux filesystem
    GUID.

## SI-27, the hashed body has no node-naming rule (round three)

**Direction that survived.**

- **The decomposition, which dissolves round two's dilemma.** Round two searched
  for one name that was simultaneously an intra-artifact reference and a device
  identity, which is why every candidate failed. They are two objects. SAFE-003's
  identity record already exists, already lives in the plan body, and already has
  a helper-side match verdict with per-field replug tolerance under ADR-C3. A node
  identifier is a **document-local address** whose only job is to let edges in one
  body reference nodes in that same body. No objection attacked this framing;
  every objection attacked a specific derivation input.
- **Derived, never stored.** The decoder recomputes each identifier from the
  node's own naming fields and rejects any edge naming an unknown referent, so a
  declared-versus-derived disagreement is unrepresentable — there is nothing
  declared for an attacker to pick.
- **Naming by position relative to already-named nodes**, which **falsifies round
  two's claim that no content-derived scheme can name a node the plan creates.**
  A new encryption layer is named from its backing partition, a new file system
  from the mapper device, a created partition from its device and declared start
  offset, and an initialized table from the parent alone so a minted disk GUID
  enters no name. The residue belongs to SI-19, which is amended accordingly.
- **Two schema identifiers** for captured and simulated topologies, so
  canonical-encoding §5 domain separation makes a simulated topology structurally
  incapable of being accepted where PLAN-006 requires a captured one.
- **The exclusion list and its reasoning**, which survive even where the positive
  scheme fails: connection path and OS instance id (SAFE-003, Section 16),
  positional indices and counters (Part 3, INV-009), partition-table state and
  checksum (PART-001 and PART-013 change it), length (UI-004 must diff a shrink),
  regenerable identifiers (PART-016, FS-008), partition type (PART-008), and
  adapter-formatted text. **"Divergence is worse than collision — collision fails
  closed, divergence produces a check that can never pass"** is right, and the
  design's own unnormalized-serial defect is an instance of it.

**Why it was not accepted.** Beyond the unencodability finding above:

1. **The collision enumeration is false.** Four families exist where two were
   claimed: multipath (one LUN as two paths — one device seen twice, the opposite
   of two ambiguous devices, and needing the opposite treatment); **every virtual
   device**, whose naming map has no discriminant beyond technology and size with
   a null source for loop devices, VHDX, and attached images; stale signatures
   (mdraid 1.2 alongside a surviving 0.90 superblock, a ZFS tail label pair, two
   file-system signatures on one reformatted partition); and table entries (Boot
   Camp hybrid MBR aliasing one offset in two views, a corrupt or fuzzed GPT,
   REC-003's conflicting recovery candidates). The proposed fold is defined over
   the device projection and reaches none of them.
2. **The design cannot be tested by its own test plan.** Section 11.3 makes
   synthetic images and loop devices the T1/T2 fixture medium, and the submitted
   verification plan calls for a four-member mdraid RAID10, a two-device Btrfs,
   and a two-store APFS container. Those are equal-size loop devices; they
   collide, and virtual devices have no fold.
3. **The fold is not closed under the naming relation.** File systems,
   aggregates, encryption layers, and volumes are named *from* their backings
   rather than nested under them, so a consumer whose backings are folded has no
   encoding — and worked example 5 commits the error in writing. Closing the fold
   means a serialled NVMe becomes unplannable because two blank USB sticks were
   attached.
4. **Naming an aggregate from its smallest member makes the name a function of
   the observed member set**, and is withdrawn. An EIO on one PV label region
   moves the minimum and renames the aggregate, every volume, every file system,
   and every mount — on the failing-disk population the product exists for,
   invisible to the verdict machinery because the aggregate is permitted in both
   probes. LIN-005 member replacement and `vgextend` do the same deliberately.
   The stated justification was also factually wrong: LVM2's VG name and id live
   in the text metadata area on every PV, not only in the label sector, so a
   native designator is available for the one technology used to argue the
   fallback was necessary.
5. **Parent-plus-offset is not injective for a partition**, and the injectivity
   proof misreads Section 11.2, which lists properties tests must prove about the
   plans the product *produces*, not a guarantee about hardware presented to
   discovery. INV-003 requires detecting hybrid and inconsistent tables; REC-003
   requires previewing conflicting candidates. Host-plus-technology is likewise
   not injective for a backing signature, and the file-system naming map omits
   the kind entirely.
6. **Serial and WWN are unnormalized text at the root of the recursion**, which
   is the design's own anti-formatted-text rule not applied one line above where
   it was stated. `naa.…`, `0x…`, and bare uppercase are three values for one
   drive; a non-UTF-8 serial cannot be encoded at all; and a one-character case
   difference renames every node in the body.
7. **Collision detection is a validation-layer property, not a codec property.**
   `decode` returns a schema-agnostic value with a codec-level error set, and the
   key comparator compares adjacent text keys during the streaming scan.
   Verifying that nodes are strictly increasing by identifier requires a
   schema-aware second pass. Since HLP-001 applies by hash and SEC-001 authorizes
   exact hashes, a body with duplicate identifiers can be hashed and circulated
   before any check runs.
8. **Names depend on other nodes.** The duplicate-designator rule re-designates an
   aggregate that was uniquely named before a collision, so attaching an ordinary
   backup clone renames a dozen nodes on the internal disk.
9. **Physical block size is in the device name**, so a 512e drive moved between a
   SATA port and a USB bridge renames — falsifying the design's own claim that
   transport is naming-invisible, and failing a T3 hardware-matrix case that
   already exists.

**What the next attempt needs.**

1. **SI-12 was the first prerequisite and resolved on 2026-08-02.** ADR-0011
   removes product deduplication from this round: platform-recognized multipath
   membership is detection-only, while an unassembled equal-identifier pair is
   SAFE-005 ambiguity. SI-27 still owns collision-safe names and the membership
   edge. The unequal-identifier unassembled residual discovered by the
   post-acceptance review is now SI-37, an input to SI-11 rather than a naming
   inference SI-27 is allowed to make.
2. **A naming input for virtual devices that is the backing object's identity**,
   which requires a **fourth edge kind** for host-backed virtual devices (loop,
   dm-linear, plain dm-crypt, VHD/VHDX, attached images) — none of which has an
   on-disk signature, and therefore none of which has any legal edge to whatever
   holds its bytes today. That edge also needs a file or byte-range-within-file-
   system node that exists neither in the proposed kinds nor in Section 5, so it
   is a spec addition. It additionally fixes CONC-001, whose bind set is currently
   *empty* for a loop device — so a plan imaging `/dev/sda` and a plan writing a
   table to a loop device backed by a file on `sda1` execute concurrently against
   one disk, invisible to the T1/T2 tiers because that is the population those
   tiers run on. **Both new edge kinds break the typing rule that no backing or
   production edge targets a physical device, which is the sole premise of the
   no-sibling-capture theorem. The theorem must be re-proved under the new edge
   set, not patched.**
3. **A collision behaviour that produces an artifact.** Every candidate must
   answer: what does the body contain when two nodes of one kind collide? Add a
   fuzz target asserting that **no on-disk byte sequence can make snapshot
   encoding fail.** Treat the proposed fold as rejected rather than repairable: it
   is not closed under the naming relation, it makes a node's name depend on other
   nodes, and it does not reach three of the four collision families.
4. **Repairs already established, which round four may assume**: a backing
   signature gains its primary signature offset; a file system gains its kind and
   primary superblock offset (both invariant under `btrfs device add`, so ADR-C5's
   SI-08 cardinality argument is untouched); serial and WWN become bytes with a
   normative, versioned per-platform canonicalization; physical block size leaves
   the naming map and stays a compared body field; the duplicate-designator case
   sets a flag without re-designating.
5. **Partition naming under a corrupt or hybrid table.** The remaining candidate —
   a distinct node kind holding conflicting entries verbatim, marked
   indeterminate, plus an explicit statement that a body carrying such evidence is
   not a planning base — satisfies INV-008 and gives REC-003 a home, and has had
   no adversarial review. Add a role discriminant to the partition table,
   re-parent partitions onto the table rather than the device, and give the
   partition type a form able to carry APM's 32-byte ASCII type string, which the
   proposed enum cannot express despite listing APM as a table kind.
6. **The array comparator, stated normatively.** Filed as SI-31.
7. **The validation pass named explicitly** as the sole decode boundary per
   schema, with its own error type distinct from the `pce/1` error enum, and
   encoder-side symmetry per canonical-encoding §6.1.
8. **A stated property, with a property test: a node's identifier depends only on
   that node and its ancestors, never on the presence of other nodes.** Round
   three violates this in two places and declares neither.

## Preconditions on round four, in either issue

These are gates, not follow-ups. Round two was rejected for building on an
unverified platform claim, and round three did it again — its own known-weakness
list recorded the designator table as untested and the design built on it anyway.

1. **A per-platform observability record, established empirically and
   non-elevated, before any ADR freezes bytes.** Started in
   `docs/quality/observability.md`; **Windows has hardware rows and SI-35's
   Windows category is now complete; Linux has real-hardware rows from the
   2026-08-04 partitioned-media matrix and a qualifying descriptor-bound loop
   record; and macOS has client rows from the 2026-08-05 increment 6 matrix,
   valid on its second sitting. All three platforms now have the non-elevated
   record this precondition asks for.** That is what this precondition
   requires and no more: it is satisfied by the *non-elevated* record, and the
   privileged comparison leg falls outside what it asks for either way.
   **M10 was subsequently taken on 2026-08-05** and is no longer the open leg
   this sentence once described. Its second-reader readback was discharged
   2026-08-08 by an independent reader session, every digest matching, so
   the custody hold this paragraph placed on register decisions is lifted;
   the discharge and its caveats are recorded in
   `docs/quality/observability.md`. (Round three
   proposed `docs/capabilities/`; it lives under `docs/quality/` instead,
   because `docs/capabilities/` is where DOC-003's generated matrix belongs and
   Section 11.7 forbids hand-editing that.)

   Measured so far, on both Windows and Debian: an unprivileged client **cannot**
   read raw partition-table sectors, and on Linux it cannot probe a device for
   signatures either (`blkid -p` is denied). What it *can* read on both platforms
   is the kernel's own view — on Windows the complete partition list with each
   partition's offset, size, type and GUID; on Linux `/proc/partitions`, sysfs
   geometry, and the world-readable udev database carrying serial, WWN, bus and
   path. **Part 5's universal roster-identity conclusion is refuted within one
   named finite projection, measured rather than argued.** WP-020's regular-file
   fixture carries a live ext4 superblock at `0x438` and a stale mdraid 0.90
   superblock in the last 64 KiB-aligned block — the end-of-device metadata that
   start-of-device formatting never reaches. On those bytes, `wipefs -n` reports
   **both** signatures while `blkid -p -o udev`, the form udev's builtin uses,
   reports exactly one: `linux_raid_member`. **`ID_FS_AMBIVALENT` did not fire**;
   the single answer is the *stale* signature, not the live file system.

   That finite interface asymmetry is not roster identity, and signature
   presence can feed a protection verdict. It does not by itself establish an
   actual cross-privilege difference between complete client and helper graphs,
   or a different final verdict. Round four must therefore establish rather
   than assume client-and-helper signature agreement. (One earlier finding
   survives and narrows the
   collision families: a partition reformatted by a *current* tool does not
   retain its old file-system signature, because `mkfs` and `mkswap` erase
   competing ones.)

   The Windows measurement already settles one thing and forces an amendment. A
   non-elevated client **cannot** read raw partition-table sectors
   (`ERROR_ACCESS_DENIED` on a read-only physical-drive handle) but **can** read
   the table's entire logical content — disk GUID, partition style, and every
   partition's offset, size, type, and GUID — plus serial, unique id, both sector
   sizes, bus type, and Storage Spaces pool membership.

   So **ADR-C3 needs an amendment stating what `Present { checksum }` is computed
   over.** Over raw sectors, every unprivileged Windows record is
   `Indeterminate` and therefore Weak, which makes UI-009 typed confirmation
   universal and unattended apply refused everywhere — and the helper *can* read
   the sectors, so the two sides disagree on a body field for unchanged hardware,
   which is the PLAN-006 failure ADR-C2 exists to prevent. ADR-C3 says "a table
   was read and hashed" without fixing *what* was read, and a checksum over the
   kernel-exposed table content still serves SAFE-003's replug clause, whose
   purpose is detecting a table rewrite. The choice is hash-visible.

   Still needed: mdraid superblock, LUKS2 header, ZFS label, LVM2 PV label and
   metadata area, and APFS container superblock, on macOS and Linux. **The
   projection is a clamping obligation on the client, not only a discard
   obligation on the helper** — `/dev/sda` is `brw-rw---- root:disk` on stock
   Debian, Ubuntu, and Fedora, so otherwise a user in the `disk` group and a user
   outside it produce different bodies on one host with one build. Round four
   must not begin believing Linux identity is universally Weak.
2. **A per-technology native designator table**, established before naming is
   frozen: LVM2 VG id from the PV metadata area, mdraid array UUID, APFS container
   UUID, ZFS pool GUID, LUKS UUID, Storage Spaces pool object id, LDM group GUID.
   With member-derived naming withdrawn this is load-bearing rather than an
   optimization: where no member-independent designator is readable, the aggregate
   is indeterminate and is not a plan operand.
3. **`probe_tag` defined normatively** — which prober, which offset, which magic
   bytes. ADR-C5 makes it load-bearing in eight enums and a naming input.
4. **Preserved-unknown structure needs its two budgets separated.** Cap depth at a
   small constant for the stack-safety property, and set the size cap against real
   metadata: LUKS2's default metadata area is 16 KiB per copy — a 4 KiB binary
   header plus a 12 KiB JSON area — so a 4 KiB cap cannot hold an ordinary
   `luksFormat --type luks2` header before anyone crafts anything, and LIN-003
   makes LUKS2 first-class. State the over-cap outcome normatively so INV-008
   stays auditable, and version the SAFE-006 redaction rule so a redaction change
   is a visible body-hash event.

## Objections that were raised and did *not* survive review

Recorded so round four does not chase them.

- **NVMe multi-namespace collision — refuted.** `/sys/block/nvme0nX/wwid` reports
  a per-namespace NGUID or EUI-64, so the identifier is present and distinct.
  Multipath and loop devices carry that objection; the NVMe sub-case does not.
- **The array-comparator worked example — corrected, conclusion upheld.** The
  originally offered byte pair does not invert under the two candidate orderings.
  The inverting pair is recorded in SI-31 and was verified against this
  repository's own encoder.
- **"Nothing forbids the helper evaluating a client-declared ruleset version" —
  downgraded.** HLP-002 makes client output "an untrusted hint, never an input to
  authorization", CAP-007 says a client cannot upgrade a capability by asserting
  it, and SAFE-008 ships schemas compiled into the helper. The spec already
  forbids the shortcut; the design merely failed to restate it where the
  implementation pressure appears.
- **"PART-014's Linux boot class is untestable" — under-enforcement upheld,
  untestability refuted.** The class cannot be computed from the declared inputs,
  but Regime B leaves CAP-003 status unchanged and produces only a reason, so the
  class never enters a verdict and need not be body content at all.

---

# Part 7 — Identity attribution for separable media, SI-28 round four

**Not accepted. SI-28 remains open and still blocks WP-010 increment 3.**

Five lenses raised twenty-two objections against a design that recommended a new
body-resident attribution axis. The final review upheld four fatal findings and
added one empirical result of its own.

The governing finding reframes the issue rather than failing a mechanism:

> **SI-28 is not a classification problem, and no classification change can
> resolve it.** SAFE-003's weak-identity policy does not discriminate the two
> cards. Typed device-name confirmation (UI-009) displays the *reader's* name for
> both; the immediate pre-apply re-probe returns a byte-identical record; and the
> replug allowance is not the vector, because the reader never leaves the port.
> Only the unattended-apply refusal bites — and SAFE-003 gives that refusal a
> first-class escape hatch ("unless the plan carries an explicit weak-identity
> override recorded at plan creation") which the dominant real workflow, batch
> card provisioning, records once in a template. **A protection that every
> affected user must switch off to do the thing they came to do is not a
> protection.**

Reclassifying the record from Strong to Weak — which is what all three filed
options do, by different routes — therefore leaves the destruction path open.

## Direction that survived

- **The honesty correction.** ADR-C3's Strong definition inherits an unstated
  assumption from SAFE-003: that a stable hardware identifier identifies the
  medium. That is false for the population SAFE-003 was written about. Saying so
  costs nothing and is not in dispute.
- **Attribution is not provenance.** ADR-C4's observation set answers "which
  adapter reported this"; attribution answers "which component the value
  denotes". `observed / unavailable / failed` has no slot for the second. This
  survives the rejection of the option it was written to defend.
- **A function's arguments must live where the function does.** ADR-C3 makes
  strength computable from one record because "it asks only what the record
  contains", so an input to a body value cannot be envelope content. The
  *evidence* half is different: which interfaces were consulted and what each
  returned is exactly an ADR-C4 observation and belongs in the envelope, since
  MODEL-004 already requires "the method used" to be recorded there.
- **The continuity witness**, now filed as SI-33. It is the only proposal that
  discriminates two media whose recorded fields are equal.

## Why the mechanism was not accepted

1. **The one claimed positive capability was never demonstrated, and is
   circular.** The design asserted that Windows can prove a medium separable by
   comparing the storage stack's reported serial against the parent USB node's.
   That comparison was never performed. Re-run non-elevated on the development
   host: `MSFT_PhysicalDisk` returns two rows, both `BusType=17` (NVMe), with
   **zero** rows in `BusType {7, 12, 13}`, and both cited PnP nodes report
   `Present=False`. There was no storage-reported serial to compare against. What
   the design would have compared is a PnP device instance ID that USBSTOR
   *composes from* the USB descriptor's `iSerialNumber` — so the match is
   manufactured by the enumerator, not observed. Strip it and the proposal
   reduces on Windows to "everything behind a bridge is Indeterminate", which is
   the blunt option plus a hash-visible body field.
2. **The observation surface is a property of the observer, not the device, so it
   cannot be body content.** Either it records what the adapter actually called —
   and the unprivileged client (udev database, CIM) and the privileged helper
   (VPD, SG_IO) then write different bodies for one device on one host, so
   PLAN-006 can never pass — or it is clamped to a constant per platform and
   transport, carrying no information beyond transport class, at which point the
   axis degenerates to the blunt option. `docs/quality/observability.md` states
   this constraint in terms, and the design did not test itself against it. The
   second horn is sharper than the objections claimed: the measured NVMe device
   reports `UniqueIdFormat=8` (SCSI Name String) with an `eui.` prefix and
   enumerates under a SCSI device path, because Windows synthesizes SCSI identity
   for NVMe through `stornvme`. So "which interface the adapter called" is one
   observation where the design's enum has three.
3. **The identifier being annotated is itself unspecified.** One NVMe device
   exposes four distinct identifier strings from a single unprivileged CIM class —
   `SerialNumber` (20 chars), `UniqueId` (20, `eui.`-prefixed),
   `AdapterSerialNumber` (25), and `FruId` (15) — with `SerialNumber` equal to
   neither `UniqueId` nor `AdapterSerialNumber`. Nothing normative says which is
   *the* stable hardware identifier, or how to canonicalize it. The proposal
   attaches a second unspecified field to each element of a set whose elements are
   themselves unspecified.
4. **The gate on freezing identity bytes is unmet.** `docs/quality/observability.md`
   states that an entry marked not established MUST NOT be relied on by an ADR
   that freezes canonical bytes. macOS is entirely unestablished and Linux only
   partly so, and the proposal's only positive case for removable media rests on
   `mmcblk*/device/cid`, on a platform where that has not been measured.

## What the next attempt needs

1. **Take the measurements before designing.** No attribution row may be frozen
   before established Linux and macOS rows exist in
   `docs/quality/observability.md`. Name IOKit keys for macOS, not
   `system_profiler` output — Section 16 forbids parsing formatted output when an
   API exists.
2. **Design over device properties, never over observer properties.** Any
   body-resident input must be something the client and the helper both read to
   the same value by construction, and must not be an observation of a *sibling*
   device. Transport class is the only discriminant that clears this bar today
   (`MSFT_PhysicalDisk.BusType`, verified non-elevated, distinguishing 7, 12, 13
   and 17). If that is the honest floor, argue against the blunt option on cost
   rather than claiming capabilities that do not exist.
3. **State a normative observation order per platform**, as a clamping rule, or
   client and helper will not agree. This is a prerequisite for all identity work,
   not only for SI-28.
4. **Settle the identifier before annotating it.** Part 6 item 4's serial and WWN
   canonicalization is a hard prerequisite, and it must fix *which* identifier is
   bound and *which single API* supplies it per platform — not merely how to
   normalize whatever an adapter happened to fetch.
5. **Do not create a set-valued field to carry this.** SAFE-003 enumerates fixed
   identifier slots, so any per-identifier annotation is a fixed-text-key map,
   which §3 already orders. Making a live data-destruction defect wait on SI-31
   would be a dependency a design chose, not one it inherited.
6. **Amend SI-28's filing text** to the general predicate rather than the
   card-reader instance, so round five does not re-derive the classification
   framing that all three filed options share.

## What this round does not license

SI-28 must not be closed by any decision that does not carry a discriminating
mechanism. A classification change that leaves the destruction path open while
this register records the issue resolved is round three's absorption-lemma
failure in a new place. **A false claim of closure is worse than an admitted
gap.**

Nothing is implemented — `crates/domain/src` holds only the `pce/1` codec and
`packages/canonical/src` its mirror, no identity type exists in either language,
and ADR-C3 is unimplemented. The correction is still free, which is exactly why
freezing a wrong rule now is the expensive option and deferring is not.

## SI-36 May a test-fixture crate contain reviewed `unsafe`?

> **WITHDRAWN 2026-07-29, the same day it was filed.** The second follow-up
> audit was right and this filing was wrong. SAFE-009 says `unsafe` "is
> permitted **only** in adapter, FFI, and helper crates inside reviewed,
> documented modules." *Only* is a rule, not an enumeration: it forbids `unsafe`
> in `crates/fixtures` and simultaneously names the route — a separate, narrow
> platform-query adapter crate with a reviewed module, or a vetted safe
> dependency. There is no conflict to file.
>
> Reading the omission of `crates/fixtures` from *both* lists as ambiguity was
> using the §0.2 process to turn an implementation-location constraint into
> permission by omission. That is the opposite of what filing is for, and the
> audit's phrasing is the one to remember: *"Do not use the spec-issue process
> to turn an implementation-location constraint into permission by omission."*
>
> WP-020 precondition 3 therefore has a known route and stays open as ordinary
> work, not as a blocked decision. The text below is kept because the toolchain
> measurement in it is still accurate and useful — `number_of_links` really is
> unstable, so FFI or a safe dependency really is the only path.

**Requirements:** SAFE-009, SAFE-007, SAFE-001 · **Withdrawn; not a conflict**

Filed 2026-07-29 from an attempt to close WP-020's third increment-2
precondition, not from a reading. The precondition asks for the Windows
equivalent of the interlock's Unix `nlink` guard: a destructive target must be
reachable under exactly one name, because a second name is a second thing a
destructive suite could reach.

**Why it cannot be written.** Link count on Windows is available through
`GetFileInformationByHandle`. Rust's standard library wraps it as
`std::os::windows::fs::MetadataExt::number_of_links`, which is **unstable**
behind the `windows_by_handle` feature (rust-lang/rust#63010, still open),
verified against the pinned 1.96.0 toolchain:

```text
error[E0658]: use of unstable library feature `windows_by_handle`
```

`rust-toolchain.toml` pins a stable release and `docs/quality/fuzzing.md`
records nightly as a bounded exception for `fuzz/` alone, so the feature is out
of reach. The remaining route is FFI — `windows-sys` and an `unsafe` call.

**The conflict.** SAFE-009 lists where `unsafe` is forbidden ("the domain,
planner, validator, journal, and rpc crates") and where it is permitted ("adapter,
FFI, and helper crates inside reviewed, documented modules"). `crates/fixtures`
is in **neither** list. It is not a domain or planner crate, so the prohibition
does not name it; it is not an adapter, FFI, or helper crate either, so the
permission does not name it. The workspace lint `unsafe_code = "deny"` currently
resolves the ambiguity in the strict direction, which is the right default but
is a lint setting, not a decision.

This cannot be settled by reading harder, because SAFE-009's two lists are
enumerations rather than a rule. Reading it strictly ("permitted only where
named") forbids the check. Reading it purposively ("forbidden where hostile
input is parsed, permitted where platform truth must be obtained") allows it,
since the interlock parses no on-disk metadata — it hashes bytes and asks the OS
questions.

**Options, and what each costs.**

1. **Amend SAFE-009 to state the rule rather than the lists**, naming the
   property that matters (no `unsafe` in parsers of externally supplied bytes;
   `unsafe` permitted in reviewed platform-query modules anywhere). Closes this
   and every future instance. Cost: a normative change, and SAFE-009 is a safety
   requirement, so the amendment needs its own adversarial review.
2. **Add the check in a separate platform-query crate** that SAFE-009's
   "adapter" category plainly covers, and depend on it from `crates/fixtures`.
   Cost: a crate whose only purpose is to be in a category the specification
   already names — honest about the letter, and arguably a way of routing around
   the spirit of the question rather than answering it.
3. **Leave the Windows other-name check unimplemented** and record the residual.
   Cost: what is already recorded in `docs/work-packages/WP-020.md` — while an
   authorization is held the share mode refuses writes through any name, so the
   exposure is the window between generation and authorization, during which a
   hard-linked impostor must still carry the fixture's exact bytes. Narrow, and
   not nothing.

Option 3 is in force now because it is what the code does, **not** because it
was chosen. Nothing here proposes an answer.
