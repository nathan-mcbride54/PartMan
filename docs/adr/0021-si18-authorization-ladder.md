# ADR-0021: Authorization is a two-tier ladder — a floor act for every apply, the interactive ceremony at Disruptive or any flag

- Status: Accepted
- Date: 2026-08-11. Accepted by Nate McBride the same day, by delegation
  in the session that ran the recommendation round ("I don't mind you
  picking a side — file it as Accepted"), the delegation recorded here as
  the acceptance basis
  (`docs/reviews/SI-18_RECOMMENDATION_ROUND_2026-08-11.md`, an untracked
  session artifact; this ADR restates everything load-bearing from it).
- Spec version: 11.2.0 (minor under §0.1 — additions only; argued in
  Decision, with the major counter-argument recorded)
- Work packages blocked: WP-040's authorization vocabulary and WP-070's
  helper authorization behavior (SI-18 resolved; WP-040's transport route
  decisions are separate gates and unchanged)
- Requirement IDs: SAFE-002, HLP-003, HLP-002, PLAN-004, PLAN-007,
  SAFE-003, CAP-007, RPC-001, UI-011, Section 0.2, Section 8
- Decision owners: Nate McBride

## Context

SAFE-002 confines privileged behavior to exactly two contexts, the first
being "the platform helper executing a validated plan after fresh,
explicit user authorization (HLP-003)." HLP-003 attaches fresh interactive
authorization to plans of severity ≥ Disruptive only, and its caching
prohibition reads "for these severities." A severity-1 plan (Reversible —
fully undoable via an emitted reversal plan, PLAN-004) still writes
storage and still needs privilege. SI-18 filed the conflict under Section
0.2: under one reading every privileged apply requires the full
interactive authorization and HLP-003's threshold is dead text; under the
other, authorization exists only at severity ≥ 2, SAFE-002's context 1 is
false of every severity-1 apply, and the caching sentence's complement
arguably licenses a remembered approval for privileged writes.

Two secondary defects sat in the same texts. PLAN-004 states that
"authorization requirements (HLP-003) key off severity plus flags," and
HLP-003's text read flags nowhere — a gap that is not hypothetical,
because adding a LUKS keyslot is fully reversible (severity 1) and
touches keys (`security-sensitive`), so a severity-only ladder would give
a key injection the lightest authorization in the product. And the
caching prohibition's "for these severities" left the lower severities'
caching posture undefined rather than merely unstated.

The register's entry also named what the answer decides: whether the plan
carries an authorization-requirement field distinct from severity. WP-040
increment 4 shipped its authentication skeleton with no authorization
vocabulary at all, gated by name on this issue.

## Safety analysis

**SAFE-002 is untouched, and the conflict dissolves in HLP-003's
mechanism vocabulary.** SAFE-002 is a Section 3 constraint; §0.2 gives it
precedence, and SI-38's resolution already recorded that bending SAFE-002
to satisfy a lower section inverts that order. Its words — fresh,
explicit, user, authorization — are all load-bearing and all preserved.
What it never said is *interactive* or *OS-mediated*: that is HLP-003's
mechanism ladder, and the two texts stop conflicting the moment HLP-003
states the ladder's lower rung instead of leaving it silent.

**The floor tier makes SAFE-002's sentence true at every severity.**
Every apply — severity 0 included — requires a fresh, explicit
authorization act: performed by the RPC-001-authenticated user, naming
the exact plan hash, single-use (one act, one apply of one plan), valid
only within the plan's PLAN-007 window, journaled, and never cached,
session-wide, or remembered. The act is distinct from channel
authentication and separately testable: RPC-001 peer verification
establishes *who is connected*; the floor act records *that this user
explicitly requested this plan, now*. A connected, fully authenticated
client that sends apply-plan without the plan-hash-bound act is refused.

**The act may be programmatic, deliberately.** A scripted CLI apply
naming the plan hash is a fresh explicit act by the authenticated user.
This keeps SAFE-003's unattended/scripted-apply clause a live population
— that clause already refuses unattended apply on weak-identity targets
without a recorded override, and severity-1 is fully-undoable by its own
definition, journaled, and carries an emitted reversal plan (PLAN-008).
The adversarial round sustained "scripted severity-1 writes happen with
no human present" as a real property and accepted it: the alternative
forecloses a population an existing requirement contemplates, and a spec
that contemplates a population in one requirement while forbidding it in
another files its own next conflict.

**The ceremony tier stands verbatim and gains the flags PLAN-004
promised.** Fresh interactive OS-mediated authorization (polkit
`auth_admin`, the documented macOS APIs, the ADR-W1 consent) binds at
severity ≥ Disruptive — the pre-existing sentence, unchanged — and now
also binds any plan carrying a step flag, regardless of severity.
Flags-nonempty is deliberately the closed rule rather than a
`security-sensitive` carve-out: the other flags
(`irreversible-after-start`, `requires-offline`, `requires-reboot`,
`requires-rescue`) describe conditions disruptive or worse in substance,
and enumerating a subset invites the next gap. A flagged plan can never
be applied unattended — fail-closed, and the keyslot case takes the
ceremony.

**The tier is computed by the observer that is trusted.** HLP-002 already
makes client-provided validation output an untrusted hint; the
authorization tier inherits that posture. The helper derives the tier
from its own recomputed severity and flags, so a forged or buggy
client-computed severity cannot lower the required authorization — an
assertion made testable in Verification, not assumed.

**Section 8 is untouched.** AwaitingAuthorization remains on every apply
path — the floor act is what a severity-1 plan awaits — so the
transition table needs no severity-conditional bypass edge to specify,
test, and defend.

**What a consumer and a plan may rely on:**

- Every apply, any severity: the journal carries a fresh authorization
  act bound to the exact plan hash; no apply proceeds from connection
  standing, cached approval, or session state alone; one act never
  applies two plans and never applies one plan twice.
- A severity-1 apply: the RPC-001-authenticated user explicitly requested
  *this* plan. Not that a human watched a prompt — that is the ceremony
  tier's guarantee, and consumers may not read it downward.
- A severity ≥ Disruptive or flagged apply: a human completed the
  platform's interactive ceremony for this exact plan hash, freshly,
  with no retained grant.
- All tiers: the enforced tier derives from the helper's own
  recomputation; no client claim participates.

## The register's named question: no authorization-requirement field

The plan carries **no** authorization-requirement field. The requirement
is a total function of body content the plan already carries — severity
and flags — recomputed by the helper at enforcement. A stored copy would
be a second encoding of the same fact, creating an agreement obligation
(field versus recomputation) with no safety the recomputation does not
already provide: ADR-0016's lesson, which reached its safety property by
making client claims unrepresentable, is reached here with no field at
all. A client-assertable authorization is also exactly what CAP-007 and
WP-040's charter forbid. Where the UI needs the tier for its
authorization-wait display (UI-011), the helper reports its computed tier
in the validate-plan response — response data, never plan body, never
client-authored.

Consequence for WP-040: the register gate lifts with no jointly-sequenced
WP-010 schema change — the authentication skeleton stays identity-only,
and the helper-computed tier arrives as validate-plan response data with
WP-070.

## Options considered

### Option (a) — read SAFE-002 through HLP-003: no authorization below Disruptive

Rejected. It inverts §0.2 — a Section 3 constraint bent to fit a Section
4.6 contract — the exact shape SI-38's resolution rejected and recorded.
It leaves the caching complement licensed (a remembered approval applying
privileged writes). It requires a Section 8 bypass edge around
AwaitingAuthorization. And it makes SAFE-002's context 1 false of every
severity-1 apply, leaving privileged behavior in an uncontemplated third
context.

### Option (b) — extend the interactive ceremony to every apply

Rejected on three costs. The rubber stamp: routine OS prompts for label
edits train the click that later approves a wipe — the SI-39 round
sustained the same attack shape as a real UI-009 risk; here it is
avoidable rather than inherent. The foreclosure: it kills scripted apply
outright, including the strong-identity severity-1 case SAFE-003
deliberately routes rather than refuses, making that clause dead text.
The flattening: every other consequence ladder in the spec — UI
consequence text, confirmation strength, EXE-002's battery gate, FS-010's
acknowledgment — scales with severity; authorization would be the one
flat ladder, and PLAN-004's severity-plus-flags sentence would become
false in the opposite direction.

### Option (c) — the two-tier ladder (accepted)

Accepted, scoped as above: SAFE-002 untouched, the floor act at every
severity, the ceremony verbatim at Disruptive and extended to flags, the
tier helper-derived, no plan field.

### Option (d) — a plan-carried authorization-requirement field, helper-authored at validation

Rejected. The ADR-0016 shape is available but buys nothing here: the
field would duplicate a total function of severity and flags already in
the hashed body, adding an agreement obligation with no safety gain. It
also forecloses nothing — if a future severity-orthogonal authorization
input ever exists (none is named today), it files its own round with its
own evidence; that is this ADR's first revisit condition.

## Decision

The two-tier ladder, landed as spec 11.2.0's amendment to HLP-003 and
only HLP-003. **SI-18 moves to Resolved.**

**Minor under §0.1, argued rather than assumed:** both pre-existing
HLP-003 sentences stand verbatim — the Disruptive-threshold sentence
never said *only* those severities require authorization, and the
caching sentence's "for these severities" scope is untouched, with the
floor's own never-cached clause covering the severities it does not
reach. The floor and flags clauses are additions; SAFE-002 is untouched;
no existing MUST is narrowed or retexted. The reading this forecloses —
promptless, authorization-free severity-1 apply — was never licensed,
because §0.2 gave SAFE-002 precedence the whole time. The counter-
argument (that extending enforcement is semantic change deserving major,
the 3.1.0 caution) was weighed and is recorded here so the numbering is
auditable; it was not taken because §0.1's rule turns on what happens to
existing requirement text, and none changes.

## Consequences

- **Positive.** SAFE-002's context 1 is satisfied at every severity
  rather than read down; the caching complement closes; PLAN-004's
  severity-plus-flags promise becomes true; the keyslot-class promptless
  path into key state never opens; Section 8's table stays total with no
  new edge.
- **Negative, accepted knowingly.** Scripted severity-1 applies proceed
  with no human present: fully-reversible by definition, journaled with
  their authorization acts, weak-identity-refused without the recorded
  override. The friction of the flags rule lands on flagged severity-1
  plans (the keyslot case), which can never be applied unattended.
- **For WP-040/WP-070.** The authentication skeleton stays identity-only;
  no authorization-requirement field exists to schema-sequence; the
  helper-computed tier ships as validate-plan response data when WP-070
  builds validation; HLP-003's enforcement tests land with WP-070.
- Nothing here is hash-visible: no plan-body field is added, and severity
  and flags were already body content.

## Verification

Owned by WP-070 (the helper) when it exists, recorded here so none is
discovered late:

1. A severity-1 apply without the floor act is refused; a second use of
   the same act is refused; an act outside the plan's PLAN-007 window is
   refused.
2. A flagged or severity ≥ Disruptive apply presenting the floor act but
   no interactive ceremony is refused — the LUKS-keyslot case as a named
   fixture.
3. A hand-forged artifact under-stating severity or omitting flags does
   not lower the enforced tier (the ADR-0012 hand-forged-artifact test
   pattern): the helper's recomputed values govern.
4. No cached-approval mechanism exists at any severity; the per-platform
   demonstration shape arrives with each transport's route decision,
   which already must record its Tier-1 test posture (WP-040).

## Revisit conditions

- A severity-orthogonal authorization input is ever named — something
  the tier must depend on that is not a function of the plan body's
  severity and flags. Option (d)'s rejection would need re-examining,
  through a new round with its own evidence.
- PLAN-004's flag vocabulary changes. The flags-nonempty ceremony rule
  reads the current closed set; a new flag joins the ceremony trigger by
  default (fail-closed), but a flag added with the *intent* of not
  escalating authorization would need this ADR amended first.
- SAFE-002's context sentence is amended by any later decision. This
  ADR's premise — that "fresh, explicit user authorization" and "fresh
  interactive authorization" are deliberately distinct vocabulary — would
  need re-examining.
