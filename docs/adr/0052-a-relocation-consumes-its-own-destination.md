# ADR-0052: A relocation consumes its own destination — PART-005's destination vocabulary, and the consumed class it needs

- Status: Accepted
- Date: 2026-08-18. Made on the revision-2 round of 2026-08-18
  (`docs/reviews/ISSUE-371_DESTINATION_VOCABULARY_ROUND_2026-08-18.md`),
  whose revision 1 was broken by a six-lens adversarial pass and rewritten
  around the trilemma below; the decision owner answered every item of its
  §4 and every question of its §5 on 2026-08-18, and this ADR is where
  those answers are recorded. Merging is not acceptance of any code: no
  Rust lands with this document.
- Spec version: **17.3.0 — minor under §0.1.** PART-005 and every other
  spec sentence stand verbatim; the changed sentence is ADR-0018's
  consumed-class definition, amended in place, and the major
  counter-argument is recorded and declined below.
- Work packages blocked: none. WP-060's move increment is unblocked by
  this ADR and named as its obligation. Issue gitea#371 stays **open** —
  the byte-wise hosted-signature duty is delivered by the increment, not
  by this decision — and re-measures its dependency list against the
  result. **Issue #370 is not moved by any of this.**
- Requirement IDs: PART-005, PART-009, PLAN-002, PLAN-008, MODEL-002,
  §6, §11.2, ADR-0018, ADR-0019, ADR-0022, ADR-0025, ADR-0033, ADR-0036,
  ADR-0040, ADR-0041, ADR-0048, ADR-0051
- Decision owners: Nate McBride

## Context

