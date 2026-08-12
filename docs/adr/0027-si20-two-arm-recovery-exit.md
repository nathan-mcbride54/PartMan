# ADR-0027: The two RecoveryRequired exits are the two arms — roll-forward continues the original; distinct recovery disposes it

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-20_RECOMMENDATION_ROUND_2026-08-12.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.5.0 (minor under §0.1 — closing-prose additions only;
  argued in Decision, with the major counter-argument recorded)
- Work packages blocked: none newly — WP-070 does not exist as an
  assignment yet; this ADR records the obligations its creation must
  carry
- Requirement IDs: Section 8, REC-009 (amended context: Section 8's
  closing prose only); REC-010, HLP-001, HLP-003, HLP-005, JRN-002,
  JRN-003, JRN-006, UI-010, PLAN-005, ADR-0022 (read, none amended)
- Decision owners: Nate McBride

## Context

Section 8's closing prose states that recovery actions "are themselves
plans under this same contract," while its transition table moves the
*original* plan `RecoveryRequired → Executing` on "user selects a valid
roll-forward action (REC-009)." SI-20 filed the fork: a recovery action
that is its own plan has its own hash and lifecycle, so executing it
moves nothing on the original — which then has no exit to `Completed`
or `Cancelled` — while reading the → Executing edge as the recovery
plan executing *as* the original runs a different plan's steps under
the original's authorized hash, the substitution plan-hash binding
(HLP-001, HLP-003) exists to forbid.

## Safety analysis

**The two exits are the two arms.** REC-009 surfaces "valid
roll-forward or recovery actions" — a disjunction the state machine
already mirrors:

**Arm one: roll-forward continues the original plan.** Same plan hash,
same journal, execution resuming from the last durable checkpoint
through the existing → Executing edge. Its state derives from the
journal plus fresh re-discovery — JRN-003's existing rule, inherited
rather than added, which answers the re-verification objection by
citation. This is the one recovery act that is *not* its own plan; the
prose sentence over-generalized, and the amendment scopes it while
every other instance of it remains true.

**Arm two: any distinct recovery action is its own `OperationPlan`,
and selecting it disposes the original.** Restoring a table from its
PART-013 backup, repairing, completing the work another way: each is a
new plan with its own draft, validation, authorization, and lifecycle —
the prose's claim kept in full. Choosing one *is* accepting that the
original will not complete as declared: the acceptance the → Failed
trigger names, read as the selection implies, with the row standing
verbatim. The terminal record carries its honest effect summary
(`partial`, per journal), the full report, and a **journaled linkage**
naming the recovery plan, so "Failed, recovered by plan X" is one
reconstructable record chain. One user act may drive both records; they
remain two records.

**Disposal is durable before the recovery plan may apply.** The
original's Failed transition is journaled durably before the recovery
plan enters its apply path — JRN-002's transition-before-dependent-
action shape. On a shared bound device set this is structural rather
than procedural: HLP-005's one-plan-per-bound-device-set already makes
a recovery plan unexecutable while the original holds the devices. The
torn state the filing implies — original undisposed, second plan
running — is unreachable in the order this fixes, and a crash between
the records leaves original-Failed plus an unapplied recovery plan,
reconstructable from the journal alone.

**No `→ Cancelled` edge, deliberately.** Cancelled's semantics —
user-initiated stop with journaled unwind, effect possibly `no-writes`
— belong to the Executing era. A plan in RecoveryRequired has
interrupted writes behind it; the honest user-initiated terminal is
Failed with its report and effect summary. The user who acts on nothing
leaves the state persisting across restarts, exactly as the prose
already says.

**The vocabulary describes the plan, and the linkage carries the
story.** "Failed" names the plan's lifecycle outcome — it did not
complete its declared steps, which is true — while the effect summary,
report, and linkage carry the situation's outcome. UI-010 owns
rendering that as the success story it may well be; the state machine
does not rename facts to be kind.

**Scope boundaries, stated:** which acts on these edges require fresh
authorization — the roll-forward resume, the recovery plan's apply —
is SI-21's question, untouched in both directions. ADR-0022's reversal
plans reverse a *completed* apply and are not in the mid-apply recovery
population; a repair plan that happens to restore prior bytes is a
repair plan, constructed and validated as one.

**What a consumer and a plan may rely on:**

