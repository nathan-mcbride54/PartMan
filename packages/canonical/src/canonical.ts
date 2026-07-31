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

/**
 * Raised by schema-level set encoding and validation.
 *
 * This is deliberately not a {@link CanonicalError}: `pce/1` accepts arrays in
 * any order, while only a schema can declare that one array is a set.
 */
export class CanonicalSetError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'CanonicalSetError'
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
  const aLength = byteLength(a)
  const bLength = byteLength(b)
  if (aLength !== bLength) return aLength < bLength ? -1 : 1
  for (let i = 0; i < aLength; i++) {
    if (a[i] !== b[i]) return (a[i] as number) < (b[i] as number) ? -1 : 1
  }
  return 0
}

// Capture the intrinsics this authorization boundary needs before it evaluates
// any caller-controlled getter. A getter can replace a shared prototype method
// during encoding; looking up `push`, `sort`, or `Array.isArray` afterwards
// would let the input alter the bytes that are hashed.
const applyIntrinsic = Reflect.apply
const definePropertyIntrinsic = Object.defineProperty
const arrayIsArrayIntrinsic = Array.isArray
const arrayPushIntrinsic = Array.prototype.push
const arraySortIntrinsic = Array.prototype.sort
const numberIsSafeIntegerIntrinsic = Number.isSafeInteger
const numberIntrinsic = Number
const bigintIntrinsic = BigInt
const Uint8ArrayIntrinsic = Uint8Array
const MapIntrinsic = Map
const mapForEachIntrinsic = Map.prototype.forEach
const mapSetIntrinsic = Map.prototype.set
const textEncoderEncodeIntrinsic = TextEncoder.prototype.encode
const textDecoderDecodeIntrinsic = TextDecoder.prototype.decode
const stringCharCodeAtIntrinsic = String.prototype.charCodeAt
const typedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype) as object
const typedArrayLengthIntrinsic = Object.getOwnPropertyDescriptor(typedArrayPrototype, 'length')
  ?.get as (this: Uint8Array) => number
const typedArrayTagIntrinsic = Object.getOwnPropertyDescriptor(
  typedArrayPrototype,
  Symbol.toStringTag,
)?.get as (this: Uint8Array) => string
const subtleIntrinsic = crypto.subtle
const subtleDigestIntrinsic = crypto.subtle.digest
const encoder = new TextEncoder()
const decoder = new TextDecoder('utf-8', { fatal: true })

function utf8(value: string): Uint8Array {
  return applyIntrinsic(textEncoderEncodeIntrinsic, encoder, [value]) as Uint8Array
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
  const encoded = encodeAtDepth(value, 0)

  // Section 6.1 says everything this encoder emits, the decoder accepts. That
  // was previously a claim about the code; here it is computed. An adversarial
  // pass showed why the difference matters: a payload can lie about itself in
  // ways no per-field check anticipates — a getter returning one thing to a
  // guard and another to the writer, an Array-like object whose reported shape
  // disagrees with its indexed entries, or a byte payload whose iterator
  // produces values outside 0..=255.
  // Each of those produces a declared head that does not match the bytes that
  // follow, and each was reachable while every field check passed.
  //
  // `decode` accepts only the unique canonical encoding, so it catches all of
  // them at once and any not yet imagined. The cost is one decode per encode,
  // which is the right trade on an authorization boundary: HLP-001 applies
  // plans by hash and SEC-001 authorizes exact hashes, so bytes nobody can
  // revalidate are worse than slow ones.
  //
  // **No test currently requires this check, and it is not counted as
  // evidence.** Each of the four attacks above is caught by a field guard
  // earlier in `writeValue`, and deleting this block leaves the whole suite
  // green — checked, not assumed. It is kept as defence in depth against the
  // variant nobody has thought of yet, which is precisely the case that cannot
  // be written as a test. If a future change makes it load-bearing, that will
  // show up as a failure here rather than as a silent bad digest.
  try {
    decode(encoded)
  } catch (cause) {
    throw new CanonicalError(
      `the encoder produced bytes its own decoder rejects (${
        cause instanceof Error ? cause.message : String(cause)
      }); the value misreported its own contents`,
    )
  }
  return encoded
}

