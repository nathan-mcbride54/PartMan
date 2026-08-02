# ADR-0012: Non-goal protection is unrepresentable, not guarded

- Status: Accepted
- Date: 2026-08-02
- Spec version: 4.4.0
- Work packages blocked: WP-010 increment 3 (SI-11 remains a direct blocker;
  this decides its axis, not its closure rules)
- Requirement IDs: Section 2.1, PART-014, HLP-002, SAFE-005, Section 0.2
- Decision owners: Nate McBride

## Context

Section 2.1 says the product MUST NOT mutate ZFS, Storage Spaces, LDM, or
Fusion — absolute, with no override: Section 0.2 grants override authority
only to Section 3, and Section 2.1 is not in Section 3. The mechanism the
spec supplies, PART-014 protected objects, is defined as refusal "without an
explicit supported plan" — **bypassable by construction** — and its
enumerated list does not include pool members, ZFS, Storage Spaces, or LDM
at all.

SI-11 filed the axis question: is a detect-only marking a **type-level
impossibility** — a plan step naming such a node is unrepresentable, the
grammar has no sentence for it — or a **runtime guard** that rejects a
representable step? The register's own words: the two are not
interchangeable, and only the first survives a bug in the guard.

Three design rounds produced mechanisms and three were rejected, each for
its own recorded reason: round one for a PART-014/MAC-009 status-mapping
conflict (it removed sealed volumes from PART-014 and required `unsupported`
where MAC-009 requires `blocked` — Part 4); round two for upward sibling
capture through containment plus the SI-27 naming gap (Part 5); round three
for the missing downward production rule, a residual arm that defaulted to
permitted, and — among its six defects — a constructor that collapsed to
the runtime-guard axis (Part 6). **The axis was never adjudicated by those
rejections**: two rounds failed off-axis entirely, and round three's drift
onto the guard side was itself one of its defects — which is evidence for
fixing the axis deliberately rather than leaving it to be decided by drift.

Deciding the axis now, before WP-010 increment 3 writes the plan type, is
exactly the sequence the register demands: the choice is structural, it
shapes the type, and reversing it after hashes are issued has no cheap exit.

## Safety analysis

- **This strengthens enforcement and weakens nothing.** A MUST NOT whose
  enforcement is a runtime check is one guard bug away from silent violation;
  a MUST NOT whose violating sentence cannot be constructed fails closed at
  the point of construction. The runtime layer is retained, not replaced.
- **Defense in depth, with its independence stated at true width.** The
  helper independently recomputes protection (HLP-002) and refuses
  regardless of what a client constructed. That second layer stays. For bugs
  **outside the shared verdict computation**, a client bug and a helper bug
  must now coincide before a violating write is even attempted. The scoping
  matters twice over: HLP-002 requires independent re-discovery and
  recomputation — independent inputs and timing, not an independent
  implementation — and the architecture is one shared Rust domain, so a
  defect in the shared closure code is both layers' bug at once (the closure
  caveat below). And the two layers compute from **different inputs**: the
  register records a measured case where the unprivileged view carries one
  signature of two and the privileged probe sees both. Where the
  protecting fact is invisible to the client, a violating plan constructs
  with no bug anywhere, and the operative layer is the helper's — for that
  input class the guarantee does not rest on unrepresentability, and saying
  otherwise would overclaim.
- **What this does not fix, stated so it cannot be rounded up.** Two
  distinct gaps survive this decision. *First, the closure.* Both axes
  consume the same computed protection verdict, and a closure that
  **wrongly computes permitted** — the documented root-on-ZFS-over-LUKS
  case, where deleting the partition reaches the encryption layer and never
  sees the pool below — defeats both axes identically: the node is unmarked
  and nothing fires. A closure that **produces no verdict at all** is a
  different case, and the axes are *not* symmetric there: a plan-type
  constructor can require a positive `permitted` before a step is even
  buildable — fail-closed by construction — where a guard checking only for
  a protected mark falls through open. That asymmetry is exactly the
  fail-closed-residual design space, it is round four's to use, and it is
  one more reason the axis choice matters. The closure rules — round
  three's downward dependency rule and fail-open residual — remain
  undecided and remain SI-11's. *Second, client observability.*
  Unrepresentability binds relative to client-visible topology; the
  measured single-signature udev view establishes that a protection input can
  be invisible to the client. **If** that helper-only input changes the
  verdict, a plan may construct without a client bug and helper refusal becomes
  the operative layer. Whether it changes the verdict is untested, as the
  register is careful to say. **This ADR removes the guard-bug failure class
  for client-visible facts, and no more.**
