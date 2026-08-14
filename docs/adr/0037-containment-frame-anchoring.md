# ADR-0037: The containment-frame anchoring rule

- Status: Accepted
- Date: 2026-08-13. Made on the adversarially reviewed round of the
  same day (`docs/reviews/ISSUE-333_ANCHORING_ROUND_2026-08-13.md`, an
  untracked session artifact; everything load-bearing is restated
  here), under the decision owner's directive to decide issue #333's
  anchoring reading. Recorded in one arc — merging is not acceptance,
  and every element below is reviewable against the round's recorded
  alternatives.
- Spec version: 12.14.0 (minor under §0.1 — an addition to previously
  unspecified territory; the counter-argument is recorded and declined
  below)
- Work packages blocked: none. **Enforcement is held**, so no package
  is waiting on code here. Issue #319's authorization half is blocked
  on **issue #338**, not on this.
- Requirement IDs: MODEL-002, INV-004, ADR-0018, ADR-0019, ADR-0036
- Decision owners: Nate McBride

## Context

`HostRange.host` decides reach exactly — `HostRange::intersects` opens
`self.host == other.host`
(`crates/domain/src/model/protection.rs:42`) — and nothing validates
it. `TopologySnapshot::assemble` stores facts verbatim
(`snapshot.rs:91-106`); the decode path checks `extent_host` only for
kind-misplacement (`snapshot.rs:421-431`). There is no containment-root
check anywhere in `crates/domain`.

Three committed fixtures disagree about which address space a child
inside a partition uses:

| fixture | child | anchored on |
| --- | --- | --- |
| `protection_tests.rs:126-132` | ZFS signature | the **device** |
| `body_vectors.rs:249-255` (cross-language golden vector) | mdraid signature | the **partition** |
| `plan_tests.rs:356-363` | file system | the **partition** |

The disagreement is load-bearing: re-anchoring only the ZFS signature
in `root_on_zfs`, with every extent still present, was measured to
leave the pool unreached and a whole-device wipe constructing —
ADR-0018's flagship destructive refusal defeated without removing a
fact.

The record already contained a rule, ambiguously and with a broken
citation. ADR-0018 states "Every range is host-qualified; one address
space per containment-forest root" (`0018:420-421`) — under
`## Safety analysis` (`0018:54`), **not** the Decision (`0018:476`).
`protection.rs:28-29` echoes it citing "ADR-0018 2.11", and **ADR-0018
has no section 2.11**; its sections are titled, not numbered.

## The decision

> **A range in a containment forest is expressed in that forest's root
> address space.** `HostRange.host` names that root. A child nested
> below the root — a partition's signature, a partition's file system —
> carries its extent in the root's coordinates, not its immediate
> host's.

This is the reading ADR-0018's safety analysis already stated and
`Partition.start_offset`'s own doc already assumed: "the entry's start
offset in **the containment root's address space**"
(`crates/domain/src/model/naming.rs:231`).

**Enforcement is held.** No enforcement of this rule has been measured
green, and this ADR does not ship one. See *Enforcement* below.

## Options considered

### Host-named — each range names its own host, nested spaces legal

**Rejected on two measured fatals.**

1. **Blind to partial destruction, on a step the delivered planner
   already emits.** `shrink_reduction` builds its freed range from
   `extent_of(target)` (`crates/planner/src/solve.rs:757-761`) and
   `plan_request` puts it in `destroyed` (`crates/planner/src/lib.rs:986`).
   Shrinking a ZFS pool member 256→128 MiB with labels at the tail:
   root-expressed refuses; host-named — **with its closure repair
   applied and proven** — gives `affected=2`, pool unreached,
   `constructs=true`. Only the anchoring differs; the extent is a fact,
   not a naming field, so the addresses are identical.
2. **`Clear` at the capability surface** for Shrink, Grow and Move over
   a protected member, which its repair does not reach.

Its textual pillar also dissolves. `ExtentLocator::Range`'s "A byte
range within the host node's own address space" (`naming.rs:191`)
governs `NamingFields::BackingExtent`, and `backing-extent` appears in
**no** Containment pair — the nine pairs are at
`crates/domain/src/model/topology.rs:244-254`; its only pair is
`HostBacking => [("backing-extent", "volume")]`. The two docs describe
different forests and never contradicted each other.

### Derived frames — the space computed from naming fields, never declared

**Rejected on three measured safety fatals.** It contributed the
round's only *positive* evidence for the accepted rule — the first
same-bytes A/B through `plan_sized`, where partition-framed the planner
plans a Shrink and a Grow of a live ZFS pool member and device-framed
the identical bytes refuse — and it killed a per-kind roster by
measurement on its way down.

### A per-kind roster — some kinds root-expressed, others host-named

**Rejected**, killed by measurement inside the derived-frames pass. The
Containment pair table lists *permitted* pairs, not required ones, and
says nothing about address spaces.

### Enforcing the rule by a topology-derived root walk at `assemble`

**Rejected as unbuildable, measured.** It produced violations
attributed to **14 committed tests** across three packages against a
622-passed baseline — including
`the_guard_stands_with_every_containment_edge_removed`
(`crates/planner/src/tests.rs:2678`), whose premise is a topology with
containment edges stripped, and it refuses the fixture its own repair
produces (`reversal_worlds` wires no partition→file-system edge,
`plan_tests.rs:401-409`).

The predicate is wrong in kind, not only in degree: six of the eleven
naming kinds never appear as a Containment target, and nothing in
`Topology::build` obliges an incoming Containment edge, so **any target
kind becomes a root the moment its edge is absent**.

