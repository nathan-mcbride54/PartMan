# WP-060 increment 10 plan — the scheme's own regions, 2026-08-13

Untracked local artifact, docs/reviews convention: never stage into a
commit; `verify-change-ownership` refuses it.

Written before the first line of code, per house convention. Its
inputs are `ISSUE-319_EXTENT_ABSENCE_ROUND_2026-08-13.md` (the
adversarially reviewed recommendation round, which recommended this
act and only this act) and a second adversarial design round of the
same day, whose settled specification this plan schedules.

## What this arc delivers

Issue #319's **planner half**, and nothing else. A host's free extents
become its own extent minus the extents the facts place on it **and
minus the regions the table schemes it declares claim at each end**,
with a separate rule that every partition the authenticated naming
fields place on the host must be one the subtraction actually removes.

**#319 is not closed by this arc.** Its authorization half — the
protection layer's reach — is blocked on issue #333 and stays open.
The PR must say so and must not carry a closing keyword.

## The three measured defects, all re-run by hand at `ecb3dc6`

1. **The filed one.** `place_create(&solver_fixture, host, 1 MiB)`
   returns `start=0 length=1048576 end_placement=Aligned` — a create
   on the protective MBR and GPT header, recorded conforming. The
   fixture's `PartitionTable` node (tests.rs:507-511) has a containment
   edge and no extent, so `free_extents` never subtracts it.
2. **Host-extent overrun.** A device whose naming fields declare
   `total_bytes: 1 GiB` carrying a 2 GiB self-extent yields
   `free = [(0, 2147483648)]`, and `place_create(1.5 GiB)` returns
   `start=0 ... Aligned` — a partition ending 512 MiB past the end of
   the device. Directly against §11.2's "Extents remain inside the
   bound device" (AGENT_BUILD_SPEC.md:856).
3. **Child-extent overrun.** A child range leaving the host's extent is
   absorbed rather than surfaced.

Defects 2 and 3 were found by attacking the fix, not by #319's filing.
**They are reported on #319 as a comment before PR C**, so the arc does
not smuggle unrelated closures into an increment.

## The decision the arc opens under

ADR-0036, reserved by PR A and landed by PR B. Its shape, settled by
the design round after two rival designs died fatally:

- **The reservation is a bound, never a measurement.** No sector size
  reaches this module and `PartitionTable` carries no geometry, so the
  bound is the smallest figure in the module's only unit
  (`DEFAULT_ALIGNMENT`) that covers the scheme's structures at every
  sector size. Head is uniform across recognized roles; only the tail
  is scheme-specific — `Gpt`/`HybridMbr` reserve a tail, `Mbr`/`Apm` do
  not. `Unrecognized { raw }` **refuses**: head/tail is the wrong shape
  of bound for an unknown layout, so reserving a maximum is a guess
  wearing a bound's clothes and reserving nothing fails open.
- **Occupancy is read from the authenticated naming fields, never from
  `topology.edges()`** — an edge is not in any node's hashed address
  preimage, so an edge-sourced roster shrinks silently when one is
  omitted. Held by test, not by argument.
- **The guard never reads `facts.table_states`.** It reads the table
  *node*. This is what keeps it clear of WP-L100 increment 3's charter,
  which emits no table-state stamp on any path (WP-L100.md:295, :399).
- **Nothing turns on issue #333.** The disputed kinds — hosted layers
  anchored on a partition — are deliberately excluded from the
  occupancy roster, and the child bound uses its upper half only. Both
  exclusions were forced by measurement: including either reproduced a
  false refusal on the delivered `created_capture` fixture.

## The PRs, in order

- **PR A — reserve ADR-0036** (`work/wp010-reserve-adr-0036`,
  WP-010-owned Markdown only, trips no stopping condition): the
  `docs/adr/0036-*.md` path added to WP-010's owned paths and the
  reservation note stating the decided shape. The f0ef237 pattern
  exactly. No normative text changes.
- **PR B — spec-change 12.13.0** (`work/wp010-adr-0036-reserved-regions`,
  WP-010-owned): ADR-0036 itself plus the two requirement amendments.
  **Both effects must be in the amendment, not one** — INV-004 gains
  the withholding *and* the unavailability arm; PART-009's lawful
  coincident edges gain the scheme's tail region. The minor price is
  **argued, not asserted**, with the counter-argument recorded and
  declined: the tail withholding under-reports free extents by design,
  and a decision owner who reads that as narrowing INV-004's detect
  duty prices 13.0.0 — in which case nothing in `crates/planner`
  changes, only the version and the changelog row. Also recorded: the
  amendment to ADR-0018's totality passage, cited as its
  **`## Safety analysis`** (0018:54) and never as spec text, and stated
  as prose in the new ADR rather than an edit to ADR-0018 or to code.
- **PR C — WP-060 increment 10** (`work/wp060-increment-10`,
  `crates/planner` only): `reserved_regions` and the occupancy guard
  inside `free_extents`, four new `SolveRefusal` variants with the
  `OccupancyGround` enum, one new `StructuralEdge::ReservedTableRegion`
  with its `Display` arm, `place_create` and `grow_extension` threaded
  through the new geometry, `shrink_reduction` untouched. Fourteen new
  tests, the WP-060.md increment-10 entry, README row, CHANGELOG,
  traceability regenerated **last**.

  Three things this PR must not do: reorder `solve_sized` against the
  table-state guard (a delivered contract it has no grant for); edit
  `solver_fixture` (the extent-less table node is the retained
  witness); or add fields to the new refusal variants without
  re-measuring `size_of::<SolveRefusal>()` — it is 112 bytes against
  clippy's `result_large_err` threshold of 128, and overflowing it
  trips the lint across a dozen untouched functions.
