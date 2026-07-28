# PartMan canonical encoding, version 1

- Spec version: 3.1.0
- Requirement IDs: MODEL-005, MODEL-001, MODEL-003
- Decided by: `docs/adr/0001-canonical-encoding-and-hashing.md` (ADR-C1)
- Profile identifier: `pce/1`

This document is normative. It defines the one byte encoding used to hash
plans, topology snapshots, and every other hashed artifact. Every
implementation, in any language, MUST produce byte-identical output for
identical logical content and MUST reject every input that is not the unique
canonical encoding of the value it denotes.

The encoding is a restricted subset of CBOR (RFC 8949). It is not a general
CBOR profile: conforming CBOR that falls outside this subset MUST be rejected
rather than accepted and re-encoded.

## 1. Value model

Exactly nine kinds of value exist:

| Kind | CBOR major type | Range |
| --- | --- | --- |
| Unsigned | 0 | `0 ..= 2^64 - 1` |
| Negative | 1 | `-2^63 ..= -1` |
| Bytes | 2 | any byte sequence |
| Text | 3 | any well-formed UTF-8 sequence |
| Array | 4 | any sequence of values |
| Map | 5 | text keys, unique, ordered per §3 |
| Bool | 7 (20, 21) | `false`, `true` |
| Null | 7 (22) | — |

Nothing else is representable. In particular the following MUST be rejected on
decode and are unreachable on encode:

- **Floating-point values** of any width (major 7, additional 25, 26, 27). The
  domain model has no floating-point field (MODEL-001 makes offsets and sizes
  unsigned byte counts). Excluding them removes `-0.0`, NaN, and infinity from
  the problem entirely.
- **Tags** (major 6), including CBOR tag 42. This profile has no
  content-addressed links.
- **`undefined`** (major 7, additional 23) and every simple value other than
  20, 21, and 22.
- **Indefinite-length** items (additional 31) of any type, and the break stop
  code.
- **Reserved** additional information values 28, 29, and 30.

## 2. Integer and length encoding

Every head is `major_type << 5 | additional_information`, followed by an
argument encoded in the shortest form that represents it:

| Argument value | Additional information | Argument bytes |
| --- | --- | --- |
| `0 ..= 23` | the value itself | 0 |
| `24 ..= 255` | 24 | 1 |
| `256 ..= 65535` | 25 | 2 |
| `65536 ..= 2^32 - 1` | 26 | 4 |
| `2^32 ..= 2^64 - 1` | 27 | 8 |

Argument bytes are big-endian. This rule governs integer values *and* the
lengths of byte strings, text strings, arrays, and maps.

A decoder MUST reject any head whose argument could have been encoded in a
shorter form. Encoding `1` as `0x1801` rather than `0x01` is not merely
discouraged; it is invalid input.

Negative integers encode the argument `-1 - value`, so `-1` is argument `0` and
`-2^63` is argument `2^63 - 1`. A major-type-1 argument above `2^63 - 1` is out
of range for this profile and MUST be rejected.

## 3. Map key ordering

Map keys MUST be text strings. Other key types MUST be rejected, including
integers, which ordinary CBOR permits.

Keys are ordered by **length first, then bytewise** over their UTF-8 bytes.
Equivalently, and identically in result, keys are ordered by a plain bytewise
comparison of their fully encoded form, because the encoded length occupies the
leading bytes of a text-string head. Implementations may use either formulation.

This is RFC 8949's *length-first core deterministic encoding requirements*
(§4.2.3), not the bytewise ordering of §4.2.1. The choice is deliberate: it
matches DAG-CBOR and the default behavior of the `serde_ipld_dagcbor` and
`cborg` implementations used as differential oracles by ADR-C1.

Duplicate keys MUST be rejected. A decoder MUST verify that each key is
strictly greater than its predecessor under this ordering, which detects
duplicates and misordering in a single comparison.

## 4. Text

Text strings are UTF-8. A decoder MUST reject ill-formed UTF-8, including
unpaired surrogates and overlong encodings.

**No Unicode normalization is applied.** Two strings that are canonically
equivalent under NFC but differ in code points are different values and hash
differently. Schemas MUST NOT rely on Unicode equivalence for identity; where a
field must compare equal across sources, it is the producing adapter's job to
normalize before the value reaches this encoder.

## 5. Hashing and domain separation

The hash of an artifact is `SHA-256` over its canonical bytes, exactly as
MODEL-005 requires. No prefix, salt, or length framing is added around those
bytes.

Domain separation is achieved inside the encoded value. Every hashed artifact
is a Map carrying at least:

- `schema` — Text, the artifact's schema identifier, for example
  `partman.plan`.
- `schema_version` — Unsigned, per MODEL-003.

Because those fields are part of the hashed content, a topology snapshot cannot
collide with a plan of otherwise identical shape, and a schema version bump
changes the hash of the same logical content by construction. Consumers MUST
treat a hash as meaningful only together with the schema it was computed under.

## 6. Decoder obligations

A conforming decoder MUST reject, rather than repair:

1. Any construct excluded by §1.
2. Any non-shortest argument encoding (§2).
3. A major-type-1 argument above `2^63 - 1`.
4. A map key that is not a text string.
5. Map keys that are equal to, or not strictly greater than, the previous key.
6. Ill-formed UTF-8 in a text string.
7. A declared length that exceeds the remaining input.
8. Nesting deeper than the implementation's declared depth limit.
9. Any trailing byte after the single top-level item.

Obligation 7 exists so that a hostile length header cannot cause a large
allocation before the data is known to be present. Obligation 8 exists so that
deeply nested input cannot exhaust the stack of a recursive decoder; the limit
is a documented constant, and exceeding it is an error rather than a crash.

## 6.1 Encoder obligations

An encoder MUST refuse any value it cannot produce canonical bytes for, and MUST
enforce **the same depth limit as the decoder** (obligation 8).

This symmetry is not tidiness. An encoder that accepts deeper nesting than the
decoder emits bytes every conforming decoder must reject, so a producer computes
and publishes a hash over an artifact that nobody can revalidate — and, being
recursive, overflows its stack rather than returning an error on a deep enough
value. Stating the limit only as a decoder rule is precisely how that gap
arises.

The property to test, in every implementation, is that **anything the encoder
emits, the decoder accepts**.

## 7. Versioning

This profile is identified as `pce/1`. A change to §1, §2, §3, or §5 is a new
profile version, never an edit to this one, because it would silently change
every hash previously issued. Adding a decoder obligation that rejects input
the current profile already declares invalid is a clarification and may be made
in place.
