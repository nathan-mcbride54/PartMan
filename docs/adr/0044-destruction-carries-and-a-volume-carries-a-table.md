# ADR-0044: Destruction carries through the cascade, and a volume carries a partition table

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-360_ROUND_2026-08-16.md`, single-author with a
  seven-mutation battery, each mutation proven applied and each killed;
  committed under WP-000 beside this act), on the two measurements the
  issue's own record already carried: the filing's direct query of the
  pair table at `7fdba38`, and the probe at `b3de0cf` that found the
  row's population under-protected — which ADR-0043 then measured, cut
  and named as this issue's remainder. Merging is not acceptance; the
  decision owner has not been put the question in person, and this ADR
  is where it is put.
- Spec version: **15.0.0 — major under §0.1.** The argument is made
  below.
- Work packages blocked: none. Issue #360 closes here as filed. Issue
  #354's kind half is unblocked as far as this issue gated it, and stays
  held on the multipath population named below. The extentless-target
  limit is filed as issue #392.
- Requirement IDs: MODEL-002, SAFE-005, CAP-003, ADR-0011, ADR-0018,
  ADR-0019, ADR-0039, ADR-0041, ADR-0043
- Decision owners: Nate McBride

## Context

`endpoint_pair_allowed` (`crates/domain/src/model/topology.rs`) is the
delivered catalogue of admissible edge endpoints. Asked directly at
`7fdba38`, it admitted `volume → file-system` and `volume →
backing-signature` and refused `volume → partition-table`: a volume may
carry a file system or a signature but not a table, an asymmetry with no
rationale on the record. The consequence is that **a partitioned mdraid
array has no containment expression** — the only route out of an
`Aggregate` is `Production` to a volume, and from the volume no row
reaches a table, so `/dev/md0p1` cannot be represented — and neither
can a GPT inside any mapped volume: LUKS opened to `/dev/mapper/x`
carrying its own table, or a partitioned logical volume.

The issue's own second measurement, at `b3de0cf`, found what the row
alone would ship. With `volume → partition-table` added and nothing
else, on `disk → mdraid signature → array → volume → table → md0p1`,
wiping the member disk that carries the array's only superblock reached
the signature, the array, the volume and the table, and **stopped
there**: `md0p1` survived the destruction of the array's only substrate.
The control on an ordinary disk showed the same shape — that was issue
#347, pre-existing — and the ordering nobody had recorded fell out:
**#347 → #360 → #354's kind half → #333's enforcement**, #347 at the
head. ADR-0043 closed #347 with a release keyed on the step's *target*:
a step whose own destroyed ranges reach its table target destroys the
table, and the destroyed table releases every partition whose name says
it describes it. Its first candidate propagated that release from a
wholly-destroyed target through the cascade — measured to close this
chain — and it was **cut**, because with the row absent the chain was
unbuildable and the clause was uncovered; recorded there, with the
numbers, as this issue's remainder: "#360's own act adds the row, the
fixture, and the propagation together."

Two other things this act stands on. ADR-0039 made every node in the
set propagate to the content it carries — reach — and merged, in the
delivered code, the reach class with what had been the destroyed class:
`cascade_destroyed` after ADR-0039 is "reached by descent", not
"destroyed", which is why a table on a volume reached by `Wipe(member)`
did not release and why a naive "any reached table releases" would make
`Label(member)` release too. And ADR-0041 established that a table's
extent is its own header bytes and that the two `partition-table`-sourced
pairs carry no span claim, which is why the closure's geometry stops at
a table and the release must be a separate rule.

## The decision

> **A step whose own destroyed ranges reach its target destroys the
> target. Destruction is carried from there along the same four arms —
> containment, production, host-backing, and the substrate half of
> backing — under the same geometric bound as reach. Every destroyed node
> releases what its name-roster says it describes: a partition table its
> partitions, every other kind nothing; and a released partition is
> destroyed in turn. And the endpoint-pair table admits
> `volume → partition-table`.**
>
> - *Seeded by the target alone.* The destroyed class holds the target
>   when `target ∈ range_destroyed` — the step's declared destroyed ranges
>   intersect the target's own extent, ADR-0043's trigger unchanged. It is
>   **never** seeded by a range that merely touches some other node: a
>   range-destroyed non-target node is reached, and descends, exactly as
>   before, and establishes no destruction of its own. That is round 2's
>   sibling capture kept out by construction — the L1 BIOS-boot row and
>   the L2 one-byte inflation hold unmoved and are re-asserted by their
>   standing guards, and mutation M3 (seed every range-destroyed node) is
>   killed by four of them.
> - *Carried, never inferred from reach.* Destruction crosses an edge only
>   from a destroyed source, and only where `descends_into` admits the
>   hop — the same bound reach obeys, so nothing this act adds can reach
>   less than reach does, and nothing it adds is unbounded. Reach itself
>   is unchanged: the six operations that destroy nothing still reach the
>   table a volume carries and still release nothing (mutation M2,
>   release on reach, is killed by five, two of them pre-existing
>   guards). Every hop that carries destruction also carries reach, so
>   the destroyed class is a subset of the affected set by construction.
> - *Releases by the roster.* `destroy` marks a node destroyed and then
>   releases every entry whose `released_by_table()` names it — for a
>   partition table that is exactly `Partition { parent_table }`, read off
>   the naming roster and never off an edge (ADR-0043's rule, verbatim);
>   for every other kind the roster names nothing and the node is simply
>   destroyed. A released partition is destroyed, not merely reached, so
>   its own content is destroyed and a table below it — a GPT inside a
>   LUKS volume on a released partition — releases in turn (mutation M4,
>   released partitions merely reached, is killed by the outer-table
>   assertion on that layout).
> - *The row, and the modelling.* `("volume", "partition-table")` joins
>   `EdgeKind::Containment`'s pairs. A partitioned aggregate is
>   `aggregate → volume → partition-table` over the existing production
>   hop: the array's block device is the produced volume, and the table's
>   `parent` names it. **No `aggregate → partition-table` row is added**;
>   aggregates stay out of the containment forest, and the test that
>   pins the honest layouts asserts the pair stays refused. The
>   `multipath-node` rows the issue lists are not added and not decided:
>   ADR-0011's detection-only decision is the plausible reason they are
>   absent, and this act does not examine it. The pair is *geometric* in
>   `containment_pair_is_geometric`'s sense — a table lies within the
>   volume's bytes — and is never compared, because a volume declares no
>   extent (ADR-0041's revisit condition, discharged by reading).

**What is not consulted.** No table's extent decides anything except
the target's own membership in `range_destroyed`. No coverage. No
intersection of a non-target table. No edge decides a release. No
operation is read — the closure still takes none — so the seed is a
property of the declared ranges and the target, as it was.

**The one named limit, pinned rather than hidden.** An extentless
target — a volume, an aggregate, an encryption layer, a multipath node —
has no extent for a canonical destroyed entry to name, so its own wipe
declares no destroyed range, the target is never in `range_destroyed`,
and destruction is never seeded: `Wipe(md0)` reaches the table `md0`
carries as content and releases nothing, `CCCCCCCCCC`. That row is
asserted as a committed limit in
`destruction_carries_only_from_the_target_and_reach_never_releases`, so
closing it is a deliberate change. The candidate that closes it — an
extentless frame root's canonical destroyed entry is its whole frame,
`HostRange { host: target, start: 0, length: u64::MAX }` — was measured
in this round: every child framed on the volume is then range-destroyed
by intersection and the pool is reached without any release; the whole
workspace stays green (669 tests) with only that pinned row moving to
`Unsupported`. It is **held**, and filed as issue #392, on the
uncovered-clause rule this act's own history enforces: it changes
`canonical_ranges`, which `crates/capability` and `crates/planner`
consume, and it changes what the planner's `destroyed_closure` removes
on `Wipe(volume)`, on a population no planner test exercises. A green
suite there proves nothing regressed; it does not prove the new
behaviour sound.

## Measured

Per-(layout, target, operation) gate tables at `d67d4df` → candidate;
`C`/`R`/`B` = Clear/Unsupported/Blocked; columns Create Grow Shrink Move
Repair Label Uuid Encrypt Decrypt Wipe.

| layout / target | before → after |
| --- | --- |
| partitioned mdraid (`disk → md sig → array → md0 → table → md0p1 → zfs → pool`) / **member disk** | unrepresentable → `CCRRCCCRCR`; `Wipe(member)` reaches all eight nodes, refuses through the pool — **#360's chain closed**. (With the row alone, ADR-0043 measured `constructs`, pool unreached.) |
| partitioned mdraid / md superblock signature | → `CCRRCCCRCR` |
| partitioned mdraid / table on `md0` | → `CCRRCCCRCR` (ADR-0043's target-keyed release, unchanged in kind) |
| partitioned mdraid / `md0p1`, zfs signature | → `RRRRRRRRRR` |
| partitioned mdraid / `md0` (volume), array | → `CCCCCCCCCC` — **the named limit**; `Label(member)` reaches the table and releases nothing |
| GPT inside LUKS (`disk → gpt → {esp, crypt} → luks → layer → mapper → inner gpt → p1 → zfs → pool`) / crypt partition | unrepresentable → refuses; `Wipe(crypt)` releases `p1`, reaches the pool |
| GPT inside LUKS / esp | → `CCCCCCCCCC` — nothing on the disk's frame connects the two |
| GPT inside LUKS / outer gpt | → refuses through the inner pool: the released `crypt` is destroyed, and its destruction carries to the inner table |
| partitioned mdraid carrying a plain ext4 / member disk | → `CCCCCCCCCC`; `Wipe(member)` reaches `md1p1` and the fs and **constructs** — the false-refusal control |
| root-on-ZFS / table, esp, member, sda, signature | `CCRRCCCRCR`, `C…`, `R…`, `CCRRCCCRCR`, `R…` — unchanged, affected counts 6/2/4/6/4 unchanged |
| LUKS chain / sdb, part, mapper | `R…` ×3 — unchanged, 8/6/3 |
| BIOS-boot / sda, table, boot (L1), esp, member | `CCRRCCCRCR`, `CCRRCCCRCR`, `C…`, `C…`, `R…` — unchanged, 7/7/3/2/4 |
| whole-disk vdev / sda, signature | `R…` ×2 — unchanged, 3/3 |
| root-on-ZFS, table extent `[0, 1 MiB+1)` / esp (L2) | `C…` — unchanged |

Every existing layout is byte-identical to `d67d4df`, gates and affected
counts both, measured by the same probe on both trees. The affected set
is a superset of HEAD's in every row by construction — the change only
inserts. ADR-0040's whole-disk pin, ADR-0042's four regressions and
ADR-0043's seven tests hold unmoved.

**Mutation battery** (seven, each proven applied by `git diff` or by
grep of the mutated line, the domain suite run): M1 destruction never
carries — killed by two (the chain and the LUKS layout); M2 destruction
carries from any in-set source, i.e. release on reach — killed by five,
two pre-existing (`a_frame_root_target_reaches_what_it_carries_and_no_more`,
`the_target_hop_is_bounded_by_the_same_geometry_as_every_other`); M3
every range-destroyed node seeds destruction — killed by four, the
L1/L2 guards among them
(`a_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table`,
`a_range_that_touches_no_gpt_structure_releases_only_from_a_table_target`,
`a_table_that_is_not_the_target_never_releases`,
`a_released_partition_refuses_only_for_what_it_carries`); M4 released
partitions reached but not destroyed — killed by one, the outer-table
assertion written for it; M5 the target seed removed — killed by seven,
ADR-0043's five among them; M6 destruction carried without the
geometric bound — killed by one (the hybrid-MBR control: the
conflicting entry it carries would be destroyed and go `Blocked`); M7
the backing arm never carries destruction — killed by two. The bound
mutation's single kill is thin and is stated as such: destruction is
carried inside the same `if descends_into` block as reach, so the
standing bound tests constrain it structurally rather than by a second
guard.

Workspace: 669 tests, 0 failed; `cargo xtask ci` exit 0;
`crates/capability` and `crates/planner` unchanged and green.

## The theorem, amended

ADR-0018's non-interference theorem as amended by ADR-0043 read: *no
node whose declared extent is comparable with its reacher's and lies
outside it is ever in the set, except a partition released by the
destruction of the table its own name says describes it, where
membership follows the naming relation and the step's target and never
geometry.* The exception's "step's target" is now too narrow: the table
that releases may be one the carried destruction reaches. Amended, in
ADR-0018's inline style, to say membership follows the naming relation
and *the destruction carried from the step's target*, never geometry
and never reach alone. The consequence stands: a sibling is never
captured by a step that destroys another partition, because a range
that touches a non-target node establishes no destruction of it. The
property tests: `the_release_roster_is_pinned_per_kind` (unchanged —
the roster is the same), and the reach-never-releases guard with the
extentless-target limit pinned.

## Options considered, and rejected

- **Two rows, `aggregate → partition-table` and `volume →
  partition-table`.** The issue's option 1. One row over the existing
  production hop is the sanctioned shape and keeps aggregates out of the
  containment forest; a second row would give the same layout two
  spellings and a partition two possible frames. Rejected; the pair is
  asserted refused.
- **The row alone, no propagation.** Measured at `b3de0cf` and again by
  ADR-0043: ships representation whose protection is known-broken — every
  partitioned array and every partitioned mapped volume would build and
  under-reach. Worse than unrepresentable. Rejected.
- **Release any reached table.** Mutation M2: `Label` would release.
  Rejected.
- **Seed destruction from every range-destroyed node.** Mutation M3:
  round 2's sibling capture returns on the BIOS-boot layout. Rejected.
- **Released partitions reached but not destroyed.** Mutation M4: a table
  below a released partition would not release. Rejected.
- **The whole-frame canonical entry for an extentless target.** Measured
  green across the workspace; held and filed, on the uncovered-clause
  rule, because it moves `canonical_ranges` and the planner's simulation
  (see the named limit).
- **Deriving #354's kind check from the table in this act.** The table
  is now right for the two populations this issue named; the third — a
  file system hosted on a multipath node — is still admitted by no row,
  and whether that is ADR-0011's intent is not examined here. Not
  taken; #354's own act decides it.

## The spec-price argument

**Major, 15.0.0.** Two normative sentences change meaning. §2.1:113's
release clause, added in 14.0.0, said "a partition table destroyed by
its own step's target releases every partition whose name says it
describes it"; a table the carried destruction reaches now releases too,
and the clause is rewritten to say so. ADR-0018's theorem as amended in
14.0.0 said membership "follows the naming relation and the step's
target"; it now follows the destruction carried from the target. Both
are existing text whose claim changes, which under §0.1 is major
whether or not reach only grows — ADR-0039's and ADR-0043's precedent
for this sentence. MODEL-002 gains the sentence that a volume may carry
a partition table and that the pair table admits the row; that alone
would be minor. The minor and patch readings are declined on the two
closure sentences.

**Not a §1.11 filing.** No two requirements conflict; MODEL-002's chain
never claimed to be a single nesting order, and ADR-0018's release
definition was an obligation ADR-0043 discharged for the target and this
act discharges for what the target's destruction carries.

## Consequences

- **Positive:** a partitioned mdraid array and a GPT inside a mapped
  volume are representable, and their protection is measured, not
  assumed; ADR-0043's named remainder is delivered; the destroyed class
  is a set the closure can name again, distinct from reach; #354's kind
  half is unblocked as far as this issue gated it.
- **Negative, accepted knowingly:**
  - The named limit: an extentless target's own wipe is not seen
    destroyed. Filed.
  - Destruction carried through the backing arm reads a destroyed
    member signature as taking the aggregate's products down — for a
    redundant array with a surviving member that over-reaches, exactly
    as ADR-0018's rule 4 already did for reach; it is fail-closed and
    unchanged in kind.
  - Any device-target step whose declared destroyed range touches the
    device now destroys the table the device carries and releases every
    partition — the priced limit ADR-0043 stated for a table target,
    one level up. Nothing delivered emits that spelling: a device
    target's canonical destroyed entry is its whole extent, and the
    planner never targets a device with a partial destroy.

## Verification

- `destroying_a_partitioned_arrays_member_reaches_what_its_partitions_carry`,
  `destruction_carries_only_from_the_target_and_reach_never_releases`,
  `a_table_inside_a_mapped_volume_releases_and_a_plain_one_constructs`
  (`protection_tests.rs`); the re-spelled
  `honest_layouts_the_kind_check_would_have_refused_still_build` and
  `a_wrong_kind_referent_still_builds_and_that_is_the_held_half`
  (`topology_tests.rs`); `every_triple_outside_the_pair_table_is_refused`
  enumerates the extended table.
- Any text implying that a table reached by an operation that destroys
  nothing releases, that a range touching a non-target node destroys it,
  that an aggregate carries a table of its own, or that this act lands
  #354's kind check, is an error against this ADR.
- Any claim that `Wipe` of a volume or an aggregate carrying a table
  reaches the table's partitions is an error against this ADR.

## What stays open

- **The extentless-target limit** — filed as issue #392 with the
  measured whole-frame candidate.
- **#354's kind half**, on the multipath population: a `FileSystem` (or
  a table) hosted on a `MultipathNode` is admitted by no row; whether
  ADR-0011 intends that is the question its act must answer before
  deriving the check.
- ADR-0043's open points stand: what a table's extent *is*, and round
  2's M6.

## Revisit conditions

- The whole-frame candidate lands: this ADR's named-limit row moves and
  the pinned assertion is rewritten deliberately, with the planner's
  simulation covered.
- A row is added whose target is a kind that may carry an extent, on any
  propagating arm: `no_propagating_pair_targets_a_kind_that_declares_bytes`
  reds; on `Containment`, classify it in `containment_pair_is_geometric`
  and re-run this ADR's battery.
- A kind other than `Partition` comes to name a table: classify it in
  `released_by_table`; the roster test reds until it is.