- **The r14 sitting + PR D — the WP-020 re-pin**
  (`work/wp020-r14-repin`): PR C is the arc's only Rust merge and trips
  WP-020's 2e stopping condition — the ninth trip from outside the
  package. **One sitting at the arc's head**, the r13 runbook copied to
  r14 with header prose alone changed, re-taking 2e, 2h
  (`ranges_written=1`) and 2j (`ranges_written=2`), teardown verified;
  PR D records the sitting and re-pins from `b50dd19`. **This
  one-sitting economics is recorded here, before the first merge** — the
  r13 record's honesty note is that the WP-L100 arc failed to do this
  and had to state the choice as its own rather than its plan's.

## The retained witness

`free_extents_are_the_hosts_minus_its_children` (tests.rs:574-593)
asserts `(0, DEFAULT_ALIGNMENT)` is free — a committed test claiming
the fail-open is correct, claimed under PLAN-001 at WP-060.md:32. It
keeps its name and its fixture; the false tuple is deleted, and a new
assertion pins the precondition (`!extents.contains_key(&table_id)`) so
the extent-less-ness becomes the thing the test proves rather than an
incidental property.

**The source annotation at tests.rs:569-573 must be rewritten in the
same commit.** The generator reads claim text out of the annotation, so
leaving it stale would publish, as established PLAN-001 evidence,
exactly the two claims the change removes.

## Verification discipline

Every PR: `cargo xtask ci`, `cargo xtask test --tier 1`,
`cargo xtask verify-change-ownership --base origin/main`, **real exit
codes, not output**. Gate runs from a worktree outside the repo with
its own target dir.

Mutation verification before proposing PR C, applied with Edit and
reverted by re-Edit — never `git checkout` over uncommitted work — and
each mutation asserted present before replacement, so a silently failed
mutation cannot read as a surviving test:

| # | Mutation | Killed by |
| --- | --- | --- |
| R1 | `Gpt`/`HybridMbr` head → 0 | the withheld-regions test, the retained witness, the no-edges property, the odd-sized-host edge test |
| R2 | `Gpt`/`HybridMbr` tail → 0 | those four, plus the grow-short test and the conflicting-view test |
| R3 | `Mbr`/`Apm` head → 0 | the withheld-regions test |
| R4 | `Mbr`/`Apm` tail → 1 MiB | the withheld-regions test **plus two delivered tests** — the delivered ADR-0023 suite already prices MBR over-reservation |
| R5 | `Unrecognized` treated as `Gpt` | the unrecognized-scheme test, the ordering test |
| R6 | conflicting-entry view role not folded | the conflicting-view test |
| O1 | occupancy check deleted | the unaccounted-occupant test, the foreign-table test |
| O2 | located-ness weakened to key presence | the unaccounted-occupant test (three of four grounds flip to `Ok`) |
| O3 | declared-start check dropped | the unaccounted-occupant test |
| O4 | foreign-table arm deleted | the foreign-table test |
| O5 | child bound deleted | the child-overrun test |
| O6 | host bound deleted | the host-overrun test |
| O7 | reserved-tail edge deleted | the odd-sized-host edge test |
| O8 | occupancy sourced from `topology.edges()` | the no-edges property |
| O9 | guard gated on `facts.table_states` | the blind-to-table-state test |

**O2, O3, O8 and O9 are the load-bearing ones**: O2 is the exact fatal
that killed two rival designs, and O8/O9 hold the two landing
conditions by test rather than by the code's current shape. A surviving
mutant means a missing fixture, added before proposal.

## Traceability — one decision to make

New tests claim **PLAN-001** (already declared) and **PART-009** (its
planning half already declared, WP-060.md:30-32).

**INV-004 is deliberately not claimed.** WP-060's Requirement IDs line
(WP-060.md:17-28) does not name it, and adding it would expand the
assignment's scope inside an implementation PR while the round's own
open question 4 — whether `free_extents` is INV-004's delivered surface
or a PLAN-001 placement computation — is unsettled. PR B amends
INV-004's text under WP-010's grant regardless; that is a spec act, not
a traceability claim. **If the decision owner wants INV-004 claimed, it
is a one-line assignment edit in PR C** on the increment-7 precedent.

Regenerate with `cargo xtask traceability --write` last; bare IDs on
`Requirements:` lines.

## Merge mechanics

A → B → C, then the r14 sitting, then D. Merging one PR invalidates the
siblings' required checks — update-branch all at once and merge in
order. `gh pr checks` races check registration: delay before watching
and let branch protection backstop; verify `gh pr view <n> --json
headRefOid` against the pushed SHA before merging, since watcher labels
read local `HEAD`.

Commit bodies and PR bodies via files (`git commit -F`,
`gh --body-file`) — PowerShell here-strings keep `''` literal.

## What this arc does not close

- **#319's authorization side** (round Q2), blocked on #333.
- **Hosted layers** — a whole-device LUKS, ZFS or superfloppy file
  system naming the device as host and carrying no extent still has its
  bytes handed out. Excluded deliberately: including it produced a
  measured false refusal on a delivered fixture. The scoped fix once
  #333 resolves is to require them located only when the host is a
  `PhysicalDevice`, which both #333 readings agree about.
- **`ConflictingTableEntry` bytes** — the variant carries `entry_start`
  and no length, so no bound over it is computable.
- **Orphan partitions** whose `parent_table` names an address no entry
  derives — domain-side referent validation, WP-010's package.
- **The tail's 50× over-reservation.** Tightening it needs real table
  extents — plural extents per node, a backup `TableRole`, or table
  regions as nodes — which is a carriage change and the subject of
  #319's open question 3.
