# SI-14 recommendation round — 2026-08-12

**Status: a recommendation adversarially reviewed, then filed toward
acceptance under Nate's directive** ("let's cleanup the register
residue", 2026-08-12, naming SI-13/SI-14/SI-28/SI-37 from the session
summary it answered), following the eleven delegated arcs this
register's resolutions have used. Untracked session artifact under
`docs/reviews/**` (WP-000); the register's own text is not modified by
this round.

The register entry is `docs/spec-issues/README.md` §SI-14. The
companion residue-sweep verification for SI-13, SI-28, and SI-37 is
`REGISTER_RESIDUE_SWEEP_2026-08-12.md`.

---

## The conflict, made precise

> **INV-004:** Detect partitions, free extents, alignment, partition
> types/flags, labels, UUIDs, volumes, file systems, encryption,
> mounts, and nested storage.

> **MODEL-004:** Every discovered property MUST record, in the
> artifact envelope (MODEL-005), the set of observations that produced
> it. […] The confidence values `authoritative`, `inferred`,
> `unavailable`, and `conflicting` are **derived** from that set and
> MUST NOT be stored independently of it […]

Free extents and alignment are not observed; they are computed from
other properties that are (partition extents, device geometry). Read
literally, INV-004 makes them "discovered properties", MODEL-004 then
demands an observation set for each, and no observation exists to put
in it: none of the four confidence values describes a computation, and
no rule composes a derived property's confidence from its inputs.

## What has changed since the filing (the issue, measured)

The filing predates the entire delivered architecture. Three delivered
decisions have since answered the question in practice:

1. **ADR-C4 (3.1.0)**: confidence is derived from observations, never
   stored — the stored-confidence variant is unrepresentable, proven
   by the absence of a constructor.
2. **The WP-060 solver (delivered)**: free extents are computed at
   planning time from the snapshot's body-carried, authenticated
   extents. Nothing stores them; nothing attaches confidence to them;
   their trustworthiness is exactly their inputs', which the typed
   boundary authenticated.
3. **ADR-0023 (12.1.0)** rejected minting a typed alignment-fact field
   outright: "the offsets are already in the bound snapshot — a
   duplicate field would add only an agreement obligation," retained
   as a revisit condition should a querying consumer ever exist.

The named gate — "Later (WP-050)" — has been reached and passed:
WP-050's delivered engine consumed extents without needing a derived
confidence, and so did WP-060's solver. Leaving the issue open now
would mean the architecture answered it silently, which is the exact
failure mode the register exists to prevent. Recording the answer is
the cleanup.

## Recommendation: a derived property is a derivation, not an observation — the absence of a composition rule *is* the rule, made normative

1. **The rule.** MODEL-004's "discovered property" means an observed
   one. A property computed from other properties — free extents and
   alignment are INV-004's two — is a **derivation**: recomputed at
   use from its inputs, never stored, carrying no observation set and
   no confidence of its own. Its trustworthiness is exactly its
   inputs', which carry the observation sets. A surface that reports a
   derivation reports it as one, naming its inputs.
2. **Fail closed on unfit inputs.** A derivation over an input whose
   observation set derives `unavailable` or `conflicting` is not
   presentable: the answer is the input's own state, surfaced as such,
   never a derived value computed over a guess. (An `inferred` input
   yields a presentable derivation — the input's confidence travels by
   reference, not by copy.)
3. **INV-004 amendment** (the one normative text change): a clause
   marking free extents and alignment as derivations over the detected
   inputs (partition extents, geometry), presented recomputed under
   the rule above. Detection of the inputs is untouched; the duty to
   produce free extents and alignment is untouched.
4. **No fifth confidence value, no composition algebra.** Rejected
   below.
5. **Verification obligations.** (a) Already-delivered evidence cited:
   ADR-C4's constructor-absence proof; the WP-060 solver's
   free-extent tests over authenticated extents; ADR-0023's rejected
   duplicate field. (b) A recorded obligation against the discovery
   packages (WP-W100/WP-L100/WP-M100, none yet created): their
   INV-004 surfaces present free extents and alignment as derivations
   with no observation set of their own, refusing presentation over
   unfit inputs — the obligation lands in each assignment when
   created, recorded in the ADR so the creation cannot omit it (the
   ADR-0030 pattern).

