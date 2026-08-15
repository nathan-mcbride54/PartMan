# SI-18 recommendation round — 2026-08-11

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-18 stays Open until a decision is recorded through a
WP-010 spec change with an ADR, the shape ADR-0015/7.0.0 set. This is an
untracked session artifact under `docs/reviews/**` (WP-000); the register's
own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-18, filed in the
early register under Part 2 with status **Later (WP-040)**. It predates the
options-and-costs filing convention the later entries use: it states the
conflict and what the answer decides, and files no options. This round
therefore constructs the option space as well as recommending from it, and
the construction is itself open to review before the decision.

---

## The conflict, made precise

The two texts, quoted:

> **SAFE-002:** Privileged behavior is confined to exactly two contexts:
> 1. The platform helper executing a validated plan after fresh, explicit
> user authorization (HLP-003). […]

> **HLP-003:** Every apply of a plan with severity ≥ Disruptive (Section 6,
> PLAN-004) requires a fresh interactive authorization bound to the exact
> plan hash […] Cached, session-wide, or remembered approvals MUST NOT
> exist for these severities.

A severity-1 plan (Reversible — fully undoable via an emitted reversal
plan) still writes storage and still needs privilege. The two readings:

- **R1, SAFE-002 maximal:** every privileged apply requires the full fresh
  interactive authorization; HLP-003's severity threshold is dead text.
- **R2, HLP-003 exclusive:** authorization exists only at severity ≥ 2;
  a severity-1 apply is privileged behavior with *no authorization act at
  all*, and SAFE-002's context 1 is false of the product. Worse, HLP-003's
  caching sentence forbids remembered approvals only "for these
  severities" — R2 reads as *licensing* a cached or session-wide approval
  for severity-1 privileged writes.

Two secondary defects sit in the same texts and would survive a resolution
that ignored them:

1. **The flags gap.** PLAN-004 states in terms that "authorization
   requirements (HLP-003) key off severity plus flags" — and HLP-003's text
   reads flags nowhere. The gap is not hypothetical: adding a LUKS keyslot
   is fully reversible (severity 1) and touches keys (`security-sensitive`).
   Under a severity-only ladder it would take the *lightest* authorization
   in the product — a promptless, channel-authenticated addition of a
   decryption key.
2. **The caching license.** "For these severities" implies the prohibition
   on cached approvals has a complement. No text says what an approval
   below Disruptive even is, so the complement is undefined rather than
   permissive — but the resolution must close it, not inherit it.

---

## Recommendation: a two-tier authorization ladder; SAFE-002 untouched

**Every apply requires fresh, explicit, plan-hash-bound authorization; the
interactive OS ceremony is the upper tier, not the definition.** Concretely:

1. **SAFE-002 does not change.** It is a Section 3 constraint; §0.2 gives
   it precedence, and SI-38's resolution already recorded that bending
   SAFE-002 to satisfy a lower section inverts that order. Its words —
   fresh, explicit, user, authorization — are all load-bearing and all
   preserved. What it never said is *interactive* or *OS-mediated*; that
   is HLP-003's mechanism vocabulary, and the conflict dissolves exactly
   there.
2. **HLP-003 gains a floor tier covering every apply at every severity,
   0 included.** A fresh, explicit authorization act: performed by the
   RPC-001-authenticated user, naming the exact plan hash, single-use
   (one act, one apply), valid only within the plan's PLAN-007 window,
   journaled, and never cached, session-wide, or remembered — the caching
   prohibition extends to all severities, closing defect 2. The act may be
   programmatic: a scripted CLI apply naming the plan hash *is* a fresh
   explicit act by the authenticated user. That is what keeps SAFE-003's
   unattended/scripted-apply clause a live population instead of dead
   text.
3. **HLP-003's existing sentence survives verbatim as the ceremony tier,
   extended by the flags PLAN-004 already promised:** fresh *interactive*
   OS-mediated authorization (polkit `auth_admin`, the documented macOS
   APIs, the ADR-W1 consent) is required when plan severity ≥ Disruptive
   **or any step flag is set**. Flags-nonempty is deliberately the closed
   rule rather than `security-sensitive` alone: the other flags
   (`irreversible-after-start`, `requires-offline`, `requires-reboot`,
   `requires-rescue`) describe conditions that are disruptive or worse in
   substance, and enumerating a subset invites the next gap. A flagged
   plan can never be applied unattended — fail-closed, and the keyslot
   case (defect 1) takes the ceremony.
