# SI-22 recommendation round — 2026-08-12

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-22 stays Later (WP-070) until a decision is recorded
through a WP-010 spec change with an ADR, the established shape. This is
an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-22, an early filing
with no options recorded. This round constructs the option space as well
as recommending from it. Two later decisions sharpened the filing:
ADR-0028 fed forward that the authorization act's journal record is
recovery-critical, and ADR-0027's disposal linkage created terminal
records that non-terminal applies reference.

---

## The conflict, made precise

> **JRN-004:** Journals live in an admin/root-protected, documented
> location per OS, with bounded size and the retention controls of
> SEC-009.

> **JRN-003:** Replay is idempotent. Recovery state derives solely from
> the journal plus fresh re-discovery, never from UI or client memory.

> **Section 8:** `RecoveryRequired` persists across restarts until the
> user acts — unbounded in time.

> **SI-22's filing:** Nothing exempts records belonging to a
> non-terminal plan, so retention can delete the records recovery needs
> and SAFE-005 then fails closed on a plan the product itself is holding
> open. How rotation preserves JRN-001's monotonic sequence and
> torn-tail semantics is also unstated.

The collision is bounded-versus-unbounded: retention must bound the
journal, recovery depends on it solely, and the state that depends on it
longest is unbounded in time by design. Post-ADR-0028 the stakes are
higher than the filing knew: a resume must trace to its authorization
act through an unbroken chain, so retention eating a live apply's
records now also destroys the authorization proof — the fail-closed
trap of the SI-16 shape, machinery fail-closed against its own purpose.

## Recommendation: liveness-scoped retention — retention governs terminal history; a non-terminal apply's records are exempt, and the live segment is bounded by budget, not by deletion

**Bounded and unbounded stop colliding when they stop sharing a
population.** Concretely:

1. **Retention MAY reclaim only records of terminal applies.** Records
   belonging to a non-terminal apply — Draft-through-suspended,
   `RecoveryRequired` included, the authorization act's record included
   (ADR-0028's fed-forward fact, absorbed) — are retention-exempt until
   their apply reaches `Completed`, `Failed`, or `Cancelled`. The
   unbounded-in-time state keeps its records for exactly as long as it
   exists, which is what "recovery state derives solely from the
   journal" requires and nothing less.
2. **The exemption closes over ADR-0027's linkage graph.** A terminal
   record referenced by a non-terminal apply's linkage (the
   Failed-original ↔ recovery-plan chain) is exempt until the
   referencing apply is terminal. Chains are finite, so the closure is
   bounded; once every member is terminal, the whole chain is ordinary
   history under retention.
3. **The live segment is bounded by budget, and budget exhaustion fails
   closed — never reclaims.** JRN-004's "bounded size" stays true
   universally through two different mechanisms: terminal history is
   bounded by SEC-009's retention controls, and the live segment is
   bounded by construction plus a per-apply journal budget — the count
   of concurrent applies is bounded by HLP-005's one-plan-per-device-
   set, each record is bounded by JRN-005's output bounds, and a
   pathological grower (a journaled retry loop) exhausts its budget as
   a **journaled failure** routing through SAFE-005's disable and
   Section 8's existing failure edges, never as a silent reclamation of
   the records recovery would then need.
4. **Reclamation is a declared act with a durable record, and the
   monotonic sequence survives it.** Sequence numbers are never reused
   or reset across rotation or compaction. A reclamation writes a
   durable **compaction record** stating the reclaimed range and its
   authority (the retention policy applied), so replay distinguishes
   policy from damage: a gap covered by a compaction record is
   legitimate history removal; a gap without one is corruption and
   fails closed. JRN-001's torn-tail rule governs the tail (incomplete
   writes); compaction records govern the head (policy); a mid-chain
   gap is neither and refuses.
5. **The execution journal and the audit log stay distinct.** The
   exemption governs the execution journal's recovery-critical records.
   The audit log (SEC-009, HLP-006) keeps its explicit user-controlled
   retention — the floor the product enforces is the exemption, and
   what users keep beyond it is policy, not correctness.
6. **ADR-0028's revisit condition is satisfied by this round**: the
   authorization record's retention (exempt while its apply lives)
   satisfies both that ADR's unbroken-chain obligation and JRN-004's
   bound, reconciled here before either ships machinery, exactly as
   that condition required.

## What a consumer and a plan may rely on

- No record a resume, roll-forward, or recovery replay depends on is
  ever reclaimed while its apply — or any apply whose linkage
  references it — is non-terminal. Retention cannot create the
  SAFE-005 trap; only damage can, and damage refuses as damage.
- Every journal gap is classified: compaction-covered (policy),
  torn-tail (incomplete write, truncated safely), or corruption
  (refuse). Nothing silent exists.
- The journal's size is bounded at all times: terminal history by
  retention, the live segment by the per-apply budget whose exhaustion
  is a journaled failure, never a deletion.
- Sequence numbers are monotonic across the journal's whole life,
  rotation and compaction included.

