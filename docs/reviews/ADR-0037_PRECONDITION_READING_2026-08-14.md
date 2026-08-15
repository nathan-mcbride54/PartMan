# ADR-0037:217 — does the resolve-only sweep satisfy the precondition? 2026-08-14

Untracked session artifact, `docs/reviews` convention.

> **CORRECTION, same day.** Sections 1–2 stand: ADR-0037:217 is not
> satisfied, and #333's enforcement stays blocked. **Section 3's premise
> is false and section 6.3's recommendation is withdrawn.** The
> adversarial round in `ISSUE-354_FIXED_KIND_ROUND_2026-08-14.md`
> measured that MODEL-002's chain does not fix these referent kinds — it
> contains neither `backing-signature` nor `conflicting-table-entry`, it
> says in its next sentence that it is not exhaustive, and ADR-0019's
> naming map deliberately names no kind for Volume's producer — and that
> the fourth pair false-refuses every host-backed virtual device (loop,
> VHD/VHDX, dm-linear, plain dm-crypt), a population INV-001 and WIN-003
> require. Read that round, not this section 3.

**Question put:** ADR-0037:217 makes "the capture-side referent sweep
exists" a verification condition for #333's frame enforcement. PR #362
landed the resolve-only half. Does that satisfy it?

**Answer: no.** And the measurement taken to establish that produced a
second finding which changes what is available to do next.

## 1. The reading

`:217` uses the definite article — "**the** capture-side referent sweep"
— referring back to `:146-150`, which defines the sweep by the harm it
exists to prevent:

> "**Owed before any enforcement:** a capture-side referent sweep.
> `Topology::build` validates edge referents and endpoint pairs but
> **nothing validates naming-field referents**, so a naming-derived frame
> can be computed **from a pairing the pair table forbids**."

The named harm is a frame derived through a **forbidden pairing**.
Resolve-only refuses a referent that resolves to *nothing*; it does not
ask what a referent resolves *to*. A wrong-kind referent still builds —
pinned on purpose by
`a_wrong_kind_referent_still_builds_and_that_is_the_held_half`.

So the harm `:146-150` names is untouched, and the condition is
**not satisfied**. A literal reading ("some sweep exists") would satisfy
it, but that reading makes the sentence vacuous: any sweep at all,
including one that checked nothing, would discharge it.

**#333's enforcement stays blocked.**

## 2. Why it matters concretely, not just textually

ADR-0037's enforcement is derive-and-compare: derive the frame root from
naming fields, compare against `extent.host`, refuse on mismatch. For a
partition the derivation is

```
partition --parent_table--> partition table --parent--> root
```

Two hops, two fields. Resolve-only guarantees both resolve *somewhere*.
Neither is guaranteed to resolve to a node of the right kind, so the
derived "root" can be a node no containment relation would admit — which
is precisely the frame the ADR says must not be computable.

## 3. The finding: the eight referent fields are not alike

The rejected panel design treated all eight uniformly, deriving one kind
check from `endpoint_pair_allowed`, and died because that table cannot
express real layouts. But **four of the eight fields have a referent kind
fixed by MODEL-002's own chain**
(`physical device → partition table → partition → encryption/container →
volume → file system`), stated in the field's own doc comment, and need
no pair table at all:

| field | doc comment | lawful kind(s) |
| --- | --- | --- |
| `Partition.parent_table` | "the table view the entry belongs to" | `partition-table` |
| `EncryptionLayer.backing_signature` | "the LUKS/BitLocker signature node" | `backing-signature` |
| `ConflictingTableEntry.table` | "the table node whose views conflict" | `partition-table` |
| `Volume.producer` | "the producing aggregate or encryption layer" | `aggregate` \| `encryption-layer` |

The other four say "the node whose bytes carry X" (or, for
`PartitionTable.parent`, "the device the table describes" — which real
layouts contradict). Those are open, and they are exactly where #360
lives:

| field | why open |
| --- | --- |
| `PartitionTable.parent` | device \| volume \| aggregate — the #360 rows |
| `FileSystem.host` | any byte-carrying node |
| `BackingSignature.host` | any byte-carrying node |
| `BackingExtent.host` | "the file system or node hosting the extent" |

**All three layouts the panel measured as false-refused are open-field
cases** — two on `PartitionTable.parent`, one on `FileSystem.host`. None
is a fixed-field case. The panel's fatal finding does not reach the fixed
half.

## 4. Measured

The fixed-kind check was applied inside `Topology::build` and the entire
workspace run with `--no-fail-fast`:

| | |
| --- | --- |
| tests run | **645 passed** |
| violations across the whole committed population | **2** |
| which tests | `a_wrong_kind_referent_still_builds_and_that_is_the_held_half`, `probe_field_fixed_kinds` |

Both are tests written *specifically* to construct wrong-kind referents.
**No committed body violates the fixed-kind rule** — the cross-language
golden vector included, which passed unmoved.

The first run of this measurement was wrong and is recorded rather than
discarded: without `--no-fail-fast`, cargo stopped after the domain
binary and reported 88 tests, which would have supported the same
conclusion on a fraction of the evidence.

## 5. Attacks on the above

- **Is `Volume.producer` really closed?** It is the weakest of the four.
  The chain says "encryption/container → volume" and the doc names two
  kinds, but nothing in the spec forbids a future producer kind. LVM on
  LUKS resolves correctly (the LV's producer is the `Aggregate`, the LUKS
  volume being the aggregate's member), but this field deserves the most
  scrutiny of the four.
- **Zero committed violations is not zero real violations.** #333 records
  that no production capture path exists yet, so this measures the
  *modelled* population, not the world. It is the strongest evidence
  available today and it is not proof about future captures.
- **The three panel layouts are honest in the world but unmodelled.**
  MODEL-002's linear chain does not place a partition table under a
  volume or an aggregate, and the pair table omits both rows. They build
  today only because nothing validates naming referents — the very defect
  #354 names. Whether they *should* build is #360's question, not one
  this reading settles.
- **The fixed half does not unblock #333.** It protects the
  `partition → table` hop and not the `table → root` hop. Anyone reading
  this as a route to #333's enforcement is reading it wrong.

## 6. Recommendation

1. **Record ADR-0037:217 as not satisfied.** #333's enforcement stays
   held. The precondition is the *pairing* check, and the pairing check
   needs the pair table to be right first.
2. **#360's blocking scope narrows to one field.** For ADR-0037's own
   derivation path, the only open hop is `PartitionTable.parent`, whose
   lawful set is device | volume | aggregate — exactly the rows #360
   reports missing. That is a much smaller question than "fix the pair
   table", and it is the one that gates #333.
3. **A landable increment exists that needs no #360**: the fixed-kind
   half, over four fields, derived from MODEL-002's chain rather than
   from the pair table, measured at zero cost across the committed
   population. It advances #354 and closes the wrong-kind hole on the
   fields where the answer is not in dispute. **It must not be described
   as closing #354 either** — the open fields remain.
4. `PartitionTable.parent`'s doc comment ("the device the table
   describes") understates the model and would mislead whoever implements
   the open half. Worth correcting whenever #360 is decided.

Per house process this recommendation is a candidate, not a decision, and
should be adversarially reviewed before any of it is implemented.
