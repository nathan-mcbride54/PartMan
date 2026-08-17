# ADR-0050: A backing extent is framed on its named host — the last unconstrained frame

- Status: Accepted
- Date: 2026-08-17. Made on the measured round of 2026-08-17
  (`docs/reviews/ISSUE-365_HALF_B_ROUND_2026-08-17.md`), with a
  two-mutation battery, each proven applied and each killed. The
  reservation and its grant landed first in PR #423; the consumer-first
  adaptation of `crates/planner` landed in PR #424 before this act, in a
  form valid under both regimes. Merging is not acceptance; the decision
  owner has not been put the question in person, and this ADR is where it
  is put.
- Spec version: **17.1.0 — minor under §0.1.** What a body may say
  narrows; no numbered requirement's text changes.
- Work packages blocked: none. Issue #365 closes here — Half B decided,
  Part 1 corrected, Part 2's remaining half delivered, Part 3 recorded as
  already discharged by ADR-0045.

## Context

ADR-0046 enforced ADR-0037's anchoring rule for every kind but one. A
backing extent lies outside every containment forest —
`naming_referent_rule("backing-extent", "host")` is `ReferentRule::Open`,
so `named_position` returns `Outside`, `frame_root` returns `None`, and
the frame check never runs on it — and no edge kind may target one, so
`containment_agrees_with_extents` never sees it either. It was the single
node in the model whose declared frame nothing constrained.

Three acts since have had to pin limits around that hole:

- **ADR-0047** recorded that an aggregate is name-unrecoverable.
- **ADR-0049** gave the closure a hosting arm and had to record that its
  bound reads a frame nothing authenticates, pinning at measured cost
  that an authored frame could suppress the arm entirely — `Clear` 10/10
  over a live pool on a body that validated.
- **WP-060's planner coverage** (PR #421) found that on that same body
  the gate permits a wipe while the simulation removes the image and the
  volume, and leaves the live pool standing.

## The model already answered it

`ExtentLocator::Range` documents itself, verbatim: **"A byte range within
the host node's own address space."** `BackingExtent.host` names that
node, in a field `derive_id` hashes into the node's address. The question
"what frames a backing extent" was answered by the type all along. What
was missing was enforcement.

That is why this act is small, and why it is not a new decision: it is
ADR-0046's instrument applied to the kind ADR-0046 carved out.

## The decision

> **A backing extent that declares an extent is framed on the node its
> own name says hosts it.** One comparison beside ADR-0046's, refusing
> the same disagreement — two positional claims about one node that
> contradict — with `BackingExtentFrameDisagreesWithName` naming both.
>
> **Absence still admits.** A `Path`-located image has no contiguous
> device range and declares no extent; ADR-0049's arm already handles it,
> and this rule only constrains a fact that exists.

**What this does not do.** It does not put a backing extent into a
containment forest. `named_position` still returns `Outside`,
`frame_root` still returns `None`, no pair-table row moves, and
`naming_referent_rule` is untouched. ADR-0049's two measured-and-rejected
routes stay rejected. Only the node's own frame is pinned.

## Measured

At `b14c333`, three reds — and **every one of them a repair, not a loss**.

1. **ADR-0022's occupancy witness**
   (`occupancy_is_read_by_geometry_and_by_name`) rebuilds on a lawful
   body with one field change and still proves what it was written to
   prove: the frame arm finds an occupant nothing else does, because
   `names_within` reaches no backing extent — one is `Outside`, so
   `named_ancestry` is empty for it. The witness had been built on a body
   whose name said one host and whose extent said another, a
   self-contradiction that assembled **only** because of the carve-out.
   That the arm survives on an honest body is the load-bearing result
   here: had it not, this act would have made ADR-0022's frame reading
   vacuous, which is issue #401's shape.
2. **ADR-0046's enumeration**
   (`the_frame_rule_reaches_every_forest_at_every_depth`) is
   **strengthened**. Its assertion moves from `(17 * 20, 17 + 21)` to
   `(18 * 20, 18)`: the backing extent stops being the exception that
   admits all twenty-one candidate frames and joins every other node in
   admitting exactly one. The enumeration now contains no exception at
   all.
3. **ADR-0049's pinned limit** closes, and its regression is rewritten
   in place as the closure — the authored frame that suppressed the
   hosting arm is now refused at assembly.

**`crates/planner` needed consumer-first treatment**, which the grant
anticipated and withheld authority for. Its disagreement witness was
built on the body this act makes unlawful; PR #424 retired it first, in
a form green under both regimes, recording why an equivalent could not
be built. `crates/capability` is green unchanged.

**Mutation battery**, each applied with an editor and proven applied:

| # | mutation | outcome |
| --- | --- | --- |
| M1 | the rule removed | killed (2 tests) |
| M2 | the comparison inverted | killed (5 tests) |

## The spec price

**Minor under §0.1**, on ADR-0041's 13.1.0 and ADR-0046's 15.2.0
precedent: what a body may say narrows, which was previously unspecified
territory for this kind, and no numbered requirement's text changes. No
schema version moves; the golden vector is unmoved, since no committed
vector carries a backing extent with a disagreeing frame.

The closure is untouched — no arm, no bound, no seed — so §2.1's closure
sentence and ADR-0018's theorem are not in scope. Section 5's naming
paragraph gains the sentence, beside ADR-0046's.

## Issue #365's other parts

- **Part 1, corrected.** `Volume.producer`'s comment read "The producing
  aggregate or encryption layer", which is wrong against
  `endpoint_pair_allowed`'s `HostBacking` row and against
  `producer_verdict`; a backing extent produces a volume, which is how
  every host-backed device is modelled. `PartitionTable.parent`'s read
  "The device the table describes", wrong since ADR-0044 admitted volume
  and multipath-node parents. Both now point at the delivered predicate
  rather than restating a list that can drift from it.
- **Part 2, delivered.** The issue asked for a committed body exercising
  a genuinely extent-produced volume, and for `one_of_each`'s conflated
  volume to be considered for splitting. ADR-0045 committed the first;
  this session's ADR-0049 and WP-060 work put host-backed bodies through
  `validate_facts`, `affected_set`, `protection_gate` and `plan()`. The
  suite is no longer blind to the shape.
- **Part 3, already discharged** by ADR-0045, whose
  `naming_referent_rule` is the lawful-referent-kind table the issue
  asked for — derived from the pair table rather than authored beside it.
  Recorded as answered rather than left open.

## Consequences

- The model has no unconstrained frame left. Every node that declares an
  extent declares it in exactly one lawful frame, and a body saying
  otherwise is refused at assembly on both construction paths.
- ADR-0049's hosting arm keeps its bound, and that bound now reads a
  value the body format authenticates. The ADR's recorded limit is
  discharged rather than inherited.

## Verification

`cargo xtask ci` exit 0 at the act's head. Any claim that this ADR puts a
backing extent into a containment forest, changes an edge or naming rule,
alters the closure, or closes issue #319, is an error against this ADR.

## What stays open

- **Issue #319's authorization half**, untouched.
- **ADR-0047's named limit** — an aggregate carries no naming referents,
  so a signature whose `Backing` edge is omitted still leaves its
  aggregate unreached, and `destroyed_closure` reaches no aggregate by
  name either. This act does not touch it, and PR #424 records where it
  now lives.

## Revisit conditions

- A backing extent gains a locator kind that positions it somewhere other
  than its host's address space: the rule keys on `host` alone and would
  need to read the locator.
- `("backing-extent", "host")` is ever given a `Sources` rule: this rule
  becomes redundant with ADR-0046's and should be removed rather than
  left as a second comparison of the same claim.
