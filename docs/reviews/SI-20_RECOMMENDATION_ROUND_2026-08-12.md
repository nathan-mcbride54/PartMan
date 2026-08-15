# SI-20 recommendation round — 2026-08-12

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-20 stays Later (WP-070) until a decision is recorded
through a WP-010 spec change with an ADR, the established shape. This is
an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-20, an early filing
with no options recorded. This round constructs the option space as well
as recommending from it.

---

## The conflict, made precise

> **Section 8 (prose):** `RecoveryRequired` persists across restarts
> until the user acts; recovery actions are themselves plans under this
> same contract.

> **Section 8 (table):** RecoveryRequired → Executing — "User selects a
> valid roll-forward action (REC-009)"; RecoveryRequired → Failed —
> "User accepts failure; full report generated." *No other transitions
> exist.*

> **REC-009:** Surface the last durable checkpoint and valid
> roll-forward or recovery actions after interruption.

The filing's fork: if a recovery action is its own plan — own hash, own
lifecycle, per the prose — then executing it moves nothing on the
*original* plan, which sits in RecoveryRequired with no exit to
`Completed` or `Cancelled`. But if the → Executing edge is read as the
recovery plan executing "as" the original, a different plan's steps run
under the original's authorized hash — exactly what plan-hash binding
(HLP-001, HLP-003) exists to forbid. Either the prose over-claims or the
table under-provides.

## Recommendation: the two exits are the two arms — roll-forward continues the original; anything else is its own plan whose selection disposes the original

**The table is complete under the reading that splits REC-009's actions
the way the architecture already splits plans. No row changes; the prose
states the split.** Concretely:

1. **A roll-forward action continues the original plan.** Same plan
   hash, same journal, execution resuming from the last durable
   checkpoint (REC-009, JRN-003 — whose recovery-state rule already
   requires journal-plus-fresh-re-discovery, so the edge inherits
   re-verification without new text). This is the existing
   RecoveryRequired → Executing edge, and it is *not* "a recovery
   action that is its own plan": it is the original plan finishing its
   own declared steps. The prose sentence over-generalized, and the
   amendment scopes it.
2. **Any distinct recovery action is its own `OperationPlan`,** with its
   own draft, validation, authorization, and lifecycle — the prose's
   claim, kept in full for this arm. Restoring a table from its
   PART-013 backup, repairing, completing the work another way: each is
   a new plan under the same contract.
3. **Selecting a distinct recovery action disposes the original through
   the existing Failed edge.** Choosing a different plan to fix the
   situation *is* accepting that this plan will not complete as
   declared — the → Failed trigger's "user accepts failure," read as
   the selection implies. The terminal record carries its honest effect
   summary (`partial`, per journal), the full report, and a **journaled
   linkage** to the recovery plan's ID, so the story is one record
   chain: original Failed-partial → recovery plan's own lifecycle. One
   user act may drive both records; they remain two records.
4. **Disposal is durable before the recovery plan may apply.** The
   original's Failed transition is journaled durably before the
   recovery plan enters its apply path — the JRN-002 shape (transition
   durable before the dependent action), and on shared device sets not
   even new machinery: HLP-005's one-plan-per-bound-device-set already
   makes a recovery plan unexecutable while the original holds the
   devices. The torn state the filing implies (original in
   RecoveryRequired, second plan running) is unreachable in the order
   this fixes.
5. **No `→ Cancelled` edge is needed, and none is added.** Cancelled's
   semantics — user-initiated stop with journaled unwind, effect
   possibly `no-writes` — belong to the Executing era. A plan in
   RecoveryRequired has interrupted writes behind it; the honest
   user-initiated terminal is Failed with its report and effect
   summary. The user who walks away acts on nothing, and the state
   persists across restarts exactly as the prose already says.
6. **"No other transitions exist" stands verbatim.** The resolution
   adds no state, no edge, and no trigger; it defines which existing
   edge each user act takes.

## What a consumer and a plan may rely on

- Every plan leaves RecoveryRequired through one of exactly two edges,
  by a user act: roll-forward of *itself*, or Failed with a full
  report — and, where recovery continues elsewhere, a journaled linkage
  naming the recovery plan.
- No plan's steps ever execute under another plan's hash or
  authorization.
- A recovery plan is an ordinary plan: validated against a fresh
  capture, authorized at its own tier (whatever SI-21 decides about
  which acts need fresh authorization — untouched here), journaled on
  its own lifecycle.
- At no instant does a recovery plan apply while the original is
  undisposed on the same device set — by journal order everywhere, and
  by HLP-005 structurally on shared devices.

## The adversarial round

**Attack 1 — "calling a successfully recovered situation 'Failed'
punishes the user; add a `Superseded` terminal."** Rejected on what the
vocabulary describes: the state names the *plan's* lifecycle outcome —
this plan did not complete its declared steps, which is true — while
the effect summary, report, and linkage carry the situation's outcome.
A new terminal costs a schema state, a property-test surface, and a
match arm in every consumer, to rename a fact; and UI-010 already owns
making the display actionable and humane. The linkage is what makes
"Failed, recovered by plan X" renderable as the success story it is.

