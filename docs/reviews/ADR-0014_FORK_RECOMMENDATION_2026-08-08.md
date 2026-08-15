# The ADR-0014 fork: may ADR-C4's guard be amended? — recommendation round, 2026-08-08

**Status: ACCEPTED by Nate McBride, 2026-08-08** — the guard is a priced
permission under the four conditions below, not an absolute veto. The
2026-08-05 handoff's drafting hold is lifted; the durable record of this
decision lands inside ADR-0014 when it is drafted, per the mechanics
section, citing this round. It deliberately does not decide SI-35's axis.
This
is an untracked session artifact (`docs/reviews/**`, WP-000); the two
earlier draft rounds died with their session's scratchpad, so this round
is reconstructed from repository records alone and cites nothing it cannot
quote.

## The fork, stated exactly

ADR-C4 carries a verification guard, shared with ADR-C3
(`docs/adr/0004-provenance-observations.md`):

> A positively absent partition table and an unreadable one produce
> different body values (shared with ADR-C3).

Both prior ADR-0014 draft rounds converged on taking partition-table state
out of the hashed body, which makes `Absent` and `Indeterminate`
indistinguishable **in the body** — the data-loss shape ADR-C4 refused,
reached by another route, because PART-001 initializes blank media and the
spec's own ADR-C4 note says conflation is what "PART-001 would then
initialize alike."

The question for the decision owner is narrow: **is that guard an absolute
veto on out-of-body table state, or a named price an amending ADR may pay
under stated conditions?**

## What each side holds, quoted

**The hold side's charter is ADR-C2 itself.** Its decision table places
"Bound device identities and strength" in the **body**, "because the
authorization names these targets," and its safety analysis is blunt: "the
boundary between hashed and unhashed is a security boundary, and every
field placed outside it is a field an attacker may alter without
invalidating an approval." Table state is a field *inside* the bound
identity record (SAFE-003). Move it out and the user's approval no longer
commits to which content state they authorized against — plan-time
`Present{checksum}` versus apply-time table-rewritten is no longer an
*identity* mismatch the hash detects, only a helper observation.

**The amend side's charter is the same ADR, plus the measurements.**
ADR-C2's placement rule: "A field belongs in the envelope only if it is
the hash itself, or the privileged helper independently re-derives it and
treats the client's copy as an untrusted hint (HLP-002)." Table state is
exactly such a field — HLP-002 re-derives it before the first write, and
M10 measured the helper positively determining every state the client
cannot. Meanwhile the completed SI-35 measurement campaign established
that **no enumerated client contract computes the three states** (the
decisive pair is byte-identical on all three platforms), and spec 7.0.0
just recorded that on macOS the client cannot reach `Absent` at all. So a
body-resident, client-authored table state is one of: **wrong** (a
conflicting table written as `Present` — the INV-003 report SI-35's
`Present` face has open), **vacuous** (always-`Indeterminate`, which
hashes fine and commits to nothing), or **not client-authored** — and the
envelope-side integrity story is recomputation, not trust: the helper
"treats the client's copy as an untrusted hint" either way. MODEL-005's
body-stability rule adds the general principle: "A fact that a verdict
needs but that fails this rule is evidence the wrong fact was chosen, not
grounds to relax the rule."

**The precedent trend is two-for-two.** ADR-0013 moved INV-003's
undetectable remainder to the privileged re-discovery; ADR-0015 moved
blank-strength's safety load to the pre-apply re-probe, "the observer
that can see." Both accepted by the decision owner. The guard predates
both.

## Recommendation: amend is permissible — as a named price, under four conditions

Not "amend now," and not a blank cheque: the recommendation is that the
guard **stops being an absolute veto and becomes a price list**, so that
ADR-0014's drafting round may weigh body-resident and out-of-body shapes
on their merits. Any draft that exercises the amendment must,
simultaneously:

1. **Keep the distinction representable everywhere the state appears.**
   ADR-C3's three-valued vocabulary survives in observation records and
   any envelope residence; only the *body residence* is in question. A
   draft that shrinks the vocabulary itself is outside this permission.
2. **Replace the body-value distinction's protective duty with a helper
   categorical invariant, in normative text**: PART-001 proceeds only on
   the helper's own fresh, positively determined `Absent` at apply time —
   never on a plan-carried claim, never on unseparated media, never on
   `Indeterminate`. With it, the two-fixture test obligation
   (`blank-512` proceeds through its strength-appropriate path;
   `gpt-conflicting-tables-512` and every `Indeterminate` refuse),
   mutation-verified in the increment that lands it. A fresh
   determination at write time is *stronger* than a stale plan-time
   commitment for the initialize case; the point of this condition is
   that the swap is stated and tested, not assumed.
