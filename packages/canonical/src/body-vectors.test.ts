/**
 * TypeScript half of the MODEL-005 parity proof for the increment-3 body
 * schemas: the topology-snapshot body, the plan body, and the node-entry
 * map both build on.
 *
 * `schemas/domain/body-vectors.json` is written by the Rust constructors
 * and re-verified against them on every Rust run; this suite proves the
 * TypeScript codec reproduces the same canonical bytes and digests from
 * the same value trees, so an authorization hash computed on either side
 * of the RPC boundary is the same hash. No domain constructor exists in
 * TypeScript on purpose — this side re-encodes decided trees, it does not
 * build topologies.
 */

import assert from 'node:assert/strict'
import { test } from 'node:test'

import { decode, encode, hash, hashEncoded, toHex } from './canonical.ts'
import { type BodyVector, loadBodyVectors } from './vectors.ts'

const vectors = loadBodyVectors()
const every: readonly BodyVector[] = [
  ...vectors.snapshots,
  ...vectors.plans,
  ...vectors.nodeEntries,
]

test('the body fixture is not quietly shrunk', () => {
  assert.ok(vectors.snapshots.length >= 4, 'snapshot vectors present')
  assert.ok(vectors.plans.length >= 2, 'plan vectors present')
  assert.ok(vectors.nodeEntries.length >= 9, 'node-entry vectors present')
})

test('every body vector encodes to its recorded canonical bytes', () => {
  for (const vector of every) {
    assert.equal(toHex(encode(vector.value)), vector.canonical, vector.name)
  }
})

test('every body vector hashes to its recorded digest', async () => {
  for (const vector of every) {
    assert.equal(toHex(await hash(vector.value)), vector.sha256, vector.name)
  }
})

test('every body vector decodes back to a tree that re-encodes identically', () => {
  for (const vector of every) {
    const decoded = decode(encode(vector.value))
    assert.equal(toHex(encode(decoded)), vector.canonical, vector.name)
  }
})

test('hashEncoded agrees with hash over every body vector', async () => {
  for (const vector of every) {
    const encoded = encode(vector.value)
    assert.equal(toHex(await hashEncoded(encoded)), vector.sha256, vector.name)
  }
})

test('each plan binds the digest recorded for its named snapshot', () => {
  for (const plan of vectors.plans) {
    assert.ok(plan.snapshot, `${plan.name} names its snapshot`)
    const snapshot = vectors.snapshots.find((entry) => entry.name === plan.snapshot)
    assert.ok(snapshot, `${plan.name} names a snapshot the fixture carries`)
    assert.equal(plan.value.kind, 'map', `${plan.name} is a body map`)
    if (plan.value.kind !== 'map') continue
    const bound = plan.value.value.get('snapshot_hash')
    assert.ok(bound && bound.kind === 'bytes', `${plan.name} carries snapshot_hash bytes`)
    if (!bound || bound.kind !== 'bytes') continue
    assert.equal(
      toHex(bound.value),
      snapshot.sha256,
      `${plan.name} must bind ${plan.snapshot}'s recorded digest`,
    )
  }
})

test('each node entry appears verbatim in its snapshot body', () => {
  for (const entry of vectors.nodeEntries) {
    assert.ok(entry.snapshot, `${entry.name} names its snapshot`)
    const snapshot = vectors.snapshots.find((body) => body.name === entry.snapshot)
    assert.ok(snapshot, `${entry.name} names a snapshot the fixture carries`)
    assert.equal(snapshot.value.kind, 'map', `${entry.snapshot} is a body map`)
    if (snapshot.value.kind !== 'map') continue
    const nodes = snapshot.value.value.get('nodes')
    assert.ok(nodes && nodes.kind === 'array', `${entry.snapshot} carries a nodes set`)
    if (!nodes || nodes.kind !== 'array') continue
    const encodedEntry = toHex(encode(entry.value))
    assert.ok(
      nodes.value.some((node) => toHex(encode(node)) === encodedEntry),
      `${entry.name} must appear byte-identically in ${entry.snapshot}'s nodes`,
    )
  }
})
