# Issue #333: which address space a child uses — recommendation round, 2026-08-13

Untracked session artifact, docs/reviews convention. Everything
load-bearing is restated in the ADR that lands the decision.

Three readings were put. One died on two measured fatals, one died on
three, and the survivor is recommended as a **rule with its enforcement
held** — because no enforcement of it has been measured green. The
round's largest finding is not about anchoring at all, and is filed
separately as issue #338.

**What is being decided.** `HostRange.host` (protection.rs:31-38) is
unvalidated: `assemble` stores facts verbatim (snapshot.rs:91-106) and
the decode path checks `extent_host` only for kind-misplacement
(snapshot.rs:421-431). `HostRange::intersects` opens
`self.host == other.host` (protection.rs:42), so the anchoring decides
reach exactly — and three committed fixtures disagree:
protection_tests.rs:126-132 anchors a signature on the device, while
the cross-language golden vector (body_vectors.rs:249-255) and
plan_tests.rs:356-363 anchor on the partition.

## What the round established

Findings measured in detached worktrees at `95680c6`. **No pass ran
`cargo xtask ci`** — every figure below is `cargo test` or hand-reading,
and nothing here may be called green until a gate run backs it.

1. **Reading (ii) is blind to partial destruction, and the delivered
   planner emits exactly that step.** `shrink_reduction` builds its
   freed range from `extent_of(target)` (solve.rs:757-761) and
   `plan_request` puts it in `destroyed` (lib.rs:986). Shrinking a ZFS
   pool member 256→128 MiB with labels at the tail: (i) refuses; (ii),
   **with its closure repair applied and proven**, gives `affected=2`,
   pool unreached, `constructs=true`. Only the anchoring differs.
2. **Reading (ii) reports `Clear` at the capability surface** for
   Shrink, Grow and Move over a protected member, and its repair does
   not reach that (see finding 5).
3. **Reading (ii)'s textual pillar dissolves.** `ExtentLocator::Range`'s
   "A byte range within the host node's own address space"
   (naming.rs:191) governs `NamingFields::BackingExtent`, and
   `backing-extent` appears in **no** Containment pair — the nine pairs
   are at topology.rs:244-254; its only pair is
   `HostBacking => [("backing-extent", "volume")]`. It never
   contradicted `Partition.start_offset`'s "the entry's start offset in
   the containment root's address space" (naming.rs:231). **Verified by
   hand.**
4. **The third reading — derived frames, the address space computed
   from naming fields rather than declared — died on three measured
   safety fatals.** It produced the round's only *positive* evidence
   for (i): the first same-bytes A/B through `plan_sized`, where
   partition-framed the planner plans a Shrink and a Grow of a live ZFS
   pool member and device-framed the identical bytes refuse.
5. **THE LARGEST FINDING, and it is not about anchoring.**
   `canonical_ranges` emits `destroyed: vec![]` for eight of twelve
   operations — `Move | Shrink | Grow | Create | Repair | Label | Uuid
   | Decrypt` — putting the extent in `written_table_extents`
   (capability.rs:150-174). In `affected_set`, `written_table_extents`
   and `consumed` intersectors enter `affected` **only**, never
   `range_destroyed` (protection.rs:230-249), and the fixpoint
   propagates solely from `range_destroyed`/`cascade_destroyed`
   (protection.rs:251-297). **For those eight operations the closure
   has no seed and never runs**; reach is one hop of same-frame extent
   overlap. **Verified by hand**, and measured on the unmodified
   committed `root_on_zfs`: a shrink destroying 128 MiB of a live vdev
   gives `affected=3`, pool reached **false**, `constructs` **true**.
   Filed as issue #338.
6. **Reading (i)'s enforcement was measured unbuildable as proposed.** A
   root walk instrumented at `assemble` (snapshot.rs:98) produced
   violations attributed to **14 committed tests** across three
   packages against a 622-passed baseline — including
   `the_guard_stands_with_every_containment_edge_removed`
   (planner tests.rs:2678), whose premise is a topology with containment
   edges stripped, and it refuses the fixture its own repair produces
   (`reversal_worlds` wires no partition→file-system edge,
   plan_tests.rs:401-409).
7. **"The roots are exactly {PhysicalDevice, Volume}" is false.** Six of
   eleven kinds never appear as a Containment target, and
   `endpoint_pair_allowed` lists *permitted*, not *required*, pairs —
   nothing in `Topology::build` obliges an incoming edge, so any target
   kind becomes a root the moment its edge is absent.
8. **Neither reading is normative today.** No "host-qualified",
   "address space per" or "containment-forest root" text appears in
   AGENT_BUILD_SPEC.md. The only normative address-space sentence is
   INV-004's 12.13.0 amendment (:522), which both readings agree about.
   So this is an **addition** — but MODEL-003 is a conjunction (:381),
   so any check at `assemble` narrowing what `schema_version` 1 accepts
   owes a bump, and that debt belongs to the enforcement PR.
9. **The delivered citation dangles.** protection.rs:28-29 cites
   "ADR-0018 2.11"; `grep -n '2\.11'` on that ADR exits 1 — its
   sections are titled (`## Safety analysis` :54, `## Decision` :476).
   The sentence spans 0018:420-421, under safety analysis.

