/**
 * Loader for the shared golden fixture.
 *
 * `schemas/canonical-encoding-vectors.json` is read by both implementations.
 * Neither language keeps its own copy, because two copies can drift and a
 * cross-language parity proof that compares an implementation against its own
 * table proves nothing.
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

import { type Value, array, bool, bytes, fromHex, map, neg, nullValue, text, uint } from './canonical.ts'

/** One entry of the shared fixture. */
export interface Vector {
  readonly name: string
  readonly value: Value
  readonly canonical: string
  readonly sha256: string
}

/** One producer-side schema-set ordering vector. */
export interface SetOrderingVector {
  readonly name: string
  readonly setDepth: number
  readonly input: readonly Value[]
  readonly canonical: string
  readonly sha256: string
}

/** One decoder-side schema-set validation vector. */
export interface SetValidationVector {
  readonly name: string
  readonly setDepth: number
  readonly observed: readonly Value[]
  readonly accepted: boolean
  readonly error?: 'duplicate' | 'not-strictly-increasing'
}

/** A compact inherited-depth case shared by both implementations. */
export interface SetDepthVector {
  readonly name: string
  readonly setDepth: number
  readonly elementArrayDepth: number
  readonly accepted: boolean
}

/** All cross-language vectors for the schema-level canonical-set rule. */
export interface SetVectors {
  readonly ordering: readonly SetOrderingVector[]
  readonly validation: readonly SetValidationVector[]
  readonly depth: readonly SetDepthVector[]
}

/**
 * The JSON representation of a value.
 *
 * Integers arrive as decimal strings. The fixture for a profile that exists
 * because JSON numbers cannot carry `u64` must not itself rely on JSON numbers.
 */
type JsonValue =
  | { uint: string }
  | { neg: string }
  | { bytes: string }
  | { text: string }
  | { array: JsonValue[] }
  | { map: [string, JsonValue][] }
  | { bool: boolean }
  | { null: true }

function build(json: JsonValue): Value {
  if ('uint' in json) return uint(BigInt(json.uint))
  if ('neg' in json) return neg(BigInt(json.neg))
  if ('bytes' in json) return bytes(fromHex(json.bytes))
  if ('text' in json) return text(json.text)
  if ('array' in json) return array(json.array.map(build))
  if ('map' in json) return map(new Map(json.map.map(([key, value]) => [key, build(value)])))
  if ('bool' in json) return bool(json.bool)
  if ('null' in json) return nullValue
  throw new Error(`unrecognized value representation: ${JSON.stringify(json)}`)
}

/** Absolute path of the shared fixture, resolved from this module's location. */
export function fixturePath(): string {
  const here = dirname(fileURLToPath(import.meta.url))
  return join(here, '..', '..', '..', 'schemas', 'canonical-encoding-vectors.json')
}

/** Absolute path of the schema-level canonical-set fixture. */
export function setFixturePath(): string {
  const here = dirname(fileURLToPath(import.meta.url))
  return join(here, '..', '..', '..', 'schemas', 'domain', 'canonical-set-vectors.json')
}

/** Load and build every vector in the shared fixture. */
export function loadVectors(): Vector[] {
  const raw = JSON.parse(readFileSync(fixturePath(), 'utf8')) as {
    profile: string
    vectors: { name: string; value: JsonValue; canonical: string; sha256: string }[]
  }
  if (raw.profile !== 'pce/1') {
    throw new Error(`fixture declares profile ${raw.profile}, expected pce/1`)
  }
  return raw.vectors.map((entry) => ({
    name: entry.name,
    value: build(entry.value),
    canonical: entry.canonical,
    sha256: entry.sha256,
  }))
}

/** Load the producer, validator, and inherited-depth set vectors. */
export function loadSetVectors(): SetVectors {
  const raw = JSON.parse(readFileSync(setFixturePath(), 'utf8')) as {
    schema: string
    schema_version: number
    rule: string
    ordering_vectors: {
      name: string
      set_depth: number
      input: JsonValue[]
      canonical: string
      sha256: string
    }[]
    validation_vectors: {
      name: string
      set_depth: number
      observed: JsonValue[]
      accepted: boolean
      error?: 'duplicate' | 'not-strictly-increasing'
    }[]
    depth_vectors: {
      name: string
      set_depth: number
      element_array_depth: number
      accepted: boolean
    }[]
  }
  if (
    raw.schema !== 'partman.canonical-set-vectors' ||
    raw.schema_version !== 1 ||
    raw.rule !== 'unsigned-lexicographic-full-pce-element-bytes'
  ) {
    throw new Error('canonical-set fixture declares an unsupported schema or ordering rule')
  }
  return {
    ordering: raw.ordering_vectors.map((entry) => ({
      name: entry.name,
      setDepth: entry.set_depth,
      input: entry.input.map(build),
      canonical: entry.canonical,
      sha256: entry.sha256,
    })),
    validation: raw.validation_vectors.map((entry) => ({
      name: entry.name,
      setDepth: entry.set_depth,
      observed: entry.observed.map(build),
      accepted: entry.accepted,
      ...(entry.error === undefined ? {} : { error: entry.error }),
    })),
    depth: raw.depth_vectors.map((entry) => ({
      name: entry.name,
      setDepth: entry.set_depth,
      elementArrayDepth: entry.element_array_depth,
      accepted: entry.accepted,
    })),
  }
}
