# Register residue sweep — 2026-08-12

**Status: a verification record**, produced under Nate's directive
"let's cleanup the register residue" (SI-13, SI-14, SI-28, SI-37).
Untracked session artifact under `docs/reviews/**`. SI-14's full
recommendation round is its own document
(`SI-14_RECOMMENDATION_ROUND_2026-08-12.md`); this document records
that the other three were measured against the current tree and what
was found.

## SI-13 — identity binding for pool and array write targets

**Finding: stays Later (WP-L110); the gate is accurate; no edit
proposed.**

Measured against the tree:

- Identities bind at **validation**, the helper's act: every delivered
  planner path emits an empty identities map, and plan-body.md §1 says
  a draft "binds identities at validation". The consumer of SI-13's
  answer is a validate-plan surface, which first exists in the helper
  era (WP-W110/L110/M110).
- Aggregates are not plannable write targets today: the delivered
  `Operation` vocabulary is closed and versioned (ADR-0031), the
  LVM/mdraid workflows (LIN-004/005) have no operations in it, and
  extending it is "WP-050's next reviewed increment" by that ADR's own
  grant language. The conservative refusal is structural, not
  discretionary.
- The ADR-0031/0032 grant explicitly reserved SI-13 ("grants no
  authority … to decide SI-13 or SI-14"), so nothing delivered has
  edged into it.

Deciding now would design a validation surface that does not exist
against helper contracts that WP-040's route decisions have not yet
shaped — deciding on guesses, the thing the register refuses. The
architecture even suggests the eventual shape (the affected-set
closure already names the member devices a step reaches; per-record
strength is ADR-C3's), which is exactly why the decision will be
cheap when its consumer arrives and is not needed before.

## SI-28 — a card reader's serial identifies the transport, not the medium

**Finding: stays Mitigated-open; the floor is in force and its record
is current; no edit proposed.**

- The interim conservative floor (destructive whole-device operations
  on removable media behind a bridge exposing no medium-attributable
  identifier are refused) is computable from decided facts and remains
  the recorded posture since the 2026-08-09 reclassification.
- The relaxation route is unchanged and unmet: ADR-0017's revisit
  condition requires **apparatus-qualification evidence** and its own
  round, and no such measurement exists. Producing it is a hardware
  campaign (the Windows PnP measurement environment of the original
  filing, the authorized fixture media), not a documentation cleanup —
  it should be its own directed arc if and when wanted.
- Part 7's warning against false closure stands; nothing in the
  PLAN-005 or WP-070 arcs touched the affected population.

## SI-37 — no fail-closed rule for unassembled paths with unequal identifiers

**Finding: stays Open/Later by its own terms; the record is current;
no edit proposed.**

- The issue's own evidence clause forbids accepting any option before
  the per-platform dual-path matrix with negative controls exists. No
  such measurement exists, and no contemplated spec change moves a
  closure-blocked multipath-capable population to `Permitted` — the
  pin's condition is not approaching.
- The filed population is typed and fail-closed today (ADR-0018's
  transport arm: `blocked`/`Indeterminate`), so the open issue guards
  a future relaxation, not a present hole.
- Producing the matrix (one LUN on two real or faithfully virtualized
  paths, framework assembled and absent, across churn and reboot) is
  a measurement campaign of its own; nothing requires it yet.

## Summary

One of four is ripe: SI-14, whose "Later (WP-050)" gate has been
reached and passed by delivered work that embodies an answer — the
recommendation round proposes recording it as ADR-0033 with a minor
INV-004 amendment. SI-13's gate is verified accurate; SI-28 and SI-37
are evidence-gated by their own recorded terms, and their relaxation
campaigns are separate hardware/measurement arcs to direct
deliberately, not documentation debt.