## The readings

- **(i) root-expressed** (recommended, as a rule): every range in a
  containment forest is expressed in the containment root's address
  space.
- **(ii) host-named**: each range names its own host; nested spaces
  legal. **Dead** on findings 1 and 2, pillar dissolved by 3.
- **(iii) derived frames**: the space is computed from naming fields,
  never declared. **Dead** on three measured safety fatals; contributed
  finding 4's positive evidence for (i) and killed a per-kind roster by
  measurement.

## The adversarial pass on the recommended reading

1. **"(i) fixes the ZFS hole."** **Not sustained — it does not.**
   Finding 5's measurement is on a uniformly device-anchored fixture,
   the shape (i) endorses. (i) makes the coordinates uniform; it does
   not make the reach sound.
2. **"Every frame boundary is a hole under (i) too."** **Sustained.**
   `volume` is a Containment source and never a target
   (topology.rs:252-253), so a ZFS label on an LVM logical volume is
   volume-framed under (i) while its PV partition is device-framed, and
   the single hop cannot cross. Measured as a planned shrink destroying
   384 MiB of a live vdev with `Refused{Zfs}` present and never
   consulted. Under `Wipe` the same snapshot refuses, because `Wipe`
   seeds `destroyed`. **The frame boundary is safe only where a
   destroyed seed exists.** (Round-reported; not independently re-run.)
3. **"The enforcement is unbuildable."** **Sustained** for the
   edge-walk form (finding 6). A naming-field-derived predicate is the
   only candidate that survives
   `the_guard_stands_with_every_containment_edge_removed` — 14 → 13,
   one verdict flipped. Recorded as the front-runner, not as delivered.
4. **"Derive-and-replace deletes facts the system needs."**
   **Sustained**, against the delivered types: `parse_ranges` builds a
   `HostRange` from bytes with no topology and no node
   (plan.rs:1010-1022); `Precondition::HostUnoccupied` searches
   `extent.host == *host` (step.rs:251-257) and goes vacuous if a
   partition can never be a frame root; `OccupancyGround::
   RangeOnAnotherHost` (solve.rs:382-384) becomes unconstructible.
   **Derive-and-compare** keeps the second fact and can refuse a bad
   capture; derive-and-replace cannot.
5. **"The anchor is authenticated, not validated."** **Sustained.**
   `Topology::build` checks edge referents and endpoint pairs
   (topology.rs:181-185, 196-213, 238-263); nothing checks naming
   referents, so a naming-derived frame can be computed from a pairing
   the pair table forbids. A capture-side sweep is owed before any
   enforcement lands.
6. **"Regenerating the golden vector is cheap."** **Not sustained as
   cheap; sustained as owed.** It is a versioned act on the plan-body
   retirement precedent, and the enforcement PR carries it.

## Rejected, recorded

- **(ii) host-named** — two measured fatals, and its pillar was a
  misreading of a doc governing a kind outside the containment forest.
- **(iii) derived frames** — three measured safety fatals; its
  derive-and-replace strong form deletes facts the delivered types
  depend on (objection 4).
- **A per-kind roster** — killed by measurement inside (iii)'s pass.
- **Enforcing (i) by a topology-derived root walk at `assemble`** —
  measured unbuildable, 14 committed tests.

## Decision carried forward

**The rule is (i): a range in a containment forest is expressed in that
forest's root address space. Enforcement is held.** No enforcement has
been measured green, and the honest record says so rather than shipping
a rule whose only delivered form breaks 14 tests.

The ADR records: the rule; the front-runner enforcement
(naming-field-derived, **derive-and-compare** never derive-and-replace)
with its 14 → 13 measurement as a candidate rather than a delivery; the
capture-side referent sweep owed before enforcement; the golden vector's
regeneration priced as a versioned act carried by the enforcement PR;
the MODEL-003 debt owed by that PR, not by this ADR; and the corrected
citation — `protection.rs:28-29`'s "ADR-0018 2.11" replaced by
`EdgeKind::Containment`'s own doc, "Positional nesting inside one
addressable byte space" (topology.rs:26-29).

**Spec version.** The rule is an addition to previously unspecified
territory (finding 8), so **minor**. The counter-argument recorded and
declined: an addition that no delivered code enforces is arguably
editorial, and a reader could price it patch — declined because the
rule decides which of two committed conventions is lawful, which is a
requirement-shaped act even before anything enforces it.

## What this unblocks — and what it does not

It does **not** unblock issue #319's authorization half. That is blocked
on **issue #338**, the closure gap, which this round found and which
neither reading fixes. ADR-0036's recorded scoped fix — require hosted
layers located when the host is a `PhysicalDevice` — becomes
well-defined under this rule, and is WP-060's to take when it chooses.

## Open, for the decision owner

1. **Enforcement is held.** If you want it now, the front-runner is
   named and its cost is measured; nothing about it is green.
2. **#338 outranks this.** Eight of twelve operations run no closure at
   all. Deciding anchoring first was the smaller half, and the record
   should not imply otherwise.
