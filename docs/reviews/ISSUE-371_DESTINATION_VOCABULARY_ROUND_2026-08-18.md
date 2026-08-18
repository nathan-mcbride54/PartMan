# Issue #371 round — PART-005's destination vocabulary

**Date:** 2026-08-18. **Base:** `522a587` (main), spec 17.2.0, plan body
version 4. **Directive:** Nate — "draft the §1.11 filing for PART-005",
then "run the adversarial review on D2 and D6", then "rewrite the round
around the trilemma".

**Revision 2.** Revision 1's D2 recommendation was broken by the
adversarial pass and is withdrawn. What it got wrong is recorded in §0.1
rather than edited away, because a round that quietly changes its answer
teaches the next reader nothing about why the first answer was
attractive.

> **A correction to the banner this round was drafted with.** Revision 1
> carried the header the ISSUE-348 round uses — "untracked local artifact…
> never stage into a commit; `verify-change-ownership` refuses it". That is
> **false**, and measured so here: `docs/reviews/**` is in WP-000's
> `owned-paths` block (`docs/work-packages/WP-000.md`), so this record is
> committable, and under a `Work-Package: WP-000` trailer the ownership gate
> admits it rather than refusing it. The banner is repeated across several
> round documents and is worth a sweep of its own.

## 0. Why this is not a §1.11 filing

Unchanged from revision 1, and unchallenged by the review. §0.2 and §1.11
are for one shape: *two requirements in this spec conflict*, so an
implementer must stop rather than pick a side. Every text bearing on a
move was read for that shape — PART-005, §6, PLAN-001/002/004/005/008,
PART-009, §11.2, ADR-0018's effect table and relocation duty,
ADR-0022/0023/0025/0033/0040. **None disagrees with another.** Each
stands verbatim; what is missing is a vocabulary the spec requires and the
model does not carry.

That is an undelivered requirement behind a hash-visible design decision,
whose practice here is a recommendation round on the owning issue,
adversarially reviewed, then a decision-owner call landing as an ADR with
its spec bump. The register stays untouched.

## 0.1 What the adversarial review broke, kept rather than erased

Six lenses attacked D2 and D6, each finding independently verified against
the delivered code, then adjudicated. Forty-seven agents. **Eight attacks
survived verification; thirteen statements in revision 1 were measured
false or unsupported.** The ones that changed the answer:

| revision 1 said | measured |
| --- | --- |
| "Reach is **not** lost under (a)" | **False for non-descendants**, which is the entire difference between the options. `descends_into` returns false when the source's extent is self-framed and the source is not the step target (`protection.rs:1081`), so with a partition as target the device cannot descend into anything hosted directly on it. A device-framed node inside S∩D is reached by no arm. |
| "`D \ S` still seeds every node the new bytes intersect" | **False.** The bytes the journaled copy writes are all of D. `seed_from_ranges` (`protection.rs:917-943`) seeds only where a declared range intersects a declared extent, so S∩D seeds nothing under (a). |
| "(b) destroys it and releases what it describes; (a) reaches it as carried content" — the *only* named behavioural difference | **False in both halves.** `destroy()` fires only on the step target or by carry from an already-destroyed source (`protection.rs:739-743`), and the module says so itself at `:767-771`: "A range-destroyed node that is not the target is reached, and descends, but establishes no destruction of its own." And `endpoint_pair_allowed` lists no `("partition","partition-table")` Containment pair (`topology.rs:352-365`), so a table can never be content a moved partition carries. |
| "the solver's freeness rule refuses it" — the whole case for (a) | **No such refusal exists.** `PlanStep::mutating_declared` (`step.rs:437`) runs the acknowledgment law and `affected_set` and performs no occupancy check; `free_extents`/`place_create` (`solve.rs:614`, `:630`) take no candidate range and have one product-code caller. Revision 1's own §1 table measured this and D2 contradicted it. |
| "(b) either forbids the overlapping move or needs a solver special-case" | **Neither branch discriminates.** D3's `∪ S` rule is needed identically by (a), because it is D — not `consumed` — that overlaps S. Revision 1 called that rule incoherent at D2 and recommended it at D3. |
| D6: "the preserve arm discharges for the signature **families** the byte layer parses" | **Over-general.** The property doing the work is the **frame**, not the family. `snapshot_tests.rs:1560-1571` carries a device-framed `Mdraid1x` at 600 MiB and a device-framed `Xfs` at 700 MiB, and `endpoint_pair_allowed` admits both device-hosted pairs. |
| D6: "no parsed signature is lost — naming it one would assert a destruction that does not happen" | **False as written.** A device-framed parsed signature whose extent lies in S∩D is overwritten in place by the copy. The destruction happens. |
| D6: population = "FS-004 signatures and file-system kinds already in the bound snapshot" | **Structurally blind to its own paradigm case.** Partition type is on the naming exclusion list (`naming.rs:200-205`) and `NamingFields::Partition` carries only `parent_table` and `start_offset` (`naming.rs:232-237`). The committed `bios_boot_gpt()` fixture (`protection_tests.rs:1465`) builds a bios_grub partition — content defined by a blocklist of absolute LBAs — with no `FileSystem` child and no `BackingSignature` child. The basis enumerates over it and returns nothing. |

