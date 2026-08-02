# ADR-C2: The body/envelope boundary for hashed artifacts

- Status: Accepted
- Date: 2026-07-27
- Spec version: 3.0.0
- Work packages unblocked at acceptance: WP-010 increment 3 (partially; see
  Scope); current issue dependencies live in the register
- Requirement IDs: MODEL-005, MODEL-004, Section 6, PLAN-006, PLAN-007,
  CONC-004, HLP-002, HLP-004, HLP-001, SEC-001, SEC-002
- Decision owners: repository CODEOWNERS

Acceptance basis: filed as SI-03, SI-05, and SI-06 in `docs/spec-issues/`, per
Section 1.11, and the decision owner then delegated the choice explicitly rather
than selecting an option. The alternatives below are recorded in full so a later
reader can see this was a decision, not a default.

## Context

Three filed conflicts turned out to be one question.

**SI-05.** Section 6 required `OperationPlan` to contain the "Cryptographic plan
hash", while MODEL-005 defined that hash as SHA-256 over the plan's canonical
bytes. A field cannot be both inside the bytes and the hash of those bytes. The
2.0.0 contract was not constructible.

**SI-06.** PLAN-006 and HLP-004 require the helper to re-discover topology and
reject a mismatch, which means comparing a fresh capture's digest against the
recorded one. But MODEL-005 hashed the whole artifact, and a topology snapshot
carries a capture timestamp, CONC-004's transitional marking, and MODEL-004
provenance. If those are hashed, **two probes of physically identical hardware
never produce equal digests**, and the freshness check can never pass.

**SI-03.** MODEL-004 requires every discovered property to record its source
adapter and confidence. Identity fields are discovered properties and are
covered by the plan hash. Nothing said whether that provenance was inside the
hashed bytes. If it is, a plan authorized while one adapter reported the device
rehashes differently after an adapter upgrade on unchanged hardware, breaking
HLP-003's binding. If it is not, a `conflicting` identity observation sits
outside the authorization boundary.

All three ask: **what is authenticated by the hash, and what merely travels
alongside it?**

## Safety analysis

The hash is the authorization boundary. HLP-001 applies plans by hash, HLP-003
binds interactive authorization to an exact hash, SEC-001 authorizes only exact
hashes, SEC-002 rejects altered plans. So the boundary between hashed and
unhashed is a security boundary, and every field placed outside it is a field an
attacker may alter without invalidating an approval.

The danger of an envelope is therefore over-use, not existence. A validity
window in the envelope would let an expired plan be revived by editing a
number. Bound device identities in the envelope would let an approval for one
disk be redirected to another. The rule below exists to make that class of
mistake hard rather than to leave it to judgement.

The opposite failure is real too: an over-inclusive body makes PLAN-006
unsatisfiable, and a freshness check that can never pass gets "fixed" by
relaxing the comparison — which is a far worse outcome than a slightly larger
envelope.

## Options considered

### Option A — Hash everything; drop the self-reference only

Keep every field in the body except the hash itself.

Benefits: the smallest possible unauthenticated surface, and the simplest rule.

Costs: does not work. Capture timestamp and provenance stay hashed, so PLAN-006
freshness can never pass on unchanged hardware. The check would have to be
weakened to a field-subset comparison, which reintroduces "which fields count?"
as an undocumented decision in the helper — the same problem, moved somewhere
less visible.

### Option B — Hash only a hand-listed subset

Enumerate the hashed fields per artifact in `schemas/`.

Benefits: total control; PLAN-006 works.

Costs: no principle, so every new field is a fresh argument, and the safe answer
is not the default one. A reviewer must notice that a newly added field was
omitted from the list, which is exactly the kind of omission review is worst at.
Over time the list drifts toward whatever was convenient.

### Option C — A body/envelope split with a derivation rule

Split every hashed artifact. Decide membership by a rule rather than a list:

> A field belongs in the envelope only if it is the hash itself, or the
> privileged helper independently re-derives it and treats the client's copy as
> an untrusted hint (HLP-002). Everything else belongs in the body.

