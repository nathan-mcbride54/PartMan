# ADR-0025: `irreversible-after-start` claims the mid-execution window; severity claims endpoints

- Status: Accepted
- Date: 2026-08-11. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-17_RECOMMENDATION_ROUND_2026-08-11.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.3.0 (minor under §0.1 — the flag had no prior
  definition to change; argued in Decision, with the major
  counter-argument recorded)
- Work packages blocked: WP-060's combination refusal (SI-17 resolved;
  SI-24 is that package's one remaining register gate)
- Requirement IDs: PLAN-004, PLAN-005, PLAN-008, UI-005, UI-009,
  HLP-003, Section 8, ADR-0021, ADR-0022
- Decision owners: Nate McBride

## Context

PLAN-004's severity 1 reads "Reversible — fully undoable before or after
apply via an emitted reversal plan." Its flag list names
`irreversible-after-start` and never defines it. Read as
"cannot be undone," the flag directly negates severity 1 while PLAN-004
declares flags orthogonal to severity; its relationship to PLAN-005's
`non-cancellable` class is unstated — cannot-stop and cannot-undo may or
may not be the same thing; and since UI-009 and HLP-003 key off severity
plus flags (with teeth since ADR-0021: any flag binds the interactive
ceremony), the model cannot silently decide whether the combination is
legal. The delivered planner refuses to emit it, by name, until the
register answers.

Two prior decisions frame this one. ADR-0022 fixed that a reversal
reverses a *completed* apply, mid-apply failure being Section 8's — a
temporal decomposition this decision stands on rather than invents. And
the 2.0.0 changelog records the oldest lesson in this territory: v1's
severity classes conflated severity with the security-sensitive
dimension, and the flags exist precisely to carry orthogonal dimensions
without corrupting the scale.

## Safety analysis

**The definition.** A step carries `irreversible-after-start` when a
reachable interrupted state exists from which the pre-step state cannot
be restored by unwinding: once the step's first write lands, stopping
cannot go back, and interruption recovery is roll-forward per the
journal (Section 8), never unwind. The criterion is a **reachable
unrestorable intermediate**, not the existence of a write. A journaled
chunk copy in the PART-005 shape — interruption always leaves the
original mapping or a fully-copied recoverable state — has windows but
no unrestorable intermediate: unflagged. An in-place multi-sector
rewrite whose half-state clobbers the original: flagged. The flag
partitions real steps, which is what a flag is for — the vacuity attack
is answered by the criterion, not by discipline.

