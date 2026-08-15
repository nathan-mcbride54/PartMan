# Handoff — 2026-08-11, the evening session (review fixes → 2i → 2j)

**From:** Claude (Fable 5), working with Nate through the evening of
2026-08-11.
**To:** whoever picks this up next.
**Follows and supersedes:** `HANDOFF_2026-08-11_FABLE_R3_TO_NEXT.md`, which
this session wrote mid-way and then outgrew twice; its content is folded in
here. The morning documents (`HANDOFF_2026-08-11_OPUS_TO_NEXT.md` and
earlier) stand as history; start here.

> **Untracked local handoff artifact.** `docs/reviews/**` stays untracked by
> convention; never stage it into a WP-020 commit — `verify-change-ownership`
> refuses it, and `git add -A` has swept it in before (see the Opus handoff
> §5.4).

## 0. Repository state

`main` at `d02a902`, spec 11.1.0. Working tree clean apart from the
untracked `docs/reviews/**` set (19 files including this one). No open pull
requests. Issues #248, #249, #250 closed. The 2e stopping condition is
pinned at `39b59f5` and holds: commits after it are Markdown-only. Any
non-Markdown merge trips it — it has now tripped six times — and the price
of a code change is a VM sitting (about an hour with the runbook, most of it
the guest's warm build).

## 1. What this session did

Three rounds, eight merged PRs, three VM sittings (VMIDs 9426–9428, all
destroyed and verified), each sitting re-pinning the stopping condition.

**Round 1 — the 2h adversarial-review findings, all three discharged.**
| PR | What |
| --- | --- |
| [#251](https://github.com/nathan-mcbride54/PartMan/pull/251) | #249: `Admission::admit` counts verified handles per fixture (multiset, not set); `unwrap_or_default` became an explicit refusal. Was the named prerequisite for increment 2's remaining scope. |
| [#252](https://github.com/nathan-mcbride54/PartMan/pull/252) | #248: the rebind probe re-reads `LOOP_GET_STATUS64` on `EINVAL`; only an attachment observed read-write names `KernelRefused`. Pure classifier, every arm pinned. |
| [#253](https://github.com/nathan-mcbride54/PartMan/pull/253) | #250: the contracted write moved into `write_contracted_range` and a Tier-1 test measures where it lands (the issue's named fallback). |
| [#254](https://github.com/nathan-mcbride54/PartMan/pull/254) | The r3 sitting (VMID 9426, `68298f2`): both acceptances re-taken on the fixed probe/write path. |

**Round 2 — increment 2i, the general destructive executor. Delivered.**
| PR | What |
| --- | --- |
| [#255](https://github.com/nathan-mcbride54/PartMan/pull/255) | The executor runs the registry's full contract shape: N fixtures, N non-overlapping ranges, a pre-flight hashing every fixture before any is attached, per-fixture self-contained chains. Registered nothing; the containment pin proves the general protocol produces exactly the 2h-recorded sequence on the 1×1 shape. Plan: `WP-020_INCREMENT_2I_PLAN_2026-08-11.md`. |
| [#256](https://github.com/nathan-mcbride54/PartMan/pull/256) | The r4 sitting (VMID 9427, `0625b07`): both acceptances re-taken through the general executor. |

**Round 3 — increment 2j, the two-range suite, on the operator's direction.
Delivered, and increment 2 itself delivered as scoped.**
| PR | What |
| --- | --- |
| [#257](https://github.com/nathan-mcbride54/PartMan/pull/257) | `gpt-basic-512-both-signatures-erase` registered: primary GPT signature at 512 and backup at 4,193,792 (measured on the generated image before the contract was written), eight zeros each. Both edit-detectors flipped, every generic-refusal test re-read (re-readings on the PR), the shipped two-range shape reduced through the real executor path at Tier 1, and the stale AGENTS/CONTRIBUTING "only runnable higher-tier acceptance" sentences repaired. |
| [#258](https://github.com/nathan-mcbride54/PartMan/pull/258) | The r5 sitting (VMID 9428, `39b59f5`): eleven controls refused, 2e passed (fifth re-take), 2h suite passed (fourth), and the 2j suite passed **on its first take** — the first real-kernel run of the multi-range chain (`ranges_written=2`, `contracted_bytes_written=16`, both signatures restored). 2j and increment 2 moved to Delivered; re-pin at `39b59f5`. |

Every fix and every new gate was mutation-verified before proposal, and each
mutant failed a named test (five mutants for 2i alone; the drifted-offset
mutant for 2j fails both shape pins).

## 2. What "increment 2 delivered" means, exactly

The row states it and the records repeat it: a Tier-2 destructive suite can
exist, two do, and each writes exactly its declared byte ranges of one
generated fixture and nothing else, under every SAFE-007 factor, in a
disposable VM, with an operator in the trigger path. It does **not** mean
the product can write — no product write path, no storage-tool invocation,
no domain types, no plan or hash surfaces. Every generic destructive Tier-2
request and every Tier-3 request still refuses, because a generic request
selects no suite. The multi-fixture half of the general shape is
Tier-1-proven only (fake-driven pre-flight and sequencing tests) and stays
that way until a contract needs a second fixture — recorded in the 2j
boundary, not an oversight.

## 3. Corrections this session made to earlier statements

- **Issue #249's `./` premise was wrong.** Rust `Path` equality normalizes
  `.` components away, so the interlock's supplied-path dedup catches that
  spelling; `..` components compare literally and are how the
  duplicate-handle shape is really built. Recorded in the test, changelog,
  and PR #251.
- **The 2e reproducibility sentence was stale** ("five times across two
  guests" against a seven-row custody table). Fixed with the correction
  noted in place; the count now reads nine passes across six guests and the
  custody table has eleven rows including two retained void runs.
- **AGENTS.md, CONTRIBUTING.md, and README all still claimed a single
  runnable higher-tier acceptance**, stale since the SI-35 selector and the
  2h suite. All three repaired (#257 for the shared status documents,
  #258 for README's copy).
- **WP-020's increment-2 row still said the registry was empty**, stale
  since 2h. Corrected in the 2i round with the staleness named in the row.

## 4. Where to pick up

Nothing is half-done. The live threads:

1. **SI-18 — still severity-1, still gating WP-040's authorization
   vocabulary.** (Correction, 2026-08-11, a later session: this item
   originally also listed the SI-39 option (c) recommendation, the
   decision briefs, and the ADR-0014 fork as untouched decision threads.
   All three concluded on 2026-08-08 — ADR-0015, Accepted, resolved
   SI-39 in spec 7.0.0, and ADR-0014, Accepted, fixed the axis the
   briefs' standing instruction gated, with SI-35 resolved in 8.0.0 —
   so the line was stale when written, carried forward from the
   2026-08-08 handoff. SI-18 was then the only live decision thread —
   and later on 2026-08-11 it too resolved: spec 11.2.0 by ADR-0021,
   Accepted by delegation, PRs #259 (reservation) and #260 (resolution),
   recommendation round `SI-18_RECOMMENDATION_ROUND_2026-08-11.md`.
   Authorization is a two-tier ladder, SAFE-002 untouched, no
   plan-carried authorization-requirement field, so WP-040's
   authorization vocabulary unlocks with no jointly-sequenced schema
   change. The WP-040 re-attribution then landed as #261 under WP-040's
   own assignment: the assignment, `schemas/rpc/authentication.md`, and
   README's row now cite the recorded decision, not the retired
   question. One named debt remains: three `crates/rpc` doc comments
   still say "SI-18 holds" — left because any non-Markdown merge
   re-opens the three VM acceptances pinned at `39b59f5`, recorded in
   #261's CHANGELOG entry, riding the next Rust change to the crate.
   Later the same day SI-19 followed the same full arc: recommendation
   round `SI-19_RECOMMENDATION_ROUND_2026-08-11.md`, accepted by
   delegation, ADR-0022 in spec 12.0.0 (major — PLAN-008 and Section
   6's reversal body item both change meaning) via PRs #262/#263, and
   the WP-060 re-attribution as #264. The reversal is an ordinary
   draft bound at its own validation, linked by reference —
   `OperationPlan` is not recursive — so WP-060's PLAN-008 reversal
   increment is startable, and that increment is also where the two
   `crates/planner` SI-19 doc comments get repaired (same
   Markdown-only reasoning, recorded in #264). SI-15 then followed the
   same arc, also on 2026-08-11: recommendation round
   `SI-15_RECOMMENDATION_ROUND_2026-08-11.md`, accepted by delegation,
   ADR-0023 in spec 12.1.0 (minor — PART-009's sentences stand
   verbatim, the authored/inherited scoping is additions) via PRs
   #265/#266, and the WP-060 re-attribution as #267. A PART-009
   deviation is authored, not inherited: the misaligned grow-at-tail
   case proceeds, and the solver's delivered refusal stands in code —
   deliberately, it is behavior for a reviewed increment implementing
   the ADR's fixtures, not a comment sweep — citing a decision rather
   than an open question until that unlock increment lands. SI-16 then
   completed the same arc, also 2026-08-11: recommendation round
   `SI-16_RECOMMENDATION_ROUND_2026-08-11.md`, accepted by delegation,
   ADR-0024 in spec 12.2.0 (minor — PART-013's sentence verbatim, the
   state-selected arms additions) via PRs #268/#269, re-attribution as
   #270. PART-013 discharges by the helper's authored table state:
   parse-level backup on Present, a journaled determination on Absent
   (no acknowledgement), a verified raw capture of the write-target
   regions for the typed REC-001 repair family on Indeterminate, and
   capture-impossible refusing except under a plan-creation
   acknowledgement naming the regions; the protection record's journal
   encoding is jointly sequenced with JRN-006 (WP-070). Three WP-060
   increments are now startable under recorded decisions (reversal,
   solver unlock, backup family), and the planner's SI-19/SI-15/SI-16
   comment-and-refusal debts all ride whichever Rust change comes
   first. SI-17 then completed the same arc, also 2026-08-11:
   recommendation round `SI-17_RECOMMENDATION_ROUND_2026-08-11.md`,
   accepted by delegation, ADR-0025 in spec 12.3.0 (minor — the flag
   had no prior definition; severity 1's text, PLAN-005, and Section 8
   verbatim) via PRs #271/#272, re-attribution as #273.
   `irreversible-after-start` is defined temporally: the flag claims
   the mid-execution window (a reachable interrupted state
   unrestorable by unwinding, recovery roll-forward), severity claims
   endpoints, the combination is legal, and a flagged step's
   cancellation claims `no-writes` only before its first write. Four
   WP-060 increments are now startable under recorded decisions
   (reversal, solver unlock, backup family, combination unlock), the
   planner's four code debts all riding whichever Rust change comes
   first. SI-24 then completed the same arc on 2026-08-12 — the round
   run 2026-08-11, the delegation the next day: ADR-0026 in spec
   12.4.0 (minor — CAP-003 and PLAN-009 both verbatim plus additions)
   via PRs #274/#275, re-attribution as #276. A dry run is an apply
   rehearsal, not CAP-003's simulation: on a preview-backed plan it
   runs and refuses at the helper's own recomputed capability gate
   with a typed pending-qualification reason, never successful, so
   PLAN-009's guarantee stands absolute; gate order stays WP-070's
   under the parity property. **WP-060's register-gate list is now
   empty** — every gate it ever named (SI-15/16/17/19/24) resolved
   through its own recorded decision, four increments startable, the
   planner's code debts riding the first Rust change among them. One
   operational note for the next session: `gh pr checks <n>` run
   immediately after PR creation can scan before checks register and
   pass vacuously — branch protection caught exactly that on #275;
   even exit 0 is not proof while checks read "expected" (#277's first
   merge attempt), so delay before watching and let branch protection
   backstop; and never checkout/pull in a background job, which raced
   the working copy during the SI-20 arc. SI-20 then completed the arc
   on 2026-08-12: ADR-0027 in spec 12.5.0 (minor — Section 8
   closing-prose additions only; the table rows, terminal list, and
   "No other transitions exist" verbatim) via PRs #277/#278, no
   re-attribution because no WP-070 assignment exists — the ADR
   records the verification obligations so that assignment's creation
   cannot omit them. The two RecoveryRequired exits are the two arms:
   roll-forward continues the original plan; a distinct recovery
   action is its own plan whose selection disposes the original
   through the Failed edge with a journaled linkage, disposal durable
   before the recovery plan applies (HLP-005-structural on shared
   devices). SI-21 then completed the arc the same day: ADR-0028 in
   spec 12.6.0 (minor — the ladder's every sentence verbatim plus the
   apply-lifecycle definition) via PRs #279/#280, no re-attribution
   (no WP-070 assignment). An authorization act authorizes one apply —
   a journal-continuous lifecycle, plan hash plus unbroken JRN-001
   chain, that interruption suspends and only terminals end — so the
   three re-entry edges reuse nothing; the authorization is a journal
   fact, never process state; PLAN-007's window bounds every re-entry
   with its existing re-approval as the fresh-act route. Fed forward
   to SI-22, undecided: the authorization record is recovery-critical.
   SI-22 then completed the arc the same day: ADR-0029 in spec 12.7.0
   (minor — JRN-004's sentence verbatim plus the liveness-scoped
   rule) via PRs #281/#282, no re-attribution (no WP-070 assignment).
   Retention governs terminal history only; a non-terminal apply's
   records — the authorization act's included, ADR-0028's revisit
   condition discharged — are exempt until their apply terminates,
   the exemption closing over ADR-0027's linkage graph; the live
   segment is bounded by a per-apply budget whose exhaustion is a
   journaled failure (fail-closed toward the writer, never the
   recoverer); compaction records classify every gap as policy, torn
   tail, or corruption; sequence numbers stay monotonic across
   rotation. SI-23 followed (ADR-0030, spec 12.8.0, PRs #283/#284):
   the REC-011 backup is a protection artifact — helper-owned store
   inheriting JRN-004's location clause, hash-only references,
   ADR-0029's liveness rule adopted, consequence-stated end of life.
   Then SI-25 and SI-26 on Nate's directive ("finish SI-25 and
   SI-26"), a double reservation (#285) then two resolutions:
   ADR-0031 (spec 12.9.0, #286 — CAP-002 is a required minimum over a
   closed-and-versioned operation vocabulary, wipe a family of
   DIA-005's kinds, the delivered Operation enum extending only via
   WP-050's next reviewed increment) and ADR-0032 (spec 12.9.1, #287,
   patch — Section 16's "Stable" is CAP-003's `supported`, the
   ADR-0020 reading-selection shape with an editorial parenthetical).
   Finally the **WP-070 assignment was created** (#288, governance —
   the checker's `Governance:` trailer is required for new
   assignments): journal + execution state machine, the first package
   born with an empty register-gate list, its thirteen imported ADR
   obligations enumerated per increment, crates/journal/** and
   crates/statemachine/** reserved. Remaining open register items:
   SI-13 (WP-L110-era), SI-14 (Later, WP-050), SI-28 (Mitigated-open,
   evidence-gated), SI-37 (measurement-gated residual). No decision
   threads remain open, and every WP-050/060/070-era register
   question is decided. **WP-070 increment 1 then delivered** (PR
   #289, 2026-08-12): `crates/statemachine`, Section 8's thirteen
   states and twenty-three-row table as `Transition` variants,
   undeclared pairs unrepresentable over all 169 ordered pairs,
   terminal-effect records structural, `schemas/state-machine.md`
   byte-fresh from the same source. The Rust merge tripped the
   stopping condition — the first trip from outside WP-020 — and the
   **r6 sitting was run the same day** (VMID 9429, runbook r5→r6):
   all three acceptances re-taken on `a2e6db2` with identical value
   sets, eleven controls refused, custody run 12 with three agreeing
   digests, teardown verified, and the condition **re-pinned at
   `a2e6db2`** by PR #290, which also corrected a pre-existing stale
   count in test-tiers.md and recorded three carried-over labels in
   the copied r6 artifacts. Runbook scripts `*-r6.sh` are current on
   the Proxmox host; VMID 9430 is next.)
2. **WP-040's remainder stays decision-gated:** one transport increment per
   OS, each behind its own recorded route decision.
3. **WP-020 increment 3** (T3 physical-lab provisioning against the
   hardware matrix) is not started and is the package's only remaining
   increment. ADR-0007's revisit condition (ii) is worth re-reading before
   starting it: if lab architecture gives privileged-test state a home
   outside the source tree, Option B (a real nonce) becomes cheap and
   should be taken.
4. **Any third suite or multi-fixture contract** is a new reviewed boundary:
   it flips the shape pin and the count pin (now at two) and re-opens every
   generic-refusal test again. The general executor is ready for it; the
   suite should exist because a measurement needs it, not to exercise
   machinery.
5. **Any Rust merge re-opens both acceptances plus the 2j one.** Runbook
   scripts `*-r5.sh` are current on the Proxmox host (`root@10.7.7.100`);
   copy to `-r6`, bump `VMID` (9429 next) and `CANDIDATE_COMMIT`, fix the
   header prose. `05-guest-sitting-r5.sh` runs all three acceptances with
   eleven negative controls.

## 5. Operational traps this session hit, so you do not repeat them

1. **A mutation that never applied reads as a surviving test.** A WSL
   pipeline's `python3` heredoc failed silently (exit 127 swallowed), the
   unmutated test passed, and the pipeline's `git checkout --` cleanup then
   reverted the entire uncommitted fix. Apply mutants with the Edit tool,
   verify they're in place, never revert with `git checkout` over
   uncommitted work. (Saved to memory.)
2. **PowerShell single-quoted here-strings keep `''` literal.** Doubled
   apostrophes shipped into two commit messages and three PR bodies before
   being caught; amending forced a stack rebase. Write commit/PR text to
   files, use `git commit -F` and `gh --body-file`. (Saved to memory.)
3. **A freshly scp'd sitting script needs `chmod +x`.** `script -e -c`
   invokes the transcript target directly; a 0644 copy voids the run at
   exit 126 before any gate. The void transcript is retained as 2e custody
   run 10. (Saved to the runbook memory.)

## 6. What I would tell a reviewer to check first

- The 2j contract's backup offset against the generator: 4 MiB − 512 =
  4,193,792, `EFI PART` there on the generated image. Then re-read the two
  fail-closed layers a drifted offset hits (admission bound check, the
  differs-before-the-write gate) — if either is weaker than the record
  claims, the record overstates.
- The 2i containment pin
  (`the_destructive_protocol_runs_its_steps_in_the_required_order`): confirm
  the pinned sequence really is the one the 2h boundary recorded, because
  it is what licenses "the general protocol contains the accepted one".
- The r5 transcript's 2j section against the record: both range dumps at
  all three points, the eleven controls, and the counters
  (`fixtures_executed=1`, `ranges_written=2`).
- `reduce_admission`'s basename matching under adversarial orderings — the
  binding-by-bytes test covers two fixtures; convince yourself the
  remove-as-matched loop holds for N.
