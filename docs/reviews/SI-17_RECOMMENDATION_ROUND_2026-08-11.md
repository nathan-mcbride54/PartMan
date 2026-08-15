# SI-17 recommendation round — 2026-08-11

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-17 stays Later (WP-060) until a decision is recorded
through a WP-010 spec change with an ADR, the established shape. This is
an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-17, an early filing
with no options recorded. This round constructs the option space as well
as recommending from it.

---

## The conflict, made precise

> **PLAN-004:** severity 1 — "Reversible — fully undoable before or after
> apply via an emitted reversal plan." Each step additionally carries
> orthogonal **flags**: […] `irreversible-after-start` […]

> **PLAN-005:** Each step MUST declare one of: cancellable,
> checkpoint-cancellable, or non-cancellable.

> **Section 8:** Executing → Cancelled — "Cancel honored at a safe point
> (PLAN-005) after journaled unwind — effect `no-writes` or `partial`."

The filing's three complaints, all real: the flag is **never defined**;
read as "cannot be undone," it directly negates severity 1's full
undoability while PLAN-004 declares flags orthogonal to severity; and its
relationship to PLAN-005's `non-cancellable` is unstated — cannot-stop
and cannot-undo may or may not be the same thing. Since UI-009 and
HLP-003 key off severity plus flags (now with teeth: ADR-0021 binds the
interactive ceremony on any flag), the model cannot silently decide
whether the combination is legal — and the delivered planner refuses to
emit it, by name, until the register answers.

What later resolutions handed this round: ADR-0022 fixed that a reversal
reverses a **completed** apply, with mid-apply failure explicitly Section
8's — a temporal decomposition this round can stand on rather than
invent. And the 2.0.0 changelog records the register's oldest lesson in
this territory: v1's severity classes "conflated severity with the
security-sensitive dimension," and the flags exist precisely to carry
orthogonal dimensions without corrupting the scale.

## Recommendation: define the flag temporally — the combination is legal, because the two texts speak about different time windows

**`irreversible-after-start` is a claim about the mid-execution window;
severity is a claim about endpoints. Defined that way — and it has never
been defined any way — the contradiction dissolves and PLAN-004's
declared orthogonality becomes true rather than aspirational.**

1. **The definition.** A step carries `irreversible-after-start` when
   there exists a reachable interrupted state from which the original
   pre-step state cannot be restored by unwinding: once the step's first
   write lands, stopping cannot go back, and interruption recovery is
   roll-forward per the journal (Section 8), never unwind. A step whose
   every interruption resolves to "landed entirely or not at all" (the
   atomic single-sector write) does not carry the flag — the criterion
   is a reachable unrestorable intermediate, not the existence of a
   write.
