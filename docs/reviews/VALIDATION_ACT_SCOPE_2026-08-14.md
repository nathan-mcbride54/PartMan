# Scoping the body-validation act — 2026-08-14

Untracked session artifact, `docs/reviews` convention. **This is a
scoping document put to the decision owner, not a recommendation to
implement.**

> **HEADER NOTE, WRITTEN AFTER THE ADVERSARIAL ROUND. Section 6's
> recommendation (option A, now) is WITHDRAWN, and the document's opening
> premise was false.** Two fatals and eleven serious findings landed; the
> ones that matter are in section 8. The taxonomy in section 2 survives.
> The recommendation and most of section 4's sequencing argument do not.
>
> The premise "nobody has stated the question in one place" is **wrong**:
> ADR-0037 already orders these acts, and in the *opposite* direction to
> "one act" — it names the #354 referent sweep as a precondition of #333's
> frame enforcement. I did not carry that to the decision owner, which is
> the single worst omission in this document.

## 1. The question

**Should the next act validate the body — its edges against its facts
against its naming fields against its structural multiplicity — rather
than add another predicate that reads it?**

## 2. Why it is being asked now

Four filed issues are four faces of one defect: **the body's structural
content is never validated against itself.**

| issue | what is unvalidated | status |
| --- | --- | --- |
| **#349** | an extent triple's well-formedness — zero length, unresolvable `host`, `start + length` overflow; and `assemble` skips the decode path's checks entirely | open |
| **#354** | naming-field referents — `parent_table`, `host`, `table` need not resolve, nor be the right kind, nor agree with the containment edge | open |
| **#356** | a containment edge against the extent facts — the edge says the signature is inside the partition, the fact puts it 400 MiB past its end | open |
| **#355** | structural multiplicity — two containment parents or two producers, resolved by sort order | **fixed** (PR #357) |

#355 is the one that was cheap enough to fix outright, because folding
`worst` over every edge is conservative by construction and needs no new
rule. The other three all require deciding **what the body is allowed to
say**, which is a decode-boundary question with MODEL-003 consequences.

**The empirical case is stronger than the taxonomy.** Across three rounds
on issue #319, **five** protection predicates were proposed and every one
died on unauthenticated body content:

- two subtracted reach on the strength of `extent_host` (ADR-0039's
  record: one turned a live-pool refusal into `Clear` on a body whose
  node ids and hash were unchanged);
- two false-refused ordinary disks;
- the fail-closed `Indeterminate` arm (2026-08-14) was silenceable by a
  decoy containment parent, and its exemption admitted the hole class it
  was built to close.

A sixth predicate — #347's whole-destruction release — is measured and
sound today, and it too reads `Facts.extents` to decide.

## 3. What an act would have to decide

