# ADR-C1: Canonical encoding and hashing for plans and snapshots

- Status: Accepted
- Date: 2026-07-27
- Spec version: 2.0.0 (decided against 2.0.0; superseded in part by ADR-C2 under 3.0.0)
- Work packages unblocked: WP-010 (and transitively WP-040, WP-050, WP-060,
  WP-070, WP-S100, WP-I100, WP-R100)
- Requirement IDs: MODEL-005, MODEL-003, MODEL-001, SEC-001, SEC-002, HLP-001,
  HLP-003, SAFE-003, SAFE-009
- Decision owners: repository CODEOWNERS

Acceptance basis: the decision owner was presented with Options A, B, and C and
the trade-offs recorded below, and delegated the choice rather than selecting an
alternative. Option B remains fully documented and is the intended fallback if
the Revisit conditions are met; nothing in WP-010 should treat Option C as
having been the only candidate.

## Context

MODEL-005 requires that every hashed artifact have exactly one canonical byte
encoding, that the plan hash be SHA-256 over those bytes, and that Rust and
TypeScript produce identical hashes for identical logical content, proven by
cross-language golden tests. This ADR fixes that encoding. Section 15 makes its
acceptance a precondition for starting WP-010.

The plan hash is not a checksum. It is the authorization boundary:

- HLP-001 accepts `apply-plan` **by plan hash**.
- HLP-003 binds fresh interactive authorization to **the exact plan hash**.
- SEC-001 authorizes **only exact plan hashes**.
- SEC-002 rejects replayed, expired, altered, cross-user, and cross-device
  plans.

Encoding ambiguity is therefore a privilege-escalation surface, not an
interoperability inconvenience. Two failure classes must be impossible:

1. **Divergence** — one logical plan yields different bytes in two components,
   so a plan the user authorized in the GUI cannot be applied, or worse, a
   revalidation compares unequal hashes for equal content and the mismatch is
   "fixed" by relaxing the comparison.
2. **Malleability** — two distinct logical plans yield the same bytes, or one
   logical plan has more than one valid encoding, so the bytes a user
   authorized are not the bytes that describe what executes.

Relevant model constraints: MODEL-001 makes every offset and size an unsigned
byte count, so `u64` values are ordinary, not exotic. The domain model
(Section 5) contains no field that is inherently a floating-point number.

Consumers are the Rust core and per-OS helpers, and the TypeScript UI. SAFE-008
requires helpers to ship schemas compiled in and forbids reading schemas from
user-writable locations, so whatever encodes canonical bytes is linked into the
helper rather than loaded at runtime.

## Safety analysis

- **Privilege boundary.** The encoder sits directly on the HLP-003
  authorization path. A defect here is not a data-format bug; it lets an
  approval for plan A authorize the execution of plan B.
- **Hostile input.** Section 11.4 names plan, journal, and RPC deserializers as
  mandatory fuzz targets. Canonical *encoding* alone is insufficient: the
  decoder MUST reject non-canonical input. If a decoder silently accepts a
  non-canonical encoding, an attacker can submit bytes that decode to an
  approved plan yet hash differently, or that re-encode to different bytes.
  Strictness is required in both directions.
- **Memory safety.** SAFE-009 forbids `unsafe` in the domain and rpc crates and
  requires fuzz targets for parsers of externally supplied bytes. Any dependency
  on this path must be `unsafe`-free or the decision must confine it to a
  reviewed adapter module.
- **Device identity.** SAFE-003 binds plans to immutable identity records whose
  fields (serial, WWN, sector sizes, table checksum) are covered by the plan
  hash. Weak identity handling depends on those fields hashing faithfully.
- **Secrets.** SAFE-006 is not delegated to the encoding. Redaction happens
  before a value reaches the encoder; no encoding choice makes a secret safe to
  include.
- **Disposable test coverage.** Nothing in this decision touches storage.
  Conformance is provable entirely at Tier 1 from vectors in the repository.

## Options considered

### Option A — JCS canonical JSON (RFC 8785)

Canonicalize JSON per RFC 8785, hash the UTF-8 bytes.

Benefits: human-readable, diffable, and aligned with the JSON that MODEL-003
and the CLI already expose. Mature implementations exist in both languages.

