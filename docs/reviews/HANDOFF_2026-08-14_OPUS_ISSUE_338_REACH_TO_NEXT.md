# Handoff — 2026-08-14, issue #338's held half, then #319's

**From:** Claude (Opus 5), the session Nate directed with "start on
#338's remaining half", then "go with U, major bump, and file the three
as separate issues", then "start on 319's authorization half". The file
name says #338 because that is where it began; **section 5 is #319 and
it is the live work.**
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-14_OPUS_ISSUE_ARC_TO_NEXT.md`. Two rounds
were written here: `ISSUE-338_REACH_ROUND_2026-08-14.md` — read its
header note first, as the predicate in its body is **not** the one that
shipped — and `ISSUE-319_AUTHORIZATION_ROUND_2026-08-14.md`, whose
candidate is **rejected**, with the grounds recorded in it.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at `8e03e68` (#352), **spec 13.0.0**, pinned at `b9d1ba2` —
`git diff --name-only b9d1ba2 HEAD` lists Markdown only. Working tree
clean apart from untracked docs/reviews. Open
issues: **#318**, **#319** (authorization half — unblocked, grounded,
one candidate rejected; section 6), **#333** (decided, enforcement
held), and the four filed today — **#347**, **#348**, **#349**,
**#353**. Issue **#338** is closed by #351. **Nothing is in flight:**
no branch, no open PR, no VM.

## 1. What this session did

| PR | What |
| --- | --- |
| #350 | Reserve ADR-0039 |
| #351 | spec-change 13.0.0 — carried-content reach, and a bounded descent |
| #352 | WP-020 r17 re-pin at `b9d1ba2` (VMID 9440, custody run 24) |

Issues filed: **#347** (a destroyed table reaches none of the partitions
it releases), **#348** (ADR-0018's relocation exemption against
ADR-0038's `Move` entry), **#349** (the body boundary accepts
zero-length, ghost-hosted and overflowing extents, and `assemble` skips
its checks entirely).

## 2. The one thing worth carrying forward above all others

**Four predicates were rejected, and every one of them was green on the
full workspace when it was proposed.** Two were killed by an adversarial
pass, two more during implementation, each by a fixture that did not
exist until someone went looking. The failures were not subtle in
hindsight:

1. bounding descent against the source's own extent **lost reach HEAD
   has** (an extentless producer could not reach its product);
2. bounding against any destroyed node's extent made protection a
   function of `extent_host`, which nothing authenticates — moving one
   field, node ids and body hash unchanged, turned a live-pool refusal
   into `Clear`;
3. no self-frame clause **false-refused ordinary disks** — a partition
   delete blocking on a stale end-anchored mdraid superblock;
4. admitting extentless containment children captured a sibling that
   merely lacks a fact.

The rule that came out of it, and the one to keep: **a bound on reach
must never be able to remove reach.** Extents are authored body content;
a predicate that can subtract reach hands that content a lever. The
acceptance test that catches this is not "the suite is green" — all four
were — but **"is the new closure a superset of the committed one on the
attackers' own fixtures"**.

## 3. Also true, and cheap to forget

- **A surviving mutant is the point.** Six mutations; the sixth survived
  against 104 green tests, meaning the `may_carry_extent` clause had no
  coverage at all. Same shape as ADR-0038's second correction, one ADR
  later. Run the mutations; write the fixture the survivor names.
- **The measurement that mattered was at `mutating_declared`, not at
  `affected_set`.** The gate had already been corrected by ADR-0038; the
  live under-refusal was at plan-body re-validation, where no capability
  gate sits. When a closure defect is filed, measure the boundary with
  the fewest layers in front of it.
- **`git`-tracked line endings.** Python's `open(f,'w')` on Windows
  writes CRLF, and `rustfmt.toml` sets `newline_style = "Unix"`; the gate
  fails with "Incorrect newline style" and no line number. Write bytes,
  or set `newline=''`.
- **The `Governance:` trailer must be in the message's last paragraph**,
  beside `Co-Authored-By:`, or `git interpret-trailers` does not see it
  and the ownership check refuses the commit.
- **Backwards sweep, again.** Three spec-version pointers were stale
  (README two bumps, CONTRIBUTING and the PR template three), and
  README's WP-060 row still said #338's halves were held. Only grepping
  for what the *new* number contradicts finds these.

## 4. What remains open

1. ~~WP-020's stopping condition.~~ **Re-pinned in #352.** The r17
   sitting (VMID 9440, 2026-08-14 UTC, kernel 5.15.0-186-generic, no
   reboot) re-took all three acceptances on `b9d1ba2` with value sets
   identical to r16, eleven negative controls refused, custody run 24
   agreeing across guest, host and workstation, teardown verified at
   2026-08-14T14:40:46Z. Worth carrying: the acceptances cover the
   descriptor-bound attachment path, **not** the closure, so their
   agreement across a protection change is what correctness looks like
   and never evidence the change was inert.
2. ~~The `canonical_ranges` table-write over-claim.~~ **Filed as #353**
   on the owner's instruction after ADR-0039 landed, with the ordering
   constraint and one measurement the ADR did not have: correcting the
   entry to §2.1's own words moves **six of the ten whole-disk ZFS gates
   from `Unsupported{Zfs}` to `Clear`**, over a live pool, **with the
   entire committed suite green (105 passed)**. The whole-disk gates
   depend on the violation, and nothing committed observes the loss.
   ADR-0039's carried-content reach does not cover it: the target is a
   physical device, and the closure refuses descent out of a self-framed
   extent by design.
3. **#319's authorization half** — unblocked by #338's closure, then
   worked far enough to reject one design. Section 5 has the state.
4. **#333's enforcement**, still held; the golden vector and
   `plan_tests.rs` remain unlawful under its rule until the enforcement
   PR regenerates them.
5. ~~The `protection.rs:28-29` citation.~~ **Discharged in #351** — it
   now cites `EdgeKind::Containment`'s own words and ADR-0037's
   anchoring rule, which is what ADR-0037:180-185 asked of whichever PR
   next touched that file. Written into this list first, and paid before
   the arc closed, because a debt recorded twice is a debt nobody pays.

## 5. Issue #319's authorization half — where it actually stands

**Read `ISSUE-319_AUTHORIZATION_ROUND_2026-08-14.md` before anything
else here.** Three things changed the problem.

1. **The previous round's blocker is gone, and not by anyone's
   intention.** Its finding 5 — a partition-framed ZFS label defeating
   the flagship destructive refusal with every extent present — is
   **fixed by ADR-0039's `child.host == source` clause**. Every
   domain-side route in that round died on finding 5. Measured both
   ways: partition-framed and device-framed now refuse alike.
2. **The defect is narrower and worse than filed.** Absence alone no
   longer hides a node; absence *of a frame* does. Content expressed in
   a node's own address space (lawful — ADR-0037's enforcement is held)
   plus that node carrying no extent (lawful — nothing requires one)
   makes the whole subtree invisible: a **whole-device wipe over a live
   ZFS vdev is approved**, `affected = 3`. The body assembles, encodes,
   decodes, recomputes, and the **decoded** snapshot's own closure gives
   the same answer. The two shapes the issue leads with — a create and a
   destructive step over a buried member — **both refuse today**.
3. **The obvious candidate is rejected, on four fatals.** It placed an
   extent-less node by its own hashed name. Do not rebuild it:
   - **The offsets are not byte positions.** `primary_offset` and
     `superblock_offset` document only "the offset the parser fixed".
     The counter-example is committed in **PR #351's own guard** — an
     end-anchored mdraid 0.90 superblock named `primary_offset: 0` with
     its extent at `1 GiB − 64 KiB`. Only `Partition.start_offset`
     states its address space.
   - **One field evades it.** No layer validates a naming referent;
     `Topology::build` checks edge endpoints only. A `parent_table`
     naming an absent or wrong-kind node returns no position and the
     hole reopens on a body that decodes cleanly.
   - **A point is not a span.** Only ranges covering the node's first
     byte reach it, so tail destroys, sibling operations and interior
     creates still construct — including ADR-0039's own worked vector.
   - **It re-derives what ADR-0039 rejected**, from the seeding scan
     rather than the descent, and the committed sibling guard then
     survives on a one-byte margin.

**What survives, and it is the useful part.**

- **Derived position is unavailable in general**, which retires the
  whole family rather than one member of it.
- **Nothing validates a naming field's referent.** The planner guards
  this (`OccupancyGround::TableIsNotThisHosts`); the domain does not, so
  any name-derived authorization gate is strictly weaker than the
  solver's own check. **Worth filing on its own**; it was not filed,
  deliberately, because the decision owner had not seen it yet.
- **The remaining arm is the one the issue itself proposed:** refuse
  rather than reach — an extent-bearing node declaring no bytes in the
  step's frame makes the answer unsound, so the step is `Indeterminate`.
  It needs no geometry, so none of the first three fatals reach it.
  **It is unmeasured.** Its cost is availability, and the 2026-08-13
  round's objection 4 already named the shape: a device's self-extent
  spans everything, so one unlocated occupant can block every
  destructive step on that disk.

**Three questions were put to the decision owner and none is answered:**
whether that availability cost is acceptable in principle before
measurement is spent on it; whether the unvalidated-referent gap is
filed and fixed first; and whether **#353 lands before either**, since
`canonical_ranges` claiming a device target's whole extent inflates the
measured cost of every option in this family.

## 6. Operational notes

- The adversarial workflow ran 31 agents. Its verify phase refuted 16 of
  29 findings, which is the phase earning its keep — but **the two
  fatals it kept were both real and both reproduced by hand**. Treat
  agent figures as leads; re-run the decisive one yourself. Every
  correction in this session came from re-running a claim, never from
  the report that raised it.
- `gh pr checks` immediately after a push reports only the checks that
  have registered. Wait, or watch until nothing is `pending`.
- **`jq` is not installed on this workstation.** Two check-watchers
  written with it would have looped silently to their timeout, and the
  silence reads exactly like "still running" — the gate-that-examined-
  nothing failure, in the tooling built to watch the gates. Write
  watchers that emit on the failure path too, and confirm they emit at
  all before trusting their quiet.
- Repo auto-merge is disabled, so a PR cannot be armed to merge itself;
  a watcher that merges on green and reports on red is the substitute.
- The session's four rejected #338 predicates and the rejected #319
  candidate share one property: **each was green on the full workspace
  when proposed.** For closure work the suite is not evidence. The
  acceptance test that keeps working is *is the new closure a superset
  of the committed one on the attackers' own fixtures* — and for #319,
  the test the candidate failed was simply *is the premise true*, which
  a two-minute read of a committed fixture answered.
