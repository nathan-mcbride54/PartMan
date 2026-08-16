# ADR-0046: The frame rule is enforced — the extent's frame, the edge, and the name agree, and occupancy is read as bytes

- Status: Accepted
- Date: 2026-08-16. Made on the measured round of 2026-08-16
  (`docs/reviews/ISSUE-333_ENFORCEMENT_ROUND_2026-08-16.md`, single-author
  with a fifteen-mutation battery, each mutation proven applied and
  each killed but one whose premise the act makes unconstructible;
  committed under WP-000 beside this act), standing on the adversarially
  reviewed anchoring round of 2026-08-13 that decided the rule
  (`ISSUE-333_ANCHORING_ROUND_2026-08-13.md`, ADR-0037), the
  precondition-reading record of 2026-08-14
  (`ADR-0037_PRECONDITION_READING_2026-08-14.md`) and the two rounds
  that discharged the precondition (issue #354, ADR-0045). Merging is not
  acceptance; the decision owner has not been put the question in
  person, and this ADR is where it is put.
- Spec version: **15.2.0 — minor under §0.1.** The argument is made
  below.
- Work packages blocked: none. Issue #333 closes here as filed, and
  issue #401 — found by this round — closes with it. ADR-0037's held
  status ends: its front-runner is delivered in the form it named.
- Requirement IDs: MODEL-002, MODEL-003, MODEL-005, SAFE-005, PLAN-008,
  ADR-0018, ADR-0022, ADR-0037, ADR-0041, ADR-0045
- Decision owners: Nate McBride

## Context

ADR-0037 decided the reading issue #333 was filed to force — **a range
in a containment forest is expressed in that forest's root address
space; `HostRange.host` names that root** — and held its enforcement,
because no enforcement had been measured green. It named the
front-runner (a naming-field-derived frame predicate, the only candidate
that survived `the_guard_stands_with_every_containment_edge_removed`),
the form (derive-and-**compare**, never derive-and-replace, which was
measured to delete facts the delivered types depend on), the
precondition (a capture-side referent sweep that refuses a pairing the
pair table forbids, so no frame can be derived through one), and the
freight (the cross-language golden vector and `plan_tests.rs`, unlawful
under the rule, regenerated in the same act with its MODEL-003
discharge). ADR-0045 discharged the precondition on 2026-08-16: every
naming referent resolves to a kind the endpoint-pair table admits as the
source of the relation the field names, checked at construction and
therefore at every decode.

Until this act, nothing validated `HostRange.host`. `validate_facts`
(ADR-0041) checked that an extent's host is absorbed and that a
containment child lies within its edge parent *where the frames are
comparable* — and left "a child in a frame its parent cannot be compared
against" alone by design, naming it ADR-0037's held enforcement. Issue
#333's own measurement stood: re-anchoring only the ZFS signature's
extent in `root_on_zfs` into its member partition's address space,
every extent still present, left the pool unreached and a whole-device
wipe constructing — ADR-0018's flagship refusal defeated without
removing a fact.

Building the enforcement found a second defect, filed as issue #401 and
closed here. `Precondition::violated_by` — ADR-0022's truthfulness
mechanism, by which a create's reversal draft refuses once data lands in
the created partition and a grow's once anything sits on the reclaimed
tail — read occupancy as **frame naming**: a node occupies a host if its
extent is *framed on* the host. Under ADR-0037's rule a partition is
never a frame, so on any capture framed as the accepted rule requires
both preconditions hold vacuously and the decayed reversal binds and
destroys. Measured at `43872c0` on the unmutated reading by re-framing
`reversal_worlds`' file system alone onto the device: the committed
decay regression fails. It was green only on the partition-framed
spelling ADR-0037 calls unlawful. ADR-0037 had seen the shape (`:134-144`)
and priced it against derive-and-*replace* only; the vacuity is a
property of the rule on a lawful population, whatever the enforcement's
form, and no fixture change makes the enforcement green without fixing
it. That is why this act is an arc of three pull requests rather than
one, in the order PR #377 set: the occupancy reading first (WP-010,
issue #401), the planner's fixtures next in a form valid under both
regimes (WP-060), then the enforcement (WP-010, this ADR).

## The decision

> **1. The frame rule is enforced, derive-and-compare, at
> `TopologySnapshot::assemble`.** For every extent a body declares, the
> containment root the node's own name leads to is derived —
> `frame_root`, walking the one naming field per kind that
> `naming_referent_rule` classifies as naming a containment source (a
> partition's table, a table's carrier, a signature's or file system's
> host, a conflicting entry's table) until a kind that names none (a
> physical device, a volume, a multipath node) — and compared with
> `HostRange.host`. A mismatch refuses the body with
> `FactError::ExtentFrameDisagreesWithName { node, declared, derived }`,
> the two facts named side by side; the declared host is never replaced.
> A backing extent is the one node the rule does not reach: its `host`
> is the one open naming field, it appears in no containment pair, and
> its range lives in its host's own address space (`ExtentLocator::Range`),
> outside every containment forest.
>
> **2. A containment edge agrees with the name.** Where a node's name
> embeds a containment source, the containment edge that nests the node
> names that source or the body is refused with
> `FactError::ContainmentEdgeDisagreesWithName { child, edge_parent,
> named_parent }` — the strength ADR-0045 held beside this issue. With it
> the three positional claims a body can make about a node — its name,
> the edge that nests it, its extent — are pairwise compared: name↔extent
> (rule 1), name↔edge (rule 2), edge↔extent (ADR-0041's rule 6, whose
> "child expressed in the parent's own space" and "incomparable frame"
> branches are unreachable on a body the first two admit and are
> collapsed to the one live branch, `contains`, which fails closed across
> frames).
>
> **3. Occupancy is read as bytes.** `Precondition::violated_by` finds an
> occupant of a host three ways, and a node found by any of them
> occupies: an extent framed on the host itself (ADR-0022's reading,
> kept, so nothing found before is lost); an extent lying on the host's
> bytes, compared in the frame the host's own extent is expressed in — a
> region translated through the host's extent into that frame, or the
> host's extent entire — with the host's own frame ancestors (its table,
> its device, read off the naming relation) excused and nothing else;
> and, for the whole-host form, a node whose own name positions it
> inside the host, extent or none. A host whose own extent is absent has
> bytes that cannot be located and is returned itself: honest absence
> fails closed at this arm as at every other. A `RegionUnoccupied` region
> stays what ADR-0022 made it — a span of the named node in the node's
> own coordinates, a relative claim about that node's bytes — and is not
> a frame claim; the frame rule governs extents.
>
> **4. The golden vector and `plan_tests.rs` are regenerated.**
> `snapshot-full-captured` and `node-entry-backing-signature-7` carry the
> mdraid signature's extent in the device's frame at
> `start_offset + primary_offset`; the fourteen other entries the
> generator reproduces are byte-identical; the TypeScript suite
> reproduces every byte and digest unchanged.

Every hop the derivation takes is a referent `Topology::build` already
resolved and kind-checked (ADR-0045), so the walk climbs strictly and
ends within the forest's depth; the enforcement is what ADR-0037:146-150
said it must not be computed without.

## Measured

At the candidate over `43872c0`, `cargo test --workspace` **678 passed,
0 failed**; `cargo xtask ci` exit 0; `cargo xtask cross-language` exit 0
with the regenerated vector.

| shape | at `43872c0` | under this ADR |
| --- | --- | --- |
| #333's measurement: `root_on_zfs`, signature re-anchored on its member, every extent present | pool unreached, `constructs=true` | `ExtentFrameDisagreesWithName{signature, member, sda}` at `validate_facts`; unrepresentable |
| the same re-anchored on the table, or on the sibling ESP | assembles | refused, `derived: sda` |
| the same with both table edges removed | assembles | refused identically — the frame is read off the name |
| the golden vector's former shape (signature framed on its partition) | assembles | refused; and the same forgery at the boundary decodes to the constructor's refusal, equal by value |
| `plan_tests.rs`' former shape (file system framed on its partition) | assembles | refused |
| a table framed on itself | assembles | refused |
| every extent-bearing node in one body holding a device forest, a volume forest and a multipath forest at every depth (17 nodes) × every absorbed node as candidate frame (21) | — | **340 refused, 38 admitted: exactly one lawful frame per forest node, and the backing extent admits all 21** |
| every containment edge in that body (16) re-sourced onto every other node (19) | — | 59 refused by the name, 245 by the pair table first, 0 admitted |
| a signature edge-nested under the device while named on the partition, at the boundary | assembles, decodes | `ContainmentEdgeDisagreesWithName`, equal by value |
| six committed layouts of `protection_tests.rs` — root-on-ZFS with and without table edges, the LUKS chain, the BIOS-boot GPT, the whole-disk vdev, the partitioned mdraid array with volume-framed extents | build | `validate_facts` `Ok(())` on every one |
| the decayed reversal over a lawfully framed capture (issue #401) | binds | `PreconditionFailed` — found by geometry and by name |
| `HostUnoccupied` over a partition with a disjoint sibling, its table and its device | — | holds: ancestors excused, siblings disjoint |
| `RegionUnoccupied` over the reclaimed tail, a file system ending 1 MiB before it | — | holds — byte-exact translation |
| a host with no extent, region or whole | — | the host itself, fail-closed |
| the "third strength" (edge agrees with name) priced across the workspace before being taken | — | 0 reds, 0 committed violations |
| a root-framed rule on a step's *declared ranges* at the step constructor, priced and **not taken** | — | 0 reds, 0 violations across every committed step; see *What stays open* |

**Mutation battery** (fifteen, each proven applied by content hash before
the run and restored by the reverse edit, hash-checked): dropping the
frame check is killed by four tests; comparing against the immediate
host instead of the root by forty-five; treating a node outside every
forest as its own root, leaving a root unchecked, or ignoring the open
rule, each by the enumeration alone (`the_frame_rule_reaches_every_forest_at_every_depth`
— the lens that would otherwise not have run); dropping the geometric
occupancy read, the naming read, the region translation, or the
unlocated-host arm, each by `occupancy_is_read_by_geometry_and_by_name`;
making the host's ancestors occupants by twenty; dropping ADR-0022's
framed-on-host reading by that test's one corner (a backing extent
framed on a bare device past its self-extent, which no other reading
sees — that reading is kept, and this is the case that keeps it live);
dropping the edge-name rule by its enumeration; dropping rule 6 by
ADR-0041's four; the planner's foreign-table arm returning nothing by
its helper test. **One survivor, recorded**: making rule 6 admit a child
in another frame — its premise is exactly what rules 1 and 2 make
unconstructible, and the enumeration shows every cross-frame spelling
refused before rule 6 is reached; the branch is collapsed rather than
kept.

## Options considered, and rejected

- **The topology-derived root walk at `assemble`.** Rejected by ADR-0037
  on measurement (14 committed tests, wrong in kind); not re-run.
- **Derive-and-replace.** Rejected by ADR-0037 against the delivered
  types; and this round found the same shape bites derive-and-compare
  through `violated_by` (issue #401) — which is why rule 3 exists rather
  than the form changing.
- **Enforce, and leave `violated_by` alone.** Not landable: the
  planner's decay regression has no form valid under both the old
  reading and the frame rule, so the enforcement cannot be green without
  rule 3, and rule 3 must land before WP-060's fixtures can move. That
  ordering is the arc.
- **Read occupancy by geometry only, or by name only.** Rejected:
  geometry misses the absent-extent spelling (#319's class, caught here
  for free by the name); the name misses a lying capture whose bytes lie
  in the host under another node's name; ancestors must be excused by
  name (a device's self-extent overlaps everything on it) — the three
  readings together are the honest superset and each is killed by a
  mutation.
- **Fail closed on every host without an extent, in the frame-rule
  half.** Not this act's: `validate_facts` refuses only what is
  positively unlawful, and an absent extent is honest absence (ADR-0041).
  Rule 3's unlocated arm is the one place absence is answered here,
  because a precondition that cannot locate the bytes it is about cannot
  hold.
- **A root-framed rule on step ranges.** Measured zero-cost and not
  taken: a range over a host-backed file's bytes is expressed in the file
  system's own address space (`ExtentLocator::Range`), and the rule as
  stated would refuse it — issue #365's open question. The delivered
  planner derives every range from extents and inherits root-framing;
  recorded, not decided.
- **Regenerating the vector as `SCHEMA_VERSION` 2.** Rejected as ADR-0041
  rejected it: the byte format, field shapes and parse rules are
  untouched; bumping would make every v1 body undecodable to migrate
  nothing.

## MODEL-003

Under the explicit-rejection limb, `SCHEMA_VERSION` left at 1, on PR
#362's and ADR-0041's precedent — with the debt ADR-0037 said travels
with the enforcement discharged here rather than deferred. The refused
population is bodies whose extent frame or edge disagrees with the name:
unlawful under 12.14.0's rule since 2026-08-13, unvalidated until now.
The one committed artifact in that population, the golden vector, is
regenerated in this act — two of sixteen generated entries move, by
exactly the two fields the rule speaks to (`extent_host`, `extent_start`) — and the
TypeScript suite reproduces it unchanged. No conforming artifact changes
meaning; every other committed body assembles.

## The spec-price argument

**Minor, 15.2.0.** The rule is 12.14.0's; §0.1's own reasoning — the
counter ADR-0037 recorded — is that a rule "is about requirements, not
about whether anything implements them", so enforcing it changes no
requirement. What this act adds is requirement-shaped and new: the
edge-name agreement, the occupancy readings, and the sentence Section 5
now carries. Bodies that decoded refuse, as under ADR-0041, which priced
exactly that minor. The patch reading (enforcement is implementation) is
recorded and declined for the same reason ADR-0041 declined it: what a
body may say narrows.

## Consequences

- **Positive:** the reach closure can no longer be defeated by
  anchoring — #333's measurement is unrepresentable on both construction
  paths, equal by value; a snapshot's three positional claims are
  pairwise consistent; ADR-0022's truthfulness holds on the lawful
  population, and holds more than before (the absent-extent spelling,
  the unlocated host); the two committed conventions ADR-0037 found are
  one; ADR-0037's held status ends.
- **Negative, accepted knowingly:**
  - **This still does not make the reach sound.** ADR-0037's frame
    boundaries remain (a ZFS label on an LVM logical volume is
    volume-framed while its PV partition is device-framed); every frame
    boundary is safe only where a destroyed seed exists (issue #338's
    territory, closed on its held half by ADR-0039). Uniform, validated
    coordinates are the premise the closure needed, not the closure.
  - **`OccupancyGround::RangeOnAnotherHost` and `TableIsNotThisHosts`
    are unreachable through a snapshot**, as `RangeIsEmpty` became under
    ADR-0041; WP-060 keeps both as the solver's own defence, asserted
    on `occupant_ground` and `occupancy_ground`.
  - **A backing extent's extent is framed on nothing in particular** —
    the rule does not reach it, by ADR-0037's own carve-out, and the
    enumeration pins that it assembles framed on any absorbed node. That
    is issue #365's open question, recorded as a limit and not a rule.
  - **A `RegionUnoccupied` region is a `HostRange` in a non-root frame
    by design** (the target's own coordinates). It is not an extent and
    the frame rule does not govern it; the doc says so. A reader who
    takes "every `HostRange`" literally will find this the one exception.
  - Two ADR-0041 branches are gone; ADR-0041's revisit condition
    ("ADR-0037's enforcement lands: rule 6's incomparable-frame branch
    should then be re-read") is discharged by the collapse.

## Verification

- `an_extent_framed_below_its_containment_root_refuses_at_both_boundaries`,
  `the_frame_rule_reaches_every_forest_at_every_depth`,
  `a_containment_edge_that_disagrees_with_the_name_refuses`,
  `a_containment_child_outside_its_parent_refuses` (re-pinned)
  (`snapshot_tests.rs`);
  `the_flagship_defeat_is_unrepresentable_and_every_layout_is_lawful`
  (`protection_tests.rs`); `occupancy_is_read_by_geometry_and_by_name`,
  `a_decayed_precondition_refuses_at_binding` (`plan_tests.rs`);
  `an_occupant_under_a_table_this_host_does_not_carry_refuses`,
  `an_unaccounted_occupant_refuses_naming_what_the_facts_carry_instead`,
  `a_decayed_reversal_refuses_instead_of_destroying` (planner);
  `snapshot_constructions_reproduce_their_recorded_bytes` and the
  TypeScript `body-vectors.test.ts` over the regenerated vector.
- Any text implying that a child extent may be framed on its immediate
  host, that a containment edge may nest a node in a parent its name
  does not embed, that ADR-0037's enforcement is held, or that
  `HostUnoccupied` reads frame naming alone, is an error against this
  ADR.
- Any claim that this ADR makes the reach sound, closes issue #319's
  authorization half, closes issue #338, or decides what frames a
  backing extent (issue #365), is an error against this ADR.

## What stays open

- **#319's authorization half**, unmeasured since #338 closed.
- **#365** — what hosts and frames a `BackingExtent`; this act carves it
  out and pins the carve-out.
- **A step's declared ranges are not frame-checked** at the constructor:
  measured zero-cost, held on #365 (a range over a host-backed file's
  bytes is expressed in its file system's space). The delivered planner
  derives every range from an extent and inherits root-framing.
- **`Partition.start_offset` against its extent's `start`** — a naming
  field versus a fact, both now in one frame; adjacent to ADR-0041's
  open "device extent against `total_bytes`" and not taken.
- **#397**, **#392**, **#370**, **#371** as filed.

## Revisit conditions

- **A kind is added to the Containment pair table**: `named_position`
  reads `naming_referent_rule`, so classify the new field there;
  `every_forest` should gain the new depth so the enumeration keeps its
  count honest.
- **Issue #365 decides a backing extent's frame**: the carve-out in
  `named_position` (`Outside`) is the one line to revisit, and the
  step-range rule priced above becomes decidable.
- **A `Precondition` kind is added**: it must say which of the three
  occupancy readings it uses and whether its host can be unlocated.
