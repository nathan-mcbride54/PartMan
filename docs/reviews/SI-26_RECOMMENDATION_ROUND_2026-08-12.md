# SI-26 recommendation round — 2026-08-12

**Status: a recommendation adversarially reviewed, then filed as Accepted
on Nate's directive** ("finish SI-25 and SI-26"), following ten identical
delegated arcs in this session pair. Untracked session artifact under
`docs/reviews/**` (WP-000); the register's own text is not modified by
this round.

The register entry is `docs/spec-issues/README.md` §SI-26.

---

## The conflict, made precise

> **Section 16 (Prohibited shortcuts):** Agents MUST NOT […] Mark a
> capability Stable without its matrix fixture and acceptance evidence.

> **CAP-003:** Return `supported`, `preview`, `unsupported`, or
> `blocked` […] `supported` — apply permitted, backed by matrix
> evidence (CAP-006) […]

"Stable" is not one of CAP-003's four values and appears nowhere else in
the specification. Either it is a stale synonym for `supported` — in
which case Section 16's evidence rule attaches there — or `Capability`
needs a maturity axis orthogonal to status.

## Recommendation: a stale synonym — "Stable" is CAP-003's `supported`, and the evidence rule already lives there

1. **The reading:** Section 16's sentence prohibits advertising a
   capability as `supported` without its CAP-006 matrix fixture and
   acceptance evidence. CAP-003's own definition already says exactly
   this — "`supported` — apply permitted, backed by matrix evidence
   (CAP-006)" — so the prohibition and the definition are one rule seen
   from two sections, and the word "Stable" is a 2.0.0-era leftover
   from before CAP-003's vocabulary was fixed.
2. **No maturity axis.** A second quality vocabulary orthogonal to
   status would double what every consumer must handle for no
   distinction any requirement draws — the vocabulary-doubling shape
   ADR-C3 removed (comparison-outcome strength) and ADR-0015's option
   (e) rejected (client/helper strength). WP-050's delivered engine
   makes the synonym reading structural already: `supported` is
   constructible only through CAP-006 qualification evidence, whose
   token has no constructor until a qualifying row exists — Section
   16's prohibition is compile-checked today under this reading.
3. **The landing is an editorial cross-reference, patch-level:**
   Section 16's sentence gains "(that is, CAP-003 `supported`)" so the
   next reader is not sent to the register. Under the selected reading
   the parenthetical changes no semantics — §0.1's editorial class —
   and the ADR carries the decision itself, the ADR-0020 shape with a
   one-word repair instead of a bare banner.

## The adversarial round

**Attack 1 — "a maturity axis is real: a capability can be `supported`
yet freshly so — consumers may want 'newly supported' caveats."**
Rejected on the spec's own evidence bar: `supported` *means* qualified
by matrix fixture and acceptance evidence — there is no
qualified-but-immature state the spec recognizes, and inventing one
would weaken what `supported` promises (the guarantee-invariance
argument from ADR-0015, again).

**Attack 2 — "patch is too light: the sentence's meaning is being
fixed, the 3.1.0 caution says major."** Weighed and answered: the
decision is the ADR's (reading selection, the ADR-0020 precedent —
which bumped nothing at all); the text edit is a parenthetical that is
purely clarifying under that selected reading. The counter-argument is
recorded for the decision to overrule with.

**Attack 3 — "this decides SI-25's vocabulary question."** Refuted:
SI-25 governs the operation list; this decides one word's referent in
the status vocabulary and adds no status — CAP-003's four values stand
verbatim, exactly as ADR-0020 and ADR-0026 left them.

## Rejected, and why

- **(a) A maturity axis orthogonal to status.** Attack 1: doubles the
  vocabulary for a state the evidence bar makes unrepresentable.
- **(b) is the recommendation** — the synonym reading with the
  editorial cross-reference.
- **(c) Rename Section 16's word to `supported` outright.** Retexts a
  prohibition sentence (semantic-change class) for what a parenthetical
  achieves editorially; rejected on the SI-20 row-rewording economy.

## If accepted, the mechanics

ADR-0032 (reservation then resolution, the established shape), landing
the parenthetical in Section 16 — **patch** (12.9.1, after SI-25's
12.9.0): an editorial fix under the selected reading, with the major
counter-argument recorded. SI-26 → Resolved. No re-attribution: WP-050's
assignment cites no SI-26 gate, and its delivered engine already
enforces the reading structurally. Verification: the existing
compile-fail proof (no CAP-006 token constructor without a qualifying
row) is cited as the standing enforcement; no new obligation.
