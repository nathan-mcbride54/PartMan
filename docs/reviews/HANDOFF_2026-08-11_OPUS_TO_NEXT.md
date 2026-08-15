# Handoff — 2026-08-11, the 2g/2h round

**From:** Claude (Fable 5 for the first half, Opus 5 from the 2h
implementation onward), working with Nate through 2026-08-11.
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-11_FABLE_TO_NEXT.md`, written earlier the same
day. That document's §5 pickup list is fully discharged; start here instead.
**Pick up at:** §6. There is no half-finished work. Every open thread listed
there is a decision or a new increment, not a loose end.

> **Untracked local handoff artifact.** `docs/reviews/**` belongs to WP-000,
> and these handoffs are kept untracked by convention. Do not stage this into
> a WP-020 commit — `verify-change-ownership` will refuse it, and it refused
> exactly that during this round when a `git add -A` swept the whole directory
> in. See §5.4 for how that went wrong and what it nearly cost.

Repository state at the end of the round: `main` at `d71e4a1`, spec 11.1.0.
Working tree clean apart from the untracked `docs/reviews/**` set, which is
now 16 files including this one. No open pull requests. Issue #175 is closed.

---

## 1. What this round did, in one line

Discharged the previous handoff's entire pickup list, then built WP-020
increment 2's first two slices — the compiled destructive-suite registry (2g)
and the first destructive suite (2h) — and took both their acceptances in a
disposable VM. **This repository can now write, under a Tier-2 gate, to
exactly one contracted byte range of one generated fixture.**

## 2. What landed

| PR | What |
| --- | --- |
| [#237](https://github.com/nathan-mcbride54/PartMan/pull/237) | Dependabot `@types/node` 26.2.0. Was already repaired and green; merged. |
| [#242](https://github.com/nathan-mcbride54/PartMan/pull/242) | WP-000 checkout-boundary fix. Rebased onto post-#237 main (content byte-identical to what had been reviewed), re-gated cold, merged. |
| [#243](https://github.com/nathan-mcbride54/PartMan/pull/243) | WP-035: exempt the root login's collision with the udev caveat. See §3. |
| [#244](https://github.com/nathan-mcbride54/PartMan/pull/244) | Issue #175 discharged: the 2e acceptance re-taken on `582e6d1`, record re-pinned. |
| [#245](https://github.com/nathan-mcbride54/PartMan/pull/245) | **Increment 2g**: the destructive-suite registry becomes a compiled type. |
| [#246](https://github.com/nathan-mcbride54/PartMan/pull/246) | **Increment 2h**: the first destructive suite, implementation and Tier-1 evidence. |
| [#247](https://github.com/nathan-mcbride54/PartMan/pull/247) | Both acceptances recorded; 2h moved to Delivered. |

Delivery rows now read: increments 1–1g, 2a–2d, 2e, 2f, **2g**, and **2h**
delivered. Increment 2's own scope is still **unbuilt**, and every surface
says so — one read-only acceptance plus one single-range destructive suite is
not a destructive harness.

## 3. The one thing to understand before touching this code

**The identity sweep can refuse on any root host, and root is the posture the
privileged acceptances require.**

WP-035's `no_output_in_any_mode_carries_an_environment_value` checks
identity-bearing variables (`USER`, `LOGNAME`, `HOME`, …) at a three-byte
floor. A root login sets `USER=root`, and `root` is a substring of the udev
caveat `apps/cli/src/devices.rs` carries in-band on every udev-database value
("computed by root's udevd"). That is static compile-time text with the
env-read source guard proving no environment read exists — a coincidence
collision, the same class as the recorded `CIUSER=partman` and
`RUNNER_OS=Windows` incidents.

Consequence before #243: the sweep could never pass on a root host, so the
issue #175 retake refused at its Tier-1 gate before reaching the acceptance.
The fix is the exemption list, in its own reviewed commit, never the commit
under proof. **If you add static output containing another identity-shaped
value, this will refuse the same way, and the remedy is the same.**

## 4. What 2g and 2h actually establish

**2g** gives "no destructive suite is registered" a compiled type. A `Suite`
names its fixture set by catalogue basename, its verified target class, its
per-fixture intended-change contract (byte ranges, each with its replacement
bytes and its stated reason, everything outside pinned by digest bracket), and
its teardown proof obligations. `Admission::admit` consumes the SAFE-007
`Authorization` and refuses anything but exactly the declared fixture set.

**2h** registers `gpt-basic-512-signature-erase`: eight bytes at offset 512 of
`gpt-basic-512.img` — the primary GPT header's signature — replaced with
zeros.

The load-bearing design choice, and the one to preserve: **the attachment is
read-write.** That is not a convenience. The 2e record forbids a destructive
path inheriting its conclusion and names read-write attachment as the first
acceptable pre-write discipline, because the loop driver refuses
`LOOP_CHANGE_FD` on a read-write attachment — the rebind becomes
*inapplicable* rather than *detected after the fact*. The suite attempts that
rebind mid-run, before writing, and a kernel that accepted it voids the run.

**That was an assumption about the kernel until the 2026-08-11 sitting, and it
is now a measurement on 5.15.0-187-generic.** It is not a claim about every
kernel. Keep the leg.

## 5. Four defects of one shape, found this round

Every one of these was a check that reported success without establishing its
claim. Listing them together because the pattern is the point.

1. **The identity sweep** (§3) — a guard that could never pass, which reads as
   protection until the first host that trips it.
2. **The 2h restoration guard.** My first draft compared
   `catalogue::generate`'s returned manifest to `catalogue::expected()` — the
   same pure function of the same compiled data on both sides — so it never
   read a byte from disk while printing `backing_regenerated_to_catalogue=true`.
   Worse, the test I added to cover it *asserted the tautology holds* rather
   than catching it. Found by the adversarial review (§5.1). Fixed by
   `catalogue::verify_on_disk`, which re-reads and re-hashes.
3. **The runbook's snapd purge.** `02-guest-provision.sh` ran
   `apt-get purge -y snapd` with stderr discarded and `|| true` appended. The
   purge failed on the 2h guest — snapd's squashfs mounts and their loop
   bindings were live — and the script continued, aborting only incidentally
   when the next recursive delete hit a read-only filesystem. Had that delete
   succeeded on a partially unmounted tree, provisioning would have reported
   success **with snapd installed**, and the sitting would have recorded the
   no-other-loop-administrator exclusion as established when it was not. Both
   host copies now unmount first, purge with output visible, and prove absence
   with `dpkg -l`. Backups are at `*.bak` on the Proxmox host.
4. **`changed_exactly_as_contracted`** named a change the protocol never
   measured — it checked equality *after* the write, which a range that
   already held those bytes and was never written also satisfies. The range is
   now read before the write and required to differ.

### 5.1 The adversarial review, and what it did not cover

Before proposing #246 I ran a five-lens adversarial review as a workflow
(kernel semantics, protocol ordering, boundary conformance, test adequacy,
contract arithmetic), with refute-by-default skeptics on the findings. Script:
`.../workflows/scripts/wp020-2h-adversarial-review-wf_1b2f8a32-1d2.js` in the
session directory; re-runnable.

Twenty raw findings, twenty after dedup. **Only the top eight were verified**
— that was the budget — so twelve were reported unverified. Six of the eight
were confirmed and all six are fixed. Two were refuted on inspection and
deliberately left alone: the detach-precedence ordering is correct as written,
and `Admission::admit` not checking registry membership is intentional (it is
what keeps its own refusal semantics testable with an unregistered suite, so
the check belongs in the executor, which is where it now is).

**Of the twelve unverified, most were duplicates of confirmed findings and are
fixed incidentally. Three were genuinely open, and all three are now filed:**

- **[#248](https://github.com/nathan-mcbride54/PartMan/issues/248) — the
  rebind probe names a kernel state it never observes.**
  `probe_forbidden_rebind` treats `EINVAL` as "the attachment is read-write, so
  the rebind is inapplicable". The driver's read-only check does run first, so
  it is right today — but a misclassified `EINVAL` is a false safety pass that
  proceeds into the write, and nothing tests the mapping. The suggested
  discharge is to re-read the status flags on `EINVAL` so the returned value
  names an observed state.
- **[#249](https://github.com/nathan-mcbride54/PartMan/issues/249) — admission
  compares fixture-name sets.** Duplicate verified handles for one fixture
  collapse to the same set. Unreachable today because `consume_admission`
  requires exactly one target — but that arity check is written for a
  single-fixture suite and would not generalize, which matters directly to
  thread 1 below.
- **[#250](https://github.com/nathan-mcbride54/PartMan/issues/250) — nothing
  tests where the contracted write lands.** The only Tier-2 write in the
  repository has no automated coverage; the fake's `write_contracted` never
  involves the offset. The post-conditions would catch a misplaced write, but
  they are checked by the same protocol whose write is in question.

None blocked delivery in my judgement. Each issue states why it is not
currently a live defect as carefully as it states the defect, so whoever picks
one up can disagree with that judgement on the evidence.

### 5.2 Evidence bundles

Outside the repository, per Section 16. Digests in the WP-020 record.

| Sitting | Workstation | Proxmox host |
| --- | --- | --- |
| 2026-08-03 original | `C:\Users\nmcbr\PartMan-evidence\WP-020-increment-2e-2026-08-03\` | `/root/partman-wp020-2e-evidence-2026-08-03` |
| 2026-08-11 issue #175 retake | `…\WP-020-increment-2e-retake-2026-08-11\` | `/root/partman-wp020-2e-evidence-retake-2026-08-11` |
| 2026-08-11 2e re-take + 2h acceptance | `…\WP-020-increment-2h-2026-08-11\` | `/root/partman-wp020-2h-evidence` |

The middle bundle retains the **refused** run (`3fc6e94c…`) alongside the pass,
following the record's keep-revisions practice.

### 5.3 Sitting mechanics worth reusing

- Operator environment: `root@10.7.7.100`, node `proxmox`, PVE 9.2.4. Scripts
  in the host's `/root/`. `05-guest-2h-sitting.sh` is this round's combined
  script: gates, nine negative controls, the 2e acceptance, then the 2h suite,
  with the declared range dumped before and after.
- **Sequencing matters.** 2e runs first and on a pristine tree, because 2h
  mutates a fixture and restores it; a read-only proof taken afterwards is a
  proof about a restored tree.
- If a guest reboots during provisioning it comes up on whatever kernel the
  image's updates installed. This one did — 5.15.0-187 where both earlier
  sittings ran -186. That is a material environment fact, recorded, not noise.
- Lazily detached loops (`umount -l` then `losetup -d`) stay bound until the
  last reference drops. Rebooting is the clean way to get an unambiguous loop
  table; `clear-snap.sh` on the host does the unmount half.

### 5.4 One mistake I made, so you do not repeat it

Amending the 2h commit, I ran `git add -A -- … docs …`, which swept the
untracked `docs/reviews/**` set into the change. `verify-change-ownership`
caught it. My first correction — `git rm -r --cached docs/reviews` — was
worse: it also unstaged the review files that were **already tracked**, so the
commit then *deleted* them. Recovered with
`git checkout origin/main -- docs/reviews`, which restores tracked files and
leaves untracked ones alone. Check `git diff --name-only origin/main...HEAD`
after any such correction; I did, which is the only reason it did not ship.

## 6. Where to pick up

Nothing is half-done. These are the live threads, most actionable first.

1. **Increment 2's own scope: a general destructive Tier-2 harness.** This is
   the honest next slice and it is not small. Constraints that still bind:
   ADR-0007's revisit conditions (an operator must stay in the trigger path;
   an unattended T2/T3 trigger forces re-deriving the third factor), the 2e
   record's rule that a destructive path may not inherit its conclusion, and
   the bright line in the 2g/2h boundaries — no product write path, no
   storage-tool invocation, no domain types. A second registry entry is a new
   reviewed boundary and re-opens every generic-refusal test again.
2. **Issues [#248](https://github.com/nathan-mcbride54/PartMan/issues/248),
   [#249](https://github.com/nathan-mcbride54/PartMan/issues/249), and
   [#250](https://github.com/nathan-mcbride54/PartMan/issues/250)** — the three
   open review findings from §5.1. Each is cheap, and each is the kind of
   inherited-check gap this package normally refuses. **#249 is a prerequisite
   for thread 1**, not merely adjacent to it: its arity check stops
   generalizing the moment a second fixture enters a suite.
3. **Any Rust change re-opens both acceptances.** The 2e stopping condition is
   pinned at `4fbb2f9` and currently holds (zero non-Markdown paths since).
   It has now tripped three times. Budget a VM sitting into any increment that
   ships code — that is the price of a proof about a compiled artifact, and
   the alternative is the "not on the code path" reasoning this package
   declines.
4. **The 2026-08-08 decision threads are untouched by this round** and stand
   where that handoff left them: the SI-39 option (c) recommendation, the
   decision briefs, and the ADR-0014 fork recommendation. `SI-18` still holds
   the severity-1 fresh-authorization question that gates WP-040's
   authorization vocabulary.
5. **WP-040's remainder stays decision-gated** exactly as its assignment
   sequences it: one transport increment per OS, each behind its own recorded
   route decision.

## 7. What I would tell a reviewer to check first

- Read the increment 2g and 2h authorization boundaries in
  `docs/work-packages/WP-020.md` and hold the code to them sentence by
  sentence. That is how the boundary-conformance lens found two of the six
  confirmed defects.
- Re-derive the claim in §4 yourself: does the loop driver actually refuse
  `LOOP_CHANGE_FD` on a read-write attachment? If that is wrong, the whole
  pre-write discipline is unsound, and everything else in 2h is decoration.
- Mutation-test anything you doubt. Thirteen gates were mutation-verified this
  round; the method found nothing new by the end, which is the point at which
  it is worth trusting.
