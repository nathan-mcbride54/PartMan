# ADR-0020: An immutable technology limit is `unsupported`, carried as an explicit reason

- Status: Accepted
- Date: 2026-08-10. Decided by Nate McBride as reading (a) of SI-40's
  filed options, directed the same day the filing landed.
- Spec version: none — deliberately. This decision selects between two
  readings of existing normative text and amends neither; the filing
  recorded, as classification, that reading (a) changes no normative
  sentence, and this ADR's consequences confirm it. If a revisit ever
  requires an amendment, that change arrives under its own grant.
- Work packages blocked: WP-050 increment 2's technology-limit
  composition (unblocked by this decision; no other increment or
  package waited on it)
- Requirement IDs: FS-007, CAP-003, CAP-005
- Decision owners: Nate McBride

## Context

SI-40 filed the conflict from WP-050 increment 1's vocabulary work.
FS-007: "Surface immutable technical limits, such as XFS not shrinking,
as explicit blocked reasons." CAP-003's definitions: "`blocked` —
implemented, but a runtime precondition fails (missing tool, version,
state)"; "`unsupported` — the product does not implement the operation
for this target." An immutable technology limit is not an implemented
operation with a failing runtime precondition, so the literal `blocked`
status contradicts `blocked`'s own definition, while `unsupported`
contradicts FS-007's word "blocked" read as a status name. One case, two
statuses, both texts normative, and the answer product-visible on every
surface at once because CAP-005 serves them all from one engine.
Increment 1 deliberately left the `TechnologyLimit` reason's status
coupling unasserted rather than decide this in a constructor.

## Safety analysis

Neither reading touches device identity, privilege, validation,
journaling, recovery, secrets, or hostile-input handling: the choice is
which refusal status a permanently impossible operation reports. Both
readings refuse the operation; no MUST weakens under either. The safety
property that matters — a user is never invited to attempt what cannot
succeed, and the refusal states why — holds identically in both. The
reading chosen keeps `blocked` meaning remediable, which preserves the
signal users and the planner act on: a `blocked` answer is worth
retrying after remediation, an `unsupported` answer is not, and an
immutable limit that reported `blocked` would invite remediation of the
unremediable.

## Options considered

### Reading (a) — the vocabulary noun phrase

"Blocked reasons" in FS-007 is the generic noun phrase for the
capability reason vocabulary — the reading this repository's prose
already used ("the blocked-reason capability surface is WP-050's") —
and the status follows CAP-003's definitions: `unsupported`, carrying
the limit as its explicit reason ([`Reason::TechnologyLimit`]) and a
remediation stating no remedy exists. Amends no normative text. Cost:
FS-007's word "blocked" reads as vocabulary, not status, and this ADR
is the record a future reader needs to read it that way.

### Reading (b) — the literal status

FS-007 mandates the literal `blocked` status, and CAP-003's definition
list is amended so `blocked` admits immutable limits. Cost: a normative
retext of CAP-003 (a spec change), and `blocked` stops meaning
remediable — every consumer that treats `blocked` as retry-after-remedy
inherits a permanent member of a temporary class.

### Reading (c) — a distinct shape

A new status or maturity axis. Cost: CAP-003's closed four-status
vocabulary widens, a larger change than the conflict warrants, with
SI-26 already holding the only open status-vocabulary question.

## Decision

Reading (a). An immutable technology limit reports CAP-003
`unsupported`, with the limit carried as its explicit reason and a
remediation stating that no remedy exists. FS-007 is satisfied by the
explicitness — the limit is named in the reason vocabulary, never
buried in a generic refusal — and CAP-003's definitions are satisfied
by the status: the product does not implement shrink for XFS because
XFS does not implement shrink, and no runtime precondition will ever
change that.

## Consequences

- WP-050 increment 2 composes technology limits as `unsupported` with
  `Reason::TechnologyLimit` and `Remediation::NoneExists`; its other
  arms were never gated on this decision.
- `blocked` keeps its definition: every `blocked` answer remains
  remediable in principle, which CAP-003's remediation field exists to
  state.
- No normative sentence changes; the specification version does not
  move. SI-40 resolves on the register with this ADR as its record.
- Any surface rendering capability answers may rely on the invariant:
  `unsupported` + `TechnologyLimit` is permanent for the technology,
  and a remediation of `NoneExists` is exact, not lazy.

## Verification

WP-050 increment 2's tests hold the coupling: a technology-limit hit
yields `unsupported` with `Reason::TechnologyLimit` and
`Remediation::NoneExists`, and the CAP-005 agreement enumeration keeps
the protection arms unaffected by limit composition. The increment-1
vocabulary tests already pin the reason's existence and the closed
enum's version discipline.

## Revisit conditions

A technology limit that stops being immutable (a file system gaining a
formerly impossible operation) is not a revisit of this ADR — it is the
limit fact's removal, after which the engine's other arms answer. This
ADR is revisited only if CAP-003's status vocabulary itself is amended
(SI-26's question), at which point the immutable-limit case must be
re-homed deliberately rather than inherited.
