# ADR-0038: Release operations seed the closure

- Status: Accepted
- Date: 2026-08-14. Made on the adversarially reviewed round of
  2026-08-13 (`docs/reviews/ISSUE-338_CLOSURE_SEED_ROUND_2026-08-13.md`,
  an untracked session artifact; everything load-bearing is restated
  here), under the decision owner's directive to work issue #338.
  Recorded and implemented in one arc — merging is not acceptance, and
  every element below is reviewable against the round's recorded
  alternatives.
- Spec version: unchanged. **This is a defect fix, not a spec change**
  — the argument is made in full below, with the counter-argument
  recorded and declined.
- Work packages blocked: none. Issue #338 **stays open** on defect (b)
  and on the six non-release operations; issue #319's authorization
  half remains blocked on it.
- Requirement IDs: MODEL-002, SAFE-005, ADR-0018
- Decision owners: Nate McBride

## Context

`affected_set` (`crates/domain/src/model/protection.rs:230-249`) has two
entry routes and they are not equivalent. A node intersecting
`ranges.destroyed` enters `range_destroyed`; a node intersecting
`written_table_extents` or `consumed` enters `affected` **and nothing
else**. The fixpoint (`protection.rs:251-297`) propagates only from
`range_destroyed` and `cascade_destroyed`.

So a node reached through `written_table_extents` or `consumed` is
**judged but reaches nothing further**. Where a protected node is
directly byte-intersected by the target's extent it is still caught;
where reaching it needs a propagation hop, it is not.

That framing took three statements to reach. The issue as filed said
"the closure does not run at all"; a first correction said
`protection_gate` "reports `Clear`". Both are wrong on `root_on_zfs`,
where all eight operations measure `Unsupported` because the member's
extent byte-intersects the nested signature directly. **The fixture
with teeth is `the_luks_descent_reaches_the_pool_below`**
(`protection_tests.rs:240-345`), whose chain reaches the pool only by
propagation. Both superseded framings are corrected on the issue rather
than left in its history.

`canonical_ranges` (`crates/domain/src/model/capability.rs:150-174`)
emits `destroyed: vec![]` for eight operations, putting the target's
extent in `written_table_extents`. Its two consumers — `protection_gate`
(`capability.rs:189`) and the non-sized `plan` path
(`crates/planner/src/lib.rs:747`) — therefore get a closure with no
propagating seed.

## The decision

Two corrections in `crates/domain`, both bringing the code to
ADR-0018's own text rather than changing that text.

1. **`Shrink` and `Move` take the conservative entry**: the whole target
   extent in `destroyed`, nothing in `written_table_extents`.
   ADR-0018:136-141 defines the destroyed class as releases — "a range
   freed from its owner — a deleted partition's extent, **a shrink's
   truncated tail, a move's source extent at commit**". Of the eight,
   only these two are named releases.
2. **The membership half of ADR-0018's rule 3 is ungated.** The ADR
   states rule 3 route-agnostically — "a `BackingSignature` **in the
   set** brings its consumer" (0018:153-154) — and contrasts it in the
   same paragraph with rule 4's "in the set **through a destroyed
   range**" (0018:155-156). The delivered code gated both on
   destruction. The substrate half stays gated; only membership is
   freed.

**The other six operations are untouched, and defect (b) is untouched.**
Both are held, and this ADR does not close #338.

## Why conservative and not truthful

Measured, and it inverts the obvious fix. Four entries on the committed
pool member:

| entry | outcome |
| --- | --- |
| today's (`written_table_extents`, empty `destroyed`) | `Refused` |
| the honest empty entry | `Ok` — pool unreached |
| **the solver's real freed tail** | **`Ok` — pool unreached** |
| the whole-target-extent `destroyed` entry | `Refused{Zfs}` |

Making the entry *truthful* is a safety regression; making it
*conservative* is not. A shrink does not destroy its whole target
extent — but `canonical_ranges` takes `(operation, target, facts)` and
**no request parameters**, so it cannot know the new length. It is a
**capability gate**, and the honest posture for a gate that cannot
compute the exact range is to over-reach rather than under-reach.

`plan_sized` is unaffected: it emits the solver's real freed range
(`crates/planner/src/lib.rs:986`), which is correct for a *plan* because
the plan knows its geometry.

## Conservatism is argued per operation, by measurement

**Monotonicity is false**, so "a larger destroyed set is always safer"
may not be assumed. The three propagating arms each carry a negative
guard `!range_destroyed.contains(&edge.target)` (`protection.rs:261`,
`:274`, `:284`), so a larger destroyed set can move a node *out* of
`cascade_destroyed` and stop descent through it. The round measured a
concrete inversion under one rejected design.

The act is therefore justified by measurement on each of the two
operations it touches, on the fixture that exercises propagation:

| operation | before | after |
| --- | --- | --- |
| Wipe | `Unsupported` | `Unsupported` |
| **Shrink** | **`Clear`** | **`Unsupported`** |
| **Move** | **`Clear`** | **`Unsupported`** |
| Grow, Create, Repair, Label, Uuid, Decrypt | `Clear` | `Clear` |
| Encrypt | `Unsupported` | `Unsupported` |

