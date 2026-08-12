# ADR-0031: CAP-002 is a required minimum over a closed-and-versioned operation vocabulary

- Status: Accepted
- Date: 2026-08-12. Accepted by Nate McBride the same day, by directive
  ("finish SI-25 and SI-26") on the adversarially reviewed
  recommendation round of the same day, following ten identical
  delegated arcs; the directive is recorded here as the acceptance
  basis (`docs/reviews/SI-25_RECOMMENDATION_ROUND_2026-08-12.md`, an
  untracked session artifact; this ADR restates everything load-bearing
  from it).
- Spec version: 12.9.0 (minor under §0.1 — CAP-002's sentence stands
  verbatim; additions only; argued in Decision)
- Work packages blocked: none — WP-050's delivered `Operation` enum
  stands until its next reviewed increment extends it under this
  discipline
- Requirement IDs: CAP-002 (amended); CAP-003, CAP-005, DIA-004,
  DIA-005, PART-007, PART-010, PART-011, MODEL-003 (read, none amended)
- Decision owners: Nate McBride

## Context

CAP-002 enumerates fourteen operations including a single `wipe`.
DIA-005 requires overwrite, crypto-erase, sanitize, format, discard,
and file deletion distinguished and never called equivalent — one
`wipe` cannot be six never-equivalent things. PART-007 (split/merge),
PART-010 (MBR↔GPT conversion), and PART-011 (clone-and-reformat
migration) map to no CAP-002 operation at all. SI-25 filed the
question: closed enumeration or required minimum? WP-050's delivered
engine already carries an `Operation` enum spelled from CAP-002's
names, so the answer also fixes that enum's extension discipline.

## Safety analysis

**The list is a floor.** CAP-002 names the operations that MUST be
modeled separately wherever they exist; it was never a claim that no
other operation may exist — PART-007/010/011 are existing normative
operations the list simply predates, and they join as named operations
when their packages build them, because their feasibility (a
conversion's structural preconditions, a merge's compatibility) is not
derivable from member operations.

**The vocabulary is closed at every moment and versioned across
time.** Additions arrive only through reviewed spec changes under
MODEL-003's schema discipline — the WP-050 reason-enum precedent —
never by drift. This is what reconciles the two readings' true halves:
the floor keeps DIA-005 implementable and required operations
representable; the instant-closure keeps CAP-005's one-engine promise
stable, because no surface can carry an operation the versioned
vocabulary does not.

**`wipe` is a family, and DIA-005's kinds are its members.** When
erase surfaces are built, the six kinds are separate operations:
capability genuinely differs per kind — sanitize requires DIA-004's
device/power/frozen-state checks, discard requires TRIM (DIA-003),
crypto-erase requires an encrypted layer — so separate modeling is
CAP-002's own separate-modeling principle applied, and it makes
DIA-005's never-equivalent **structural** rather than a behavior to
police. The kind-discriminated alternative (one `wipe` with a kind
field) was rejected as equivalence at the modeling layer — the exact
thing DIA-005 forbids.

**The delivered enum stands until extended under this discipline.**
`crates/capability`'s `Operation` is Rust; any extension is WP-050's
next reviewed increment, with the vocabulary-closure test moving in
the same change — the standing code-debt pattern this register has
recorded five times.

## Options considered

### Option (a) — closed enumeration

Rejected: makes DIA-005 unimplementable and PART-007/010/011
permanently unmodelable — the fail-closed posture spent making
required features unrepresentable.

### Option (b) — open minimum without versioning

Rejected: surfaces drift apart and CAP-005's one-engine promise
degrades to a hope.

### Option (c) — required minimum over a closed-and-versioned vocabulary (accepted)

Accepted, scoped as above.

### Option (d) — kind-discriminated `wipe`

Rejected: a discriminant is equivalence at the modeling layer, and
capability answers differ per kind anyway.

## Decision

Option (c), landed as spec 12.9.0's amendment to CAP-002 and only
CAP-002. **SI-25 moves to Resolved.**

**Minor under §0.1:** CAP-002's sentence stands verbatim; the floor,
versioning, and family rules are additions; DIA-004, DIA-005,
PART-007/010/011, CAP-003, and CAP-005 are untouched. The
counter-argument (a first extension-discipline fixes semantics the
delivered enum depended on) is recorded; not taken because no existing
requirement text changes.

## Consequences

- **Positive.** DIA-005 becomes implementable as structure;
  PART-007/010/011 become representable; CAP-005's agreement is
  guarded by closure-per-version rather than discipline.
- **Negative, accepted knowingly.** Every future operation costs a
  reviewed spec change — deliberate friction, the same reviewed-bump
  posture the envelope and handshake schemas already carry.
- **For WP-050.** The enum extension and its closure-test move ride
  the next reviewed increment; the six erase kinds arrive with the
  packages that build erase surfaces, each with its own capability
  arms.

## Verification

Owned by the packages that build them: the vocabulary-closure test
extended per version (the WP-040 claim-closure shape); the six erase
kinds as distinct operations with distinct capability answers when
built; a drift test that no surface carries an operation the versioned
vocabulary does not.

## Revisit conditions

- MODEL-003's versioning discipline changes; the closure-per-version
  rule reads the current one.
- An operation family arrives whose members' capability answers are
  provably identical; a discriminant might then be honest — that case
  files its own round rather than reading this one down.