Costs and failure modes: RFC 8785 constrains numbers to values expressible as
IEEE 754 double precision, so integer fidelity ends at 2^53. The RFC's own
guidance is to carry larger integers as JSON strings. Since MODEL-001 makes
`u64` byte counts pervasive, conformance would depend on a standing convention
that no hashed field is ever a JSON number — enforced by review, forever, across
two languages. The first field typed as a number instead of a string is a silent
hash divergence that appears only for values above 2^53, which no realistic
fixture would exercise. JSON also has no byte-string type, so checksums and
identifiers need a further encoding convention with its own canonicalization
rules.

Rejected: the dominant failure mode is silent, latent, and permanent.

### Option B — Deterministic CBOR through third-party libraries

Adopt `serde_ipld_dagcbor` (Rust) and `cborg` (TypeScript), both of which
default to DAG-CBOR-style determinism — shortest-form integers, definite
lengths, text-string map keys, and keys sorted length-first then bytewise, which
is RFC 8949's "length-first core deterministic encoding requirements".

Benefits: no encoder to write. `serde_ipld_dagcbor` is widely deployed
(~1.1M downloads); `cborg` has zero runtime dependencies and is actively
maintained. Both offer strict decode modes that reject non-shortest integers and
duplicate map keys. Native `u64`, so Option A's precision problem disappears.

Costs and failure modes:

- `cborg` encodes a JavaScript `Number` outside ±2^53 as a **float**, silently.
  A `u64` field that reaches the encoder as `Number` rather than `BigInt`
  produces different bytes from Rust with no error raised on either side. The
  library cannot enforce the rule; only our own typing and tests can.
- `@ipld/dag-cbor` would drag in `multiformats` solely for CID tag 42, which
  this profile must forbid. Using `cborg` directly avoids that, but then the two
  sides reach determinism through independent code paths with no shared
  conformance suite between them.
- Hash stability becomes a property of two independently versioned third-party
  libraries. `cborg` already exposes `rfc8949EncodeOptions`, which switches map
  ordering to plain bytewise; any future change of default silently changes every
  hash the product has ever issued.
- The dependency is linked into every privileged helper (SAFE-008).

### Option C — A restricted canonical CBOR profile with a first-party codec

Define the profile normatively in `schemas/` and implement a small encoder and
strict validating decoder in each language, with no runtime dependency:

- Value model: unsigned integers, negative integers, text strings, byte strings,
  booleans, null, arrays, and maps with text-string keys. **No floating-point
  values, no tags, no indefinite-length items, no duplicate map keys, no
  undefined.**
- Integers use the shortest CBOR representation. Strings and arrays use definite
  lengths.
- Map keys sort length-first, then bytewise — RFC 8949's length-first core
  deterministic ordering, matching both libraries in Option B.
- The decoder rejects any input that is not the unique canonical encoding of the
  value it decodes to.

Benefits: the restricted value model makes whole ambiguity classes
*unrepresentable* rather than merely discouraged — excluding floats removes
`-0.0`, NaN, and infinity handling entirely; excluding tags removes tag
confusion. Hash stability is owned by this repository and pinned by golden
vectors rather than inherited from upstream defaults. The codec is small, has no
dependencies to audit or ship into a privileged helper, and is trivially
fuzzable. Option B's libraries remain useful as independent differential oracles.

Costs: two implementations to write and keep in step, and the general and
correct instinct not to hand-roll serialization. Both are tempered by the fact
that MODEL-005 already mandates cross-language byte-parity proof, so the
verification burden exists under every option; Option C makes the thing being
verified a written specification rather than the coincident behavior of two
upstream projects.

## Decision

**Option C.** Hashed artifacts are encoded with a restricted canonical CBOR
profile, specified byte-for-byte in `schemas/`, implemented first-party in Rust
and TypeScript, and differentially tested against `serde_ipld_dagcbor` and
`cborg` as independent oracles.

The plan hash is SHA-256 over those canonical bytes, exactly as MODEL-005
states. Domain separation is achieved **inside** the encoded value rather than by
prefixing the hash input: every hashed artifact carries its schema version and
artifact kind as encoded fields, so a topology snapshot cannot collide with a
plan of structurally identical shape. This keeps the literal MODEL-005 rule —
SHA-256 over the canonical bytes — intact while still separating domains, and
avoids introducing a hashing construction the spec does not describe.

