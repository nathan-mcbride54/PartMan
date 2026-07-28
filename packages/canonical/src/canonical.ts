/**
 * The `pce/1` canonical encoding, specified in `schemas/canonical-encoding.md`
 * and decided by ADR-C1. This is the TypeScript half of the cross-language
 * parity MODEL-005 requires; `crates/domain` is the Rust half.
 *
 * Every integer is a `bigint`. That is not a stylistic choice. A `number`
 * above 2^53 is not merely imprecise here, it changes CBOR major type: the
 * `cborg` oracle encodes `Number(2**53)` as the single-precision float
 * `fa5a000000` rather than the integer `1b0020000000000000`. Admitting
 * `number` into the value model would reintroduce exactly the divergence this
 * profile exists to prevent, so it is rejected at the type level and again at
 * runtime.
 */

/** Identifier of this encoding profile. */
export const PROFILE = 'pce/1'

/**
 * Maximum nesting depth accepted by {@link decode}.
 *
 * The decoder is recursive, so hostile input must not be able to exhaust the
 * stack. Must match `MAX_DEPTH` in the Rust implementation.
 */
export const MAX_DEPTH = 128

/** A value representable in the `pce/1` profile. */
export type Value =
  | { readonly kind: 'uint'; readonly value: bigint }
  | { readonly kind: 'neg'; readonly value: bigint }
  | { readonly kind: 'bytes'; readonly value: Uint8Array }
  | { readonly kind: 'text'; readonly value: string }
  | { readonly kind: 'array'; readonly value: readonly Value[] }
  | { readonly kind: 'map'; readonly value: ReadonlyMap<string, Value> }
  | { readonly kind: 'bool'; readonly value: boolean }
  | { readonly kind: 'null' }

/** Raised for any value that cannot be encoded or input that must be rejected. */
export class CanonicalError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'CanonicalError'
  }
}

const U64_MAX = (1n << 64n) - 1n
const I64_MIN = -(1n << 63n)

/** Convenience constructors. */
export const uint = (value: bigint): Value => ({ kind: 'uint', value })
export const neg = (value: bigint): Value => ({ kind: 'neg', value })
export const bytes = (value: Uint8Array): Value => ({ kind: 'bytes', value })
export const text = (value: string): Value => ({ kind: 'text', value })
export const array = (value: readonly Value[]): Value => ({ kind: 'array', value })
export const map = (value: ReadonlyMap<string, Value>): Value => ({ kind: 'map', value })
export const bool = (value: boolean): Value => ({ kind: 'bool', value })
export const nullValue: Value = { kind: 'null' }

/**
 * Order two map keys as the profile requires: length first, then bytewise over
 * UTF-8 bytes.
 *
 * This is not JavaScript's default string comparison, which is by UTF-16 code
 * unit and ignores length. `'z'` precedes `'aa'` here because it is shorter.
 */
export function compareKeys(left: string, right: string): number {
  const a = utf8(left)
  const b = utf8(right)
  if (a.length !== b.length) return a.length < b.length ? -1 : 1
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return (a[i] as number) < (b[i] as number) ? -1 : 1
  }
  return 0
}

const encoder = new TextEncoder()
const decoder = new TextDecoder('utf-8', { fatal: true })

function utf8(value: string): Uint8Array {
  return encoder.encode(value)
}

/**
 * Encode a value into its one canonical byte string.
 *
 * Throws if a value nests deeper than {@link MAX_DEPTH}. That limit is
 * deliberately the same one {@link decode} enforces: an encoder without it
 * emits bytes every conforming decoder must reject, so a producer would publish
 * a hash over an artifact nobody can revalidate.
 */
export function encode(value: Value): Uint8Array<ArrayBuffer> {
  const out: number[] = []
  writeValue(out, value, 0)
  return Uint8Array.from(out)
}

