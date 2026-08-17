# ADR-0039: Carried-content reach, and a bounded descent

- Status: Accepted
- Date: 2026-08-14. Made on the adversarially reviewed round of
  2026-08-14 (`docs/reviews/ISSUE-338_REACH_ROUND_2026-08-14.md`, an
  untracked session artifact; everything load-bearing is restated here),
  under the decision owner's directive to work issue #338's held half,
  take the wide act, and pay the major price. Recorded and implemented
  in one arc — merging is not acceptance, and every element below is
  reviewable against the round's recorded alternatives.
- Spec version: **13.0.0 — major under §0.1.** The argument is made in
  full below, with the counter-argument recorded and declined.
- Work packages blocked: none. Issue #338's held half closes here;
  issue #319's authorization half is unblocked but not delivered.
- Requirement IDs: Section 2.1, MODEL-002, MODEL-003, SAFE-005, ADR-0018
- Decision owners: Nate McBride

## Context

ADR-0038 corrected the release half of issue #338 and **held** the rest:
defect **(b)**, partial destruction missing children outside the
destroyed sub-range, and defect **(a)** for the six operations that
destroy nothing. This ADR is that held half.

**The measurement that ends the hold.** `PlanStep::mutating_declared`
(`crates/domain/src/model/step.rs:391`, closure at `:417`) is the
constructor `parse_step` calls when a recorded plan body is re-validated
(`crates/domain/src/model/plan.rs:912`). **No capability gate sits in
that path** — deliberately, because the affected set is not body
content. On the committed `root_on_zfs` shape it accepted a declared
partial shrink truncating 128 MiB off a live ZFS vdev: the solver's real
freed tail is `[640,768) MiB`, the ZFS label sits at `[512,513) MiB`,
the label's bytes survive, and the old closure could not reach past
them. ADR-0012's unrepresentability axis was not discharged at that
boundary.

**And the six.** `Grow`, `Create`, `Repair`, `Label`, `Uuid` and
`Decrypt` declare no destroyed range, so they seeded no propagating
class at all. On `the_luks_descent_reaches_the_pool_below` all six gated
`Clear` with a `Refused{Zfs}` pool below.

## The decision

Two changes in `crates/domain`, one rule.

1. **Carried-content reach.** Descent runs from any node in the affected
   set, not only from the destroyed classes. A mutating step reaches the
   content its target carries. This is what gives the six operations a
   reach at all.
2. **A per-edge-target bound**, replacing the three negative
   `!range_destroyed.contains(&edge.target)` guards:

```rust
fn descends_into(topology, facts, kind, source, target) -> bool
```

with four clauses, each answering a measured failure:

- **An extent on a kind the decode path forbids one on is ignored.**
  `snapshot.rs` refuses an `extent_host` on an aggregate, volume,
  encryption layer or multipath node; the closure now reads the same
  `NamingFields::may_carry_extent` predicate rather than a second copy
  of the list. A fact the body format rejects must not steer reach.
- **A node's own address space is never a descent source.** A
  self-framed extent declares a frame, not destruction: every range on a
  disk lies inside a device's self-extent, so descending out of one
  captures every sibling.
- **Where both sides declare bytes**, descend into content that lies
  within the source, into content framed by the source, and into
  anything expressed in a frame this one cannot be compared against.
- **Where the source declares bytes and the child does not**, descend on
  the propagating arms — a product carries no extent by construction —
  and not under containment, where the child is a node positioned inside
  a known frame whose position is unstated, and admitting it would
  capture a sibling that merely lacks a fact.
  *(Amended in 17.2.0 by ADR-0051, on issue #319's authorization half.
  The behaviour this clause protects is kept; its stated ground was a
  mis-description. Measured: the capture it prevents is not a sibling's
  but a **partition table's** — a table's extent is its own header bytes,
  not the region it governs, so descent from a table into a partition
  must be refused, and for an extent-bearing partition the geometric
  comparison already refuses it. This clause was extending that refusal
  to extentless children under a name describing something else. Descent
  now admits an unlocated child where the containment pair is
  **geometric** and refuses it where the pair is **structural**, naming
  ADR-0041's `containment_pair_is_geometric` — which this arm had never
  consulted. The refusal on absence under a geometric pair was a
  fail-open: removing a ZFS signature's one extent fact took a whole-disk
  wipe from refusing every mutating operation to `Clear` on all ten, over
  a live pool.)*

