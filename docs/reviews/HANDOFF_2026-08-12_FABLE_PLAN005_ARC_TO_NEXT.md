# Handoff — 2026-08-12/13, the PLAN-005 cancellation arc (decision + slices 3n/3o + increment 9 + r12)

**From:** Claude (Fable 5), the session Nate directed with "start with
PLAN-005."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-13_FABLE_WP060_ARC_TO_NEXT.md` (the
increments 5–8 arc). The arc plan this session wrote before its first
line of code is `WP-060_INCREMENT_9_PLAN_2026-08-12.md`.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at the #310 merge (the r12 re-pin), spec 12.9.1 — unchanged by
this arc: no spec text moved; the one recorded decision lives in the
WP-060 assignment, in the WP-035 route-decision shape. Working tree
clean apart from untracked docs/reviews. No open PRs. The stopping
condition is re-pinned at `77b0dd7`; VMID 9435 destroyed and verified;
runbook scripts `*-r12.sh` current on the Proxmox host; VMID 9436
next.

## 1. What this session did — five merged PRs, one sitting

| PR | What |
| --- | --- |
| #306 | The recorded cancellation-class decision in WP-060.md (the WP-035 shape: routes, costs, choice, what-it-makes-true, revisit conditions): the class is a per-family stated declaration over the fail-closed `non-cancellable` floor, never a derivation from `InterruptionProfile` (12.3.0 records the independence); `Move`/`Copy` stated `checkpoint-cancellable` on PART-005 + ACC-012 before the planner emits them; every emitted family on the floor with named revisit conditions (the overwrite wipe's measured safe-stop story first) |
| #307 | WP-010 slice 3n — plan body v4: required per-step `cancellation` closed at PLAN-005's three words, typed `Cancellation` with the floor default, `mutating_declared` the fully-declared constructor, draft steps pinned to the floor like their class; **version 3 retired** (v2 precedent), vectors regenerated as v4, TypeScript suite reproduces |
| #308 | WP-060 increment 9 — the vocabulary delivered: `cancellation_class` wired explicitly into all four construction sites, the partition fixture, the end-to-end v4 body fixture, `CancelClaim` untouched; **the assignment's beyond-list is now fully delivered** |
| #309 | WP-010 slice 3o — **the v1 plan-body retirement** (the previous arc's named leftover): `OperationPlan::assemble` removed, the linkage a required field (plan-without-linkage unconstructible), v1 refuses at decode, the identity-bound vector's SAFE-003 coverage surviving as `plan-v4-bound-identity-wipe`; version 4 is now the sole live version |
| #310 | WP-020 r12 re-pin: the record sweep and the new pin at `77b0dd7` |

The r12 sitting (VMID 9435, 2026-08-13 UTC): eleven controls refused,
2e's twelfth re-take with the identical value set, 2h's tenth
(`ranges_written=1`, 8 bytes), 2j's seventh (`ranges_written=2`, 16
bytes, both signatures restored), custody run 19 with three agreeing
digests (`b42574de…d4d5`), teardown verified 2026-08-13T02:36:40Z.
**One sitting covered all three Rust merges** — recorded in the arc's
plan before the first merge. The sitting's **first invocation was
void** (custody run 18): launched through `sudo`, refused by 2e's own
Tier-1 redaction sweep on the injected `SUDO_USER` — an operator
mistake the runbook memory already warned about; the guest was rolled
back to the pre-acceptance snapshot and the cited run was
root-invoked. Recorded in the WP-020 record, not smoothed over.

## 2. Decisions this session made, worth review

- **The cancellation-class decision itself** (WP-060.md, PR #306).
  Nate directed "start with PLAN-005"; the decision was recorded and
  implemented in one autonomous arc. Merging is not acceptance: if the
  table (Move/Copy checkpoint-cancellable; everything else the floor)
  or the floor-first posture reads wrong, the decision section has
  named revisit conditions and its own recorded rejected routes.
- **Wipe is non-cancellable for now**, deliberately: its mechanism
  could be a hardware sanitize command that cannot stop, so the
  cancellable claim waits for the executor era's measurement — the
  decision's first named revisit condition. A user-visible cost,
  recoverable by a reviewed table update.
- **Draft steps are pinned to the floor** exactly as their class is
  pinned to `ordinary`; a draft family off the floor is a future
  reviewed extension.
- **One planner mutant recorded as semantically equivalent** rather
  than claimed killable (a call site bypassing the derivation with the
  floor constant — indistinguishable today because every emitted
  family sits on the floor). Recorded in the assignment's increment 9
  text.
- **The v1 retirement kept the `reversal()` accessor's `Option` shape**
  (planner tests consume it); the field itself is required now. A
  follow-up could straighten the accessor when the planner next
  touches those tests, but nothing forces it.

## 3. What remains open, in this territory

1. **WP-060's assignment is fully delivered** — increments 1–9 and
   every beyond-list item. The planner package is at a natural resting
   point; its next work arrives with new decided vocabularies (e.g.
   the consequence-text hashed carriage ADR-0023 deferred, or
   PLAN-005 class earnings under the decision's revisit conditions).
2. **WP-040 transports** (route decisions unrecorded), **WP-020
   increment 3**, and the register residue (SI-13, SI-14, SI-28,
   SI-37) — untouched, as before.
3. Mutation discipline held: 9 mutants killed by named tests across
   the three Rust PRs (4 in 3n, 3 in increment 9, 2 in 3o), plus the
   one recorded equivalent.

## 4. Operational notes

- The r12 sitting ran fast again (~6 min provision, ~2.5 min sitting)
  once launched correctly. The void first invocation cost one rollback
  and about ten minutes.
- `/tmp` on the guest is cleaned every boot: a `qm rollback` + start
  loses scripts staged there. Stage in `/root` (inside the snapshot)
  and reapply post-snapshot enablements (root login) after any
  rollback. The runbook memory now says this.
- The macOS Tier-1 CI runner flaked once on a crates.io fetch
  ("failed to get `serde_json`"); `gh run rerun --failed` cleared it.