function writeValue(out: number[], value: Value, depth: number): void {
  if (depth > MAX_DEPTH) throw new CanonicalError(`nesting deeper than ${MAX_DEPTH}`)
  switch (value.kind) {
    case 'uint': {
      requireRange(value.value, 0n, U64_MAX, 'uint')
      writeHead(out, 0, value.value)
      return
    }
    case 'neg': {
      requireRange(value.value, I64_MIN, -1n, 'neg')
      writeHead(out, 1, -1n - value.value)
      return
    }
    case 'bytes': {
      // `number[]` is narrowed by `Uint8Array.from` on the way out, which
      // truncates modulo 256 rather than failing, so a value that is not
      // actually a byte array must be refused before it reaches the buffer.
      if (!(value.value instanceof Uint8Array)) {
        throw new CanonicalError('bytes must be a Uint8Array')
      }
      writeHead(out, 2, BigInt(value.value.length))
      for (const byte of value.value) out.push(byte)
      return
    }
    case 'text': {
      // `TextEncoder.encode` coerces a non-string instead of failing, and
      // `requireWellFormed` iterates `.length`, which is `undefined` on a
      // number — so the loop body never runs and nothing catches it.
      const payload: unknown = value.value
      if (typeof payload !== 'string') {
        throw new CanonicalError(`text must be a string, not ${typeof payload}`)
      }
      requireWellFormed(value.value, 'text string')
      const raw = utf8(value.value)
      writeHead(out, 3, BigInt(raw.length))
      for (const byte of raw) out.push(byte)
      return
    }
    case 'array': {
      writeHead(out, 4, BigInt(value.value.length))
      for (const item of value.value) writeValue(out, item, depth + 1)
      return
    }
    case 'map': {
      writeHead(out, 5, BigInt(value.value.size))
      // A JavaScript Map iterates in insertion order, which is not encoding
      // order, so the keys are sorted here. Well-formedness is checked before
      // the sort, so the refusal does not depend on insertion order.
      const keys = [...value.value.keys()]
      for (const key of keys) requireWellFormed(key, 'map key')
      keys.sort(compareKeys)
      for (const key of keys) {
        const raw = utf8(key)
        writeHead(out, 3, BigInt(raw.length))
        for (const byte of raw) out.push(byte)
        writeValue(out, value.value.get(key) as Value, depth + 1)
      }
      return
    }
    case 'bool': {
      out.push(value.value ? 0xf5 : 0xf4)
      return
    }
    case 'null': {
      out.push(0xf6)
      return
    }
    default: {
      // Rust reaches this by exhaustive `match` at compile time; TypeScript
      // cannot, because a `Value` can be forged at runtime by any caller who
      // constructs the object literal. Without this arm the switch fell
      // through, `encode` returned zero bytes, and `hash` published SHA-256 of
      // the empty string as a well-formed digest over an artifact that has no
      // encoding — a producer authorising bytes nobody can revalidate, which is
      // exactly what section 6.1 forbids.
      const unknown: never = value
      throw new CanonicalError(
        `value kind ${JSON.stringify((unknown as { kind?: unknown }).kind)} is not in the profile`,
      )
    }
  }
}

function requireRange(value: bigint, low: bigint, high: bigint, kind: string): void {
  if (typeof value !== 'bigint') {
    throw new CanonicalError(`${kind} must be a bigint, not ${typeof value}`)
  }
  if (value < low || value > high) {
    throw new CanonicalError(`${kind} value ${value} is outside ${low}..=${high}`)
  }
}

/**
 * Reject a string that is not well-formed UTF-16.
 *
 * This is the string counterpart of the `number`-versus-`bigint` hazard, and it
 * is the more dangerous of the two because nothing in the type system hints at
 * it. A JavaScript string may hold an unpaired surrogate, and `TextEncoder`
 * *repairs* rather than refuses: it substitutes U+FFFD. Two distinct strings
 * therefore encode to identical bytes, which breaks two profile properties at
 * once.
 *
 * `encode` stops being injective, so `text('\uD800')` and `text('�')`
 * produce the same `63efbfbd`. Worse, {@link compareKeys} then reports the two
 * map keys equal while `Map` still holds them as two entries, so the encoder
 * emitted `a263efbfbd0163efbfbd02`: a map declaring two entries whose keys are
 * byte-identical. Section 3 makes that input invalid and this package's own
 * decoder rejects it — violating section 6.1's rule that anything the encoder
 * emits, the decoder accepts. A producer would publish a hash over an artifact
 * nobody can revalidate.
 *
 * Rust cannot construct the value at all, because `String` is validated UTF-8,
 * so refusing here is also what keeps the two implementations agreeing on what
 * is encodable (MODEL-005).
 *
 * Reachable without an attacker: NTFS permits unpaired surrogates in names and
 * volume labels, and INV-008 requires such structures be represented rather
 * than discarded. Repairing to U+FFFD would be the malleability ADR-C1 forbids,
 * so this refuses instead.
 */