/**
 * Encode a schema-declared set as a `pce/1` array.
 *
 * `setDepth` is the array's actual depth in the complete artifact, with the
 * root at zero. It is mandatory because computing every element's sort key at
 * depth zero would reset the enclosing depth budget. Elements are sorted by an
 * unsigned lexicographic comparison of their complete canonical bytes;
 * duplicates are rejected rather than silently removed.
 *
 * Ordinary semantic arrays must use {@link encode}; `pce/1` itself has no Set
 * kind and this function does not change their order.
 */
export function encodeSetArray(
  elements: readonly Value[],
  setDepth: number,
): Uint8Array<ArrayBuffer> {
  requireSetDepth(setDepth)
  const items = snapshotSetElements(elements)
  const keyed: { originalIndex: number; bytes: Uint8Array<ArrayBuffer> }[] = []
  for (let originalIndex = 0; originalIndex < items.length; originalIndex++) {
    defineArrayElement(keyed, originalIndex, {
      originalIndex,
      bytes: encodeSetElement(items[originalIndex] as Value, originalIndex, setDepth + 1),
    })
  }
  sortArray(keyed, (left, right) => compareBytes(left.bytes, right.bytes))

  for (let index = 1; index < keyed.length; index++) {
    const previous = keyed[index - 1] as (typeof keyed)[number]
    const current = keyed[index] as (typeof keyed)[number]
    if (compareBytes(previous.bytes, current.bytes) === 0) {
      const previousFirst = previous.originalIndex < current.originalIndex
      const first = previousFirst ? previous.originalIndex : current.originalIndex
      const second = previousFirst ? current.originalIndex : previous.originalIndex
      throw new CanonicalSetError(
        `set elements ${first} and ${second} have identical canonical bytes`,
      )
    }
  }

  const out: number[] = []
  writeHead(out, 4, bigintIntrinsic(keyed.length))
  for (let elementIndex = 0; elementIndex < keyed.length; elementIndex++) {
    const element = keyed[elementIndex] as (typeof keyed)[number]
    const length = byteLength(element.bytes)
    for (let index = 0; index < length; index++) {
      appendArray(out, element.bytes[index] as number)
    }
  }
  const encoded = bytesFromNumbers(out)
  try {
    decode(encoded)
  } catch (cause) {
    throw new CanonicalSetError(
      `the set producer emitted bytes pce/1 rejects (${
        cause instanceof Error ? cause.message : String(cause)
      })`,
    )
  }
  return encoded
}

/**
 * Validate the observed order of a decoded schema-declared set array.
 *
 * This never sorts or repairs. The schema decoder supplies the array's actual
 * depth after the ordinary `pce/1` decoder has produced its elements.
 */
export function validateSetArray(elements: readonly Value[], setDepth: number): void {
  requireSetDepth(setDepth)
  const items = snapshotSetElements(elements)
  const keyed: Uint8Array<ArrayBuffer>[] = []
  for (let index = 0; index < items.length; index++) {
    defineArrayElement(
      keyed,
      index,
      encodeSetElement(items[index] as Value, index, setDepth + 1),
    )
  }

  for (let index = 1; index < keyed.length; index++) {
    const order = compareBytes(
      keyed[index - 1] as Uint8Array<ArrayBuffer>,
      keyed[index] as Uint8Array<ArrayBuffer>,
    )
    if (order === 0) {
      throw new CanonicalSetError(
        `set elements ${index - 1} and ${index} have identical canonical bytes`,
      )
    }
    if (order > 0) {
      throw new CanonicalSetError(
        `set element ${index} does not follow element ${index - 1} in canonical byte order`,
      )
    }
  }
}

function encodeAtDepth(value: Value, depth: number): Uint8Array<ArrayBuffer> {
  const out: number[] = []
  writeValue(out, value, depth)
  return bytesFromNumbers(out)
}

