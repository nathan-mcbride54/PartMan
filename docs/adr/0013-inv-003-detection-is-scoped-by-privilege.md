# ADR-0013: INV-003's detection duty is scoped by privilege

- Status: Accepted
- Date: 2026-08-05
- Spec version: 6.0.0
- Work packages blocked: WP-010 increment 3 (SI-38 resolved; SI-35 unblocked
  and still open)
- Requirement IDs: INV-003, SAFE-002, SAFE-005, HLP-001, HLP-002, HLP-005,
  ADR-C3, ADR-C4, Section 0.2
- Decision owners: Nate McBride

## Context

INV-003 requires detecting GPT, MBR, Apple Partition Map, missing tables,
hybrid/inconsistent tables, and corrupt metadata. It sits in Section 7.1
beside INV-001, INV-002 and INV-004 — the discovery layer's duties. SAFE-002
places that layer at no elevation.

Three decision-complete measurements establish that the unprivileged layer
cannot separate a healthy GPT from one whose two tables describe different
partitions: Linux 2026-08-03 through a descriptor-bound loop device with
partitions materialized; Windows 2026-08-04 under total-retention and
mandatory-layout-probe gates; macOS 2026-08-05, byte-identical `diskutil`
output. On Windows, W-Q4 additionally found that nothing flags hybrid
aliasing. All three records are in `docs/quality/observability.md`.

The privileged leg (M10, 2026-08-05) supplies the mechanism. The two GPT
fixtures have identical first-64-KiB digests and different last-64-KiB
digests: the disagreement lives in the backup table, and the read that reaches
it was denied to the unprivileged client on the same attachment, in the same
sitting.

So the conflict is not that the fact is unobservable. It is that observing it
requires a privilege SAFE-002 withholds from the layer INV-003 addresses.
Section 0.2 requires such a conflict be filed rather than silently resolved,
and it was, as SI-38.

**There is no privileged inventory path, and the reading matters enough to
state.** HLP-001 permits the helper "status/enumeration queries", which could
be read as device enumeration and would then supply one. It does not, on three
grounds. Its neighbours in that list — validate-plan, apply-plan, cancel,
resume, journal queries — are all plan and job operations. HLP-005 permits the
helper to idle locked-down and **exit when idle**, and a component the spec
allows to be absent cannot be a required participant in inventory, which
happens precisely when there is no work. And SAFE-002 confines privileged
behavior to exactly two contexts while forbidding any component to
auto-elevate, so an inventory that starts the helper is auto-elevation under
another name.

The opposite reading would not have dissolved the conflict. It would have
produced a second one, HLP-001 against SAFE-002's two-contexts rule.

## Safety analysis

**The naive fix is unimplementable, and that shaped the decision.** Requiring
the discovery layer to "report undetermined where it cannot detect" assumes it
can identify those cases. It cannot: a conflicting-tables medium presents as
an ordinary valid GPT with no anomaly signal of any kind. That rule would
either never fire, or would mark every GPT medium undetermined — and the
second is not a usable partition manager.

**What replaces it is per-contract, not per-device.** The reach declaration is
static platform knowledge, already measured for all three supported platforms.
It cannot be vacuous, because it does not depend on detecting the case it
describes.

**The safety property is preserved by the fourth clause, not the first.**
Scoping detection by privilege, alone, would leave the unprivileged inventory
silently incomplete — the hazard ADR-C4 refused when it declined to collapse
`Absent` and `Indeterminate`, since PART-001 initializes blank media.
Forbidding a consumer from reading an unprivileged inventory as evidence of
absence is what keeps that closed.

**And the fourth clause routes the determination rather than refusing.** An
earlier phrasing had SAFE-005 govern "any write that depends on" an undeclared
state. Read broadly that refuses every write to a GPT medium, because every
such write depends on the table being what it appears to be — which would make
this decision the naive proposal in disguise. The clause therefore routes the
determination to the privileged re-discovery HLP-002 already requires before
the first write, and forbids the unprivileged layer both from refusing on the
ground of its own blindness and from representing that blindness as a
determination. Refusing a genuinely conflicting medium is what ADR-C3's
`Indeterminate` exists to cause; refusing every GPT medium because the client
cannot tell is not, and this decision must not be readable as requiring it.

**SAFE-002 is untouched, deliberately.** It is a Section 3 constraint with
precedence over everything, and qualifying it to satisfy a Section 7
functional requirement would invert Section 0.2's ordering. It would also mean
producing an inventory requires elevation — a product change far larger than
the defect.

