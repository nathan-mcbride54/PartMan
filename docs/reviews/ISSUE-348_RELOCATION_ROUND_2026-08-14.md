# Issue #348 round — the relocation exemption against the `Move` entry

**Date:** 2026-08-14. **Base:** `b3de0cf` (main), spec 13.0.0.
**Worktree:** outside the repo, own `CARGO_TARGET_DIR`.
**Directive:** Nate — "resolve issue 348 and update the documentation."

> Untracked local artifact, `docs/reviews` convention: never stage into a
> commit; `verify-change-ownership` refuses it.

## 0. Why the issue's framing did not survive contact

#348 was filed at `5b795df` and framed the conflict as *ADR-0018's
exemption versus ADR-0038's entry*, with two arms: either `Move` gets a
distinct canonical entry, or the exemption is amended. **ADR-0039 landed
between the filing and this round**, and it changes which arm is even
reachable.

Four facts, read off the delivered code at main before any design work:

| fact | evidence |
| --- | --- |
| the closure takes no `Operation` | `affected_set(topology, facts, target, ranges)` — `protection.rs:219-224`; neither call site passes one (`protection.rs:449`, `step.rs:417`) |
| the target is seeded and descends unconditionally | `affected.insert(target)` `:239`; `\|\| affected.contains(&edge.source)` `:269-271` |
| `Move` is unrepresentable in the planner | `planner/src/lib.rs:376` — "moves and copies need a destination vocabulary this model does not carry yet" |
| the byte-wise-preservation duty is delivered nowhere | zero hits for `byte-wise` / `enumerate their loss` across `crates/` |

So the exemption is a promise made on behalf of an implementation that
does not exist, resting on a contract (PART-005) that is not delivered,
expressed in terms the closure cannot represent.

## 1. The measurement that decided it

**M1 — `Move` moved from the destroyed arm to the written-extents arm.**
Gate identical on all six targets; full domain suite green. Not one
committed regression can observe M1.

**M3 — `Move` surrenders its canonical entry entirely.**

| target | before | after |
| --- | --- | --- |
| `part(luks-host)` | `Unsupported{InheritedFromConsumerOrProducer}` | unchanged |
| **`sda(device)`** | **`Unsupported{Zfs}`** | **`Clear`** |
| `member(part)` | `Unsupported{Zfs}` | unchanged |

**Full domain suite green under M3.** A surviving mutant that opens a
whole-disk gate over a live ZFS pool.

**The split is structural.** On a partition target carried-content reach
alone refuses. On a disk target it cannot: `descends_into` refuses a
self-framed extent as a descent source — the very clause that stops a
disk's extent capturing its siblings. Reach there is entirely
range-driven, so **ADR-0038's release entry is load-bearing after
ADR-0039, not superseded by it.**

### A correction this round made to its own work

The first pass measured M3 on the **partition target only** and
concluded "Move still refuses, so the exemption is unreachable through
`canonical_ranges`". That conclusion was right for partitions and
**false for whole disks**, where the same mutation opens a live-pool
gate. The disk case was found by asking what the byte-scan catches that
edge-descent does not — a node whose extent lies inside the target's but
which has no edge path from it. Both halves are now in the pin.

## 2. What was rejected, and on what ground

| # | option | ground |
| --- | --- | --- |
| R1 | the issue's own option 1 — a distinct canonical entry for `Move` | **measured**: M1 changes nothing; M3 opens `Clear` over a live pool |
| R2 | relocation as a fourth range class with a computed preservation predicate | **structural, off the types**: the predicate keys on `facts.extents`, and no §2.1 aggregate can carry an extent under `may_carry_extent` — inert on exactly the class it must relieve, live on the guard standing in for it |
| R3 | pass `Operation` into `affected_set`, exempt on relocation class | subtracts reach; ADR-0039's invariant; keyed on a class reachable from an authored body at `mutating_declared`, which has no capability gate |
| R4 | record the refusal as settled product policy | **declined by the decision owner** |
| R5 | delete the byte-wise duty along with the exemption | judgement, and the weakest call — it is the only recorded statement that a relocation can lose a hosted signature |
| R6 | hold #348 open | separable: the record correction and the coverage land now, the product question stays open |

## 3. The pin, and proof it bites

`a_release_over_a_whole_disk_reaches_the_aggregate_it_carries` asserts
three things per release operation: the range **class**, that the
canonical entry **reaches the pool**, and the **gate outcome**.

| mutation | result |
| --- | --- |
| M3 | **only red** — 116 passed, 1 failed, real exit 101 (gate assertion) |
| M1 | **only red** — fails on the range-class assertion |

M1 is the mutation the workflow's own candidate pin declared in advance
it would survive. Asserting the range class is what closes it.

The `affected_set` assertion is currently **unexercised** — the other
two trip first under every mutation tried. Recorded as such rather than
counted as coverage.

## 4. Method note

Eleven agents: four grounding, three candidate designs (one died on a
connection error), one measurement pass, two adversarial refuters, one
synthesizer. **Both surviving candidates were judged fatal by their own
refuters**, which is why the accepted decision is neither of them — it
is the null act on production code plus the coverage the measurement
proved was missing.

Every load-bearing figure was **re-measured by hand** rather than taken
from an agent. The disk-target correction in §1 is the reason that
matters.

## 5. Residuals

1. The availability gap — filed.
2. The undelivered byte-wise duty — filed.
3. **Issue #353's interaction**: correcting the written-arm over-claim
   moves six whole-disk ZFS gates to `Clear` over a live pool with the
   suite green, including this pin. Recorded on #353.
4. #347's interaction is unmeasured here; this worktree is at main.