## Enforcement — held, with a named front-runner and a named dead end

**Front-runner:** a **naming-field-derived** frame predicate. It is the
only candidate measured to survive
`the_guard_stands_with_every_containment_edge_removed` — 14 → 13, one
verdict flipped — and it is that test which made the edge walk
unbuildable. Recorded as a candidate with its measured cost, **not as a
delivery**.

**Its strong form is rejected against the delivered types.**
Derive-and-*replace* — computing the frame and dropping `HostRange.host`
— deletes facts the system depends on: `parse_ranges` builds a
`HostRange` from bytes with no topology and no node in scope
(`crates/domain/src/model/plan.rs:1010-1022`);
`Precondition::HostUnoccupied` searches `extent.host == *host`
(`crates/domain/src/model/step.rs:251-257`) and goes vacuous if a
partition can never be a frame root; `OccupancyGround::RangeOnAnotherHost`
(`crates/planner/src/solve.rs:382-384`) becomes unconstructible.
**Derive-and-compare** keeps the second fact and can refuse a bad
capture. Any enforcement lands in that form or not at all.

**Owed before any enforcement:** a capture-side referent sweep.
`Topology::build` validates edge referents and endpoint pairs
(`topology.rs:181-185, 196-213, 238-263`) but **nothing validates
naming-field referents**, so a naming-derived frame can be computed
from a pairing the pair table forbids.

## Consequences

- **Positive:** the coordinates become uniform, and the two committed
  conventions stop both being defensible. The rule is now citable
  rather than inferred from a safety-analysis sentence with a broken
  reference.
- **Negative, accepted knowingly:**
  - **The rule makes the coordinates uniform; it does not make the
    reach sound.** Every frame boundary remains a hole in the
    non-destroying arm. `volume` is a Containment source and never a
    target (`topology.rs:252-253`), so a ZFS label on an LVM logical
    volume is volume-framed under this rule while its PV partition is
    device-framed, and the single hop cannot cross. Measured as a
    planned shrink destroying 384 MiB of a live vdev with
    `Refused{Zfs}` present and never consulted. Under `Wipe` the same
    snapshot refuses, because `Wipe` seeds `destroyed`. **The frame
    boundary is safe only where a destroyed seed exists** — which is
    issue #338, not this ADR.
  - **The cross-language golden vector and `plan_tests.rs` are
    unlawful under this rule and are not corrected here.** Regenerating
    the vector is a versioned act carried by the enforcement PR, with
    its own MODEL-003 discharge (`AGENT_BUILD_SPEC.md:381` is a
    conjunction). Until then two committed fixtures contradict an
    accepted rule, recorded rather than hidden.
  - **Nothing here is gate-green.** Both rounds are `cargo test` and
    hand-reading; **no pass ran `cargo xtask ci`**, and the
    LVM-framed figure is round-reported rather than independently
    re-executed.
- **Corrections owed, recorded rather than made:**
  `protection.rs:28-29`'s citation of the nonexistent "ADR-0018 2.11"
  should be re-cited to `EdgeKind::Containment`'s own doc — "Positional
  nesting inside one addressable byte space" (`topology.rs:26-29`) —
  by whichever PR next touches that file under WP-010's grant. This
  ADR changes no code.
- **ADR-0036's recorded blocker is corrected.** That ADR says #319's
  authorization half is blocked on #333. It is blocked on **#338**.
  ADR-0036's scoped fix — require hosted layers located when the host
  is a `PhysicalDevice` — becomes well-defined under this rule and is
  WP-060's to take when it chooses.

## The version argument

**Minor, 12.14.0.** No "host-qualified", "address space per" or
"containment-forest root" text appears in `AGENT_BUILD_SPEC.md`. The
only normative address-space sentence is INV-004's 12.13.0 amendment
(`:522`), which every reading agreed about. This is therefore an
addition to previously unspecified territory, and no existing
requirement's text changes or narrows.

**The counter-argument, recorded and declined.** A rule that no
delivered code enforces is arguably editorial, and a reader could price
it **patch** under §0.1's editorial class. Declined: the rule decides
which of two committed conventions is lawful, and that is a
requirement-shaped act whether or not anything enforces it yet — the
same reasoning §0.1 states in the other direction, that the rule "is
about requirements, not about whether anything implements them"
(`AGENT_BUILD_SPEC.md:17`). **The MODEL-003 debt is not owed here**: no
check narrows what `schema_version` 1 accepts until enforcement lands,
and that debt travels with the enforcement PR.

## Verification

- When enforcement lands: a snapshot whose child extent names its
  immediate host rather than the containment root is refused, in
  derive-and-compare form, with the two facts named side by side; the
  capture-side referent sweep exists; the golden vector is regenerated
  in the same act with its MODEL-003 discharge.
- Any text implying that a nested address space is lawful, or that
  `ExtentLocator::Range`'s doc governs a containment-forest member, is
  an error against this ADR.
- Any claim that this rule closes issue #319's authorization half is an
  error against this ADR.

## Revisit conditions

- **Issue #338 is resolved.** If the fix makes the closure seed from
  something other than `destroyed`, the frame-boundary cost priced
  above changes and this ADR's negative consequence should be re-read.
- A kind is added that can be both a Containment source and target, or
  `Topology::build` begins obliging incoming Containment edges — either
  changes what "root" denotes.
- An enforcement is measured green, at which point the held status ends
  and the front-runner is either delivered or replaced on its own
  round.
