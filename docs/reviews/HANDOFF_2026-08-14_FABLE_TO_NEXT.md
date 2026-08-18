# Handoff — 2026-08-14, the #319 rejection arc and the verdict-multiplicity fix

**From:** Claude (Fable 5), the session Nate directed with "review the
current project progress and determine what the most valuable next steps
are", then step by step through the #319 arm, its adversarial round, the
defect that round exposed in committed code, an r18 sitting, and two
further rejected designs.
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-14_OPUS_ISSUE_338_REACH_TO_NEXT.md`. The
rounds this session wrote are
`ISSUE-319_INDETERMINATE_ARM_ROUND_2026-08-14.md` (candidate
**rejected** — read its header note),
`ISSUE-347_TABLE_RELEASE_ROUND_2026-08-14.md` (candidate **rejected** —
same), and `VALIDATION_ACT_SCOPE_2026-08-14.md` (recommendation
**withdrawn** — same).

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at `7fdba38`, **spec 13.0.0** (unchanged this session — nothing
here needed a spec bump). WP-020 re-pinned at **`c9cd4bb`** after the r18
sitting; `git diff --name-only c9cd4bb HEAD` must list Markdown only, and
does. Working tree clean apart from untracked docs/reviews.

Open issues: **#318**, **#319**, **#333** (decided, enforcement held),
**#347**, **#348**, **#349**, **#353**, **#354**, **#356**. **#355 is
closed by PR #357.** Nothing is in flight: no branch, no open PR, no VM.

## 1. What this session did

| PR | What |
| --- | --- |
| #357 | WP-010: the verdict computation folds over every edge, not the first (issue #355) |
| #358 | WP-020 r18 re-pin at `c9cd4bb` (VMID 9441, custody run 25) |
| #359 | WP-020: the r18 sweep missed the reproducibility count |

Issues filed: **#354** (naming referents validated by nobody), **#355**
(the verdict-steering bypass — fixed and closed), **#356** (containment
edges never cross-validated against extent facts). Comments added to
**#319** recording two further measured shapes of its defect, and to
**#356** correcting one of its own controls.

## 2. The one thing worth carrying above all others

**Four designs were proposed this session and all four were rejected.
Every one was green on the full workspace when proposed.** That is now
the fifth, sixth, seventh and eighth such rejection in this area across
four sessions. The rejections:

1. **#319's fail-closed `Indeterminate` arm** — never ran at
   `PlanStep::mutating_declared`, the sole step constructor, so it
   guarded only the advisory surface and broke CAP-005 agreement in the
   direction where the backstop is weaker than the advice.
2. **#347's whole-destruction release** — the predicate asked whether
   **one** declared range covers the extent, so the same destroyed bytes
   split into two adjacent ranges defeat it entirely; and it was
   **anti-monotone in the node's own declared extent**, so one byte of
   inflation removed the refusal.
3. **The validation-act recommendation** — its central claim (a
   validated body "changes the odds for the next closure predicate") was
   checked against the records and would have prevented **at most one of
   five** past deaths; its frame rule as written would have *removed*
   reach from loop-file stacks.
4. (Earlier in the arc, by the same standard: the point-reach candidate
   from the prior session's round.)

**The test that keeps working**, and it is not the suite: *can any
authored field — a number, or the way a set is partitioned — remove or
weaken the refusal this adds?* Every one of the four failed that
question, and none failed `cargo test`.

## 3. The defect that came out of it, and it was in committed code

The #319 adversarial round chased a decoy-parent trick one step further
and landed on HEAD. **Three arms of `node_verdict` selected one edge with
`.find()`** where a body may present several — a signature's consumer, a
node's producer, and the containment ascent to the device whose scope arm
is inherited — with the winner decided by `NodeId` sort order, a digest
over grindable hashed fields.

Measured at HEAD: a file system on a `RecognizedRemote` device went
`Unsupported{InheritedDeviceScope}` → **`Clear`** behind one added
containment edge; a volume produced by a live ZFS pool went `Unsupported`
→ **`Clear` on all ten mutating operations** behind one added
`Production` edge. PR #357 folds `worst` over every matching edge — the
module's own combinator — so an added edge can only ever add refusal, and
single-ancestry bodies answer exactly as before (no committed test
moved).

**Deliberately not taken there:** forbidding the multiplicity at
construction. That is a decode-boundary rule with its own MODEL-003 debt,
and MODEL-002 gives membership unbounded in-degree on purpose. It remains
the decision owner's.

## 4. The ordering finding, verified by hand, which settles a question I had wrong

I wrote a scoping document arguing #333's enforcement and a
body-validation act should be one act, on the premise that "nobody has
stated the question in one place". **That premise is false.**

**ADR-0037 already orders them** (`docs/adr/0037-containment-frame-anchoring.md:146-150`):

> "**Owed before any enforcement:** a capture-side referent sweep.
> `Topology::build` validates edge referents and endpoint pairs but
> **nothing validates naming-field referents**, so a naming-derived frame
> can be computed from a pairing the pair table forbids."

and `:217` makes "the capture-side referent sweep exists" a verification
condition for enforcement, **with the golden vector regenerated in the
same act**. So the order is **#354 → #333's enforcement**, and the
enforcement act carries the golden vector and its MODEL-003 discharge.

**This makes #354 the clear next step**, and it is not a new policy
question — it discharges an obligation an accepted ADR already recorded.

## 5. What remains open

1. **#354 — the referent sweep, now split in two by a measured finding.**
   A four-design judge panel ran; its record is
   `ISSUE-354_REFERENT_SWEEP_PANEL_2026-08-14.md`. The winning design
   derives its kind check from `endpoint_pair_allowed`, which is the
   right idea — no second authored list to drift — but **I verified by
   hand that it refuses three honest layouts that build today**: a GPT
   inside a LUKS volume, a partitioned mdraid array, and an xfs on a
   dm-multipath node. It imports the pair table's incompleteness into a
   mandatory field.
   - **Land resolve-only now**; it has no honest-body cost and is a
     genuine partial discharge, but **must not be described as closing
     #354**, since ADR-0037's stated harm is the forbidden *pairing*.
   - **Hold the kind half** behind **#360**.
   Note what this says about panels: two judges scored the winner 8 and
   7.5 and neither tried a real-world layout, while the finding that
   killed it was already in the transcript attached to a *different*
   design. A panel is only as good as the shapes its judges think to try.
2. **#360 — the pair table cannot express a partitioned mdraid array.**
   Filed this session. `aggregate → partition-table` and
   `volume → partition-table` are both absent, so there is no path from
   `md0` to `md0p1`, and none to a partition table inside any mapped
   volume. It is a MODEL-002 question, it blocks #354's kind half, and
   adding containment rows triggers ADR-0018:210-217's re-proof
   obligation.
2. **#347.** The defect is real and re-measured on current main. Its
   round records what the next design must satisfy: union semantics over
   declared ranges, monotonicity in declared extents, and the re-proof
   obligation ADR-0018:210-217 makes a precondition of acceptance. Two
   findings in that round survive the rejection and narrow the issue:
   **the defect requires the table to carry an extent** (an extent-less
   table already releases correctly), and **`Containment` is documented
   as byte-nesting but `table → partition` is not nested at all**.
3. **#319's authorization half**, still open, now with **three** measured
   shapes rather than one — all three approve destructive work over a
   live pool at HEAD.
4. **#356**, with its control corrected: the absent-extent spelling
   reaches the same approval as the contradiction, so it overlaps #319's
   class rather than being cleanly separable. #347's candidate closed
   #356's contradiction body, which argues the two are one act.
5. **#333's enforcement**, still held, now with its precondition named.
6. **#348**, **#349**, **#353**, **#318** untouched.

## 6. Operational notes, and three are new

- **`gh pr checks --watch` exits 0 having watched only the checks
  registered when it launched.** On PR #359 it returned success with
  **1 of 12 passed and the rest pending**. The watcher is what you reach
  for to avoid the registration race, and it has the race. **Never merge
  on its exit code**: query directly and assert a pass count equal to the
  expected number *and* `mergeStateStatus=CLEAN`. `BLOCKED` with
  everything green means checks are still registering.
- **The inverse also bit**: `gh pr merge` exited **1** on PR #357 while
  the merge had already **succeeded** remotely — the failure was local,
  `gh` unable to switch branches because `main` is checked out in another
  worktree. Check `state`/`mergedAt` before believing either outcome.
- **`| tail` still eats exit codes.** `cargo xtask traceability | tail`
  reported `EXIT: 0` while cargo had exited 1. Capture real exit codes
  directly for every gate.
- **A sweep keyed on the previous commit message's vocabulary misses.**
  The r18 sweep searched "twenty passes"/"seventeen guests" and missed
  "passed twenty **times** across seventeen **disposable** guests" —
  three paragraphs above that section's own warning about exactly that
  failure. Sweep for the **numbers**, not for a prior summary's phrasing.
- **Print-only probes cannot kill mutants.** The #347 probes printed
  rather than asserted; a mutation pass over them would have measured
  nothing. Write the assertions first.
- **A false-refusal control on a fixture with nothing protected is
  vacuous.** "Ordinary disk, 10/10 Clear" cannot fail under any widening,
  however extreme, because every node in it is already `Permitted`.
- **"Closes issue #NNN" does not auto-close.** GitHub needs the number
  adjacent to the keyword. The phrasing that prevents accidental closes
  also prevents intended ones — close explicitly.
- The r18 guest **inherited a destroyed VM's IP** and the stale host key
  was cleared deliberately; the scripts run `StrictHostKeyChecking=no`
  and would have proceeded silently.
- **A sitting lapse, recorded:** r15, r16 and r17 each named their
  sitting in the tripping PR's body **before** the merge. #357 did not.
  The sitting ran the same day, and both the WP-020 record and
  `test-tiers.md` say so.

## 7. Session evidence

The scratchpad for this session holds the probe files, gate tables,
findings and verdict JSON from both adversarial rounds, and the r18
transcript. The worktrees created during it (`pm-fix-355`, `pm-347`,
`pm-wt-319`, `pm-r18`) have been removed; `pm-354` remains for the
in-flight #354 work.