2. **Severity 1's claim is untouched and remains true.** "Fully undoable
   before or after apply" quantifies over endpoints: before the first
   write (trivially) and after completion (via the emitted reversal
   draft, ADR-0022's machinery). It has never claimed the mid-flight
   window — that window is Section 8's, exactly where ADR-0022 already
   drew the line. A severity-1 step with the flag is therefore coherent:
   endpoints fully undoable, mid-window roll-forward-only. The
   combination is **legal**, and the planner's named refusal unlocks.
3. **The one coupling rule, stated where the flag is defined:** a
   flagged step's cancellation can claim effect `no-writes` only before
   its first write; after it, the honest outcomes are `partial` (at a
   checkpoint) or completion. Section 8's cancel row already offers
   exactly `no-writes` or `partial` — the rule selects between existing
   values, adding none. PLAN-005's classes are otherwise independent of
   the flag in both directions: a checkpoint-cancellable step may carry
   the flag (you can stop at checkpoints; stopping does not restore),
   and a non-cancellable step may lack it (no safe stopping point, yet
   trivially reversible endpoints). Cannot-stop and cannot-unwind are
   different facts, and now the vocabulary says so.
4. **The risk surface is already guarded, and this round adds no new
   guard.** Any flag binds the interactive ceremony (ADR-0021), so the
   combination can never be applied unattended; UI-005 displays severity
   and flags as separate facts with the flag's own consequence line —
   the honest display is both dimensions, never a collapsed worst-case
   scalar, which would be the 2.0.0 conflation returned; and the
   severity-1 reversal draft obligation (ADR-0022) stands, its
   completed-apply boundary agreeing with this definition rather than
   straining against it.

## What a consumer and a plan may rely on

- A step's severity states what its *completed* effect does and whether
  it is undoable; its flags state orthogonal facts, and
  `irreversible-after-start` states exactly one: interruption past the
  first write recovers forward, never back.
- A flagged step never reports a cancellation as `no-writes` after its
  first write — the journal's effect claim is honest by vocabulary, not
  by discipline.
- A severity-1 flagged plan still carries its truthful reversal draft,
  still takes the interactive ceremony, and can never be applied
  unattended.
- The severity scale carries no smuggled mid-flight semantics; a
  consumer reading severity 1 reads endpoint reversibility, full stop.

## The adversarial round

**Attack 1 — "the temporal definition makes the flag vacuous: every
multi-write step has an interruption window, so every real step is
flagged."** Refuted by the criterion the attack forced into point 1: the
flag marks a *reachable unrestorable* intermediate, not the existence of
writes. A journaled chunk copy in the PART-005 shape — interruption
always leaves the original mapping or a fully-copied recoverable state —
has windows but no unrestorable intermediate: unflagged. A multi-sector
in-place rewrite whose half-state clobbers the original: flagged. The
flag partitions real steps, which is what a flag is for.

**Attack 2 — "a plan labeled Reversible that can strand a device
mid-flight misleads the user; legality launders the danger."** Refuted
by locating what informs. The user reads UI-005's display of both facts —
severity 1 *and* the flag's consequence line — under an interactive
ceremony that the flag itself binds (ADR-0021). The alternative the
attack implies, inflating severity to encode the mid-flight property, is
the exact dimension-conflation 2.0.0 unwound, and it lies in the other
direction: a consumer reading the inflated severity would conclude the
*completed* effect is not undoable, which is false. Honesty here is two
orthogonal facts displayed as two facts.

**Attack 3 — "illegality is simpler: just forbid the combination and be
done."** Rejected on what forbidding forces. A genuinely
endpoint-reversible operation with an unwindable mid-window must then
claim severity ≥ 2 to be representable — severity inflation as a
modeling tax — or the flag must be dropped from it, hiding the one fact
the interruption case needs displayed. Both corrupt a vocabulary to
avoid defining a word.

**Attack 4 — "the cancel-effect rule is PLAN-005's business; writing it
into PLAN-004's flag definition is cross-requirement creep."** Refuted by
scope arithmetic: the rule adds no cancellation class, changes no
Section 8 row, and selects between Section 8's two existing effect
values by the flag's own semantics. It lives with the flag because it
*is* the flag's semantics applied to cancellation; PLAN-005 keeps its
three classes untouched.

**Attack 5 — "roll-forward recovery presumes journal machinery that does
not exist yet."** Refuted by kind: the flag describes the step's nature
(what interruption exposes), citing Section 8's existing recovery
design; it implements nothing and waits on nothing. WP-070 builds the
journal under the same Section 8 either way.

**Attack 6 — "this decides SI-20's RecoveryRequired exits or SI-24's
preview parity."** Refuted by scope, stated: nothing here touches the
transition table, recovery-plan semantics, or preview. The definition's
roll-forward language points at Section 8; it does not resolve Section
8's own open exit question.

## Rejected, and why — to be recorded with the decision

- **(a) The combination is illegal; the planner refuses permanently.**
  Attack 3's costs: severity inflation or flag suppression, each
  corrupting one vocabulary to avoid defining another — the 2.0.0
  conflation lesson repeated in reverse.
- **(b) Define the flag as endpoint-irreversibility.** Makes it
  redundant where severity already speaks (3 and 4 carry the
  loss-possible and destructive claims), contradictory at severity 1 by
  construction, and dead weight everywhere — a definition chosen to
  manufacture the conflict the filing asked about.
- **(c) is the recommendation** — the temporal definition, legality,
  and the single coupling rule.
- **(d) Drop the flag.** Deletes the one word that warns the user about
  the interruption window before Apply — UI-005's display loses exactly
  the fact that distinguishes a strandable step from a clean one — and
  withdrawing a flag from PLAN-004's list is a semantic change to
  existing text for the sake of avoiding a definition that costs an
  addition.

## Deliberately not decided

SI-20 (RecoveryRequired exits) and all REC-*/Section 8 recovery design;
SI-24 (preview versus dry-run); PLAN-005's class vocabulary (untouched);
which concrete step families carry the flag (each package declares per
step type, testably, when it builds steps).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0025 is the next free number; reservation PR
before resolution PR, the established shape), amends **PLAN-004 only** —
the severity scale and the flag list stand verbatim; the flag's
definition, the legality statement, and the cancel-effect coupling rule
land as additions where the flag is named — bumps **minor** (12.3.0: the
flag had no prior definition to change, severity 1's text is untouched,
PLAN-005 and Section 8 are untouched), and moves SI-17 to Resolved. The
major counter-argument is recorded for the decision to overrule with.
WP-060's re-attribution follows (the established shape): the planner's
named combination refusal unlocks, the code change riding the same
future Rust increment as the SI-19/SI-15/SI-16 debts.

Verification obligations for the ADR, owned by the packages that build
them: the combination constructs (severity 1 + flag, with its reversal
draft, through the sole constructors); a flagged step's cancellation
cannot claim `no-writes` after its first write (unconstructible, not
discouraged); an unflagged journaled-copy step and a flagged in-place
rewrite as the partition fixtures for the flag's criterion; and the
ceremony binding on the flagged severity-1 plan (the ADR-0021 test
extended to this combination).
