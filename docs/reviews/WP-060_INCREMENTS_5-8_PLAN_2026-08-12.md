# WP-060 increments 5–8 — the four startable unlocks, planned before built

Date: 2026-08-12. Author: Claude (Fable 5), working session under Nate's
direction: "Complete WP-060's four startable increments (reversal, solver
unlock, backup family, combination unlock)."

Untracked session artifact (docs/reviews/** stays out of every commit).
Each increment below is a separately reviewed change in the WP-035
route-decision shape the assignment names; this document is the review
artifact for the arc, written before the first line of code.

## Standing constraints the arc honors

- **One PR, one work package.** `verify-change-ownership` refuses a range
  declaring two packages. Every jointly-sequenced plan-body change is its
  own `Work-Package: WP-010` PR landing immediately before the WP-060
  increment that consumes it — "jointly sequenced" is adjacency and
  order, not co-residence.
- **The stopping condition trips on the first Rust merge** (currently
  pinned at `59ba1f6`, r10). The arc accepts the trip, merges its six
  Rust PRs in sequence, and closes with one r11 sitting re-taking all
  three acceptances on the final head, then the re-pin PR — the same
  economics the assignment's "riding whichever Rust change comes first"
  language embraces for the four code debts.
- **Mutation verification before proposal** for every new gate (the
  standing memory: run the mutants, don't reason about them).
- Gate runs happen in a temp worktree outside the repo with its own
  target dir; `cargo xtask ci` and `cargo xtask test --tier 1` real exit
  codes checked; traceability regenerated last.

## Increment 5 — SI-15 solver unlock (ADR-0023, spec 12.1.0). WP-060 only.

Replaces `SolveRefusal::MisalignedLegacyGrowth` with the decided
behavior. Design:

- **A deviation is an act.** The solver judges only boundaries the plan
  authors. `grow_extension` stops reading the start's alignment; the
  start is inherited, byte-identical, and reported as a typed inherited
  fact.
- **Authored-boundary policy, no fourth state.** Every authored boundary
  (create start, create end, grow end, shrink end) must be 1 MiB-aligned
  or coincident with a pre-existing structural edge (the next child's
  start, the host's extent end). Anything else refuses typed
  (`UnalignedAuthoredBoundary`, naming the boundary and the nearest
  conforming values) — the deviation-override vocabulary stays
  inexpressible. This tightens create/grow/shrink ends, which previously
  escaped policy entirely; that omission would otherwise be the fourth
  state.
- **Carriage.** The solver returns typed placement records
  (`Aligned` / `Coincident{edge}`) and inherited-fact records
  (`InheritedMisalignedStart{target, start}`); `Planned` carries them as
  `consequences` — typed values with rendered user-facing text, the
  planner-layer half of PART-009's consequence-text obligation. Body
  carriage waits for the consequence-text vocabulary (recorded boundary;
  ADR-0023 rejected typed hashed carriage).
- **Fixtures** (ADR-0023's verification list): 63-sector-start grow with
  aligned end (start byte-identical, inherited fact carried);
  grow-to-fill against a misaligned neighbor (coincident, recorded as
  coincident); the §11.2 authored/inherited split; the no-fourth-state
  property over solver outputs.
- SI-15 comment debts in `solve.rs`/`lib.rs`/`tests.rs` repaired in the
  same change.

## Increment 6 — PLAN-008 reversal (ADR-0022, spec 12.0.0). Two PRs.

**PR 6a (`Work-Package: WP-010`): plan body v2.** `SCHEMA_VERSION` 2, v1
refused at decode (MODEL-003's explicit-rejection arm, the journal's
precedent). Additions:

- Body gains required `reversal` linkage: `draft {plan_id, draft_hash}` |
  `impossible {statements}` | `reapply-forward {forward_plan_id}` (the
  draft's own field — the regress terminates as a reference). Statements
  are per-step, typed, closed reasons (`data-destroyed`,
  `prior-value-not-carried`, `no-destination-vocabulary`, …) — no free
  text (JRN-005 discipline).
- Step map gains `preconditions` (required array; kind
  `region-unoccupied {range}`), evaluated at the typed boundary against
  the binding snapshot: an overlapping fact refuses
  (`PreconditionFailed`). This is the two-time truthfulness re-check.
- Draft representation: `ReversalDraft` — plan-shaped body whose steps
  may spell `target` as an address **or** `step-output {index}` (the
  forward step's output; an address spelling for a created node is
  deliberately inexpressible in a draft that declares the reference).
  Resolution API takes the fresh capture **and** the forward plan:
  forward step's consumed range → the one node the capture places
  exactly there; zero or many refuses. `OperationPlan::assemble` gains
  the linkage parameter; **a severity-1 step in a plan whose linkage is
  not a draft refuses assembly and the boundary** (ADR-0022: no draft,
  no Reversible — structural).
- Acyclicity structural: the forward side carries a hash and no ID-only
  variant; the draft side carries an ID and no hash variant; a
  mutual-hash construction has no spelling.
- `schemas/domain/plan-body.md` updated; `body-vectors.json` regenerated
  as v2 (wipe plans gain `impossible` linkage + empty preconditions);
  `packages/canonical` parity suite updated.

**PR 6b (`Work-Package: WP-060`): emission.** `Planned` gains the
reversal output. Per operation, honestly:

- Sized **Create** → draft: one step destroying the placed range,
  target spelled `step-output(creating step)`, precondition
  `region-unoccupied(placed)` (metadata-only at emission; refuses once
  anything lands — ADR-0022's named fixture). Severity of the create
  step becomes **Reversible(1)** — claimable exactly because the
  truthful draft exists.
- Sized **Grow** → draft: shrink-back step, address target (the target
  pre-exists), precondition `region-unoccupied(extension)`. Grow's
  severity stays Disruptive (conservative-up, stated: the draft exists;
  the claim is not upgraded until the FS-grow reversibility story is
  measured).
- **Wipe** → per-step `data-destroyed`; **Shrink** → `data-destroyed`
  (the freed tail is destroyed; re-extension restores no bytes);
  **Label/Uuid** → `prior-value-not-carried` (the model carries no
  values to restore). `plan_set` (canonical ops only) → per-step
  statements.
- Draft determinism held as bytes (ADR-0022 v1); draft body hash in the
  forward linkage; draft plan ID derived deterministically from the
  forward plan ID (`plan_id ‖ "/reversal"` — deterministic, distinct).
- Tests: v1–v5 of ADR-0022's verification list (v6, the reversal's own
  authorization, is HLP-003/ADR-0021 surface territory — recorded as
  boundary, not claimable by a pure library).
- SI-19 comment debts in `crates/planner` repaired here.

## Increment 7 — SI-16 backup family (ADR-0024, spec 12.2.0). Two PRs.

**PR 7a (`Work-Package: WP-010`): plan body v3.** Additions:

- Step map gains required `class`: `ordinary` | `table-repair` (REC-001's
  typed family — the arm attaches to the step type, never an intent
  flag).
- Acknowledgment vocabulary gains `uncapturable-regions {table, regions}`
  (strictly ascending, non-overlapping, nonzero — the journal's
  region discipline). Constructor law: lawful only on a `table-repair`
  step whose covered node's authored table state is `Indeterminate`;
  unconstructible outside the family (ADR-0024 fixture 3's last clause).
- `IdentityBoundRestore`'s arm exists at last: lawful on a
  `table-repair` step covering a node whose table state is
  `Indeterminate` (ADR-0018's carried-closed kind, now constructible on
  exactly its decided arm).
- Schema doc, vectors v3, TS parity.

**PR 7b (`Work-Package: WP-060`): the family and the arms.**

- **SAFE-005 planner-half:** an ordinary mutating request whose target
  device's authored table state is `Indeterminate` refuses typed before
  any protection obligation is computed (fixture 4: PART-013 never
  reached).
- **Repair becomes plannable:** `Operation::Repair` targets a device
  whose facts carry a table-region child extent; the step is
  `table-repair` class, `written_table_extents` = exactly the table
  regions; simulation drops the stamp (post-repair state unestablished
  until a real capture — the wipe precedent) and changes nothing else.
  Missing table-region facts refuse (fail-closed, no invented regions).
- **Protection obligations computed, never stored:** `Planned` carries
  the derived obligation per affected table-bearing device — `Present` →
  parse-backup; `Absent` → journaled-determination (no acknowledgement,
  fixture 1); `Indeterminate` + repair family → raw-capture of exactly
  the write-target regions (fixture 2's planner half; the byte
  round-trip through REC-001's restore is WP-R100's, recorded as
  boundary); with the `uncapturable-regions` acknowledgement → the
  acknowledged-unpreserved arm naming those exact regions. Derived from
  the authored states at every computation — obligations are not body
  content (ADR-C4/0016 discipline; the journal's protection record is
  the durable artifact, already delivered by WP-070).

## Increment 8 — SI-17 combination unlock (ADR-0025, spec 12.3.0). WP-060 only.

- **The criterion, typed:** `InterruptionProfile`
  (`LandsEntirelyOrNot` | `RecoverableIntermediate` |
  `UnrestorableIntermediate`) with the flag derived from it — the
  partition fixtures pin journaled-copy → unflagged, in-place
  multi-sector rewrite → flagged (fixture 3, at the criterion level; no
  move/copy steps exist to emit yet).
- **Honest flag emission:** Wipe and the table-repair step carry
  `irreversible-after-start` (in-place destruction/rewrite —
  unrestorable intermediates); everything else planner-emitted today is
  unflagged.
- **The combination constructs** (fixture 1): severity-1 + flag with its
  reversal draft assembles through the sole constructors; the same
  construction without a draft refuses via increment 6's structural
  rule. The withheld-combination comments replaced by the decided
  behavior.
- **The coupling rule, unconstructible** (fixture 2): a typed
  cancellation-claim vocabulary (the PLAN-005-adjacent planner half,
  scoped to the rule): claiming `no-writes` requires
  before-first-write on a flagged step — the wrong claim has no
  constructor.
- **Ceremony binding** (fixture 4): a planner-side test consuming
  `partman-journal`'s `AuthorizationTier` derivation (dev-dependency):
  the severity-1-flagged plan derives `interactive-ceremony`, never the
  floor act. If the journal exposes no derivation function, the test
  pins the tier rule through whatever public seam exists, and the gap is
  recorded rather than papered.

## Sequencing and closure

Branch/PR order: 5 → 6a → 6b → 7a → 7b → 8, each merged on green with
checks given time to register (the #275/#277 lesson). Then the record
sweep (assignment increments prose, README row, CHANGELOG, traceability
regenerated, stale counts swept per the standing memory), the r11
sitting on the Proxmox host (next VMID after 9433 = 9434, runbook
`*-r10.sh` copied to `-r11`), and the re-pin PR.
