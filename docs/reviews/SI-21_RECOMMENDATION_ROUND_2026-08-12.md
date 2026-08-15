# SI-21 recommendation round — 2026-08-12

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-21 stays Later (WP-070) until a decision is recorded
through a WP-010 spec change with an ADR, the established shape. This is
an untracked session artifact under `docs/reviews/**` (WP-000); the
register's own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-21, an early filing
with no options recorded. This round constructs the option space as well
as recommending from it.

---

## The conflict, made precise

> **SI-21's filing:** The table reaches `Executing` from
> `RecoveryRequired`, and `Protecting` from `Revalidating` after
> `RebootPending`, without passing `AwaitingAuthorization`. So a
> roll-forward or post-reboot resume writes storage under an
> authorization granted before the interruption — possibly after the
> helper exited, which HLP-005 permits. WIN-009 suggests reuse is
> intended; that is exactly the retained grant HLP-003 forbids.

> **HLP-003 (as ADR-0021 landed it):** every apply requires a fresh
> floor act — single-use, "one act authorizes one apply of one plan,
> never a second plan and never a second apply" — journaled, bound to
> the exact plan hash, valid only inside the plan's PLAN-007 window;
> the interactive ceremony at ≥ Disruptive or any flag; "Cached,
> session-wide, or remembered approvals MUST NOT exist for these
> severities."

> **WIN-009:** Reboot/offline operations must resume the same
> cryptographically bound plan.

Three re-entry edges continue writing without re-passing
`AwaitingAuthorization`: `Paused → Executing` (topology re-verified
first), `RebootPending → Revalidating` (WIN-009, same plan hash, full
revalidation), and `RecoveryRequired → Executing` (ADR-0027's
roll-forward arm, JRN-003's fresh re-discovery inherited). The filing
predates ADR-0021 and reads sharper against it: the floor act is
*single-use*, so if a resume is a second apply, every one of these
edges violates the ladder. The question is one definition deep: **what
is "an apply"?**

## Recommendation: an authorization authorizes one apply, and an apply is a journal-continuous execution — interruption does not end it, a terminal state does

**No reuse occurs on any of the three edges, because nothing is used
twice.** Concretely:

1. **An apply is one execution lifecycle of one plan:** from the
   authorization act through the pipeline to a terminal state,
   identified by the plan hash and an unbroken journal chain from the
   act's record to the current position (JRN-001's monotonic sequence
   and torn-tail rule are what "unbroken" means). Pause, a declared
   reboot, and a recovery interruption suspend the apply; only
   `Completed`, `Failed`, or `Cancelled` end it. Resume and
   roll-forward continue *the same apply* under *the same* journaled,
   hash-bound, single-use act — which was consumed once, at the apply's
   start, and is not consumed again.
2. **The authorization is a journal fact, not process state — which
   dissolves the helper-exit worry.** ADR-0021's floor act is
   journaled; JRN-003 reconstructs execution state from the journal
   plus fresh re-discovery; HLP-005's idle exit discards nothing that
   was ever supposed to persist in a process. A helper instance that
   restarts and continues a journal-continuous apply holds no retained
   grant — the journal holds the apply, and the helper holds nothing.
   What HLP-003's caching sentence forbids is an *approval outliving
   its apply* to authorize another; it does not forbid an *apply
   outliving an interruption*.
3. **PLAN-007's window bounds every entry to the apply path, and its
   existing re-approval sentence is the fresh-act boundary.** Resume or
   roll-forward within the plan's validity window continues under the
   original act. A re-entry past expiry is rejected exactly as HLP-004
   already requires ("rejecting expired or stale plans"), and
   PLAN-007's own sentence — re-approval after expiry requires
   re-validation against a fresh snapshot — names the route back: a
   fresh authorization act for the same continuing apply. ADR-0021's
   "one act, one apply" is a ceiling on acts' reach, not a floor on
   their count: two acts for one long-interrupted apply is compliant;
   one act for two applies never is.
4. **Each edge keeps its named verification, untouched.** Paused
   re-entry re-verifies topology; RebootPending re-entry runs full
   revalidation (HLP-002, PLAN-006); roll-forward derives from journal
   plus fresh re-discovery (JRN-003, per ADR-0027). This round adds no
   verification and weakens none — authorization continuity never
   substitutes for revalidation, and revalidation never substitutes for
   authorization.
5. **WIN-009 reads as continuity, not as a grant.** "Resume the same
   cryptographically bound plan" is the same-apply requirement stated
   from the Windows side: same hash, same journal chain. The user
   authorized an apply whose body declared its reboot span (Section 6
   carries reboot requirements; UI-005 displayed them before Apply) —
   the act covered the declared span, and re-prompting at each boot
   would ask the user to confirm information they already confirmed,
   with no new fact in front of them.

## What a consumer and a plan may rely on

- Every write, on every edge, traces through the journal to exactly one
  authorization act bound to this plan hash, consumed by this apply —
  or to a fresh act taken after expiry under PLAN-007's existing rule.