function requireSetDepth(setDepth: number): void {
  if (!numberIsSafeIntegerIntrinsic(setDepth) || setDepth < 0 || setDepth > MAX_DEPTH) {
    throw new CanonicalSetError(
      `set array depth must be an integer in 0..=${MAX_DEPTH}, got ${String(setDepth)}`,
    )
  }
}

function snapshotSetElements(elements: readonly Value[]): Value[] {
  if (!arrayIsArrayIntrinsic(elements)) {
    throw new CanonicalSetError(`set elements must be an Array, not ${describe(elements)}`)
  }

  // Copy indexed entries into an ordinary internal Array. `slice` is not a
  // neutral copy for an Array subclass: it consults `constructor[Symbol.species]`,
  // and the resulting object can override `map` or `sort`. That would let input
  // code replace the byte keys or suppress the canonical sort after every
  // element check passed. Capture the source length once, read each entry once,
  // and do all later work on our own plain Array.
  try {
    return copyArrayElements(elements)
  } catch (cause) {
    if (cause instanceof CanonicalSetError) throw cause
    throw new CanonicalSetError('set elements report an array shape that cannot be snapshotted')
  }
}

function encodeSetElement(
  element: Value,
  index: number,
  elementDepth: number,
): Uint8Array<ArrayBuffer> {
  try {
    return encodeAtDepth(element, elementDepth)
  } catch (cause) {
    if (cause instanceof CanonicalError) {
      throw new CanonicalSetError(`set element ${index} is not encodable: ${cause.message}`)
    }
    throw cause
  }
}

function compareBytes(left: Uint8Array, right: Uint8Array): number {
  const leftLength = byteLength(left)
  const rightLength = byteLength(right)
  const shared = leftLength < rightLength ? leftLength : rightLength
  for (let index = 0; index < shared; index++) {
    if (left[index] !== right[index]) {
      return (left[index] as number) < (right[index] as number) ? -1 : 1
    }
  }
  if (leftLength === rightLength) return 0
  return leftLength < rightLength ? -1 : 1
}

function appendArray<T>(target: T[], value: T): void {
  applyIntrinsic(arrayPushIntrinsic, target, [value])
}

function defineArrayElement<T>(target: T[], index: number, value: T): void {
  definePropertyIntrinsic(target, index, {
    configurable: true,
    enumerable: true,
    value,
    writable: true,
  })
}

function sortArray<T>(target: T[], compare: (left: T, right: T) => number): void {
  applyIntrinsic(arraySortIntrinsic, target, [compare])
}

/**
 * Copy one real Array without consulting its constructor, species, iterator,
 * or mutable prototype methods.
 */
function copyArrayElements<T>(source: readonly T[]): T[] {
  const length: unknown = source.length
  if (
    !numberIsSafeIntegerIntrinsic(length) ||
    (length as number) < 0 ||
    (length as number) > 0xffff_ffff
  ) {
    throw new CanonicalError('array reports an invalid length')
  }

  const snapshot: T[] = []
  for (let index = 0; index < (length as number); index++) {
    // Read the caller once before touching the destination. The captured
    // defineProperty intrinsic then creates an own data property even if that
    // getter has just poisoned Array.prototype or its indexed setters.
    const value = source[index] as T
    defineArrayElement(snapshot, index, value)
  }
  return snapshot
}

function bytesFromNumbers(source: number[]): Uint8Array<ArrayBuffer> {
  const output = new Uint8ArrayIntrinsic(source.length)
  for (let index = 0; index < source.length; index++) {
    output[index] = source[index] as number
  }
  return output
}

function copyBytes(source: Uint8Array): Uint8Array<ArrayBuffer> {
  let length: number
  try {
    const tag = applyIntrinsic(typedArrayTagIntrinsic, source, []) as string
    if (tag !== 'Uint8Array') {
      throw new CanonicalError(`bytes must be a Uint8Array, not ${tag}`)
    }
    length = byteLength(source)
  } catch (cause) {
    if (cause instanceof CanonicalError) throw cause
    throw new CanonicalError(
      'bytes have the right prototype but not the internals of a real Uint8Array',
    )
  }

  const output = new Uint8ArrayIntrinsic(length)
  for (let index = 0; index < length; index++) {
    output[index] = source[index] as number
  }
  return output
}