PART-005 requires moves: "copy-then-commit where extents do not overlap,
and via journaled chunk copy with a durable progress map where they do".
At `f647baa` a request is `PlanRequest { operation, target }` — one node,
no second operand; the sized vocabulary is `Create { host, size }`,
`Grow { target, new_length }`, `Shrink { target, new_length }` — a size or
a length, never a **position**; and `Operation::Move | Operation::Copy`
refuse as `NotRepresentable` ("moves and copies need a destination
vocabulary this model does not carry yet", `crates/planner/src/lib.rs:376`).
PR #433 pinned that absence with `no_representable_request_relocates_bytes`
and a compile-time tripwire, and that test is the **sole** PART-005
evidence row in the tree (`docs/traceability/WP-060.md:25`).

Two conclusions of the round are load-bearing and were unchallenged:

- **A move step needs no new body field.** Its shape is the delivered one
  — `target` plus the three declared range sets of ADR-0018's effect
  table, which the body has carried since plan body version 4
  (`plan.rs:785-790`, `:951-952`). The vocabulary gap is at the request
  layer and in the solver.
- **The destination is reachable by the closure once declared**, because
  `seed_from_ranges` (`protection.rs:917-943`) reads ranges, not
  operations. Issue #370's "the closure cannot see the operation" is true
  and irrelevant here.

What decides the shape is a **trilemma** that no single reviewer
assembled and revision 1 posed nowhere. For an **overlapping** move — the
mode PART-005 names second and mandates — with source extent S and
destination extent D, S∩D ≠ ∅:

> **(i) Closure reach.** Every byte the journaled chunk copy rewrites —
> all of D, plus the released S\D — must lie in
> `destroyed ∪ consumed ∪ written`, or `seed_from_ranges` seeds no node
> those bytes touch and the protection closure never sees them.
> `seed_from_ranges` seeds only where a declared range intersects a
> node's **declared extent**; `descends_into` refuses a self-framed
> extent as a descent source unless it is the step's own target
> (`protection.rs:1081`); and `destroy()` fires only on the step target,
> the ADR-0048 identity arm, or by carry from an already-destroyed source
> (`:739-743`, `:767-771`). So a device-framed node inside S∩D is reached
> by **no arm** unless S∩D is in a declared set.
>
> **(ii) The delivered step-output contract.** `resolve_step_output`
> (`plan.rs:1428-1452`) destructures the forward step's `consumed` as
> **exactly one** range — `let [created_range] = …` — and matches world
> extents by **exact `HostRange` equality**. For the reversal draft to
> resolve the moved partition as the forward step's output, `consumed`
> must be `[D]`: the post-move extent, entire.
>
> **(iii) ADR-0018's consumed class.** ADR-0018:135-137 defines consumed
> ranges as "verified by the constructor to intersect no existing node's
> extent — Section 11.2's overlap invariant enforced at construction".
> When S∩D ≠ ∅, D intersects the moving partition's own pre-move extent,
> so `consumed = [D]` violates the definition as written.

**(ii) forces `consumed = [D]`. (iii) forbids it.** One of the three must
give.

| option | (i) reach | (ii) step output | (iii) ADR-0018 |
| --- | --- | --- | --- |
| **(a) precise** — `destroyed = S\D`, `consumed = D\S` | ✗ S∩D in neither set | ✗ sub-range of D, zero candidates | ✓ |
| **hybrid** — `destroyed = S`, `consumed = D\S` | ✓ | ✗ fails identically to (a) | ✓ |
| **(b) conservative** — `destroyed = S`, `consumed = D` | ✓ | ✓ | ✗ |
| **(b′)** — as (b), but widen `resolve_step_output` instead of amending (iii) | ✓ | ✓ (after change) | ✓ |

The hybrid is named because two independent verifiers proposed it as the
minimal correct repair and neither checked it against
`resolve_step_output`, where it fails exactly as (a) does.

One further measurement matters: **nothing enforces (iii) today.** The
sole step constructor, `PlanStep::mutating_declared` (`step.rs:437`),
runs the acknowledgment law and `affected_set` and performs no occupancy
check; `free_extents`/`place_create` (`solve.rs:614`, `:630`) take no
candidate range. The `StepRanges::consumed` doc-comment ("verified free by
the constructor", `protection.rs:620`) restates a promise the constructor
does not keep. The definition being amended is decided text with no
delivered enforcement behind it — which is not a licence to ignore it, but
is why amending it costs no code.

## Safety analysis

- **(i) is the safety property.** A range the closure never sees is a
  node the closure never protects; every option that fails (i) leaves a
  device-framed signature or file system inside S∩D — committed body
  content, `snapshot_tests.rs:1560-1571` — overwritable with a `Clear`
  verdict. That is the fail-open shape this repository refuses at every
  layer.
- **(ii) is delivered mechanism with a golden vector and a reversal
  contract (ADR-0022) behind it.** Widening it is possible ((b′)) but
  touches WP-010's plan boundary, changes what "the forward step's
  output" means for every step class, and re-opens ADR-0022's
  created-node resolution for a case that has exactly one lawful
  answer — the moved partition — which `consumed = [D]` already spells.
- **(iii) is decided text written before any relocation existed to test
  it.** Its purpose — no step may claim free bytes that belong to
  someone else — survives intact under the amendment, because the
  exception admits only the moving target's own bytes and those of nodes
  named within it.

Under (b), the authenticated closure — not an unauthenticated solver
predicate — is what refuses a destination laid over a device-framed node
in S∩D. Under (a) that refusal would live only in the solver, outside the
hash and outside HLP-002's recomputation. That is a second, independent
argument for (b) over (a).

## Options considered

### Option A — (a) precise, or the hybrid

Rejected: fails (ii). Under either, `emit_reversal(…)?` at `lib.rs:860`
refuses **the whole forward plan**, because D5's `MoveDraft` cannot spell
its target. (a) additionally fails (i).

### Option B — (b) conservative, amending ADR-0018 (iii)

**Adopted.** `destroyed = S`, `consumed = D`, `written = T`. The consumed
class gains a narrow, statable exception; nothing else moves.

### Option C — (b′), keeping (iii) and widening `resolve_step_output`

Rejected, and recorded because it is not obviously wrong. It preserves
ADR-0018's sentence and would let `consumed = D\S` (or a multi-range
`consumed`) resolve. But it changes a WP-010 contract with a golden
vector for the benefit of a case that has one lawful answer already
expressible; it moves the "which node is the output" decision from exact
extent equality to a containment or union rule that ADR-0022 never
priced; and it leaves (iii)'s promise unenforced exactly as before, so
the sentence it preserves would still be untrue of the delivered
constructor. Amending decided text openly is preferred to preserving text
the code does not honour.

### Option D — a stored copy mode, or a free-extent destination reference

Both rejected on grounds already decided: a stored mode that could
disagree with the ranges is the authored-value-never-validates class
ADR-0041 refuses; a free-extent reference names a derivation "recomputed
at use, never stored" (ADR-0033) and does not survive to the body.

## Decision

**D1 — the destination.** `Move { target, new_start: u64 }`: an authored
byte offset in the host's address space. `Copy` is out of scope — a
separate CAP-002 operation with its own content class, cross-device
identity story and §11.2 invariant.

**D2 — the declaration.** `destroyed = S` (the whole source extent),
`consumed = D` (the whole destination extent), `written = T` (the host's
table extents). ADR-0018:135-137 is amended so that:

> A consumed range is verified to intersect no existing node's extent,
> **except that a relocation's consumed range may intersect the extent of
> the step's own target and of nodes named within it** (`names_within`,
> `protection.rs:491-493`) **— and no other node's.**

