# ADR-0032: Section 16's "Stable" is CAP-003's `supported`

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride the same day, by directive
  ("finish SI-25 and SI-26") on the adversarially reviewed
  recommendation round of the same day, following ten identical
  delegated arcs; the directive is recorded here as the acceptance
  basis (`docs/reviews/SI-26_RECOMMENDATION_ROUND_2026-08-12.md`, an
  untracked session artifact; this ADR restates everything load-bearing
  from it).
- Spec version: 12.9.1 (patch under §0.1 — the decision is this ADR's
  reading selection, the ADR-0020 shape; the one-phrase parenthetical
  is editorial under the selected reading; the major counter-argument
  is recorded in Decision)
- Work packages blocked: none — WP-050's delivered engine already
  enforces the reading structurally
- Requirement IDs: Section 16 (editorial parenthetical); CAP-003,
  CAP-006 (read, none amended)
- Decision owners: Nate McBride

## Context

Section 16's prohibited-shortcuts list forbids marking a capability
"Stable" without its matrix fixture and acceptance evidence. "Stable"
is not one of CAP-003's four values (`supported`, `preview`,
`unsupported`, `blocked`) and appears nowhere else in the
specification. SI-26 filed the fork: a stale synonym for `supported` —
in which case Section 16's evidence rule attaches there — or a
maturity axis orthogonal to status.

## Safety analysis

**The synonym reading is the specification's own.** CAP-003 defines
`supported` as "apply permitted, backed by matrix evidence (CAP-006)" —
which is Section 16's prohibition stated as a definition. The two
sentences are one rule seen from two sections, and "Stable" is a
2.0.0-era leftover from before CAP-003's vocabulary was fixed.

**No maturity axis.** A second quality vocabulary orthogonal to status
would double what every consumer must handle for a state no requirement
recognizes: `supported` *means* qualified by matrix fixture and
acceptance evidence, so there is no qualified-but-immature state to
carry — inventing one weakens what `supported` promises, the
guarantee-invariance argument from ADR-0015, and repeats the
vocabulary-doubling shape ADR-C3 removed and ADR-0015's option (e)
rejected.

**The reading is already structural in delivered code.** WP-050's
engine makes `supported` constructible only through CAP-006
qualification evidence, whose token has no constructor until a
qualifying row exists — Section 16's prohibition is compile-checked
today under this reading, which is evidence the reading matches the
architecture rather than merely rescuing the text.

## Options considered

### Option (a) — a maturity axis orthogonal to status

Rejected: doubles the vocabulary for a state the evidence bar makes
unrepresentable, and weakens `supported`'s promise.

### Option (b) — the synonym reading with an editorial cross-reference (accepted)

Accepted: Section 16's sentence gains "(that is, CAP-003 `supported`)"
so the next reader is not sent to the register.

### Option (c) — rename the word to `supported` outright

Rejected: retexts a prohibition sentence — the semantic-change class —
for what a parenthetical achieves editorially; the SI-20
row-rewording economy.

## Decision

Option (b), landed as spec 12.9.1. **SI-26 moves to Resolved.**

**Patch under §0.1, argued rather than assumed:** the decision itself
is this ADR's reading selection — ADR-0020's precedent, which bumped
nothing at all — and the parenthetical changes no semantics under the
selected reading, §0.1's editorial class. The counter-argument (fixing
a word's referent is fixing meaning — the 3.1.0 caution, arguing
minor or major) was weighed and is recorded so the numbering is
auditable; it was not taken because under the selected reading the
sentence's obligation is byte-for-byte what CAP-003 already required,
and no implementation could distinguish the spec before from after.

## Consequences

- **Positive.** The last dangling status word is grounded; CAP-003's
  four values stand verbatim, exactly as ADR-0020 and ADR-0026 left
  them; no consumer gains a match arm.
- **Negative.** None identified beyond the parenthetical's four words.
- **For WP-050.** Nothing: the delivered compile-fail proof (no
  CAP-006 token constructor without a qualifying row) is cited as the
  standing enforcement — no new obligation.

## Verification

The existing compile-fail proof is the enforcement, cited rather than
duplicated. No new obligation is created.

## Revisit conditions

- CAP-003's status vocabulary is ever widened (a question ADR-0020 and
  ADR-0026 each noted as formerly SI-26's territory — now nobody's
  until filed anew); the synonym reading survives any widening that
  keeps `supported`'s evidence bar.
- A requirement ever needs a qualified-but-immature state; that is a
  new filing with its own evidence, not a re-reading of this word.
