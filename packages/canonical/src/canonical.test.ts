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
  bytes,
  compareKeys,
  decode,
  encode,
  fromHex,
  hash,
  hashEncoded,
  map,
  neg,
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

test('every variant refuses a forged runtime payload', () => {
  // This test used to cover `text` and `bytes` only, and a progress review
  // found what the gap cost: the `bool` arm used JavaScript truthiness, so
  // `{ kind: 'bool', value: 'false' }` encoded as `f5` — canonical **true** —
  // and `hash` authenticated the opposite logical value on the MODEL-005
  // authorization boundary. TypeScript types do not protect an object that
  // arrived as JSON, over RPC, from a plugin, or as `unknown`, which is the
  // whole reason this module validates at runtime.
  //
  // One forged payload per variant, so a new variant added without validation
  // has to be added here too.
  // Each case names the phrase its refusal must contain. Asserting only the
  // error *class* is not enough, and an adversarial pass proved it here: the
  // original `array` case used the string `'not an array'`, which has a
  // `.length`, so the encoder walked it as a sequence and refused one of its
  // characters. The array guard never ran — deleting it left all 24 tests
  // green. A refusal for the wrong reason is evidence of nothing.
  const forged: ReadonlyArray<readonly [string, unknown, string]> = [
    ['uint', { kind: 'uint', value: 42 }, 'uint must be a bigint'],
    ['neg', { kind: 'neg', value: -42 }, 'neg must be a bigint'],
    ['bytes', { kind: 'bytes', value: [300, -1] }, 'bytes must be a Uint8Array'],
    ['text', { kind: 'text', value: 42 }, 'text must be a string'],
    ['array', { kind: 'array', value: 42 }, 'array must be an Array'],
    ['map', { kind: 'map', value: { a: 1 } }, 'map must be a Map'],
    ['map key', { kind: 'map', value: new Map([[1, { kind: 'null' }]]) }, 'map keys must be strings'],
    ['map value', { kind: 'map', value: new Map([['a', undefined]]) }, 'has no value'],
    ['bool', { kind: 'bool', value: 'false' }, 'bool must be a boolean'],
    ['bool zero', { kind: 'bool', value: 0 }, 'bool must be a boolean'],
    ['shape', 'not a value at all', 'must be an object'],
    ['shape null', null, 'must be an object'],
    ['shape kind', { value: 1n }, 'must carry a string `kind`'],
    ['unknown kind', { kind: 'tag', value: 1n }, 'not in the profile'],
  ]

  for (const [what, payload, reason] of forged) {
    assert.throws(
      () => encode(payload as Value),
      (error: unknown) => {
        assert.ok(
          error instanceof CanonicalError,
          `a forged ${what} payload must be refused with a CanonicalError, not a native throw: ${String(error)}`,
        )
        assert.ok(
          error.message.includes(reason),
          `a forged ${what} payload was refused for the wrong reason. Expected a message containing ${JSON.stringify(reason)}, got ${JSON.stringify(error.message)}`,
        )
        return true
      },
    )
  }
})

test('every kind the encoder accepts has a forged case above', () => {
  // Makes the table exhaustive by construction rather than by inspection. A new
  // variant added to `Value` without validation, and without a row above, fails
  // here — an adversarial pass showed the previous hand-written list did not
  // force that, so the exact bug just fixed would pass 24/24 under a new name.
  const accepted = [
    uint(0n),
    neg(-1n),
    bytes(new Uint8Array()),
    text(''),
    array([]),
    map(new Map()),
    bool(true),
    nullValue,
  ]
  const kinds = new Set(accepted.map((value) => value.kind))
  assert.equal(kinds.size, 8, 'every variant of Value must be represented here')

  // `null` carries no payload to forge, so it is the one exemption and is named
  // rather than silently absent.
  const covered = new Set(['uint', 'neg', 'bytes', 'text', 'array', 'map', 'bool'])
  for (const kind of kinds) {
    assert.ok(
      covered.has(kind) || kind === 'null',
      `variant ${kind} has no forged payload case; add one before it can carry a hash`,
    )
  }
})

test('a forged boolean is refused rather than encoded as the other value', () => {
  // Stated on its own because it is the one that silently changed *meaning*
  // rather than throwing something. Both directions: a truthy non-boolean must
  // not become `true`, and a falsy one must not become `false`.
  for (const truthy of ['false', 'true', 1, [], {}]) {
    assert.throws(() => encode({ kind: 'bool', value: truthy } as unknown as Value), CanonicalError)
  }
  for (const falsy of [0, '', null, undefined, Number.NaN]) {
    assert.throws(() => encode({ kind: 'bool', value: falsy } as unknown as Value), CanonicalError)
  }
  // And the real booleans still encode to the two bytes the profile fixes.
  assert.equal(toHex(encode(bool(true))), 'f5')
  assert.equal(toHex(encode(bool(false))), 'f4')
})