The canonical request-less entry (`capability.rs:181-186`'s split for
shrink) stays as delivered: whole target extent destroyed, conservative
because it cannot know the geometry.

**D3 — what "free" means for a destination.** `D ⊆ free_extents(host) ∪ S`,
aligned per PART-009, inside the host, outside ADR-0036's scheme-claimed
regions, and **not intersecting the extent of any node that is neither the
target nor named within it** — the same predicate as D2's exception. The
literal form "not intersecting any *other* node's extent" was measured to
refuse the ordinary partition-with-device-framed-ext4 fixture
(`planner/src/tests.rs:1302-1312`) on every downward overlapping move and
is rejected. **Named residue:** an extentless hosted node is subtracted by
neither `free_extents` nor this clause — ADR-0051's pinned #319 shape-3
limit, inherited and stated.

**D4 — the copy mode is derived**, from `S ∩ D ≠ ∅`; no stored mode.

**D5 — the reversal.** `ReversalMaterial::MoveDraft { source: S,
destination: D }`, target spelled as the forward step's output (which
`consumed = [D]` resolves under the unchanged `resolve_step_output`), with
a `DraftPrecondition` that S\D is unoccupied at the reversal's own
validation. A moved partition renaming is decided, legitimate behaviour
(ADR-0019:93-96), not a destruction to enumerate.