Exactly the two operations the act touches move, in the refusing
direction, over a live ZFS pool reached through a LUKS chain. The six
held operations are visibly still `Clear` — the held half of (a) shown
rather than asserted.

## Options considered

All three designs the round put were **rejected on measurement**.

### Re-let the target, upper-bound the gate

Rejected: measured to re-derive ADR-0018's own no-sibling-capture
outcome (0018:190-192) on a body `canonical_ranges` itself authors —
`n=6` with the ESP captured against a baseline `n=2`. The committed
guard passed **vacuously**, its own destroyed range missing the table's
extent. Also measured: a non-monotonicity turning a refusal into a
construction, and `gate(Wipe, device) = Clear` while
`gate(Wipe, partition)` refuses — reach inversely ordered with
destructiveness.

### Content descent from a non-frame target

Rejected: measured on a whole-disk ZFS vdev to move `Label`, `Uuid`,
`Repair`, `Create` and `Grow` from `Unsupported` to **`Clear`** over a
live `Refused{Zfs}` aggregate. A design for #338 that opens a new reach
hole is disqualified on that alone.

### Carried-content reach

Rejected: its load-bearing structural claim — that reading the
endpoint-pair table discharges ADR-0018's theorem obligation by
construction — was falsified by two mutations that left the workspace
green. Its predicate inspects only Containment pairs while Backing and
Production also propagate.

### Emitting the solver's real freed tail

Rejected on the measurement above: truthful and less safe.

## Consequences

- **Positive:** a shrink or move over a protected node reached by
  propagation now refuses at the capability gate and at the non-sized
  plan path. The two operations ADR-0018 names as releases are seeded
  as releases.
- **Negative, accepted knowingly:**
  - **Over-reach on `Shrink` and `Move`.** The whole target extent is
    declared destroyed when only part of it is. A shrink that touches
    no protected byte can now refuse where it previously constructed.
    This is the priced direction, and it is the gate's posture, not the
    plan's — `plan_sized` still computes the real freed range.
  - **Six operations keep the defect.** `Grow`, `Create`, `Repair`,
    `Label`, `Uuid` and `Decrypt` destroy nothing, so ADR-0018 licenses
    no destroyed entry for them, and their reach still needs the
    propagation widening this ADR **holds**.
  - **Defect (b) is untouched.** Partial destruction still misses a
    child whose declared bytes lie outside the destroyed sub-range —
    measured on the unmodified fixture at `affected=3`, pool unreached,
    shrink constructing. `plan_sized` seeds correctly and still fails
    this way.
  - **Monotonicity remains false**, so any future widening must repeat
    the per-operation measurement rather than inherit this one.
- **Measured to have no blast radius beyond `crates/domain`:** the
  round required `crates/capability`'s CAP-005 enumeration measured
  before this PR, since it asserts ground equality between the engine
  and the constructor. Both corrections applied together leave the full
  workspace green — 21 test binaries, zero failures — so no WP-050 PR
  is owed. The measurement was taken *because* it could have moved, not
  to confirm that it would not.

## The spec-price argument

**No spec version change.** `AGENT_BUILD_SPEC.md` is untouched and
ADR-0018's text is untouched; the code is being brought to both.
ADR-0018:136-141 already names these two operations' ranges as
releases, and 0018:153-154 already states rule 3 on set membership.

**The counter-argument, recorded and declined.** §2.1 delegates the
closure to ADR-0018 by name (`AGENT_BUILD_SPEC.md:110`), so a reader may
price any ADR-0018-shaped act as requirement-shaped. Declined: the ADR
text is unchanged and nothing weakens. If the register disagrees, the
correct price is **minor**, never major — a defect fix that makes a gate
refuse more cannot narrow a requirement. Not a §1.11 filing either: that
register needs two conflicting requirements, and this is code against
text.

## Verification

- The LUKS fixture's gate table above, asserted per operation — the two
  that move and the six that do not.
- Both committed guards re-measured on **membership**, not merely re-run
  green: `a_sibling_esp_is_never_captured` asserts set membership only,
  and the flagship's create half is a size-2 smallness guard.
- A false-refusal control on each corrected operation: a shrink over a
  device with no protected chain still constructs.
- Mutations applied with `Edit` and **proven applied** before each run.
- `cargo xtask ci`'s real exit code checked directly, not through a
  pipe.

## Revisit conditions

- **The propagation widening lands** (issue #338's held half). Its
  measurement supersedes this ADR's conservatism argument, and the
  `Shrink`/`Move` over-reach should be re-examined then — a correct
  closure may make the whole-extent entry unnecessary.
- **`canonical_ranges` gains request parameters**, at which point the
  truthful range becomes computable and the conservative entry becomes
  a choice rather than a necessity.
- **Defect (b) is fixed**, which changes what a seeded destroyed range
  reaches and therefore what conservatism buys.
