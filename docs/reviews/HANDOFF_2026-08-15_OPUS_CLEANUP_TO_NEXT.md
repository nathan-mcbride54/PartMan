# Handoff — 2026-08-15, the cleanup sitting and the open-issue map

**From:** Claude (Opus 5), the session Nate directed with "review the
current state, clean up local commits/branches, sync main, remove the
stray folders, update the README and the handoff, and leave a clean
state to resume later."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-14_OPUS_ISSUE_354_RESOLVE_TO_NEXT.md`, which
was accurate at `b3de0cf` and is now four merges stale (§1).

> `docs/reviews` artifact. **This file is committed** — see §6.1, which
> corrects a claim the last four handoffs each repeated.

**This session wrote no product code and merged no pull request.** It is
a cleanup and record sitting. Everything below about the code is a
*reading* of main, not a change to it.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`6d743a3`**, synced with `origin/main` (0 ahead, 0 behind) |
| Spec | **13.0.0** (`AGENT_BUILD_SPEC.md`) |
| `cargo xtask ci` | **exit 0** — 604 annotations, 50 evidence rows, 85 requirements, 640 live tests, 10 generated documents |
| Local branches | **`main` only** |
| Local worktrees | **none** (the main checkout only) |
| Stashes | **none** |
| Open PRs | **none** |
| Working tree | clean apart from this handoff and the two review files §5 lists |
| Open issues | **12** — #319, #333, #347, #349, #353, #354, #356, #360, #365, #366, #370, #371 |
| **WP-020 acceptances** | **RE-OPENED. An r21 sitting is owed — see §2.** |

Issues **#318**, **#338**, **#348** and **#355** are now **closed**.
The last handoff listed #318 and #348 as open; both closed after it was
written.

---

## 1. What landed after the previous handoff (four merges it does not cover)

| PR | What |
| --- | --- |
| #368 | WP-035: restate L8's `wwid` result, which R2 corrected in L2 alone |
| #367 | WP-L100: the issue-318 Linux field-record sweep (comment-only `.rs`) |
| #369 | Governance: reserve ADR-0040 for issue #348's resolution |
| #372 | WP-010: **ADR-0040** — the relocation exemption is retired, and the release entry stands (issue #348) |
| #373 | WP-020 r20 re-pin at `6d4a8fc` |

#348 closed under **ADR-0040**, which retired ADR-0018:141-145's
relocation *exemption* as void where it stood (§0.2 rule 4 — an ADR may
not weaken a spec MUST) and split the grievance behind it into two new
issues, **#370** and **#371**.

---

## 2. The one thing that is owed: the r21 sitting

**This is the first thing to deal with, ahead of any issue work.**

`docs/work-packages/WP-020.md:1202-1205` states the stopping condition:

```
git diff --name-only 6d4a8fc HEAD
```

> must list Markdown files only.

**It does not.** It lists `crates/domain/src/model/protection_tests.rs`
— 48 lines PR #372 added. So WP-020's three acceptances (2e, 2h, 2j) are
re-opened and **an r21 sitting is owed on `6d743a3`**, after which
WP-020 re-pins there.

**How it slipped, which is the useful part.** #372's body *did* name its
sitting, correctly and before the merge, per the r15–r19 practice. It
named the pin it was working from as `86db930` (r19) and promised "one
sitting, taken on this PR's merge commit, followed by the WP-020 r20
re-pin." But the r20 sitting was taken on **`6d4a8fc`** — PR #367's
merge, the issue-318 record sweep — which landed *before* #372. So the
r20 pin discharged #367/#368's trip, #372's Rust change landed on top of
it, and #373 then re-pinned at `6d4a8fc` with #372 already past it. Two
correct-looking records, and the debt fell in the gap between them.

**The general shape, worth more than the instance:** naming a sitting in
a PR body binds the sitting to *that PR's merge commit*. When another
arc's sitting lands at a different commit in between, the promise is
silently transferred to a pin that does not cover the change. **Check
the stopping condition against `HEAD`, not against the pin the PR body
happened to cite.** The one-line check is the whole audit:

```bash
git diff --name-only $(grep -oP '(?<=git diff --name-only )\w+' docs/work-packages/WP-020.md | head -1) HEAD | grep -v '\.md$'
```