**What this does not fix.** It does not decide who computes ADR-C3's table
states or from what contract; that is SI-35, now unblocked and still open. It
does not supply the reach declaration's encoding, which is an implementation
obligation on the discovery layer. And it makes no claim that no
client-readable interface could separate these states: three complete
projections failed to, which is a negative over what was enumerated, not a
proof about what was not.

## Options considered

### Option A — scope INV-003 by privilege, and no more

Rejected as insufficient rather than wrong. It describes reality but leaves
the unprivileged inventory silently incomplete on a safety-relevant case, with
nothing requiring the client to say so.

### Option B — add a fail-closed clause: report the remainder as undetermined

**Rejected as unimplementable**, per Safety analysis. The client cannot
identify the remainder. Recorded because it is the obvious proposal, it was
this decision's own first recommendation, and its defect is not obvious — the
same defect had already been found one layer down and was reintroduced one
layer up before review caught it.

### Option C — qualify SAFE-002 to permit a privileged discovery leg

Rejected on precedence. SAFE-002 is Section 3; INV-003 is Section 7; Section
0.2 grants Section 3 override authority over everything. Bending the
constraint to satisfy the functional requirement inverts that. Recorded
because omitting the option the precedence rules argue hardest against is how
a register starts misrepresenting a decision's shape.

### Option D — establish a client-readable interface that separates the states

Rejected as **unsupported, not impossible**. No candidate has been named,
three complete projections failed to supply one, and M10 gives a mechanical
account of why. If such an interface is ever named and measured on every
supported platform, that is a revisit condition below.

### Option E — scope by privilege plus a per-contract reach declaration (accepted)

Accepted. It is honest about what each layer can do; it is implementable,
because the reach is static platform knowledge rather than per-device
detection; and it keeps the ADR-C4 hazard closed through a consumer obligation
rather than through a detection the client cannot perform.

## Decision

Option E, landed as spec 6.0.0's amendment to INV-003. **SI-38 moves to
Resolved.**

**SI-35 is unblocked and remains open.** This decides which layer owes which
detection, not who computes ADR-C3's states or from what contract. Recording
SI-35 as touched by this ADR would be the overclaim the register discipline
exists to prevent.

An ADR may record a decision whose implementation weakens a MUST, with the
spec amendment as the instrument — decided separately by Nate McBride on
2026-08-05, because neither ADR-0011 nor ADR-0012 tested it, both being purely
additive. Section 0.2 item 4 continues to forbid an ADR *being* that
instrument.

## Consequences

- **Positive.** The spec stops requiring something measurably impossible and
  says so in a form a consumer can act on. SI-35 proceeds from a settled
  premise instead of picking a side of this conflict implicitly — which a
  draft SI-35 axis ADR did, before review caught it.
- **Negative.** This narrows a MUST. The unprivileged inventory is explicitly
  incomplete on a safety-relevant case, and every consumer must learn to read
  the reach declaration. A consumer that ignores it is in exactly the hazard
  the fourth clause forbids, and nothing but review enforces that.
- **Negative — inventory display fidelity drops, and this is the real cost.**
  A user browsing an inventory learns "this platform cannot tell", not "this
  disk is fine". Full detection still happens before any write, so nothing
  unsafe follows; but the moment a user most wants a per-device answer is
  exactly the moment this decision stops promising one. Accepted knowingly
  rather than argued away.
- **Negative.** The reach declaration is a new implementation obligation on
  the discovery layer, and its encoding is undecided.
- Section 7's other inventory requirements are untouched. INV-004 in
  particular is not amended: the macOS finding that foreign signatures project
  identically to a blank disk concerns technologies that are not
  platform-applicable there, which the increment's own scoping anticipated.

## Verification

- The reach declaration exists for every supported platform, is derived from
  the platform contract rather than from any device, and is present with a
  negative answer where the contract does not separate a state.
- A fixture-backed test that an unprivileged inventory of
  `gpt-conflicting-tables-512` does not report the table as consistent.
- A test that an unprivileged layer does not refuse a write solely because its
  contract cannot reach a state — the regression guard for the rejected
  broad reading of the fourth clause.
- Register: SI-38's entry reads Resolved; **SI-35 remains a direct blocker**,
  and any text implying otherwise is an error against this ADR.

## Revisit conditions

- A client-readable interface is named and measured that separates the states
  on every supported platform.
- HLP-001's "enumeration queries" is amended to include device enumeration, or
  HLP-005's permission for the helper to exit when idle is withdrawn. Either
  would create a privileged inventory path, and this decision's premise would
  need re-examining. The Context above reads both as they currently stand, not
  as they might be rewritten.
- SAFE-002's permitted contexts change.
