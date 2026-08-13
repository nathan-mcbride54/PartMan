# ADR-0036: The scheme's own regions, and located occupancy

- Status: Accepted
- Date: 2026-08-13. Made on two adversarially reviewed rounds of the
  same day — the recommendation round
  (`docs/reviews/ISSUE-319_EXTENT_ABSENCE_ROUND_2026-08-13.md`) and the
  design round that settled the shape after two rival designs died on
  measured fatals; both untracked session artifacts, everything
  load-bearing restated here — under the decision owner's directive to
  start issue #319's planner half. Recorded and implemented in one
  arc: merging is not acceptance, and every element below is reviewable
  against the rounds' recorded alternatives.
- Spec version: 12.13.0 (minor under §0.1 — the argument is made in
  full below, with the major counter-argument recorded and declined)
- Work packages blocked: none. WP-060 increment 10 implements this and
  lands after; a cross-package obligation on WP-L100 increment 3 is
  recorded below.
- Requirement IDs: INV-004, PART-009, PLAN-001, ADR-0018, ADR-0023,
  ADR-0033
- Decision owners: Nate McBride

## Context

Issue #319 filed a fail-open in delivered code: a node present in the
topology with containment edges but absent from `Facts.extents` is
invisible to `free_extents`, which subtracts only children whose
`range.host == host` (`crates/planner/src/solve.rs:257`) and never
consults containment. The bytes read as free.

The defect is measured, not argued. At `ecb3dc6`, against the
**unmodified** delivered solver fixture:

```
free_extents        -> [(0, 1048576), (68157440, 36700672), (138412544, 935329280)]
place_create(1 MiB) -> start=0 length=1048576 end_placement=Aligned
```

A create of exactly `DEFAULT_ALIGNMENT` places a partition over the
protective MBR and the GPT header and records it as conforming. That
fixture's `PartitionTable` node carries a containment edge and no
extent. `align_up(0, 1 MiB) == 0` (`solve.rs:190-192`) with the skip at
`solve.rs:312` make **zero the only reachable sub-1 MiB start**, so the
defect is one range wide, not a class.

The fail-open has a committed witness asserting it is correct:
`free_extents_are_the_hosts_minus_its_children` asserts the first free
range is `(0, DEFAULT_ALIGNMENT)` (`crates/planner/src/tests.rs:583`),
claimed under PLAN-001 at `docs/traceability/WP-060.md:32`.

Two further defects surfaced by attacking the fix, both against
§11.2's "Extents remain inside the bound device"
(`AGENT_BUILD_SPEC.md:856`): a host whose extent exceeds the size its
own naming fields declare (measured — a 1 GiB device carrying a 2 GiB
self-extent placed a 1.5 GiB partition, recorded `Aligned`), and a
child extent leaving its host. Both are closed here.

## The decision

> A host's free extents are its own extent minus the extents the facts
> place on it **and minus the regions the table schemes it declares
> claim at each end**. A scheme's claim is a **bound, never a
> measurement**. Separately, every partition the authenticated naming
> fields place on the host must be one the subtraction actually
> removes, at the offset its own hashed name declares — where it is
> not, free space is not computable and the solver refuses.

The claim is derived from the table node's own `TableRole`
(`crates/domain/src/model/naming.rs:68-84`), which rides in the node's
hashed address preimage (`derive_id`, `naming.rs:383-395`) and is
therefore authenticated rather than asserted.

**Why a bound.** No sector size reaches this module — the value lives
only as `logical_sector_size: Option<u64>` in the identity envelope
(`crates/domain/src/model/identity.rs:149`) and never arrives — and
`PartitionTable { parent, role }` carries no geometry
(`naming.rs:219-224`). A GPT's real head is 17,408 bytes at 512 B and
24,576 at 4Kn; its entry count is a header field this module never
sees. The honest form is the smallest figure in the module's only unit
that covers the structures at every sector size.

### The reservation table

