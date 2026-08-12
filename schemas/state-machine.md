# The execution state machine, machine-readably

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` Section 8
- Owner: WP-070 (`docs/work-packages/WP-070.md`)
- Generated from `crates/statemachine`'s `published_markdown()` and
  held byte-fresh by the `the_published_table_is_byte_fresh` test:
  one source — the `Transition` variants the property tests prove
  equal to Section 8's table — three views (types, tests, this
  document). To regenerate, write that function's output over this
  file; the test arbitrates.

This document records a delivered vocabulary. It decides nothing: a
row exists here because Section 8 publishes it and the crate encodes
it, never because this document says so.

States: `Draft`, `Validated`, `AwaitingAuthorization`, `Revalidating`, `Protecting`, `Executing`, `Verifying`, `Completed`, `Paused`, `RebootPending`, `RecoveryRequired`, `Failed`, `Cancelled`.

Terminal states: `Completed`, `Failed`, `Cancelled` — every terminal
record carries an effect summary (`no-writes`, `partial`,
`complete`), held structurally by `TerminalRecord`.

| From | To | Trigger |
| --- | --- | --- |
| Draft | Validated | Validator passes |
| Validated | Draft | User edit, or invalidation (CONC-003) |
| Validated | AwaitingAuthorization | User/CLI submits apply |
| AwaitingAuthorization | Revalidating | Authorization granted (HLP-003) |
| AwaitingAuthorization | Cancelled | User declines, or validity window expires (PLAN-007) — effect `no-writes` |
| Revalidating | Protecting | Helper revalidation passes (HLP-002, PLAN-006) |
| Revalidating | Failed | Identity/topology mismatch — effect `no-writes` (ACC-007) |
| Protecting | Executing | Metadata/encryption backups complete and verified (PART-013, REC-011) |
| Protecting | Failed | Backup failure (SAFE-005) — effect `no-writes` |
| Executing | Verifying | Final step complete |
| Executing | Paused | User pause at a cancellable or checkpoint boundary |
| Executing | RebootPending | Declared reboot step reached |
| Executing | RecoveryRequired | Step failure with recovery actions, or interruption detected on restart |
| Executing | Cancelled | Cancel honored at a safe point (PLAN-005) after journaled unwind — effect `no-writes` or `partial` |
| Paused | Executing | User resumes; topology re-verified first |
| Paused | Cancelled | User cancels — effect per journal |
| Paused | RecoveryRequired | Topology changed while paused |
| RebootPending | Revalidating | Same plan hash resumes after boot (WIN-009) |
| RebootPending | RecoveryRequired | Resume impossible or state divergent |
| Verifying | Completed | Postconditions pass (UI-012) |
| Verifying | RecoveryRequired | Postcondition failure |
| RecoveryRequired | Executing | User selects a valid roll-forward action (REC-009) |
| RecoveryRequired | Failed | User accepts failure; full report generated |

No other transitions exist — structurally: an undeclared pair has no
`Transition` variant, and the Section 11.6 property test proves the
variant set equals this table exactly.

The two `RecoveryRequired` exits are the two arms (ADR-0027):
roll-forward continues the *original* plan — same hash, same
journal, state derived from journal plus fresh re-discovery
(JRN-003) — and is the one recovery act that is not its own plan;
accepting failure is the disposal arm, which selecting a distinct
recovery action *is*, with the journaled linkage and the
disposal-durable-before-apply ordering landing with the journal
increments. Interruption suspends an apply and only terminals end
it (ADR-0028); the re-entry edges continue the same apply under the
same journaled act, within the PLAN-007 window.
