# The operation-plan body format

- Spec version: 12.9.1
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
  delivered by WP-010 increment 3 slices 3h, 3i, and 3j (version 1),
  the jointly-sequenced ADR-0022 and ADR-0024 schema changes (slices
  3l and 3m, version 3), and the jointly-sequenced PLAN-005 schema
  change (slice 3n, version 4)
- Underlying byte profile: `pce/1` (unchanged)
- Shared vectors: `schemas/domain/body-vectors.json`, `plans` section

This document records a delivered format. It decides nothing: a field
exists here because `crates/domain` encodes it, never because this
document says so. The Section 6 items whose vocabularies other packages
own (outcome text, privileges, environment requirements, backup actions,
capability versions) are absent here and land as WP-050/WP-060 deliver
them — their absence is the increment's recorded boundary, not an
omission of this document. The reversal item is absent no longer: the
linked form carries it (ADR-0022). Neither is the cancellation item:
version 4 carries PLAN-005's per-step declaration (slice 3n, under the
WP-060 recorded cancellation-class decision).

## 0. The two versions

**Version 1** is the pre-ADR-0022 form: no `reversal` item, no step
`preconditions`. It is still emitted by `OperationPlan::assemble` and
accepted at the boundary while WP-060's planner migrates to the linked
form; its retirement is its own reviewed change (MODEL-003's
explicit-migration discipline — nothing is silently coerced, and any
other version refuses at decode).

**Version 4** (the linked form, ADR-0022 + ADR-0024 + PLAN-005 under
the WP-060 recorded cancellation-class decision) adds to version 1
exactly four things: Section 6's required `reversal` linkage item, a
required `preconditions` array on every step, a required `class` on
every step (ADR-0024's typed repair family), and a required
`cancellation` on every step (PLAN-005's declaration, closed at the
requirement's own three words). A linked body with a Reversible step
whose linkage is not a draft or reapply-forward statement refuses
(**no draft, no Reversible**), and the rule reaches back: a version-1
body cannot carry a Reversible step at all.

**Versions 2 and 3** — the linked form without the class field, and
the linked form without the cancellation field — each existed for
exactly one change window (slices 3l and 3m), gained no emitter
outside it and no surviving artifact (version 3's vectors were
regenerated as version 4 in the change that retired it), and are
refused at decode; each retirement is recorded in the changelog rather
than smoothed over.

**A prediction never binds.** `OperationPlan::from_canonical_body`
refuses a `partman.topology-snapshot.simulated` snapshot as its binding
base before reading a single field, for either version.

## 1. The body map

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.plan` (MODEL-003). |
| `schema_version` | Unsigned | `1` or `4` (§0). |
| `plan_id` | Bytes | The plan's identifier bytes. |
| `created_at` | Unsigned | Creation timestamp, seconds since the epoch. |
| `snapshot_hash` | Bytes(32) | The source snapshot's body hash **as bound at validation** (PLAN-006, 8.0.0's rule). A plan presented against any other snapshot refuses — the ACC-007 stale-plan shape at the type layer. In a reversal **draft**, these bytes are the simulated proposal's hash, and the draft's own boundary (§5) is the only one that accepts them. |
| `not_after` | Unsigned | PLAN-007's validity window, body content deliberately: enforced, never re-derived, so an unauthenticated expiry cannot be extended without invalidating the authorization bound to the plan (ADR-C2's row). |
| `identities` | Map | Bound device identities keyed by the target's derived address in lowercase hex (Section 6; §2). Empty in a draft — a draft binds identities at validation, and carrying them in a prediction would be a client-authored claim. |
| `steps` | Array | The step graph in dependency order — a semantic array, never a set (MODEL-006 distinguishes exactly this). One step map per step (§3). |
| `reversal` | Map, version 4 only, required | Section 6's reversal linkage (§4). |

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
| `preconditions` | Array, version 4 only, required | Precondition maps (§3b), re-checked at every validation boundary against the binding snapshot — ADR-0022's two-time truthfulness. A failed precondition refuses the plan. |
| `class` | Text, version 4 only, required | `ordinary` or `table-repair` — ADR-0024's typed step class. The repair family is a class, never an intent flag; its protection arms and acknowledgment kinds attach to it. A draft step is `ordinary` (a repair-family draft is a future reviewed extension). |
| `cancellation` | Text, version 4 only, required | `cancellable`, `checkpoint-cancellable`, or `non-cancellable` — PLAN-005's declaration, spelled exactly as the requirement spells it, closed at three. The class is each building package's per-family stated declaration over the fail-closed `non-cancellable` floor (the WP-060 recorded decision); it is independent of `irreversible-after-start` in both directions (spec 12.3.0). A draft step is `non-cancellable` (a draft family off the floor is a future reviewed extension). |

### 3a. `HostRange`

`host` Bytes(32), `start` Unsigned, `length` Unsigned — a byte range on
the named host node.

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
claim with no reversal to stand on, or a decayed precondition never
parses into a plan. The derived protection verdict is committed through
the body-carried inputs the closure reads (topology, facts), not stored
beside them — ADR-0016's substance in the anti-assertion shape ADR-C4
set.

## 7. Conformance

The shared vectors pin, over the same base capture: the bare destructive
wipe and its identity-bound twin (version 1), the version-4 wipe with
its impossibility statements, the version-4 forward create carrying its
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