function byteLength(source: Uint8Array): number {
  return applyIntrinsic(typedArrayLengthIntrinsic, source, []) as number
}

function writeValue(out: number[], value: Value, depth: number): void {
  if (depth > MAX_DEPTH) throw new CanonicalError(`nesting deeper than ${MAX_DEPTH}`)
  // Both fields are read exactly once, here, and everything below works from
  // the captured copies.
  //
  // Reading twice is not a style question on this path. An adversarial pass
  // built a value whose `kind` getter returned `'bool'` to the shape check and
  // `'null'` to the dispatch, so the arm that validated was not the arm that
  // ran. Nothing downstream can restore the guarantee once the two reads can
  // disagree, and no per-arm check can notice.
  // Typed as the union of kinds so the `default` arm below still narrows to
  // `never`, which is what makes adding a variant to `Value` without a case
  // here a compile error. The assertion is about types only: at runtime the
  // string can be anything, and that is exactly what `default` handles.
  const kind = requireValueShape(value) as Value['kind']
  const payload: unknown = (value as { value?: unknown }).value

  switch (kind) {
    case 'uint': {
      requireRange(payload as bigint, 0n, U64_MAX, 'uint')
      writeHead(out, 0, payload as bigint)
      return
    }
    case 'neg': {
      requireRange(payload as bigint, I64_MIN, -1n, 'neg')
      writeHead(out, 1, -1n - (payload as bigint))
      return
    }
    case 'bytes': {
      // TypeScript's declared Uint8Array does not prove the runtime brand. A
      // forged or wider typed view must be refused before bytes reach output.
      if (!(payload instanceof Uint8ArrayIntrinsic)) {
        throw new CanonicalError(`bytes must be a Uint8Array, not ${describe(payload)}`)
      }
      // Copy the intrinsic byte length into a fresh plain Uint8Array. Typed
      // array `slice` consults Symbol.species, so a subclass could otherwise
      // return a wider view whose indexed values and backing bytes disagree.
      const raw = copyBytes(payload)
      const length = byteLength(raw)
      writeHead(out, 2, bigintIntrinsic(length))
      for (let index = 0; index < length; index++) {
        appendArray(out, raw[index] as number)
      }
      return
    }
    case 'text': {
      // `TextEncoder.encode` coerces a non-string instead of failing, and
      // `requireWellFormed` iterates `.length`, which is `undefined` on a
      // number — so the loop body never runs and nothing catches it.
      if (typeof payload !== 'string') {
        throw new CanonicalError(`text must be a string, not ${typeof payload}`)
      }
      requireWellFormed(payload, 'text string')
      const raw = utf8(payload)
      const length = byteLength(raw)
      writeHead(out, 3, bigintIntrinsic(length))
      for (let index = 0; index < length; index++) {
        appendArray(out, raw[index] as number)
      }
      return
    }
    case 'array': {
      // Without this, `.length` on a non-array is `undefined`, `BigInt(undefined)`
      // throws a native `TypeError`, and the module's contract that everything
      // it refuses is a `CanonicalError` quietly stops holding.
      if (!arrayIsArrayIntrinsic(payload)) {
        throw new CanonicalError(`array must be an Array, not ${describe(payload)}`)
      }
      // Snapshot before measuring. The head declares a count and the body then
      // writes the items, so reading the source twice lets a subclass with a
      // `length` getter, or an array mutated during iteration, declare one
      // number and emit another.
      let items: unknown[]
      try {
        items = copyArrayElements(payload)
      } catch (cause) {
        if (cause instanceof CanonicalError) throw cause
        throw new CanonicalError('array reports a shape that cannot be snapshotted')
      }
      writeHead(out, 4, bigintIntrinsic(items.length))
      for (let index = 0; index < items.length; index++) {
        writeValue(out, items[index] as Value, depth + 1)
      }
      return
    }
    case 'map': {
      const source = payload
      if (!(source instanceof MapIntrinsic)) {
        throw new CanonicalError(`map must be a Map, not ${describe(source)}`)
      }
      // Snapshot through a captured Map intrinsic, so a value getter that
      // poisons Map.prototype — or a subclass overriding entries, keys, get,
      // or size — cannot substitute pairs after validation.
      const pairs: { key: unknown; value: unknown }[] = []
      brandedCopy(() => {
        let index = 0
        applyIntrinsic(mapForEachIntrinsic, source, [
          (value: unknown, key: unknown) => {
            defineArrayElement(pairs, index, { key, value })
            index++
          },
        ])
      }, 'map')
      writeHead(out, 5, bigintIntrinsic(pairs.length))
      // A JavaScript Map iterates in insertion order, which is not encoding
      // order, so the keys are sorted here. Every key is checked before the
      // sort, so a refusal does not depend on insertion order.
      for (let index = 0; index < pairs.length; index++) {
        const key = (pairs[index] as (typeof pairs)[number]).key
        // A non-string key is the quietest failure in this module.
        // `requireWellFormed` iterates `.length`, which is `undefined` on a
        // number, so its loop never runs and the key passes — and `utf8` then
        // coerces `1` to `"1"`. A map keyed by a forged number encoded as one
        // keyed by text, silently.
        if (typeof key !== 'string') {
          throw new CanonicalError(`map keys must be strings, found ${describe(key)}`)
        }
        requireWellFormed(key, 'map key')
      }
      sortArray(pairs, (left, right) => compareKeys(left.key as string, right.key as string))
      for (let pairIndex = 0; pairIndex < pairs.length; pairIndex++) {
        const pair = pairs[pairIndex] as (typeof pairs)[number]
        const key = pair.key as string
        const raw = utf8(key)
        const length = byteLength(raw)
        writeHead(out, 3, bigintIntrinsic(length))
        for (let index = 0; index < length; index++) {
          appendArray(out, raw[index] as number)
        }
        if (pair.value === undefined) {
          throw new CanonicalError(`map key ${JSON.stringify(key)} has no value`)
        }
        writeValue(out, pair.value as Value, depth + 1)
      }
      return
    }
    case 'bool': {
      // The defect a progress review reproduced. This arm used JavaScript
      // truthiness — `value.value ? 0xf5 : 0xf4` — so a runtime-forged
      // `{ kind: 'bool', value: 'false' }` encoded as `f5`, canonical **true**,
      // and `hash` authenticated the opposite logical value. TypeScript types do
      // not protect an object deserialized from JSON, RPC, a plugin, or
      // `unknown`, which is why every other arm here already validates.
      // Captured as `unknown` first: the declared type is `boolean`, so
      // narrowing on it leaves `never` and the compiler stops helping exactly
      // where the runtime check is needed. The declared type is what is in
      // doubt.
      const flag = payload
      if (typeof flag !== 'boolean') {
        throw new CanonicalError(`bool must be a boolean, not ${describe(flag)}`)
      }
      appendArray(out, flag ? 0xf5 : 0xf4)
      return
    }
    case 'null': {
      appendArray(out, 0xf6)
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
      const unhandled: never = kind
      throw new CanonicalError(
        `value kind ${JSON.stringify(unhandled as unknown)} is not in the profile`,
      )
    }
  }
}