Two attacks that did **not** survive, recorded so they are not re-raised:
the naming-re-derivation attack fails against decided text — ADR-0019:93-96,
"A moved partition renames: its new address is its new position… addresses
are not identities" — and the planner-only-enumeration attack fails on
revision 1's own words, which say "delivered-in-planner, **pending-in-body**,
stated in those words rather than claimed whole".

Revision 1 named the reach argument as the place to attack and staked its
"minor under §0.1" pricing on it (D7). The review broke it, so that
pricing is unanswered and is re-made in §4 below.

## 1. What is delivered, measured at `522a587`

Read off the code. No planner or domain path has changed since #371's
2026-08-18 re-measurement at `12c13b5`, so its citations still hold; these
extend them. Rows marked **†** were added by the adversarial pass and are
the ones that decide this revision.

| fact | where |
| --- | --- |
| A request is `PlanRequest { operation, target }` — one node, no second operand | `planner/src/lib.rs:70-75` |
| The sized vocabulary is `Create { host, size }`, `Grow { target, new_length }`, `Shrink { target, new_length }` — a size or a length, never a **position** | `:396-418` |
| `Move \| Copy` are `NotRepresentable`: "moves and copies need a destination vocabulary this model does not carry yet" | `:376-377` |
| That absence is pinned by `no_representable_request_relocates_bytes` with a compile-time tripwire | `planner/src/tests.rs:3961`; PR #433 |
| The body already carries every step's `consumed` and `destroyed` ranges (v4) | `plan.rs:785-790`, `:951-952` |
| The plan layer declares **real** ranges: shrink `destroyed: vec![freed]`, create `consumed: vec![placed]`, grow `consumed: vec![extension]` | `lib.rs:991-995`, `:920-923`, `:955` |
| **†** `seed_from_ranges` seeds a node only where a declared range intersects that node's **declared extent** | `protection.rs:917-943` |
| **†** `descends_into` refuses a self-framed extent as a descent source unless it is the step's own target | `protection.rs:1081` |
| **†** `destroy()` fires only on the step target, the ADR-0048 extentless-identity arm, or by carry from an already-destroyed source | `protection.rs:739-743`, `:767-771` |
| **†** No `("partition","partition-table")` Containment pair exists; `("physical-device","backing-signature")` and `("physical-device","file-system")` do | `topology.rs:352-365` |
| **†** `resolve_step_output` destructures **exactly one** consumed range and matches world extents by **exact equality**: `let [created_range] = …` then `*extent == created_range` | `plan.rs:1428-1452` |
| **†** ADR-0018 defines consumed as "*consumed free ranges* (verified by the constructor to intersect no existing node's extent — Section 11.2's overlap invariant enforced at construction)" | `ADR-0018:135-136` |
| **†** Nothing enforces that: the sole constructor runs the acknowledgment law and `affected_set` only | `step.rs:437`; and the stale doc-comment claiming otherwise at `protection.rs:620` |
| **†** `host_geometry` already subtracts every extent with `**node != host && range.host == host` — deep descendants included | `solve.rs:565-598` |
| **†** `names_within(topology, node, host)` — "whether `node`'s own name positions it inside `host`, at any depth" — is delivered | `protection.rs:491-493` |
| **†** Device-framed signatures and file systems are committed body content | `snapshot_tests.rs:1560-1571` |
| **†** `docs/traceability/WP-060.md:25` is the **sole** PART-005 evidence row in the tree | ibid. |
| Reversal material is closed at `CreateDraft`, `GrowDraft`, `Impossible` | `lib.rs:1015-1031` |
| `Consequence` has two variants and rides the planner's output, not the body | `lib.rs:263`, `:272`, `:333` |

