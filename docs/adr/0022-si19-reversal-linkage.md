# ADR-0022: The reversal is an ordinary draft, linked by reference — `OperationPlan` is not recursive

- Status: Accepted
- Date: 2026-08-11. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-19_RECOMMENDATION_ROUND_2026-08-11.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.0.0 (major under §0.1 — PLAN-008's and Section 6's
  existing texts change meaning; argued in Decision)
- Work packages blocked: WP-060's PLAN-008 reversal increment (SI-19
  resolved; SI-15, SI-16, SI-17, SI-24 unchanged and still gating their
  own increments), with the linkage field's byte encoding a
  jointly-sequenced WP-060/WP-010 schema change when implemented
- Requirement IDs: PLAN-008, PLAN-002, PLAN-006, PLAN-007, PLAN-004,
  HLP-002, HLP-003, HLP-004, Section 6, REC-010, UI-005, ADR-0019,
  ADR-0021
- Decision owners: Nate McBride

## Context

PLAN-008 requires the planner to emit a reversal plan at planning time
(or a machine-readable per-step impossibility statement), feeding
REC-010's rollback advertisement. Section 6 requires every
`OperationPlan` body to carry a source topology snapshot hash, and
PLAN-006 requires the helper to reject a mismatch — but the topology a
reversal plan runs against does not exist until the forward plan has been
applied. SI-19 filed the conflict, sketching three postures: bind the
simulated final topology, emit unbound and re-plan after apply, or
exempt; and named what the answer decides — whether `OperationPlan` is
recursive. Naming round three amended the filing: most created nodes can
be named positionally from existing nodes, but a residue — volumes minted
inside an existing container (`newfs_apfs`), LVM snapshots — has no
position to be named from until it exists, and that residue is SI-19's.

**The filing predates spec 8.0.0, and 8.0.0 dissolved its core.** Since
ADR-0014's amendments, binding is a validation act for every plan: the
client's planning-time snapshot is a proposal, and the hash the
authorized plan carries is the one HLP-002's re-discovery produces at
validate-plan. A reversal plan emitted at planning time is exactly as
unbound as every other plan at planning time. What remained genuinely
open: carriage (recursion, reference, or nothing), the created-node
residue's spelling, and truth decay — a deletion that reverses "create
volume" is metadata-only at emission and destroys user data once
anything lands in that volume.

Two delivered facts constrain every answer, both held by committed
tests: `SnapshotKind::Simulated` can never be a planning base or satisfy
a PLAN-006 comparison, structurally (WP-060 increment 4, re-asserting
WP-010 3c); and `schemas/domain/plan-body.md` deliberately carries no
reversal field, recorded as a boundary, so no schema was grandfathered
into this decision.

## Safety analysis

**A reversal plan is an `OperationPlan` with no exemption.** It carries a
source-snapshot proposal like every draft — the forward plan's simulated
final topology, the best prediction in existence — and binds at its own
validate-plan, after the forward apply. PLAN-006 then compares fresh
captures at reversal-apply time, exactly as for any plan. The
Simulated-never-binds rule is not touched, eased, or excepted: a
prediction proposes, and only a helper capture ever binds. Nobody ever
applies a prediction.

**The forward body carries linkage, not the reversal.** One body item:
the emitted reversal draft's plan ID and draft body hash, or PLAN-008's
impossibility statement. `OperationPlan` is **not recursive**. The
reference asymmetry is deliberate and acyclic: the forward body names the
reversal draft **by hash** — freezing what was advertised at
authorization time — while the reversal draft names the forward plan **by
plan ID only**, because a hash reference in both directions is
unconstructible (each body's hash would depend on the other's).

**Created-node targets are step-output references, and no naming
authority moves.** Where a reversal step targets a node the forward plan
creates, the draft spells the target as a typed reference to the creating
step's output, never as an address. A reference names a step in the same
plan, not a device: the address is derived by the helper at the
reversal's validation from its own capture, per ADR-0019's
recompute-at-decode discipline, and an unresolvable reference refuses.
For the round-three residue classes this spelling is the only possible
one — which is why the residue was this issue's.

**Truthfulness is a two-time property.** PLAN-008's "only where truthful"
is judged at emission and re-checked at the reversal's validation through
the draft's own preconditions, which are body content rather than
advisory prose. The named fixture: a draft that deletes a created volume
carries the precondition that the volume holds no user data; once data
lands, the reversal refuses by precondition instead of silently becoming
a destructive plan wearing a reversal's advertisement. The reversal's
severity is computed from its own steps by PLAN-004's ordinary rules and
is not bounded by the forward plan's severity.

**The regress terminates by construction.** A reversal draft's own
PLAN-008 field is the machine-readable statement that its reversal is
re-application of the forward plan, named by plan ID — a reference, not a
third plan. No infinite chain, no pressure on canonical depth budgets.

**Authorization is untouched and un-borrowed.** Applying a reversal is an
apply: it takes its own ADR-0021 floor act and, at its severity or flags,
its own interactive ceremony. Nothing about having authorized the forward
plan authorizes its reversal — which is why embedding the reversal in the
forward body would have bought no authorization value at all.

**Staleness has one fallback, already in the spec.** A draft past its
PLAN-007 window, or refused at validation because the world drifted from
the simulated proposal, is re-planned against a fresh capture — PLAN-007's
re-approval rule doing its existing job. The draft is a feasibility
witness and an advertisement (REC-010; UI-005 shows recovery before
Apply from it); the operative artifact is always the validated
descendant, journaled against the draft's ID.

