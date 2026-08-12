# ADR-0026: A dry run is an apply rehearsal, not CAP-003's simulation

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride by delegation ("I don't
  mind you picking a side — file it as Accepted") on the adversarially
  reviewed recommendation round of the previous day, the delegation
  recorded here as the acceptance basis
  (`docs/reviews/SI-24_RECOMMENDATION_ROUND_2026-08-11.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.4.0 (minor under §0.1 — additions defining an
  undefined word and an unaddressed case; argued in Decision, with the
  major counter-argument recorded)
- Work packages blocked: none newly — WP-060's last register gate
  clears; the dry-run pipeline this constrains is WP-070's, unbuilt
- Requirement IDs: CAP-003, PLAN-009 (amended); PLAN-001, PLAN-002,
  CAP-006, CAP-007, HLP-002, Section 20 glossary (read, none amended)
- Decision owners: Nate McBride

## Context

CAP-003's `preview` permits "planning and simulation" while apply is
refused pending qualification evidence. PLAN-009's dry run traverses the
identical pipeline as a real apply, and its success means only physical
outcomes remain. SI-24 filed the fork: a dry run of a preview-backed
plan must either fail — contradicting "simulation permitted," if a dry
run is a simulation — or succeed while apply is guaranteed-refused for a
non-physical reason, gutting PLAN-009's success semantics.

The conflict turns on one undefined word, and the spec's own vocabulary
had already split it: PLAN-002 names its output the "simulated final
topology," PLAN-009 never uses "simulation" for the dry run, and the
Section 20 glossary defines Preview as "planning allowed, apply refused
pending qualification" with no dry-run mention. The delivered surfaces
agree: WP-050's engine returns `preview` as the planning permission
WP-060's planner consumes, and WP-060's increment 4 emits the simulated
final topology through `SnapshotKind::Simulated`, the prediction that
can never bind.

**Timing, stated because the filing binds when dry-run exists:** no
evidence clause names an unbuilt artifact or untaken measurement. The
decision defines a word across two existing texts and constrains the
unbuilt pipeline's semantics rather than reading them — the same class
as ADR-0022 constraining the unbuilt reversal increment. Deciding before
WP-070 builds the pipeline is what keeps the pipeline from being built
on a guess.

## Safety analysis

**"Simulation" is the planner's prediction.** CAP-003's "planning and
simulation" means the pure planner surface: PLAN-001 planning and
PLAN-002's simulated final topology — client-side, side-effect-free,
exactly what the delivered planner consumes and the glossary describes.
A dry run is a rehearsal of the apply pipeline — PLAN-009's own words —
and belongs to the apply surface `preview` refuses.

**The dry run runs, and the helper's own gate refuses it.** A dry run of
a preview-backed plan is not refused upfront from the client's
capability view: CAP-007 makes that view advisory, and an upfront
client-side refusal would make it authoritative — the
upgrade-by-assertion inversion in the refusing direction. The identical
pipeline runs until the helper's own recomputed capability gate fires
(HLP-002), which preserves parity by construction, catches drift between
the client's answer and the helper's, and makes the dry run useful on
preview plans: everything before the gate is exercised, and the answer
is an authenticated, typed statement of exactly why apply would refuse —
reason pending-qualification, remediation naming the CAP-006 evidence
gap, distinguishable by type from every validation-failure class. "Your
plan is fine, the combination is unqualified" is never conflatable with
"your plan is broken."

**PLAN-009's guarantee survives absolute.** Such a dry run is never
*successful* — it is a typed refusal, before Protecting, with no writes,
the dry run telling the truth about apply rather than failing at it. The
success sentence keeps its full strength with no caveat class: a
successful dry run still means only physical outcomes remain, and
success-with-caveat is not a representable outcome.

**Gate order is deliberately not decided.** Parity is the property; the
pipeline's internal order is WP-070's implementation. The ADR fixes that
the dry run refuses exactly where and how apply would — sameness of the
refusal pair is what verification asserts, over the pair, not over an
ordering.

**What a consumer and a plan may rely on:**

- `preview` licenses exactly the pure surface: a plan and its simulated
  final topology. Nothing about `preview` licenses a write or a
  successful apply rehearsal.
- A successful dry run means what PLAN-009 says, absolutely.
- A dry-run capability refusal is typed and names the qualification gap
  and its remediation; it is distinguishable from validation failure by
  type, not prose.
- The refusing gate is the helper's own recomputation; a client cannot
  dry-run past a capability by asserting `supported`.

## Options considered

### Option (a) — the dry run runs and refuses at the helper's capability gate, typed (accepted)

Accepted, scoped as above, with the definitional boundary that dissolves
the filed contradiction.

### Option (b) — the dry run succeeds with a carried would-refuse-at-apply caveat

Rejected: the asterisk that eats the one crisp guarantee dry-run makes.
Every consumer must then parse two kinds of success, and the caveat
trains surfaces to ignore it — the rubber-stamp shape in a new coat. It
also shows success while apply is guaranteed-refused, misleading under
CAP-007's advisory discipline. What (b) wanted — visibility past the
gate — the accepted option delivers differently: everything before the
gate is exercised, and the refusal is typed, not opaque.

### Option (c) — a partial pipeline excluding the capability gate

Rejected as the parity killer: a second pipeline is what PLAN-009's
"identical pipeline" exists to forbid, and it diverges from the real one
by construction — the validation-surprise class the requirement was
written to end.

### Option (d) — narrow `preview` to forbid simulation as well

Rejected: retexts CAP-003 to delete the capability's entire value — the
labeled look-ahead — solving a definitional problem by amputation, at a
major bump, for nothing.

### Option (e) — refuse the dry-run request upfront from the client's capability view

Rejected on CAP-007: it makes the advisory view authoritative. A client
MAY display the expected refusal before submitting — advisory UX is its
job — but may not substitute for the helper's answer.

## Decision

Option (a), landed as spec 12.4.0's amendments to CAP-003 and PLAN-009.
**SI-24 moves to Resolved — WP-060's last register gate clears.**

**Minor under §0.1, argued rather than assumed:** both existing texts
stand verbatim. CAP-003 gains a definitional sentence for a word it
never defined; PLAN-009 gains the preview-arm sentence for a case it
never addressed; no existing claim narrows. The counter-argument (a
first definition fixes semantics other text depended on — the 3.1.0
caution) was weighed and is recorded so the numbering is auditable; it
was not taken because §0.1's rule turns on what happens to existing
requirement text, and none changes.

## Consequences

- **Positive.** The filed fork dissolves without weakening either text;
  dry-run keeps its one crisp guarantee; preview keeps its whole value;
  the unbuilt pipeline inherits decided semantics instead of a guess;
  drift between client capability views and helper recomputation is
  caught by the rehearsal itself.
- **Negative, accepted knowingly.** A preview-backed plan's dry run
  spends a full pipeline traversal to produce a refusal a client could
  have predicted — deliberately, because the helper's answer is the
  authoritative one and the traversal is what authenticates it.
- **For WP-060.** Its register-gate list empties; the gate-list comment
  in `crates/planner` rides the next Rust increment with the other
  debts.
- **For WP-070.** The dry-run pipeline arrives with its preview-arm
  semantics decided: typed refusal at the recomputed gate, no
  success-with-caveat outcome, parity asserted over the refusal pair.
- Nothing here is hash-visible: no field, no schema, no state-machine
  edge.

## Verification

Owned by WP-070 when the pipeline exists, with client-side halves owned
by their surfaces, recorded here so none is discovered late:

1. The parity property: a dry run of a preview-backed plan and a real
   apply of the same plan refuse at the same gate with the same typed
   reason — sameness asserted over the pair, not over an ordering.
2. The refusal's reason class distinguishes pending-qualification from
   every validation-failure class, by type.
3. A hand-forged client capability assertion (`supported` claimed over a
   preview combination) does not change the dry run's outcome — the
   ADR-0012 hand-forged pattern applied to the rehearsal.
4. A successful dry run is constructible only where the helper's
   recomputed capability permits apply; success-with-caveat is not a
   representable outcome.

## Revisit conditions

- CAP-003's status vocabulary changes (SI-26's territory, if ever
  opened); the definitional sentence reads the current four statuses.
- WP-070's pipeline design finds a gate whose refusal cannot be typed
  distinguishably; the typed-distinguishability property is the part to
  keep and the gate's reporting the part to redesign.
- A future surface wants a rehearsal that deliberately continues past
  the capability gate (a "what else would fail" mode); that is a new
  surface with its own round, not a reading of PLAN-009, whose identical-
  pipeline sentence this ADR leaves absolute.
