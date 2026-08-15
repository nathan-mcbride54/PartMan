# SI-19 recommendation round — 2026-08-11

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-19 stays Later (WP-060) until a decision is recorded
through a WP-010 spec change with an ADR, the shape ADR-0015/0021 set. This
is an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-19, an early filing
amended by naming round three (the created-node residue — volumes minted
inside an existing container, LVM snapshots — is SI-19's, not SI-27's). The
filing sketches three postures: the reversal binds the simulated final
topology, is emitted unbound and re-planned after apply, or is exempt; and
it names what the answer decides — whether `OperationPlan` is recursive.

---

## The conflict, made precise — and what has dissolved under it since filing

The filed texts:

> **PLAN-008:** For every plan, the planner MUST either emit a reversal
> plan (only where reversal is truthful, e.g., metadata-only changes) or a
> machine-readable, per-step statement of why reversal is impossible. This
> output feeds REC-010: rollback may be advertised only where a reversal
> plan exists.

> **Section 6 (body):** Source topology snapshot body hash, **as bound at
> validation**: the client's draft snapshot is a proposal, and the
> snapshot whose hash the authorized plan binds is the one HLP-002's
> re-discovery produces during validate-plan. *(8.0.0, ADR-0014)*

> **PLAN-006:** The helper MUST re-discover target topology and reject a
> mismatch before the first write […] over **body** hashes.

The filing predates spec 8.0.0, and 8.0.0 changed the ground under it.
When SI-19 was filed, a plan's snapshot binding read as a planning-time
fact, so a reversal plan — whose source topology does not exist at
planning time — looked unbindable by construction. Since 8.0.0, **binding
is a validation act for every plan**: the client's planning-time snapshot
is a proposal, and the hash the authorized plan carries is the one the
helper's own re-discovery produces at validate-plan. A reversal plan
emitted at planning time is exactly as unbound as every other plan at
planning time. What remains genuinely open is threefold:

1. **Carriage:** does the forward plan's hashed body contain the reversal
   plan itself (recursion), a reference to it, or nothing?
2. **The created-node residue:** a reversal draft for "create volume V"
   must target V, and round three established that the residue classes
   (container-minted volumes, LVM snapshots) have no positional address
   until they exist. What does the draft say?
3. **Truth decay:** PLAN-008 permits a reversal only where truthful, and
   truth is evaluated twice — a deletion that reverses "create volume" is
   metadata-only at planning time and destroys user data once anything
   lands in that volume.

Two delivered facts constrain every answer, both held by committed tests:
`SnapshotKind::Simulated` can never be a planning base or satisfy a
PLAN-006 comparison, structurally (WP-060 increment 4, re-asserting
WP-010 3c); and the plan-body schema deliberately carries no reversal
field yet — `schemas/domain/plan-body.md` records that absence as a
boundary, so no schema is grandfathered into the decision.

## Recommendation: the reversal is an ordinary draft, linked by reference — `OperationPlan` is not recursive

**Emit the reversal at planning time as a draft `OperationPlan`; bind it
at its own validation, after the forward apply; carry it in the forward
body by reference, not by value.** Concretely:

1. **A reversal plan is an `OperationPlan` with no special exemption.**
   It carries a source-snapshot proposal like every draft — its proposal
   is the forward plan's simulated final topology, the best prediction in
   existence — and its *binding* happens at its own validate-plan, after
   the forward apply, when its source topology exists and HLP-002's
   re-discovery can capture it. PLAN-006 then compares fresh captures at
   reversal-apply time, exactly as for any plan. The delivered
   Simulated-is-never-a-base rule is not touched, eased, or excepted: the
   simulated snapshot proposes, and only a helper capture ever binds.
