# The operation-plan body format

- Spec version: 11.1.0
- Requirement IDs: MODEL-003, MODEL-005, MODEL-006, PLAN-004, PLAN-006,
  PLAN-007, SAFE-003, Section 6
- Decided by: `docs/adr/0012-non-goal-protection-is-unrepresentable.md`
  (the hand-forged-artifact refusal), `docs/adr/0016-si34-verdict-placement.md`
  (verdict through body-carried inputs),
  `docs/adr/0018-si11-protection-closure.md` (steps run the closure;
  the acknowledgment vocabulary), `docs/adr/0014-si35-table-state-axis.md`
  and `docs/adr/0017-si33-continuity-witness.md` (the identity record's
  table state and witness); delivered by WP-010 increment 3 slices 3h,
  3i, and 3j
- Underlying byte profile: `pce/1` (unchanged)
- Shared vectors: `schemas/domain/body-vectors.json`, `plans` section

This document records a delivered format. It decides nothing: a field
exists here because `crates/domain` encodes it, never because this
document says so. The Section 6 items whose vocabularies other packages
own (outcome text, privileges, environment requirements, backup actions,
cancellation, capability versions, reversal) are absent here and land as
WP-050/WP-060 deliver them — their absence is the increment's recorded
boundary, not an omission of this document.

## 1. The body map

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.plan` (MODEL-003). |
| `schema_version` | Unsigned | `1`. |
| `plan_id` | Bytes | The plan's identifier bytes. |
| `created_at` | Unsigned | Creation timestamp, seconds since the epoch. |
| `snapshot_hash` | Bytes(32) | The source snapshot's body hash **as bound at validation** (PLAN-006, 8.0.0's rule). A plan presented against any other snapshot refuses — the ACC-007 stale-plan shape at the type layer. |
| `not_after` | Unsigned | PLAN-007's validity window, body content deliberately: enforced, never re-derived, so an unauthenticated expiry cannot be extended without invalidating the authorization bound to the plan (ADR-C2's row). |
| `identities` | Map | Bound device identities keyed by the target's derived address in lowercase hex (Section 6; §2). |
| `steps` | Array | The step graph in dependency order — a semantic array, never a set (MODEL-006 distinguishes exactly this). One step map per step (§3). |

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
| `target` | Bytes(32) | The step's target address. |
| `written_table_extents` | Array | `HostRange` maps (§3a) the step writes as table content. |
| `consumed` | Array | `HostRange` maps the step consumes. |
| `destroyed` | Array | `HostRange` maps the step destroys. |
| `acknowledgments` | Array | Maps of `kind` Text (`release`, `opaque-destruction`, `identity-bound-restore` — ADR-0018's vocabulary, closed at three) and `node` Bytes(32). |
| `severity` | Unsigned | PLAN-004's ordinal: 0 informational, 1 reversible, 2 disruptive, 3 data-moving, 4 destructive. Plan severity is the step maximum. |
| `flags` | Array | The set flags' names, in the fixed order: `security-sensitive`, `irreversible-after-start`, `requires-offline`, `requires-reboot`, `requires-rescue` (PLAN-004's orthogonal flags; unset flags are omitted). |

### 3a. `HostRange`

`host` Bytes(32), `start` Unsigned, `length` Unsigned — a byte range on
the named host node.

**The affected set is not body content.** A step's affected set is
recomputed by the closure at every validation; recording it would be a
client-authored claim. This is how the hand-forged artifact refuses.

## 4. The typed boundary

`OperationPlan::from_canonical_body(bytes, &snapshot)` takes the plan
bytes **and the snapshot they claim to bind**. It refuses a
`snapshot_hash` that does not equal that snapshot's recomputed body hash,
then re-runs every step through the sole constructor — the same closure
over the snapshot's authenticated facts, the same acknowledgment law
(ADR-0012's second verification row). A tampered range, a smuggled
acknowledgment, or a step whose reach the closure refuses never parses
into a plan. The derived protection verdict is committed through the
body-carried inputs the closure reads (topology, facts), not stored
beside them — ADR-0016's substance in the anti-assertion shape ADR-C4
set.

## 5. Conformance

The shared vectors pin two plans over the same base capture: the bare
destructive wipe and its identity-bound twin. Each records the digest of
the snapshot vector it binds, and the parity suites in both languages
assert the embedded `snapshot_hash` bytes equal that vector's recorded
digest — the PLAN-006 binding held across the fixture itself.
`crates/domain/tests/body_vectors.rs` proves the constructors reproduce
the recorded bytes and the typed boundary revalidates them;
`packages/canonical/src/body-vectors.test.ts` proves the TypeScript codec
reproduces the same bytes and digests.
