# The RPC handshake

- Spec version: 11.1.0
- Requirement IDs: RPC-002, MODEL-003, SEC-006
- Owner: WP-040 (`docs/work-packages/WP-040.md`)
- Underlying byte profile: `pce/1` (unchanged)

This document records a delivered format. It decides nothing.

## 1. The handshake map

Connections begin with each side sending the canonical `pce/1` encoding
of:

| Key | Type | Content |
| --- | --- | --- |
| `schema` | Text | `partman.rpc.handshake`. |
| `schema_version` | Unsigned | `2` (MODEL-003; version 2 constrained `build` from free text to the build-version grammar — a reviewed bump taken while no consumer existed, the envelope-v2 posture). |
| `protocol_version` | Unsigned | The protocol version this side speaks (`PROTOCOL_VERSION`, currently `1`). Bumped only by reviewed schema changes. |
| `build` | Text | The build version — used in the refusal's remediation message to name what to update, never in compatibility logic. Held to the build-version grammar (`schemas/rpc/redaction.md` §3) at encode and decode: the redaction boundary's structural arm for what was the protocol's one free-entry text position. |

Decoding is strict, exactly as the envelope's: unknown fields, wrong
schema, and mistyped fields refuse by name; a build outside its
grammar refuses without echoing the value.

## 2. Refuse, never degrade (RPC-002)

Compatibility is a total function over the two `protocol_version`
values: **equal is compatible, unequal refuses** — with a typed refusal
carrying both versions and a remediation message naming the older side
and the build to update to. There is no downgrade arm to reach: a build
that wants to interoperate updates, and nothing negotiates downward
silently.

The rule is deliberately exact equality rather than a range: a
compatibility window is a reviewed decision with its own schema
consequences, and until one is decided the honest rule is the one that
cannot admit an untested pairing.