| `TableRole` | Head | Tail | Ground |
| --- | --- | --- | --- |
| `Gpt` | `DEFAULT_ALIGNMENT` | `DEFAULT_ALIGNMENT` | Head covers LBA0's protective MBR, LBA1's primary header, and the entry array; tail covers the backup array and header at the last LBAs. |
| `HybridMbr` | `DEFAULT_ALIGNMENT` | `DEFAULT_ALIGNMENT` | **A fail-closed choice, not an entailment.** Nothing in `TableRole` makes a `HybridMbr` node imply a `Gpt` sibling — `derive_id` hashes the role with no sibling dependency. Tail 0 would hand out the backup GPT whenever the sibling node is missing. |
| `Mbr` | `DEFAULT_ALIGNMENT` | 0 | The boot sector and embedded-loader gap are at the low end; the scheme defines nothing at the high end. A tail bound here would reserve bytes no structure claims. |
| `Apm` | `DEFAULT_ALIGNMENT` | 0 | Block 0's driver descriptor record and the partition map, low end only. Recorded as a floor: a map exceeding the bound is conceivable, but the map region is itself an `Apple_partition_map` entry, so a well-formed detection carries it as a located partition and the child subtraction covers it. |
| `Unrecognized { raw }` | — | — | **Refuses.** Head/tail is the wrong *shape* of bound for an unknown layout — its metadata may sit anywhere, so reserving a known maximum is a guess wearing a bound's clothes and reserving nothing fails open. |
| No table node on the host | 0 | 0 | Reserves nothing, and does **not** refuse on that ground: refusing on an absent node manufactures a refusal from absence, the mirror of manufacturing free space from it. The hole is closed positively instead, by the occupancy rule's foreign-table arm. |

The `Mbr`/`Apm` tail is not a judgment call — it is forced by the
delivered suite. Mutating it to a reserved tail fails
`no_authored_boundary_has_a_fourth_state` and
`misaligned_growth_authors_only_the_aligned_end`: the delivered
ADR-0023 suite already prices MBR over-reservation.

### The occupancy rule

A partition the authenticated naming fields place on this host must be
one the subtraction removes, on five closed grounds — no range at all;
a range in another host's address space; a range on this host that is
empty and so removes nothing; a range that does not begin where the
occupant's own hashed name declares; and an occupant located under a
table view this host does not carry.

**Located-ness is not key presence.** `contains_key` is not what the
arithmetic consumes; the filter at `solve.rs:257` is. The grounds are
written so that the guard's notion of "accounted" is identical to the
subtraction's own behaviour: there is no state in which the guard says
accounted and the subtraction removes nothing. This is the whole reason
the rule exists in this shape, and it is what defeated the rival
designs.

**Occupancy is read from the naming fields, never from
`topology.edges()`.** An edge rides in no node's address preimage, so
an edge-sourced roster shrinks silently when one is omitted. Held as a
property: the free list must be byte-identical with and without every
containment edge.

**`ConflictingTableEntry` is not an occupant, by construction** — the
occupancy match names `NamingFields::Partition` and nothing else, read
off the delivered type. It cannot be one: the variant carries
`entry_start` and **no length**, so no bound over it is computable.
ADR-0024's repair family is structurally out of the solver's reach —
`plan_repair` reaches its regions through `located_table_regions`
(`crates/planner/src/lib.rs:1210-1224`), and no path from it enters the
solver.

## Options considered

### Keying the guard on extent presence

Rejected on a measured fatal. Requiring that every extent-bearing kind
carry an entry in `Facts.extents` — including at the constructor, with
the defective snapshot made unconstructible — proves a key exists and
proves nothing about the `HostRange` the key holds. Every reach in the
system is host-qualified (`HostRange::intersects` opens
`self.host == other.host`, `crates/domain/src/model/protection.rs:42`).
With every roster kind populated and a buried occupant anchored to its
containment parent, the create was still placed over it. This is the
same fatal that issue #333 records, reached independently.

### Refusing outright on an unlocated table

Rejected. `free_extents`' module doc says the solver "does not invent
one — the math is over what the body authenticates"
(`solve.rs:5-9`), which reads as an argument for refusing rather than
subtracting a synthetic region. But refusing on an absent node
manufactures a refusal from absence — the mirror of the defect — and
the sentence conflated two responses to absence. **A bound read off an
authenticated `TableRole` is not an invention**: the role is body-carried
and hashed. That module doc is amended by this ADR.

### Requiring `ConflictingTableEntry` to carry an extent

Rejected: it bricks ADR-0024's repair family, and the variant carries
no length from which any bound could be computed.

### Including hosted layers in the occupancy roster

Rejected on measurement. Requiring backing signatures, file systems and
volumes to be located produced a **false refusal on a delivered
fixture** whose file system is partition-anchored — squarely inside
issue #333's unresolved ambiguity. Excluding them is what keeps this
decision independent of #333. It is the largest recorded residual.

