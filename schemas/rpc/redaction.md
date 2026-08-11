# The redaction boundary

- Spec version: 11.1.0
- Requirement IDs: SEC-006, RPC-002, RPC-003
- Owner: WP-040 (`docs/work-packages/WP-040.md`)
- Underlying byte profile: `pce/1` (unchanged)

This document records a delivered rule. It decides nothing: the
boundary table lives in `crates/rpc`'s `redaction` module, and the gate
test is the authority wherever a sentence could be read two ways.

## 1. The rule

SEC-006 names the identifier classes — device serials, paths, labels,
usernames, keys, and file names. The protocol edge holds its deny-floor
as **a schema-level classification of every field position this
package owns**: an allowlist of the positions that may carry
identifier-class bytes at all, held the WP-035 way — the allowlist
needs no knowledge of what the denied classes are, because every
position outside it is structurally incapable of carrying them, and
the strict validator (RPC-003) is the mechanism rather than a filter:

| Class | Why an identifier cannot stand there |
| --- | --- |
| pinned constant | any value but the pinned one refuses `WrongSchema` |
| unsigned number | bytes cannot live in it at all |
| closed tag | anything outside the vocabulary refuses by field |
| build version | the grammar (§3) refuses every class that carries structure |
| **allowlisted** | identifier-class bytes may cross; the governing authority is named (§2) |

The full table (format × field × class) is `redaction::FIELD_RULES`;
the gate test pins its per-format field sets to the wire's actual key
sets as literals, so widening the allowlist — or adding a field
without classifying it — is a visible reviewed edit. There is also no
position to invent: an unknown field refuses by name, so a raw
identifier planted anywhere outside the allowlist — including as a
field's own key — refuses before it can cross.

## 2. The allowlist, and who governs each entry

Exactly two positions:

| Position | Governing authority |
| --- | --- |
| envelope `body` | The `schemas/`-defined operation type the bytes encode. SEC-006's field classes apply at the schema defining each field — the body is where typed operations cross, and its redaction obligations live with those schemas, not with this envelope for which the bytes are opaque. |
| resume token `execution` | WP-070. The helper mints the handle and owes its opacity; nothing at this layer can verify what helper-chosen bytes were derived from, and this document says so rather than pretending a check exists. |

An allowlist entry does not mean identifiers are welcome there; it
means the position is one of the two places bytes of any class may
flow, so the redaction obligation is named and assigned rather than
silently assumed held.

## 3. The build-version grammar (handshake v2)

The handshake's `build` was the protocol's one free-entry text
position. RPC-002 calls the field a build *version*, and v2 holds it
to that word, in both directions (encode and decode — this side cannot
emit what the peer would refuse):

- `digits '.' digits '.' digits`, each run nonempty;
- optionally one `+` or `-` followed by a nonempty suffix over
  `[A-Za-z0-9._+-]`;
- ASCII throughout, nonempty, at most 64 bytes
  (`BUILD_VERSION_MAX_BYTES`).

Paths, file names, spaced labels, and armored keys cannot fit: each
carries a separator, a space, or a shape the leading
`digits.digits.digits` refuses. The refusal (`NotABuildVersion`) names
the rule and **never echoes the presented value** — a refusal that
quoted the bytes would itself carry what the boundary exists to keep
out.

The constraint is a reviewed schema bump taken while no consumer
exists — the envelope v2 posture exactly — so `partman.rpc.handshake`
moves to schema version 2 and a v1-stamped handshake refuses rather
than being read under rules it was not written to.

## 4. What a grammar cannot do, stated

A bare token deliberately shaped like a version — a serial renumbered
`1.2.3`, an identifier hidden in a `+` suffix — cannot be
distinguished from a version by any grammar. The boundary's stated
reach is therefore **raw** identifier-class values: every exemplar the
gate test plants refuses, and deliberate smuggling inside the admitted
alphabet is the peer violating its schema obligation, named here as
such. The value of the rule is that a leak now requires deliberate
shaping — an accident cannot fit — and the obligation's home is
written down rather than diffused.

Two refusal-vocabulary facts, recorded so nothing reads as more than
it is: the unknown-field refusal carries the key it refuses, because
RPC-003's strictness refuses *by name* and the name is the violation;
and `VersionRefusal`'s remediation renders a build — the one place
this crate composes human-facing prose from wire data — which under
the v2 rule is always a validated version token by the time it can
reach that message. Surfaces that render any refusal remain bound by
their own SEC-006 obligations (the WP-035 CLI discipline).
