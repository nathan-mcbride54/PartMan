# ADR-C4: Provenance is a set of observations, held in the envelope

- Status: Accepted
- Date: 2026-07-28
- Spec version: 3.1.0
- Requirement IDs: MODEL-004, SAFE-003, SAFE-005, INV-007, UI-010, HLP-002,
  CAP-007, PLAN-009
- Resolves: SI-04
- Decision owners: repository CODEOWNERS

Acceptance basis: filed under Section 1.11, analysed with an adversarial review,
and delegated to the implementer. The review rejected part of the original
proposal; see "What was rejected".

## Context

MODEL-004 requires every discovered property to record "its source adapter"
(singular) and one of `authoritative`, `inferred`, `unavailable`, or
`conflicting`. But `conflicting` means two or more adapters disagreed, and a
single-source field cannot say which disagreed or what each reported. SAFE-005
requires ambiguous device identity to disable the affected write, which needs to
know what was ambiguous; INV-007 requires raw discovery evidence to be
inspectable; UI-010 requires errors to state a cause and a next step.

ADR-C2 changed the economics of this question. Provenance is now **envelope**
content, because HLP-002 makes client discovery an untrusted hint the helper
re-derives. In 2.0.0 a richer provenance shape was hash-visible, so every added
field churned plan hashes on adapter upgrades over unchanged hardware. That cost
is gone, and with it the only argument for the impoverished shape.

## Decision

**A provenance record holds a set of observations.** Each observation names its
source adapter and adapter version, the method used, and an outcome:

- `observed(bytes)` — the value read, as its `pce/1` canonical bytes.
- `unavailable(reason)` — the adapter looked and the platform did not expose it.
- `failed(error)` — the read itself errored.

Carrying each reported value as canonical bytes makes "these adapters
disagreed" mean byte inequality: total, decidable, and identical in Rust and
TypeScript, with no per-type equality to implement and no Unicode-normalization
ambiguity, since the encoding profile already assigns normalization to adapters.

**MODEL-004's four confidence values are derived from the observation set, never
stored.** One directly observed value is `authoritative`; a heuristic method is
`inferred`; zero observed is `unavailable`; two or more observed with distinct
canonical encodings is `conflicting`. Storing them alongside the observations
would permit a record that claims `authoritative` while holding two disagreeing
reads. Deriving them makes that unrepresentable.

**Provenance lives entirely in the envelope**, per ADR-C2. It is an explanation
and a fail-closed hint, never an input to a privileged decision.

**SAFE-005's ambiguous-identity refusal is computed by each party from its own
discovery** — authoritatively by the helper, advisorily by the planner — and
never from a counterparty's supplied confidence. The planner's advisory copy
exists so PLAN-009 holds: a dry run that passes should mean only physical
outcomes remain uncertain, so a doomed plan must be refused at plan time rather
than surfacing as an opaque late revalidation mismatch.

## What was rejected, and why

The original proposal also collapsed the **body** to `resolved(value)` or
`unresolved` per discovered property, with `unavailable` and `conflicting` both
yielding `unresolved` and the body forbidden from recording why. **Rejected on
two independent grounds.**

**It weakens a Section 3 MUST.** SAFE-003 requires every plan that writes
storage to bind each target to a record containing "all available identifiers".
Take the case SI-04 exists for: two adapters read a serial and disagree. That
serial *is* available — twice over. A body that omits it violates SAFE-003's
completeness requirement. Section 0.2 forbids an ADR from weakening a MUST.

**It erases the distinction ADR-C3 depends on.** A partition-table checksum is a
discovered property, so under the collapse a factory-blank disk (`Absent`) and a
disk whose GPT failed to parse (`Indeterminate`) become the same body value —
`unresolved` — and PART-001 would initialize the second as though it were the
first. Two decisions made independently would have combined into a data-loss
path that neither contains alone.

The reconciliation, which neither proposal stated: **a positively observed
absence is a value, not an unavailability.** `Absent` is something an adapter
determined; `unavailable` is something it could not determine. Conflating them
is what produced the hazard.

The body therefore keeps the discovered value. Disagreement is surfaced by the
helper's own re-derivation under SAFE-005, and explained from the envelope.

## Consequences

- **This is a semantic change to MODEL-004**, so it needs a spec amendment, not
  an ADR alone.
- Envelope size grows. Every discovered property carries an observation set, and
  the overwhelmingly common single-observation case duplicates the value it
  describes. Snapshots grow materially on discovery-heavy hosts.
- **Observation outcomes carry arbitrary canonical bytes**, which is a route by
  which key material could reach a diagnosable artifact. SAFE-006 applies to the
  envelope exactly as it applies to the body: redaction happens before a value
  reaches provenance, and the observation set is not an exemption.
- SI-14 — derived properties such as free extents and alignment have no
  confidence rule — remains open, and this decision makes its absence more
  visible rather than less.

## Verification

- A record holding two `observed` outcomes with distinct bytes derives
  `conflicting`; one derives `authoritative`; zero derives `unavailable`. The
  stored-confidence variant is unrepresentable, proven by the absence of a
  constructor.
- Editing an envelope observation does not change the artifact hash; editing a
  body value does.
- A disputed serial still appears in the body, with the dispute visible in the
  envelope. This is the regression guard for the rejected collapse.
- A positively absent partition table and an unreadable one produce different
  body values (shared with ADR-C3).
- Redaction tests over the observation set (SAFE-006).
- Test tier: T1, unprivileged.

## Revisit conditions

- A consumer appears that cannot re-derive discovery the way the helper does,
  making envelope provenance untrustworthy for it.
- Envelope growth becomes a transport problem, at which point observations are a
  candidate for on-demand retrieval rather than a shape change.
