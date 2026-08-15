# SI-15 recommendation round — 2026-08-11

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-15 stays Later (WP-060) until a decision is recorded
through a WP-010 spec change with an ADR, the shape ADR-0015/0021/0022
set. This is an untracked session artifact under `docs/reviews/**`
(WP-000); the register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-15, an early
filing with no options recorded. This round constructs the option space
as well as recommending from it.

---

## The conflict, made precise

> **PART-009:** Align partitions to 1 MiB boundaries by default.
> Deviations occur only when the device's published geometry requires
> different alignment or the user explicitly overrides; both are recorded
> in the plan.

> **Section 11.2:** automated tests MUST prove […] Required alignment is
> preserved.

The filed case: a legacy MBR partition at a non-1 MiB offset (the XP-era
63-sector start is the population) grown at its tail. Growing never
touches the start, so the misalignment is neither created nor curable by
the operation — yet under a strict reading of PART-009, the resulting
layout "deviates" without either permitted cause, and the only
PART-009-clean paths are refusing the grow or realigning the start, which
forces a PART-005 data move the user did not request. The delivered
solver (WP-060 increment 3) conservative-refuses the case by name, with
the gate string carried, and its deviation-override vocabulary is
deliberately inexpressible until decided.

The ambiguity is in one word: whether a **deviation** is a state the
finished layout is in, or an act the plan performs. Section 11.2's
"required alignment is preserved" inherits the same fork — preserved on
what the plan touched, or restored on everything it didn't?

## Recommendation: a deviation is authored, not inherited — grow proceeds, the fact is recorded

**PART-009 governs boundaries a plan authors; a pre-existing boundary the
plan does not move is an inherited fact, not a deviation.** Concretely:

1. **An authored boundary is one whose byte offset the plan sets.** All
   authored boundaries follow the 1 MiB default, with PART-009's two
   existing deviation causes (published geometry, explicit override)
   untouched and still the only ways to author off-policy. A boundary
   that is byte-identical before and after the plan is inherited: it
   demands no override, satisfies no deviation clause, and blocks
   nothing.
2. **The filed case therefore proceeds.** Growing a misaligned partition
   at its tail authors one boundary — the new end — which follows
   policy. The untouched misaligned start is inherited. The plan MUST
   record the inherited off-policy boundary as a stated fact in its
   consequence text (UI-005 already displays it; consequence text is
   existing body content, so no schema changes), phrased as a fact about
   the device, never as a deviation the user granted.
3. **A boundary placed coincident with a pre-existing structural edge is
   policy-conformant.** Grow-to-fill sets the new end at the next
   partition's start or the device's end. That offset is authored in the
   trivial sense but chosen by nothing — aligning it down instead would
   mint an unusable sliver of free space. The rule: an authored boundary
   placed coincident with a pre-existing structural edge (a neighbor's
   boundary, the device end) conforms to policy and is recorded as
   coincident. This case is inside SI-15's scope because grow-at-tail-
   to-fill is the filed operation's common form, and deciding the start
   while leaving the end undefined would re-file the same issue.
4. **Realignment remains available and explicit.** A user who wants the
   start moved requests a move: a PART-005 operation at its own severity
   (3, data-moving), its own consequence text, its own authorization
   tier. Nothing converts a grow into a move, in either direction.
5. **Section 11.2's invariant reads as the distinction implies, with no
   text change:** required alignment is *preserved* — authored
   boundaries meet policy; inherited boundaries are byte-identical
   before and after. A test proving both proves the invariant.
6. **The solver's refused case unlocks without the override
   vocabulary.** Under this reading the filed case involves no
   deviation, so the deliberately-inexpressible override machinery stays
   inexpressible — a separately gated vocabulary for the day a user
   authors an off-policy boundary on purpose. The unlock is exactly the
   refused population, nothing wider.

## What a consumer and a plan may rely on

- A plan never changes a boundary it does not name: inherited boundaries
  are byte-identical before and after, and the §11.2 test obligation
  covers exactly that.
- Every authored boundary either meets the 1 MiB default, is coincident
  with a named pre-existing structural edge, or carries one of PART-009's
  two recorded deviation causes. There is no fourth state.
- A plan touching a device with inherited off-policy boundaries says so
  in its consequence text — the user sees the fact without being asked
  to authorize it as if it were the plan's doing.
- Severity is honest: a grow is never silently a move.

## The adversarial round

