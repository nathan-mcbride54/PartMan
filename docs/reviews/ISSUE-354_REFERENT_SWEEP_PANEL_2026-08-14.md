# Issue #354: the referent sweep — judge panel and verification, 2026-08-14

Untracked session artifact, `docs/reviews` convention.

Four independent designs, each judged twice (soundness lens, cost lens),
with any design a judge found capable of removing reach disqualified.
**Then I verified the winner by hand, and it fails on a shape no judge
tested.** Measured at `7fdba38` in a clean worktree.

## 1. The mandate

Not a new policy question. ADR-0037:146-150 records the obligation:

> "**Owed before any enforcement:** a capture-side referent sweep.
> `Topology::build` validates edge referents and endpoint pairs but
> **nothing validates naming-field referents**, so a naming-derived frame
> can be computed from a pairing the pair table forbids."

`:217` makes its existence a verification condition for #333's frame
enforcement, with the golden vector regenerated in the same act. So the
order is **#354 → #333**, verified by reading.

## 2. The panel

| design | avg | discharges obligation | can remove reach |
| --- | --- | --- | --- |
| **type-shaped** — sweep in `Topology::build`, kind check derived from `endpoint_pair_allowed` | **7.75** | yes | no |
| kind-and-edge — same, plus an edge-agreement clause (which its author disqualified on MODEL-002) | 7.5 | yes | no |
| minimal — resolve-only | 7.0 | **no** | no |
| capture-side — obligation on the capture path only | 4.5 | **no** | **yes** (one judge) |

The two strong designs converge on the same shape, and its best idea is
genuinely good: **the kind check is not a second authored list.** It asks
`endpoint_pair_allowed` — the delivered pair table the edge validator
already uses — the same question about the *name* that it asks about the
*edge*. Nothing to drift.

Both also delete the planner's duplicate referent roster
(`simulate.rs:85`) and delegate to one shared accessor, which is
load-bearing rather than tidying: `destroyed_closure` closes over that
roster, so "capture swept ⇒ simulated rebuild swept" is a theorem only
while the two lists are one list.

## 3. The winner fails, and the mechanism is the good idea

**CONFIRMED by hand**, patch applied to a clean tree and proven applied:

| honest layout | `7fdba38` | winning design |
| --- | --- | --- |
| GPT inside a LUKS volume (`PartitionTable.parent` names a `Volume`) | builds | **`ForbiddenNamingReferent{PartitionTableParent, "volume"}`** |
| partitioned mdraid array (`parent` names an `Aggregate`) | builds | **`ForbiddenNamingReferent{PartitionTableParent, "aggregate"}`** |
| xfs on a dm-multipath node (`FileSystem.host` names a `MultipathNode`) | builds | **`ForbiddenNamingReferent{FileSystemHost, "multipath-node"}`** |

The root cause is exactly what the *disqualified* capture-side design's
judge found on the same mechanism, and which the winner's own two judges
never tested: **the sweep imports the pair table's incompleteness into a
mandatory field.** That table lists the containment pairs the *edge*
validator needs. It was never a complete catalogue of what a naming field
may legitimately reference, and deriving a mandatory check from it
promotes an omission into a refusal.

Two judges scoring 8 and 7.5 on the winner, and neither ran an honest
real-world layout through it. **A panel is only as good as the shapes its
judges think to try** — the finding that killed the winner was already in
the transcript, attached to a different design.

## 4. The larger finding underneath, which is not about #354

Asking the pair table directly (`endpoint_pair_allowed(Containment, …)`):

```
volume         -> partition-table : NOT IN TABLE
aggregate      -> partition-table : NOT IN TABLE
aggregate      -> file-system     : NOT IN TABLE
multipath-node -> file-system     : NOT IN TABLE
volume         -> file-system     : ADMISSIBLE   (control)
```

**A partitioned mdraid array has no containment expression at all.** The
only route to a volume is `Production` (`aggregate → volume`, admissible),
and `volume → partition-table` is absent, so there is no path from `md0`
to `md0p1`. The same absence blocks a partition table inside any mapped
volume — the LUKS-with-inner-partitions layout.

That is a modelling gap in committed code, independent of this issue, and
it is the reason the kind half of the sweep cannot land yet. Filed
separately.

## 5. Recommendation

**Split the act.**

1. **Land the resolve-only half now.** Every referent must be the address
   of an absorbed entry. It closes the dangling-referent attack, has no
   honest-body cost (a dangling referent is nonsense under any reading of
   any table), and is a genuine partial discharge. **It must not be
   described as closing #354**, because ADR-0037's stated harm is the
   forbidden *pairing*, which resolve-only leaves open — the judges were
   right to mark `minimal` as not discharging.
2. **Hold the kind half** until the containment pair table covers the
   real population. Deriving it from the table is still the right idea —
   it is the only version with no second list to drift — but it inherits
   whatever the table gets wrong, so the table has to be right first.
3. **The pair-table gap is the prerequisite**, and it is a MODEL-002
   question rather than a protection one.

## 6. Process failure, mine

I told all four design agents they could prototype in one worktree,
`D:\pm-354`. They did, concurrently. One found it carrying 199
uncommitted lines with duplicate `naming_referent` definitions from two
peers writing the same accessor into the same file, **discarded its own
first baseline as void**, and moved to an isolated tree — which is the
only reason its numbers are usable.

This is the shared-checkout trap applied to subagents rather than
sessions. **Give each prototyping agent its own tree**, or give none of
them write access. The three designs that stayed in the shared tree
reported measurements I cannot trust and did not re-run.

## 7. Not established here

- The resolve-only half is designed and measured by others but **not yet
  implemented by me**; the patch on disk is the full winner, kind check
  included.
- No `cargo xtask ci`, no mutation pass, no adversarial round on the
  split recommendation itself.
- Whether the pair table's omissions are oversights or deliberate is
  **unestablished** for the multipath rows in particular, since multipath
  is detection-only in v1 and its absence may be intentional. The
  `aggregate`/`volume → partition-table` rows have no such explanation.
