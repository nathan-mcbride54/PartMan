# ADR-0033: A derived property is a derivation, not an observation

- Status: Accepted
- Date: 2026-08-12. Acceptance basis: the decision owner's 2026-08-12
  directive ("let's cleanup the register residue", naming
  SI-13/SI-14/SI-28/SI-37 from the session summary it answered) on the
  adversarially reviewed recommendation round of the same day,
  following the eleven identical delegated arcs
  (`docs/reviews/SI-14_RECOMMENDATION_ROUND_2026-08-12.md` with the
  companion `docs/reviews/REGISTER_RESIDUE_SWEEP_2026-08-12.md`,
  untracked session artifacts; this ADR restates everything
  load-bearing from them).
- Spec version: 12.10.0 (minor under §0.1 — INV-004 gains a scoping
  clause and one new fail-closed prohibition; detection of the inputs
  and the duty to produce free extents and alignment survive verbatim;
  the major counter-argument is recorded in Decision)
- Work packages blocked: none — the delivered architecture already
  embodies the rule; the recorded obligations bind future assignments
  at their creation
- Requirement IDs: INV-004 (amended); MODEL-004, MODEL-005 (read, not
  amended — the rule is a reading of MODEL-004's existing "discovered
  property")
- Decision owners: Nate McBride

## Context

INV-004 lists free extents and alignment among the properties the
inventory detects. MODEL-004 requires every discovered property to
record the set of observations that produced it, with the four
confidence values derived from that set. Free extents and alignment
are not observed by any adapter; they are computed from properties
that are — partition extents and device geometry. SI-14 filed the gap:
no confidence value describes a computation, and no rule composes a
derived property's confidence from its inputs.

The filing predates the delivered architecture, which has since
answered the question in practice three times:

- **ADR-C4 (3.1.0):** confidence is derived from observations and
  never stored — the stored-confidence record is unrepresentable,
  proven by the absence of a constructor.
- **The WP-060 solver (delivered):** free extents are computed at
  planning time from the snapshot's body-carried, authenticated
  extents. Nothing stores them; nothing attaches confidence to them.
- **ADR-0023 (12.1.0):** a typed alignment-fact field was rejected
  outright — "the offsets are already in the bound snapshot; a
  duplicate field would add only an agreement obligation" — retained
  as that ADR's revisit condition.

SI-14's gate ("Later, WP-050") has therefore been reached and passed:
the consuming engine shipped without needing a derived-confidence
rule, because the architecture's answer is that no such rule exists.
This ADR records that answer before a discovery package implements the
literal reading and mints observation sets for computed values. The
gate misnomer is acknowledged: WP-050 consumes extents but reports no
INV-004 inventory; the true reporting consumers are WP-W100/WP-L100/
WP-M100, none yet created.

## Decision

**A property computed from other properties is a derivation, not an
observation.** MODEL-004's "discovered property" means an observed
one. A derivation — INV-004's free extents and alignment are the two —
is recomputed at use from the detected inputs it names, is never
stored, and carries no observation set and no confidence of its own.
Its trustworthiness is exactly its inputs', which carry the
observation sets. A surface that reports a derivation reports it as
one, naming its inputs.

**Fail closed on unfit inputs.** A derivation over an input whose
observation set derives `unavailable` or `conflicting` MUST NOT be
presented as a value; the input's own state is surfaced instead. An
`inferred` input yields a presentable derivation — the input's
confidence travels by reference, never by copy.

**No fifth confidence value, no composition algebra.** The absence of
a derived-confidence value is not a vocabulary gap; it is the
vocabulary saying derivations are not its subject.

**The landing is minor under §0.1.** INV-004's duty — the inventory
produces partitions, free extents, alignment, and the rest — survives
verbatim; the clause specifies only how two list items are produced,
which was never specified, and adds one prohibition. Nothing formerly
required is dropped; nothing formerly forbidden is permitted. The
major counter-argument — any disambiguation is a semantic change, the
3.1.0 caution — is recorded here and not taken, on the 12.1.0 and
12.3.0 precedents for scoping additions of exactly this shape.

**Scope.** The rule concerns MODEL-004 inventory properties. A plan's
own declared material (PLAN-004 severity and flags, PLAN-005's
cancellation class) is a declaration authenticated by the body hash
and re-run at the typed boundary — not a discovered property, and not
this ADR's subject.

## Options considered

### Option (a) — a composition rule (derived confidence computed from input confidences)

Rejected. MODEL-004 confidence describes observation trust, not
computational correctness: a composed value would be exactly as wrong
as a buggy derivation it described, while letting a record assert a
confidence its observations do not carry — the
assertable-independent-of-observations record ADR-C4 made
unconstructible. Computational correctness is held where the codebase
already holds it: determinism tests, the solver's placement fixtures,
the typed boundary's recompute.

### Option (b) — a fifth confidence value, `derived`

Rejected. A fifth stored value re-introduces stored confidence for a
class of records; every consumer must already know derived-ness from
the property's identity, so the tag informs nobody; and it repeats
the vocabulary-doubling shape ADR-C3 removed and ADR-0032 declined
for maturity.

### Option (c) — store derivations as observations of a synthetic "computation adapter"

Rejected. It fabricates provenance: an observation names a source
adapter, method, and outcome so a reviewer can weigh what was seen;
a computation saw nothing. It also creates the stored copy whose
agreement with its inputs must then be policed — ADR-0023's rejected
duplicate-field shape, generalized.

### Option (d) — the chosen rule

A derivation is recomputed at use, never stored, confidence-free, and
unpresentable over unfit inputs. Matches ADR-C4, the delivered solver,
and ADR-0023; adds exactly one new normative sentence (the fail-closed
presentation rule), which the adversarial round produced.

## Verification

1. **Delivered evidence, cited:** ADR-C4's constructor-absence proof
   (a stored confidence is unrepresentable); the WP-060 solver's
   free-extent computation from body-carried authenticated extents,
   held by its placement and refusal tests; ADR-0023's rejected
   alignment-fact field.
2. **Obligations recorded against future assignments** (the ADR-0030
   pattern — they land in each assignment at creation, and this ADR is
   the record the creation cannot omit): WP-W100, WP-L100, and WP-M100
   INV-004 surfaces present free extents and alignment as derivations
   — no observation set of their own, inputs named — and refuse
   presentation over an input whose observation set derives
   `unavailable` or `conflicting`, surfacing the input's state
   instead, with a fixture for each arm.
3. **No hashed surface moves:** the rule confirms the delivered
   no-field state; MODEL-003 versioning is not engaged; no vector
   changes.

## Revisit conditions

- A future consumer that must *persist* a derived value (for example,
  a cached free-extent map in a long-running inventory daemon) files
  its own round: persistence re-opens the agreement obligation this
  ADR's rule avoids, and ADR-0023's revisit condition already names
  the alignment-fact variant of that case.
- If a platform contract ever *observes* free extents directly (an
  adapter-reported value rather than a computation), that value is an
  observation under MODEL-004 unchanged — and its disagreement with
  the computed derivation is `conflicting` input evidence, not a
  contradiction of this rule.