Only the canonical profile is used for hashing. Human-facing and CLI output
remain JSON under MODEL-003; JSON is never hashed, and the canonical bytes are
never presented as a user-facing format.

## Consequences

Positive:

- One written definition of canonical bytes that both languages implement
  against, versioned with the schemas it describes.
- No third-party encoding dependency inside privileged helpers.
- Ambiguity classes eliminated by construction rather than by convention.
- The decoder's rejection of non-canonical input is a testable, fuzzable
  property rather than an upstream configuration flag.

Negative and to be managed:

- Two codecs to maintain. Divergence is caught only by the golden vectors, so
  those vectors are load-bearing and must be extended whenever the value model
  changes.
- Schema evolution needs care: adding a field changes the hash of otherwise
  identical logical content, so migration vectors are required per MODEL-003 and
  Section 11.1.
- TypeScript must represent every 64-bit integer as `BigInt`. This is a typing
  and codegen obligation, and it is exactly the trap Option B would also have
  set; it must be enforced by the schema-to-type generation and asserted by
  tests, not left to reviewer discipline.

## Verification

Required before WP-010 is accepted:

- Golden vectors in the repository, hashed identically by Rust and TypeScript,
  covering at minimum: `0`; `23`/`24` and each integer width boundary;
  `2^53 - 1`, `2^53`, `2^64 - 1`; empty string, array, and map; keys that differ
  only in length; keys that are prefixes of one another; non-ASCII and
  astral-plane text; text containing an embedded NUL; and empty byte strings.
- Negative decode vectors that MUST be rejected: non-shortest integers,
  indefinite-length items, duplicate map keys, out-of-order map keys, any float,
  any tag, and trailing bytes after a complete item.
- Differential tests asserting the first-party encoder agrees byte-for-byte with
  `serde_ipld_dagcbor` and `cborg` on the profile's supported value space, so
  divergence from the wider ecosystem is detected deliberately rather than
  discovered later.
- A `cargo-fuzz` target per Section 11.4 for the decoder, with round-trip and
  canonical-rejection invariants.
- Confirmation that the Rust codec contains no `unsafe`, per SAFE-009.

  **Corrected after audit.** As first written this line required that "any
  dependency it retains" also contain no `unsafe`. That overreached: SAFE-009
  constrains "the domain, planner, validator, journal, and rpc crates" and
  parsers of on-disk metadata, not the dependency graph. The stricter reading is
  also unachievable — every mainstream Rust SHA-256 implementation carries
  `unsafe` for CPU feature detection and SIMD.

  The audit, on `sha2` 0.10.9: `generic-array` 78 occurrences, `sha2` 29,
  `cpufeatures` 9, `block-buffer` 4; `digest`, `crypto-common`, and `typenum`
  each declare `forbid(unsafe_code)`; `cfg-if` has none. `sha2`'s `force-soft`
  feature does not remove them, because `cpufeatures` is a non-optional
  target-gated dependency on x86 and aarch64.

  SAFE-009 is satisfied as written: `partman-domain` denies `unsafe` through
  `[workspace.lints]`, and the decoder — the only parser of externally supplied
  bytes here — contains none. The residual consideration is recorded rather than
  hidden: the hash on the authorization path depends on audited third-party
  `unsafe`. Removing it would mean a first-party SHA-256, which trades reviewed
  upstream code for unreviewed local code and is not obviously safer.
- Test tier: T1, unprivileged. No fixture media and no block-device access.

## Revisit conditions

- `draft-ietf-cbor-cde` (CBOR Common Deterministic Encoding) reaches RFC status.
  It is a draft today, which is why this ADR pins a profile rather than citing
  it; a published CDE with matching semantics would be a reason to re-express
  the profile in its terms.
- A hashed artifact acquires a genuine floating-point field, which the profile
  currently makes unrepresentable.
- A third language needs to compute hashes, changing the cost balance between
  maintaining first-party codecs and adopting a shared library.
- A differential test shows the profile diverging from both oracles, suggesting
  the ordering or integer rules were misread.