4. **The tier is computed by the helper from its own recomputed severity
   and flags.** HLP-002 already makes client-provided validation output an
   untrusted hint; the authorization tier inherits that posture. No client
   claim participates in tier selection, so a forged or buggy
   client-computed severity cannot lower the required authorization.
5. **No authorization-requirement field enters the plan** — the register's
   named question, answered. The requirement is a total function of body
   content the plan already carries (severity, flags); a stored copy would
   be a second encoding of the same fact, creating an agreement obligation
   with no safety gain — ADR-0016's lesson, which reached its safety
   property precisely by making client claims unrepresentable rather than
   checked. A client-assertable authorization is also exactly what
   CAP-007 and WP-040's charter forbid. Where the UI needs the tier for
   its AwaitingAuthorization display (UI-011), the helper computes and
   reports it in the validate-plan response — response data, never plan
   body, never client-authored.
6. **Versioning: minor.** HLP-003's existing sentence survives verbatim;
   the floor and the flags clause are additions; SAFE-002 is untouched; no
   existing MUST is narrowed or retexted. The R2 reading this forecloses
   was never licensed — §0.2 gave SAFE-002 precedence the whole time, so
   the addition states what precedence already required. Argued, not
   assumed: the decision may overrule to major out of 3.1.0 caution, and
   the ADR should record whichever argument is accepted.

## What a consumer and a plan may rely on (the evidence-clause statement)

- **Every apply, any severity:** the journal carries a fresh authorization
  act bound to the exact plan hash; no apply proceeds from connection
  standing, cached approval, or session state alone. One act never applies
  two plans, and never applies one plan twice.
- **A severity-1 apply:** the RPC-001-authenticated user explicitly
  requested *this* plan. Not that a human watched a prompt — that is the
  ceremony tier's guarantee, and consumers may not read it downward.
- **A severity ≥ 2 or flagged apply:** a human completed the platform's
  interactive ceremony for this exact plan hash, freshly, with no retained
  grant.
- **All tiers:** the tier that was enforced derives from the helper's own
  recomputation. A client cannot lower it by mis-stating severity or
  omitting flags.

## The adversarial round

**Attack 1 — "the floor is channel authentication in disguise; severity-1
gains nothing over reading (a)."** Refuted by locating the act. RPC-001
peer verification authenticates *who is connected*; the floor records
*that this user explicitly requested this plan, now*. They are separable
and separately testable: a connected, fully authenticated client that
sends apply-plan without the plan-hash-bound act is refused. Single-use
and PLAN-007 expiry make it a per-apply fact, not a standing one. The
attack sharpened point 2's wording — the floor is defined as an act with
four testable properties (fresh, hash-bound, single-use, journaled), not
as a channel property.

**Attack 2 — "extending the ceremony to flags is scope creep beyond the
filed conflict."** Refuted by PLAN-004's own text, which already promises
authorization keys off "severity plus flags"; HLP-003 not reading flags is
part of the same gap SI-18 filed, and the LUKS-keyslot case makes it
concrete. Leaving it would resolve the filed conflict while shipping a
known promptless path into key state. The attack did force a scoping
choice: flags-nonempty rather than a security-sensitive carve-out, for
closure (recorded in point 3).

