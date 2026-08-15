# WP-060 increment 9 plan — PLAN-005's full cancellation vocabulary, 2026-08-12

**Written before the first line of code**, the arc-plan convention.
Local artifact, docs/reviews convention: never staged into a commit.

## What this arc delivers

The one WP-060 item left from the assignment's beyond-list: PLAN-005's
full cancellation vocabulary — each step declares `cancellable`,
`checkpoint-cancellable`, or `non-cancellable` — with its
jointly-sequenced WP-010 hashed-body carriage (plan body v4), opened by
a recorded decision in the WP-035 route-decision shape as the
assignment requires. The arc also carries the named leftover from the
increments 5–8 arc: the v1 plan-body retirement, now that nothing
outside `crates/domain`'s own tests and vectors emits v1.

## The decision the arc opens under (recorded in WP-060.md by PR A)

Three routes for assigning classes to step families:

1. **Derive from `InterruptionProfile`** — rejected. Spec 12.3.0
   records cannot-stop (PLAN-005's `non-cancellable`) and cannot-unwind
   (`irreversible-after-start`) as independent facts in both
   directions; a derivation re-couples them. The counterexamples are
   concrete: `Wipe` and `Shrink` share `UnrestorableIntermediate` yet
   differ in stoppability, and the journaled chunk copy has
   interruption windows yet stops safely at its declared checkpoints.
2. **The all-non-cancellable floor with no family table** — rejected as
   the vacuous shape: it discharges PLAN-005's letter while deleting
   the vocabulary's point, leaves ACC-012's family claim unstated, and
   records no place where a family earns its class.
3. **Per-family stated declaration over a conservative floor** —
   chosen. 12.3.0's closing prose already says flag assignment to
   concrete step families "is each building package's testable
   declaration"; the class follows the same discipline. A family
   claims `cancellable` or `checkpoint-cancellable` only on decided
   text or a measured mechanism; the floor is `non-cancellable`
   (claiming less cancellation than reality is safe — the UI offers
   nothing; claiming more makes the UI offer a stop the executor
   cannot honor, exactly the PLAN-005 violation).

The table this decision states today:

| Family | Class | Backing |
| --- | --- | --- |
| `Move`, `Copy` | `checkpoint-cancellable` | PART-005's normative journaled chunk copy with a durable progress map; ACC-012's "stops at the declared checkpoint". Stated for the family before the planner emits it — the increment-8 `InterruptionProfile` precedent. |
| Everything else the planner emits (entry writes, sized create, grow, shrink, wipe, table repair) | `non-cancellable` | The floor. No measured safe-stop story exists for any of them; the executor does not exist yet. |

Revisit conditions, named: a measured safe-stop story for the
overwrite wipe (the natural first candidate — long-running,
structurally consistent at every prefix — but its mechanism could be a
hardware sanitize command that cannot stop, so the claim waits for the
executor era's measurement); a checkpointed story for the shrink's FS
transform if one appears; EXE-004's two-second acknowledgment binds the
executor, never this table; any new family arrives with its own stated
class. Each earning updates the recorded decision.

What stays untouched: `CancelClaim` and ADR-0025's coupling rule (the
independence is now *visible*: the atomic entry write is
non-cancellable yet unflagged; a flagged family may earn
checkpoint-cancellable). The UI must-not-offer law and EXE-004 are the
UI and executor packages' boundaries, recorded as such.

## The PRs, in order

- **PR A — the assignment edit** (`work/wp060-cancellation-decision`,
  WP-060-owned Markdown only, trips no stopping condition): the
  recorded decision section in WP-060.md, the increment 9 entry, the
  Requirement IDs line updated (PLAN-005 loses its "when the vocabulary
  exists" hedge). The WP-035 increment-10 shape: routes, costs, choice,
  what-it-makes-true, revisit conditions.
- **PR B — WP-010 slice 3n, plan body v4**
  (`work/wp010-plan-body-v4`): `Cancellation` enum in
  `crates/domain/src/model/step.rs` closed at PLAN-005's three values,
  `NonCancellable` the fail-closed default; `PlanStep` carries it;
  `mutating`/`mutating_classed` default it, the full-form constructor
  takes it explicitly; body v4 with required per-step `cancellation`
  Text (`cancellable` | `checkpoint-cancellable` | `non-cancellable`,
  unknown refuses), the draft's steps carrying it identically; v3
  refused at decode with its retirement recorded (the v2 precedent:
  one change window, no emitter, vectors regenerated in the same
  slice); `schemas/domain/plan-body.md` updated (§0, §3, Requirement
  IDs gain PLAN-005); vectors regenerated as v4; the TypeScript suite
  reproduces them unchanged; CHANGELOG entry. Non-breaking for the
  planner: the 3m precedent (class defaulted through the old
  constructors) — verify the planner pins no v3 digest before landing.
- **PR C — WP-060 increment 9** (`work/wp060-increment-9`):
  `cancellation_class(operation)` stated per family exactly as the
  recorded table, wired explicitly into every step construction (never
  the default relied on, so a family change flows); fixtures — the
  partition (every `Operation` maps, pinned like the
  `interruption_profile` partition), an emitted plan's v4 body carrying
  `non-cancellable` on every step end to end, the chunk-copy family
  statement pinned, the repair family's step declared through the same
  seam; traceability regenerated (PLAN-005 rows backed by live tests);
  README row, CHANGELOG, WP-060.md increment row flipped to delivered.
- **PR D — WP-010 slice 3o, the v1 retirement**
  (`work/wp010-plan-body-v1-retirement`): migrate
  `plan_tests.rs`/`body_vectors.rs` off `OperationPlan::assemble`,
  remove the v1 emission and acceptance (any non-current version
  refuses at decode), drop the two v1 vectors, record the retirement
  in plan-body.md §0 and the CHANGELOG. Its own reviewed change, as
  §0 has promised since 3l.
- **The r12 sitting + PR E — the WP-020 re-pin**
  (`work/wp020-r12-repin`): PRs B–D are Rust merges and trip the 2e
  stopping condition. **One sitting at the arc's head** (VMID 9435,
  the runbook's `*-r11.sh` scripts adapted to r12) re-takes 2e, 2h
  (`ranges_written=1`), and 2j (`ranges_written=2`), teardown
  verified; PR E records the sitting and re-pins. The one-sitting
  economics is recorded here, before the first merge, as the r11
  record demands.

## Verification discipline

Every PR: `cargo xtask ci`, `cargo xtask test --tier 1`,
`cargo xtask verify-change-ownership --base origin/main`, real exit
codes. Mutation verification before proposing each substantive PR
(applied with Edit, reverted by re-Edit, never git checkout over
uncommitted work):

- PR B mutants: default flipped to `Cancellable`; the v3 refusal
  dropped (decode accepts version 3); an unknown `cancellation`
  spelling accepted; the draft-step field dropped from encode.
- PR C mutants: `Move`'s family statement flipped to `NonCancellable`;
  the wiring reverted to the default (derivation dead);
  `cancellation_class` made non-total (a family falls to a wildcard
  that misstates it — if the shape permits).

Each mutant must be killed by a named test; a surviving mutant means a
missing fixture, added before proposal (the 3m precedent).

## Merge mechanics

PRs are chained in sequence; merging one invalidates the siblings'
required checks — update-branch all at once, merge in order A → B → C
→ D, then the sitting, then E. `gh pr checks` races check
registration: delay before watching, let branch protection backstop.