test('a payload that misreports its own contents cannot be encoded', () => {
  // Field checks alone cannot close this: each of these passes every guard and
  // then tells the writer something different from what it told the guard. What
  // catches them is `encode` decoding its own output, which turns section 6.1
  // from a claim about the code into a computation.
  //
  // All four were found by an adversarial pass, and each produced bytes whose
  // declared head did not match the body that followed — bytes `hash` would
  // have signed.
  const misreporting: ReadonlyArray<readonly [string, unknown]> = [
    [
      // A real array's `length` is non-configurable, so the lie has to come
      // through a Proxy — which `Array.isArray` still reports as an array.
      'an array whose reported length disagrees with its contents',
      {
        kind: 'array',
        value: new Proxy([nullValue, nullValue], {
          get: (target, property, receiver) =>
            property === 'length' ? 5 : Reflect.get(target, property, receiver),
        }),
      },
    ],
    [
      'a bytes payload whose iterator yields values outside a byte',
      {
        kind: 'bytes',
        // `Uint8Array.prototype` defines `length` as a getter, so this has to
        // be an own property rather than an assignment.
        value: Object.defineProperties(Object.create(Uint8Array.prototype), {
          length: { value: 2 },
          [Symbol.iterator]: {
            value: function* () {
              yield 300
              yield -1
            },
          },
        }),
      },
    ],
    [
      'a map whose size disagrees with its entries',
      {
        kind: 'map',
        value: Object.defineProperties(Object.create(Map.prototype), {
          size: { value: 9 },
          entries: { value: () => [['a', nullValue]][Symbol.iterator]() },
        }),
      },
    ],
  ]

  for (const [what, payload] of misreporting) {
    assert.throws(
      () => encode(payload as Value),
      CanonicalError,
      `${what} must be refused, not encoded into bytes whose head lies about its body`,
    )
  }
})

test('a value is read once, so it cannot be validated as one kind and written as another', () => {
  // An adversarial pass built a value whose `kind` getter answered 'bool' to
  // the shape check and 'null' to the dispatch, so the arm that validated was
  // not the arm that ran. Both arms happened to be safe, which is exactly why
  // this is worth pinning: the guarantee held by luck rather than by design,
  // and no per-arm check could have noticed.
  //
  // The fix is that `kind` and the payload are each read exactly once. The
  // encoder therefore commits to the first reading and produces its canonical
  // encoding — a defined answer rather than a mismatch between head and body.
  let kindReads = 0
  let payloadReads = 0
  const shifty = {
    get kind() {
      return kindReads++ === 0 ? 'bool' : 'null'
    },
    get value() {
      payloadReads++
      return true
    },
  }

  // Encoded once: this object answers differently on every read, so a second
  // call would be measuring the test's own bookkeeping rather than the encoder.
  const encoded = encode(shifty as unknown as Value)
  assert.equal(toHex(encoded), 'f5')
  assert.equal(kindReads, 1, 'the kind must be read exactly once')
  assert.equal(payloadReads, 1, 'the payload must be read exactly once')

  // And the result is a real encoding of a real value, not a lie about one.
  assert.deepEqual(decode(encoded), bool(true))
})

test('hashEncoded validates the same bytes it hashes', () => {
  // `decode` walks the array through its `length` property while
  // `crypto.subtle.digest` reads the underlying buffer. A view whose `length`
  // is shadowed therefore had its prefix validated and its whole buffer
  // hashed — a digest over bytes nothing proved canonical, which is the one
  // thing this function exists to prevent.
  const real = encode(nullValue) // f6, one byte
  const wider = new Uint8Array(4)
  wider.set(real)
  wider[1] = 0xf6 // trailing bytes: decode must reject the whole thing
  const lying = Object.defineProperty(wider, 'length', { get: () => 1 })

  return assert.rejects(
    async () => hashEncoded(lying as Uint8Array<ArrayBuffer>),
    CanonicalError,
    'a view that under-reports its length must not get its prefix validated and its buffer hashed',
  )
})

test('everything the encoder accepts decodes back to the same value', () => {
  // The property the review asked for, over the shared vectors. Refusing
  // forged payloads is only half of section 6.1; the other half is that
  // whatever survives encoding round-trips, so a producer never publishes a
  // hash over bytes that mean something else when read back.
  for (const vector of loadVectors()) {
    const encoded = encode(vector.value)
    assert.deepEqual(
      decode(encoded),
      vector.value,
      `${vector.name} did not round-trip through encode/decode`,
    )
  }
})

test('hashEncoded refuses bytes that are not canonical', async () => {
  // The raw-byte hash constructor was exported and hashed whatever it was
  // given, with a comment asking callers to pass canonical bytes. That is an
  // instruction, not a guarantee, and SEC-001 authorizes by exact hash.
  await Promise.all([
    // A non-shortest integer argument: decodes to 0, but is not the canonical
    // encoding of 0, so hashing it would authorize a second digest for one value.
    assert.rejects(async () => hashEncoded(Uint8Array.from([0x18, 0x00])), CanonicalError),
    // Trailing bytes after a complete item.
    assert.rejects(async () => hashEncoded(Uint8Array.from([0xf6, 0xf6])), CanonicalError),
    // A float, which the profile excludes entirely.
    assert.rejects(
      async () => hashEncoded(Uint8Array.from([0xfa, 0x00, 0x00, 0x00, 0x00])),
      CanonicalError,
    ),
  ])
})

test('hashEncoded agrees with hashing the value it came from', async () => {
  // Narrowing the API must not have changed any digest.
  await Promise.all(
    loadVectors().map(async (vector) => {
      const encoded = encode(vector.value)
      assert.equal(
        toHex(await hashEncoded(encoded)),
        toHex(await hash(vector.value)),
        vector.name,
      )
    }),
  )
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
