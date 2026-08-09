# ADR-0017: The continuity witness is a refusal input, never an assurance

- Status: Accepted
- Date: 2026-08-09. The round and its resolution chain were accepted
  together by Nate McBride the same day
  (`docs/reviews/SI-33_ROUND_2026-08-09.md`, an untracked session
  artifact; everything load-bearing is restated here).
- Spec version: 10.0.0 (major under §0.1 — SAFE-003's identity record
  gains a field and a consumer MUST NOT, which changes what an existing
  requirement's record contains)
- Work packages blocked: WP-010 increment 3 (SI-33 resolved; SI-11,
  SI-27, SI-28 unchanged — SI-28's floor and Mitigated-open state
  explicitly untouched)
- Requirement IDs: SAFE-003, SAFE-005, PLAN-006, PLAN-007, HLP-002,
  HLP-004, ADR-C2, ADR-C3, SEC-006, Section 0.2
- Decision owners: Nate McBride

## Context

SI-33 was filed by SI-28's round four as the only proposed mechanism
that discriminates two media whose recorded identity fields are equal:
witness non-interruption rather than identity. The S4 sittings made the
need concrete: a same-model reader pair presents one identical
placeholder serial at every serial-bearing layer, a card exchange is
invisible at every serial surface, and unit continuity across the
exchange is unverifiable unprivileged. For that population the
plan/swap/apply vector — seconds apart, one attach session — is
undetectable by any identity comparison, because every identity field
lies identically.

The liveness precondition was discharged 2026-08-05 on real hardware
with three limits declared before data existed, and this design lives
inside them: the ceiling (prompt movement cannot be attributed to
exchange-synchronous detection; the strongest recordable positive is
"no staleness observed under these conditions"), the bound (one reader,
one bridge, one build, slot-exchange family), and the non-monotonicity
(a measured decrease across a PnP-arrival boundary; re-arrival resets
to the epoch floor; the storage-node PDO name qualified as an
unprivileged epoch signal while ContainerId and the USB-node name were
refuted). The filing's own trap governs the shape: a witness that is
evaluable but stale fails open in exactly the vector it exists for —
"worse than no witness, because it converts an admitted gap into a
false assurance."

## Safety analysis

**The witness is a field of SAFE-003's identity record** — an epoch
token plus a counter reading, taken and verified by the helper at
validation like every other record field, re-read at revalidation and
before the first write. It is deliberately **not** a MODEL-005
authoring-set entry: the counter was measured on zero-access
unprivileged handles, so this is a client-readable field the helper
re-derives and verifies — a serial's class, not a helper-only
derivation. The authoring set stays closed at its two entries.

**Scope: only where it means something.** The field is present on
exchange-capable targets, on platforms whose signal-plus-epoch
apparatus is qualified under the standing sitting discipline — the
ADR-0013 reach pattern applied to a witness. One apparatus is qualified
today (Windows: the `IOCTL_STORAGE_CHECK_VERIFY2` counter with the
storage-node PDO epoch token). Where the field is absent, targets keep
the conservative handling they already have: absence is the status
quo, never a regression. Fixed, strongly-identified media never carry
the field; SAFE-003's replug allowance continues to govern
strongly-identified removables, a disjoint population.

**Comparison semantics, exactly as the measurements dictate.** Readings
are comparable iff the epoch token is unchanged **and** the apply-time
value is not below the stamp. Then:

- same epoch, value equal → **`no-exchange-observed`** — the ceiling's
  own words, and the strongest word the closed vocabulary contains;
- same epoch, value moved → an exchange may have occurred: for covered
  targets this is SAFE-003's "identity has changed," and the plan
  rejects before the first write;
- epoch token changed, or value **decreased within a token** — a reset
  the token failed to witness, the adversarial round's sharpest finding
  — → incomparable: covered targets reject.

**`no-exchange-observed` never relaxes anything.** No plan proceeds on
the witness that would not have proceeded without it. This is the
fail-closed inversion of the filing's trap: on unmeasured hardware a
stale counter costs exactly the assurance that was never claimed, while
movement is always a genuine event-stream fact. The consumer rule is
normative: a consumer MUST NOT treat `no-exchange-observed` as evidence
of continuity. The closed outcome vocabulary
(`no-exchange-observed` / `exchange-observed` / `incomparable` /
`unavailable`) contains no word an assurance could borrow.

**SI-28's floor is not relaxed, and the route is named rather than
lost.** The relaxation SI-33 was filed hoping for is unsound off the
measured apparatus, by the recorded bound. It becomes this ADR's named
revisit condition: an apparatus-qualification mechanism — per-family
qualification evidence, or an in-session user-mediated exchange ritual
whose counter movement demonstrates liveness on the hardware in
question — with its own round and its own evidence. SI-33 resolves
delivering the safety half; the convenience half keeps its name.

**Placement falls out of the field's home.** The identity record is
body content by ADR-C2's standing rule; the witness is hashed with the
record it belongs to. Body-stability holds within an epoch (no media
events, no movement — the double-read stability rows); an epoch
boundary inside PLAN-007's validity window changes the body, and for
the covered population that is the correct outcome — continuity across
the boundary is genuinely unknowable. SI-33 was classed
non-hash-visible while placement was open; this decision makes it
hash-visible through placement, and the register row is corrected
rather than left contradicting it. The epoch token adds no redaction
category the record does not already carry: SAFE-003 already binds the
OS device instance identifier and connection path, the token's class.

## Options considered

### A bare counter comparison, equality-only

Rejected by measurement: a reading decreased across a PnP boundary, so
equality-only is unsafe — the filing's own liveness note said a design
must characterize the epoch or use another witness, and the epoch was
characterized.

### A witness that upgrades — unchanged readings relax SI-28's floor

Rejected as the filing's named trap realized: staleness on unmeasured
apparatus converts the admitted gap into false assurance. The recorded
bound (one reader, one bridge, one build) makes any product-wide
trust inference from an unchanged reading unsound.

### An envelope-resident witness

Rejected on ADR-C2's danger line: an envelope field is one an attacker
may alter without invalidating an approval, and altering a
refusal-input's stamp suppresses the refusal — a data-loss vector
wearing bookkeeping's clothes. The witness rides the body with the
record it protects.

### Defer SI-33 into increment 3

Rejected on both of the round's counts: the record type must decide the
field's existence now or pay a hashed-schema major later, and the
safety delivery is real today — the S4 pair's swap vector is currently
undetectable and becomes a measured refusal on qualified apparatus.

### The refusal-only witness (accepted)

As specified in the Safety analysis.

## Decision

**SI-33 moves to Resolved.** SAFE-003's identity record gains the
continuity-witness field — epoch token plus counter reading, closed
outcome vocabulary, helper-verified at validation, re-read at
revalidation and pre-write — scoped to exchange-capable targets on
qualified apparatus, comparing under the semantics above, refusing on
movement or incomparability for covered targets, asserting nothing on
`no-exchange-observed`, and relaxing nothing anywhere. SI-28's interim
conservative floor and Mitigated-open state are untouched; its
relaxation route is this ADR's named revisit condition.

## Consequences

- **Positive.** The one measured undetectable data-loss vector — swap
  between plan and apply on identity-indistinguishable media — becomes
  a refusal on qualified apparatus, with liveness measured rather than
  hoped.
- **Negative, accepted knowingly.** Covered targets reject on any epoch
  boundary inside the validity window — a reboot or replug kills such
  plans, correctly, continuity being unknowable. And the witness
  delivers no convenience: SI-28's confirmations remain exactly as
  heavy as they are today.
- **Obligation forward (operator):** qualifying any second platform's
  apparatus is sitting work under the standing discipline, per the
  reach pattern; unqualified platforms simply lack the field.
- **Obligations forward (first write-capable increment), in SI-33's
  banner beside SI-35's and SI-34's:** the movement-rejects and
  incomparable-rejects arms demonstrated end to end on the S4 apparatus
  family, mutation-verified.

## Verification

- When increment 3's types exist: the witness field's outcome
  vocabulary is the closed set above, with no predicate or ordering a
  proceed decision could key on (the compile-fail pin pattern), and the
  field is absent-by-type for non-covered target classes.
- When a write path exists: the two banner obligations, on the
  stale-signature-free S4 shape (identical serials, distinct media).
- Register: SI-33 reads Resolved and hash-visible-via-placement; SI-28
  reads Mitigated-open with the relaxation route named here; any text
  reading `no-exchange-observed` as continuity is an error against this
  ADR.

## Revisit conditions

- **The floor-relaxation route:** an apparatus-qualification mechanism
  with measured in-session liveness — per-family evidence or a
  user-mediated exchange ritual — arrives with its own round; only then
  may an unchanged witness carry any weight beyond refusal's absence.
- A qualified apparatus is measured whose counter moves without media
  events inside a stable epoch (driver noise), which would convert
  genuine-movement refusals into spurious ones broadly enough to
  matter; the semantics would need a per-apparatus noise
  characterization.
- PLAN-007's validity windows lengthen materially, changing the
  epoch-boundary rejection's practical weight.