## Consequences

- **Positive:** the filed defect closes — no accepted placement starts
  below the reserved floor. Two further §11.2 defects close with it.
  The guard reads only authenticated material, so it cannot be
  defeated by omitting an edge or a stamp.
- **Negative, accepted knowingly:**
  - **Head, every role: `DEFAULT_ALIGNMENT` where the structures need
    ≤24 KiB.** The measured cost is exactly the placements starting at
    offset zero — the entire class the defect consists of — and no
    others, since zero is the only reachable sub-1 MiB start. One
    delivered case loses a real accepted placement: a 32,256-byte
    create at offset 0 that the solver recorded `Coincident` —
    affirmatively conforming, on the boot sector.
  - **Tail, GPT and `HybridMbr` only: `DEFAULT_ALIGNMENT` where the
    backup GPT needs ≤132 KiB** — roughly 50× over-reservation, real
    bytes, untightenable here. A GPT grow-to-fill-the-device now stops
    one reservation short and is spelled coincident with the scheme's
    region rather than the device end, which is the point: filling to
    the physical end of a GPT disk overwrites the backup header.
  - **`TableRole::Unrecognized` refuses every create and grow on that
    device**, permanently, with no remediation in this vocabulary
    beyond locating a recognized scheme. Verified non-redundant: such a
    device plans today, since `own_arm` returns `Permitted` for every
    `PartitionTable` regardless of role (`protection.rs:395-408`) and
    `TableRole` appears nowhere in `crates/capability`.
  - **Refusals name numbers but not causes** — `NoFitForSize` does not
    say "1 MiB is reserved for the backup GPT". Mitigated by the
    reservation being a public computation a surface can render beside
    the numbers; mitigation, not closure.
- **Amended, and recorded rather than edited away:**
  - `solve.rs`'s "the solver does not invent one" sentence, per the
    rejected-route reasoning above.
  - **ADR-0018's "The extent accessor is total by domain restriction"
    passage** (`docs/adr/0018-si11-protection-closure.md:416-419`).
    That totality is unsatisfiable under the delivered carriage:
    `Facts.extents` is `BTreeMap<NodeId, HostRange>`, one range per
    node (`protection.rs:79`), so the passage's own
    "primary/backup/MBR/EBR extents" cannot be carried. **This
    amendment is prose recorded here — not an edit to ADR-0018, and
    not code.** The passage sits in that ADR's `## Safety analysis`
    (heading at `0018:54`), never in its Decision, and must be cited
    that way. A computed check over `Facts.extents` would live in
    `crates/domain`, which is WP-010's package and needs its own PR.
  - The committed assertion at `tests.rs:583`. Its fixture and its test
    name survive; the false tuple does not, and the test gains an
    assertion pinning the extent-less-ness it was silently relying on.

## The version argument

**Minor, 12.13.0.** Two requirements are touched and the amendment
states both: INV-004 gains the withholding **and** the unavailability
arm; PART-009's lawful coincident edges gain the scheme's tail region.

1. **The role is a detected input, and INV-004's clause is a list of
   them.** INV-004 says free extents are "computed at use from the
   detected inputs it names (partition extents, device geometry)" — the
   naming is done by the derivation, not by INV-004, so the
   parenthetical illustrates rather than closes the admissible set. The
   table scheme is detected: INV-003 names it explicitly
   (`AGENT_BUILD_SPEC.md:515`).
2. **The closed reading indicts 12.12.0 as shipped.** Delivered
   `free_extents` already consumes more than the parenthetical's two —
   the filter at `solve.rs:257` excludes nothing by kind, so signature
   and file-system extents are already admitted. A reading that makes
   the current text already false is not the reading that governs.
3. **Nothing INV-004 says becomes false.** Free extents remain a
   derivation under ADR-0033: recomputed on every call, stored nowhere,
   carrying no observation set. The clause's only MUST — the
   fail-closed presentation rule — is untouched and is in fact the
   ground for the new unavailability class.
4. **The head exclusion was never a lawful free extent.** Bytes holding
   the protective MBR or the boot sector are not free; reporting them
   as free was a defect *against* INV-004. §0.1 is explicit that the
   rule "is about requirements, not about whether anything implements
   them."