**Attack 1 — "'authored' will creep: the grown end's placement is
constrained by the misaligned start, so the plan launders the start's
misalignment into its own output."** Refuted by the definition the attack
forced into point 1: authored means the plan sets the byte offset, and
the end's offset is independent of the start's — a partition starting at
sector 63 can end on any 1 MiB boundary. Nothing about the start
propagates into the authored end. The attack's residue is the
coincident-edge case, which point 3 answers in terms rather than leaving
to creep.

**Attack 2 — "the strict reading is safer: refusing growth forces users
to confront and fix XP-era layouts instead of entrenching them."**
Rejected as safety theater. Refusing a grow fixes no alignment — it
blocks maintenance on the population while the misalignment persists
either way; the fix (a move) stays available, explicit, and honestly
priced at severity 3. And the strict reading's other exit is worse:
auto-realign converts a low-severity grow into a data-moving operation
silently, the exact severity laundering PLAN-004's model exists to
prevent — the same class of silent whole-device-consequence path the
3.1.0 round rejected in its blank-media carve-out.

**Attack 3 — "grow-to-fill authors an off-policy end whenever the
neighbor is misaligned, and now there is no override vocabulary to record
it."** Sustained as the round's sharpest finding and absorbed as point 3:
without the coincident-edge rule, resolving the start question re-files
the same issue about the end. The rule is scoped tightly — coincident
with a *pre-existing structural edge*, recorded as such — so it licenses
no free-floating off-policy placement.

**Attack 4 — "recording inherited facts belongs in a typed field, not
consequence text — prose is unqueryable."** Rejected for now, recorded as
a revisit condition. The consumer who needs the fact (the user at UI-005,
the reviewer reading the plan) reads consequence text; no delivered
surface queries alignment facts. Minting a typed body field for it is a
hashed-schema change with a real cost (the ADR-0016/0021 lesson: fields
that duplicate derivable facts create agreement obligations — the offsets
are already in the bound snapshot). If a consumer that must query the
fact ever exists, that is its round.

**Attack 5 — "this decides SI-16/SI-17 territory by touching the
severity argument."** Refuted by scope: the severity-honesty point cites
PLAN-004's existing scale; no flag legality (SI-17), no backup family
(SI-16), no preview semantics (SI-24) is touched, and the round says so.

## Rejected, and why — to be recorded with the decision

- **(a) Strict reading: any plan touching an off-policy layout carries a
  deviation and needs the override.** Semantically wrong — the user is
  not overriding a policy the plan never applied — and operationally it
  gates a routine maintenance operation on machinery (the override
  vocabulary) built for a different act, blocking the legacy population
  for no data-safety return.
- **(b) is the recommendation.**
- **(c) Auto-realign on grow.** The filing's own objection, sharpened:
  an unrequested PART-005 data move smuggled inside a grow, severity
  laundered from metadata-write to data-moving, the silent-consequence
  shape this register has rejected every time it has appeared.
- **(d) Refuse permanently.** Over-refusal with no compensating
  guarantee: the misalignment persists, the population is locked out of
  tail growth, and the product's fail-closed posture is spent on a case
  with no failure to close against.

## Deliberately not decided

SI-16, SI-17, SI-24; the deviation-override vocabulary (stays
inexpressible until a round needs it); any typed carriage of alignment
facts (Attack 4's revisit condition); move/realign UX.

## If accepted, the mechanics

WP-010 files the ADR (ADR-0023 is the next free number; reservation PR
before resolution PR, the established shape), amends **PART-009 only** —
the two existing sentences stand verbatim, gaining the authored/inherited
scoping, the coincident-edge rule, and the inherited-fact recording
obligation — bumps **minor** (12.1.0: additions; the foreclosed strict
reading was never text, and ADR-0020's precedent shows reading-selection
alone amends nothing — the bump here pays for the added sentences), and
moves SI-15 to Resolved. The major counter-argument (disambiguation as
semantic change, the 3.1.0 caution) is recorded for the decision to
overrule with. WP-060's solver unlock follows as its own re-attribution
(#261/#264 shape): the named refusal case becomes a permitted case with
its inherited-fact recording, a code change that rides the crate's next
Rust increment together with the planner's SI-19 comment debt.

Verification obligations for the ADR: the grow-at-tail fixture on a
63-sector-start MBR image (authored end aligned, start byte-identical,
consequence text carrying the inherited fact); the grow-to-fill fixture
against a misaligned neighbor (end coincident and recorded); the §11.2
invariant test extended to state the authored/inherited split; and the
no-fourth-state property (every authored boundary aligned, coincident,
or cause-carrying).