/**
 * Run a captured collection intrinsic that reads internal slots, turning its
 * native brand failure into a `CanonicalError`.
 *
 * `instanceof` only inspects the prototype chain, so `Object.create(Map.prototype)`
 * passes it without Map internals. A caller must still be able to distinguish
 * invalid input from an encoder failure.
 */
function brandedCopy<T>(copy: () => T, kind: string): T {
  try {
    return copy()
  } catch {
    throw new CanonicalError(
      `${kind} has the right prototype but not the internals of a real Map`,
    )
  }
}

/**
 * Name a runtime value for a refusal message, without trusting its `toString`.
 *
 * A forged payload is exactly the kind of object whose `toString` might throw or
 * lie, and this runs on the path that is refusing it.
 */
function describe(value: unknown): string {
  if (value === null) return 'null'
  if (arrayIsArrayIntrinsic(value)) return 'an Array'
  const kind = typeof value
  return kind === 'object' ? (value?.constructor?.name ?? 'an object') : kind
}

/**
 * Reject anything that is not shaped like a {@link Value} before its payload is
 * read.
 *
 * The `default` arm below catches an unrecognized `kind`. It cannot catch a
 * `value` that is not an object at all, or one with no `kind`: reading
 * `value.kind` on `undefined` throws a native `TypeError`, and this module's
 * contract is that everything it refuses is a `CanonicalError`. A caller
 * distinguishing "invalid input" from "the encoder is broken" needs that
 * distinction to hold.
 */
