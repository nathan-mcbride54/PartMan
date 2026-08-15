# SI-24 recommendation round — 2026-08-11

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-24 stays Later (WP-050) until a decision is recorded
through a WP-010 spec change with an ADR, the established shape. This is
an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-24, an early filing
with no options recorded. This round constructs the option space as well
as recommending from it.

**Why this is decidable now**, though the filing's subject binds when
dry-run exists: the conflict is between two existing texts, the decision
is a terminological boundary plus refusal semantics, and no evidence
clause names an unbuilt artifact or untaken measurement. The decision
*constrains* the future dry-run implementation rather than reading it —
deciding before WP-070 builds the pipeline is precisely what keeps the
pipeline from being built on a guess.

---

## The conflict, made precise

> **CAP-003:** `preview` — planning and simulation permitted, apply
> refused pending qualification evidence, labeled as such in GUI and
> CLI […]

> **PLAN-009:** Dry run MUST traverse the identical pipeline as a real
> apply — including helper revalidation (HLP-002) — and stop before the
> Protecting state. A successful dry run means the only remaining
> variables are physical execution outcomes, not validation surprises.

A dry run of a preview-backed plan must therefore either fail —
contradicting "simulation permitted," if a dry run is a simulation — or
succeed while apply is still guaranteed to be refused for a non-physical
reason, contradicting PLAN-009's success semantics. The filing's fork is
exact, and it turns on one undefined word: whether "simulation" in
CAP-003 includes PLAN-009's dry run.

What the delivered surfaces already give this round: WP-050's engine
returns `preview` as a distinct answer whose planning permission WP-060's
planner consumes ("`preview` permits planning"); WP-060's increment 4
emits the PLAN-002 simulated final topology through
`SnapshotKind::Simulated`, the prediction that can never bind; and the
glossary already defines Preview as "planning allowed, apply refused
pending qualification" — with no mention of dry run.

## Recommendation: "simulation" is the planner's prediction; a dry run is an apply rehearsal — it runs, and it refuses exactly where apply would

**The two texts stop conflicting the moment the undefined word is
defined, and the definition the architecture already implies is the
right one.** Concretely:

1. **CAP-003's "planning and simulation" means the pure planner
   surface:** PLAN-001 planning and PLAN-002's simulated final topology
   — the client-side, side-effect-free prediction. That is what
   `preview` licenses, exactly what the delivered planner already
   consumes, and exactly what the glossary already says. A dry run is
   not a simulation: it is a rehearsal of the apply pipeline (PLAN-009's
   own words — "the identical pipeline as a real apply"), and it
   belongs to the apply surface that `preview` refuses.
2. **A dry run of a preview-backed plan runs — and terminates at the
   helper's own recomputed capability gate with a typed CAP-003
   refusal.** It is not refused upfront on the client's capability view:
   CAP-007 makes that view advisory, and the helper's recomputation is
   the authority (HLP-002). Running the identical pipeline until the
   helper's own gate fires preserves parity by construction, catches
   drift between the client's capability answer and the helper's, and
   makes the dry run *useful* on preview plans: it proves everything up
   to the gate and returns an authenticated, typed statement of exactly
   why apply would refuse — reason `pending qualification`, remediation
   naming the CAP-006 evidence gap, never conflatable with a validation
   failure of the plan itself.
3. **Such a dry run is never "successful," so PLAN-009's guarantee
   survives absolute.** The success sentence — only physical outcomes
   remain — keeps its full strength with no caveat class, because the
   preview case never reaches success: it is a typed refusal, before
   Protecting, with no writes, which is the dry run doing its job
   (telling the truth about apply) rather than failing at it.
4. **Labeling extends naturally.** CAP-003's "labeled as such" already
   requires preview plans to wear their status in GUI and CLI; the
   dry-run refusal's reason class is that label reaching the pipeline's
   answer. No UI requirement changes.

## What a consumer and a plan may rely on

- `preview` licenses exactly the pure surface: a plan and its simulated
  final topology. Nothing about `preview` ever licenses a write or a
  successful apply rehearsal.
- A **successful** dry run means what PLAN-009 says, absolutely: only
  physical outcomes remain. There is no success-with-asterisk.
- A dry run's capability refusal is typed and distinguishable from a
  validation failure: it names the qualification gap and its CAP-006
  remediation. A consumer can tell "your plan is fine, the platform
  combination is unqualified" from "your plan is broken" by the reason
  class, not by prose.
