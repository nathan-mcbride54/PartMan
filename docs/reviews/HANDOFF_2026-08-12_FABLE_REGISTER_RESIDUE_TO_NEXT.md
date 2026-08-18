# Handoff — 2026-08-12/13, the register-residue arc (SI-13/14/28/37)

**From:** Claude (Fable 5), the session Nate directed with "let's
cleanup the register residue" (immediately after the PLAN-005
cancellation arc, same session — see
`HANDOFF_2026-08-12_FABLE_PLAN005_ARC_TO_NEXT.md`).
**To:** whoever picks this up next.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. Repository state

`main` at the #312 merge, **spec 12.10.0**. Working tree clean apart
from untracked docs/reviews. No open PRs. The WP-020 stopping
condition still pins at `77b0dd7` — both residue PRs were Markdown
only, so no sitting was owed and none ran.

## 1. What this arc did — two merged PRs, one resolution

| PR | What |
| --- | --- |
| #311 | Governance: reserve `docs/adr/0033` for SI-14's resolution in WP-010.md, with the grant's reach and non-reach stated (no MODEL-004 text amendment, no fifth confidence value, no typed derived-property field, SI-13/28/37 untouched, no discovery-assignment creation) |
| #312 | The resolution: ADR-0033 + the INV-004 amendment (spec 12.10.0, minor) + SI-14's register banner and status rows + the version-reference footprint + CHANGELOG |

**SI-14 — resolved.** A derived property is a derivation, not an
observation: recomputed at use from the detected inputs it names,
never stored, no observation set, no confidence of its own; a
derivation over an input whose observation set derives `unavailable`
or `conflicting` is not presentable (the round's one genuinely new
normative sentence, produced by its own adversarial attack 3). The
gate ("Later, WP-050") had been reached and passed by delivered work
that embodied the answer — ADR-C4, the WP-060 solver, ADR-0023's
rejected duplicate field — so this records rather than invents.
Presentation obligations for WP-W100/WP-L100/WP-M100 land in those
assignments at creation (the ADR-0030 pattern); **whoever creates
those assignments must carry them in** — ADR-0033 §Verification 2 is
the record the creation cannot omit.

**SI-13 — verified, not edited** (beyond a gate note in the Later
row): identities bind at validation (every delivered planner path
emits empty identity maps), aggregates are not plannable write targets
(the closed Operation vocabulary has no LVM/mdraid entries; extending
it is WP-050's next reviewed increment per ADR-0031), so the WP-L110
pin is accurate and the conservative refusal is structural.

**SI-28 — verified, not edited**: Mitigated-open, floor in force.
Relaxation needs apparatus-qualification evidence under ADR-0017's
revisit condition — a Windows-side hardware measurement campaign with
the authorized fixture media. A deliberate future arc, not
documentation debt.

**SI-37 — verified, not edited**: open by its own evidence clause (the
per-platform dual-path matrix with negative controls), which nothing
existing satisfies and nothing contemplated needs yet. The filed
population is typed and fail-closed today (ADR-0018's transport arm).

## 2. Decisions worth review

- **ADR-0033 itself**, made in the delegated-arc pattern under the
  broad directive. If the fail-closed presentation rule (unfit inputs
  → no derived value) is too strict for some UI case, the ADR's
  revisit conditions name the persistence round as the escape route —
  never a silent softening.
- **The acceptance-basis phrasing** in ADR-0033 records the directive
  verbatim ("let's cleanup the register residue") rather than a
  per-issue directive like the SI-25/26 precedent; if Nate wants the
  record to carry an explicit per-issue confirmation, a one-line
  follow-up to the ADR's Date field does it.

## 3. What remains open

The register now holds: **SI-13** (Later, WP-L110), **SI-28**
(Mitigated-open, floor in force), **SI-37** (Open/Later, evidence
clause) — all three deliberately, each with an accurate recorded gate
— and nothing else. The residue's two live follow-up campaigns, if
ever directed: SI-28's apparatus-qualification measurements, SI-37's
dual-path matrix.

## 4. Session tally (both arcs)

Seven merged PRs this session before this arc (#306–#310 plus the two
earlier merges recorded in the PLAN-005 handoff), two in this arc
(#311, #312). Spec moved 12.9.1 → 12.10.0. WP-060's assignment is
fully delivered; WP-010's plan body is at version 4 sole-live; the
register's residue is one resolution smaller and three accurate gates.