- No approval ever authorizes a second apply or a second plan
  (ADR-0021's single-use, unchanged); no helper process ever holds an
  authorization in memory that the journal does not hold more
  authoritatively.
- An interrupted apply older than its validity window cannot resume
  silently — the re-entry is rejected and re-approval is the only
  route, against a fresh snapshot.
- The ceremony's severity/flag scaling is untouched: the fresh act a
  past-window re-entry takes is at the plan's tier, same as ever.

## The adversarial round

**Attack 1 — "the reboot case writes with no human present; a
compromised boot environment 'resumes' whatever it likes."** Refuted by
locating what protects. A resume requires the journaled act, the exact
plan hash, an unbroken journal chain (JRN-001 checksums, torn-tail
truncation), and full revalidation against a fresh capture. Forging a
resume means forging the journal — JRN-001/SEC territory that a fresh
prompt would not repair, since an attacker who owns the boot
environment owns the prompt too. The authorization model is not the
defense surface for journal integrity, and pretending it is would buy
rubber-stamp friction instead of integrity.

**Attack 2 — "the window rule strands long recoveries: a
RecoveryRequired that persists past day seven can never roll
forward."** Sustained as a real property and accepted deliberately —
it is PLAN-007's existing design doing its job. The state persists
(Section 8's prose), the re-entry takes a fresh act against a fresh
snapshot, and the alternative — an authorization of unbounded
temporal reach — is precisely the remembered approval HLP-003 exists
to forbid. The cost lands as one re-approval on the rare stale
recovery, not as a prompt on every resume.

**Attack 3 — "'journal-continuous' is a new load-bearing term doing
undefined work."** Sustained as a drafting demand and answered in
point 1: the term is defined by existing machinery — same plan hash,
unbroken JRN-001 sequence from the act's record to the resume point,
with the torn-tail rule bounding "unbroken." No new journal semantics
are introduced; the definition names what the journal already
guarantees.

**Attack 4 — "option (a) is safer: re-prompt on every resume, and
users who find it tedious can avoid pausing."** Rejected on the
rubber-stamp economics this register has priced four times now
(SI-39, SI-18, SI-16, SI-17): a prompt that fires on every boot of a
multi-reboot migration, showing nothing the user has not already
confirmed, trains the click that later approves something real. It
also requires new transition-table edges into `AwaitingAuthorization`
from three states — breaking "No other transitions exist" days after
ADR-0027 preserved it verbatim — for a consent ceremony with no new
information to present.

**Attack 5 — "severity should scale the resume rule: destructive
plans re-prompt, label edits don't."** Rejected as complexity without
principle: the original act already scaled with severity and flags
(ADR-0021's ladder), the window rule covers staleness uniformly, and
a severity-split resume rule would mean a plan's *interruption*
changes its authorization semantics — a second encoding of severity
the ladder already carries.

**Attack 6 — "SI-22 can delete the journal segment holding the act's
record; then the resume cannot prove its authorization."** Sustained
as a genuine interaction and routed rather than decided: the
authorization record joins the class of records recovery depends on,
which is exactly SI-22's subject. This round feeds that fact forward
to SI-22's eventual round and decides nothing about retention.

## Rejected, and why — to be recorded with the decision

- **(a) Every resume and roll-forward re-passes
  `AwaitingAuthorization`.** Attack 4: rubber-stamp training, new
  transition edges breaking a sentence preserved verbatim through
  seven resolutions, and no new information in front of the user.
- **(b) Authorization as retained helper state across interruptions.**
  Contradicts HLP-003's caching sentence outright and HLP-005's idle
  exit; rejected without needing the adversarial round.
- **(c) is the recommendation** — the apply-lifecycle reading, the
  journal as the authorization's home, PLAN-007's window bounding
  every re-entry.
- **(d) Severity-scaled resume prompting.** Attack 5: a second
  encoding of a dimension the ladder already carries, keyed to the
  accident of interruption.

## Deliberately not decided

SI-22 (journal retention — with the fed-forward fact that the
authorization record is recovery-critical); SI-23; journal integrity
mechanisms (JRN-001's, as they stand); the transition table (untouched,
again); WIN-009's platform mechanics beyond its reading; any UI
presentation of resume (UI-011's).

## If accepted, the mechanics

WP-010 files the ADR (ADR-0028 is the next free number; reservation PR
before resolution PR, the established shape), amends **HLP-003 only** —
the ladder's text stands verbatim, gaining the apply-lifecycle
definition (an apply runs from its act to a terminal state, identified
by plan hash and unbroken journal chain; interruption suspends, only
terminals end; re-entry within the window continues under the original
act, re-entry past it is rejected into PLAN-007's existing re-approval
route; the caching prohibition forbids approvals outliving their
apply, not applies outliving interruptions) — bumps **minor** (12.6.0:
additions defining a term the ladder used and the filing showed was
undefined; PLAN-007, HLP-004, HLP-005, WIN-009, and Section 8 all
stand verbatim and read naturally under the definition), and moves
SI-21 to Resolved. The major counter-argument is recorded for the
decision to overrule with. **No re-attribution PR follows** — no
WP-070 assignment exists; the ADR records the verification obligations
so that assignment's creation cannot omit them (the SI-20 precedent).

Verification obligations for the ADR, owned by WP-070 when it exists:

1. A resume on each of the three edges traces to the original act
   through an unbroken journal chain, and a broken chain refuses.
2. A re-entry past the PLAN-007 window is rejected, and a fresh act
   against a fresh snapshot readmits the same apply — two acts, one
   apply, journaled as such.
3. One act can never admit a second apply of the same plan or any
   apply of another plan (ADR-0021's test, extended to the resume
   path).
4. A helper restart mid-apply holds no authorization state the journal
   does not: the hand-forged in-memory-grant test, the ADR-0012
   pattern applied to process state.
