# ADR-0041: The body's facts are validated against its topology at assembly

- Status: Accepted
- Date: 2026-08-15. Made on the measured round of 2026-08-15
  (`docs/reviews/ISSUE-349-356_BODY_VALIDITY_ROUND_2026-08-15.md`,
  single-author with a twelve-mutation battery, each mutation proven
  applied and each killed; committed under WP-000 beside this act).
  Everything load-bearing is restated here. Merging is not acceptance —
  every element below is reviewable against the round's recorded
  alternatives, and the decision owner has not been put the question
  in person; this ADR is where it is put.
- Spec version: **13.1.0 — minor under §0.1.** The argument is made
  below, with the patch reading recorded and declined.
- Work packages blocked: none. Issues #349 and #356 close here as
  filed; what each leaves open is named in *What stays open*.
- Requirement IDs: MODEL-002, MODEL-003, MODEL-005, SAFE-005, ADR-0037
- Decision owners: Nate McBride

## Context

`TopologySnapshot` carries a validated `Topology` and a `Facts` set —
extents, transports, member counts, table states — that the protection
closure reads as authenticated body content. The topology was validated
at construction since increment 3c: edge endpoints resolve, endpoint
pairs are in the pair table, naming referents resolve (ADR-0037's owed
sweep, PR #362). **The facts were not.** `assemble` stored them verbatim
(`snapshot.rs:91-106` at `b8d6a90`); the decode path checked an extent
only for kind-misplacement, and `extract_extent` accepted any triple
whose `host` was 32 bytes and whose numbers were unsigned.

Two issues measured the consequences at `5b795df` and `8e03e68`:

- **#349.** A body declaring a zero-length extent, an extent hosted by an
  address no entry carries, or one whose `start + length` overflows,
  decodes, validates, recomputes and hashes cleanly. A ZFS label declared
  with `length: 0` is invisible to `HostRange::intersects`. And `assemble`
  applied none of the decode path's placement checks, so the planner and
  the capability engine — in-process callers — could hold a snapshot the
  body boundary would refuse: an extent on a `Volume` assembles and
  fails to round-trip.
- **#356.** A `Containment` edge and an extent fact are two positional
  claims about the same node, and nothing compared them. A body saying
  a signature is inside a partition at `[0, 100 MiB)` while its extent
  puts it at 500 MiB assembles, round-trips with agreeing hashes, and
  the closure — correctly, given the facts it is handed — approves
  deleting the partition that carries a live pool's label. The issue's
  own correction (2026-08-14) established that the **absent**-extent
  spelling reaches the same approval by a different route, which is
  issue #319's class and not this one.

A scoping document of 2026-08-14 put "validate the whole body" to an
adversarial pass and withdrew it (`VALIDATION_ACT_SCOPE_2026-08-14.md`
§8): full validation would have prevented at most one of five rejected
closure predicates; a uniform frame rule would subtract `HostBacking`
reach; and #356's contradiction body is not the only escape. What
survived that pass is the narrow act this ADR takes: **well-formedness,
one constructor path, and containment agreement where the two claims are
comparable** — and nothing that decides what ADR-0037 holds.

## The decision

> **The evidence-contract facts are validated against the topology at
> `TopologySnapshot::assemble`, and a snapshot that fails validation is
> not constructed.** `assemble` is the one path: the decode boundary
> rebuilds through it, so no snapshot can exist whose facts would be
> refused on the other path. Six rules, each refusing only what is
> positively unlawful, each naming the node it is about:
>
> 1. **Every fact is keyed by an absorbed entry** (`OrphanFact`), and
> 2. **that entry's kind carries the fact** (`MisplacedFact`): a
>    transport or table state on a physical device, a member count on an
>    aggregate, an extent on a kind `may_carry_extent` admits — the same
>    predicates the decode path read, applied to both paths.
> 3. **An extent's `host` is an absorbed entry** (`UnresolvedExtentHost`).
> 4. **An extent has at least one byte** (`ZeroLengthExtent`).
> 5. **An extent's `start + length` does not overflow `u64`**
>    (`ExtentOverflows`).
> 6. **A containment child lies within its parent, where the pair is
>    geometric and the frames are comparable**
>    (`ExtentOutsideContainmentParent`). Comparable means: the two extents
>    share a host, in which case `parent.contains(child)`; or the child is
>    framed on the parent itself, in which case `child.end <= parent.length`.
>    Geometric means every Containment pair whose source is not a
>    `partition-table`.

**Why the table pairs are structural.** A partition table's extent is the
table structure's own bytes — protective MBR, header, entry array — not
the region it governs. Every committed GPT fixture puts `p1` at
`table.start + table.length` exactly, and the BIOS-boot layout the
issue-347 round-2 panel asked to have committed puts one entry *inside*
the first MiB and the rest beyond it. Read as geometric, the rule refuses
every honest GPT disk in the population; measured under mutation M6, a
**pre-existing** step test (`a_declared_partial_shrink_over_a_live_vdev_is_unconstructible`)
goes red beside the two new ones. So `partition-table` → `partition` and
`partition-table` → `conflicting-table-entry` carry no span claim; the
seven other pairs — a table, signature or file system inside a device, a
signature or file system inside a partition or a volume — do.

**Why "refuse the body", not "prefer a claim".** The edge and the fact
are both authored. Trusting the extent silently re-admits #319's class;
trusting the edge discards the byte scan's own evidence. A snapshot whose
two claims contradict is not a capture of anything, and `Facts`' own
posture — absence is honest and fails closed at the arm that needs it —
does not extend to presence that lies. Refusal at construction is the
one place a contradiction can be answered rather than propagated.

**Under MODEL-003, the explicit-rejection limb, `SCHEMA_VERSION` left at
1.** On PR #362's precedent: the byte format, field shapes and parse
rules are untouched; the refused population is bodies that were never
lawful, only unvalidated. Bumping the version would make every existing
v1 body undecodable, the cross-language golden vector included, to
migrate nothing. Evidence that no conforming artifact changes meaning:
the golden vector, `body_vectors.rs`, and all 640 previously committed
tests are unmoved but one — a planner test that built two of the refused
shapes on purpose, adjusted first under WP-060's own grant (PR #377) so
this act lands green.

**One consequence for the decode path's error surface.** Its four
placement checks are deleted and `SnapshotSchemaError::MisplacedFact` is
retired: a misplaced fact now surfaces as
`SnapshotSchemaError::Rebuild(SnapshotError::Facts(FactError::MisplacedFact{..}))`,
carrying the node and its kind, and it is **equal by value** to what the
in-process constructor returns for the same facts. That equality is a
committed test.

## Measured

At the candidate (`d1b9c34`, over `b8d6a90`), `cargo test -p partman-domain`
126 passed:

| shape | at `b8d6a90` | under this ADR |
| --- | --- | --- |
| zero-length partition extent | assembles | `ZeroLengthExtent` |
| `start = u64::MAX-1, length = 2` | assembles | `ExtentOverflows` |
| `start = u64::MAX-1, length = 1` | assembles | assembles (a range) |
| extent framed on an unabsorbed address | assembles | `UnresolvedExtentHost` |
| any fact keyed by an unabsorbed address | assembles, fact silently absent from the body | `OrphanFact` |
| extent on a `Volume`, in-process | assembles; decode refuses | `MisplacedFact`, both paths, equal by value |
| #356: signature edge-nested in `[0,100 MiB)`, extent at 500 MiB, device-framed | assembles; delete constructs | `ExtentOutsideContainmentParent{sig, part}` |
| the same, partition-framed at 500 MiB | assembles | refused |
| signature at `[100 MiB−1, +2)` — starts inside, ends outside | assembles | refused |
| signature at 1 MiB / 99 MiB, either frame | assembles | assembles |
| signature framed on an unrelated absorbed device | assembles | assembles (left alone) |
| signature under a partition with no extent (golden vector's shape) | assembles | assembles (left alone) |
| **#356's absent-extent spelling**, delete the partition | constructs, `affected=2`, pool unreached | **constructs, `affected=2`, pool unreached — unchanged, #319's class** |
| BIOS-boot GPT: bios_grub `[17408, 1 MiB)` inside table `[0, 1 MiB)`, ESP and member beyond it | assembles | assembles; its `f11`/`f12` assertions hold |
| GPT table extending past its device | assembles | `ExtentOutsideContainmentParent{table, dev}` |
| honest bodies, workspace-wide (LUKS chain, LVM, mdraid, hybrid MBR, host-backed loop, every planner and engine fixture) | assemble | assemble — 640 tests, 0 failed with WP-060's adjustment |

**Mutation battery** (twelve, each proven applied by `git diff` before
the run, the domain suite run, the file restored): every mutation was
killed. Dropping the zero-length, overflow or host rule is killed by
`an_extent_that_is_not_a_range_refuses_at_assembly`; dropping the orphan
check by `an_orphan_fact_refuses_at_assembly`; dropping the misplacement
check by `assembly_and_decode_refuse_the_same_facts` and the older
`misplaced_facts_are_typed_refusals`; making the table pair geometric by
the BIOS-boot fixture, the table-pair test, **and a pre-existing step
test**; making every pair structural, dropping the same-frame branch, or
skipping the geometry check entirely by four tests; dropping the
parent-framed branch, or writing `<` for `<=` in it, by
`a_containment_child_outside_its_parent_refuses`; and not calling the
validation from `assemble` by eight.

## Options considered, and rejected

- **Full boundary validation (frames, referent kinds, sibling
  non-overlap).** Rejected on the 2026-08-14 adversarial pass: it would
  enforce ADR-0037's held rule, subtract `HostBacking` reach, and refuse
  hybrid views' aliased entries. This ADR carves each of those out
  explicitly.
- **A blanket "child within parent" for every containment pair.**
  Rejected on measurement — mutation M6 above. It refuses the honest
  population.
- **Prefer the edge, or prefer the extent, when they disagree.** Rejected
  as stated in *The decision*; either preference hands authored content
  a lever.
- **`Indeterminate` at the consuming arm instead of refusal at
  assembly.** Rejected: it leaves the snapshot constructible, so every
  in-process consumer still holds the contradiction, and it is the shape
  the 2026-08-14 round found silenceable by a decoy parent.
- **Keep the decode path's own placement checks beside the constructor's.**
  Rejected: two textual copies of one rule are what let the asymmetry
  arise; the decode boundary now reads the constructor's answer.
- **Zero-length as lawful ("a positively observed absence is a value",
  MODEL-004).** Rejected: an extent is a positional claim about bytes,
  and MODEL-004's sentence is about *observations*, whose honest form
  here is omitting the fact. A zero-length node cannot intersect
  anything, so admitting it admits an unscannable claim.
- **`SCHEMA_VERSION` 2.** Rejected as above.

## The spec price argument

**Minor, 13.1.0.** No text in `AGENT_BUILD_SPEC.md` says what a
snapshot body's facts may declare; MODEL-005 fixes the encoding and
MODEL-003 the versioning, and neither narrows here. This ADR decides
what a body may say — a requirement-shaped act in previously unspecified
territory, which is precisely ADR-0037's pricing for its anchoring rule
(12.14.0), and no existing requirement's text changes or narrows.

**The patch reading, recorded and declined.** ADR-0040 priced a
retirement that changed no production line as patch. This one changes
production lines: bodies that decoded refuse. §0.1 prices requirements,
not implementations, and the requirement here is new rather than
corrected — minor.

## Consequences

- **Positive:** a snapshot's facts are well-formed and agree with its
  containment structure wherever the two can be compared, on both
  construction paths; a closure predicate reading `Facts.extents` can
  assume every extent is a range framed on a real entry; #356's
  contradiction body cannot exist; the population gains the
  overlapping-geometry fixture round 3 of #347 needs, with `f11`/`f12`
  standing as committed assertions.
- **Negative, accepted knowingly:**
  - **This does not make the reach sound.** The absent-extent spelling
    of #356's approval is unchanged and is #319's; a child in an
    incomparable frame is unvalidated until ADR-0037's enforcement
    lands (#333); sibling overlap is not checked. The 2026-08-14 pass's
    finding stands: validation buys self-consistency, which an author of
    the whole capture satisfies for free.
  - `OccupancyGround::RangeIsEmpty` and the unabsorbed-host spelling of
    `RangeOnAnotherHost` are unreachable through a snapshot. WP-060 kept
    both, asserted on its extracted helper, as the solver's own defence.
  - The decode boundary's error for a misplaced fact changed shape.
- **The pair-table reading is now load-bearing twice.** `endpoint_pair_allowed`
  says which pairs exist; `containment_pair_is_geometric` says which of
  them carry a span claim. A kind added to the Containment table must be
  classified there, and the BIOS-boot fixture is the regression that
  catches a wrong classification of the table pairs.

## Verification

- `an_extent_that_is_not_a_range_refuses_at_assembly`,
  `an_orphan_fact_refuses_at_assembly`,
  `assembly_and_decode_refuse_the_same_facts`,
  `a_containment_child_outside_its_parent_refuses`,
  `a_partition_beyond_its_tables_own_bytes_is_lawful`,
  `a_forged_extent_refuses_at_the_boundary_before_any_closure_runs`
  (`snapshot_tests.rs`);
  `a_bios_boot_gpt_disk_assembles_under_the_validity_rules`,
  `a_sibling_esp_is_never_captured_when_the_deleted_partition_nests_in_the_table`,
  `a_range_that_touches_no_gpt_structure_does_not_release_the_disk`
  (`protection_tests.rs`).
- Any text implying that a zero-length or overflowing extent is a lawful
  fact, that `assemble` accepts what `from_canonical_body` refuses, or
  that a `partition-table` → `partition` edge asserts span containment,
  is an error against this ADR.
- Any claim that this ADR closes issue #319's remaining half, enforces
  ADR-0037, or closes issue #347, is an error against this ADR.

## What stays open

- **#319's authorization half**, whose absent-extent spelling this ADR
  re-measured and left constructing.
- **#333**, ADR-0037's held enforcement — a child in a frame its parent
  cannot be compared against passes rule 6 by design.
- **#347**, whose round 3 now has the fixture the round-2 panel required.
- **A device's extent against its own `total_bytes`** — a naming-field
  versus fact contradiction, adjacent to #349 and not taken; the
  planner's `HostExtentExceedsDevice` remains its consumer-side check.

## Revisit conditions

- A kind is added to the Containment pair table: classify it in
  `containment_pair_is_geometric` and extend the fixture population.
- ADR-0037's enforcement lands: rule 6's "incomparable frame" branch
  should then be re-read, since fewer frames will be incomparable.
- A capture adapter is measured producing a zero-length extent for a real
  structure: the zero-length rule's premise — that the honest form is
  omission — would need re-arguing against that adapter's evidence.