## The adversarial round

**Attack 1 — "'bounded by construction' is asserted, not proven: a
journaled retry loop grows a live apply without bound."** Sustained as
the round's sharpest finding and absorbed as point 3's budget: the
live segment's bound is enforced, not assumed, and the enforcement is
fail-closed in the right direction — exhaustion is a journaled failure
that stops the writer, never a reclamation that blinds the recoverer.
A plan that exhausts its journal budget was already in pathological
retry; surfacing it through the failure edges is the honest outcome.

**Attack 2 — "compaction records are new journal machinery — schema
creep decided in a register round."** Refuted by the register's own
precedent, three times over: semantics decided here, encoding landed
with JRN-006's schema under WP-070, jointly sequenced — exactly as the
SI-16 protection record and the SI-19 linkage encoding. A retention
design with no declared-reclamation record would leave gap-versus-
corruption undecidable, which is the filing's own rotation complaint.

**Attack 3 — "the linkage closure pins too much: a long chain of
recovery-of-recovery keeps a graveyard of Failed records alive."**
Refuted by arithmetic and honesty both: chains are finite (each link
is a disposal that required a new plan), the pinned set is exactly the
story a live recovery still needs, and the alternative — reclaiming
the original's terminal record while its recovery runs — deletes the
record that says *why* the running plan exists. Once the chain
terminates, all of it ages into ordinary history.

**Attack 4 — "SEC-009 lets the user set audit retention to zero and
delete history a dispute later needs."** Refuted by scope: the audit
log's retention is explicitly the user's (SEC-009 says so), and the
product's enforced floor is the exemption — correctness, not archival
policy. A compliance regime wanting more sets more; nothing here
narrows what SEC-009 already grants.

**Attack 5 — "the budget adds a new failure mode: plans now fail
because a journal filled."** Sustained as a real property and priced:
the failure is journaled, routed through existing edges, and strictly
better than either alternative — unbounded growth (violating JRN-004)
or silent live-record reclamation (the filed trap). The budget's size
is an implementation constant WP-070 tunes; the *shape* — exhaustion
fails closed toward the writer, never the recoverer — is the decided
part.

**Attack 6 — "this decides SI-23's backup-artifact protection
question."** Refuted by scope: the encryption-metadata backup artifact
(SI-23) is not a journal record, and nothing here touches its
protection ownership.

## Rejected, and why — to be recorded with the decision

- **(a) Retention wins uniformly: live records reclaimable under
  policy.** The filed trap ratified: the product deletes what its own
  open state depends on, then fails closed on it — SAFE-005 turned
  against the machinery it protects. Rejected without needing the
  adversarial round.
- **(b) Recovery wins absolutely: nothing reclaimable while any
  reference exists, transitively, forever.** Unbounded journal —
  JRN-004's bound becomes false — and terminal-history references
  (audit trails) would pin everything ever written.
- **(c) is the recommendation** — liveness-scoped retention, the
  linkage closure, the budget, the compaction record.
- **(d) Time-capped exemption: live records reclaimable after N
  days.** Re-creates the filed hazard on exactly the state Section 8
  makes unbounded in time; a `RecoveryRequired` older than N would
  lose its records while still demanding them.

## Deliberately not decided

The budget's magnitude and the compaction record's byte encoding
(JRN-006, WP-070's, jointly sequenced); audit-log retention policy
beyond the enforced floor (SEC-009's, the user's); SI-23; any
rotation file-layout mechanics (JRN-004's documented-location clause,
WP-070's implementation).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0029 is the next free number; reservation PR
before resolution PR, the established shape), amends **JRN-004 only** —
its sentence stands verbatim, gaining the liveness-scoped exemption,
the linkage closure, the per-apply budget with fail-closed exhaustion,
and the compaction-record rule with its gap classification — bumps
**minor** (12.7.0: additions; JRN-001, JRN-003, SEC-009, Section 8,
and SAFE-005 all stand verbatim and read naturally under the rule),
and moves SI-22 to Resolved. The major counter-argument is recorded
for the decision to overrule with. **No re-attribution PR follows** —
no WP-070 assignment exists; the ADR records the verification
obligations so that assignment's creation cannot omit them (the
ADR-0027/0028 precedent).

Verification obligations for the ADR, owned by WP-070 when it exists:

1. A retention pass over a journal holding a non-terminal apply
   reclaims nothing of that apply or its linkage closure — the
   exemption as a property, not a filter.
2. Budget exhaustion is a journaled failure through existing Section 8
   edges; no code path reclaims a live record, structurally.
3. Replay classifies every gap: compaction-covered proceeds,
   torn-tail truncates, anything else refuses — with the mid-chain-gap
   fixture as the named corruption case.
4. Sequence monotonicity holds across rotation and compaction — the
   property test extended over a compacted journal.
5. ADR-0028's chain-tracing test passes over a journal that has been
   compacted around the live apply — the two decisions' reconciliation
   as one fixture.
