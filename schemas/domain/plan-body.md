# The operation-plan body format

- Spec version: 17.3.0
- Requirement IDs: MODEL-003, MODEL-005, MODEL-006, PLAN-004, PLAN-005,
  PLAN-006, PLAN-007, PLAN-008, SAFE-003, Section 6
- Decided by: `docs/adr/0012-non-goal-protection-is-unrepresentable.md`
  (the hand-forged-artifact refusal), `docs/adr/0016-si34-verdict-placement.md`
  (verdict through body-carried inputs),
  `docs/adr/0018-si11-protection-closure.md` (steps run the closure;
  the acknowledgment vocabulary), `docs/adr/0014-si35-table-state-axis.md`
  and `docs/adr/0017-si33-continuity-witness.md` (the identity record's
  table state and witness), `docs/adr/0022-si19-reversal-linkage.md`
  (the reversal linkage, step preconditions, and the draft's
  step-output spelling), `docs/adr/0024-si16-state-selected-protection.md`
  (the typed step class and the capture-impossible acknowledgment),
  and the WP-060 recorded cancellation-class decision, 2026-08-12
  (PLAN-005's per-step declaration, its fail-closed floor, and the
  draft's pinned floor);
  delivered by WP-010 increment 3 slices 3h, 3i, and 3j (version 1,
  since retired), the jointly-sequenced ADR-0022 and ADR-0024 schema
  changes (slices 3l and 3m, version 3, since retired), the
  jointly-sequenced PLAN-005 schema change (slice 3n, version 4, since
  retired), the slice-3o version-1 retirement, and the jointly-sequenced
  consequence-text schema change (slice 3p, version 5, with WP-060
  increment 12)
- Underlying byte profile: `pce/1` (unchanged)
- Shared vectors: `schemas/domain/body-vectors.json`, `plans` section

This document records a delivered format. It decides nothing: a field
exists here because `crates/domain` encodes it, never because this
document says so. The Section 6 items whose vocabularies other packages
own (outcome text, privileges, environment requirements, backup actions,
capability versions) are absent here and land as WP-050/WP-060 deliver
them — their absence is the increment's recorded boundary, not an
omission of this document. The reversal item is absent no longer: the
linked form carries it (ADR-0022). Neither is the cancellation item
(PLAN-005's per-step declaration, slice 3n, under the WP-060 recorded
cancellation-class decision), nor the consequence text: version 5
carries Section 6's user-facing consequence sentences as a set (slice
3p, in ADR-0023's form — text, never typed hashed carriage of the
facts).

## 0. The one live version, and the retired ones

