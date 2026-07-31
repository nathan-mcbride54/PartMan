# Canonical collection semantics for PartMan schemas

- Spec version: 4.1.0
- Requirement IDs: MODEL-005, MODEL-006, SAFE-005
- Decided by: `docs/adr/0008-canonical-set-order-and-depth.md` (ADR-C6)
- Underlying byte profile: `pce/1` (unchanged)

This document is normative. It defines how a PartMan schema distinguishes a
semantic sequence from a set when both use the `pce/1` Array kind.

## 1. Scope belongs to schemas, not `pce/1`

`pce/1` has no Set kind and an Array can contain any sequence of values. This
rule therefore applies only to a field its schema explicitly declares
**set-valued**.

- A semantic array MUST retain the order supplied by its schema. A codec or
  generic decoder MUST NOT sort it.
- A set-valued field MUST be encoded as a `pce/1` Array and MUST satisfy the
  producer and consumer rules below.
- A generic `pce/1` decoder MUST continue to accept descending arrays. The
  schema validation pass is the sole boundary that may reject one for violating
  a set declaration.

This does not change `schemas/canonical-encoding.md` and does not create a new
`pce` profile.

## 2. Sort key and comparison

For each element, the sort key is the element's **complete canonical `pce/1`
encoding**, including its head and all nested content.

Keys are compared lexicographically as sequences of **unsigned bytes**:

1. Compare corresponding bytes from left to right as values `0..=255`.
2. At the first difference, the lower byte sorts first.
3. If one key is a prefix of the other, the shorter key sorts first.
4. Equal keys denote a duplicate and are invalid.

The comparison is plain bytewise, not length-first. This is deliberately
different from `pce/1` map-key ordering. The shared extent vector makes the two
answers disagree so a length-first implementation fails the conformance suite.

## 3. Producer rule

A producer MUST:

1. canonically encode every element at its actual depth under §5;
2. sort the resulting complete byte strings by §2;
3. reject equal adjacent keys rather than silently deduplicating them; and
4. emit an Array head followed by those exact sorted element bytes.

Computing each element once is significant. Sorting one read of mutable input
and emitting a later read can produce bytes in an order their own final
encodings contradict.

## 4. Consumer rule

After strict `pce/1` decoding, a schema validator MUST reconstruct each set
element's canonical bytes at its actual depth and require every key to be
strictly greater than its predecessor.

It MUST reject:

- equal adjacent keys as duplicate logical elements;
- a key that precedes its predecessor as a misordered set;
- an element that cannot be canonically encoded at its inherited depth; and
- a set array whose own depth exceeds the codec's declared limit.

The validator MUST NOT sort, deduplicate, or otherwise repair input. Set errors
belong to the schema layer and MUST be distinguishable from `pce/1` codec
errors, because the same descending Array can be valid under a sequence field
and invalid under a set field.

## 5. Depth inheritance

The root value is at depth zero, matching both canonical codec implementations.
If a set-valued Array occurs at depth `d`, each element's sort-key encoding MUST
start at depth `d + 1`; nested containers continue incrementing from there.

An implementation MUST NOT call its public standalone encoder for sort keys if
that encoder resets depth to zero. A 100-level element that is valid by itself
can be invalid inside a set already 28 levels deep. Accepting the standalone
key and splicing it into that artifact would emit bytes the conforming decoder
must reject.

The shared depth vectors pin both sides of the boundary for the implementations'
declared `MAX_DEPTH = 128`:

- set depth 27 plus one element edge plus 100 nested arrays reaches 128 and is
  accepted;
- set depth 28 reaches 129 and is rejected.

## 6. Shared conformance fixture

`schemas/domain/canonical-set-vectors.json` is the single fixture consumed by
the Rust and TypeScript implementations. It contains:

- producer inputs intentionally stored out of order, exact canonical Array
  bytes, and SHA-256 digests;
- consumer arrays that are ascending, descending, or duplicate; and
- compact inherited-depth boundary cases.

Neither language may keep its own copy. A golden vector already stored in
sorted order would prove only that both implementations can preserve fixture
order, so every producer vector is intentionally unsorted and each suite asserts
that fact before testing the sort.