function requireWellFormed(value: string, kind: string): void {
  for (let i = 0; i < value.length; i++) {
    const unit = value.charCodeAt(i)
    if (unit < 0xd800 || unit > 0xdfff) continue
    // A trailing surrogate here has no leading partner, and a leading surrogate
    // must be followed by a trailing one.
    if (unit >= 0xdc00) throw new CanonicalError(`${kind} has an unpaired surrogate at ${i}`)
    const next = i + 1 < value.length ? value.charCodeAt(i + 1) : -1
    if (next < 0xdc00 || next > 0xdfff) {
      throw new CanonicalError(`${kind} has an unpaired surrogate at ${i}`)
    }
    i++
  }
}

/** Write `major << 5 | additional`, then the shortest argument encoding. */
function writeHead(out: number[], major: number, argument: bigint): void {
  const majorBits = major << 5
  if (argument <= 0x17n) {
    out.push(majorBits | Number(argument))
  } else if (argument <= 0xffn) {
    out.push(majorBits | 0x18)
    pushBytes(out, argument, 1)
  } else if (argument <= 0xffffn) {
    out.push(majorBits | 0x19)
    pushBytes(out, argument, 2)
  } else if (argument <= 0xffffffffn) {
    out.push(majorBits | 0x1a)
    pushBytes(out, argument, 4)
  } else {
    out.push(majorBits | 0x1b)
    pushBytes(out, argument, 8)
  }
}

function pushBytes(out: number[], argument: bigint, width: number): void {
  for (let shift = (width - 1) * 8; shift >= 0; shift -= 8) {
    out.push(Number((argument >> BigInt(shift)) & 0xffn))
  }
}

/**
 * Decode canonical bytes, rejecting anything that is not the unique canonical
 * encoding of the value it denotes.
 */
export function decode(input: Uint8Array): Value {
  const reader = new Reader(input)
  const value = reader.readValue(0)
  const remaining = reader.remaining()
  if (remaining !== 0) {
    throw new CanonicalError(`${remaining} byte(s) after the top-level item`)
  }
  return value
}

class Reader {
  private position = 0
  private readonly input: Uint8Array

  // Written out rather than as a parameter property: parameter properties are
  // not erasable, and this package is run by Node's native type stripping.
  constructor(input: Uint8Array) {
    this.input = input
  }

  remaining(): number {
    return this.input.length - this.position
  }

  private take(count: number): Uint8Array {
    const end = this.position + count
    if (end > this.input.length) {
      throw new CanonicalError('input ended inside an item')
    }
    const slice = this.input.subarray(this.position, end)
    this.position = end
    return slice
  }

  private takeByte(): number {
    return this.take(1)[0] as number
  }

  private peekByte(): number {
    if (this.position >= this.input.length) {
      throw new CanonicalError('input ended inside an item')
    }
    return this.input[this.position] as number
  }

  /** Read a head for major types 0 to 6, enforcing the shortest-form rule. */
  private readHead(): { major: number; argument: bigint } {
    const initial = this.takeByte()
    const major = initial >> 5
    const additional = initial & 0x1f

    let argument: bigint
    if (additional <= 23) {
      argument = BigInt(additional)
    } else if (additional === 24) {
      argument = BigInt(this.takeByte())
      if (argument < 24n) throw new CanonicalError('argument is not encoded in the shortest form')
    } else if (additional === 25) {
      argument = this.readArgument(2)
      if (argument <= 0xffn) throw new CanonicalError('argument is not encoded in the shortest form')
    } else if (additional === 26) {
      argument = this.readArgument(4)
      if (argument <= 0xffffn) {
        throw new CanonicalError('argument is not encoded in the shortest form')
      }
    } else if (additional === 27) {
      argument = this.readArgument(8)
      if (argument <= 0xffffffffn) {
        throw new CanonicalError('argument is not encoded in the shortest form')
      }
    } else if (additional <= 30) {
      throw new CanonicalError(`reserved additional information ${additional}`)
    } else {
      throw new CanonicalError('indefinite-length items are excluded')
    }

    return { major, argument }
  }

  private readArgument(width: number): bigint {
    let value = 0n
    for (const byte of this.take(width)) value = (value << 8n) | BigInt(byte)
    return value
  }

  /**
   * Convert a declared length only after proving the bytes exist, so a hostile
   * length header cannot force a large allocation.
   */
  private checkedLength(declared: bigint): number {
    const remaining = BigInt(this.remaining())
    if (declared > remaining) {
      throw new CanonicalError(
        `declared length ${declared} exceeds ${remaining} remaining byte(s)`,
      )
    }
    return Number(declared)
  }

