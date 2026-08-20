# The protection-artifact store

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` REC-011 (the
  four ADR-0030 rules), PART-013 (ADR-0024's arms), JRN-004 (the
  inherited location clause), SAFE-006, SEC-009
- Owner: WP-070 (`docs/work-packages/WP-070.md`), the store increment
  commissioned by the increment-4b opening round
  (`docs/reviews/LINUX_4B_OPENING_ROUND_2026-08-20.md`, decision 1)
- Implementation: `crates/artifact-store`, over an injected
  [`StoreSeam`]; each helper's seam and on-disk root land under that
  helper's own grant (the table below)

This document records the delivered layout and discipline. It decides
nothing; the decisions are ADR-0030's and the opening round's.

## What the store is

The dedicated, helper-owned home for protection artifacts: PART-013's
parse-level table backups, the typed repair family's raw region
captures, and — with later packages — REC-011's encryption-metadata
backups. It is **sibling to and never inside the journal** (ADR-0030
Rule 1): journal records reference artifacts, artifact bytes never
enter journal records, and the store keeps no metadata of its own —
which plan an artifact insures, which PART-013 arm produced it, and
which regions a raw capture covers are the journal's protection
records' facts (`schemas/journal/records.md`). A second copy here
could disagree with the first.

## Object naming and layout (ADR-0030 Rule 2)

An artifact's identity is the SHA-256 of its exact bytes — the same
`content` hash its `ProtectionArtifactRef` carries in every journal
record and plan body. The store is one flat namespace of objects:

- **Object name:** the content hash rendered as exactly 64 lowercase
  hexadecimal characters. The name *is* the reference; there is no
  index, no sidecar, and no second identity to fall out of agreement
  with the first.
- **Object content:** the artifact's raw bytes, nothing else — no
  header, no framing, no envelope. Framing would change the bytes'
  hash and break the one identity.
- **Layout:** one file per object, named as above, directly in the
  store root. A directory entry that does not parse as a canonical
  object name is not an object; the store never repairs a spelling
  (uppercase included — one canonical spelling, or one directory could
  hold two objects with one identity).

A future artifact class needing store-side metadata arrives as its own
schema change under WP-070's grant, never as an ad-hoc sidecar.

## Verification (REC-011's "create and verify")

- **Deposit** hashes the offered bytes, writes the object, **re-reads
  it through the seam, and recomputes the hash** before handing back a
  reference. A reference in a caller's hands is proof the store could
  reproduce the exact bytes at deposit time. No failure arm returns a
  reference.
- **Fetch** re-verifies the held bytes against the reference before
  returning them. The store never serves content whose hash is not the
  name it was asked for by — a restore can never be fed a backup that
  no longer is one.
- **Ordering (PART-013's discharge):** the depositing helper appends
  the journal's protection record *after* the deposit's verified
  return, so no journal record ever references an artifact the store
  had not durably verified. `StoreSeam::put` is durable-on-return by
  contract; the platform truth of both sentences is each helper's
  Tier-2 acceptance obligation, exactly as JRN-002's seam records.

## Location (the inherited JRN-004 clause)

The store root is admin-protected and documented per OS, a sibling of
the journal's state directory, never inside it. Each row lands under
its helper's own grant:

| OS | Store root | Status |
| --- | --- | --- |
| Linux | a `/var/lib/partman`-sibling directory, root-owned `0700` | Reserved — lands with WP-L110 increment 4b |
| Windows | — | Lands with WP-W110 under its own grant |
| macOS | — | Lands with WP-M110 under its own grant |

Only the helper reads the store (SAFE-008); a raw store read outside
the helper is made structurally impossible per platform by that
platform's protections, proven in that helper's acceptance — ADR-0030
obligation 3's platform half, recorded in WP-070's assignment.

## Retention (ADR-0030 Rule 3 — the ADR-0029 liveness rule)

A retention pass classifies every held object from the decoded journal
and from nothing else:

- **Exempt:** referenced by at least one apply whose ADR-0029 linkage
  closure is live. Untouchable; no decision overrides this arm.
- **Terminated closure:** every referencing apply has wholly
  terminated. Eligible for an explicit end-of-life decision — and for
  nothing automatic.
- **Orphan:** referenced by no journal record. The closure cannot be
  proven terminated, so the object fails closed toward retention and
  is never reclaimed by any pass or decision.
- **Corrupt:** the held bytes no longer hash to the object's name.
  Never reclaimable, whatever the liveness — a corrupt recovery asset
  is a finding, not garbage.

A journal reference to an object the store does not hold is surfaced
by every pass: a record is promising bytes the store cannot produce.

## End of life (ADR-0030 Rule 4)

Deletion happens only through an explicit `DeleteDecision` naming one
artifact; the store recomputes the retention pass itself and refuses
every arm but a verified terminated closure. Silence retains — the
fail-closed direction. The deciding surface — SEC-009-shaped,
displayed, changeable — is the surface package's obligation and MUST
render both consequence sentences, pinned here and in the crate
(`RETAIN_CONSEQUENCE`, `DELETE_CONSEQUENCE`) in doc-code agreement:

> Retaining this backup preserves the state it captured at backup time: a passphrase or key revoked since then remains usable with the backup.

> Deleting this backup forfeits the disaster-recovery copy: metadata corrupted or lost later is restorable only from a backup that still exists.