Benefits: principled, and the default is the safe one — an unrecognised field
lands in the body, which is authenticated. The rule is already implied by
HLP-002: the helper re-discovers topology and treats client discovery output as
a hint, so hashing the client's provenance authenticates data the helper has
already decided not to trust. PLAN-006 works, because capture metadata is
envelope content.

Costs: "re-derives" needs the distinction below to be stated, or it will be
misread.

## Decision

**Option C.**

The distinction that makes the rule usable: **enforcing a value is not
re-deriving it.** The helper enforces the validity window (HLP-004); it does not
recompute when the plan should expire. So the window is body content. The helper
does re-discover topology (HLP-002), so discovery provenance is envelope
content. Applied to the artifacts that exist today:

| Field | Side | Why |
| --- | --- | --- |
| Plan hash | Envelope | It is the hash of the body |
| Validity window, creation timestamp | Body | Enforced, not re-derived |
| Bound device identities and strength | Body | The authorization names these targets |
| Step graph, risk, consequence keys, byte ranges | Body | The substance being authorized |
| Source snapshot body hash | Body | A reference to another artifact, not self-referential |
| Discovery provenance, adapter attribution | Envelope | HLP-002 makes the client's copy an untrusted hint |
| Snapshot capture timestamp | Envelope | Otherwise PLAN-006 can never pass |
| Snapshot transitional marking | **Body** | See below |

The transitional marking is the one place this ADR deliberately over-includes.
By the rule it could sit in the envelope, since the helper knows whether it is
executing. Putting it in the body makes "a transitional snapshot cannot be
hash-equal to a stable one" a property of the encoding rather than a check that
some code path might skip. CONC-004 says transitional snapshots are not valid
planning bases; this makes that unrepresentable rather than merely enforced.

## Consequences

Positive:

- Section 6 becomes constructible, and PLAN-006 becomes satisfiable.
- New fields have a default, and the default is authenticated.
- The unauthenticated surface is small and enumerable: a hash, and data the
  helper already refuses to trust.

Negative and to be managed:

- **Every hashed artifact now has two types**, not one. The domain model carries
  `PlanBody`/`Plan` and `SnapshotBody`/`Snapshot` pairs, and the codec hashes
  the body type only. A function taking the envelope type where the body was
  meant is a bug the type system must catch.
- **`conflicting` provenance is outside the authorization boundary.** That is
  correct under HLP-002 — the helper forms its own view — but it means SAFE-005's
  refusal on ambiguous identity is the *helper's* obligation, computed from the
  helper's own discovery, never from a client-supplied confidence value. This
  must be stated wherever that refusal is implemented.
- At acceptance, MODEL-004's shape was still unresolved under SI-04: whether a
  provenance record could hold several observations was a separate decision
  this ADR did not prejudge. SI-04 later resolved in spec 3.1.0 by ADR-C4 as a
  set of observations in the envelope, preserving rather than revising this
  ADR's authorization boundary.

## Verification

- Golden vectors for a body/envelope pair proving the hash covers the body and
  is unchanged by any envelope edit.
- A test that two snapshots of identical topology, differing only in capture
  timestamp and provenance, produce equal body hashes — the PLAN-006 property.
- A test that a transitional snapshot and a stable snapshot of the same topology
  produce **different** body hashes.
- A negative test that editing a validity window changes the plan hash.
- Cross-language parity for all of the above (MODEL-005), in the shared fixture.
- Test tier: T1, unprivileged.

## Revisit conditions

- Historical condition discharged without triggering a revisit: SI-04 resolved
  in spec 3.1.0 by ADR-C4 with provenance remaining outside the hash. A future
  normative change that puts provenance back inside the hash would trigger one.
- A field appears that the helper re-derives but which must still be
  authenticated, breaking the rule's assumption that those two sets are
  disjoint.
- A second consumer appears that cannot re-derive discovery the way the helper
  does, making the envelope untrustworthy for it.

## Scope

This decision unblocked the three conflicts about hashing that it was accepted
to resolve. **At acceptance on 2026-07-27**, WP-010 increment 3 remained blocked
on the other eight then-open issues: SI-01, SI-02, SI-04, and SI-07 through
SI-11. That list is retained only as an acceptance-time snapshot; the single
authoritative source for current issue classes, counts, and states is
`docs/spec-issues/README.md`'s **Status of every issue** table.
