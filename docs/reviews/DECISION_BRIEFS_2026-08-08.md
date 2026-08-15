# Three decisions only Nate can make — briefs, 2026-08-08

Untracked session artifact (`docs/reviews/**`, WP-000). Each brief states a
fork, the evidence each side rests on, and what each choice forecloses. None
of these decides anything, and the third brief's recommendation is the prior
analysis's, restated rather than invented here.

---

## 1. ADR-0014 (SI-35's axis): may ADR-C4's guard be amended?

**The fork.** Two drafts and two adversarial rounds (2026-08-05 session;
drafts lived in that session's scratchpad and are gone — only this fork
survived into the handoff) converged on the same move: take partition-table
state **out of the hashed body**. That move collides with a standing
verification guard in ADR-C4 (`docs/adr/0004-provenance-observations.md`,
shared with ADR-C3):

> A positively absent partition table and an unreadable one produce
> different body values.

Remove the field from the body and `Absent` and `Indeterminate` collapse
there — which is the data-loss shape ADR-C4 refused, reached by another
route. The spec's own ADR-C4 note says why it refused it: conflating the two
"collapses a blank device and an unreadable one into the same record, which
PART-001 would then initialize alike."

**Why the move keeps winning the drafts anyway.** The completed SI-35
measurements say no enumerated client contract computes the three states;
MODEL-005's body-stability rule says a hashed body may carry only facts
invariant under re-probe of unchanged hardware; and PLAN-006/ADR-C2 exist
because two observers at different privilege must not produce different
bodies. Table state in the body therefore forces one of SI-35's two
expensive shapes: privilege-tagged state (option (a) — the basis becomes
hash-visible body content, the exact placement problem SI-34 is open on) or
a clamped client projection (option (b) — which requires a separating
client contract that two decision-complete measurement campaigns failed to
find). Out-of-body state sidesteps both. That is the attraction, and it is
real.

**The decision.** Either:

- **Amend the guard.** Body values stop encoding the blank/unreadable
  distinction; the distinction survives in the vocabulary and in whatever
  non-body residence the ADR picks; and the PART-001 protection must be
  re-proven through another mechanism (most plausibly: the helper — which
  M10 showed can positively determine the state everywhere — plus SAFE-005
  and the SAFE-003 identity-change rejection). The amendment must name the
  new guard, or the old one's deletion is just a lost regression.
- **Hold the guard.** ADR-0014 must take a body-resident shape, and the
  next draft round starts from SI-35 (a) or (b) with their recorded costs,
  not from a third attempt at the same collision.

**What not to decide here.** SI-35's own option choice, and the Present-face
question SI-39 deliberately left to it. Note the SI-39 recommendation round
(same date, sibling document) is orthogonal: it keeps the three-valued
vocabulary and the guard intact regardless of which way this goes.

**Standing instruction, unchanged:** no third draft before this fork is
decided. `docs/adr/0014-si35-table-state-axis.md` stays reserved and empty.

---

## 2. Increment 9 (macOS adapter): the plist route

**The obstacle.** `diskutil` emits plists — the increment 6 matrix read
`diskutil info -plist` / `list -plist`, so this is the measured interface,
not a guess. `apps/cli` has an enforced empty dependency closure (a
structural guard asserted through `cargo metadata`, whose stated purpose is
that no hash or plan implementation can arrive from outside the crate) and
inherits `unsafe_code = "deny"`. Increment 9 is authorized "through the
existing SAFE-004 launcher seam" — subprocess launch is settled; parsing
the bytes that come back is not.

**Route A — hand-write a bounded plist reader.** Keeps both guards intact.
Costs: a real parser of externally supplied bytes, which under this
project's posture attracts a Section 11.4-shaped fuzz obligation (the
`fuzz/` scaffolding exists and the project knows how to pay this); XML
plist is a nontrivial grammar if generalized. The mitigation that makes it
tractable is the increment 8 pattern: a *bounded* reader for exactly the
keys the increment needs, refusing anything unexpected — over-limit,
non-UTF-8, unknown structure — rather than interpreting it. Refusal is
cheaper than generality and this package has already made that trade three
times.

**Route B — take a plist dependency.** Costs: the empty-closure guard must
be restated, not quietly widened — the guard is currently the mechanical
half of the no-hash/no-plan boundary, and after any dependency lands, that
boundary is review-enforced rather than structural. Plus dependency-policy
gates (pinning, licence, supply chain) for a parser crate. Benefit: no
hand-rolled parser to fuzz and maintain.

**Route C — IOKit instead of diskutil.** Named for completeness: FFI is
barred in `apps/cli` by the workspace `unsafe` denial, and hosting it in a
separate crate both restates the closure guard (a workspace crate is still
a dependency) and adds SAFE-009 reviewed-unsafe surface. Strictly worse
than B unless diskutil is rejected for its own reasons; nothing measured
rejects it.

**Lean, stated as a lean:** Route A. Both guards were bought with
adversarial rounds; restating them to avoid writing a bounded reader spends
a structural property on a convenience. The fuzz obligation is the honest
price. If the parser risk is weighed heavier than the guard, B restated
openly is defensible; C is not the cheapest form of either property.

**Whichever route:** the macOS reach-declaration cells and their measured
table land with the code that reads the interface — increment 7's rule,
"never ahead of it."

---

## 3. Increment 10 (Windows): choose a route, where deferral is a route

**The constraint triangle.** No Windows enumeration route is simultaneously
dependency-free, `unsafe`-free, and clean against the tool-invocation rules:

- **WMI/CIM** needs FFI — barred in `apps/cli` under the workspace `unsafe`
  denial, and a separate FFI crate breaks the empty-closure guard (brief 2's
  Route C cost, plus reviewed-unsafe surface).
- **PowerShell** adds a shell to the SAFE-004 roster — the requirement
  whose first sentence is "no shell strings" — and its output still needs a
  JSON reader, so it pays brief 2's parser cost *on top of* the roster
  widening and version-pinning awkwardness.
- **Defer to WP-W100** and ship what spec 6.1.0 already built the gate to
  accept: the published reach declaration plus the typed `not-implemented`
  answer naming the recorded decision that defers it. The 6.1.0 text was
  written so that "a platform whose access route is an open structural
  question" does exactly this without holding M0.5 hostage.

**The prior analysis recommended deferral, and nothing has changed its
inputs.** The recorded-choice requirement in WP-035's grant ("increment 10
opens only after a recorded choice among its three named routes") is
satisfied by recording *deferral* — the choice must be recorded, not
necessarily implementational. Deferral closes increment 10 as "deferred to
WP-W100," the typed refusal starts naming that decision instead of a bare
package name, and increment 11's sweep carries it into the records.

**What deferral costs, plainly:** no real Windows enumeration until
WP-W100. The alpha reads real hardware on Linux now, on macOS after
increment 9, and answers Windows with a typed refusal that names a decision
rather than an absence. Given that both live routes compromise a Section 3
constraint or a bought structural guard to deliver an interim surface a
platform package will replace anyway, the deferral is the only route that
spends nothing it cannot get back.
