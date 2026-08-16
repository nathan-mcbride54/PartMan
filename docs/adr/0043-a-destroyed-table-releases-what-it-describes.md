# ADR-0043: A destroyed partition table releases the partitions it describes

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-347_ROUND_3_2026-08-16.md`, single-author with a
  six-mutation battery, each mutation proven applied and each killed;
  committed under WP-000 beside this act), after two adversarially
  reviewed rounds rejected two earlier designs
  (`ISSUE-347_TABLE_RELEASE_ROUND_2026-08-14.md`,
  `ISSUE-347_RELEASE_ROUND_2_ADVERSARIAL_2026-08-14.md`). Everything
  load-bearing is restated here. Merging is not acceptance; the decision
  owner has not been put the question in person, and this ADR is where
  it is put — every element is reviewable against those two rounds'
  recorded fatals.
- Spec version: **14.0.0 — major under §0.1.** The argument is made
  below; it carries a correction to ADR-0042's pricing.
- Work packages blocked: none. Issue #347 closes here as filed. Issue
  #360's row stays held on the remainder this ADR names.
- Requirement IDs: MODEL-002, SAFE-005, CAP-003, ADR-0018, ADR-0039,
  ADR-0041, ADR-0042
- Decision owners: Nate McBride

## Context

ADR-0018 defines the destroyed class as releases: "a range freed from
its owner — a deleted partition's extent, a shrink's truncated tail, a
move's source extent at commit — is destroyed even though no byte is
overwritten, because its content ceases to be referenced." Destroying a
partition table releases every partition it describes. The delivered
closure reached none of them: `canonical_ranges(Wipe, table)` destroys
the table's own extent, the byte scan finds the table and its device,
and the partitions — whose extents lie beyond the table's `[0, 1 MiB)` —
were never seeded. Measured at `5b795df` and re-measured at every commit
since: on `root_on_zfs`, `Wipe(table)` gave `affected = {table, sda}`,
the pool's `Refused{Zfs}` never consulted, the ten-operation gate on the
table 10/10 `Clear`. Issue #347 was also measured to gate #360, which
gates #354's kind half, which gates #333's enforcement.

**Two designs died inside the closure on the table's own geometry, and
the panel measured why no third can live there.**

- Round 1 (2026-08-14) released when the destroyed ranges *covered* the
  table's declared extent. Fatal, measured: coverage is anti-monotone in
  `Facts.extents` — inflate the extent by one byte and the refusal
  disappears (§10.2), a fail-open on authored content.
- Round 2 released when the table entered `range_destroyed` by
  *intersection*, on the two `partition-table`-sourced pairs. Monotone,
  no fail-open path across 4200 measured rows — and rejected on three
  surviving grounds: **sibling capture from a partition-target step**
  (L1: on a BIOS-boot disk whose `bios_grub` at LBA 34 nests inside the
  table's conventional `[0, 1 MiB)`, wiping that partition intersected
  the table and released every sibling, ESP, member and pool); the
  `conflicting-table-entry` half of the pair set unjustified and
  uncovered (L3); and the theorem amendment not covering L1's route,
  whose reacher is a partition and not the table. The panel's
  impossibility result: round 1 §11's two requirements are jointly
  unsatisfiable over `Facts.extents` — every coverage predicate is
  anti-monotone in the authored extent, and the intersection test has no
  strength. Its direction for round 3: derive the release from the
  **naming relation** and gate it on **something structural about the
  step**, never on whether a declared range happens to touch an authored
  extent; and commit the overlapping-geometry fixture first.

Since then: ADR-0041 committed `bios_boot_gpt` with the panel's `f11`
and `f12`, refused zero-length and overflowing extents (round 2's M7),
and established that a table's extent is its own header bytes, not the
region it governs; ADR-0042 made the step's target a first-class notion
inside `affected_set` (`descends_into` now knows it). Both are what this
design stands on.

## The decision

> **A step whose target is a partition table, and whose own destroyed
> ranges reach that table, destroys the table; and a destroyed table
> releases every partition whose name says the table describes it.**
>
> - *Destroys the table* means: `target` is in `range_destroyed` — the
>   step's declared destroyed ranges intersect the target's own extent.
>   The trigger is the target's *identity* plus the step's *own*
>   destruction of it. A table some other step's range happens to touch
>   never releases; a table reached by descent never releases; a table
>   that is the target of a step destroying nothing (Repair, Label,
>   Uuid, Create, Grow, Decrypt) never releases.
> - *Releases every partition whose name says the table describes it*
>   means: every node whose `NamingFields::released_by_table()` is
>   `Some(target)` enters the cascade class. That is exactly
>   `Partition { parent_table }`. It is read off the naming roster
>   `Topology::build` already sweeps — a partition cannot be represented
>   without naming its table — never off a containment edge, which a
>   body may omit. `ConflictingTableEntry { table }` names a table and is
>   **not** released by it: ADR-0019 holds it verbatim as a record inside
>   the table's own bytes, and ADR-0036 decided it is not an occupant of
>   the region it names; destroying its table destroys the record, which
>   the ordinary geometry already reaches, and releases nothing beyond
>   it. Every other kind names no table.
> - A released partition is in the cascade class: it descends into what
>   it carries by the ordinary geometry (`descends_into`, unchanged), and
>   a signature it carries brings its consumer (rule 3, unchanged).

**What is not consulted.** The table's declared extent decides nothing
except the target's own membership in `range_destroyed` — and for a
step that destroys its own target, that membership holds at every size
of the extent (a step's canonical destroyed entry *is* that extent;
inflate it, deflate it to 17 408 bytes, or under-declare the wipe to
the protective MBR's 512 bytes and the release still fires). No coverage
test. No test of whether a range from a partition-target step touches
the table. No edge.

**The one priced limit, asserted rather than hidden.** A step whose
target is the *table* and which destroys any byte the body attributes to
the table is read as destroying the table. The closure has no
non-authored way to tell one GPT entry from the header — that is round
2's impossibility result — and it reads the case fail-closed. Nothing
delivered emits that spelling: a partition delete is `Wipe` with the
partition as target (the row the panel measured as L1, 10/10 `Clear`
here), and a table wipe is `Wipe` with the table as target. The
`bios_boot_gpt` fixture pins both spellings side by side, so the
boundary is a committed row and not an inference.

**The one named remainder.** The #360 chain — an mdraid array producing
a volume that carries a table, the member disk wiped — needs the release
to *propagate*: the target disk is wholly destroyed, its signature, the
array, the volume and the table follow through the cascade, and the
table then releases. That propagation was built, **measured to close the
chain** (`Wipe(member disk)`: HEAD `constructs`, pool unreached; with
propagation `Err`, pool reached — under the `volume → partition-table`
row added for the measurement only), and **cut from this act**: with the
row absent the chain is unbuildable, so the clause was uncovered, and an
uncovered clause is what round 2 treated as a proposal blocker. #360's
own act adds the row, the fixture, and the propagation together. `Wipe`
of a *volume* target stays open there too: a volume carries no extent
(ADR-0041), so its own canonical `destroyed` is empty and the closure
cannot see it destroyed.

## Measured

Per-(layout, target, operation) gate tables at `a8f6117` → candidate;
`C`/`R`/`B` = Clear/Unsupported/Blocked; columns Create Grow Shrink Move
Repair Label Uuid Encrypt Decrypt Wipe.

| layout / target | before → after |
| --- | --- |
| root-on-ZFS / **table** | `CCCCCCCCCC` → `CCRRCCCRCR` — **#347 closed**; `Wipe(table)` affected `{table, sda}` → `{table, sda, esp, member, signature, pool}` |
| root-on-ZFS / esp, member, sda | unchanged |
| root-on-ZFS, table extent `[0, 1 MiB+1)` / esp (**round 2's L2**) | `C…` → `C…` — unchanged; the byte does nothing |
| root-on-ZFS, table extent `[0, 768 MiB)` / esp | `R…` → `R…` — HEAD's own descent, unchanged |
| root-on-ZFS, table extent `[0, 17408)` / table | `C…` → `CCRRCCCRCR` — the release fires at every size |
| root-on-ZFS, target table, destroyed `[0, 512)` | constructs → refuses (the priced limit's other spelling) |
| root-on-ZFS, `table → partition` edges omitted / table | `C…` → `CCRRCCCRCR` — round 2's M5 closed |
| BIOS-boot / **bios_grub** partition target (**round 2's L1**) | `CCCCCCCCCC` → `CCCCCCCCCC`; `Wipe(bios_grub)` affected `{boot, table, sda}`, ESP/member/pool out |
| BIOS-boot / target table, destroyed `[17408, 1 MiB)` (`f11`/`f12`'s spelling) | constructs → releases (the priced limit) |
| BIOS-boot / table | `C…` → `CCRRCCCRCR` |
| ordinary disk with an orphan LVM2 signature on `data` / table (round 2's M2) | `C…` → `CCBBCCCBCB` — the released partition's own gate is `B` ×10; Blocked is the acknowledgeable arm |
| plain ext4 disk / table | `C…` → `C…`; `Wipe(table)` reaches p1 and fs and **constructs** |
| hybrid disk / hybrid MBR view (round 2's M3) | `C…` → `C…` — it describes no partition |
| hybrid disk / gpt | `C…` → `CCRRCCCRCR` (pool under sda2) |
| #360 chain (row added for measurement) / member disk | constructs → constructs **without** propagation (cut); refuses **with** it |

Every non-table-target row on five layouts is byte-identical to HEAD.
The affected set is a superset of HEAD's in every row by construction —
the change only inserts. ADR-0040's whole-disk pin and ADR-0042's four
regressions hold unmoved.

**Mutation battery** (six, each proven applied by `git diff`, the domain
suite run): release disabled — killed by six; `ConflictingTableEntry`
also released — killed by the roster test and the hybrid control;
release from every range-destroyed table rather than the target — killed
by four, the L1/L2 guards among them; release on target identity without
destruction — killed by six, **four of them pre-existing guards**
(`a_sibling_esp_is_never_captured`, `the_root_on_zfs_regression_pair_holds`,
`an_ordinary_disk_keeps_its_siblings_out_of_the_set`,
`ungating_rule_three_membership_never_captures_a_sibling`); release never
fires — killed by five; released partitions not cascaded — killed by
five. Two earlier mutations on the cut propagation survived, which is
why it was cut.

Workspace: 666 tests, 0 failed; `cargo xtask ci` exit 0; `crates/capability`
and `crates/planner` unchanged and green.

## The theorem, amended

ADR-0018's non-interference theorem as amended by ADR-0039 read: *no
node whose declared extent is comparable with its reacher's and lies
outside it is ever in the set.* A released partition lies outside its
table's extent and is in the set. The theorem is amended, in ADR-0018's
own inline style, to:

> no node whose declared extent is comparable with its reacher's and
> lies outside it is ever in the set, **except a partition released by
> the destruction of the table its own name says describes it, where
> membership follows the naming relation and the step's target and never
> geometry** (14.0.0, ADR-0043).

Round 2's objection to its own amendment — that L1's reacher is a
partition, so an exception on the `table → partition` edge cannot cover
it — does not arise: the exception is not on an edge, and a
partition-target step releases nothing. The consequence the theorem was
written for is restated with the same care: **a sibling is never
captured by a step that destroys another partition**; a step that
destroys the table releases every partition it describes, which is
release, not capture. The property test ADR-0018:210-217 demands is
`the_release_roster_is_pinned_per_kind`: quantified over the naming
roster, one kind is released and the table that releases it is the one
its name declares; a kind that comes to name a table lands there.

## Options considered, and rejected

- **Coverage of the table's extent** (round 1). Anti-monotone; rejected
  on measurement there and re-argued nowhere here.
- **Intersection with the table's extent, on the two table pairs**
  (round 2). Sibling capture from a partition-target step; rejected
  there. This ADR's trigger differs in kind: it is the *target's* own
  destruction, and a non-target table never releases (mutation M3).
- **Release on target identity alone.** Killed by four pre-existing
  guards (mutation M4): `Label(table)` would release.
- **Release the conflicting-table-entry too.** Round 2's L3; killed by
  the roster test and the hybrid control (mutation M2).
- **`canonical_ranges` declaring the released extents** (the issue's
  option 2). Needs the topology at a signature two other packages call,
  and moves release out of the closure into every range producer, so a
  plan step under-declaring a table wipe would construct — a new
  fail-open. Rejected.
- **Propagate the release from a wholly-destroyed target through the
  cascade** (this round's first candidate). Measured to close the #360
  chain and uncovered without #360's row; cut, recorded as the
  remainder.
- **Seed on any non-empty destroyed set for an extentless target.**
  Would make `Wipe(volume)` distinguishable from `Label(volume)` only by
  the presence of ranges, which for an extentless target is empty in
  both; not implementable at the closure without the operation.

## The spec-price argument

**Major, 14.0.0.** Two normative sentences change meaning. ADR-0018's
theorem as amended in 13.0.0 is amended again — a class of nodes outside
their reacher is now in the set. And §2.1:113 changes twice: this ADR
adds "a partition table destroyed by its own step's target releases
every partition whose name says it describes it, by the naming
relation"; and the sentence "**a node's own address space is never a
descent source**, so a device's whole-disk extent cannot carry reach into
its siblings" is corrected to "never a descent source **except for the
step's own target**". **That second correction is ADR-0042's, not this
one's.** ADR-0042 admitted descent out of the target's self-framed
extent, which changed that sentence's meaning, and priced itself
"spec version unchanged" — a mispricing under §0.1's rule that semantic
changes to existing text bump major, and under ADR-0039's own precedent
for the same sentence. It is corrected here, on the first act that
touches the sentence again, and recorded rather than smoothed over. The
minor and patch readings are declined on ADR-0039's reasoning: reach is
added and nothing weakens, but existing sentences change meaning.

**Not a §1.11 filing.** No two requirements conflict; ADR-0018's release
definition was an unmet obligation, and this act discharges it.

## Consequences

- **Positive:** destroying a partition table reaches what it releases;
  the whole family's fixture (`bios_boot_gpt`) is committed with both
  spellings pinned; the release is enumerable over the naming roster and
  reads no authored geometry but the target's own; issues #360, #354's
  kind half and #333's enforcement are unblocked *as far as this issue
  gated them* — #360 must still deliver its row with the propagation
  this ADR names.
- **Negative, accepted knowingly:**
  - The priced limit: a table-target step destroying any byte the body
    attributes to the table releases. Fail-closed, on a spelling nothing
    delivered emits.
  - A table wiped on an ordinary disk whose partition carries an orphan
    signature goes `Blocked` (acknowledgeable), where HEAD was `Clear`.
    That is the released partition's own gate.
  - The remainder: the #360 chain and `Wipe(volume)`.
- ADR-0018:210-217's property test now has a naming-roster form beside
  its edge-taxonomy form.

## Verification

- `destroying_a_partition_table_releases_the_partitions_it_describes`,
  `the_release_follows_the_naming_relation_not_the_edges_or_the_extent`,
  `a_table_that_is_not_the_target_never_releases`,
  `a_released_partition_refuses_only_for_what_it_carries`,
  `the_release_roster_is_pinned_per_kind`, and the two `bios_boot_gpt`
  assertions
  `a_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table`
  and `a_range_that_touches_no_gpt_structure_releases_only_from_a_table_target`
  (`protection_tests.rs`).
- Any text implying that a table reached by intersection or by descent
  releases, that the release depends on a containment edge or on the
  table's extent size, or that a `ConflictingTableEntry` is released by
  its table, is an error against this ADR.
- Any claim that this ADR closes issue #360, or that a `Wipe` of a
  volume carrying a table reaches the table's partitions, is an error
  against this ADR.

## What stays open

- **#360**: the `volume → partition-table` row, with the release
  propagation this ADR measured and cut, and `Wipe(volume)`.
- **What a `partition-table` node's extent *is*** (round 2's open point
  2): no adapter computes extents; the fixtures' `[0, 1 MiB)` is the
  ADR-0036 convention. This ADR does not depend on the answer.
- Round 2's M6: the same bytes spelled under `written_table_extents`
  release nothing, because they destroy nothing. `PlanStep` enforces no
  consistency law between the classes; pre-existing.

## Revisit conditions

- #360's row lands: add the propagation and the chain fixture in the
  same act, and re-run this ADR's battery.
- A kind other than `Partition` comes to name a table in its own
  fields: classify it in `released_by_table` — the roster test reds
  until it is.
- An adapter emits a table's structural extent: the priced limit's
  spelling then means what it says, and `f12` should be re-read.
