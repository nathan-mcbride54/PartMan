# The consequence-text body slice — WP-010 slice 3p and WP-060 increment 12, 2026-08-18

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Written before the first line of code, per house convention;
> its base is `93c11dd` (main), spec 17.3.0, plan body version 4.

**Directive:** Nate — "start the consequence-text body slice".
**What it closes:** issue #371's last rider — "in the plan" is the hashed
`OperationPlan` body's §6 consequence-text item, and the planner's
enumeration is planner-layer carriage until this slice lands
(ADR-0052 D6: *delivered-in-planner, pending-in-body*).

## 0. What is already decided, so this plan decides nothing

- **§6:** the body MUST contain "Risk classification and user-facing
  consequence text" (`AGENT_BUILD_SPEC.md:463`). Risk is carried per step
  already (PLAN-004, slice 3i); the consequence text is the item this
  slice adds.
- **ADR-0023 (12.1.0):** "recorded in the plan" means **in its consequence
  text as a fact about the device**; typed hashed carriage of the facts
  was **rejected** — "a hashed-schema field duplicating offsets already
  present in the bound snapshot would add only an agreement obligation;
  consequence text serves the only consumers that exist" — and retained
  as a revisit condition keyed on a consumer that must *query* the facts.
  No such consumer exists. So the body carries **text**, and the typed
  facts stay where they are (`Planned.consequences`).
- **ADR-0052 D6:** the enumeration vocabulary's silence is bounded — it
  is not a boot-consequence verdict; the body item inherits that bound
  by carrying the vocabulary's own sentences and nothing more.
- **MODEL-003:** a body change is a schema-version change with explicit
  migration; **MODEL-006:** a collection is a semantic array or a
  declared set, never ambiguous.
- **The 3n/3o precedent** for a jointly-sequenced body change: the domain
  slice lands first with a delegating constructor that keeps every
  existing emitter valid, the version whose only emitters were inside
  the change window is retired in the same change, the golden vectors
  are regenerated and reproduced by the TypeScript suite unchanged, and
  the producer's increment follows in the same arc; **one sitting at the
  arc's head**, named in both PR bodies before the first merge.

## 1. Measured at `93c11dd`

| fact | where |
| --- | --- |
| Body version is 4; `body_value` emits `schema, schema_version, plan_id, created_at, snapshot_hash, not_after, identities, steps, reversal`; unknown keys refuse | `crates/domain/src/model/plan.rs:54`, `:267-296`, `schemas/domain/plan-body.md:70-84` |
| `assemble_linked` is the sole assembly path (slice 3o retired `assemble`) with four planner callers and eleven in the domain's own tests | `plan.rs:180`; `planner/src/lib.rs:843,939,1387,1530` |
| The reversal draft's body is a linked plan body sharing `LINKED_SCHEMA_VERSION` | `plan.rs:1556`, `:1627` |
| Four version-4 golden vectors in `schemas/domain/body-vectors.json` (`plans`), reproduced by `crates/domain/tests/body_vectors.rs` and the TypeScript suite (`packages/canonical/src/vectors.ts`) | measured |
| `Planned.consequences: Vec<Consequence>` is planner-layer carriage beside the plan; three variants, each with a `Display` sentence | `planner/src/lib.rs:263`, `:273-337` |
| Version 4's emitters are the planner (through `assemble_linked` → `body_value`) and the vectors — none survives a domain-side version bump | measured |

## 2. The shape

**Body item `consequences`** — required, **set-valued** `Array` of `Text`
(MODEL-006, `schemas/domain/canonical-collections.md`): the plan's
user-facing consequence sentences, each non-empty UTF-8, unique, sorted
by canonical bytes by the producer and refused unsorted or duplicated
by the consumer. A set rather than a semantic array because the
sentences are facts about the device with no order of their own, and a
set makes the hash independent of the planner's emission order. **Empty
is lawful** — a plan with no consequence to state carries an empty set,
and the boundary's silence bound is the ADR-0052 one: an empty set
asserts nothing beyond "the vocabulary had nothing to say".

**Body version 5.** Version 4 is retired in the same change on the v2/v3
precedent — one change window old, no emitter outside it once the
domain emits 5, its vectors regenerated as v5 — and refuses at decode.