function requireValueShape(value: Value): string {
  if (typeof value !== 'object' || value === null) {
    throw new CanonicalError(`a value must be an object, not ${describe(value)}`)
  }
  const kind: unknown = (value as { kind?: unknown }).kind
  if (typeof kind !== 'string') {
    throw new CanonicalError(`a value must carry a string \`kind\`, not ${describe(kind)}`)
  }
  // Returned rather than re-read by the caller, so the kind that was validated
  // is necessarily the kind that is dispatched on.
  return kind
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
    const unit = applyIntrinsic(stringCharCodeAtIntrinsic, value, [i]) as number
    if (unit < 0xd800 || unit > 0xdfff) continue
    // A trailing surrogate here has no leading partner, and a leading surrogate
    // must be followed by a trailing one.
    if (unit >= 0xdc00) throw new CanonicalError(`${kind} has an unpaired surrogate at ${i}`)
    const next =
      i + 1 < value.length
        ? (applyIntrinsic(stringCharCodeAtIntrinsic, value, [i + 1]) as number)
        : -1
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
    appendArray(out, majorBits | numberIntrinsic(argument))
  } else if (argument <= 0xffn) {
    appendArray(out, majorBits | 0x18)
    pushBytes(out, argument, 1)
  } else if (argument <= 0xffffn) {
    appendArray(out, majorBits | 0x19)
    pushBytes(out, argument, 2)
  } else if (argument <= 0xffffffffn) {
    appendArray(out, majorBits | 0x1a)
    pushBytes(out, argument, 4)
  } else {
    appendArray(out, majorBits | 0x1b)
    pushBytes(out, argument, 8)
  }
}

function pushBytes(out: number[], argument: bigint, width: number): void {
  for (let shift = (width - 1) * 8; shift >= 0; shift -= 8) {
    appendArray(out, numberIntrinsic((argument >> bigintIntrinsic(shift)) & 0xffn))
  }
}

/**
 * Decode canonical bytes, rejecting anything that is not the unique canonical
 * encoding of the value it denotes.
 */
