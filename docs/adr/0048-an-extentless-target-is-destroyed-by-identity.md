# ADR-0048: An extentless target is destroyed by identity — its whole frame, and the seed's second source

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-392_IDENTITY_DESTRUCTION_ROUND_2026-08-16.md`,
  single-author with a five-mutation battery, each proven applied; three
  killed, one killed only after the round added the regression it was
  missing, and one recorded as a survivor that is a proof). The
  reservation and its grant block — which lifts, for this act only, the
  standing denial on altering `canonical_ranges` and the closure's reach —
  landed first in PR #413. Merging is not acceptance; the decision owner
  has not been put the question in person, and this ADR is where it is put.
- Spec version: **16.0.0 — major under §0.1.** The argument, and the
  rejected minor reading, are below.
- Work packages blocked: none. Issue #392 closes here as filed, and
  ADR-0044's named limit with it. The planner simulation coverage this
  population lacks is **WP-060's own pull request under its own grant**
  and is named below as an obligation, not discharged here.

## Context

An extentless target — a `Volume`, an `Aggregate`, an `EncryptionLayer`,
a `MultipathNode`, for which `NamingFields::may_carry_extent` is false —
has no extent for `canonical_ranges` to name. Measured at `d204af6` on
the committed `partitioned_mdraid` fixture, a complete ADR-0046-lawful
body with nothing omitted:

```
canonical_ranges(Wipe, md0) = destroyed 0, written 0, consumed 0
affected_set(Wipe, md0)     = { md0, table }
protection_gate(md0, op)    = Clear on 10 of 10 mutating operations
protection_gate(array, op)  = Clear on 10 of 10
```

A live ZFS pool sits on `md0p1`, on the table `md0` carries. The step
declares nothing destroyed, so the target is never destroyed, so
destruction never carries, so the table is never destroyed, so ADR-0043's
release never fires and the pool is never reached. Every layer behaves as
specified; the defect is that the entry is empty.

## The decision

> **1. An extentless target's canonical destroyed entry is its whole
> frame.** For `Wipe`, `Encrypt`, `Move` and `Shrink`, where the target
> declares no extent, the entry is `HostRange { host: target, start: 0,
> length: u64::MAX }` — every byte expressed in the target's own address
> space, and by `HostRange::intersects`' frame equality, no byte in any
> other.
>
> **2. The destroyed-target seed gains a source.** A target that declares
> no extent, named by a destroyed range framed on itself, is destroyed by
> identity.

Both are load-bearing, and the measurement is what establishes it. Rule 1
alone range-destroys what is framed *on* the target — a volume's table,
partition and signature — and so closes `Wipe(volume)`. It leaves
`Wipe(aggregate)` at `Clear` 10/10, because **nothing is framed on an
aggregate**: its content is framed on the volume it produces. Only rule 2
destroys the aggregate itself, which is what lets ADR-0039's downward
production descent carry to that volume. The filed candidate was rule 1
alone and does not close the defect its own issue title states.

## Measured

At `d204af6`, in a detached worktree outside the checkout.

| target | before | after |
| --- | --- | --- |
| `md0` (volume) | `Clear` 10/10 | `Clear` 6/10 — the four destroying operations refuse |
| `array` (aggregate) | `Clear` 10/10 | `Clear` 6/10 |
| `Label(sda)` control | `{array, md0, md_signature, sda, table}` | **byte-identical** |
| `Wipe(sda)` control | reaches the pool | **byte-identical** |

The six operations that write and destroy nothing are unchanged and still
gate `Clear` on the volume — ADR-0039's distinction between reach and
destruction, deliberately untouched, and the reason the regressions name
four operations rather than ten.

**Cost: one red across the workspace** — ADR-0044's pinned limit row,
rewritten in place so the discharge is visible in the diff.
`cargo xtask ci` exit 0.

**Mutation battery**, each applied with an editor and proven applied:

| # | mutation | outcome |
| --- | --- | --- |
| M1 | the whole-frame entry removed | killed |
| M2 | the identity seed removed | killed (3 tests) |
| M3 | the entry's length set to zero | killed |
| M4 | the absent-extent guard dropped from the seed | **survives** — proof below |
| M5 | the seed fires on any non-empty destroyed set | **killed, but only after this round added the regression that catches it** |

**M4 is a survivor that is a proof.** Dropping `facts.extents.get(&target).is_none()`
leaves `ranges.destroyed.iter().any(|range| range.host == target)`, which
for an extent-bearing target is true only where the target's extent is
framed on itself — that is, a frame root, a physical device. A device's
canonical entry *is* its self-extent, and a range always intersects
itself (ADR-0041 refuses a zero-length extent), so `range_destroyed`
already contains it and the first clause of the seed has already fired.
The guard is therefore redundant on every constructible body. It is kept
because its redundancy is a property of what `canonical_ranges` currently
emits, not of this function, and a guard whose necessity depends on
another function's behaviour is the kind of reasoning that rots.

**M5 is the round's own finding.** It survived the committed suite because
`canonical_ranges` always frames its whole-frame entry on the target, so
"any destroyed range" and "a range framed on the target" are
indistinguishable through the gate. But `affected_set` is `pub` over
caller-supplied `StepRanges`, and a plan step declares its own ranges: a
step destroying bytes in another frame would have destroyed an unrelated
extentless target by identity. The round added
`the_identity_seed_is_frame_equal_not_merely_non_empty`, which pins the
distinction at the closure, and M5 is killed by it. A mutation that
survives is a coverage report, not a licence.

## The spec price

**Major under §0.1**, and the amendment is the **seeding change**, not
the entry. Two normative texts move, in the 15.0.0 shape:

1. **§2.1's seeding sentence** gains the second source.
2. **ADR-0018's theorem membership sentence** (`0018:194-222`), which
   already carries three parentheticals — 13.0.0 (ADR-0039), 14.0.0
   (ADR-0043), 15.0.0 (ADR-0044) — gains a fourth in the same form.

**The rejected minor reading, recorded.** It is arguable that the
whole-frame entry alone is an addition to previously unspecified
territory: no requirement says what an extentless target's entry is,
because the case was never considered, and the entry falsifies nothing —
a volume's frame holds only its own content, and ADR-0042 already
licenses a target's own address space as a descent source, which is
exactly the exception that makes a whole-frame entry on the target safe
rather than sibling-capturing. That reading is **correct about the entry
and wrong about the act**: rule 2 changes when a node is destroyed, which
is what the closure's membership theorem is about, and a membership
change is not an addition. The act is priced on rule 2.

**An editorial correction rides with the act, and is not the act's own
claim.** "The operation's minimal invariant ranges" is **not ADR-0018
text**. It occurs four times repository-wide — `CHANGELOG.md:2298`,
`crates/domain/src/model/capability.rs:8` and `:140`, and
`docs/adr/0042-frame-roots-are-never-written-wholesale.md:23` — and
`capability.rs:140` attributes it to "ADR-0018's canonical-step rule",
which ADR-0018 does not contain. Issue #392's body inherits the
misattribution from that comment. The entry was already non-minimal
before this act: ADR-0038 made `Shrink` and `Move` declare the whole
target extent rather than the freed tail. The code comments are corrected
here; ADR-0042's sentence is an accepted ADR's own text and is cited as
the gloss's origin rather than rewritten. The invariant that does hold,
and is now stated in the comment, is **conservatism**: an entry may
over-approximate what an operation touches, never under-approximate it.

The same comment's claim that "a create writes the host's table extents
and consumes an unspecified free range" is corrected in the same pass: the
delivered code declares the target's own extent filtered by
`extent.host != target`, and hardcodes `consumed: vec![]`. That was an
error against ADR-0042's own verification clause and had stood since that
act landed.

## Consequences

- `crates/planner` feeds `canonical_ranges(...).destroyed` into
  `Effects.destroyed`, so `destroyed_closure` now removes every node
  framed on the volume when simulating `Wipe(volume)`. The planner suite
  is green under the act unchanged, but **green is not coverage**: no
  planner test exercises this population. Adding it is WP-060's own pull
  request under its own grant, and is this act's named obligation.
- An `EncryptionLayer` and a `MultipathNode` gain the same entry. Nothing
  frames content on either today, so both are inert — recorded rather
  than claimed as closed.
- Issue #319's authorization half is untouched, and this act does not
  narrow it. Its third measured shape — an unlocated ZFS signature hiding
  the pool from every target — is **pinned as an open limit** in
  `the_identity_seed_never_weakens_a_gate_on_an_absent_extent`, so that
  closing #319 is a deliberate change. For every node this act's own arms
  read, removing an extent fact never opens a gate.

## Verification

`cargo xtask ci` exit 0 at the act's head. Any claim that this ADR closes
issue #319, decides what frames or hosts a backing extent (#365, #409),
or discharges the planner simulation coverage named above, is an error
against this ADR.

## Revisit conditions

- `canonical_ranges` gains the topology or the request (ADR-0042's own
  revisit condition): it could then distinguish "this kind carries no
  extent" from "this extent is absent", and the whole-frame entry could
  be scoped to the four kinds that can never carry one. Measured not to
  matter today — a device with its extent removed gates 6/10, matching
  its honest baseline, and a partition with its extent removed is
  unchanged at 0/10 — but the distinction is real and would become
  load-bearing if the entry ever gained a consumer that reads it as a
  claim about the target's size.
- A kind is added that frames content on an `EncryptionLayer` or a
  `MultipathNode`: the inert arms above become live and want their own
  regressions.
- Issue #319's authorization half lands: the pinned open limit in
  `the_identity_seed_never_weakens_a_gate_on_an_absent_extent` moves, and
  should move deliberately.
