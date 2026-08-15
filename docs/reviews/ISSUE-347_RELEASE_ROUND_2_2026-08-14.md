# Issue #347: a destroyed table releases what it describes — round 2, 2026-08-14

Untracked session artifact, `docs/reviews` convention.

> **Candidate measured, NOT landed.** Branch
> `work/wp010-347-release` (pushed, no PR). It satisfies three of the four
> requirements the previous round's §11 set; the fourth is a theorem
> amendment that is a decided act and is deliberately not made.
> Supersedes `ISSUE-347_TABLE_RELEASE_ROUND_2026-08-14.md`, whose §3
> candidate was rejected — its §1, §2, §7, §8, §10 and §11 stand.

## 1. The defect, re-measured on current main (`b3de0cf`)

The rule is to measure an issue before working it. Ten-operation gate,
committed `root_on_zfs` fixture, **table** target:

| | HEAD | candidate |
| --- | --- | --- |
| Wipe / Encrypt / Move / Shrink | **`Clear`** | `Unsupported{Zfs}` |
| Grow / Create / Repair / Label / Uuid / Decrypt | `Clear` | `Clear` |
| affected set, wipe | 2 (table, sda) | 6 |
| pool reached | **false** | true |

**HEAD is 10/10 `Clear`.** Wiping the partition table of a disk carrying a
live ZFS vdev constructs, with the pool's own `Refused{Zfs}` never
consulted. Device-target behaviour is identical under both
(`Wipe → Unsupported{Zfs}`, `Repair → Unsupported{Zfs}`), so nothing
claimed here is inherited from ADR-0039's device reach.

## 2. The design, and why it is this shape

Round 1 §2's finding is the whole basis. `EdgeKind::Containment` is
documented as "positional nesting inside one addressable byte space" and
carries two different relations: for `partition → file-system` the bytes
really are nested; for **`table → partition` they are not** — the
committed fixture has the GPT at `[0,1 MiB)` and its partitions past it.
A partition is *described by* its table.

So ADR-0039's geometric bound is correct for nesting and asks an
unanswerable question on the descriptive edge, answering it "no".

`describes_rather_than_nests(topology, source, target)` names the two
descriptive pairs — `partition-table → partition` and
`partition-table → conflicting-table-entry` — read off delivered kind
names. For those, and only when the source is in a **destroyed** class,
the release bypasses the geometric bound.

## 3. Against the previous round's §11 requirements

| requirement | status |
| --- | --- |
| 1. **Union semantics** — whole-destruction over the union, never one range | **Met by construction.** Membership in `range_destroyed` is decided per range by `intersects`; the same bytes re-spelled as two adjacent ranges give an *identical* set. Asserted. |
| 2. **Monotone in the declared extent** | **Met by construction.** Growing the extent makes intersection more likely, never less. Both inflation and deflation asserted; the previous `contains` predicate died to one byte of inflation. |
| 3. **The re-proof obligation, named explicitly** | **Named, not discharged.** See §5 — it needs an amendment, which is a decided act. |
| 4. **Can any authored field remove the refusal?** | Tested three levers: range re-spelling (no), extent inflation (no), extent deflation (no). A fourth is closed structurally — `("partition-table","partition")` is the *only* containment pair that can target a partition, so an author cannot re-kind the source and keep the edge. A fifth remains open: see §6. |

## 4. Mutations — applied with `Edit`, proven applied before each run

| mutation | outcome |
| --- | --- |
| release clause disabled | killed by both new tests |
| release gated on `affected` rather than destroyed | killed by **five**, four of them *committed* guards |
| `describes_rather_than_nests` widened to every containment pair | killed by **five**, four of them *committed* guards |

The committed guards that fire are `a_sibling_esp_is_never_captured`,
`an_ordinary_disk_keeps_its_siblings_out_of_the_set`,
`the_root_on_zfs_regression_pair_holds` and
`ungating_rule_three_membership_never_captures_a_sibling`. **The
narrowness and the gating are load-bearing and the existing suite already
protects both** — which is the strongest evidence available that this
shape is not free-floating.

Gates: `cargo test --workspace --no-fail-fast` 648 passed, 0 failed;
`cargo xtask ci` **exit 0**, 605 annotations over 641 live tests;
`verify-change-ownership` 3 paths, WP-010.

## 5. The open decision, which is why this is not landed

ADR-0039 amended ADR-0018's non-interference theorem to "no node whose
declared extent is comparable with its reacher's and lies outside it is
ever in the set" (`0018:187-190`), and ADR-0018:210-217 makes re-proving
it a **precondition of acceptance**.

**This candidate makes that theorem false as written.** The ESP's extent
is comparable with the table's and lies outside it, and this puts the ESP
in the set — exactly the property round 1's §10.3 flagged.

The argument for *amending* rather than abandoning: a descriptive edge
relates no two extents, so "lies outside it" has no meaning across one.
The theorem's force is about **nesting** containment, where an extent
comparison is a real statement; extending it to an edge that never made a
geometric claim is what produced the defect. A candidate amendment:

> …is ever in the set, **except across an edge that describes rather than
> nests, where the two extents stand in no containment relation and the
> release is structural rather than geometric.**

That is a decided act. Round 1 §6.1 judged this ground a decided act
rather than a defect fix — "it changes what a step's affected set
contains, which is exactly the ground ADR-0038 and ADR-0039 each took an
ADR and a spec bump for" — and that judgment is not disturbed by this
round being better measured.

## 6. Known limits, stated rather than left to be found

- **The release is edge-driven.** A body omitting the
  `table → partition` edge gets no release. That lever is pre-existing
  and is #356's and #354's territory, not introduced here.
- **The hybrid `ConflictingTableEntry` cost** priced in round 1 §7.2 is
  unchanged in kind by this shape and was not re-measured.
- **A LUKS chain is still unmeasured**, as in round 1.
- **No adversarial round has run on this candidate.** Round 1's candidate
  was green on the full workspace and died to two fatals a round found.
  Three designs in this family have now been rejected after looking
  green; this one should not be trusted further than that record allows.
- The `Containment` documentation correction that round 1 §2 identified
  is still owed by whichever act lands, and outlives every candidate.

## 7. What this unblocks if it lands

Measured this session and recorded on #360 and #347: the chain is
**#347 → #360 → #354's kind half → #333's enforcement**. Landing a
release fix is the head of that queue — #360's `volume → partition-table`
row cannot land while a destroyed table reaches none of the partitions it
releases, because it would make partitioned mdraid arrays representable
and every one of them would under-protect.