**Severity 1's claim is untouched and remains true.** "Fully undoable
before or after apply" quantifies over endpoints: before the first write
(trivially) and after completion (via the emitted reversal draft,
ADR-0022's machinery). It has never claimed the mid-flight window — that
window is Section 8's, exactly where ADR-0022 drew the line. A
severity-1 step with the flag is therefore coherent: endpoints fully
undoable, mid-window roll-forward-only. **The combination is legal**,
and PLAN-004's declared orthogonality becomes true rather than
aspirational.

**The one coupling rule.** A flagged step's cancellation claims effect
`no-writes` only before its first write; after it, the honest outcomes
are `partial` (at a checkpoint) or completion. Section 8's cancel row
already offers exactly `no-writes` or `partial` — the rule selects
between existing values by the flag's own semantics, adding none. It
makes the journal's effect claim honest by vocabulary rather than by
discipline: a flagged step structurally cannot report a
post-first-write cancellation as if nothing happened.

**Cannot-stop and cannot-unwind are independent, in both directions.** A
checkpoint-cancellable step may carry the flag: you can stop at
checkpoints, and stopping does not restore the original. A
non-cancellable step may lack it: no safe stopping point, yet trivially
reversible endpoints. PLAN-005's three classes are untouched; the
vocabulary now states the distinction the filing said was unstated.

**The risk surface is already guarded; this decision adds no new
guard.** Any flag binds the interactive ceremony (ADR-0021), so the
combination can never be applied unattended. UI-005 displays severity
and flags as separate facts with the flag's own consequence line — the
honest display is both dimensions. The alternative, inflating severity
to encode the mid-flight property, is the 2.0.0 conflation returned,
and it lies in the other direction: a consumer reading the inflated
severity would conclude the *completed* effect is not undoable, which
is false.

**What a consumer and a plan may rely on:**

- Severity states what the completed effect does and whether it is
  undoable; `irreversible-after-start` states exactly one orthogonal
  fact: interruption past the first write recovers forward, never back.
- A flagged step never reports a cancellation as `no-writes` after its
  first write.
- A severity-1 flagged plan carries its truthful reversal draft, takes
  the interactive ceremony, and cannot be applied unattended.
- The severity scale carries no smuggled mid-flight semantics.

## Options considered

### Option (a) — the combination is illegal; the planner refuses permanently

Rejected on what forbidding forces: a genuinely endpoint-reversible
operation with an unwindable mid-window must claim severity ≥ 2 to be
representable (severity inflation as a modeling tax) or must shed the
flag (hiding the one fact the interruption case needs displayed). Both
corrupt a vocabulary to avoid defining a word — the 2.0.0 conflation
lesson repeated in reverse.

### Option (b) — define the flag as endpoint-irreversibility

Rejected: redundant where severity already speaks (3 and 4 carry the
loss-possible and destructive claims), contradictory at severity 1 by
construction, and dead weight everywhere — a definition chosen to
manufacture the conflict the filing asked about.

### Option (c) — the temporal definition, legality, and the coupling rule (accepted)

Accepted, scoped as above.

### Option (d) — drop the flag

Rejected: deletes the one word that warns the user about the
interruption window before Apply, and withdrawing a flag from
PLAN-004's list is a semantic change to existing text for the sake of
avoiding a definition that costs an addition.

## Decision

Option (c), landed as spec 12.3.0's amendment to PLAN-004 and only
PLAN-004. **SI-17 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** the flag had no prior
definition, so defining it changes no existing claim; severity 1's text,
the flag list, PLAN-005, and Section 8's rows all stand verbatim; the
definition, legality statement, and coupling rule are additions. The
counter-argument (a first definition fixes semantics other text already
depended on — the 3.1.0 caution) was weighed and is recorded so the
numbering is auditable; it was not taken because §0.1's rule turns on
what happens to existing requirement text, and none changes.

## Consequences

- **Positive.** The contradiction dissolves without touching severity
  1; the interruption-window warning survives as a displayed fact; the
  journal's cancel-effect claims become honest by vocabulary; the
  planner's refused combination unlocks.
- **Negative, accepted knowingly.** A plan can honestly read
  "Reversible" and still strand a device mid-flight in a
  roll-forward-only state — by design, with the flag's consequence line
  and the ceremony carrying the disclosure. The two-fact display asks
  more of UI-005 than a single scalar would; the single scalar was
  rejected because it lies.
- **For WP-060.** The combination refusal unlocks; the code change
  rides the crate's next Rust increment with the SI-19/SI-15/SI-16
  debts. Assigning the flag to concrete step families is each building
  package's per-step-type declaration, testable, when it builds steps.
- Nothing here is hash-visible: no field is added; the flag already
  existed in the plan body's vocabulary.

## Verification

Owned by the packages that build them, recorded here so none is
discovered late:

1. The combination constructs: severity 1 + flag, with its reversal
   draft, through the sole constructors — the planner's named refusal
   replaced by construction.
2. A flagged step's cancellation cannot claim `no-writes` after its
   first write — unconstructible, not discouraged.
3. The criterion's partition fixtures: a journaled-copy step unflagged,
   an in-place multi-sector rewrite flagged.
4. The ceremony binds on the flagged severity-1 plan (the ADR-0021
   flags-nonempty test extended to this combination).

## Revisit conditions

- SI-20's recovery design gives interrupted states a richer vocabulary;
  the roll-forward language here cites Section 8 as it stands and
  should be restated in the new terms, not weakened.
- PLAN-005's class vocabulary changes; the independence statement reads
  the current three classes.
- A step family is found whose interruption behavior fits neither arm
  of the criterion (restorable by unwind *sometimes*); that family's
  package files the case rather than picking a reading — §0.2's
  standing rule.