**The invariant that makes this defensible: the act can never remove
reach.** Every arm the committed closure has is preserved, two are
widened, and the bound refuses only on a positive geometric
contradiction. That is not aesthetic. `facts.extents` is authored body
content that nothing authenticates — ADR-0037 decided the anchoring rule
and **held its enforcement** — so a predicate able to *subtract* reach
hands an author a lever on protection.

**The premise, generalized and enumerated.** ADR-0018's theorem premise
was "no backing or production edge targets a physical device"
(0018:180-184). Descent out of a destroyed node is unbounded on those
three arms, which is safe exactly while none of their pairs can target a
node declaring bytes of its own. So the premise becomes: **no
`Backing`, `Production` or `HostBacking` pair may target a kind that can
carry an extent**, with the extent-bearing set read off the decode rule
rather than hand-authored, enumerated over the endpoint-pair table as a
property test. That discharges MODEL-002's standing obligation
(`AGENT_BUILD_SPEC.md:378`) that the theorem be "re-proved under the
extended edge set as a property test", which no committed test did.

## Why four predicates were rejected first

Every one was green on the full workspace when it was proposed. Three
were killed by an adversarial pass and reproduced by hand before the
design changed.

| predicate | how it failed | measured |
| --- | --- | --- |
| bound against the source's own extent, extentless source **bars** | lost reach HEAD has: an extentless encryption layer could no longer reach a mapper carrying a device-framed extent | 6 → 3, refuse → construct |
| bound against any destroyed node's non-self-framed extent | protection became a function of `extent_host`, which nothing authenticates: moving that one field, node ids and body hash unchanged, turned `gate(Wipe, pv)` from `Unsupported` to **`Clear`** over a live pool | 3 forged frames, all `Clear` |
| no self-frame clause | a partition delete on an ordinary disk blocked on a stale end-anchored mdraid superblock hosted by the device | `Ok` → `Err(Indeterminate)` |
| admit extentless containment children | a partition delete captured a sibling that merely lacks an extent fact | ESP captured, `Ok` → `Err` |

**The standing acceptance test is therefore not "the suite is green".**
It is *the new closure is a superset of the committed one on the
attackers' own fixtures* — which is how the last two were caught, both
on trees whose committed suite was entirely green.

## Measured

On the committed fixtures, `5b795df` versus this act:

| measurement | before | after |
| --- | --- | --- |
| declared partial shrink at `mutating_declared` | **constructs** | `Reached { pool, Refused{Zfs} }` |
| (b) at the closure: affected / pool reached | 2 / no | 4 / **yes** |
| the six on the LUKS chain | `Clear` | **refuse** |
| the four destroying operations | refuse | refuse |
| source-class operations | `Clear` | `Clear` |
| sibling guard: ESP in set / pool in set | no / yes | no / yes |
| whole-disk ZFS, device and signature targets | 10/10 refuse | 10/10 refuse |
| ordinary disk: delete, shrink, stale superblock, extentless sibling | construct | construct |
| forged frame, ghost host, zero length, saturating length | 2 of 4 construct | **all refuse** |
| monotonicity in the declared ranges | **false** — a strict superset reached less | true |

The last two rows are gains the act was not designed for. A zero-length
or saturating extent is invisible to `HostRange::intersects`, so the
byte scan misses it and the old closure constructed; descent reaches it
anyway now. That is **masking, not fixing** — the validation holes are
issue #349, and one forgery (a label declared one byte before its
parent's start) still escapes both closures.

Monotonicity is restored because no negative guard remains: a larger
declared destroyed set can now only grow the closure. ADR-0038 had to
argue conservatism per operation by measurement precisely because that
was false; this supersedes that reasoning.

## Consequences

- **Positive:** every mutating operation over a target carrying
  protected content refuses, at the capability gate, at the plan
  constructor, and at body re-validation. Issue #338 closes. Issue
  #319's authorization half is unblocked.