export function decode(input: Uint8Array): Value {
  // Snapshot the intrinsic byte length into a plain view. Besides giving the
  // parser stable bytes, this refuses forged typed-array prototypes and avoids
  // every Symbol.species path.
  const reader = new Reader(copyBytes(input))
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
    return byteLength(this.input) - this.position
  }

  private take(count: number): Uint8Array {
    const end = this.position + count
    if (end > byteLength(this.input)) {
      throw new CanonicalError('input ended inside an item')
    }
    const slice = new Uint8ArrayIntrinsic(count)
    for (let index = 0; index < count; index++) {
      slice[index] = this.input[this.position + index] as number
    }
    this.position = end
    return slice
  }

  private takeByte(): number {
    return this.take(1)[0] as number
  }

  private peekByte(): number {
    if (this.position >= byteLength(this.input)) {
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
      argument = bigintIntrinsic(additional)
    } else if (additional === 24) {
      argument = bigintIntrinsic(this.takeByte())
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
    const bytes = this.take(width)
    const length = byteLength(bytes)
    for (let index = 0; index < length; index++) {
      value = (value << 8n) | bigintIntrinsic(bytes[index] as number)
    }
    return value
  }

  /**
   * Convert a declared length only after proving the bytes exist, so a hostile
   * length header cannot force a large allocation.
   */
  private checkedLength(declared: bigint): number {
    const remaining = bigintIntrinsic(this.remaining())
    if (declared > remaining) {
      throw new CanonicalError(
        `declared length ${declared} exceeds ${remaining} remaining byte(s)`,
      )
    }
    return numberIntrinsic(declared)
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
        return bytes(this.take(this.checkedLength(argument)))
      case 3:
        return text(this.readText(argument))
      case 4: {
        const length = this.checkedLength(argument)
        const items: Value[] = []
        for (let i = 0; i < length; i++) appendArray(items, this.readValue(depth + 1))
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
      return applyIntrinsic(textDecoderDecodeIntrinsic, decoder, [raw]) as string
    } catch {
      throw new CanonicalError('text string is not well-formed UTF-8')
    }
  }

  private readMap(argument: bigint, depth: number): Value {
    const declared = this.checkedLength(argument)
    const entries = new MapIntrinsic<string, Value>()
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

      applyIntrinsic(mapSetIntrinsic, entries, [key, this.readValue(depth + 1)])
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
 *
 * This is a generic `pce/1` primitive, not an artifact-schema verdict. Future
 * authorization code must use a typed boundary that validates the complete
 * schema before it reaches this digest operation.
 */
export async function hash(value: Value): Promise<Uint8Array> {
  return digestOf(encode(value))
}

/**
 * Hash bytes, after proving they are the canonical encoding of some value.
 *
 * The proof is {@link decode} itself, which accepts only the unique canonical
 * encoding — so bytes that survive it are canonical by construction rather than
 * by the caller's say-so.
 *
 * This proves only `pce/1` byte canonicality. It cannot know whether an Array
 * occupies a schema-declared set field or whether an artifact satisfies any
 * other domain invariant. A future typed artifact boundary must complete
 * schema validation before hashing the exact validated bytes and must not use
 * this generic function as its authorization decision.
 *
 * The parameter is `Uint8Array<ArrayBuffer>`, not a bare `Uint8Array`. Since
 * TypeScript 5.7 the array is generic over its backing buffer, and Web Crypto's
 * `BufferSource` excludes `SharedArrayBuffer`-backed views — a shared buffer can
 * be mutated by another thread while the digest is being computed, so hashing
 * one has no well-defined result. Stating the requirement in the signature makes
 * that a caller's compile error rather than this function's silent assumption.
 *
 * @throws CanonicalError if the bytes are not the unique canonical encoding.
 */
export async function hashEncoded(input: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
  // Validate and hash **the same bytes**. `decode` walks the array through its
  // `length` property while `crypto.subtle.digest` reads the underlying buffer,
  // so a view whose `length` is shadowed — a `Uint8Array` subclass, say — got
  // its prefix validated and its whole buffer hashed. The digest would then
  // cover bytes nothing proved canonical, which is the one thing this function
  // exists to prevent. A species-free copy reads the intrinsic byte length
  // into a fresh plain Uint8Array, so both steps see the same bytes even when a
  // subclass requests a wider typed-array species.
  const snapshot = copyBytes(input)
  decode(snapshot)
  return digestOf(snapshot)
}

/**
 * SHA-256 over bytes this module has just produced or just validated.
 *
 * Deliberately not exported. Every caller is in this file and holds bytes that
 * {@link encode} returned or {@link decode} accepted a line earlier, so the
 * precondition is visible at the call site rather than asserted in a comment.
 */
async function digestOf(input: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
  const digest = (await applyIntrinsic(subtleDigestIntrinsic, subtleIntrinsic, [
    'SHA-256',
    input,
  ])) as ArrayBuffer
  return new Uint8ArrayIntrinsic(digest)
}

/** Lowercase hexadecimal, matching the Rust `Hash::to_hex` output. */
export function toHex(input: Uint8Array): string {
  let out = ''
  const digits = '0123456789abcdef'
  const length = byteLength(input)
  for (let index = 0; index < length; index++) {
    const byte = input[index] as number
    out += (digits[byte >> 4] as string) + (digits[byte & 0x0f] as string)
  }
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
  const out = new Uint8ArrayIntrinsic(input.length / 2)
  for (let i = 0; i < byteLength(out); i++) {
    out[i] = Number.parseInt(input.slice(i * 2, i * 2 + 2), 16)
  }
  return out
}