**Attack 2 — "one act, two records: a crash between them leaves the
torn state."** Sustained as the round's sharpest finding and absorbed
as point 4: the disposal-first ordering is normative, in the JRN-002
shape, and HLP-005 already enforces it structurally wherever the
recovery plan touches the same devices. A crash between records leaves
original-Failed plus a recovery plan that has not applied — clean, and
reconstructable from the journal alone (JRN-003).

**Attack 3 — "roll-forward → Executing skips the re-verification the
Paused → Executing edge names."** Refuted by citation rather than
addition: JRN-003 already requires recovery state to derive from the
journal *plus fresh re-discovery*, so the edge inherits re-verification
from existing text. The round adds nothing and needed to add nothing —
naming it in the ADR prevents the misreading.

**Attack 4 — "this decides SI-21's authorization question."** Refuted
by scope, stated: which acts on these edges require fresh authorization
— roll-forward resume, the recovery plan's apply — stays SI-21's
entirely. This round fixes state topology and disposal ordering; it
neither grants nor denies any authorization reuse.

**Attack 5 — "a reversal plan (ADR-0022) used as the recovery action
contradicts its completed-apply boundary."** Refuted by vocabulary: the
recovery actions REC-009 surfaces for a mid-apply interruption are
roll-forward and repair-class plans; ADR-0022's reversal reverses a
*completed* apply and is not in this population, and nothing here
touches that boundary. A repair plan that happens to restore prior
bytes is a repair plan, constructed and validated as one.

**Attack 6 — "the Failed row's trigger text says 'accepts failure,'
not 'selects recovery' — the reading retexts the row by stealth."**
Sustained as a drafting risk and answered in the open: the amendment
states in prose that selecting a distinct recovery action *is* the
acceptance the row names, and the row stands verbatim. The alternative
— rewording the row — was rejected because it retexts a published
machine-readable table row (a semantic change to existing text, the
major class) to say what one prose sentence says at minor cost.

## Rejected, and why — to be recorded with the decision

- **(a) Read → Executing as the recovery plan executing "as" the
  original.** Breaks plan-hash binding: a different plan's steps under
  the original's authorized hash, the exact substitution HLP-001/HLP-003
  and the journal's hash discipline exist to forbid. Rejected without
  needing the adversarial round.
- **(b) Add exits or a terminal — RecoveryRequired → Completed on the
  recovery plan's success, → Cancelled, or `Superseded`.** Couples two
  plans' lifecycles (the original "completes" on work it did not do, or
  gains a terminal that duplicates Failed-with-linkage), adds state
  machine surface the property tests must then prove unreachable-except,
  and pays a schema state for a renaming. Attack 1's costs.
- **(c) is the recommendation** — the two-arm reading, journaled
  linkage, disposal-first ordering.
- **(d) Reword the Failed row's trigger to name recovery selection.**
  Attack 6: retexts a machine-readable table row (major) to achieve what
  prose achieves at minor; rejected on economy with the honesty
  preserved by the explicit acceptance-by-selection sentence.

## Deliberately not decided

SI-21 (authorization reuse on resume and roll-forward — both edges'
authorization posture); SI-22 and SI-23 (untouched); the recovery-action
UX and REC-009's surfacing format; the machine-readable transition
table's schema encoding (WP-070-era, per Section 8's own publication
requirement); the linkage record's journal encoding (JRN-006, WP-070's,
jointly sequenced exactly as the SI-16 protection record).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0027 is the next free number; reservation PR
before resolution PR, the established shape), amends **Section 8's
closing prose only** — the transition table rows, the terminal-state
list, and "No other transitions exist" all stand verbatim; the prose
gains the two-arm scoping, the acceptance-by-selection sentence, the
journaled linkage, and the disposal-durable-before-apply ordering —
bumps **minor** (12.5.0: additions; the over-general prose sentence is
scoped, not withdrawn — roll-forward is the one recovery act that is
not its own plan, stated as a scoping of "recovery actions are
themselves plans" whose every other instance remains true), and moves
SI-20 to Resolved. The major counter-argument (scoping an existing
sentence as semantic change, the 3.1.0 caution) is recorded for the
decision to overrule with.

**No re-attribution PR follows**: no WP-070 assignment document exists
yet to cite the gate. The verification obligations land in that
assignment when it is created, recorded in the ADR so the creation
cannot omit them.

Verification obligations for the ADR, owned by WP-070 when it exists:

1. The property tests over the machine-readable table prove exactly the
   published transitions representable — unchanged obligation, now with
   the two-arm reading as documentation, not as new edges.
2. The disposal ordering: a recovery plan's apply is unreachable while
   the original's Failed record is not durable — and on a shared device
   set, structurally unreachable via HLP-005, tested as such.
3. The linkage record: a Failed-by-recovery-selection terminal carries
   the recovery plan's ID; a crash replay reconstructs the chain from
   the journal alone.
4. The roll-forward edge derives its state from journal plus fresh
   re-discovery (JRN-003's existing rule, tested on this edge by name).
