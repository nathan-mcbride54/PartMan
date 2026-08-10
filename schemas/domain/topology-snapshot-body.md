# The topology-snapshot body format

- Spec version: 11.1.0
- Requirement IDs: MODEL-002, MODEL-003, MODEL-004, MODEL-005, MODEL-006,
  PLAN-006, CONC-004
- Decided by: `docs/adr/0002-hashed-artifact-body-and-envelope.md` (body
  versus envelope), `docs/adr/0019-si27-node-naming.md` (nodes, edges),
  `docs/adr/0018-si11-protection-closure.md` (the evidence facts),
  `docs/adr/0014-si35-table-state-axis.md` (the table-state stamp);
  delivered by WP-010 increment 3 slices 3b, 3c, and 3f
- Underlying byte profile: `pce/1` (unchanged)
- Shared vectors: `schemas/domain/body-vectors.json`, `snapshots` section

This document records a delivered format. It decides nothing: a field
exists here because `crates/domain` encodes it, never because this
document says so.

## 1. Body and envelope

The **body** is the hashed artifact: the canonical `pce/1` encoding of the
map below, digested with SHA-256 (MODEL-005). The **envelope** — capture
timestamp and MODEL-004 provenance observations — travels beside the body
and never enters its bytes: editing the envelope must never move the body
hash, and a committed test holds that property. ADR-C4's confidence is
derived from provenance at read time, never stored.

## 2. The body map

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.topology-snapshot.captured` for a discovery capture; `partman.topology-snapshot.simulated` for a planner-simulated final topology (PLAN-002). The two never hash equal; a simulated snapshot is never a planning base and never accepted where PLAN-006 requires a capture — the schema string enforces that structurally. |
| `schema_version` | Unsigned | `1` (MODEL-003). |
| `transitional` | Bool | CONC-004's transitional marking, in the body deliberately. |
| `nodes` | Array as MODEL-006 set | One node entry per node (`schemas/domain/node-entry-format.md`), sorted by each element's complete canonical bytes. |
| `edges` | Array as MODEL-006 set | One edge map per edge (§3), same set rule. |

Unknown keys refuse at the typed boundary. Both sets reject duplicates
and mis-sorted input rather than repairing them.

## 3. Edges

Each edge is a map:

| Key | Type | Content |
| --- | --- | --- |
| `kind` | Text | `containment`, `backing`, `production`, `host-backing`, or `platform-membership` — the five MODEL-002 edge kinds with ADR-0018's semantics classes. |
| `source` | Bytes(32) | The source node's derived address. |
| `target` | Bytes(32) | The target node's derived address. |

Construction is fail-closed: unknown referents, self-edges, duplicates,
and any endpoint pair outside the kind's committed pair table refuse as
typed values. The theorem premise — no backing, production, or
host-backing edge targets a physical device — is enforced by that table
and proved by exhaustive enumeration in the delivered tests.

## 4. The typed boundary

`TopologySnapshot::from_canonical_body` is the sole decode path
(MODEL-005's codec-remediation rule: nothing authorizes through the
generic value or raw-hash APIs). It validates every schema rule above,
rebuilds the topology through the same fail-closed constructor, and
requires the recomputed body to reproduce the input bytes exactly — the
decode-recompute equality. The returned snapshot carries an empty
envelope: envelope content never lives in body bytes.

## 5. Conformance

The shared vectors pin four snapshots — minimal captured, its
simulated/transitional twin (same content, different schema string and
flag, disjoint bytes and digests), the full capture exercising every
fact class, a six-edge chain, and a collision group, and the plan-base
capture the plan vectors bind. `crates/domain/tests/body_vectors.rs`
proves the constructors reproduce the recorded bytes and the typed
boundary round-trips them; `packages/canonical/src/body-vectors.test.ts`
proves the TypeScript codec reproduces the same bytes and digests.
