# Issues #349 and #356 — the body-validity round, 2026-08-15

`docs/reviews` artifact, committed under WP-000 in its own pull request
beside the act it records (WP-010, ADR-0041). **Single-author.** No
multi-agent panel ran; the adversarial content here is a twelve-mutation
battery, each mutation proven applied, plus probes for false refusals and
for the escapes the act does not close. That is a weaker review than the
rounds behind ADR-0037–0040 and it is stated as such; the ADR is where
the decision owner is put the question.

## 1. The question, and what was already known

Should the body's evidence-contract facts be validated against its
topology, and if so where and how strictly? #349 (an extent triple's
well-formedness; `assemble` versus decode) and #356 (a containment edge
against the extent facts) each listed four decisions.

What the record already held, and this round did not re-derive:

- `VALIDATION_ACT_SCOPE_2026-08-14.md` §8: full boundary validation was
  **withdrawn** — it would prevent at most one of five rejected closure
  predicates; a uniform frame rule subtracts `HostBacking` reach; #356's
  contradiction body is not the only escape (the absent-extent spelling
  reaches the same approval, #319's class).
- ADR-0037: frame enforcement is **held** (#333); the golden vector is
  partition-framed and unlawful under the rule and not to be corrected
  before the enforcement PR.
- PR #362's MODEL-003 treatment: explicit rejection, `SCHEMA_VERSION`
  left at 1, no spec bump for an owed sweep.
- The #347 round-2 panel's instruction: commit the overlapping-geometry
  shape (`f11`/`f12` on `bios_boot_gpt`) before measuring any candidate
  in that family.

## 2. The finding that shaped the rule

**A blanket "containment child lies within its parent" refuses the
honest population.** In every committed GPT fixture the partitions are
containment *children of the table* and lie geometrically *outside* the
table's `[0, 1 MiB)` extent — `table.start + table.length == p1.start`
exactly. So `EdgeKind::Containment`'s "positional nesting inside one
addressable byte space" is a statement about the *frame*, not about the
parent's span, for the two `partition-table` pairs. Measured under
mutation M6: making the table pair geometric reds the BIOS-boot fixture,
the pair test, **and a pre-existing step test**
(`a_declared_partial_shrink_over_a_live_vdev_is_unconstructible`). The
rule is therefore per pair: seven geometric, two structural, read off the
pair table's source kind.

## 3. The candidate

`validate_facts(topology, facts)` in `protection.rs`, called by
`TopologySnapshot::assemble` — the one path. Six rules; each refuses only
what is positively unlawful; each names its node. The decode path's four
placement checks are deleted and `SnapshotSchemaError::MisplacedFact`
retired; the boundary's refusal is `Rebuild(SnapshotError::Facts(..))`,
**equal by value** to the constructor's. Rule 6 compares a containment
child to its parent only where the pair is geometric and the frames are
comparable (same host → `parent.contains(child)`; child framed on the
parent → `child.end <= parent.length`); an incomparable frame and a
parent with no extent are left alone.

## 4. Measured

Domain suite at the candidate: 126 passed. Workspace with WP-060's
adjustment: 649 live tests, 0 failed, `cargo xtask ci` exit 0.

| shape | before | after |
| --- | --- | --- |
| zero-length / overflowing / unabsorbed-host extent | assembles | refused, node named |
| `start = u64::MAX-1, length = 1` | assembles | assembles (a range) |
| orphan fact of each kind | assembles, absent from the body | `OrphanFact` |
| extent on a `Volume`, in-process | assembles | `MisplacedFact`, both paths, equal by value |
| #356's contradiction (device- or partition-framed at 500 MiB) | assembles, delete constructs | `ExtentOutsideContainmentParent{sig, part}` |
| starts inside, ends outside | assembles | refused |
| honest, either frame, exact fit | assembles | assembles |
| unrelated absorbed frame; parent with no extent | assembles | assembles (left alone) |
| **#356 absent-extent spelling** | constructs, `affected=2`, pool unreached | **unchanged** — #319's class |
| `bios_boot_gpt` (bios_grub inside the table's MiB, ESP beyond) | assembles | assembles; `f11`, `f12` hold |
| GPT table past its device | assembles | refused |

**Blast radius.** One committed test moved in the whole workspace:
WP-060's `an_unaccounted_occupant_refuses_naming_what_the_facts_carry_instead`,
which built a zero-length extent and an unabsorbed-host extent on purpose
to reach two `OccupancyGround` arms. It could not be edited under WP-010,
and a domain change that reds a planner test has no green ordering — so
WP-060 landed first (PR #377), asserting every ground on an extracted
`occupancy_ground(located, host, declared_start)` helper and naming a
device the snapshot absorbs for the other-host case. Green at HEAD and
green with the act, measured on a throwaway merge before either landed.

## 5. The mutation battery

Each applied by `sed`, proven applied by a non-empty `git diff --stat`,
the domain suite run, the file restored from the candidate commit.

| # | mutation | outcome |
| --- | --- | --- |
| M1 | zero-length rule dropped | killed: `an_extent_that_is_not_a_range_refuses_at_assembly` |
| M2 | overflow rule dropped | killed: same |
| M3 | host-resolution dropped | killed: same |
| M4 | orphan check dropped (misplacement kept) | killed: `an_orphan_fact_refuses_at_assembly` |
| M5 | misplacement dropped (orphan kept) | killed: `assembly_and_decode_refuse_the_same_facts`, `misplaced_facts_are_typed_refusals` |
| M6 | table pair made geometric | killed: `a_bios_boot_gpt_disk_assembles_under_the_validity_rules`, `a_partition_beyond_its_tables_own_bytes_is_lawful`, **`a_declared_partial_shrink_over_a_live_vdev_is_unconstructible`** (pre-existing) |
| M7 | every pair made structural | killed: 4 tests |
| M8 | same-frame containment dropped | killed: 4 tests |
| M9 | parent-framed branch dropped | killed: `a_containment_child_outside_its_parent_refuses` |
| M10 | parent-framed `<` for `<=` | killed: same |
| M11 | validation not called from `assemble` | killed: 8 tests |
| M12 | geometry check skipped entirely | killed: 4 tests |

## 6. Rejected here

- Blanket child-within-parent (§2).
- Preferring either claim on contradiction; `Indeterminate` at the arm
  instead of refusal at assembly (leaves the contradiction constructible
  and is the shape the 2026-08-14 round found silenceable).
- Keeping the decode path's own placement checks beside the
  constructor's (two textual copies of one rule are what let the
  asymmetry arise).
- Zero-length as lawful under MODEL-004's "positively observed absence":
  that sentence is about observations; the honest form is omitting the
  fact.
- `SCHEMA_VERSION` 2.
- Sibling non-overlap; a device's extent against `total_bytes`;
  enforcing ADR-0037's frame rule; regenerating the golden vector.

## 7. What this does not establish

- That the reach is sound. Validation buys self-consistency; the
  2026-08-14 pass's §8.5 stands.
- Anything about #319, #333, #347 or #360 beyond what §4 measured.
- A multi-agent adversarial verdict. If one is wanted, §3–§6 are the
  claims to attack; the fixture that would most repay attention is any
  honest layout in which a geometric pair's child is legitimately
  outside its parent's span — none was found in the committed population
  or in the layouts the closure records name (LUKS chain, LVM, mdraid,
  hybrid MBR, host-backed loop, root-on-ZFS, BIOS-boot GPT).