5. **PART-009 precedent is direct.** 12.1.0 added the coincident-edge
   rule and priced it minor because "the two pre-existing sentences
   stand verbatim … and no existing MUST narrows"
   (`AGENT_BUILD_SPEC.md:48`). Adding a third named edge to the same
   list is the same act.

### The counter-argument, recorded and declined

**The tail is the strong form, and argument 4 does not reach it.** On a
512 B GPT device the real backup structures are ~17,408 bytes and this
decision withholds 1,048,576. Those bytes *are* free, 12.12.0 reported
them as free, and 12.13.0 will not — under-reporting free extents is a
departure from INV-004's detect duty in the opposite direction from the
head fix. The answer offered is narrow: the exact region is fixed by
fields this layer never receives, so INV-004's own "MUST NOT be
presented as a value" clause applies to an input the module cannot
resolve, and a bound is the honest form.

**A decision owner who reads the tail withholding as narrowing
INV-004's detect duty prices 13.0.0** — and nothing in
`crates/planner` changes, only the version and the changelog row.
**The counter-argument was put to the decision owner in exactly that
form on 2026-08-13, with the 13.0.0 flip named, and 12.13.0 minor was
held.** It is recorded here so the pricing reads as a decision taken
against the objection rather than one made without it.

Two further cautions recorded. The 12.10.0 precedent is **not** "this
exact reasoning": its minor rested on the clause specifying something
"never previously specified", and that premise is spent — 12.10.0 is
what specified it. And ADR-0033:131-137 must be cited as its
`## Verification` item labelled "Delivered evidence, cited", not as
INV-004's normative surface; its actual words characterize the solver's
inputs as "body-carried authenticated extents", which is the sharper
objection — a `TableRole` **is** body-carried and authenticated, it is
simply not an *extent*.

## Verification

- No accepted placement starts below the reserved floor on any
  recognized scheme, and the sub-`DEFAULT_ALIGNMENT` size sweep finds
  none.
- The free list is byte-identical with and without every containment
  edge — the naming-field sourcing, held as a property.
- Identical output for a `Present` table state, an `Absent` one, and no
  `table_states` entry at all — the guard reads the table node, never
  the stamp.
- All five occupancy grounds asserted as whole values including the
  declared start; the scheme refusal ordered before the occupant
  refusal, on a body carrying both.
- An unrecognized scheme refuses whether or not the facts locate the
  table, `raw` verbatim, and no placement exists below the floor.
- `plan_repair` still plans on a repairable device carrying a
  conflicting entry.
- Each rule mutation-verified before proposal, every mutant killed by a
  named test.

## Revisit conditions

- **Table regions gain real extents** — plural extents per node, a
  backup `TableRole`, or table regions as distinct nodes. Any of these
  makes the tail bound unnecessary and it should be removed rather than
  kept beside a measurement.
- **Issue #333's anchoring question is decided.** Under either reading
  the hosted-layer exclusion can be narrowed: require those kinds
  located when, and only when, the host node is a `PhysicalDevice`,
  which both readings agree about.
- **A scheme's structures are measured to exceed a bound** — the
  `Apm` map is the named candidate, since its floor is recorded as a
  floor rather than a ceiling.
- **`crates/capability` or the protection layer begins reading
  `TableRole`**, which would make the unrecognized-scheme refusal
  redundant with an earlier gate.

## Forward obligations

- **WP-L100 increment 3.** The guard needs an authenticated
  `TableRole`, and the delivered Linux contract's field roster "is now
  fixed and contains no partition-table key at all"
  (`crates/adapter-linux/src/reach.rs:18-20`, pinned by
  `no_partition_table_key_is_in_the_roster_the_reach_describes`).
  Increment 3 must either designate a client-readable table-role source
  or record that the solver reserves nothing on Linux client drafts
  until HLP-002 re-discovery supplies a table node. Reading
  `facts.table_states` is not an available exit — this decision
  forbids it, so the guard stays clear of that package's charter.
- **The WP-W100/WP-L100/WP-M100 INV-004 surfaces** must name the table
  scheme among their inputs and fixture the unavailable arm — the
  obligation ADR-0033 already records against them
  (`0033:138-145`), extended by one named input.
- **Issue #319 stays open.** This decision is its planner half only.
  The authorization half — the protection layer's reach over an
  extent-less node — is unaddressed here and blocked on #333.