- **Ambiguity remains fail-closed.** SAFE-005 and the round-three Regime A′
  mapping are untouched: an object that cannot be positively classified is
  blocked, not defaulted.

## Options considered

### Option A — runtime guard on a representable step

The plan type can express "delete this ZFS member"; validation rejects it.
Rejected: PART-014's own definition ("without an explicit supported plan") is
bypassable by construction; a guard bug yields a silent Section 2.1 violation
with no override authority anywhere; and the register filed precisely this
distinction, naming the guard as the option that does not survive its own
bug. Round three's mechanism failed partly by collapsing to this.

### Option B — type-level unrepresentability, runtime layer retained (accepted)

A mutating plan step whose target resolves, from the client's visible topology,
to a Section 2.1 non-goal node is unrepresentable in the plan type: constructing
it is a type error, not a validation failure. The helper's independent
recomputation remains as the second layer. For client-visible protecting facts,
the construction layer is the primary guarantee. For bugs outside the shared
verdict computation, client-side construction and helper-side recomputation are
separate chances to refuse before a violating write. Where the protecting fact
is invisible to the client, a plan can construct without a client bug and the
helper is the operative layer.

### Option C — either layer suffices, unspecified which

Rejected: "either" means the weaker in practice, because the weaker is the
one that ships bugs silently; and an unspecified guarantee is the ambiguity
Section 0.2's precedence rules exist to prevent.

## Decision

Option B. When WP-010 increment 3 writes the plan type, a Section 2.1
non-goal node cannot appear as the target of a mutating step in a well-formed
plan — unrepresentable by construction — and the helper's independent
recomputation under HLP-002 remains as an unweakened second layer.

**SI-11 stays open.** This decides the axis the register filed and nothing
else; the remaining protection-construction, closure, residual, and status
obligations exposed across the three rejected rounds are round four's work, now
with a fixed foundation. Round one was rejected for a PART-014/MAC-009 status
conflict, round two for sibling capture and the separate SI-27 naming gap, and
round three for six recorded defects including closure and residual failures.
Recording SI-11 as resolved on the strength of this ADR would repeat the exact
overclaim this project's register discipline exists to prevent.

## Consequences

- Positive: once shared computation correctly marks a client-visible protecting
  fact, a mutating step for it is structurally unavailable. The retained helper
  supplies a second refusal opportunity for later client-only defects. Round
  four starts from a fixed axis instead of re-litigating it, and the plan type's
  shape gains a hard constraint before it is written, not after. Client-
  invisible protecting facts and shared-classification defects retain the
  limits stated in Safety analysis.
- Negative: unrepresentability must be *provable* in the implementation — a
  compile-fail or construction-refusal proof, in the style the CLI chassis
  already set with its non-`Hash` output type — which is more work than an
  `if`. That cost is the point.
- The closure correctness burden is unchanged and remains SI-11's.
- PART-014's enumerated list remains as-is; nothing is removed (MAC-009's
  Recovery-only semantics untouched — round one's error is not repeated).

## Verification

- When the plan type lands: a compile-fail (or equivalent
  construction-refusal) test proving a mutating step cannot be built against
  a non-goal node, in the pattern of the chassis's compile-fail non-`Hash`
  proof; plus tests that the helper independently refuses a hand-forged
  artifact that bypasses the type layer (defense in depth exercised, not
  assumed).
- Register: SI-11's entry gains an axis-decided state and **remains in the
  direct-blocker row**; any text implying SI-11 is resolved is an error
  against this ADR.

## Revisit conditions

- Section 2.1's non-goal list changes.
- Round four's closure design produces evidence that unrepresentability
  cannot be expressed for some legitimate plan shape (none is currently
  known).
- Any proposal to relax the retained runtime layer — this ADR's acceptance
  is conditional on both layers existing.
- Evidence that unprivileged discovery cannot observe a Section
  2.1-protecting fact on a supported platform **and** that the resulting
  verdicts diverge — one instance of the observability half is already
  measured (the single-signature udev view); divergence is the untested
  half, and confirming it would move the operative guarantee for that input
  class from the type layer to the helper.
