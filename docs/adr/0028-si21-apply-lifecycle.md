# ADR-0028: An authorization act authorizes one apply — a journal-continuous lifecycle that interruption suspends and only terminals end

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-21_RECOMMENDATION_ROUND_2026-08-12.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 12.6.0 (minor under §0.1 — additions defining a term the
  ladder used; argued in Decision, with the major counter-argument
  recorded)
- Work packages blocked: none newly — WP-070 does not exist as an
  assignment yet; this ADR records the obligations its creation must
  carry (the ADR-0027 precedent)
- Requirement IDs: HLP-003 (amended); HLP-004, HLP-005, PLAN-006,
  PLAN-007, WIN-009, Section 8, JRN-001, JRN-003, ADR-0021, ADR-0027
  (read, none amended)
- Decision owners: Nate McBride

## Context

Section 8's table reaches `Executing` from `RecoveryRequired`, and
`Protecting` from `Revalidating` after `RebootPending`, without passing
`AwaitingAuthorization`; `Paused → Executing` likewise resumes directly.
SI-21 filed the conflict: a roll-forward or post-reboot resume writes
storage under an authorization granted before the interruption —
possibly after the helper exited, which HLP-005 permits — and WIN-009's
"resume the same cryptographically bound plan" suggests reuse is
intended, which reads as exactly the retained grant HLP-003 forbids.

The filing predates ADR-0021 and reads sharper against it: the floor
act is single-use — "one act authorizes one apply of one plan, never a
second plan and never a second apply" — so if a resume is a second
apply, every re-entry edge violates the ladder. The question is one
definition deep: what is "an apply"? The ladder used the word without
defining it, and this filing is where the gap surfaces.

## Safety analysis

**An apply is one execution lifecycle of one plan.** It runs from its
authorization act to a terminal state — `Completed`, `Failed`, or
`Cancelled` — and is identified by the plan hash and an unbroken
journal chain from the act's record to the current position, where
"unbroken" means what the journal already guarantees: JRN-001's
monotonic sequence numbers with the torn-tail rule bounding detection.
Pause, a declared reboot, and a recovery interruption *suspend* an
apply; only terminals end it. Resume and roll-forward therefore
continue the *same* apply under the *same* journaled, hash-bound,
single-use act — consumed once, at the apply's start, and never again.
Nothing is used twice, so nothing is reused.

**The authorization is a journal fact, never process state — which
dissolves the helper-exit worry.** ADR-0021's floor act is journaled;
JRN-003 reconstructs execution state from the journal plus fresh
re-discovery; HLP-005's idle exit discards nothing that was ever
supposed to persist in a process. A helper instance that restarts and
continues a journal-continuous apply holds no retained grant: the
journal holds the apply, and the helper holds nothing. What HLP-003's
caching sentence forbids is an *approval outliving its apply* to
authorize another; it does not forbid an *apply outliving an
interruption*.

**PLAN-007's window bounds every entry to the apply path, and its
existing re-approval sentence is the fresh-act boundary.** A resume or
roll-forward within the plan's validity window continues under the
original act. A re-entry past expiry is rejected exactly as HLP-004
already requires ("rejecting expired or stale plans"), and PLAN-007's
own sentence — re-approval after expiry requires re-validation against
a fresh snapshot — names the route back: a fresh authorization act for
the same continuing apply. One-act-one-apply is a ceiling on an act's
reach, never a floor on their count: two acts for one long-interrupted
apply is compliant; one act for two applies never is.

**Each re-entry edge keeps its named verification, untouched.** Paused
re-entry re-verifies topology; RebootPending re-entry runs full
revalidation (HLP-002, PLAN-006); roll-forward derives from journal
plus fresh re-discovery (JRN-003, per ADR-0027). Authorization
continuity never substitutes for revalidation, and revalidation never
substitutes for authorization.

**WIN-009 reads as continuity, not as a grant.** "Resume the same
cryptographically bound plan" is the same-apply requirement stated from
the Windows side: same hash, same journal chain. The user authorized an
apply whose body declared its reboot span (Section 6 carries reboot
requirements; UI-005 displayed them before Apply); re-prompting at
each boot would ask the user to confirm information already confirmed,
with no new fact in front of them.

**The compromised-boot objection is answered by locating what
protects.** A resume requires the journaled act, the exact plan hash,
an unbroken chain (JRN-001 checksums, torn-tail truncation), and full
revalidation against a fresh capture. Forging a resume means forging
the journal — JRN-001/SEC territory that a fresh prompt would not
repair, since an attacker who owns the boot environment owns the
prompt too. The authorization model is not the defense surface for
journal integrity.

