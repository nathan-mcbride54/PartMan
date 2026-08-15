# SI-39 recommendation round — 2026-08-08

**Status: a recommendation for Nate's decision, adversarially reviewed. It
decides nothing.** SI-39 stays Open until a decision is recorded through a
WP-010 spec change with an ADR, the shape ADR-0013/6.0.0 set. This is an
untracked session artifact under `docs/reviews/**` (WP-000); the register's
own text is not modified by this round.

The register entry is `docs/spec-issues/README.md` §SI-39, filed 2026-08-05
with four options, none recommended. The custody dependency it recorded was
discharged 2026-08-08 (independent readback, every digest matching), so the
measurement half no longer waits on anything.

---

## Recommendation: option (c), scoped as a qualification of the derived sentence

**Accept the consequence: a blank device is Strong only where a contract
positively determines absence.** Concretely:

1. **SAFE-003's strength rule does not change.** Strong still requires a
   stable hardware identifier, total size, both sector sizes, and a
   *positively determined* table state. The guarantee "Strong" makes to a
   consumer is identical on every platform. This is the load-bearing
   difference from option (a), which would weaken the guarantee itself.
2. **The derived sentence is qualified, because it is the only text that is
   false.** "A blank device can therefore be Strong" presupposes `Absent` is
   generally reachable. It is not: on macOS the measured client contract
   cannot separate `blank-512` from a LUKS2 container, so INV-003 (6.0.0)
   forbids the client reporting `Absent`, so no client-derived record for
   blank macOS media carries a positively determined state, so such records
   are Weak — **by the existing rule, with no amendment**. The sentence
   becomes conditional: a blank device can be Strong *where the observing
   contract positively determines absence* — true on contracts that read raw
   sectors (the helper everywhere; M10 measured root reading true bytes on
   macOS), unreachable for the macOS client today.
3. **The qualifier is contract-relative, not platform-named.** If a
   separating macOS client interface is ever measured (option (d)'s hope),
   records start qualifying again with no further spec change. Option (d)
   collapses into a standing revisit condition instead of a blocking hope
   with no named candidate.
4. **This is a major bump.** It narrows what an existing requirement's text
   claims. The spec's own §0 records that 3.1.0 was mis-numbered for exactly
   this class of change; do not repeat that.

## What a consumer and a plan may rely on (the register's required statement)

For a medium in the unseparated case, tested against `blank-512` and
`luks2-whole-disk-512`:

- **An inventory consumer** may rely on: the record carries no positively
  determined table state; strength is Weak; the platform's reach declaration
  carries a negative `Absent` cell citing the observability row. It may
  **not** rely on the medium being empty — that is the entire finding.
- **A plan** targeting such a medium (PART-001 initialization is a
  destructive whole-device operation) inherits SAFE-003's weak-identity
  policy in full: typed device-name confirmation (UI-009), an immediate
  pre-apply re-probe, and refusal of unattended apply without the recorded
  override. The pre-apply re-probe and HLP-002's privileged re-discovery
  **can** separate the pair — M10 established the helper reads the true
  bytes and that the four "blank-identical" client projections carry four
  distinct helper digests — and SAFE-005 plus SAFE-003's identity-change
  rejection then refuse the write when the target is not what the plan
  assumed. The safety load lands on the observer that can actually see.
- **Test shape, when a package may build it:** both fixtures produce client
  records classified Weak with byte-identical client projections; the
  helper-view digests differ. The first half is Tier-1 replay work; the
  second half is already an established observability row rather than a new
  test obligation.

## The adversarial round

**Attack 1 — "this is option (a) in disguise: strength just became
platform-relative."** Refuted by locating what varies. Under (a), the
*meaning* of Strong varies: a consumer reading Strong on macOS would get a
weaker guarantee than on Linux. Under (c), the meaning is invariant and only
the *attainable population* varies — exactly as it already does for devices
behind serial-less USB bridges, which SAFE-003's own text names as the
common Weak case. Nobody calls that platform-relative strength; this is the
same shape. The attack sharpened point 1 and is why the recommendation
forbids touching the strength rule's text.

**Attack 2 — "every fresh disk on macOS now needs typed confirmation and
an override for scripted use; users will rubber-stamp."** Sustained as a
real cost and accepted deliberately. The friction lands only on macOS
whole-device destructive operations against media the client cannot
distinguish from an encrypted container, an mdraid member, or a live file
system. The alternative is initializing what might be a LUKS2 container
with the *lighter* path. A rubber stamp is a UI-009 design risk that exists
for every weak-identity flow already; it does not distinguish (c) from the
status quo ante, which reached the same flows through USB bridges.

**Attack 3 — "the qualifier will be read as licensing `Indeterminate`
laundering: clients report Indeterminate everywhere and nothing is ever
Strong."** Refuted by INV-003's other face: a client MUST detect every
state its contract *can* distinguish. A Linux client whose contract
separates blank from occupied may not flatten it to Indeterminate; the
reach declaration (increment 7's surface) makes the contract's reach a
published, citable fact rather than an adapter's mood. The qualifier
changes nothing on platforms whose contracts separate the case.

**Attack 4 — "this prejudges SI-35's Present face or ADR-0014's axis."**
Refuted by scope. The filing deliberately left "is reporting `Present` on a
conflicted-table medium itself a forbidden report" to SI-35; this
recommendation touches only the `Absent` face and says so. It also keeps
ADR-C3's three-valued vocabulary and ADR-C4's body-value guard intact —
the *representational* distinction between Absent and Indeterminate is
untouched; what changes is which values one platform's client can emit.
ADR-0014's fork (whether table state may leave the hashed body) is neither
eased nor foreclosed.

**Attack 5 — "PART-001 still says 'initialize blank media' — how does a
plan ever name blank media on macOS?"** The sharpest attack, and it forced
a refinement rather than a retreat: on macOS the plan's *claim* cannot be
"this is blank" (the client cannot know it); the plan's claim is "initialize
this device, which the client could not distinguish from occupied," and the
weak-identity path plus the helper's separating re-probe carry it. The
resolution text should say this in terms, or PART-001 will be read as
unimplementable on macOS rather than as routed through the weak path.

## Rejected, and why — to be recorded with the decision

- **(a) Strength relative to published reach.** Weakens the one guarantee
  the whole identity design hangs on; ADR-C3 chose the absolute notion
  deliberately and SI-02's resolution rests on it. Rejected on the Attack-1
  distinction.
- **(b) Reportable `Absent` under caveat.** The recorded data-loss path:
  a macOS client would report `Absent` for a disk holding a LUKS2
  container, and PART-001 initializes blank media. ADR-0013 was written to
  end exactly this report; the spec's ADR-C4 note says conflation is what
  "PART-001 would then initialize alike." Rejected without needing the
  adversarial round.
- **(d) Find a separating client interface.** Not a resolution — a hope
  with no named candidate, after the matrix measured the interfaces the
  contract reads and M10 located the separating fact behind a privileged
  read. Retained as a revisit condition that the contract-relative wording
  makes self-executing.
- **(e) — considered here, not in the register — split strength into
  client-strength and helper-strength.** Reintroduces the
  comparison-outcome confusion 3.1.0 removed and doubles the vocabulary
  every consumer must handle, for no safety the weak-identity path does not
  already provide. Rejected.

## If accepted, the mechanics

WP-010 files the ADR (ADR-0014 is reserved for SI-35's axis; this would be
its own number), amends SAFE-003's sentence, bumps major, moves SI-39 to
Resolved, and the register's direct-blocker count drops to nine — with the
SI-35 overlap sentence left exactly as filed.
