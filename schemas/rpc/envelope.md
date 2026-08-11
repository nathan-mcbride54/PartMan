# The RPC message envelope

- Spec version: 11.1.0
- Requirement IDs: RPC-003, RPC-004, RPC-005, MODEL-003
- Owner: WP-040 (`docs/work-packages/WP-040.md`)
- Underlying byte profile: `pce/1` (unchanged)

This document records a delivered format. It decides nothing: a field
exists here because `crates/rpc` encodes it, and the strict validator's
tests are the authority wherever a sentence could be read two ways.

## 1. The envelope map

Every protocol message is the canonical `pce/1` encoding of:

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.rpc.envelope`. |
| `schema_version` | Unsigned | `1` (MODEL-003). |
| `channel` | Text | `request`, `response`, or `event` — RPC-004's stream separation, typed. Event-stream sequence numbering and resume tokens arrive with the streams increment; the class vocabulary is closed now so this shape does not move under them. |
| `body` | Bytes | The canonical `pce/1` encoding of one `schemas/`-defined operation type (RPC-005). Re-proved canonical at wrap and at decode — an envelope cannot launder bytes the codec would refuse. |

## 2. The strict rules (RPC-003, both directions)

One validator serves both ends, so the helper-side strictness the
requirement demands is also the client's: unknown fields refuse by
name, mistyped or absent declared fields refuse by field, the schema
identity and version must match exactly, and the body must decode as
canonical `pce/1`.

## 3. The size bound (RPC-004)

No encoded message exceeds `MAX_MESSAGE_BYTES` (1 MiB). The bound binds
the wire: it is checked at the decode entry **before any parsing
touches the bytes**, at body wrap, and at encode. Oversized input
refuses with both numbers named.

## 4. What the vocabulary cannot say (RPC-005)

No envelope field carries a path to execute, a command string, or
dynamic code, and the type vocabulary contains nothing that could. The
protocol carries typed operations defined under `schemas/`, encoded as
canonical bytes — CLI-004 at the transport layer, held structurally.