Note also that #372's change is **test-only**, which is exactly the
exemption r19 already declined once (#361, two fixture filenames) and
r20 declined again (comment-only `.rs`). The rule has now refused the
same invitation three times; do not accept it on the fourth.

Sitting mechanics are in the Proxmox runbook: `root@10.7.7.100`, scripts
in `/root`, **launch by absolute path** (the script sets `SELF` after
`cd`), and verify `cloud-init status --wait` plus free dpkg locks before
provisioning.

---

## 3. The open issues, and what I think about them

### 3.1 The chain that orders most of the work

The previous handoff established this by measurement, and nothing since
has moved it:

> **#347 → #360 → #354's kind half → #333's enforcement**

**#347 is the head of the queue for this whole family.** It reads like a
self-contained closure defect and is in fact the gate on three other
issues. If you are looking for the next thing to work, it is this — with
the caveat in §3.2 that its obvious fix has already been tried and
killed.

### 3.2 #347 — destroying a partition table reaches none of the partitions it releases

**Status: round 2's candidate was REJECTED.** Full record in
`ISSUE-347_RELEASE_ROUND_2_ADVERSARIAL_2026-08-14.md` (13 agents, every
FATAL handed to a separate refuting agent). Read it before proposing
anything — it refutes several of its own first-pass findings, and the
refutations are as valuable as the survivors.

**Do not re-derive the round-2 shape.** It gated the release on
`range_destroyed` membership, decided by one-byte `HostRange::intersects`
against `Facts.extents`. Three grounds survived refutation:

1. **Sibling capture with no extent inflation at all.** On a disk whose
   first partition starts at LBA 34 while the table declares the
   conventional `[0, 1 MiB)`, destroying that partition captures every
   sibling and everything under it. The entire committed fixture
   population has `table.start + table.length == p1.start` exactly, so
   558 tests stayed green over a population with no instance of the
   failing shape. **That is the #354 rejection ground verbatim.**
2. **The `conflicting-table-entry` half of the pair set is unjustified
   and uncovered** — deleting it left 558 passed, 0 failed.
3. **The offered ADR-0018 theorem amendment does not discharge
   ADR-0018:210-217**, which is a precondition of acceptance.

**The measured impossibility result is the most useful thing round 2
produced,** and any ADR that lands must state it: round 1 §11's two
requirements are **jointly unsatisfiable over `Facts.extents`**. Every
coverage-strength predicate is anti-monotone in the declared extent
(inflate by one byte and a refusal disappears — fail-open); the
intersection test is monotone but has no strength. You cannot repair a
predicate over `Facts.extents` into shape. **This is the argument for
deciding the release structurally.**

**The proposed round-3 direction** (reasoned, *not* measured): derive
the release from the **naming relation**, not the edge set and not the
extents. `NamingFields::Partition { parent_table }` and
`ConflictingTableEntry { table }` are on the one roster
`Topology::build` sweeps, so a partition cannot be represented without
naming its table — which closes the omitted-edge escape by construction,
is enumerable over `naming_referents` as the property test
ADR-0018:210-217 demands, and is a property of a delivered type rather
than a doc comment. Gate it on something structural about the *step*,
never on whether a declared range touches an authored extent.

**Before measuring any candidate in this family**, commit the
overlapping-geometry shape to the fixture population. The panel's own
`f11` and `f12` assertions already exist, pass at HEAD, and fail under
the rejected candidate; I preserved their source at
`ISSUE-347_ROUND_2_ADVERSARY_PROBE_2026-08-14.md` (§5) because the
worktree holding them is gone.

### 3.3 The rest, briefly, with my reading of each

- **#360** (pair table cannot express a partitioned mdraid). One row —
  `("volume","partition-table")` — suffices, and the workspace is
  646/646 with it. **It must not land yet**: the newly-representable
  population under-protects, and the cause is #347, pre-existing.
  Landing it first ships representation that builds and silently
  under-reaches. Blocked on #347 by measurement, not by preference.
- **#354** (naming-field referents validated by nobody). Resolve-only
  landed in #362; the **kind half** is open and blocked on #360. The
  only measured constructive path derives the set from
  `endpoint_pair_allowed`, which needs the pair table right first. A
  narrower four-pair candidate was proposed and killed (filed as #365);
  **do not re-derive it** — it false-refuses every host-backed virtual
  device.
- **#333** (reach closure misses children anchored outside the device's
  address space). Rule decided, **enforcement held**. The ADR-0037:217
  precondition question is answered: **not satisfied** — resolve-only
  never asks what a referent resolves *to*, so #333 is gated on #360.
  The single open hop for ADR-0037's derivation path is
  `PartitionTable.parent`.
- **#319** (absent child extents fail open). The occupancy half landed
  under ADR-0036. The **authorization half** remains. Its recorded
  blocker was #338, which is now **closed** by ADR-0039 — so this is
  worth re-measuring: it may be unblocked. Nobody has checked since
  #338 closed, and I did not. **Check before assuming either way.**
- **#356** (nothing cross-validates a containment edge against the
  extent facts). This is the issue round 2 kept walking into: a table's
  extent is never compared to its containment children, so an inflated
  body round-trips through decode. Fixing it would remove one of #347's
  escape routes rather than #347 itself.
