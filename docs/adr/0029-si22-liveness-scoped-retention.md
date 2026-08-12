# ADR-0029: Liveness-scoped retention — retention governs terminal history; the live segment is bounded by budget, never by deletion

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-22_RECOMMENDATION_ROUND_2026-08-12.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.7.0 (minor under §0.1 — additions; JRN-004's sentence
  stands verbatim; argued in Decision, with the major counter-argument
  recorded)
- Work packages blocked: none newly — WP-070 does not exist as an
  assignment yet; this ADR records the obligations its creation must
  carry (the ADR-0027/0028 precedent)
- Requirement IDs: JRN-004 (amended); JRN-001, JRN-002, JRN-003,
  JRN-005, JRN-006, SEC-009, HLP-005, HLP-006, Section 8, SAFE-005,
  ADR-0027, ADR-0028 (read, none amended)
- Decision owners: Nate McBride

## Context

JRN-004 requires bounded journals with SEC-009's retention controls.
JRN-003 requires recovery state to derive solely from the journal plus
fresh re-discovery. Section 8 makes `RecoveryRequired` persist until
the user acts — unbounded in time. SI-22 filed the collision: nothing
exempted records belonging to a non-terminal plan, so retention could
delete the records recovery needs, and SAFE-005 would then fail closed
on a plan the product itself holds open — machinery fail-closed
against its own purpose, the SI-16 shape. The filing also noted that
how rotation preserves JRN-001's monotonic sequence and torn-tail
semantics was unstated.

Two later decisions sharpened the filing. ADR-0028 fed forward that
the authorization act's journal record is recovery-critical — a resume
must trace to its act through an unbroken chain — and named this
reconciliation as its own revisit condition. ADR-0027's disposal
linkage created terminal records that non-terminal applies reference.

## Safety analysis

**Bounded and unbounded stop colliding when they stop sharing a
population.**

**Retention MAY reclaim only records of terminal applies.** Records
belonging to a non-terminal apply — Draft through every suspension,
`RecoveryRequired` included, the authorization act's record included —
are retention-exempt until their apply reaches `Completed`, `Failed`,
or `Cancelled`. The unbounded-in-time state keeps its records for
exactly as long as it exists, which is what "recovery state derives
solely from the journal" requires and nothing less.

**The exemption closes over ADR-0027's linkage graph.** A terminal
record referenced by a non-terminal apply's linkage (the
Failed-original ↔ recovery-plan chain) is exempt until the referencing
apply is terminal. Chains are finite — each link is a disposal that
required a new plan — so the closure is bounded, and the pinned set is
exactly the story a live recovery still needs: reclaiming the
original's terminal record while its recovery runs would delete the
record that says why the running plan exists. Once the chain
terminates, all of it ages into ordinary history.

**The live segment is bounded by budget, and exhaustion fails closed —
never reclaims.** JRN-004's bound stays true universally through two
mechanisms. Terminal history is bounded by SEC-009's retention
controls. The live segment is bounded by construction — concurrent
applies bounded by HLP-005's one-plan-per-device-set, each record
bounded by JRN-005 — plus a **per-apply journal budget** that turns
the construction argument into an enforced property: a pathological
grower (a journaled retry loop under EXE-003) exhausts its budget as a
journaled failure routing through SAFE-005's disable and Section 8's
existing failure edges. The failure direction is the decided part:
exhaustion stops the writer honestly; it never blinds the recoverer by
reclaiming live records. The budget's magnitude is WP-070's constant
to tune.

**Reclamation is a declared act, and the sequence survives it.**
Sequence numbers are never reused or reset across rotation or
compaction. A reclamation writes a durable **compaction record**
stating the reclaimed range and its authority — the retention policy
applied — so replay classifies every gap: compaction-covered is
legitimate history removal; a torn tail is an incomplete write,
truncated safely under JRN-001's existing rule (which governs the tail
while compaction governs the head); any other gap is corruption and
refuses. Nothing silent exists, which answers the filing's rotation
complaint in terms.

**The execution journal and the audit log stay distinct.** The
exemption is the correctness floor the product enforces on the
execution journal. The audit log (SEC-009, HLP-006) keeps its explicit
user-controlled retention; a compliance regime wanting more archival
keeps more, and nothing here narrows what SEC-009 grants.