- The gate that refuses is the helper's own recomputation — a client
  cannot dry-run its way past a capability by asserting `supported`
  (CAP-007's rule reaching the rehearsal).

## The adversarial round

**Attack 1 — "this reads 'simulation' down to rescue the design;
CAP-003's author may have meant dry run."** Refuted by the surrounding
text's own testimony: the glossary defines Preview with no dry-run
mention; PLAN-002 names its output "simulated final topology" and
PLAN-009 never uses the word "simulation" for the dry run — the spec's
own vocabulary already splits the two. The round defines the word where
it was undefined; it retexts nothing.

**Attack 2 — "running the pipeline just to refuse wastes the user's
time; refuse upfront from the client's capability answer."** Rejected on
CAP-007: the client's view is advisory, and an upfront client-side
refusal would make it authoritative — the exact upgrade-by-assertion
inversion CAP-007 forbids, in the refusing direction. The helper's gate
is the truth; the pipeline run is how the truth is produced. A client
MAY still *display* the expected refusal before submitting (advisory UX
is its job); it may not substitute for the helper's answer.

**Attack 3 — "option (b) is kinder: succeed with a carried
would-refuse-at-apply caveat, so users see the whole rehearsal."**
Rejected as the asterisk that eats the guarantee. PLAN-009's success
sentence is the one crisp promise dry-run makes; a success-with-caveat
class makes every consumer parse two kinds of success, and the caveat
trains surfaces to ignore it — the rubber-stamp shape in a new coat.
What (b) wanted — visibility past the gate — the recommendation
delivers differently: everything before the gate is exercised, and the
refusal is typed, not opaque.

**Attack 4 — "a partial pipeline that skips the capability gate would
let preview users rehearse everything else."** Rejected as the parity
killer: a second pipeline is exactly what PLAN-009's "identical
pipeline" exists to forbid, and it would diverge from the real one by
construction — the validation-surprise class the requirement was
written to end.

**Attack 5 — "where in the pipeline does the capability gate fire? The
recommendation smuggles in an ordering decision."** Sustained as a
boundary and answered by not deciding it: parity is the property, gate
order is the implementation. The ADR fixes that the dry run refuses
*exactly where and how apply would* — whatever order the pipeline has,
sameness is the requirement. WP-070 owns the order; the parity tests
own proving sameness.

**Attack 6 — "deciding before dry-run exists repeats the
build-on-unmeasured-claims mistake the register rejected in rounds two
and three of the blockers."** Refuted by kind: those rounds built
*designs* on unmeasured *platform facts*. This decision defines a word
across two existing texts and constrains an unbuilt component's
semantics — the same class as SI-19's resolution constraining the
unbuilt reversal increment, which is the register's working precedent
five times over today.

## Rejected, and why — to be recorded with the decision

- **(b) Dry run succeeds on preview plans with a carried caveat.**
  Attack 3: the asterisk that eats PLAN-009's one crisp guarantee, and
  a misleading success under CAP-007's advisory discipline.
- **(c) A partial pipeline excluding the capability gate.** Attack 4:
  kills parity by construction — the second pipeline PLAN-009 exists to
  forbid.
- **(d) Narrow `preview` to forbid simulation as well.** Retexts
  CAP-003 to delete the capability's entire value — the labeled
  look-ahead — solving a definitional problem by amputation; major, and
  for nothing.
- **(e) Refuse the dry-run request upfront from the client's capability
  view.** Attack 2: inverts CAP-007 by making the advisory view
  authoritative.

## Deliberately not decided

The pipeline's internal gate order (WP-070's, under the parity
property); any CAP-006 qualification-evidence process; SI-20 and REC-*;
the dry-run UX beyond the typed reason class; SI-25's operation-list
span question (untouched).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0026 is the next free number; reservation PR
before resolution PR, the established shape), amends **CAP-003 and
PLAN-009** — both existing texts stand verbatim; CAP-003 gains the
definitional sentence ("planning and simulation" = PLAN-001/PLAN-002's
pure surface; a dry run is apply-surface), and PLAN-009 gains the
preview-arm sentence (a dry run of a preview-backed plan terminates at
the helper's recomputed capability gate with the typed CAP-003 refusal,
so it is never successful and the success guarantee stands absolute) —
bumps **minor** (12.4.0: additions defining an undefined word and an
unaddressed case; no existing claim narrows), and moves SI-24 to
Resolved. The major counter-argument is recorded for the decision to
overrule with. WP-060's re-attribution follows (the established shape):
its last register gate clears, and the planner's gate-list comment
rides the debt train. WP-050 is checked for citations in the same pass
(none found today beyond the register's own entry).

Verification obligations for the ADR, owned by WP-070 when the pipeline
exists, with the client-side halves owned by their surfaces:

1. The parity property: a dry run of a preview-backed plan and a real
   apply of the same plan refuse at the same gate with the same typed
   reason — sameness asserted over the pair, not over an ordering.
2. The refusal's reason class distinguishes pending-qualification from
   every validation-failure class, by type.
3. A hand-forged client capability assertion (`supported` claimed over
   a preview combination) does not change the dry run's outcome — the
   ADR-0012 hand-forged pattern applied to the rehearsal.
4. A successful dry run is constructible only where the helper's
   recomputed capability permits apply — success-with-caveat is not a
   representable outcome.