  readValue(depth: number): Value {
    if (depth > MAX_DEPTH) throw new CanonicalError(`nesting deeper than ${MAX_DEPTH}`)

    if (this.peekByte() >> 5 === 7) {
      const initial = this.takeByte()
      return this.readSimpleOrFloat(initial & 0x1f)
    }

    const { major, argument } = this.readHead()
    switch (major) {
      case 0:
        return uint(argument)
      case 1: {
        if (argument > (1n << 63n) - 1n) {
          throw new CanonicalError(`negative argument ${argument} exceeds i64 range`)
        }
        return neg(-1n - argument)
      }
      case 2:
        return bytes(Uint8Array.from(this.take(this.checkedLength(argument))))
      case 3:
        return text(this.readText(argument))
      case 4: {
        const length = this.checkedLength(argument)
        const items: Value[] = []
        for (let i = 0; i < length; i++) items.push(this.readValue(depth + 1))
        return array(items)
      }
      case 5:
        return this.readMap(argument, depth)
      default:
        throw new CanonicalError(`tag ${argument} is excluded`)
    }
  }

  private readSimpleOrFloat(additional: number): Value {
    switch (additional) {
      case 20:
        return bool(false)
      case 21:
        return bool(true)
      case 22:
        return nullValue
      case 25:
      case 26:
      case 27:
        // Consume the payload so the reported error is the float itself.
        this.take(additional === 25 ? 2 : additional === 26 ? 4 : 8)
        throw new CanonicalError('floating-point values are excluded from this profile')
      case 24:
        throw new CanonicalError(`simple value ${this.takeByte()} is excluded`)
      case 31:
        throw new CanonicalError('indefinite-length items are excluded')
      default:
        if (additional >= 28) throw new CanonicalError(`reserved additional information ${additional}`)
        throw new CanonicalError(`simple value ${additional} is excluded`)
    }
  }

  private readText(argument: bigint): string {
    const raw = this.take(this.checkedLength(argument))
    try {
      return decoder.decode(raw)
    } catch {
      throw new CanonicalError('text string is not well-formed UTF-8')
    }
  }

  private readMap(argument: bigint, depth: number): Value {
    const declared = this.checkedLength(argument)
    const entries = new Map<string, Value>()
    let previous: string | undefined

    for (let i = 0; i < declared; i++) {
      if (this.peekByte() >> 5 !== 3) {
        throw new CanonicalError('map keys must be text strings')
      }
      const { argument: keyLength } = this.readHead()
      const key = this.readText(keyLength)

      if (previous !== undefined && compareKeys(previous, key) >= 0) {
        throw new CanonicalError(`map key ${JSON.stringify(key)} duplicates or precedes its predecessor`)
      }

      entries.set(key, this.readValue(depth + 1))
      previous = key
    }

    return map(entries)
  }
}

/**
 * SHA-256 over the canonical bytes (MODEL-005).
 *
 * No prefix, salt, or length framing is added. Domain separation belongs inside
 * the value, as the `schema` and `schema_version` fields.
 */
export async function hash(value: Value): Promise<Uint8Array> {
  return hashCanonicalBytes(encode(value))
}

/**
 * Hash bytes already known to be canonical.
 *
 * The parameter is `Uint8Array<ArrayBuffer>`, not a bare `Uint8Array`. Since
 * TypeScript 5.7 the array is generic over its backing buffer, and Web Crypto's
 * `BufferSource` excludes `SharedArrayBuffer`-backed views — a shared buffer can
 * be mutated by another thread while the digest is being computed, so hashing
 * one has no well-defined result. Stating the requirement in the signature makes
 * that a caller's compile error rather than this function's silent assumption.
 */
export async function hashCanonicalBytes(input: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
  const digest = await crypto.subtle.digest('SHA-256', input)
  return new Uint8Array(digest)
}

/** Lowercase hexadecimal, matching the Rust `Hash::to_hex` output. */
export function toHex(input: Uint8Array): string {
  let out = ''
  for (const byte of input) out += byte.toString(16).padStart(2, '0')
  return out
}

/** Parse lowercase hexadecimal into bytes. */
export function fromHex(input: string): Uint8Array {
  if (input.length % 2 !== 0) throw new CanonicalError('hex needs an even length')
  // `Number.parseInt` returns NaN for a non-hex pair, which stores as 0. Two
  // different textual digests would then decode to the same bytes, and this
  // function sits beside `hash` in the module SEC-001 authorizes against.
  if (!/^[0-9a-fA-F]*$/.test(input)) {
    throw new CanonicalError('hex contains a character outside 0-9a-fA-F')
  }
  const out = new Uint8Array(input.length / 2)
  for (let i = 0; i < out.length; i++) {
    out[i] = Number.parseInt(input.slice(i * 2, i * 2 + 2), 16)
  }
  return out
}