**ADR-0028's revisit condition is discharged.** The authorization
record's retention — exempt while its apply lives — satisfies both
that ADR's unbroken-chain obligation and JRN-004's bound, reconciled
here before either decision ships machinery, exactly as the condition
required.

**What a consumer and a plan may rely on:**

- No record a resume, roll-forward, or recovery replay depends on is
  ever reclaimed while its apply — or any apply whose linkage
  references it — is non-terminal. Retention cannot create the
  SAFE-005 trap; only damage can, and damage refuses as damage.
- Every journal gap is classified: policy, torn tail, or corruption.
- The journal's size is bounded at all times, by retention on terminal
  history and by the budget on the live segment.
- Sequence numbers are monotonic across the journal's whole life.

## Options considered

### Option (a) — retention wins uniformly: live records reclaimable under policy

Rejected without needing the adversarial round: the filed trap
ratified. The product deletes what its own open state depends on, then
fails closed on it — SAFE-005 turned against the machinery it
protects.

### Option (b) — recovery wins absolutely: nothing reclaimable while any reference exists, transitively, forever

Rejected: the journal's bound becomes false, and terminal-history
references (audit trails) would pin everything ever written.

### Option (c) — liveness-scoped retention with the linkage closure, the budget, and the compaction record (accepted)

Accepted, scoped as above.

### Option (d) — time-capped exemption: live records reclaimable after N days

Rejected: re-creates the filed hazard on exactly the state Section 8
makes unbounded in time — a `RecoveryRequired` older than N loses its
records while still demanding them.

## Decision

Option (c), landed as spec 12.7.0's amendment to JRN-004 and only
JRN-004. **SI-22 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** JRN-004's sentence
stands verbatim; the liveness scoping, linkage closure, budget rule,
and compaction-record rule are additions; JRN-001, JRN-003, SEC-009,
Section 8, and SAFE-005 are untouched and read naturally under the
rule. The counter-argument (the additions constrain how the existing
bound may be honored — the 3.1.0 caution) was weighed and is recorded
so the numbering is auditable; it was not taken because §0.1's rule
turns on what happens to existing requirement text, and none changes.

## Consequences

- **Positive.** The filed trap is unconstructible by design rather
  than avoided by discipline; the rotation question has a complete
  answer (every gap classified); the unbounded state and the bounded
  journal coexist without exception clauses; ADR-0028's reconciliation
  lands before machinery exists to get it wrong.
- **Negative, accepted knowingly.** A plan can now fail because its
  journal budget filled — journaled, routed through existing edges,
  and strictly better than unbounded growth or a blinded recoverer;
  the budget's magnitude is WP-070's to tune generously. And a stuck
  `RecoveryRequired` pins its records indefinitely — bounded per-apply
  and by device-set count, the honest price of Section 8's
  persistence.
- **For WP-070, when its assignment is created.** The verification
  obligations below are this ADR's record; the assignment's creation
  MUST import them. The compaction record's encoding and the budget's
  constant land with JRN-006's schema, jointly sequenced like the
  SI-16 protection record and the SI-19 linkage.
- Nothing here is hash-visible: no plan field, no state, no
  transition.

## Verification

Owned by WP-070 when it exists, recorded here so its assignment's
creation cannot omit them:

1. A retention pass over a journal holding a non-terminal apply
   reclaims nothing of that apply or its linkage closure — the
   exemption as a property, not a filter.
2. Budget exhaustion is a journaled failure through existing Section 8
   edges; no code path reclaims a live record, structurally.
3. Replay classifies every gap: compaction-covered proceeds, torn tail
   truncates, anything else refuses — with the mid-chain-gap fixture
   as the named corruption case.
4. Sequence monotonicity holds across rotation and compaction — the
   property test extended over a compacted journal.
5. ADR-0028's chain-tracing test passes over a journal compacted
   around the live apply — the two decisions' reconciliation as one
   fixture.

## Revisit conditions

- SI-23's resolution assigns the encryption-metadata backup artifact
  its protection owner; if that artifact's lifecycle ever routes
  through the journal, it inherits this rule or files its own round.
- JRN-006's schema lands: if the compaction record or the budget
  cannot be encoded as specified, the gap classification and the
  fail-closed exhaustion direction are the parts to keep.
- A journal architecture with more than one file per journal (segment
  files) arrives; the monotonic-sequence and gap-classification rules
  apply across segments, and a layout that cannot honor them is the
  part to redesign.