**D6 — the hosted-signature enumeration.** ADR-0018's relocation duty —
"preserve hosted signatures byte-wise or enumerate their loss explicitly
in the plan" — is discharged **by naming position, not by family**. The
round said "by frame"; under ADR-0046 that word cannot carry the
distinction, because every containment child's extent is expressed in
the containment root's address space — a partition's own file system is
device-framed exactly as a device-hosted signature is
(`planner/src/tests.rs:1302-1304` says so of the solver fixture). What
separates them is `names_within` (`protection.rs:491-493`), the same
predicate D2 and D3 already use: a node **named within the moved
target** — a file system or signature whose `host` chain reaches it —
moves with it, its offsets relative to the target's content, and is
preserved by a byte-preserving copy; a node **not named within it** whose
extent lies in S is not carried — inside S∩D the destination rule (D3)
refuses the move outright, and inside S\D it is released with the source
range and, release being destruction (ADR-0018), **is destroyed, and the
authenticated closure is what says so** under D2 wherever the solver is
not the arbiter. The planner's `Consequence` vocabulary gains a variant
enumerating exactly that release, at kind level (`fs_kind`, signature
family, and the node's kind for anything else). **Its negative space is bounded
here, explicitly:** the vocabulary names position dependence *where the
bound snapshot can see it*; it carries no partition type or role, and its
silence on a bios_grub partition or an ESP is **not** a boot-consequence
verdict and asserts nothing. **The "checking boot consequences"
justification is dropped**, not gated on a future INV-004 dependency.
Carriage was *delivered-in-planner, pending-in-body* at this ADR's
acceptance and is **delivered** since 2026-08-18: "in the plan" is §6's
hashed consequence-text item, and the jointly-sequenced consequence-text
slice (WP-010 slice 3p, plan body version 5; WP-060 increment 12) states
the vocabulary's sentences into it as a canonical set.

**D7 — where it lands.**

- **WP-060:** `SizedRequest::Move { target, new_start }`, the D3 rule,
  the D2 declaration, `MoveDraft`, the `Consequence` variant, and the
  tripwire `no_representable_request_relocates_bytes` taken down **in the
  same change as its producer**, with a **named replacement PART-005
  traceability row** — the row it replaces is the only one PART-005 has.
- **WP-010:** no body field, no contract change. Independently, the stale
  `StepRanges::consumed` doc-comment at `protection.rs:620` is corrected
  to say where freeness is actually enforced (the solver, D3) — a
  documentation-versus-code defect owed whether or not a move lands, and
  Rust, so it owes its own sitting when it moves.
- **WP-050 — a stated obligation, not solved here:** a forward step
  ordered after a move needs a target spelling that survives the
  re-addressing. Each plan binds one snapshot today and no delivered
  boundary breaks; the pre/post address mapping is named as WP-050's when
  multi-step plans over a relocation exist.
- **WP-070 — a named dependency, deferred:** §11.2:894's interrupted-move
  invariant is an apply-time obligation of the executing packages; WP-060
  holds PART-005's planning half only. The move *request* lands without
  waiting on an executor; no apply of a move exists until WP-070 owns
  the invariant.
- **Sitting:** any Rust in the increment trips WP-020's stopping
  condition; the PR body names its sitting before merge.

## The spec price

**Minor under §0.1.** PART-005 stands verbatim; so does §11.2's "Partition
extents do not overlap" (`:884`), which constrains *nodes* and is
untouched by a *step range* intersecting the moving node's own pre-move
extent — after the plan, the partition sits at D and nothing sits at S\D.
No requirement's text narrows or changes meaning. The changed sentence is
ADR-0018's, amended in place with a §0.3 row, in the ADR-0040 shape.

**The major counter-argument, recorded and declined.** One could read
ADR-0018's consumed class as load-bearing for §11.2's overlap invariant —
"enforced at construction" — so that admitting any overlap is a semantic
change to how §11.2 is discharged. Declined: the constructor enforces
nothing of the kind today (`step.rs:437`), the invariant is over node
extents not step ranges, and the exception admits only the target's own
bytes, which no other node can hold under §11.2 in the first place.
Revision 1's "minor" was staked on a reach argument that the adversarial
pass broke; this pricing rests on the sentence test instead.

## Consequences

- WP-060's move increment is unblocked and fully specified above; it
  owes the replacement traceability row, the tripwire's removal beside
  its producer, and its sitting.
- ADR-0018:135-137 reads with the amendment note; ADR-0040's retirement
  note beneath it stands.
- The `Consequence` variant's silence is bounded in text, so a UI or CLI
  surface may never present "no consequences enumerated" as "no position
  dependence exists".
- Issue #371's blocker list is re-measured against the increment, not
  closed here.

## Verification

Owed by the increment, not this document: (1) the overlapping-move
fixture with a device-framed signature in S∩D refuses through the
closure, not the solver; (2) the same fixture with the signature named within the
moved node is `Clear` and the simulation preserves it at the
destination; (3) `MoveDraft`
resolves under the unchanged `resolve_step_output`; (4) the D3 rule
admits the ordinary partition-with-its-own-ext4 downward move it was
measured to refuse under the literal form; (5) the tripwire's
successor row traces PART-005 to a test that exercises a move; (6)
mutations proven applied and killed per the standing rule.

## What stays open

- **#371** — the duty was delivered by WP-060 increment 11 and its
  last rider — "in the plan" being the hashed body's §6 consequence text
  — by the jointly-sequenced consequence-text slice (WP-010 slice 3p,
  plan body version 5, and WP-060 increment 12; PRs #461/#462, arc head
  `0378bd5`, 2026-08-18). D6's carriage sentence reads *delivered*;
  the issue closed on that merge.
- **#370** — unmoved; a byte-preserving relocation of a *protected*
  structure still refuses, and relief still needs a preservation proof
  this vocabulary does not supply.
- The extentless-hosted-node residue under D3 (ADR-0051's pinned limit).
- The WP-050 obligation and the WP-070 dependency above.

## Revisit conditions

- A fixture showing D2 over-refusing a lawful move that (a) would permit,
  where the refusal is *not* warranted — the round found the converse
  twice; the symmetric case was not tested.
- The consequence-text body slice landing, which removes D6's
  "pending-in-body" qualification.
- WP-070 owning §11.2:894, which converts the deferral into a delivery.
- Any reading under which `consumed = D` needs no exception — e.g. if
  "existing node's extent" is read as excluding the step's own target by
  construction — which would make the amendment editorial rather than
  substantive; the text does not say so today.
