# Handoff — 2026-08-12/13, the WP-060 unlock arc (increments 5–8 + slices 3l/3m + r11)

**From:** Claude (Fable 5), the session Nate directed with "Complete
WP-060's four startable increments (reversal, solver unlock, backup
family, combination unlock)."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-11_FABLE_R5_TO_NEXT.md` (WP-070 arc era;
its state description was already superseded by the r6–r10 sittings
recorded on main). The arc plan this session wrote before its first
line of code is `WP-060_INCREMENTS_5-8_PLAN_2026-08-12.md`.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at the #305 merge (r11 re-pin recorded at `6e3cea3` on branch,
pinning commit `667f6aa`), spec 12.9.1 — unchanged by this arc: every
change implemented recorded decisions, no spec text moved. Working
tree clean apart from untracked docs/reviews. No open PRs. The
stopping condition is re-pinned at `667f6aa`; VMID 9434 destroyed and
verified; runbook scripts `*-r11.sh` current on the Proxmox host;
VMID 9435 next.

## 1. What this session did — seven merged PRs, one sitting

| PR | What |
| --- | --- |
| #299 | WP-060 increment 5 — the SI-15 solver unlock (ADR-0023): deviations are authored, not inherited; the misaligned grow-at-tail case proceeds authoring only the aligned end, the untouched start carried as a typed inherited fact in new planner-level consequence material; coincident-edge placement recorded naming the edge; the no-fourth-state property swept — with authored-end policy newly enforced on create and shrink ends, which had escaped judgment entirely (the fourth state, closed) |
| #300 | WP-010 slice 3l — plan body v2 (ADR-0022, jointly sequenced): the reversal linkage (draft by ID+hash / per-step impossibility statements / reapply-forward by ID — acyclic with no mutual-hash spelling), per-step preconditions re-checked at every boundary, the `ReversalDraft` artifact with step-output target spellings and its own compose/decode/bind boundaries, no-draft-no-Reversible structural, prediction-never-binds guards |
| #301 | WP-060 increment 6 — the reversal (PLAN-008): every plan emits a truthful draft (sized create — which now honestly claims Reversible — and grow) or per-step machine-readable statements; end-to-end fixtures: draft byte-determinism, resolve-after/refuse-before, truth decay by precondition, the prediction never binds |
| #302 | WP-010 slice 3m — plan body v3 (ADR-0024, jointly sequenced): the typed step class (`ordinary`/`table-repair`), the `uncapturable-regions` acknowledgment with the journal's region discipline, the class-conditioned acknowledgment law (identity-bound-restore's arm exists at last; both table-state kinds unconstructible outside the family, re-run at the boundary), `pre-state-preserved-for-recovery`; v2 (one change window, no artifact) refused at decode, retirement recorded |
| #303 | WP-060 increment 7 — the backup family (PART-013's planning half): derived protection obligations per touched table-bearing device (parse-backup / journaled-determination with no acknowledgement / raw-capture of exactly the located table regions); SAFE-005's planner half refusing ordinary operations on Indeterminate media before any obligation; `plan_repair` as the typed family's entry point, fail-closed both directions; the capture-impossible arm only under the plan-creation acknowledgement riding the hashed body |
| #304 | WP-060 increment 8 — the combination unlock (ADR-0025): the flag derived from the typed `InterruptionProfile` criterion (wipe/shrink/repair flagged; entry writes not; the journaled chunk copy stated unflagged for its family); severity-1+flag constructing exactly on its truthful draft and refusing without; `CancelClaim::no_writes` unconstructible flagged-after-first-write; `plan_flags` union making the ceremony inputs derivable, enforcement recorded as the helper packages' boundary |
| #305 | WP-020 r11 re-pin: the record sweep and the new pin at `667f6aa` |

The r11 sitting (VMID 9434, 2026-08-13 UTC): eleven controls refused,
2e's eleventh re-take with the identical value set, 2h's ninth
(`ranges_written=1`, 8 bytes), 2j's sixth (`ranges_written=2`, 16
bytes, both signatures restored), custody run 17 with three agreeing
digests (`10297fab…4b8d`), teardown verified 2026-08-13T00:53:24Z.
**One sitting covered all six Rust merges** — the arc's plan recorded
that economics before the first merge, and the WP-020 record states it
as the arc's plan rather than a shortcut discovered later.

## 2. Decisions this session made (within the ADRs' bounds), worth review

- **Version dance:** one-PR-one-package forbids atomic cross-crate
  breaking changes, so each schema change landed as a non-breaking
  WP-010 PR (parallel `assemble_linked` path, v1 kept) with the WP-060
  consumer following. v2 lived one change window and was refused at
  decode in 3m with its retirement recorded. **v1 is still emitted by
  `OperationPlan::assemble` and still accepted** — its retirement,
  with the v1 vectors' regeneration, is the arc's named leftover (a
  small WP-010 cleanup slice, once nothing emits v1; today the domain
  tests and the two v1 vectors do).
- **Grow keeps Disruptive severity** despite its truthful draft — the
  rule is one-directional (no draft, no Reversible; a draft does not
  compel the claim), and the FS-grow reversibility story is unmeasured.
- **Shrink is flagged** `irreversible-after-start` on the
  destroyed-tail argument (destroyed bytes make every post-first-write
  state unrestorable). If a shrink family with a restorable
  intermediate ever appears, ADR-0025's revisit condition says file the
  case.
- **`plan_repair` requires an Indeterminate table state** — a repair
  over a Present table is a refusal (`RepairNeedsAnIndeterminateTable`),
  deliberately: ADR-0024's family exists for that state; widening is a
  reviewed extension.
- **ADR-0022 verification item 6 and ADR-0025 fixture 4's enforcement
  half** (the reversal's own authorization; the tier computation) are
  recorded as helper-package boundaries in the assignment — no tier
  derivation function exists anywhere yet, and the planner did not
  invent one.
- The **consequence-text carriage** for SI-15's inherited facts is
  planner-level (`Planned.consequences`, typed with rendered
  sentences); hashed body carriage waits for the consequence-text
  vocabulary's own jointly-sequenced change.

## 3. What remains open, in this territory

1. **PLAN-005's full cancellation vocabulary** — the one WP-060 item
   left from the assignment's beyond-list, with its jointly-sequenced
   WP-010 body carriage. Increment 8's `CancelClaim` covers only
   ADR-0025's coupling rule.
2. **The v1 plan-body retirement** (WP-010 cleanup slice, above).
3. **Mutation-verification discipline held**: every increment's gates
   were mutant-tested before proposal (18 mutants total, each killed by
   a named test; slice 3m's pass exposed a missing fixture — the
   Present-state identity-bound-restore refusal — which was added
   before the mutant could be killed, recorded in that PR).
4. WP-040 transports, WP-020 increment 3, and the open register
   residue (SI-13, SI-14, SI-28, SI-37) are untouched by this arc.

## 4. Operational notes

- The r11 sitting ran fast (~6 min provision, ~2.5 min sitting) — the
  hour estimate is dominated by slower hosts; do not assume something
  skipped: the log shows the full clone, both builds, all three
  acceptances, and the teardown proof.
- The `gh pr checks --watch` timeout pattern: back it with a
  `until … grep pending` background loop keyed on log content, the
  same lesson as the sitting monitor.
- The traceability generator accepted PART-009/PART-013/PLAN-008 rows
  for WP-060 once the assignment's Requirement IDs named the claims
  (PART-013's planning half was added to the list in increment 7's own
  PR — an assignment self-edit under the ordinary trailer, the
  established pattern).
