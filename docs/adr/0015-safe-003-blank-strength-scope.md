# ADR-0015: A blank device is Strong where a contract positively determines absence

- Status: **Proposed — awaiting acceptance by the decision owner. Nothing
  below is in force, no spec version changes, and SI-39 remains Open until
  this line is replaced by an acceptance record.**
- Date: 2026-08-08 (drafted; the recommendation round of the same date is
  the input, `docs/reviews/SI-39_RECOMMENDATION_ROUND_2026-08-08.md`)
- Spec version: 7.0.0 (on acceptance; major under §0.1, and deliberately so
  — see Decision)
- Work packages blocked: WP-010 increment 3 (SI-39 resolved on acceptance;
  SI-11, SI-27, SI-28, SI-33, SI-34, SI-35 unchanged)
- Requirement IDs: SAFE-003, INV-003, PART-001, HLP-002, SAFE-005, UI-009,
  ADR-C3, ADR-C4, Section 0.2
- Decision owners: Nate McBride

## Context

SAFE-003's identity-record clause states, of partition-table state: "Only
the first two are positively determined. **A blank device can therefore be
Strong**; a device whose table failed to parse cannot." INV-003, as spec
6.0.0 amended it, forbids the unprivileged layer reporting "a medium as
positively without a table" where its platform contract does not separate
that case.

The macOS increment 6 matrix (2026-08-05, valid on its second sitting;
second-reader readback discharged 2026-08-08) measured that non-separation
directly: `blank-512` and media carrying a live ext4 with a stale mdraid
superblock, an mdraid member, a LUKS2 container, and an LVM2 orphan all
produce **byte-identical** unprivileged projections. M10, the privileged
leg of the same date, located the separating facts behind a read the
client is denied: the four "blank-identical" media each carry a distinct
helper digest.

So on macOS: the client may not report `Absent`; the table state in a
client-derived record is therefore never positively determined for blank
media; by SAFE-003's own strength rule such records are Weak — and
SAFE-003 says in terms that a blank device can be Strong. Both texts are
requirements, and they cannot both hold on that platform. SI-39 filed the
conflict under Section 0.2, recording that this repository created it:
INV-003's governing sentence is ADR-0013's, whose adversarial round did
not reach SAFE-003.

## Safety analysis

**The strength rule itself is not in conflict, and this decision does not
touch it.** Strong requires a stable hardware identifier, total size, both
sector sizes, and a positively determined table state — invariantly, on
every platform. What is false is only the derived sentence's universal
presupposition that `Absent` is generally reachable. The conflict lives in
one sentence, so the amendment is scoped to that sentence.

**The qualifier is contract-relative, not platform-named.** A blank device
can be Strong *where the observing contract positively determines
absence*: the helper's raw read everywhere (M10 measured root reading true
bytes on macOS), and any client contract whose published INV-003 reach
separates the absent case. Naming macOS in the rule would freeze a
measurement into normative text; naming the contract lets a future
measured separating interface (SI-39's option (d), retained below as a
revisit condition) restore client-side Strong records with no further
amendment.

**The safety load lands on the observer that can see.** A plan targeting
a medium the client cannot distinguish from occupied inherits SAFE-003's
weak-identity policy in full: typed device-name confirmation (UI-009) for
destructive whole-device operations, an immediate pre-apply re-probe, and
refusal of unattended apply without the recorded override. The pre-apply
re-probe and HLP-002's privileged re-discovery **can** separate the
decisive pair — that is M10's finding — and SAFE-003's identity-change
rejection plus SAFE-005 then refuse the write when the target is not what
the plan assumed.

**PART-001 remains implementable, routed rather than blocked.** On a
platform whose client cannot reach `Absent`, a plan's claim is never "this
medium is blank" — the client cannot know it. The claim is "initialize
this device, which the client could not distinguish from occupied," and
the weak-identity path plus the helper's separating re-probe carry it.
This sentence is in the amendment's text because without it PART-001
reads as unimplementable on macOS, which the adversarial round found to
be the sharpest misreading available.

**What a consumer may and may not rely on, in the unseparated case** —
the statement SI-39's evidence clause requires, tested against
`blank-512` and `luks2-whole-disk-512`:

- An inventory consumer may rely on: the record carries no positively
  determined table state; strength is Weak; the platform's reach
  declaration carries a negative `Absent` cell citing its observability
  row. It may **not** rely on the medium being empty.
- A plan may rely on: nothing beyond the weak-identity policy, whose
  pre-apply re-probe is the separating observation.

**The representational guard is untouched.** ADR-C3's three-valued
vocabulary and ADR-C4's body-value distinction between positively absent
and unreadable stand exactly as written; what changes is which values one
platform's *client* can emit, not what the record can represent. This
decision neither eases nor forecloses ADR-0014's open fork.