**The draft** (`ReversalDraft`) carries the item **pinned empty**, exactly
as its step class is pinned `ordinary` and its cancellation pinned to
the floor: a draft is a prediction, its consequences are authored at
its own planning when it binds, and a draft body claiming any refuses
at decode.

**Domain API.** `assemble_linked` keeps its signature and delegates with
an empty set (every existing emitter stays valid under both regimes —
the consumer-first shape without a consumer change); the fully-stated
form `assemble_linked_stated(…, consequences: Vec<String>)` takes the
sentences, refuses an empty string, and sorts and dedups into the set.
`OperationPlan::consequences() -> &[String]` reads them back.

**Boundary.** `from_canonical_body` requires the key, requires an Array
of Text, refuses an empty element, refuses a set violation (unsorted or
duplicate — the canonical-set consumer rule), and, for a draft, refuses
a non-empty set.

**What the boundary does not do:** recompute the sentences. ADR-0023's
reading stands — the text serves UI-005 and REC-010; the facts it states
are in the bound snapshot already; the helper's HLP-002 recomputation is
of the topology and the closure, not of prose.

## 3. The two acts

**WP-010 slice 3p** (`crates/domain`, `schemas/domain/plan-body.md`,
`schemas/domain/body-vectors.json`, `crates/domain/tests/body_vectors.rs`,
CHANGELOG, WP-010 record, generated traceability):
- `LINKED_SCHEMA_VERSION = 5`; the item in `body_value` and in the draft's
  body; the boundary rules above; v4 refused at decode.
- `assemble_linked_stated`; `assemble_linked` delegating.
- Vectors regenerated as `plan-v5-*` (four, plus the re-encoded draft),
  reproduced by the TypeScript suite unchanged.
- Tests: the item round-trips; an empty string refuses; an unsorted or
  duplicated set refuses; v4 refuses; a draft with a non-empty set
  refuses; the delegating form emits an empty set. Mutations, each
  proven applied and killed: the version refusal dropped, the set check
  dropped, the empty-string check dropped, the draft pin dropped, the
  delegating default flipped non-empty.

**WP-060 increment 12** (`crates/planner`, WP-060 record, README row,
CHANGELOG, generated traceability):
- Every planning path (`plan`, `plan_sized`, `plan_set`, `plan_repair`)
  assembles through `assemble_linked_stated` with
  `consequences.iter().map(ToString::to_string)`; `Planned.consequences`
  stays as the typed carriage beside the plan, and its doc-comment loses
  the "later jointly-sequenced body change" clause because the change
  has landed.
- Tests: the body's set equals the sorted-deduped `Display` of the typed
  facts on the move fixture (three sentences: coincident, inherited,
  release) and is empty where the vocabulary is silent; the body hash
  moves when a consequence is added and not when the typed facts are
  reordered; the draft's body carries an empty set. Mutation: the
  planner passing an empty set (the pre-slice shape) fails the equality
  test.
- ADR-0052 D6's carriage sentence flips from *pending-in-body* to
  *delivered*; issue #371 closes on this act's merge with the record
  re-measured; the README #371 row and ADR-0052's "What stays open" are
  updated in the arc's WP-000 record commit.

**Sequencing:** slice 3p first (its own PR, `Work-Package: WP-010`), then
increment 12 (its own PR, `Work-Package: WP-060`) rebased on it — both
Rust, **one sitting (r41, VMID 9466) at the arc's head**, named in both
PR bodies before the first merge; the WP-000 record (this plan, the
#371 closure, ADR-0052's open-list update) lands after.

## 4. Pricing

No spec text moves — §6 already requires the item; ADR-0023 already
fixed its form. No ADR: the decisions above are ADR-0023's and MODEL-006's
applied. Body version bump is a MODEL-003 schema change recorded in
`plan-body.md` and the CHANGELOG, as 3l/3m/3n were.

## 5. What would change this plan

- A consumer that must *query* consequence facts (ADR-0023's revisit
  condition) — then typed carriage gets its own round; this slice's text
  item would stand beside it, not be replaced by it.
- A reason for the draft to carry consequences at emission — none is
  known; drafts are re-planned at binding.
- The TypeScript suite failing to reproduce a v5 vector — which would be
  a codec finding, not a schema one, and would stop the slice.