3. **Name what the authorization hash stops committing to, and assign the
   remainder.** The genuine loss is hash-detected content-state change on
   *occupied* media — plan-time `Present{checksum X}`, apply-time
   `Present{checksum Y}`. That burden moves to the freshness design
   (PLAN-006 over what the body still carries, SAFE-005's stale-topology
   duty, and SI-34's open freshness projection), and the draft must say
   so in terms. If the draft cannot state where that detection now
   lives, condition 3 fails and the amendment is not available to it.
4. **Quote, not paraphrase, the ADR-C2 tension it is resolving** — the
   "bound device identities are body" row against the "helper re-derives
   it" rule — and record which body-resident alternatives were rejected
   and why, including the helper-stamped-at-validation shape (below).

## The adversarial round

**Attack 1 — "conditional permission is a rubber stamp; every guard is
'amendable with conditions', so this decides nothing."** Sustained in
part, and it sharpened the framing: what this decides is the *authority
question* the handoff recorded as blocking — whether drafting may proceed
into the space two prior rounds converged on. The alternative answer
(hold, absolute) is a real option with real consequences: it vetoes that
space while the guard's client-side satisfiability is measured-false, and
forces ADR-0014 into shapes the register already prices as expensive
(privilege-tagged bodies: hash-visible basis, "the PLAN-006 problem
ADR-C2 exists to prevent") or measured-unsupported (clamped projections:
"equality in one finite libblkid projection neither supplies that
contract nor refutes the existence of another"). Rejecting absolutes in
both directions and pricing the middle is the decision.

**Attack 2 — "amendment is unnecessary: a body-resident, helper-stamped
table state satisfies the guard."** The strongest attack, and it forced
condition 4. The shape: the client's draft plan carries its hint;
validation (privileged, before authorization) stamps the
helper-determined state into the body; HLP-003 then binds the user's
approval to the post-validation hash. This keeps the guard, keeps the
authorization commitment, and matches the flow's ordering. It is a
genuine candidate — **and deciding for it here would be the fork
overreaching into SI-35's axis**, because it presumes validation may
author body content (today the helper *enforces* body values, and ADR-C2
rests on "enforcing a value is not re-deriving it"), presumes a helper
exists at plan-validation time on every path that wants a hashable plan,
and inherits SI-35 option (a)'s question of what the stamp's observation
basis does to cross-platform hash stability. The fork's job is to make
the draft weigh this shape against out-of-body honestly — condition 4
forces exactly that — not to crown it unexamined.

**Attack 3 — "conditions 2 and 3 quietly decide SI-35's option (c)."**
Refuted by scope: they bind only a draft that *exercises* the amendment.
A body-resident draft (attack 2's shape, or option (a)) never triggers
them, and nothing here ranks the axis options. The fork's output is a
permission with a price, not a design.

**Attack 4 — "the guard protected authorization semantics; 'the helper
will check' is exactly the trust-shift ADR-C2 warned against."** The
attack misreads which side of ADR-C2 is in play. ADR-C2's warning is
about fields an attacker may alter *and thereby defeat an approval* —
envelope fields the apply path trusts. Condition 2's invariant trusts
nothing that travels: the helper acts on its own fresh determination, so
there is no alterable field whose forgery changes behavior. What is
genuinely lost is condition 3's item — hash-detected change on occupied
media — and the answer to that is an assignment, not a denial. Sustained
as the reason condition 3 exists.

## Rejected framings, recorded

- **Hold, absolute.** Rejected as deciding SI-35's axis by veto while the
  vetoing guard's client-side premise is measured-false — the mirror
  image of the overreach attack 2 refuses. Its honest core (the
  authorization-commitment loss) survives as condition 3.
- **Amend, unconditional.** Rejected without needing the round: it
  discards the PART-001 protection the guard exists for, and the register
  would rightly file it as data-loss-shaped.
- **Defer the fork into the ADR-0014 draft itself.** Rejected on process:
  the handoff's instruction exists because two draft rounds burned
  against an undecided premise. The premise gets decided first, by its
  owner, on the record.

## If accepted, the mechanics

The decision is recorded where ADR-0013's meta-decision precedent puts it:
inside ADR-0014 when it is drafted ("the guard-as-price decision was taken
separately by Nate McBride on <date>"), with this document as the round it
came from. No spec text changes now; ADR-0004's guard line stands until an
accepted ADR-0014 exercises the permission and restates it. The drafting
hold lifts, and the next round is SI-35's actual axis — with attack 2's
shape and the out-of-body shape both on the table, each paying its stated
price.