**The `Present` face is deliberately not decided.** INV-003's same
sentence also forbids reporting a table as consistent where the contract
does not separate that case, and whether reporting `Present` on a
conflicted-table medium is such a report overlaps SI-35's open axis
question on all three platforms. SI-39's filing left it to SI-35; this
ADR does the same, and any text implying otherwise is an error against
both.

## Options considered

### Option (a) — make Strong relative to the published INV-003 reach

Rejected: it weakens the guarantee itself. Under (a), a consumer reading
Strong on macOS would receive a weaker promise than on Linux — the exact
relativization ADR-C3 chose the single-record absolute notion to prevent,
and SI-02's resolution rests on that notion. Under the accepted option,
the *meaning* of Strong is invariant and only the attainable population
varies — as it already does for serial-less USB bridges, which SAFE-003's
own text names as the common Weak case. That distinction is the decision's
load-bearing wall, and it is why the strength rule's text is untouched.

### Option (b) — amend INV-003 so an unseparated medium is reportable as `Absent` under a caveat

Rejected without needing the adversarial round: it is the recorded
data-loss path. A macOS client would report `Absent` for a disk holding a
LUKS2 container, and PART-001 initializes blank media. ADR-0013 was
written to end exactly this report, and the spec's ADR-C4 note names the
consequence — conflation is what "PART-001 would then initialize alike."

### Option (c) — accept the consequence, qualifying only the derived sentence (accepted)

Accepted, scoped as above: the strength rule unchanged, the false
universal sentence made contract-relative, the weak-identity policy
carrying the load on the platform whose client is blind.

### Option (d) — establish a separating client-readable macOS interface

Not a resolution: no candidate is named, the matrix measured the
interfaces the contract reads, and M10 located the separating fact behind
a privileged read. Retained as a revisit condition that the
contract-relative wording makes self-executing — a measured separating
contract restores client-side Strong records without another amendment.

### Option (e) — split strength into client-strength and helper-strength

Considered in the recommendation round though not in the register's
filing; rejected. It reintroduces the comparison-outcome confusion 3.1.0
removed, doubles the vocabulary every consumer must handle, and buys no
safety the weak-identity path does not already provide.

## Decision

Option (c), landed as spec 7.0.0's amendment to SAFE-003's derived
sentence. **SI-39 moves to Resolved on acceptance.**

**Major under §0.1, stated before anyone asks:** the amendment narrows
what an existing requirement's text claims, which is a semantic change to
an existing requirement. The spec's own §0 records that 3.1.0 was
mis-numbered for exactly this class of change and left as issued; this
decision does not repeat that.

## Consequences

- **Positive.** SAFE-003 stops asserting something measurably false on a
  supported platform, and the false sentence is repaired at its exact
  scope rather than by rebuilding the strength notion around it.
- **Negative, accepted knowingly.** On macOS, ordinary blank media carry
  Weak identity at plan time: whole-device destructive operations —
  PART-001 initialization included — require typed device-name
  confirmation, and unattended or scripted apply requires the recorded
  weak-identity override. The friction lands precisely where the client
  cannot distinguish blank from an encrypted container, and the
  alternative was initializing possibly-occupied media on the lighter
  path. The rubber-stamp risk of routine typed confirmation is a UI-009
  design concern that already exists for every weak-identity flow.
- **Negative.** ADR-C3's recorded consequence that "a strong-identity
  blank removable now qualifies for SAFE-003's replug path-change
  allowance" narrows: on macOS no client-derived blank record is Strong,
  so no such record reaches that allowance. The helper-derived record's
  eligibility is unchanged.
- Strength stays one notion. No consumer-facing schema, hash rule, or
  ADR-C3 state changes; nothing here is hash-visible beyond what SI-39's
  eventual increment-3 types would have carried anyway.

## Verification

- A Tier-1 fixture-backed test that client-derived identity records for
  `blank-512` and `luks2-whole-disk-512` are classified Weak, with
  byte-identical client projections — the two-fixture statement SI-39's
  evidence clause names. (The helper-side half — that the pair separates
  for a privileged reader — is the established M10 observability row, not
  a new test obligation.)
- A test that the weak-identity policy engages for a whole-device
  destructive plan against such a record: typed-confirmation demanded,
  unattended apply refused absent the recorded override. Owned by the
  package that first implements plan binding; recorded here so it is not
  discovered late.
- Register: SI-39's entry reads Resolved; the direct-blocker count drops
  by one; SI-35's entry is untouched, including its `Present`-face scope.

## Revisit conditions

- A client-readable macOS interface is named and measured, under the
  custody rules the existing protocols use, to separate a blank medium
  from an occupied one. The contract-relative wording then restores
  client-side Strong records without amendment; this ADR's record of
  option (d) is superseded to that extent.
- SAFE-003's strength rule itself is amended by any later decision. This
  ADR's premise — that the rule and the derived sentence are separable —
  would need re-examining.
- SI-35's axis decision (ADR-0014's fork) relocates partition-table state
  out of the hashed body. This decision reads ADR-C3/ADR-C4 as they stand.