- **#349** (the body boundary accepts zero-length, ghost-hosted and
  overflowing extents). Note the interaction: a `length: 0` extent
  removes the round-2 candidate's refusal outright. #349 is a
  precondition for trusting any extent-keyed predicate.
- **#353** (`canonical_ranges` writes the target's whole extent, which
  §2.1:110 forbids). Filed on the decision owner's instruction after
  ADR-0039. Self-contained; the most tractable of the twelve.
- **#365** (host-backed producing relation under-represented). A wrong
  doc comment, no committed fixture, and the suite blindness that let
  #354's kind candidate through. Small, and it buys a fixture the next
  kind-check attempt will need.
- **#366** (WP-035 transport-discrimination deferral addresses the IPC
  route decision, so its real consumer will never pick it up). WP-035,
  outside the domain family — parallelizable with any of the above.
- **#370** (a byte-preserving relocation of a protected structure
  refuses; relief needs a preservation proof not computable today) and
  **#371** (PART-005's hosted-signature duty is undelivered and has no
  plan vehicle). Both split from #348 by ADR-0040. #371 is the starker
  one: an exhaustive grep for the duty's terms returns **exactly one
  hit — the spec sentence stating it**. No type, no field, no test, no
  plan-body item.

### 3.4 If you want my recommendation

Take the **r21 sitting first** (§2) — it is owed, it is mechanical, and
leaving it open makes every later "the gate is green" claim weaker.

Then, for issue work, I would **not** start at #347 despite it heading
the chain. Round 2 established that the whole extent-keyed family is
dead and that the structural replacement needs a fixture population that
does not exist yet. The cheapest real progress is **#349 plus #356** —
both are about the extent facts being unvalidated, both are
preconditions for any predicate that reads them, and between them they
build exactly the overlapping-geometry fixtures §3.2 says to commit
before measuring a #347 candidate. #347 round 3 then starts from a
population that can see its own defect, instead of a green suite that
is structurally blind.

If you want a self-contained win rather than a chain, take **#353**.

---

## 4. What this session cleaned up

- **Local branches: 45 → 1.** 36 were fully merged into `main`. Nine
  (`adv-*`, `adv/*`) were workflow worktree branches all pointing at the
  same commit `6e1706b` — round 2's **rejected** #347 candidate. Its
  design, its measurements and its rejection are in the round record, so
  the branches were deleted rather than kept as a trap for whoever finds
  them next.
- **Worktrees: 16 → 1.** Fourteen under `.claude/worktrees` plus two in
  other sessions' scratchpads, all removed and pruned.
- **Remote branches: 162 → 4.** 159 were fully merged into `main` and
  were deleted with Nate's explicit approval. **Three unmerged branches
  were deliberately kept**: `codex/wp-030-desktop-shell-inc2-v2`,
  `codex/wp-030-slint-feasibility`, `work/wp-000-ci-minutes`. They carry
  commits not in `main`; nobody has established they are dead, so they
  were not touched.
- **One stash dropped** (`759ff3a`) — the sibling session's #319 planner
  draft. **Superseded and rejected:** it keyed on extent *presence* and
  refused outright on an unlocated table, which are the two measured
  fatals ADR-0036 recorded when it killed that design. Main carries the
  landed replacement (`SchemeReservation`, `reserved_regions`).
