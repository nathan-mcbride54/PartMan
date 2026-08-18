# Handoff — 2026-08-13/14, the #319 → #333 → #338 arc

**From:** Claude (Opus 5), the session Nate directed with "What should we
tackle next?", then step by step through four issues, three spec
versions and three sittings.
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-13_OPUS_WPL100_ARC_TO_NEXT.md` and
`HANDOFF_2026-08-13_FABLE_WP060_ARC_TO_NEXT.md`. The rounds this session
wrote before its decisions are `ISSUE-319_EXTENT_ABSENCE_ROUND_2026-08-13.md`,
`ISSUE-333_ANCHORING_ROUND_2026-08-13.md`,
`ISSUE-338_CLOSURE_SEED_ROUND_2026-08-13.md`, and the arc plan
`WP-060_INCREMENT_10_PLAN_2026-08-13.md`.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at the #346 merge, **spec 12.14.0**. Working tree clean apart
from untracked docs/reviews. WP-020 re-pinned at `901c7d2` after the r16
sitting; `git diff --name-only 901c7d2 HEAD` must list Markdown only.
Three open issues: **#319**, **#338**, **#333**(decided, enforcement
held), plus the pre-existing **#318**.

## 1. What this session did — thirteen merged PRs, three issues filed

| PR | What |
| --- | --- |
| #334 / #335 | Reserve ADR-0036; spec-change 12.13.0 — the scheme's own regions and located occupancy |
| #336 | WP-060 increment 10 — the `crates/planner` implementation |
| #337 | WP-020 r14 re-pin at `1f9f2c7` |
| #339 / #340 | Reserve ADR-0037; spec-change 12.14.0 — the containment-frame anchoring rule |
| #342 / #343 | The `plan_set` panic fix; WP-020 r15 re-pin at `f463d58` |
| #344 / #345 | Reserve ADR-0038; the release-operation correction |
| #346 | WP-020 r16 re-pin at `901c7d2` |

Issues filed: **#333** (anchoring), **#338** (the closure's seed class),
**#341** (the `plan_set` panic — fixed and closed).

## 2. Decisions worth review

Merging is not acceptance. Each is reviewable and reversible.

- **ADR-0036**: `Shrink`/`Move` aside, free extents withhold the regions
  a host's declared table schemes claim, as a **bound** derived from the
  hashed `TableRole`. Occupancy from naming fields, never edges.
  Rejected on measured fatals: keying on extent *presence*, and refusing
  outright on an unlocated table.
- **ADR-0037**: a range in a containment forest is expressed in that
  forest's **root** address space. **Enforcement held** — the only
  delivered enforcement broke 14 committed tests. The front-runner is
  naming-field-derived, **derive-and-compare only**.
- **ADR-0038**: `Shrink` and `Move` take the **conservative** entry (the
  whole target extent destroyed), and rule 3's membership half is
  ungated. Conservatism is argued **per operation by measurement**
  because monotonicity is false.
- **INV-004 claimed by WP-060** (increment 10). A scope expansion I made
  rather than one Nate approved; the reversal condition is written into
  the assignment — read `free_extents` as PLAN-001-only and the entry
  and annotations come out with no code change.

## 3. What remains open

1. **#338 is the real blocker**, not #333. Defect **(b)** — partial
   destruction missing children outside the destroyed sub-range — and
   defect (a) for the **six non-release operations**. All three widening
   designs were rejected on measurement; grounds in the round.
2. **#319's authorization half**, blocked on #338.
3. **#333's enforcement**, held. Its rule is decided; the golden vector
   and `plan_tests.rs` are **unlawful under it** until the enforcement
   PR regenerates them — a versioned act with its own MODEL-003 debt.
4. **#318** items 3 (protocol half), untouched.
5. **The `protection.rs:28-29` citation** still points at a nonexistent
   "ADR-0018 2.11". ADR-0037 records re-citing it to
   `EdgeKind::Containment`'s own doc as an obligation on whichever PR
   next touches that file.

## 4. Corrections this session made to its own work

Five, and the pattern is the point: **every one came from grounding the
next step, never from the round that preceded it.**

1. **#338's body overstated the defect** — "the closure does not run at
   all" is too broad; `solve_sized` seeds correctly. Corrected on the
   issue.
2. **The first correction was also wrong** — `protection_gate` does not
   return `Clear` on `root_on_zfs`; all eight refuse there. The fixture
   with teeth is `the_luks_descent_reaches_the_pool_below`. Second
   correction posted.
3. **Stale spec-version references** in README and `test-tiers.md`
   (12.12.0 against a spec at 12.14.0) — drift I introduced by landing
   two spec changes without sweeping the documents that name the
   version.
4. **The README's "#319 blocked on #333"** — I wrote the correction into
   ADR-0037 and left the README contradicting it.
5. **The README's "eight of the twelve operations"** — true when
   ADR-0037 was written, stale the same day when ADR-0038 narrowed it to
   six.

**Corrections 3–5 were found only by sweeping *backwards*** — grepping
for what the new numbers contradict. The forward sweep never finds them.

## 5. Operational notes

- **GitHub's closing-keyword parser reads negations as closes.** PR
  #334's "It does **not** close #319" and #344's "no authority to
  **close #338**" both auto-closed those issues on merge. Both reopened
  with an explanation. **Never put `close`/`fixes` adjacent to an issue
  number in a disclaimer** — write "must not be closed by this PR
  (issue 338)" or similar.
- **A gate that examined nothing reads exactly like one that passed.**
  Four instances: `verify-change-ownership` reporting "no paths changed"
  before the commit; `xtask ci` green while a new file was untracked;
  `| tail` printing `tail`'s exit code while the script aborted; and a
  merge guard that checked *checks* but not *mergeability* and silenced
  the merge output. `xtask ci` caught traceability drift **three
  times** — always run it after the last edit, never before.
- **`pgrep -f <pattern>` matches the polling command itself.** The
  runbook warns about it; I did it anyway in r14 and got a permanent
  false RUNNING. r15 and r16 key on log content and detect completion in
  one cycle.
- **A mutant that survives is the point of the exercise.** ADR-0038's
  second correction had **no coverage at all** — re-gating rule 3's
  membership half left 98 tests green. The missing fixture was written
  before proposal. Without the mutation pass, half that act would have
  shipped untested.
- **Workflows are good at breadth and at killing designs, unreliable on
  the single load-bearing claim.** Every real correction above came from
  re-running the claim by hand afterwards. Verify the decisive
  measurement yourself; treat agent figures as leads.
- **Sittings**: r14 (VMID 9437), r15 (9438), r16 (9439), all
  `5.15.0-186-generic`, no reboots, identical value sets, custody runs
  21–23 with three-way digest agreement. **r16 is the first sitting
  whose merged change altered protection behaviour, and the acceptances
  measured byte-for-byte what they measured before it.**
- **A guest may inherit a destroyed VM's IP**, producing a host-key
  change warning. Clear the stale `known_hosts` entry deliberately;
  the scripts run `StrictHostKeyChecking=no` and would proceed silently.
