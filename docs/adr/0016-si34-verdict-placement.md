# ADR-0016: The protection verdict is body content, helper-authored at validation

- Status: Accepted
- Date: 2026-08-09. The round and its resolution chain were accepted
  together by Nate McBride the same day
  (`docs/reviews/SI-34_ROUND_2026-08-09.md`, an untracked session
  artifact; everything load-bearing is restated here). This is the
  adversarial round SI-34's own entry recorded that its recommended
  option never had.
- Spec version: 9.0.0 (major under §0.1 — it changes what 8.0.0's
  closed authoring-set sentence claims)
- Work packages blocked: WP-010 increment 3 (SI-34 resolved; SI-11,
  SI-27, SI-28, SI-33 unchanged)
- Requirement IDs: MODEL-005, PLAN-006, PLAN-007, HLP-002, HLP-003,
  CAP-007, SAFE-003, SAFE-005, ADR-C2, Section 0.2
- Decision owners: Nate McBride

## Context

SI-34 asked whether the helper's derived protection verdict is frozen
into the hashed body. It was filed when round two's universal premise —
every client/helper asymmetry is a roster-identity fact — was refuted by
the stale-signature fixture: the single-answer interface reports only
the stale mdraid signature while the enumerating interface reports both
it and the live ext4. All three filed options answered one underlying
problem, **two observers authoring one body**: clamp both to an agreed
projection (a), drop the verdict from the body (b), or freeze a
cross-privilege freshness projection plus a monotone floor (c) — the
project review's recommendation, never adversarially reviewed, and
resting on two open dependencies: the projection's membership, and a
proof that extra evidence can never loosen a verdict.

Two things changed after filing. **ADR-0014 (spec 8.0.0) removed the
two-observer world**: the snapshot an authorized plan binds is
helper-produced at validation, PLAN-006 compares helper-authored against
helper-recomputed, and MODEL-005 carries a named
authoring-at-validation verb closed to partition-table state — closed
"so 'the helper writes some body fields' cannot creep by analogy,"
which is to say: extendable only by a decided register resolution.
**And the measurement campaign completed**: the both-views
stale-signature comparison SI-34's evidence clause names is measured on
real Linux (L10: both single-answer interfaces report exactly the stale
signature, the enumerating probe reports both, helper reads
double-capture byte-stable) and on macOS (M7: the client's projection
is byte-identical to blank — it sees neither signature nor their
conflict; M10: the helper's digests are distinct, the stale-pair
fixture differing at head and tail). SI-34's currency note predated
M10's taking and said the macOS half could not be measured until M10
exists; M10 exists, its readback is discharged, and the sentence is
corrected with this resolution.

## Safety analysis

**Placement: the hashed body.** ADR-C2's charter — bound device
identities are body because the authorization names its targets —
extends to the safety decision about those targets. The verdict a user
authorizes is in the bytes they authorize.

**Authorship: the helper, at validation.** MODEL-005's authoring set
reopens by this decision and closes again at exactly two entries:
partition-table state (8.0.0) and the derived protection verdict. Both
are fields only the helper can derive — the measured macOS client
cannot see the facts a verdict rests on at all — and both are stamped
during validate-plan, before HLP-003 binds authorization to the
resulting hash, and recomputed at revalidation and before the first
write (HLP-002, CAP-007's recomputation duty restated as the recompute
half of authoring). A client-authored value for either field never
validates. A client cannot declare an object safe — the property no
round disputed — now holds by construction: no client claim is
representable in a bindable artifact.

**Divergence rejects, strictly — because the stricter rule is already
normative.** Any verdict change within the bound target topology
between validation and apply is a body mismatch, and SAFE-003/PLAN-006
already reject the plan on changed identity or topology. Option (c)'s
floor, monotone-tightening protocol, and journaled-continue arm were
machinery for safely *relaxing* strictness across two observers; with
one author there is nothing to relax, and the machinery would be
complexity commemorating a dead constraint. Evidence outside the bound
targets never entered the bound snapshot and cannot mismatch it:
PLAN-006's own scope, "target topology," does the work (c) built a
projection artifact for.