Two conclusions from revision 1 survive intact and are load-bearing:

- **A move step needs no new body field.** Its shape is the delivered one
  — `target` plus declared ranges. The vocabulary gap is at the request
  layer and in the solver.
- **The destination is reachable by the closure once declared**, because
  `seed_from_ranges` reads ranges, not operations. #370's "the closure
  cannot see the operation" is true and irrelevant here.

## 2. What the record already decides

Unchanged, cited, and not re-argued: severity 3; `irreversible-after-start`
**not** carried (the journaled copy is ADR-0025's unflagged fixture);
`checkpoint-cancellable`; the source extent destroyed at commit because
"its content ceases to be referenced"; the moved subtree reached, ADR-0040
having retired the exemption; the reversal an ordinary draft targeting the
forward step's output; a destination boundary **authored** under PART-009
with no fourth state; "in the plan" reading as the hashed body's §6
consequence-text item, whose arrival is a WP-010/WP-060 joint slice
already named at `plan.rs:11-13`.

Added by the review: **a moved partition renaming is decided, legitimate
behaviour**, not a destruction to enumerate (ADR-0019:93-96).

## 3. The trilemma

This is the round's central finding and revision 1 posed it nowhere. It
is what decides D2, and it was assembled by no single attacker — three
constraints, each measured separately, that cannot hold together.

For an **overlapping** move (the mode PART-005 names second and mandates),
with source extent S and destination extent D, S∩D ≠ ∅:

> **(i) Closure reach.** Every byte the journaled chunk copy rewrites —
> all of D, plus the released S\D — must lie in
> `destroyed ∪ consumed ∪ written`, or `seed_from_ranges` does not seed
> the nodes those bytes touch and the protection closure never sees them.
>
> **(ii) The delivered step-output contract.** `resolve_step_output`
> destructures the forward step's `consumed` as **exactly one** range and
> matches candidates by **exact `HostRange` equality**. For the reversal
> to resolve the moved partition, `consumed` must be `[D]` — the moved
> partition's post-move extent, entire.
>
> **(iii) ADR-0018's consumed class.** Consumed ranges are "verified…
> to intersect no existing node's extent". When S∩D ≠ ∅, D intersects the
> moved partition's own pre-move extent, so `consumed = [D]` violates the
> definition.

**(ii) forces `consumed = [D]`. (iii) forbids it.** One of the three must
give, and which one gives is the decision.

| option | (i) reach | (ii) step output | (iii) ADR-0018 |
| --- | --- | --- | --- |
| **(a) precise** — `destroyed = S\D`, `consumed = D\S` | ✗ S∩D in neither set | ✗ sub-range of D, zero candidates | ✓ |
| **hybrid** — `destroyed = S`, `consumed = D\S` | ✓ | ✗ fails identically to (a) | ✓ |
| **(b) conservative** — `destroyed = S`, `consumed = D` | ✓ | ✓ | ✗ |

The hybrid is worth naming because it is the intuitive minimal fix — two
independent verifiers proposed it as the smallest correct repair, and
**neither checked it against `resolve_step_output`**, where it fails
exactly as (a) does. It closes the reach hole and satisfies ADR-0018, and
it still cannot spell a reversal target.

**(i) and (ii) are the safety property and the delivered mechanism.
(iii) is decided text that can be amended.** That ordering is the whole
argument, and it is what §5 asks the next pass to attack.

## 4. The decisions

### D1. What a destination is — **unchanged**

`Move { target, new_start: u64 }`, an authored byte offset in the host's
address space. Rejected: a free-extent reference (D1b), because free
extents are a derivation "recomputed at use, never stored" (ADR-0033) and
an index into one names a value that does not survive to the body.
`Copy` is **out of scope** — separate CAP-002 operation, content class in
the effect table, cross-device identity story, and its own §11.2
invariant. Unattacked.

### D2. What the step declares — **REVERSED: (b) conservative**

**`destroyed = S`, `consumed = D`, `written = T`.**

It is the only option satisfying both (i) and (ii). S∩D lies inside
`destroyed = S`, so every node the move rewrites is seeded; and `consumed`
is the single range exactly equal to the moved partition's post-move
extent, so D5's step-output spelling keeps working without touching
`resolve_step_output`.

**What it costs, priced honestly this time.** Not a solver special-case —
that cost does not exist and would be shared with (a) anyway. The cost is
**a stated exception to ADR-0018:135-136**: for a relocation whose
destination overlaps its source, the consumed range may intersect the
moving node's own pre-move extent, and no other node's. That is decided
text being changed, and under the ADR-0040 precedent this round already
cites, it owes an ADR-0018 amendment and a §0.3 changelog row — **not**
the silent "minor" revision 1 assumed.

The exception is narrow and statable as an invariant: *`consumed` may
intersect the extent of the step's own target and of nodes named within
it, and no other.* `names_within` (`protection.rs:491-493`) is delivered
and expresses exactly that.

The canonical request-less entry stays as delivered — whole target extent
destroyed, conservative because it cannot know the geometry — on the split
`capability.rs:181-186` already draws for shrink.

### D3. What "free" means for a destination — **scoped**

`D ⊆ free_extents(host) ∪ S`, aligned per PART-009, inside the host,
outside ADR-0036's scheme-claimed regions.

The third clause must be **scoped**, not literal. Revision 1 wrote "`D`
not intersecting any *other* node's extent", which the review measured as
forbidding the very move it was written to enable: `host_geometry` already
subtracts every extent framed on the host including deep descendants, so
the clause is inert over `free_extents` and operative only over `∪ S`,
where the only extents present are the target's own descendants'. The
delivered `solver_fixture(with_fs = true)` (`planner/src/tests.rs:1302-1312`)
frames its ext4 file system on the device at the partition's own start, so
every downward overlapping move of that ordinary partition was refused.

Scoped form: **`D` not intersecting the extent of any node that is neither
the target nor named within it** — the same `names_within` predicate D2's
exception uses.

Its real job survives scoping: refusing a destination laid over a
device-framed node inside S. **Named residue:** an extentless hosted node
is subtracted by neither `free_extents` nor this clause — ADR-0051's
pinned issue #319 shape-3 limit, inherited here and stated rather than
discovered later.

### D4. Whether the copy mode is body content — **unchanged**

Derived from `S ∩ D ≠ ∅`, both ranges being body content. A stored mode
that could disagree with the ranges is the authored-value-never-validates
class ADR-0041 refuses. Unattacked.

### D5. The reversal draft — **unchanged, and now actually works**

`ReversalMaterial::MoveDraft { source: S, destination: D }`, target spelled
as the forward step's output, with a `DraftPrecondition` that S\D is
unoccupied at the reversal's own validation. Under D2(b), `consumed = [D]`
resolves; under (a) or the hybrid it would not, and
`emit_reversal(…)?` at `lib.rs:860` would refuse **the whole forward
plan**.

### D6. What ADR-0018's enumeration covers — **shape stands, population and two sentences replaced**

**The preserve arm is about the frame, not the family.** It discharges for
signatures and file systems anchored in the **moved node's own frame** —
their offsets are host-relative, so a byte-preserving copy preserves them
intact. It does **not** discharge for a signature or file system framed on
an ancestor, whatever family it is; `endpoint_pair_allowed` admits
device-hosted pairs and the tree carries committed instances.

**A device-framed node inside S∩D is destroyed, and the closure must be
what says so.** Revision 1's "a destruction that does not happen" was
false. Under D2(b) the authenticated closure refuses it, which is the
right side of this repository's standing rule that protection is computed
and authenticated rather than asserted at an advisory layer. Under (a) it
would have been decided only by an unauthenticated solver predicate —
another argument for (b).

**The variant is kind-level, and its silence means nothing.** Its finest
discrimination is `fs_kind` plus signature `family`. It cannot say "this
is an ESP" — no partition type or role exists anywhere in the bound
snapshot. So the ADR text must **bound its negative space explicitly**: a
Consequence vocabulary advertised as naming position dependence, silent on
a bios_grub partition, would read as positive evidence that no position
dependence exists. It is not a boot-consequence verdict.

**The "checking boot consequences" justification is withdrawn.** The
population cannot address that clause with a basis structurally blind to
partition type. Either drop the justification, or state a named dependency
on INV-004's partition-type/flags detection — owned by
WP-W100/WP-L100/WP-M100 — before the population claims it. Revision 1
claimed the clause it could not reach.

Carriage is unchanged: **delivered-in-planner, pending-in-body**, stated
in those words.

### D7. Where the change lands, and what it prices — **re-priced**

- **Planner (WP-060):** `SizedRequest::Move { target, new_start }`, the
  scoped D3 rule, the D2(b) declaration, `MoveDraft`, the `Consequence`
  variant, and the tripwire down **in the same change** as its producer.
- **Domain (WP-010):** **no body field, and no contract change** — which
  is now an argument for (b) rather than an assumption. Under (a)+D5,
  `resolve_step_output`'s resolution contract (`plan.rs:1428-1452`) would
  have had to change; revision 1 priced only a doc-comment fix.
  Independently: the stale `consumed` doc-comment at `protection.rs:620`
  is corrected to say where freeness is actually enforced — a
  documentation-versus-code defect worth fixing whether or not a move
  lands.
- **Spec + ADR:** PART-005's text stands verbatim. **ADR-0018:135-136 is
  amended** with D2(b)'s overlap exception, which is decided text
  changing and carries a §0.3 changelog row. Whether the aggregate is
  minor or major under §0.1 is now an **open question for the decision
  owner** rather than a claim this round makes: revision 1's "minor" was
  staked on a reach argument that has since been broken.
- **§11.2:** `:894`'s interrupted-move invariant is an apply-time
  obligation owned by the executing packages; WP-060 holds PART-005's
  planning half only. Named here as a **deferral** so the decision owner
  sees the gate, which revision 1 did not show.
- **Traceability:** `no_representable_request_relocates_bytes` is the
  **sole** PART-005 evidence row in the tree
  (`docs/traceability/WP-060.md:25`). Taking the tripwire down without a
  successor leaves PART-005 with no traced evidence at all. The change
  must name its replacement row.
- **Sitting:** any Rust trips WP-020's stopping condition; named in the
  PR body before the merge.

## 5. Open questions this round does not answer

Surfaced by the review, and genuinely for the decision owner:

1. **Is (iii) the right constraint to break?** The round argues it is, on
   the ground that (i) is a safety property and (ii) is delivered
   mechanism while (iii) is decided text. A contrary reading — that
   ADR-0018's consumed class is load-bearing for §11.2's overlap
   invariant and (ii) is the thing that should change — is not obviously
   wrong, and would mean amending `resolve_step_output` instead.
2. **What does the absence of a Consequence assert?** D6 names the need
   for an explicit bound; the words belong in the ADR.
3. **Does a forward step ordered after a move need a target spelling that
   survives the re-addressing?** No delivered boundary breaks today —
   each plan binds one snapshot — but the pre/post mapping is unassigned.
   Likely a WP-050 obligation; it should be stated, not discovered.
4. **Is the §11.2 interrupted-move invariant's deferral acceptable**, or
   must a WP-070 dependency be named before a move request lands?

## 6. What would change this round's mind

- A fixture showing D2(b) over-refusing a lawful move that (a) permits,
  where the refusal is *not* warranted. The review found the converse
  twice; the symmetric case was not tested.
- A reading of ADR-0018:135-136 under which `consumed = D` needs no
  exception — for instance, if "existing node's extent" is read as
  excluding the step's own target by construction. The text does not say
  so, but it was written before a relocation existed to test it.
- The consequence-text body slice landing first, which would remove D6's
  "pending-in-body" qualification.

## 7. Next acts, in order

1. Decision-owner call on D1–D7 and the four open questions in §5.
2. If (b) is adopted: ADR amending ADR-0018:135-136, with the §0.1
   pricing decided rather than assumed, and the major counter-argument
   recorded.
3. WP-060's increment; the tripwire down with its producer and its
   replacement traceability row; sitting named before the merge.
4. #371's dependency list re-measured against the result. **#370 is not
   moved by any of this**, and says so.