1. **Which relation is normative when two parts of the body disagree.**
   Trusting the extent silently re-admits #319's class; trusting the edge
   discards the byte scan's own evidence. `Indeterminate` at the boundary
   is the third option and the one consistent with `Facts`' stated
   posture ("absence of a fact is honest absence and fails closed at the
   arm that needs it").
2. **Where the check lives.** `Topology::build` never receives `Facts`,
   so either the check moves to `assemble` / `from_canonical_body` (which
   see both), or the type changes so a topology and its facts are
   validated together. **#349's `assemble`-versus-decode asymmetry must
   be resolved by the same act**, or the check exists on one path only —
   and the planner and capability engine use the path without it.
3. **How strict, per relation.** Referent resolves / resolves-to-kind /
   agrees-with-edge are three strengths (#354). Frame agreement / span
   containment / sibling non-overlap are three more (#356). Span
   containment is what #356 measures; sibling non-overlap is stronger and
   a hybrid view's aliased entries may legitimately violate it.
4. **MODEL-003.** Every measured attack body decodes cleanly today.
   Refusing them is a versioned decode-boundary change requiring "a new
   version and migration or explicit rejection".
5. **The golden vector.** ADR-0037 already records the cross-language
   golden vector's node-framed convention as **unlawful under its decided
   rule and not corrected there**. Any frame-agreement rule collides with
   it.

## 4. The sequencing collision, which is the most actionable thing here

**#333's enforcement and this act both regenerate the golden vector.**

#333's rule is decided and its enforcement is held; the record states the
golden vector and `plan_tests.rs` are **unlawful under it** until an
enforcement PR regenerates them. A body-validation act touching frame
agreement lands on the same artifact.

Run as two separate versioned acts, they churn the same cross-language
fixture twice, with two MODEL-003 entries, two migration stories, and two
opportunities for the TypeScript and Rust sides to disagree in between.
**They want to be one act, or explicitly ordered with the second's cost
priced before the first lands.** This is worth settling regardless of how
the main question is answered.

## 5. Three shapes the act could take

**A. Full validation at the boundary.** Referents resolve and are
kind-correct; extents are well-formed; extent frames agree with
containment; `assemble` and decode share one path. Retires #349, #354,
#356 and probably narrows #319 and #333.
*Cost:* the largest MODEL-003 change the project has taken; collides with
the golden vector; needs a migration story for bodies in the wild (of
which there are none in production — no apply path exists — which makes
**now** unusually cheap).

**B. Well-formedness only.** #349's triple checks plus the
`assemble`/decode symmetry, leaving referents and edge-fact agreement
alone.
*Cost:* small and safe. *Benefit:* does not retire #354 or #356, and
leaves every closure predicate still reading content that can contradict
the graph.

**C. Decline, and keep hardening the closure.** Treat each face
separately, accept that predicates must be written defensively against
authored content, and adopt "can any authored field remove the refusal
this adds?" as the standing acceptance test.
*Cost:* the last three rounds are the evidence for what this costs. It is
not obviously wrong — the closure has genuinely improved — but the same
class of defect keeps being found by whoever looks next.

## 6. What I would recommend, and the honest caveat

**A, taken now, while there is no apply path and therefore no deployed
body to migrate.** The MODEL-003 cost will never be lower, the golden
vector has to be regenerated for #333 anyway, and it is the only option
that changes the odds for the *next* closure predicate rather than
hardening the last one.

**The caveat is real**: A is the largest single change on this board, and
this document has had one author and no adversarial pass. The measured
facts in it are solid; the recommendation is a judgment, and every
recommendation in this family that went unexamined has been wrong at
least once. **It should be adversarially reviewed before anything is
reserved**, and the review should be told to attack the recommendation
rather than the taxonomy.

## 8. THE ADVERSARIAL ROUND — what survives and what does not

Two fatals, eleven serious, three minor. The recommendation is withdrawn.
Recorded by what a reader must not carry forward:

### 8.1 FATAL — the case for A was not supported

Section 6's sole differentiating argument was that A "changes the odds
for the next closure predicate." Checked against the ADR and round
records one by one, **a validated body would have prevented at most one
of the five rejected predicates**, and for three of them validation is
the wrong instrument in kind. The empirical case in section 2 supports
*doing something*; it does not select A.

### 8.2 FATAL — A does not retire #356, and #356's own control was wrong

A's frame-agreement half is largely redundant against #347's release
clause (which already closes #356's contradiction body), while the same
live-pool approval reached by **omitting** the extent is lawful under all
four of A's rules and stays open.

**Reproduced by hand, and it corrects an issue I filed today.** At
`c9cd4bb`, on #356's own topology and delete: the contradiction body
constructs (affected 2, pool unreached) — and the **absent-extent body
constructs identically**, where #356 records `Err`. The filing's
conclusion "the escape is *exactly* the contradiction" is therefore
false. [Corrected on the issue.](https://github.com/nathan-mcbride54/PartMan/issues/356)

### 8.3 SERIOUS — the sequencing argument, three ways wrong

- The collision holds **for option A only**; under B or C there is
  nothing to settle, so "worth settling regardless" is wrong.
- Under A there is **no collision to sequence**, because A *contains*
  #333's enforcement rather than colliding with it.
- **ADR-0037 already states the ordering**, and names #354 as a
  precondition of #333's enforcement — the opposite direction from "one
  act". Two of section 4's four stated costs were measured not to exist.

### 8.4 SERIOUS — A can subtract reach, which disqualifies it as stated

"Extent frames agree with containment", applied uniformly, makes every
`BackingExtent`'s extent self-framed — which `descends_into` treats as
never a descent source — **silently removing HostBacking reach out of
loop-file stacks**. ADR-0037 carved `BackingExtent` out deliberately. An
act that removes reach violates the standing invariant, so A's one-line
frame rule is not implementable as written.

### 8.5 SERIOUS — my framing of "unauthenticated" was wrong

Section 2 said a forged predicate flipped a refusal on "a body whose node
ids and hash were unchanged". Node ids: yes. **Body hash: no** — extents
*are* hashed body content. What they lack is *address* coverage, not
authentication. Validation at the boundary therefore buys only
self-consistency, which an author of the whole capture satisfies for
free — a materially weaker benefit than section 2 implies.

### 8.6 SERIOUS — the option set omits the shape that has actually won

Twice now this defect class has been offered "validate at the boundary"
versus "make the closure conservative", and **the closure won both
times** (#355's fold; #347's release clause). Section 5's three options do
not contain that shape, and section 5C mischaracterizes declining as
writing predicates "defensively", which is not what either fix did.

## 9. Where this actually leaves the question

The taxonomy stands: four faces, one underlying gap. The recommendation
does not. What the record now supports is narrower and more useful:

1. **#354 first**, because ADR-0037 already makes it a precondition of
   #333's enforcement and nothing else is blocked behind it.
2. **#347 and #356 as one act**, since #347's clause closes #356's
   contradiction body and the two overlap on the same topology.
3. **The absent-extent spelling is #319's class and stays open** under
   every option here — it is not retired by validation.
4. **Any frame rule must carve out `BackingExtent`** per ADR-0037, or it
   removes reach.

The open question is no longer "validate or harden" but **"what does
#333's enforcement actually have to regenerate, and in what order does
ADR-0037 already say it must happen"** — which is answerable by reading,
not by a new act.

## 7. Not established here

- No option has been implemented or measured. Sections 3–5 are analysis
  over filed, measured issues; the costs are argued rather than run.
- The interaction with #353 (`canonical_ranges`' §2.1 over-claim) is
  unexamined and may matter: it is the other open item whose fix
  direction is unsettled.
- Whether A actually retires #319's remaining half is **not** established.
  It plausibly narrows it; nobody has measured a validated-body world
  against #319's three measured shapes.
