# ADR-C6: Canonical set order and inherited encoding depth

- Status: Accepted
- Date: 2026-07-30
- Spec version: 4.1.0
- Work packages blocked: WP-010 increment 3 (one blocker resolved; others remain)
- Requirement IDs: MODEL-005, MODEL-006, SAFE-005
- Resolves: SI-31
- Decision owners: repository CODEOWNERS

Acceptance basis: SI-31's recommended answer survived adversarial review before
this ADR was written. The decision owner then accepted the project-lead
recommendations and authorized implementation on 2026-07-30. This record lands
the previously settled answer, its corrected scope, and the encoder prerequisite
that the review found.

## Context

Section 5 contains set-valued fields, but the only byte profile, `pce/1`, has no
Set kind. It represents both sequences and prospective sets as Arrays. Map keys
have a length-first ordering rule; array elements have none.

Two plausible set comparators disagree on ordinary domain values. SI-31's
extent example is:

```text
{len: 5, off: 300}  -> a2636c656e05636f666619012c   (13 bytes)
{len: 6, off:   0}  -> a2636c656e06636f666600       (11 bytes)
```

Length-first puts the second encoding first. Plain bytewise puts the first
encoding first. Choosing implicitly would let Rust and TypeScript hash the same
logical set differently.

The review also found two scope errors in the first proposal:

- sorting every Array would narrow `pce/1`, which currently accepts semantic
  sequences in any order and would require a new profile; and
- encoding a sort key through either language's public encoder reset nesting
  depth to zero, so a deep element could encode alone and then make the
  containing artifact impossible to decode after its bytes were spliced in.

## Decision

### Sets are a schema concept

Only a field explicitly declared set-valued by its schema is subject to this
rule. It remains encoded with the existing `pce/1` Array kind. Semantic arrays
retain their order, and generic `pce/1` decoding remains unchanged.

The schema layer owns a distinct validation error because an ordering that is
invalid for a set is still valid for an ordinary Array.

### Compare full canonical bytes, unsigned and lexicographically

The sort key is each element's complete canonical `pce/1` encoding. Keys are
ordered by ordinary unsigned lexicographic byte comparison. Equal keys are
duplicates and MUST be rejected rather than deduplicated.

Plain bytewise ordering is chosen because it is the least surprising portable
primitive for opaque byte strings and Rust's slice ordering implements it
directly. This creates a second convention beside map keys; hiding that fact
would be worse than recording it.

### Inherit the enclosing depth budget

If the set Array occurs at depth `d`, sort-key encoding begins at `d + 1`.
Nested containers continue from that value. The caller must provide the actual
set depth; neither implementation offers a default that could silently reset it.

The producer emits the exact bytes it sorted, rather than re-reading elements
after the sort. The consumer reconstructs keys at the same inherited depth and
requires strict ascent without repair.

The normative algorithm and boundary vectors live in
`schemas/domain/canonical-collections.md` and
`schemas/domain/canonical-set-vectors.json`.

## Options considered

### Length first, then bytewise

Rejected. It resembles `pce/1` map-key ordering but is easier to implement
incorrectly for arbitrary encoded values, and the language most likely to use
its byte-slice default would silently choose the other answer.

### Sort every `pce/1` Array

Rejected. Arrays carry semantic order, and both existing decoders accept
descending arrays. Changing that would alter the profile rather than define a
schema rule, break round-trip properties, and change already committed hashes.

### Add a Set kind to `pce/1`

Rejected for this profile. It would require a new discriminant or tag and
therefore a new profile version. The domain schemas do not yet exist, so the
schema boundary can settle their hash-visible set convention without changing
any previously issued set artifact.

### Encode each key as a standalone value

Rejected. It resets depth to zero and violates the encoder/decoder symmetry
required by `schemas/canonical-encoding.md` §6.1 once a deep element is placed
inside a deep artifact.

### Silently remove duplicate keys

Rejected. Deduplication loses information and makes malformed input appear to
describe a different logical set. SAFE-005 requires a refusal.

## Consequences

Positive:

- Rust and TypeScript have one hash-visible set ordering and one shared fixture.
- The fixture input is deliberately unsorted and includes a case on which
  length-first and bytewise disagree, so the test exercises the decision.
- Deep sort keys cannot bypass the canonical decoder's nesting limit.
- Ordinary arrays and all existing `pce/1` vectors and hashes remain unchanged.

Negative and accepted:

- Schema encoders must carry the current depth to a low-level collection
  boundary.
- Two collection conventions exist: length-first text map keys in `pce/1`, and
  plain bytewise full encodings for schema sets.
- A generic `pce/1` decoder cannot identify set fields; every artifact schema
  must invoke validation at the right paths.

## Verification

- Rust and TypeScript read the same producer, validator, and depth vectors.
- Producer vectors assert their input is not already sorted before comparing
  exact bytes and hashes.
- A dedicated test proves the extent vector's shorter element is not the one
  bytewise ordering emits first.
- Producers and validators in both implementations accept the combined depth
  of 128 and reject 129 while proving the element still encodes standalone.
- Stable tests and both existing fuzz targets cover producer/validator fixed
  points, duplicate refusal, and the distinction between arrays and sets.

## Scope left open

This resolves SI-31 only. The authoritative table in
`docs/spec-issues/README.md` continues to gate WP-010 increment 3 on its
remaining entries. No Section 5 domain type is introduced by this decision.