**What a consumer and a plan may rely on:**

- Every write, on every edge, traces through the journal to exactly one
  authorization act bound to this plan hash and consumed by this
  apply — or to a fresh act taken after expiry under PLAN-007's
  existing rule.
- No approval ever authorizes a second apply or a second plan; no
  helper process ever holds an authorization in memory that the
  journal does not hold more authoritatively.
- An interrupted apply older than its validity window cannot resume
  silently.
- The ceremony's severity/flag scaling is untouched: a past-window
  re-entry's fresh act is at the plan's tier, same as ever.

## Options considered

### Option (a) — every resume and roll-forward re-passes `AwaitingAuthorization`

Rejected. A prompt firing at every boot of a multi-reboot migration,
showing nothing the user has not already confirmed, trains the click
that later approves something real — the rubber-stamp economics this
register has priced in the SI-39, SI-18, SI-16, and SI-17 rounds. It
also requires new transition edges into `AwaitingAuthorization` from
three states, breaking "No other transitions exist" days after
ADR-0027 preserved it verbatim, for a ceremony with no new information
to present.

### Option (b) — authorization as retained helper state across interruptions

Rejected without needing the adversarial round: it contradicts
HLP-003's caching sentence outright and HLP-005's idle exit.

### Option (c) — the apply-lifecycle definition, the journal as the authorization's home, PLAN-007 bounding every re-entry (accepted)

Accepted, scoped as above.

### Option (d) — severity-scaled resume prompting

Rejected: complexity without principle. The original act already
scaled with severity and flags (ADR-0021's ladder), the window rule
covers staleness uniformly, and a severity-split resume rule would
make a plan's *interruption* change its authorization semantics — a
second encoding of a dimension the ladder already carries.

## Decision

Option (c), landed as spec 12.6.0's amendment to HLP-003 and only
HLP-003. **SI-21 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** the ladder's every
pre-existing sentence stands verbatim; the apply-lifecycle definition
is an addition defining a term ADR-0021's text used and the filing
showed was undefined; PLAN-007, HLP-004, HLP-005, WIN-009, Section 8's
table, and the JRN-* contract are untouched and read naturally under
the definition. The counter-argument (a first definition fixes
semantics other text depended on — the 3.1.0 caution) was weighed and
is recorded so the numbering is auditable; it was not taken because
§0.1's rule turns on what happens to existing requirement text, and
none changes.

## Consequences

- **Positive.** The three re-entry edges are compliant by definition
  rather than by exception; the helper-exit worry dissolves into the
  journal's existing guarantees; freshness has a uniform boundary in
  PLAN-007's existing machinery; no new state, edge, prompt, or
  schema.
- **Negative, accepted knowingly.** A recovery stale past its validity
  window takes one re-approval against a fresh snapshot — PLAN-007
  doing its job on the rare stale case. And a multi-reboot apply
  writes storage at boots where no human is present at that moment —
  by the user's prior authorization of exactly that declared span,
  displayed before Apply.
- **Fed forward, undecided.** The authorization act's journal record
  is recovery-critical: SI-22's retention round must weigh it in the
  class of records recovery depends on.
- **For WP-070, when its assignment is created.** The verification
  obligations below are this ADR's record because no assignment
  document exists yet; the assignment's creation MUST import them (the
  ADR-0027 precedent).
- Nothing here is hash-visible: no field, no schema, no transition.

## Verification

Owned by WP-070 when it exists, recorded here so its assignment's
creation cannot omit them:

1. A resume on each of the three re-entry edges traces to the original
   act through an unbroken journal chain, and a broken chain refuses.
2. A re-entry past the PLAN-007 window is rejected, and a fresh act
   against a fresh snapshot readmits the same apply — two acts, one
   apply, journaled as such.
3. One act can never admit a second apply of the same plan or any
   apply of another plan — ADR-0021's test, extended to the resume
   path.
4. A helper restart mid-apply holds no authorization state the journal
   does not: the hand-forged in-memory-grant test, the ADR-0012
   pattern applied to process state.

## Revisit conditions

- SI-22's retention decision constrains which journal records may age
  out; if the authorization record's retention cannot satisfy both
  that decision and obligation 1 above, the two rounds reconcile
  before either ships machinery.
- PLAN-007's window semantics change; the re-entry boundary here reads
  the current default-24-hour/maximum-7-day design.
- A re-entry class is ever added to Section 8 (a new suspension
  state); it inherits this definition by default, and a class that
  cannot files its own round.