- Every plan leaves RecoveryRequired through one of exactly two edges,
  by a user act: roll-forward of itself, or Failed with a full report —
  and, where recovery continues elsewhere, a journaled linkage naming
  the recovery plan.
- No plan's steps ever execute under another plan's hash or
  authorization.
- At no instant does a recovery plan apply while the original is
  undisposed on the same device set.
- A restart reconstructs the whole chain — original terminal, linkage,
  recovery plan state — from the journal alone.

## Options considered

### Option (a) — the recovery plan executes "as" the original through the → Executing edge

Rejected without needing the adversarial round: a different plan's
steps under the original's authorized hash is the exact substitution
plan-hash binding and the journal's hash discipline exist to forbid.

### Option (b) — add exits or a terminal: → Completed on the recovery plan's success, → Cancelled, or `Superseded`

Rejected: → Completed couples two plans' lifecycles (the original
"completes" on work it did not do); `Superseded` pays a schema state, a
property-test surface, and a match arm in every consumer to rename a
fact the Failed-with-linkage record already carries; → Cancelled
imports unwind semantics into a post-interruption world where
`no-writes` is unclaimable.

### Option (c) — the two-arm reading, journaled linkage, disposal-first ordering (accepted)

Accepted, scoped as above. No state, edge, or trigger is added.

### Option (d) — reword the Failed row's trigger to name recovery selection

Rejected on economy with honesty preserved: it retexts a published
machine-readable table row — a semantic change to existing text, the
major class — to say what one prose sentence (selection is the
acceptance the row names) achieves at minor.

## Decision

Option (c), landed as spec 12.5.0's amendment to Section 8's closing
prose and nothing else. **SI-20 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** the transition table
rows, the terminal-state list, and "No other transitions exist" stand
verbatim; the additions scope one over-general prose sentence
(roll-forward is the one recovery act that is not its own plan — every
other instance of the sentence remains true) and state the linkage and
ordering rules. The counter-argument (scoping an existing sentence as
semantic change, the 3.1.0 caution) was weighed and is recorded so the
numbering is auditable; it was not taken because the sentence's claim
is narrowed for no case that was ever coherent — the one act it now
excludes was never executable as its own plan without breaking hash
binding.

## Consequences

- **Positive.** The filed no-exit gap dissolves with no new state
  machine surface; the torn state is unreachable by order everywhere
  and by structure on shared devices; the record chain makes recovery
  auditable end to end; the property-test obligation ("undeclared
  transitions unrepresentable") is unchanged in size.
- **Negative, accepted knowingly.** A plan whose situation was fully
  recovered by a follow-on plan still terminates as Failed — the
  honest lifecycle fact, with the linkage carrying the better news, and
  UI-010 owning the rendering. And disposal-before-apply means a user
  cannot keep the original "open" while experimenting with a recovery
  plan on the same devices; that serialization is HLP-005's, not new.
- **For WP-070, when its assignment is created.** The verification
  obligations below are this ADR's record precisely because no
  assignment document exists yet to carry the gate; the assignment's
  creation MUST import them. The linkage record's journal encoding
  lands with JRN-006's schema, jointly sequenced like the SI-16
  protection record.
- Nothing here is hash-visible: no plan-body field, no schema state,
  no new transition.

## Verification

Owned by WP-070 when it exists, recorded here so its assignment's
creation cannot omit them:

1. The machine-readable transition table's property tests prove exactly
   the published transitions representable — unchanged obligation, with
   the two-arm reading as documentation, not new edges.
2. The disposal ordering: a recovery plan's apply is unreachable while
   the original's Failed record is not durable; on a shared device set,
   structurally unreachable via HLP-005, tested as such.
3. The linkage chain: a Failed-by-recovery-selection terminal carries
   the recovery plan's ID, and a crash replay reconstructs the chain
   from the journal alone (JRN-003).
4. The roll-forward edge derives its state from journal plus fresh
   re-discovery — JRN-003's existing rule, tested on this edge by name.

## Revisit conditions

- SI-21's resolution assigns authorization posture to these edges; its
  round must preserve the two-arm topology or amend this ADR first.
- The machine-readable transition-table schema lands (Section 8's own
  publication requirement); if the two-arm reading cannot be carried as
  documentation on the existing rows, the reading is the part to keep.
- A recovery-action vocabulary richer than roll-forward-or-distinct-plan
  is ever designed; the disjunction here mirrors REC-009's current
  text, and a third class files its own round.