**What dissolves with the second author, named so the dissolutions are
decisions rather than omissions:** the monotonicity proof obligation
(no client verdict exists to tighten — the dissolution SI-35's
helper-only move produced for its own field); the freshness-projection
artifact (freshness stays owned by PLAN-006, PLAN-007's windows,
revalidation, and SI-33's open route); and the evidence-clause contest
of a client `permitted` against a helper `refused` (no contest is
representable).

**The named-contract obligation — this round's sharpest finding.** The
intra-helper asymmetry is real: `wipefs` enumerates both signatures
where root `blkid -p` reports one. Single authorship dissolves the
cross-observer problem only if the verdict binds to a **named,
deterministic helper evidence contract** — which probes, in what
precedence, over which facts — exactly as ADR-0014 named the raw-sector
parser rather than "run a privileged tool." That contract requirement
transfers into SI-11's shape round as a hard input: an unnamed evidence
set would be round two's refuted universal premise returned. Its
re-probe stability is partially measured (the double-capture rows);
what SI-11's contract adds must bring its own stability measurement.

**Display cost, accepted a third time as policy.** Clients show neither
table state nor a protection verdict before validation; the honest
client surface is observations, the reach declaration, and
pending-validation. ADR-0013 priced this for detection and ADR-0014 for
table state; M7 is the proof by demonstration that the alternative is
fabrication.

## Options considered

### Option (a) as filed — keep freezing, clamp the projection

Rejected: the clamp existed to make two observers agree, and blinds the
helper to evidence it holds. With no client author, the clamp's purpose
is empty; the placement half of (a) survives in this decision, the
clamp does not.

### Option (b) — drop the verdict from the body

Rejected: it un-authenticates the one value the user most needs bound,
and under the ADR-0014-fork price list it pays condition 3 — naming
what the authorization hash stops committing to — to lose a property
single authorship keeps for free.

### Option (c) as filed — freshness projection plus monotone floor

Rejected as superseded, with its useful half kept: a client cannot
weaken the safety decision, now by construction rather than by floor.
Its two recorded dependencies — projection membership and the
monotonicity proof — were real costs paid to bridge two authors, and
both dissolve with the second author.

### Helper-authored body residence (accepted)

As specified in the Safety analysis.

## Decision

**SI-34 moves to Resolved.** The derived protection verdict is hashed
body content, authored by the privileged helper at validation from a
named evidence contract, recomputed at revalidation and before first
write, with any within-target divergence rejecting under the existing
SAFE-003/PLAN-006 rules. MODEL-005's authoring set holds exactly two
entries and remains closed to creep.

The verdict's internal shape — total fail-closed semantics, PART-014
classification, the reason vocabulary, the Storage Spaces and
sealed-volume cases (SI-29, SI-30), the unequal-identifier multipath
coverage (SI-37) — is SI-11's round, untouched. Node naming and edge
typing are SI-27's, and the filing's own correction stands: this
placement changes SI-27's protection-specific burden only. SI-28 and
SI-33 are untouched.

## Consequences

- **Positive.** The user authorizes the helper's own safety decision
  about the exact targets bound, and what was authorized is what
  executes or the plan rejects — option (b)'s silent-divergence cost
  and option (a)'s blinded-helper cost both structurally absent.
- **Negative, accepted knowingly.** Any within-target verdict change
  between validation and apply kills the plan; there is no
  journaled-continue arm. If write-path experience demands (c)'s
  relaxation, that is a future decision with its own evidence — 
  foreseen here, not foreclosed.
- **Obligation forward (SI-11):** the named helper evidence contract,
  with measured re-probe stability, as a hard input to the verdict
  shape round.
- **Obligations forward (first write-capable increment), recorded in
  SI-34's resolution banner beside SI-35's:** a helper-only fact that
  changes protection rejects before the first write with a structured
  divergence; out-of-target evidence blocks nothing; both demonstrated
  end to end when a write path exists to demonstrate them on.

## Verification

- When increment 3's types exist: the verdict field is
  helper-authored-only at the type level (no client constructor for a
  bindable artifact carrying it), the shape pinned the compile-fail way
  the interlock pins non-cloneability.
- When a write path exists: the two banner obligations above, on the
  stale-signature fixture family, mutation-verified.
- Register: SI-34 reads Resolved; SI-11's entry carries the
  named-contract input; any text implying this ADR decided verdict
  semantics is an error against it.

## Revisit conditions

- SI-11's shape round finds a verdict component that cannot be
  helper-derived deterministically from a nameable contract — the
  premise of single authorship would need re-examination, and the
  fork's price list governs any out-of-body retreat.
- A future decision reintroduces client-side verdict claims in any
  bindable artifact. This ADR reads MODEL-005's authoring set as
  closed; that decision would reopen more than placement.
- PLAN-006's target-topology scoping changes, which carries the
  out-of-target-evidence-blocks-nothing property.
