/**
 * Cross-language conformance tests for the `pce/1` profile.
 *
 * These read the same `schemas/canonical-encoding-vectors.json` that the Rust
 * suite reads. That shared file, not a copy in either language, is what makes
 * MODEL-005's parity requirement provable.
 */

import assert from 'node:assert/strict'
import { test } from 'node:test'

import {
  CanonicalError,
  MAX_DEPTH,
  type Value,
  array,
  bool,
  compareKeys,
  decode,
  encode,
  fromHex,
  hash,
  map,
  nullValue,
  text,
  toHex,
  uint,
} from './canonical.ts'
import { loadVectors } from './vectors.ts'

const vectors = loadVectors()

test('the shared fixture is not empty', () => {
  assert.ok(vectors.length >= 30, `expected the full fixture, got ${vectors.length}`)
})

test('every shared vector encodes to exactly the recorded bytes', () => {
  for (const vector of vectors) {
    assert.equal(toHex(encode(vector.value)), vector.canonical, vector.name)
  }
})

test('every shared vector hashes to exactly the recorded digest', async () => {
  for (const vector of vectors) {
    assert.equal(toHex(await hash(vector.value)), vector.sha256, vector.name)
  }
})

test('every shared vector round-trips through decode', () => {
  for (const vector of vectors) {
    const decoded = decode(fromHex(vector.canonical))
    assert.equal(toHex(encode(decoded)), vector.canonical, vector.name)
  }
})

test('map key ordering is length-first, not JavaScript default', () => {
  assert.equal(compareKeys('z', 'aa'), -1)
  // JavaScript's own comparison disagrees, which is the bug this prevents.
  assert.ok('z' > 'aa', 'JavaScript compares by code unit, ignoring length')
  assert.equal(compareKeys('ab', 'ac'), -1)
  assert.equal(compareKeys('a', 'a'), 0)
})

test('encoding is independent of Map insertion order', () => {
  const forward = map(
    new Map<string, Value>([
      ['z', uint(1n)],
      ['aa', uint(2n)],
      ['b', uint(3n)],
    ]),
  )
  const reversed = map(
    new Map<string, Value>([
      ['b', uint(3n)],
      ['aa', uint(2n)],
      ['z', uint(1n)],
    ]),
  )
  assert.equal(toHex(encode(forward)), toHex(encode(reversed)))
})

test('non-canonical and excluded input is rejected', () => {
  const cases: [string, string][] = [
    ['non-shortest 1-byte argument', '1801'],
    ['non-shortest 2-byte argument', '190017'],
    ['non-shortest 4-byte argument', '1a00000017'],
    ['non-shortest 8-byte argument', '1b0000000000000017'],
    ['non-shortest text length', '78016161'],
    ['half-precision float', 'f93c00'],
    ['single-precision float', 'fa3f800000'],
    ['double-precision float', 'fb0000000000000000'],
    ['undefined', 'f7'],
    ['tag 42', 'd82a01'],
    ['tag 0', 'c001'],
    ['indefinite-length array', '9f01ff'],
    ['indefinite-length text', '7f6161ff'],
    ['reserved additional information 28', '1c'],
    ['integer map key', 'a10101'],
    ['negative beyond i64', '3b8000000000000000'],
    ['trailing byte', '0000'],
    ['truncated head', '18'],
    ['duplicate map key', 'a2616101616102'],
    ['bytewise-ordered map keys', 'a262616102617a01'],
    ['ill-formed utf8', '6180'],
    ['unpaired surrogate', '63eda080'],
    ['length beyond input', '5affffffff'],
  ]
  for (const [name, input] of cases) {
    assert.throws(() => decode(fromHex(input)), CanonicalError, `${name} (${input})`)
  }
})

test('empty input is rejected', () => {
  assert.throws(() => decode(new Uint8Array()), CanonicalError)
})

test('nesting beyond the depth limit is rejected, not a stack overflow', () => {
  const deep = new Uint8Array(MAX_DEPTH + 3)
  deep.fill(0x81)
  deep[deep.length - 1] = 0x00
  assert.throws(() => decode(deep), CanonicalError)

  const shallow = new Uint8Array(MAX_DEPTH - 1)
  shallow.fill(0x81)
  shallow[shallow.length - 1] = 0x00
  assert.doesNotThrow(() => decode(shallow))
})

test('the encoder enforces the same depth limit as the decoder', () => {
  // Before this was enforced, encode() emitted bytes decode() rejected, so a
  // producer could publish a hash over an artifact nobody could revalidate.
  let deep: Value = uint(0n)
  for (let i = 0; i <= MAX_DEPTH; i++) deep = array([deep])
  assert.throws(() => encode(deep), CanonicalError)

  let permitted: Value = uint(0n)
  for (let i = 0; i < MAX_DEPTH - 1; i++) permitted = array([permitted])
  const bytes = encode(permitted)
  assert.doesNotThrow(() => decode(bytes))
})

test('anything the encoder emits, the decoder accepts', () => {
  for (const vector of vectors) {
    assert.doesNotThrow(() => decode(encode(vector.value)), vector.name)
  }
})