**Attack 3 — "scripted severity-1 writes happen with no human present."**
Sustained as a real property and accepted deliberately. The population is
fully-undoable-by-definition (severity 1's own criterion), carries an
emitted reversal plan (PLAN-008), is journaled with the authorization act,
and inherits SAFE-003's unattended-apply refusal on weak-identity targets
unless the recorded override exists. The alternative — option (b) —
forecloses unattended apply entirely, making SAFE-003's clause dead text.
A spec that contemplates a population in one requirement and forbids it in
another would be filing its own next conflict.

**Attack 4 — "why not just require the ceremony everywhere? It is simpler
and strictly safer."** The simplicity is real; the safety is not strict.
Cost one: the rubber stamp — routine OS prompts for label edits train the
click that later approves a wipe (the SI-39 round sustained the same
attack shape as a real UI-009 risk; here it is avoidable rather than
inherent). Cost two: it kills scripted apply outright, including the
strong-identity severity-1 case SAFE-003 deliberately routes rather than
refuses. Cost three: every other consequence ladder in the spec — UI
consequence text, confirmation strength, EXE-002's battery gate, FS-010's
acknowledgment — scales with severity; authorization would be the one
flat ladder, and PLAN-004's "key off severity plus flags" sentence would
become false in the opposite direction.

**Attack 5 — "the tier keys off severity, and the client computed the
plan."** Refuted by HLP-002, which this recommendation cites rather than
extends: the helper independently revalidates, and the tier binds to the
helper's own severity/flags computation. The verification list makes it a
test, not an assertion: a hand-forged artifact claiming severity 0 for a
disruptive step set must take the ceremony tier or refuse.

**Attack 6 — "severity-0 plans make the floor absurd: authorizing a
no-op."** Sustained as aesthetics, refused as a carve-out. A severity-0
plan that reaches apply is degenerate but well-formed; giving it the floor
keeps the rule total ("every apply") and keeps Section 8's transition
table untouched — AwaitingAuthorization remains on every apply path, with
no severity-conditional bypass edge to specify, test, and defend. A
carve-out would buy one skipped journal line at the price of a second
authorization topology.

## Rejected, and why — to be recorded with the decision

- **(a) Read SAFE-002 through HLP-003: no authorization below
  Disruptive.** Inverts §0.2 — a Section 3 constraint bent to fit a
  Section 4.6 contract — the exact shape SI-38's resolution rejected and
  recorded. Licenses the caching complement (a remembered approval
  applying privileged writes). Requires a Section 8 bypass edge around
  AwaitingAuthorization. Makes SAFE-002's context 1 false of every
  severity-1 apply, leaving privileged behavior in an uncontemplated
  third context.
- **(b) Extend the interactive ceremony to every apply.** Rejected on
  Attack 4's three costs: rubber-stamp degradation of the ceremony where
  it carries real load, the foreclosure of a population SAFE-003's own
  text contemplates, and the flattening PLAN-004 contradicts.
- **(d) A plan-carried authorization-requirement field, helper-authored
  at validation (the ADR-0016 shape).** The field would duplicate a total
  function of severity and flags already in the hashed body, adding an
  agreement obligation (field versus recomputation) with no safety the
  recomputation does not already provide. ADR-0016's achievement was
  making client claims unrepresentable; here that is reached with no
  field at all. Also forecloses nothing: if a future severity-orthogonal
  authorization input ever exists (none is named today), it files its own
  round with its own evidence.

*(No option (c) label is used: the recommendation absorbed what would
have been (c) — the two-tier reading — as its main body.)*

## If accepted, the mechanics

WP-010 files the ADR (ADR-0021 is the next free number), amends HLP-003
and only HLP-003 — the floor clause, the flags clause, the all-severities
caching prohibition, the existing sentence verbatim — bumps **minor**
(11.2.0) unless the decision records the major argument instead, and
moves SI-18 to Resolved. WP-040's register gate lifts with the recorded
answer: **no authorization-requirement field, no jointly-sequenced WP-010
schema change** — the authentication skeleton stays identity-only and the
helper-computed tier arrives as validate-plan response data with WP-070.

Verification obligations, recorded in the ADR and owned by WP-070 (the
helper) when it exists:

1. A severity-1 apply without the floor act is refused; with a second use
   of the same act, refused; outside the PLAN-007 window, refused.
2. A flagged or severity ≥ 2 apply presenting the floor act but no
   interactive ceremony is refused — the keyslot case as a named fixture.
3. A hand-forged artifact under-stating severity or omitting flags does
   not lower the enforced tier (the ADR-0012 hand-forged-artifact test
   pattern).
4. No cached-approval mechanism exists at any severity: the per-platform
   demonstration shape arrives with each transport's route decision, as
   WP-040's transport increments already require route-recorded test
   postures.