## The adversarial pass — attacks mounted against the recommendation

**Attack 1: "INV-004 says *detect*; re-reading it as *derive* narrows
a MUST — this is major, not minor."** The duty INV-004 imposes — the
inventory produces partitions, free extents, alignment, and the rest —
survives verbatim; what changes is only *how* two list items are
produced (computed from the others rather than independently
observed), which was never specified. Nothing formerly required is
dropped; nothing formerly forbidden is permitted. The counter-argument
(any disambiguation is a semantic change — the 3.1.0 caution) is
recorded in the ADR; the 12.1.0 and 12.3.0 precedents took minor for
scoping additions of exactly this shape and recorded the same
counter-argument.

**Attack 2: "A derivation can be wrong even over authoritative inputs
— a bug in the computation. Composed confidence would catch that."**
No it would not: MODEL-004 confidence describes *observation* trust —
what an adapter saw — not computational correctness. A composed value
would be exactly as wrong as the buggy derivation it described.
Computational correctness is held where the codebase already holds it:
determinism tests (PLAN-001, byte-equal), the solver's placement
fixtures, and the typed boundary's recompute. A stored composed
confidence would additionally let a record assert what its inputs
contradict — the assertable-independent-of-observations record ADR-C4
made unconstructible.

**Attack 3: "What happens when an input is `conflicting` or
`unavailable`? The recommendation's first draft had no answer."**
Sustained — this attack produced item 2 (fail closed on unfit inputs),
which is the round's one genuinely new normative sentence. Without it,
an implementation could present free extents computed over a
conflicting partition list, which is a guess wearing a computation's
clothes.

**Attack 4: "Add a fifth confidence value `derived` instead — it is
the smaller change."** Rejected: a fifth *stored* value re-introduces
stored confidence for a class of records (the thing MODEL-004's second
sentence forbids); every consumer must already know derived-ness from
the property's identity, so the tag informs nobody; and it invites the
vocabulary-doubling shape ADR-C3 removed and SI-26's round rejected
for maturity. The absence of a derived-confidence value is not a gap
in the vocabulary — it is the vocabulary saying derivations are not
its subject.

**Attack 5: "SI-14's gate said WP-050; WP-050 shipped without the
decision — doesn't that prove the gate was wrong and this round is
late?"** Partially sustained: the gate named the wrong first consumer
(the capability engine consumes extents but reports no INV-004
inventory), and the true reporting consumers (WP-W100/L100/M100) do
not exist yet. But late-or-not, the delivered packages *did* embody an
answer, and the round's job is to record it before a discovery package
implements the literal reading and mints observation sets for computed
values. The ADR records the gate misnomer.

**Attack 6: "Does the plan body's `cancellation`/`severity`/`flags`
material contradict 'derived is never stored'? Those are planner
judgments carried in the hashed body."** No: they are *declarations* a
plan makes about itself (PLAN-004/PLAN-005 vocabulary), authenticated
by the hash and re-run at the boundary — not MODEL-004 discovered
properties of hardware. The rule here concerns inventory properties;
the ADR's scope sentence says so.

## What this does not decide

SI-13 (aggregate write-target identity binding) — untouched, its
WP-L110 gate verified separately. SI-28's floor and SI-37's evidence
clause — untouched. No hashed body or vector changes anywhere: the
rule confirms the delivered no-field state; MODEL-003 versioning is
not engaged.

## Proposed landing

ADR-0033 (`docs/adr/0033-si14-derived-values-are-derivations.md`),
reserved by a governance PR in the #69 shape; the INV-004 amendment
with the §0.3 changelog row and document-control version row (minor,
12.10.0); SI-14's authoritative rows and entry; the version references
the resolution footprint already establishes (`README.md`,
`CONTRIBUTING.md`, `.github/pull_request_template.md`,
`docs/quality/test-tiers.md`); and a matching `CHANGELOG.md` entry.
