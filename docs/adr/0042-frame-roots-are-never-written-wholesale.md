# ADR-0042: A frame root is never written wholesale, and a target frame root reaches what it carries

- Status: Accepted
- Date: 2026-08-15. Made on the measured round of 2026-08-15
  (`docs/reviews/ISSUE-353_CANONICAL_RANGES_ROUND_2026-08-15.md`,
  single-author with a five-mutation battery, each proven applied;
  committed under WP-000 beside this act). Everything load-bearing is
  restated here. Merging is not acceptance; the decision owner has not
  been put the question in person, and this ADR is where it is put.
- Spec version: **unchanged.** This is a defect fix against a sentence
  that is already normative — §2.1:110, "Table writes target the table
  node's own extents, never the parent device wholesale" — on ADR-0038's
  pricing. The counter-argument is recorded below and declined.
- Work packages blocked: none. Issue #353 closes here as filed; what it
  leaves standing is named in *What stays open*.
- Requirement IDs: MODEL-002, SAFE-005, CAP-003, ADR-0018, ADR-0038,
  ADR-0039
- Decision owners: Nate McBride

## Context

`canonical_ranges(operation, target, facts)` is ADR-0018's canonical-step
entry: the minimal invariant ranges an operation has over a target,
derivable from the body's facts with no plan in scope, and the ranges
`protection_gate` runs the closure over. For the six non-destroying
mutating operations — `Create`, `Grow`, `Repair`, `Label`, `Uuid`,
`Decrypt` — the delivered entry put **the target's whole extent** in
`written_table_extents`. For a device target that is the parent device
wholesale, in as many words what §2.1 forbids.

Issue #353 measured, at `b9d1ba2`, why that could not simply be
corrected: on a whole-disk ZFS vdev the ten mutating gates all refuse,
and **six refuse only because of the over-claim** — correct the entry
alone and `Create`, `Grow`, `Repair`, `Label`, `Uuid` and `Decrypt`
open `Clear` over a live pool with the entire suite green. The reason
is structural. ADR-0039's `descends_into` refuses descent out of a
self-framed extent — the clause that stops a disk's own extent capturing
every sibling when a partial range intersects it — so on a whole-disk
layout the label is reached by the byte scan alone, and only because
the over-claimed write covers the entire device. ADR-0039's consequences
recorded that the correction "must land **after** this act and never
before"; ADR-0040 added a whole-disk pin and a revisit condition asking
this act to re-run it.

Two things the issue named as decisions and this ADR does **not** take:
a truthful per-operation entry (a create writes the host's table extents
and consumes a free range; a grow writes one entry and consumes an
extension; a label writes inside its own structure) needs the request or
the topology, and `canonical_ranges` has neither — its callers are in
WP-050 and WP-060, so widening its signature is a separate,
cross-package act.

## The decision

> 1. **A frame root declares no written range.** In `canonical_ranges`,
>    for the six write operations, the entry is the target's own extent
>    **unless that extent is expressed in the target's own address space**
>    (`extent.host == target` — a device), in which case
>    `written_table_extents` is empty. Below a frame root the entry is
>    unchanged.
> 2. **A target frame root reaches what it carries.** In `descends_into`,
>    the refusal to descend out of a self-framed extent is kept for every
>    node **except the step's target**. The operand is in the affected
>    set by identity, not because a range intersected its self-extent, so
>    the sibling-capture hazard ADR-0039's clause exists for does not
>    arise; and ADR-0039's own rule — a step reaches the content its
>    target carries — then holds for a disk: its table and whatever is
>    hosted directly on it, with descent from those children onward
>    bounded by the same geometry as every other hop.