**Version 5** (the linked form, ADR-0022 + ADR-0024 + PLAN-005 under
the WP-060 recorded cancellation-class decision + Section 6's
consequence text in ADR-0023's form) is the sole version the boundary
emits or accepts: Section 6's required `reversal` linkage item, a
required `consequences` set of sentences, a required `preconditions`
array on every step, a required `class` on every step (ADR-0024's typed
repair family), and a required `cancellation` on every step (PLAN-005's
declaration, closed at the requirement's own three words). A body with
a Reversible step whose linkage is not a draft or reapply-forward
statement refuses (**no draft, no Reversible**).

**Every other version is retired and refuses at decode** (MODEL-003's
explicit-migration discipline — nothing is silently coerced), each
retirement recorded in the changelog rather than smoothed over.
**Version 1** — the pre-ADR-0022 unlinked form, with no `reversal`
item and no step `preconditions` — outlived the linked versions'
change windows because it had real emitters; slice 3o retired it once
its last emitters (this crate's own tests and the two v1 vectors) were
migrated, and `OperationPlan::assemble` went with it: a plan without a
linkage is now unconstructible, not merely refused. The identity-bound
vector's SAFE-003 coverage survived the retirement as
`plan-v5-bound-identity-wipe`, which since slice 3p also states two
consequence sentences so the set form is pinned cross-language.
**Versions 2, 3 and 4** — the linked form without the class field,
without the cancellation field, and without the consequence text —
each existed for exactly one change window (slices 3l, 3m and 3n),
gained no emitter outside it and no surviving artifact (each version's
vectors were regenerated as the next in the change that retired it;
version 4's emitter was the planner through the domain's own
`assemble_linked`, which emits version 5 from the same call).

**A prediction never binds.** `OperationPlan::from_canonical_body`
refuses a `partman.topology-snapshot.simulated` snapshot as its binding
base before reading a single field.

## 1. The body map

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.plan` (MODEL-003). |
| `schema_version` | Unsigned | `4` (§0; every other version refuses). |
| `plan_id` | Bytes | The plan's identifier bytes. |
| `created_at` | Unsigned | Creation timestamp, seconds since the epoch. |
| `snapshot_hash` | Bytes(32) | The source snapshot's body hash **as bound at validation** (PLAN-006, 8.0.0's rule). A plan presented against any other snapshot refuses — the ACC-007 stale-plan shape at the type layer. In a reversal **draft**, these bytes are the simulated proposal's hash, and the draft's own boundary (§5) is the only one that accepts them. |
| `not_after` | Unsigned | PLAN-007's validity window, body content deliberately: enforced, never re-derived, so an unauthenticated expiry cannot be extended without invalidating the authorization bound to the plan (ADR-C2's row). |
| `identities` | Map | Bound device identities keyed by the target's derived address in lowercase hex (Section 6; §2). Empty in a draft — a draft binds identities at validation, and carrying them in a prediction would be a client-authored claim. |
| `steps` | Array | The step graph in dependency order — a semantic array, never a set (MODEL-006 distinguishes exactly this). One step map per step (§3). |
| `reversal` | Map, required | Section 6's reversal linkage (§4). |
| `consequences` | Array of Text, required, **set-valued** | Section 6's user-facing consequence text (slice 3p): the plan's consequence sentences, each non-empty, sorted by each sentence's complete canonical `pce/1` bytes — **length-first**, so a shorter sentence precedes a longer one whatever their letters — and unique (`canonical-collections.md`); the producer sorts and dedups, the consumer refuses an unsorted or repeated element as a set violation and an empty sentence outright. **Empty is lawful** and asserts nothing beyond it (ADR-0052 D6's bound). Text and only text: ADR-0023 rejected typed hashed carriage of the facts, which are in the bound snapshot already. In a reversal **draft** the set is **pinned empty** — a draft is a prediction whose consequences are authored when it binds — and a draft body claiming any refuses at decode. |

Unknown keys refuse at the typed boundary.

## 2. The bound identity record (SAFE-003)

Each identity is a map: `serial` Bytes?, `wwn` Bytes?, `os_instance_id`
Bytes?, `connection_path` Bytes?, `total_bytes` Unsigned,
`logical_sector_size` Unsigned?, `physical_sector_size` Unsigned?,
`table` (the three-valued table-state map of
`schemas/domain/node-entry-format.md` §5), and `witness` Map? (ADR-0017's
continuity witness, where the apparatus is qualified). Optional fields
are omitted when absent.

Two derivations are deliberately **not** in the bytes:

- **Strength is derived, never stored.** SAFE-003's Strong/Weak verdict
  is computed from the record alone; a forged `strength` key refuses as
  an undeclared field.
- **The table state must agree with the snapshot's stamp.** The helper
  stamps ADR-C3's table state into the snapshot body at validation
  (ADR-0014); a plan identity whose `table` disagrees with the stamp for
  the same target refuses as the client-authored value that never
  validates (slice 3j's committed regression).

## 3. The step map

| Key | Type | Content |
| --- | --- | --- |
| `target` | Bytes(32) | The step's target address. In a **draft** step this key may be replaced by `target_step_output` (§5); a bound plan presenting that spelling refuses. |
| `written_table_extents` | Array | `HostRange` maps (§3a) the step writes as table content. |
| `consumed` | Array | `HostRange` maps the step consumes. |
| `destroyed` | Array | `HostRange` maps the step destroys. |
| `acknowledgments` | Array | Maps of `kind` Text (`release`, `opaque-destruction`, `identity-bound-restore`, `uncapturable-regions` — ADR-0018's vocabulary plus ADR-0024's entry, closed at four) and `node` Bytes(32); the `uncapturable-regions` kind additionally carries `regions`, an Array of `{start: Unsigned, length: Unsigned}` on the covered device, strictly ascending, non-overlapping, nonzero. The table-state kinds (`identity-bound-restore`, `uncapturable-regions`) are lawful only on a `table-repair`-class step over a device whose authored table state is `Indeterminate` — the constructor law, re-run at the boundary. |
| `severity` | Unsigned | PLAN-004's ordinal: 0 informational, 1 reversible, 2 disruptive, 3 data-moving, 4 destructive. Plan severity is the step maximum. Severity 1 requires the linkage rule (§0). |
| `flags` | Array | The set flags' names, in the fixed order: `security-sensitive`, `irreversible-after-start`, `requires-offline`, `requires-reboot`, `requires-rescue` (PLAN-004's orthogonal flags; unset flags are omitted). |
| `preconditions` | Array, required | Precondition maps (§3b), re-checked at every validation boundary against the binding snapshot — ADR-0022's two-time truthfulness. A failed precondition refuses the plan. |
| `class` | Text, required | `ordinary` or `table-repair` — ADR-0024's typed step class. The repair family is a class, never an intent flag; its protection arms and acknowledgment kinds attach to it. A draft step is `ordinary` (a repair-family draft is a future reviewed extension). |
| `cancellation` | Text, required | `cancellable`, `checkpoint-cancellable`, or `non-cancellable` — PLAN-005's declaration, spelled exactly as the requirement spells it, closed at three. The class is each building package's per-family stated declaration over the fail-closed `non-cancellable` floor (the WP-060 recorded decision); it is independent of `irreversible-after-start` in both directions (spec 12.3.0). A draft step is `non-cancellable` (a draft family off the floor is a future reviewed extension). |

### 3a. `HostRange`

`host` Bytes(32), `start` Unsigned, `length` Unsigned — a byte range on
the named host node.

**`length` is nonzero and `start + length` does not overflow `u64`**;
either refuses at decode (`ZeroLengthRange`, `RangeOverflows`). These are
ADR-0041's rules 4 and 5, which the fact boundary applies at
`TopologySnapshot::assemble` and the journal applies to a protection
record's regions — the same geometry, so the same two rules. The step
boundary did not carry them until this increment, and the closure cannot
supply them: its reach math saturates, so a wrapping range reads as
touching nothing and passes clean. An end of exactly `u64::MAX` is lawful
and is not refused here.

### 3b. Preconditions

Closed vocabulary, each a map with a `kind` Text:

- `region-unoccupied` — `host` Bytes(32), `start` Unsigned, `length`
  Unsigned: no authenticated fact places any node (other than the host
  itself) on these bytes. The shrink-back shape.
- `host-unoccupied` — `host` Bytes(32): the named node's entire address
  space hosts nothing. The delete-created-structure shape.
- `step-output-unoccupied` — `step` Unsigned: **draft bodies only** —
  the forward step's created output must host nothing, resolved to
  `host-unoccupied` at the draft's binding when the created node has an
  address.

**The affected set is not body content.** A step's affected set is
recomputed by the closure at every validation; recording it would be a
client-authored claim. This is how the hand-forged artifact refuses.

## 4. The reversal linkage (ADR-0022)

A map with a `kind` Text, exactly one of:

- `draft` — `plan_id` Bytes, `hash` Bytes(32): a truthful reversal
  draft was emitted; the hash freezes what was advertised at
  authorization time.
- `impossible` — `statements` Array of `{step: Unsigned, reason: Text}`:
  PLAN-008's per-step machine-readable statements, covering exactly the
  plan's step indices in ascending order. Reasons are a closed
  vocabulary: `data-destroyed`, `prior-value-not-carried`,
  `pre-state-preserved-for-recovery` (ADR-0024's repair arm: the raw
  capture is the reversal substrate, and putting it back is REC-001's
  recovery plan, never a planner emission).
- `reapply-forward` — `plan_id` Bytes: the draft's own linkage — its
  reversal is re-application of the forward plan, named by ID.

**The asymmetry is acyclic by construction**: the forward side names the
draft by hash, the draft side names the forward plan by ID only, and a
mutual-hash spelling has no encoding — a `reapply-forward` map carrying
a `hash` key refuses as an undeclared field.

## 5. The reversal draft (PLAN-008)

A draft is an ordinary version-4 plan body whose `snapshot_hash` is the
forward plan's **simulated final topology** hash (the proposal), whose
`identities` map is empty, whose `reversal` is the `reapply-forward`
statement, and whose steps may spell a created-node target as
`target_step_output` Unsigned — a typed reference to the creating
forward step's output, never an address, because the address does not
exist until the forward apply.

The draft has its own typed boundary (`ReversalDraft::from_canonical_body`:
decode, strict parse, recompute equality — no snapshot, no closure) and
a binding boundary (`ReversalDraft::bind`): binding takes the helper's
fresh **captured** snapshot and the forward plan, refuses a simulated
snapshot (a prediction proposes and never binds), resolves each
step-output reference to the one node the capture places at the creating
step's consumed range (zero or many refuses), re-checks every
precondition, re-runs every step through the sole constructor, and
assembles an ordinary bound plan whose `snapshot_hash` is the capture's
— 8.0.0's rule: binding is a validation act.

## 6. The typed boundary

`OperationPlan::from_canonical_body(bytes, &snapshot)` takes the plan
bytes **and the snapshot they claim to bind**. It refuses a simulated
snapshot outright, refuses a `snapshot_hash` that does not equal that
snapshot's recomputed body hash, then re-runs every step through the
sole constructor — the same closure over the snapshot's authenticated
facts, the same acknowledgment law (ADR-0012's second verification row),
and re-checks every precondition. A tampered range, a smuggled
acknowledgment, a step whose reach the closure refuses, a severity-1
claim with no reversal to stand on, a decayed precondition, or a
malformed consequence set (unsorted, repeated, empty-sentenced, or
absent) never parses into a plan. The sentences themselves are not
recomputed here — they are prose for UI-005 and REC-010; what the
helper recomputes under HLP-002 is the topology and the closure. The derived protection verdict is committed through
the body-carried inputs the closure reads (topology, facts), not stored
beside them — ADR-0016's substance in the anti-assertion shape ADR-C4
set.

## 7. Conformance

The shared vectors pin, over the same base capture: the version-4 wipe
with its impossibility statements, its identity-bound twin (the
SAFE-003 identity-record coverage the retired version-1 vectors
carried), the version-4 forward create carrying its
draft linkage, the create-reversal draft itself bound to the
simulated-created snapshot vector, and the version-4 table-repair plan
over the indeterminate-table snapshot, carrying the
capture-impossible acknowledgment and the pre-state-preserved
statement. Each records the digest of the
snapshot vector it binds (for the draft: proposes), and the parity
suites in both languages assert the embedded `snapshot_hash` bytes equal
that vector's recorded digest — the PLAN-006 binding held across the
fixture itself. `crates/domain/tests/body_vectors.rs` proves the
constructors reproduce the recorded bytes and the typed boundaries
revalidate them; `packages/canonical/src/body-vectors.test.ts` proves
the TypeScript codec reproduces the same bytes and digests.
