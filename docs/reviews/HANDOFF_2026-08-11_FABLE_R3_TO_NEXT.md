# Handoff — 2026-08-11, the review-finding round (r3)

> **SUPERSEDED.** This document was written mid-session and extended twice
> as the same session continued into increments 2i and 2j. The complete,
> final handoff for the whole session is
> `HANDOFF_2026-08-11_FABLE_R5_TO_NEXT.md` — start there. This file is
> retained for the round-level detail its sections carry (the #248/#249/#250
> discharge specifics and the r3 sitting).

**From:** Claude (Fable 5), working with Nate through the evening of
2026-08-11.
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-11_OPUS_TO_NEXT.md`, written earlier the same
day. That document's §6 thread 2 — issues #248, #249, #250 — is fully
discharged, and thread 3's price (a VM sitting) is paid. Start at §4 here.

> **Untracked local handoff artifact.** `docs/reviews/**` stays untracked by
> convention; never stage it into a WP-020 commit.

Repository state at the end of the round: `main` at `d02a902`, spec 11.1.0.
Working tree clean apart from the untracked `docs/reviews/**` set (now 18
files). No open pull requests. Issues #248, #249, #250 closed. The 2e
stopping condition is pinned at `39b59f5` and holds: commits after it are
Markdown-only.

> **This handoff was extended in-session, twice.** After the review-finding
> round below, the same sitting-day continued into **increment 2i — the
> general destructive executor** (PRs #255/#256; plan in
> `WP-020_INCREMENT_2I_PLAN_2026-08-11.md`): the executor runs the
> registry's full contract shape (N fixtures, N non-overlapping ranges, a
> pre-flight over every fixture before any is attached), delivered on the
> VMID-9427 sitting. Then, on the operator's direction, **increment 2j
> registered the two-range suite** `gpt-basic-512-both-signatures-erase` —
> both GPT header signatures of `gpt-basic-512.img`, the backup at offset
> 4,193,792 measured before the contract was written — behind its own
> boundary (PR #257), with both edit-detectors flipped, every
> generic-refusal test re-read, and the stale AGENTS/CONTRIBUTING
> availability sentences repaired in the same change. Its acceptance passed
> **on its first take** in the VMID-9428 sitting (`ranges_written=2`,
> eleven controls refused, transcript `a788471b…`; the void first
> invocation — missing execute bit on the copied script — is retained as
> custody run 10), and **increment 2 itself is Delivered as scoped**
> (PR #258 records exactly what that does and does not mean). §4's pickup
> list is superseded: the live threads are now the 2026-08-08 decision
> items (§4.3), WP-040's decision-gated transports (§4.4), WP-020
> increment 3 (the physical lab, not started), and any third suite or
> multi-fixture contract — each a new reviewed boundary. Sitting scripts
> `*-r5.sh` are current on the Proxmox host; six sittings have now run
> from this runbook in one day, VMIDs 9424–9428 all destroyed and
> verified.

## 1. What this round did, in one line

Discharged all three open findings from the 2h adversarial review — one PR
each, merged in a stack — then re-took both Tier-2 acceptances in a fresh
disposable VM because those fixes re-opened them, and re-pinned the record.

| PR | What |
| --- | --- |
| [#251](https://github.com/nathan-mcbride54/PartMan/pull/251) | #249: `Admission::admit` counts verified handles per fixture (multiset, not set), `unwrap_or_default` becomes an explicit refusal. **The prerequisite for increment 2's multi-fixture scope.** |
| [#252](https://github.com/nathan-mcbride54/PartMan/pull/252) | #248: the rebind probe re-reads `LOOP_GET_STATUS64` on `EINVAL`; only an attachment observed read-write may name `KernelRefused`. The mapping is a pure classifier with every arm pinned. |
| [#253](https://github.com/nathan-mcbride54/PartMan/pull/253) | #250: the contracted write moves into `write_contracted_range(device, contract)` and a Tier-1 test measures where it lands (the issue's named fallback; the loop mapping stays the acceptance's measurement). |
| [#254](https://github.com/nathan-mcbride54/PartMan/pull/254) | The r3 sitting recorded; both acceptances re-pinned at `68298f2`. |

Every fix was mutation-checked before proposal (blind-EINVAL and
inverted-flag mutants for #248, a weakened count gate for #249, a
transposed-field mutant for #250 — each failed exactly the new test).

## 2. The sitting

VMID 9426, fresh disposable Proxmox VM from the same pinned jammy image,
kernel **5.15.0-186-generic** (no mid-provision reboot this time), commit
`68298f2`, runbook `*-r3.sh` copies on the host. 2e first on a pristine
tree, then 2h; nine negative controls refused; identical value sets to the
earlier sittings. The kernel's `LOOP_CHANGE_FD` refusal is now measured on
both -186 and -187, and since this re-take it is classified from an observed
status re-read rather than from the errno alone. Transcript
`ee3304014479b29b5f6efe8f7da2fc085a06e806b3fe47bd0ef456afd82c28bc`, custody
verified at all three hops. Teardown verified: no config, volume, or
snapshot. Bundles: `/root/partman-wp020-evidence-r3` (host),
`C:\Users\nmcbr\PartMan-evidence\WP-020-issue-fix-retake-2026-08-11`
(workstation).

## 3. Corrections this round made to earlier statements

- **Issue #249's `./` premise was wrong, and the test caught it.** Rust
  `Path` equality normalizes `.` components away, so `root/./x.img` IS
  deduplicated by the interlock's supplied-path `BTreeSet` after all. `..`
  components compare literally and survive, which is how the duplicate-handle
  authorization is really constructed. Recorded in the test comment, the
  changelog, and PR #251.
- **The 2e reproducibility sentence was stale** — "five times across two
  guests" while the custody table held seven runs. The 2g/2h round updated
  the table but not the sentence. Fixed with the correction noted in place.

## 4. Where to pick up

1. **Increment 2's own scope — the general destructive Tier-2 harness — is
   now genuinely unblocked.** #249 was its named prerequisite and is
   discharged. Constraints that still bind, verbatim from the last handoff:
   ADR-0007's revisit conditions, the 2e record's no-inherited-conclusion
   rule, and the 2g/2h bright line (no product write path, no storage-tool
   invocation, no domain types). A second registry entry is a new reviewed
   boundary and re-opens every generic-refusal test.
2. **Any Rust change re-opens both acceptances** — the condition is pinned at
   `68298f2` and has now tripped four times. Budget the sitting; the r3
   scripts on the Proxmox host make it roughly an hour end to end.
3. **The 2026-08-08 decision threads remain untouched** by both of today's
   rounds: SI-39 option (c), the decision briefs, ADR-0014 fork
   recommendation, and SI-18 gating WP-040's authorization vocabulary.
4. **WP-040's remainder stays decision-gated** — one transport increment per
   OS, each behind its own recorded route decision.

## 5. Two operational traps hit this round, so you do not repeat them

1. **A mutation that never applied reads as a surviving test.** A WSL
   pipeline used `python3` (not installed there); the mutation silently
   failed with its exit code swallowed, the unmutated test passed, and —
   worse — the pipeline's `git checkout --` cleanup then reverted the entire
   uncommitted fix, not just the mutation. Apply mutations with a tool that
   errors loudly when the target text is missing, verify the mutant is in
   place before trusting a red-or-green answer, and never use `git checkout`
   to unwind a mutation while uncommitted work sits in the same file.
2. **PowerShell single-quoted here-strings keep `''` literal.** Doubled
   apostrophes leaked into two commit messages and three PR bodies before
   being caught; the commits were amended (forcing a stack rebase) and the
   bodies edited. Write commit messages and PR bodies to files and pass
   `-F`/`--body-file`.

## 6. What I would tell a reviewer to check first

- Re-read `classify_rebind_probe` in `crates/ffi-linux-loop/src/linux.rs`
  against issue #248's discharge criterion: nothing may reach
  `KernelRefused` from an unexamined `EINVAL`. The test
  `the_rebind_probe_names_only_an_observed_kernel_state` should make the
  criterion mechanical.
- Check `Admission::admit`'s counting against a three-handle,
  two-fixture authorization on paper — that is the case #249 said the old
  arity check would not survive, and it is the shape increment 2 will
  actually build.
- The r3 transcript's negative-control section, against the record's claim
  that no refusal path wrote anything.