The two halves are one act. The first alone opens six gates over a live
pool (the issue's table); the second alone changes nothing observable,
because the over-claim already reached everything the hop reaches and
more.

**On the partition-target entry, kept as it is.** For a target below a
frame root the entry stays the target's own extent. That is an
over-approximation the issue names — a partition is not a table node,
and a label writes inside the partition — but it is bounded by the
target, reach-equivalent to ADR-0039's carried content, and it is what
the plan layer's touched-device derivation reads: a `Label` on a
partition carries its disk's PART-013 parse-backup obligation and
refuses on Indeterminate media *through that host*. Dropping it was
measured to survive the entire domain suite while silently removing both
in the planner; WP-060 now pins both (PR #382), and it is that test that
kills the mutation. Removing the over-approximation truthfully needs the
per-kind entry this ADR does not take.

## Measured

Per-(layout, target, operation) gate tables at `1f450c6` and under the
candidate; `C` = Clear, `R` = Unsupported, `B` = Blocked; the ten
columns are Create Grow Shrink Move Repair Label Uuid Encrypt Decrypt
Wipe.

| layout | target | before | after |
| --- | --- | --- | --- |
| whole-disk ZFS vdev (no table, label on the device) | sda | `R R R R R R R R R R` | `R R R R R R R R R R` — **held, through the hop** |
| root-on-ZFS (GPT; ESP; member on sda2) | sda | `R R R R R R R R R R` | `C C R R C C C R C R` |
| root-on-ZFS | table, esp | all `C` | unchanged |
| root-on-ZFS | member, signature | all `R` | unchanged |
| LUKS chain (disk with no self-extent) | sdb, part, mapper | all `R` | unchanged |
| ordinary disk (ESP without extent; stale device-hosted superblock) | sdz | all `B` | unchanged |
| ordinary disk | table, esp, data | all `C` | unchanged |
| BIOS-boot GPT | sda | all `R` | `C C R R C C C R C R` |
| BIOS-boot GPT | table, boot, esp | all `C` | unchanged |
| BIOS-boot GPT | member | all `R` | unchanged |

The one row that moves is a device target on a disk whose protected
content lives on a **partition**: creating a partition in free space,
re-labelling the disk, repairing its table, do not touch sda2, and the
refusal they lost was the over-claim's. The four release operations
still destroy the whole extent and still refuse. `Label sda`'s affected
set is `{sda, table}` — not the ESP, not the member, not the pool. On
the BIOS-boot layout the hop reaches bios_grub through the table,
because it lies inside the table's declared bytes, and nothing beyond.

**ADR-0040's pin** (`a_release_over_a_whole_disk_reaches_the_aggregate_it_carries`)
holds unmoved: `Move` and `Shrink` on the whole disk still seed the
destroyed class and still refuse. Its revisit condition is discharged in
the direction that leaves its verdict standing.

**Mutation battery**, each applied by `sed`, proven applied by a
non-empty `git diff --stat`, the domain suite run:

| # | mutation | outcome |
| --- | --- | --- |
| M1 | target exemption removed (ADR-0039's clause restored verbatim) | killed: whole-disk pin, frame-root reach, BIOS-boot bound |
| M2 | exemption widened to every source (self-framed always descends) | killed: `an_ordinary_disk_keeps_its_siblings_out_of_the_set` (pre-existing) |
| M3 | wholesale write restored | killed: the same three as M1 |
| M4 | written entry dropped for every target | **survives the domain suite**; killed by WP-060's `a_partition_write_still_touches_its_disk_for_the_protection_arms` |
| M5 | M1 and M3 together (the pre-act behaviour) | killed: the same three |

Workspace with WP-060's adjustment: 661 tests, 0 failed; `cargo xtask
ci` exit 0.

## Options considered, and rejected

- **Correct the entry alone.** The issue's table: six gates open over a
  live pool with a green suite. Rejected on that measurement.
- **Declare nothing for all six, everywhere** (M4). Truthful in one
  sense — what they write cannot be named here — but it removes the
  planner's touched-device inference for partition targets with a green
  domain suite. Rejected; the consumer's dependence is now pinned.
- **Widen descent out of self-framed extents to any node in the set**
  (M2). Re-derives round two's sibling capture; killed by a committed
  guard.
- **A truthful per-kind entry.** Needs the request or the topology;
  `canonical_ranges` has neither and its signature is shared with two
  other packages. Left as the next act on this entry, not this one.
- **Reach the whole-disk label by a rule independent of byte overlap.**
  That is what the target hop is; a rule independent of *containment*
  too was not needed and would be a new class of reach.

## The spec-price argument

**Unchanged.** §2.1:110 is already normative and this act brings the
delivered entry into line with it; ADR-0039's stated rule already
promised the target its carried content. Nothing is added to previously
unspecified territory and no requirement's text moves — ADR-0038's
pricing for a defect fix. **The counter-argument, recorded and
declined:** the gate table changes for six device-target pairs on a
partitioned disk, and a reader could call that a behaviour change worth
a patch row. Declined: §0.1 prices requirements, not the correction of
code that violated one; the CHANGELOG carries the behaviour change.

## Consequences

- **Positive:** the canonical entry no longer writes a device wholesale;
  the whole-disk gates rest on carried content instead of an over-claim;
  the six false refusals on a partitioned disk are gone; the regression
  the issue said nothing committed observed is committed.
- **Negative, accepted knowingly:**
  - The partition-target entry is still an over-approximation labelled
    `written_table_extents`. Named, pinned on the consumer side, and
    left for the per-kind act.
  - `descends_into` now takes the step's target: one more parameter to
    a private function, and one more clause a reader must hold.
- ADR-0039's clause reads: "descend out of a self-framed extent never" →
  "never, unless it is the target". Its sibling-capture guard is
  re-asserted on the very disk whose target case now descends.

## Verification

- `whole_disk_gates_hold_without_the_wholesale_write`,
  `a_frame_root_target_reaches_what_it_carries_and_no_more`,
  `a_frame_root_that_is_not_the_target_still_never_descends`,
  `the_target_hop_is_bounded_by_the_same_geometry_as_every_other`
  (`protection_tests.rs`); ADR-0040's pin unmoved; WP-060's
  `a_partition_write_still_touches_its_disk_for_the_protection_arms`.
- Any text implying that a device target's canonical entry writes its
  own extent, that a self-framed extent is never a descent source, or
  that this ADR delivered a per-kind truthful entry, is an error against
  this ADR.

## What stays open

- **The per-kind entry**: what each of the six truthfully writes, which
  needs the request or the topology at `canonical_ranges`. A future
  cross-package act.
- **Issue #347** — the table's own reach into the partitions it releases
  is unchanged here; `Wipe table` on a disk carrying a live pool member
  is still `Clear`.

## Revisit conditions

- `canonical_ranges` gains the topology or the request: the frame-root
  branch should then become the host's table extents for `Create` and
  `Repair`, and the partition over-approximation should be retired.
- A kind other than `PhysicalDevice` is measured carrying a self-framed
  extent as its lawful form: the frame-root branch keys on the frame,
  not the kind, and would apply to it — re-read whether that is wanted.