2. **The forward body carries reversal linkage, not the reversal.** One
   body item: the emitted reversal draft's plan ID and draft body hash,
   or PLAN-008's per-step impossibility statement. `OperationPlan` is
   **not recursive** — the register's named question answered. The
   reference asymmetry is deliberate and acyclic: the forward body names
   the reversal draft **by hash** (freezing what was advertised); the
   reversal draft names the forward plan **by plan ID only**, because a
   hash reference in both directions is unconstructible (each body's hash
   would depend on the other's).
3. **Created-node targets are step-output references.** Where a reversal
   step targets a node the forward plan creates, the draft spells the
   target as a typed reference to the creating step's output ("the volume
   step N mints"), not as an address. This is not a naming claim:
   ADR-0019 addresses are derived facts recomputed at decode, and the
   reference resolves to a real derived address only at the reversal's
   validation, against the helper's own capture, after the node exists.
   An unresolvable reference refuses validation. For the round-three
   residue classes this spelling is the only possible one — which is why
   the residue is this issue's.
4. **Truth is re-evaluated where it can decay.** The reversal draft
   carries its own preconditions (Section 6 already requires per-step
   preconditions), and the truthfulness PLAN-008 demands at emission is
   re-checked at the reversal's own validation: the draft that deletes a
   created volume carries the precondition that the volume holds no user
   data, and a reversal whose preconditions fail refuses — it does not
   quietly become a destructive plan wearing a reversal's advertisement.
   The reversal's severity is computed from its own steps by PLAN-004's
   ordinary rules; nothing bounds it by the forward plan's severity.
5. **The regress terminates by construction.** The reversal draft's own
   PLAN-008 field is the machine-readable statement that its reversal is
   re-application of the forward plan, named by plan ID — a reference,
   not a third plan. No infinite chain, no depth pressure on `pce/1`
   budgets.
6. **Authorization is untouched and un-borrowed.** Applying a reversal is
   an apply: it takes its own floor act and, at its severity or flags,
   its own interactive ceremony (HLP-003 as ADR-0021 landed it). Nothing
   about having authorized the forward plan authorizes its reversal —
   which is why embedding the reversal in the forward body would have
   bought no authorization value at all.
7. **Staleness has one fallback, already in the spec.** A reversal draft
   past its PLAN-007 window, or refused at validation because the world
   drifted from the simulated proposal, is re-planned against a fresh
   capture — PLAN-007's re-approval rule doing its existing job. The
   draft is a feasibility witness and an advertisement (REC-010, UI-005
   show recovery before Apply from it); the operative artifact is always
   the validated descendant, journaled against the draft's ID.

## What a consumer and a plan may rely on

- **At forward plan time:** the reversal draft exists (or the
  impossibility statement does); REC-010 may advertise rollback and
  UI-005 may display it. The draft is a prediction bound to nothing.
- **At reversal apply time:** the applied reversal was validated against
  a fresh helper capture of the post-apply world; PLAN-006 held over
  capture hashes; its preconditions held at validation; its authorization
  was its own. Nobody applied a prediction.
- **Severity 1 (Reversible) claims** stand only on an emitted truthful
  draft — the planner's existing withheld-claim posture becomes the rule:
  no draft, no Reversible.
- **The forward body's linkage hash** proves which reversal was
  advertised at authorization time; the journal connects it to the
  validated reversal that actually ran.

## The adversarial round

**Attack 1 — "the draft is stale by construction: the post-apply world
never byte-matches a simulation, so the draft never validates and the
emission is theater."** Partly sustained, and it sharpened point 7. The
draft's *proposal* being replaced at validation is the design, not a
defect — that is 8.0.0's architecture for every plan. What the attack
kills is any reading where the draft's proposal must *match* the capture
for the reversal to proceed; the round rejects that reading in terms. The
draft buys three real things: the feasibility witness PLAN-008 demands at
planning time, REC-010's truthful advertisement, and the severity-1
claim's basis. Where drift defeats even re-validation, re-planning is the
recorded fallback and the advertisement was still honest about
constructibility at plan time.

**Attack 2 — "step-output references are client-authored naming through
the back door — the drift ADR-0019 exists to kill."** Refuted by locating
the authority. A reference names a *step in the same authorized plan*,
not a device or an address; the address is derived by the helper at the
reversal's validation from its own capture, per ADR-0019's
recompute-at-decode discipline, and an unresolvable reference refuses.
Nothing the client spells ever becomes an address by being spelled.

**Attack 3 — "a reversal of a partially applied forward plan reverses the
wrong world."** Out of scope, and the boundary is stated rather than
implied: PLAN-008's reversal reverses a *completed* apply. Mid-flight
failure is Section 8's and the journal's (REC-*, SI-20's exit question),
and this round decides none of it. The reversal's validation against a
fresh capture is what makes this safe rather than aspirational: a
half-applied world fails the reversal's preconditions or refuses
resolution, and the refusal routes to recovery, not to a forced undo.

**Attack 4 — "embedding the reversal in the forward body is strictly
safer: the user authorizes exactly what rollback will do."** Refuted on
three grounds. The authorization is illusory — the reversal cannot run
without its own validation and its own fresh authorization (HLP-003), so
the embedded copy would be authorized theater that a later validation is
designed to supersede: a frozen-draft agreement obligation, ADR-0016's
lesson again. The regress is real — PLAN-008 applies to every plan, and
embedding makes the reversal's reversal a third body, against fixed
canonical depth budgets. And the linkage hash already gives the
non-illusory half: proof of what was advertised.

**Attack 5 — "truth decay makes every reversal draft a lie waiting to
happen — 'only where truthful' cannot survive the gap between emission
and apply."** Sustained as the sharpest finding and absorbed as point 4:
truthfulness is a two-time property, checked at emission *and* re-checked
as preconditions at the reversal's validation. The volume-deletion case
is the named fixture: metadata-only at emission, destructive once data
lands, refused by precondition rather than reclassified silently. What
the attack forced: the preconditions are body content of the draft, not
advisory prose, so the re-check is PLAN-006-adjacent machinery, not
goodwill.

**Attack 6 — "exemption is simpler: reversal plans skip snapshot binding
and PLAN-006, since their world is unknowable at emission."** Rejected as
the fail-open arm. It creates the only plan class the helper would apply
without stale-topology protection, at exactly the moment the topology is
least certain — post-apply, possibly post-partial-failure, possibly
post-user-modification. Every refusal surface this spec has built points
the other way.

## Rejected, and why — to be recorded with the decision

- **(a) Bind the simulated final topology as the reversal's source.**
  Collides with a delivered, mutation-tested structural rule: `Simulated`
  is never a planning base and never satisfies a PLAN-006 comparison
  (WP-060 increment 4, WP-010 3c). Accepting it means either the helper
  treats a prediction as a capture — the conflation 8.0.0 exists to end —
  or every reversal is unvalidatable by construction.
- **(c) Exempt reversal plans from binding.** Attack 6: the fail-open
  arm; an unbindable plan class with no PLAN-006 protection.
- **(d) Emit nothing at planning time; re-plan on demand after apply.**
  Makes PLAN-008's planning-time emission dead text, REC-010's
  advertisement unfoundable, UI-005's pre-apply recovery display empty,
  and severity 1's definition ("via an emitted reversal plan")
  unsatisfiable. Its live half survives inside the recommendation as the
  staleness fallback.
- **(e) Full recursive embedding.** Attack 4: regress, depth budgets, and
  a frozen draft whose supersession is the design.

## Deliberately not decided

SI-17 (severity 1 with `irreversible-after-start`), SI-16 (backup step
family), SI-24 (preview versus dry-run), SI-20 and every REC-* behavior
(recovery is WP-070/WP-R100's), and the linkage field's byte encoding —
the jointly-sequenced WP-010 schema change WP-060's boundary already
names, landing when the field is implemented, not when it is decided.

## If accepted, the mechanics

WP-010 files the ADR (ADR-0022 is the next free number; the reservation
PR precedes the resolution PR, the #259/#260 shape), amends **PLAN-008**
(draft emission, validation-time binding, step-output references,
two-time truthfulness, the regress statement) and **Section 6's body
item** ("Reversal plan or reversal-impossibility statement" becomes
"Reversal linkage: the emitted reversal draft's plan ID and draft body
hash, or the per-step impossibility statement"), bumps **major** —
Section 6's body item and PLAN-008's text both change meaning, the §0.1
class 3.1.0 mis-numbered — and moves SI-19 to Resolved. WP-060's
PLAN-008 increment unlocks behind it, with the body carriage landing as
the jointly-sequenced WP-010 schema change already recorded in that
package's boundary.

Verification obligations for the ADR, owned by the packages that build
them: the draft-emission determinism test (PLAN-001 over the reversal
draft); the linkage acyclicity fact (forward→hash, reversal→ID) as a
constructor property; a step-output reference resolving against a
post-apply capture and refusing against a pre-apply one; the
volume-with-data precondition refusal as a named fixture; and the
Simulated-never-binds re-assertion extended to the reversal path.
