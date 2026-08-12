# ADR-0023: A PART-009 deviation is authored, not inherited

- Status: Accepted
- Date: 2026-08-11. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-15_RECOMMENDATION_ROUND_2026-08-11.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.1.0 (minor under §0.1 — additions only; argued in
  Decision, with the major counter-argument recorded)
- Work packages blocked: WP-060's solver refusal case (SI-15 resolved;
  SI-16, SI-17, SI-24 unchanged and still gating their own increments)
- Requirement IDs: PART-009, PART-004, PART-005, PLAN-004, UI-005,
  Section 11.2
- Decision owners: Nate McBride

## Context

PART-009 permits alignment deviation only when published geometry
requires it or the user explicitly overrides, both recorded in the plan.
SI-15 filed the case that fits neither cause: a legacy MBR partition at a
non-1 MiB offset (the XP-era 63-sector start) grown at its tail. The
grow never touches the start, so the misalignment is neither created nor
curable by the operation — yet under a strict reading the finished
layout "deviates," and the only strict-reading exits are refusing the
grow or realigning the start, which forces a PART-005 data move the user
did not request. Section 11.2's "required alignment is preserved"
invariant inherits the same fork. The delivered solver (WP-060 increment
3) conservative-refuses the case by name, and its deviation-override
vocabulary is deliberately inexpressible until decided.

The ambiguity is one word: whether a **deviation** is a state the
finished layout is in, or an act the plan performs.

## Safety analysis

**A deviation is an act.** An authored boundary is one whose byte offset
the plan sets; PART-009's policy and its two deviation causes govern
authored boundaries and nothing else. A pre-existing boundary the plan
does not move — byte-identical before and after — is an inherited fact:
it demands no override, blocks no operation, and the plan records it in
its consequence text as a fact about the device, never as a grant by the
user. The override vocabulary exists for users overriding policy; a user
growing a partition is not overriding anything, and making them say they
are would corrupt the vocabulary's meaning on the day it is actually
needed.

**The filed case proceeds, and its safety story is complete.** Growing a
misaligned partition at its tail authors one boundary — the new end —
which follows the 1 MiB default. The untouched start is inherited; its
performance consequences existed before the plan and are unchanged by
it; the FS grow operates inside the partition and the table edit changes
the end entry. There is no data-safety failure to close against, which
is exactly why spending the fail-closed posture here was over-refusal.

**Coincident placement is conformant, or the issue re-files itself.**
Grow-to-fill sets the new end at the next partition's start or the
device end — authored in the trivial sense, chosen by nothing. Aligning
it down instead would mint an unusable sliver of free space. The rule:
an authored boundary placed coincident with a pre-existing structural
edge conforms to policy and is recorded as coincident. Without this,
resolving the start question leaves the end question as the same issue
under a new number. The rule is scoped to *pre-existing structural
edges*, so it licenses no free-floating off-policy placement.

**Severity stays honest in both directions.** Realignment remains
available only as an explicit PART-005 move at its own severity (3,
data-moving), its own consequence text, its own authorization tier
(ADR-0021). Nothing converts a grow into a move — the auto-realign
alternative is severity laundering, the silent-consequence shape this
register rejected in the 3.1.0 round's blank-media carve-out and every
appearance since. And nothing converts a move into a grow: the inherited
fact's recording never implies the plan will fix it.

**Section 11.2 needs no text change.** "Required alignment is preserved"
reads as the distinction implies: authored boundaries meet policy;
inherited boundaries are byte-identical before and after. A test proving
both proves the invariant.

**What a consumer and a plan may rely on:**

- A plan never changes a boundary it does not name; inherited boundaries
  are byte-identical before and after, and the §11.2 obligation covers
  exactly that.
- Every authored boundary meets the default, is coincident with a named
  pre-existing structural edge, or carries one of the two recorded
  deviation causes. There is no fourth state.
- A plan touching a device with inherited off-policy boundaries says so
  in consequence text (UI-005 displays it); the user sees the fact
  without being asked to authorize it as the plan's doing.
- A grow is never silently a move.

## Options considered

### Option (a) — strict reading: any plan touching an off-policy layout carries a deviation

Rejected. Semantically wrong — the user is not overriding a policy the
plan never applied — and operationally it gates routine maintenance on
override machinery built for a different act, locking the legacy MBR
population out of tail growth for no data-safety return. Its claimed
safety is theater: refusing a grow fixes no alignment.

### Option (b) — authored/inherited distinction (accepted)

Accepted, scoped as above, with the coincident-edge rule the adversarial
round forced (its sharpest finding: without it, the same issue re-files
about the end).

### Option (c) — auto-realign on grow

Rejected: an unrequested PART-005 data move smuggled inside a grow,
severity laundered from metadata-write to data-moving.

### Option (d) — refuse permanently

Rejected: over-refusal with no compensating guarantee — the misalignment
persists, the population is locked out, and the fail-closed posture is
spent on a case with no failure to close against.

Also rejected within the round: **typed carriage of alignment facts** (a
hashed-schema field duplicating offsets already present in the bound
snapshot would add only an agreement obligation — the ADR-0016/0021
lesson; consequence text serves the only consumers that exist), retained
as a revisit condition below.

## Decision

Option (b), landed as spec 12.1.0's amendment to PART-009 and only
PART-009. **SI-15 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** PART-009's two
pre-existing sentences stand verbatim; the authored/inherited scoping,
the coincident-edge rule, and the inherited-fact recording obligation
are additions; Section 11.2 is untouched; no existing MUST narrows.
ADR-0020's precedent establishes that selecting between readings of
existing text amends nothing by itself — the bump pays for the added
sentences. The counter-argument (disambiguation as semantic change, the
3.1.0 caution) was weighed and is recorded here so the numbering is
auditable; it was not taken because §0.1's rule turns on what happens to
existing requirement text, and none changes.

## Consequences

- **Positive.** The legacy misaligned population regains tail growth;
  the override vocabulary keeps its meaning for the day it is needed;
  severity honesty holds in both directions; the §11.2 invariant gains a
  precise reading.
- **Negative, accepted knowingly.** Inherited misalignment persists
  unbidden: the product will grow a partition whose performance-relevant
  start is off-policy, saying so rather than fixing it. The fix stays
  one explicit, honestly-priced move away.
- **For WP-060.** The solver's named refusal case unlocks: the
  misaligned-growth conflict becomes a permitted plan carrying the
  inherited fact. The code change rides the crate's next Rust increment
  (the `39b59f5` stopping-condition economics, with the SI-19 comment
  debt already riding the same increment); the deviation-override
  vocabulary stays deliberately inexpressible.

## Verification

Owned by WP-060 when the unlock lands, recorded here so none is
discovered late:

1. The grow-at-tail fixture on a 63-sector-start MBR image: authored end
   aligned, start byte-identical before and after, consequence text
   carrying the inherited fact.
2. The grow-to-fill fixture against a misaligned neighbor: end
   coincident with the neighbor's start and recorded as coincident.
3. The §11.2 invariant test stated as the split: authored boundaries
   meet policy, inherited boundaries byte-identical.
4. The no-fourth-state property: every authored boundary aligned,
   coincident, or cause-carrying — anything else unconstructible.

## Revisit conditions

- A consumer that must *query* alignment facts (rather than read
  consequence text) comes to exist; typed carriage then gets its own
  round with that consumer's requirements as evidence.
- The deviation-override vocabulary is designed (a user authoring an
  off-policy boundary on purpose); its round must preserve the
  authored/inherited boundary this ADR fixes, or amend this ADR first.
- PART-009's default changes (a different alignment quantum); the
  distinction survives, the constant does not.