- **Negative, accepted knowingly:**
  - **Availability cost on carried content.** `Grow`, `Label`, `Uuid`,
    `Repair` and `Decrypt` over a partition carrying a LUKS-wrapped ZFS
    vdev now report `unsupported`. This is the priced direction: §2.1
    requires the product to protect what it cannot mutate, and the
    controls show the cost lands only where something protected is
    actually below — an ordinary partition, an ext4 filesystem, and an
    LVM stack with nothing protected under it all stay `Clear`.
  - **The bound still reads authored facts.** It can only ever admit
    more reach than the geometry warrants, never less, but a body that
    declares a child one byte before its parent escapes the (b) fix —
    as it escapes the committed closure. Issue #349 owns the boundary.
  - **`canonical_ranges` still over-claims its written extents.**
    §2.1 says table writes target the table node's own extents, never
    the parent device wholesale; the delivered entry uses the target's
    own extent, which for a device target is the whole device. The
    whole-disk gates currently refuse **because** of that over-claim,
    so correcting it must land **after** this act and never before —
    inverting the order is how one rejected design opened five `Clear`
    gates over a live pool. Recorded here rather than filed, and it is
    the one item from this round with no issue of its own.
  - **Three defects stay open and filed:** the table-release under-reach
    (#347), ADR-0018's relocation exemption against ADR-0038's `Move`
    entry (#348), and the extent-validation holes with the
    `assemble`/`from_canonical_body` asymmetry (#349).
- **MODEL-003, discharged by the rejection arm.** A body lawful under
  the old closure that reaches a protected node under the new one fails
  to decode with a typed `StepRefusal`, wrapped as
  `PlanSchemaError::Step` — MODEL-003's own "explicit rejection", the
  arm the plan schema already uses for retired versions 1–3
  (`plan.rs:342-346`). `LINKED_SCHEMA_VERSION` is **not** bumped:
  bumping it would additionally refuse every *lawful* old body, which is
  strictly worse and buys nothing. No golden vector encodes an
  affected-set answer — `body_vectors.rs` and the shared vectors are
  green under the act. The residual with no artifact at all is a helper
  holding a hash-frozen reversal advertisement across the change; that
  is named, not solved.
- **No blast radius outside `crates/domain`:** `crates/capability`'s
  CAP-005 ground-equality enumeration and the planner's suite are green
  under the act, so no WP-050 or WP-060 PR is owed. Measured because it
  could have moved, not to confirm that it would not.

## The spec-price argument

**Major, 13.0.0.** Four normative sentences said containment descent is
bounded by the destroyed **ranges** — `AGENT_BUILD_SPEC.md:110`,
ADR-0018:151-152 (rule 2), ADR-0018's own Decision at :481-483, and the
§0.3 changelog row for 11.0.0 — and defect (b) *is* that bound. Fixing
it falsifies all four. §0.1's rule is "semantic changes to existing
requirements bump major", explicitly not a narrowing test, and the
house's own precedent states it in the same words: "**Major under
§0.1**: PLAN-008's and Section 6's existing texts change meaning"
(`AGENT_BUILD_SPEC.md:51`).

**The counter-argument, recorded and declined.** Reach is *added* and
nothing weakens, so a reader may price this as minor, as ADR-0037 was.
Declined: ADR-0037 added a rule in previously unspecified territory and
changed no existing sentence; this changes four. ADR-0038's
pre-commitment that the price is "minor, never major" (0038:196-198)
governed an act that moved no text and does not govern this one —
inheriting it would repeat exactly the mis-numbering §0.1 records "so
the rule is not read as optional".

**Not a §1.11 filing.** That register needs two conflicting
requirements, named and quoted, and there is no such pair. §2.1:110
against MODEL-002:378 is an unmet obligation, not a conflict — and this
act discharges it.

## Verification

- Every figure in the tables above, on the committed fixtures.
- Six mutations, applied with `Edit` and **proven applied** before each
  run. Five were killed by the committed regressions. **The sixth
  survived** — disabling the `may_carry_extent` filter left 104 tests
  green, which meant that clause had no coverage at all, exactly as
  ADR-0038's second correction did. The missing fixture was written
  before proposal: a mapper volume carrying an extent the body format
  forbids, hosting a ZFS member signature that carries none. It kills
  the mutant.
- The pair-table mutation `("aggregate","partition")` reds the premise
  property test rather than silently widening the closure.
- `cargo xtask ci`'s real exit code checked directly, not through a
  pipe.

## Revisit conditions

- **Issue #349 lands**, at which point the bound stops reading facts no
  layer validates, and the fourth forgery closes.
- **Issue #347 lands** (a destroyed table releasing its partitions),
  which adds a seed this act deliberately does not.
- **`canonical_ranges` is corrected** to §2.1's table-write sentence.
  The whole-disk gates must be re-measured in the same act.
- **ADR-0037's enforcement lands**, which turns the frame values this
  bound reads from unvalidated into checked, and may allow the bound to
  be tightened where it currently admits on mismatch.
- **A kind becomes able to carry an extent that could not before**, or a
  propagating edge kind gains a pair targeting an extent-bearing kind:
  the premise property test is the tripwire, and it reds by design.