**The mid-apply boundary is stated, not implied.** PLAN-008's reversal
reverses a *completed* apply. Mid-flight failure is Section 8's and the
journal's (SI-20's exit question included), and this decision touches
none of it. The reversal's validation against a fresh capture is what
makes the boundary safe: a half-applied world fails the reversal's
preconditions or refuses resolution, and the refusal routes to recovery,
never to a forced undo.

**What a consumer and a plan may rely on:**

- At forward plan time: the reversal draft exists (or the impossibility
  statement does); REC-010 may advertise rollback and UI-005 may display
  it; the draft is a prediction bound to nothing.
- At reversal apply time: the applied reversal was validated against a
  fresh helper capture of the post-apply world, PLAN-006 held over
  capture hashes, its preconditions held, and its authorization was its
  own.
- Severity 1 (Reversible) claims stand only on an emitted truthful
  draft — the planner's delivered withheld-claim posture becomes the
  rule: no draft, no Reversible.
- The forward body's linkage hash proves which reversal was advertised
  at authorization time; the journal connects it to the validated
  reversal that ran.

## Options considered

### Option (a) — bind the simulated final topology as the reversal's source

Rejected. It collides with a delivered, mutation-tested structural rule:
`Simulated` is never a planning base and never satisfies a PLAN-006
comparison (WP-060 increment 4, WP-010 3c). Accepting it means either
the helper treats a prediction as a capture — the conflation 8.0.0
exists to end — or every reversal is unvalidatable by construction.

### Option (b) — draft emission, validation-time binding, linkage by reference (accepted)

Accepted, scoped as above.

### Option (c) — exempt reversal plans from snapshot binding

Rejected as the fail-open arm: it creates the only plan class the helper
would apply without stale-topology protection, at exactly the moment the
topology is least certain — post-apply, possibly post-partial-failure,
possibly post-user-modification.

### Option (d) — emit nothing at planning time; re-plan on demand after apply

Rejected: it makes PLAN-008's planning-time emission dead text, REC-010's
advertisement unfoundable, UI-005's pre-apply recovery display empty, and
severity 1's definition ("via an emitted reversal plan") unsatisfiable.
Its live half survives inside the accepted option as the staleness
fallback.

### Option (e) — full recursive embedding

Rejected: the regress is real (PLAN-008 applies to every plan, so the
embedded reversal needs its own embedded reversal, against fixed
canonical depth budgets); the embedded copy would be authorized theater a
later validation is designed to supersede — a frozen-draft agreement
obligation, ADR-0016's lesson; and the linkage hash already provides the
non-illusory half, proof of what was advertised.

## Decision

Option (b), landed as spec 12.0.0's amendments to PLAN-008 and Section
6's reversal body item. **SI-19 moves to Resolved.**

**Major under §0.1:** PLAN-008 gains normative architecture its first
paragraph did not carry, and Section 6's body item changes from "reversal
plan or reversal-impossibility statement" to reversal linkage — an
existing body item's meaning changes, the class 3.1.0 mis-numbered and
this decision does not.

## Consequences

- **Positive.** The register's recursion question closes with the
  non-recursive answer; the created-node residue gets its only possible
  spelling with no naming authority moving; truth decay is refused by
  machinery rather than hoped away; nobody ever applies a prediction.
- **Negative, accepted knowingly.** A reversal draft is routinely
  superseded: its proposal never survives contact with the post-apply
  world, and where drift defeats re-validation the fallback is
  re-planning. The advertisement REC-010 makes from the draft is a
  constructibility claim at plan time, not a promise the exact draft will
  run; UI surfaces must render it as such.
- **Hash-visible, jointly sequenced.** The linkage item enters the hashed
  body when implemented; `schemas/domain/plan-body.md` records the field
  as absent today, and the byte encoding lands as the jointly-sequenced
  WP-060/WP-010 schema change that package's boundary already names.
- **For WP-060.** The PLAN-008 reversal increment unlocks behind this
  decision; the planner's withheld Reversible claim becomes claimable
  exactly where a truthful draft exists.

## Verification

Owned by the packages that build them, recorded here so none is
discovered late:

1. Draft-emission determinism: PLAN-001 holds over the reversal draft
   (byte-equal drafts for equal inputs).
2. Linkage acyclicity as a constructor property: forward→hash,
   reversal→ID; a mutual-hash construction is unrepresentable.
3. A step-output reference resolves against a post-apply capture and
   refuses against a pre-apply one.
4. The volume-with-data precondition refusal as a named fixture: the
   reversal that was metadata-only at emission refuses once data landed.
5. The Simulated-never-binds re-assertion extended to the reversal path:
   a reversal presenting its simulated proposal as a binding fails
   structurally.
6. A reversal apply demands its own ADR-0021 authorization; the forward
   plan's grants satisfy nothing.

## Revisit conditions

- SI-17's decision (severity 1 with `irreversible-after-start`) changes
  what a truthful reversal can claim; this ADR reads PLAN-004's severity
  scale as it stands.
- Section 8's recovery design (SI-20, REC-*) gives mid-apply worlds a
  richer vocabulary; the completed-apply boundary here would deserve
  restating in its terms, not weakening.
- The jointly-sequenced schema change lands the linkage encoding; if the
  encoding cannot express the ID/hash asymmetry as specified, the
  asymmetry is the part to keep and the encoding the part to redesign.
