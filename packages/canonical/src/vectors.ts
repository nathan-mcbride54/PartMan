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