test('a number instead of a bigint is refused rather than silently encoded', () => {
  // The hazard ADR-C1 recorded. `cborg` turns Number(2**53) into the float
  // fa5a000000; this codec refuses to accept a non-bigint at all, and the
  // profile would reject the resulting float anyway.
  const bad = { kind: 'uint', value: 2 ** 53 } as unknown as Value
  assert.throws(() => encode(bad), CanonicalError)
  assert.throws(() => decode(fromHex('fa5a000000')), CanonicalError)
})

test('nested containers agree with the Rust encoder', () => {
  const value = map(
    new Map<string, Value>([['x', array([map(new Map([['y', nullValue]])), bool(true)])]]),
  )
  assert.equal(toHex(encode(value)), 'a1617882a16179f6f5')
})

test('text is not Unicode-normalized', () => {
  // Precomposed and decomposed forms are different values and must not collide,
  // per schemas/canonical-encoding.md section 4.
  const precomposed = text('é')
  const decomposed = text('é')
  assert.notEqual(toHex(encode(precomposed)), toHex(encode(decomposed)))
})

test('an unpaired surrogate is refused rather than repaired to U+FFFD', () => {
  // `TextEncoder` substitutes U+FFFD for an unpaired surrogate instead of
  // failing, which would make `encode` non-injective: text('\uD800') and the
  // replacement character would both produce 63efbfbd. Repairing rather than
  // refusing is the malleability ADR-C1 forbids.
  assert.throws(() => encode(text('\uD800')), CanonicalError)
  assert.throws(() => encode(text('\uDFFF')), CanonicalError)
  assert.throws(() => encode(text('a\uD800b')), CanonicalError)

  // The replacement character itself is an ordinary value and still encodes.
  assert.equal(toHex(encode(text('\uFFFD'))), '63efbfbd')
  // A well-formed pair is one scalar value and must not be caught by this.
  assert.equal(toHex(encode(text('\u{1F600}'))), '64f09f9880')
})

test('the encoder never emits a map whose declared size counts byte-equal keys', () => {
  // The regression this guards. `compareKeys` reports these two distinct
  // JavaScript strings equal, because both encode to efbfbd, while `Map` still
  // holds two entries. Before the well-formedness check the encoder emitted
  // a263efbfbd0163efbfbd02 -- a map declaring two entries with identical keys,
  // which section 3 makes invalid and this package's own decoder rejects.
  // Section 6.1 requires that anything the encoder emits, the decoder accepts.
  assert.equal(compareKeys('\uD800', '\uFFFD'), 0)

  const both = new Map<string, Value>([
    ['\uD800', uint(1n)],
    ['\uFFFD', uint(2n)],
  ])
  assert.equal(both.size, 2)
  assert.throws(() => encode(map(both)), CanonicalError)

  // Insertion order must not change the answer, since keys are checked before
  // they are sorted.
  const reversed = new Map<string, Value>([
    ['\uFFFD', uint(2n)],
    ['\uD800', uint(1n)],
  ])
  assert.throws(() => encode(map(reversed)), CanonicalError)
})

test('Rust cannot represent what TypeScript had to be taught to refuse', () => {
  // Not a test of Rust, but a statement of why the fix is one-sided: Rust's
  // `String` is validated UTF-8, so the ill-formed value is unconstructible
  // there and only the TypeScript half could produce it. The decoder side was
  // always symmetric -- both reject the surrogate encoded on the wire.
  assert.throws(() => decode(fromHex('63eda080')), CanonicalError)
})

test('an unknown value kind is refused rather than encoding to nothing', () => {
  // Rust reaches this by exhaustive `match` at compile time. TypeScript cannot,
  // because a `Value` can be forged at runtime. Without a `default` arm the
  // switch fell through, `encode` returned zero bytes, and `hash` published
  // SHA-256 of the empty string as a well-formed digest over an artifact with no
  // encoding -- section 6.1's failure exactly.
  const forged = { kind: 'tag', value: 42n } as unknown as Value
  assert.throws(() => encode(forged), CanonicalError)
})

test('a value whose payload has the wrong runtime type is refused', () => {
  // `TextEncoder.encode` coerces a non-string and `Uint8Array.from` truncates a
  // non-byte-array, so both would encode something rather than fail.
  const badText = { kind: 'text', value: 42 } as unknown as Value
  const badBytes = { kind: 'bytes', value: [300, -1] } as unknown as Value
  assert.throws(() => encode(badText), CanonicalError)
  assert.throws(() => encode(badBytes), CanonicalError)
})

test('fromHex refuses input that is not hexadecimal', () => {
  // `Number.parseInt` yields NaN for a non-hex pair, which stores as 0, so two
  // distinct textual digests would decode to identical bytes.
  assert.throws(() => fromHex('zz'), CanonicalError)
  assert.throws(() => fromHex('00zz'), CanonicalError)
  assert.throws(() => fromHex('0 '), CanonicalError)
  // And still accepts real hex, in either case.
  assert.equal(toHex(fromHex('00FF0a')), '00ff0a')
})