- **Five stray checkouts removed** — `D:\pm-354b`, `pm-354base`,
  `pm354min`, `pm-354o`, `pm-354-opus`, **7.1 GB**. All were remote-less
  copies based on the superseded `7fdba38`; #354's resolve-only half
  landed in #362 and its kind half was rejected and filed as #365.

---

## 5. Two review files this session committed, and why

1. **`ISSUE-347_ROUND_2_ADVERSARY_PROBE_2026-08-14.md`** — the verbatim
   source of the round-2 adversarial probe (894 lines), including `f11`
   and `f12`. The panel closed by instructing that this geometry be
   committed to the fixture population **before any candidate in the
   family is measured again**, and the only copy was untracked in a
   workflow worktree this session was about to delete. It is a probe,
   not delivered test code: `//! Not for merge`, no annotations.
   Landing any of it is WP-010 work with its own
   `Requirements:`/`Evidence:` blocks.
2. **This handoff.**

The other 62 previously-untracked `docs/reviews` files were committed
unchanged — see §6.1.

Two documents were also corrected:

- **`README.md`** — a new **Open issues** section (§3's table, plus the
  r21 obligation), and the WP-020 status row now records the sixteenth
  trip rather than ending at the r20 re-pin.
- **`docs/quality/test-tiers.md`** — cited spec **12.14.0** against a
  spec at **13.0.0**. This is the *same drift, in the same file*, that
  the 2026-08-14 handoff recorded correcting once already (it was
  12.12.0 against 12.14.0 then). ADR-0039 took the spec to 13.0.0 and
  nothing swept the documents naming the version. **A file that has
  drifted once will drift again**; the sweep is
  `grep -rn '<old-version>' --include='*.md' .` on every spec bump,
  reading past the CHANGELOG and ADR rows, which cite versions
  historically and are correct as they stand.

---

## 6. Corrections to the record

### 6.1 "Never commit `docs/reviews`" is wrong, and it cost the record 63 files

Every handoff since 2026-08-03 carries some form of this banner:

> Untracked local artifact, docs/reviews convention: never stage into a
> commit; `verify-change-ownership` refuses it.

**The gate does not refuse it.** `docs/work-packages/WP-000.md:43`
lists `docs/reviews/**` in WP-000's `owned-paths` block. A commit
trailered `Work-Package: WP-000` touching only those paths passes; this
commit is the demonstration.

What the gate actually refuses is `docs/reviews` bundled into a
*code* change under a different work package — which is true, is
presumably where the belief started, and is a different rule.

**What it cost.** 63 review documents sat untracked, and tracked files
in `main` cited them: `docs/work-packages/WP-010.md` alone references
`SI-11`, `SI-18`, `SI-19`, `SI-27`, `SI-33`, `SI-34` and more, none of
which existed in the repository. **Main contained dangling citations to
its own reasoning.** They are now committed and the citations resolve.

The narrower true rule, which is what the banner should have said:
**session records go in their own WP-000 commit, never bundled with
code.**

### 6.2 The previous handoff's issue list is stale

It listed #318 and #348 as open; both are closed. Its §5 remains
accurate on every issue it discusses.

---

## 7. Operational notes

- **A shell `cd` into a worktree persists across tool calls.** Half an
  hour of this session's reading was done against `6e1706b` — the
  rejected candidate — instead of `main`, because an earlier inspection
  `cd`'d into `.claude/worktrees/wf_f2620bc0-76d-1` and never came back.
  The reading looked coherent the whole time: a "14 commits behind"
  fast-forward followed by a diffstat showing three files. **Print
  `git rev-parse --show-toplevel` in any command whose answer depends on
  which checkout you are in.**
- **`| tail -3` on a diffstat is a false-green of its own.** It showed
  the three files of the wrong branch's diff and read exactly like a
  complete answer.
- **`git branch --merged` is answered against the branch you are on.**
  Fast-forward `main` *before* classifying branches, or the merged set
  is computed against a stale base and under-reports.
- Remove worktrees before deleting their branches; `git branch -d`
  refuses a branch checked out in one, and the refusal is easy to read
  as "this branch has unmerged work."
- The gate figures in §0 were taken with `cargo xtask ci`'s exit code
  captured on the command itself, not through a pipe.
