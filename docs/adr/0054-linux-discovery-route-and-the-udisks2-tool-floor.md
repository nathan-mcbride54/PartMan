# ADR-0054: LIN-001's discovery route, and the UDisks2 floor moves to the tool it gates

- Status: Accepted
- Date: 2026-08-19. Made on the adversarially reviewed recommendation
  round of the same day
  (`docs/reviews/LINUX_UDISKS2_ROUTE_ROUND_2026-08-19.md`, a committed
  session record; option B taken by the decision owner, with the
  tool-floor entry deferred to first invocation), under the route
  decision WP-L100's assignment claimed LIN-001 increment-gated behind
  (`docs/reviews/WP-L100_ASSIGNMENT_PLAN_2026-08-12.md` §4) and the
  increment-5 plan deferred again (§7.3). Recorded before its first
  consumer is written — merging is not acceptance.
- Spec version: 18.0.0 (major under §0.1 — LIN-001's sentence changes
  meaning, and the Section 9 Debian/Ubuntu floor row loses a conjunct;
  one bump covers both)
- Work packages blocked: none (the first consumer is WP-L100's floor
  determination, which drops the conjunct on this ADR; WP-L110's
  authorization/mutation route stays its own decision)
- Requirement IDs: LIN-001, Section 9, CAP-004, CAP-006, SAFE-002,
  SAFE-004, HLP-002, LIN-009, MODEL-004, ADR-0006, ADR-0013
- Decision owners: Nate McBride

## Context

LIN-001 read, since 2.0.0: "Use UDisks2 for discovery/authorization and
libblockdev or authoritative native tools for mutations." Section 9's
Debian/Ubuntu floor row read "Debian 12 / Ubuntu 22.04 LTS; kernel ≥
5.15; UDisks2 ≥ 2.9", and Section 9 closes: "Per-tool version floors
live with the capability fixtures (CAP-006) in `docs/capabilities/`, not
in this spec."

WP-L100 delivered the Linux read-only inventory adapter (increments 1
through 5b, 2026-08-13 to 2026-08-19) as a pure library over an injected
read seam with an empty dependency closure, reading four client-readable
interfaces — sysfs, the udev database, procfs, and `os-release` — each
entered by a measured row in `docs/quality/observability.md` (DR1–DR19,
on Ubuntu 22.04, Debian 12 and Arch guests), opening no device node and
launching no process (a structural guard holds it). Its assignment
**claimed LIN-001 increment-gated** rather than let that route become
LIN-001 by drift: "adopting UDisks2 buys LIN-001's named interface at the
price of a D-Bus client dependency and an IPC surface, and that trade is
a recorded choice, never drift."

The floor determination (increment 5b, obligation 4) then measured and
determines the row's first two conjuncts on every tier and has **no
source for the third**: no file under the four interfaces carries the
daemon's version; it is reachable only over D-Bus (`Manager.Version`,
which activates the daemon if installed) or by launching `udisksctl`. So
the adapter reports the UDisks2 conjunct `Undetermined` by construction
(WP-050 increment 5's arm), and **the engine blocks every operation on
every Debian/Ubuntu host, naming UDisks2** — including every host the
product has ever been measured on. Measured, not argued: UDisks2 is not
installed by default on the Debian 12 genericcloud image (DR19) nor the
Arch cloud image (DR18); the jammy image ships it and every WP-020
acceptance guest purges it as a recorded deviation (`WP-020.md:915-918`);
no guest in the record has run the daemon, and the read-only product has
passed fifty-four acceptances without it. No crate depends on a D-Bus
client; `docs/capabilities/tool-version-floors.json` is empty because no
storage tool is invoked yet, and its format says a tool's floor "arrives
with the first package that invokes it, under review, with its basis
stated".

The round (§2) costed taking LIN-001 literally — a D-Bus client as the
client's discovery interface: zero rows for that interface against
nineteen for the files; a provoked `udisksd` launch that SAFE-004's
identity and allow-list discipline cannot see; second-hand MODEL-004
provenance over the udev database the adapter reads first-hand today;
and the read-only product `blocked` on every default Debian 12 and Arch
install by a floor written for an inventory that opens nothing. It
costed reading the conjunct from the package manager's record: a file
read that would author "met" about a daemon never consulted. And it
asked whether deciding for the measured route is the drift the
assignment warned of — it is not, because it is a decision, priced as a
spec change, reasoned on the rows and recommended as it would be if the
code did not exist.

## The decision

1. **LIN-001's discovery half is decided for the measured route.** The
   Linux discovery layer reads the kernel's and the distribution's
   client-readable interfaces — sysfs, the udev database, procfs, and
   `os-release` — each interface entered only by a measured row, as
   WP-L100's contract states and ADR-0013 publishes. UDisks2 is not the
   client's discovery interface. LIN-001 is revised accordingly (18.0.0).
2. **LIN-001's authorization/mutation half stays the helper's own route
   decision.** UDisks2, libblockdev, or authoritative native tools for
   authorization and mutations are WP-L110's recorded choice, where
   HLP-002's independent re-discovery and LIN-009's plan-scoped polkit
   rules live. Nothing here forecloses UDisks2 there; ADR-0006's "IPC is
   not linking" stands.
3. **The UDisks2 ≥ 2.9 floor is a tool floor, not a platform
   conjunct.** Section 9's Debian/Ubuntu row becomes "Debian 12 / Ubuntu
   22.04 LTS; kernel ≥ 5.15". The number moves to the CAP-006 store
   (`docs/capabilities/tool-version-floors.json`), **entered by the first
   package that invokes UDisks2, with this ADR as its basis** — not
   before, per that store's own rule — and gates, through CAP-004's
   delivered `ToolFloor`/`tool_state` machinery (fail-closed: missing →
   blocked), exactly the operations that use the tool. The engine may
   still narrow at runtime and may still never widen below the row; what
   changed is the row, by ADR, as Section 9 permits.

## Options considered

### UDisks2 as the client's discovery interface (option A)

Rejected on the rows: an interface with no rows replacing one with
nineteen; a provoked process launch outside SAFE-004's sight; second-hand
provenance; and a read-only matrix blocked on two tiers' default
installs. Not rejected on licensing (ADR-0006) or on code size — a D-Bus
client is small; the cost is the rows and the floor consequence. What
would reopen it is in the round's §6.

### The package manager's record as the conjunct's source (option C)

Rejected as a decision: `/var/lib/dpkg/status` is world-readable
(measured on two Debian 13 hosts for the round) and `pacman`'s local
database is its analogue, but each determines the installed *package*,
not the daemon's availability, and reporting "met" from it would be the
widening-by-assertion the floor rules forbid. Kept as the natural *probe*
for the tool floor when a mutating package owns the invocation.

### Qualifying the platform row in place ("where used") (option D)

Rejected: it is this decision with the number left where Section 9 says
tool floors do not live, and `PlatformFact` is per-platform, not
per-operation.

### Leaving the conjunct undetermined (option E)

Rejected: a floor no host meets by the product's own reading is not a
floor.

## Consequences

- **Positive:** the floor is finishable — two measured conjuncts on the
  Debian/Ubuntu row, `MeetsFloor` on every measured guest; the UDisks2
  number is kept and attached to what it gates; LIN-001 says what the
  record established instead of what no row measured; WP-L110's route
  stays open in both directions.
- **Negative, accepted knowingly:** a major spec bump for two sentences;
  until a package invokes UDisks2 the ≥ 2.9 floor lives in this ADR and
  in no fixture — it gates nothing because nothing uses it, which is the
  store's own rule; and a deployment that relies on UDisks2 for desktop
  integration gets no signal from the read-only product about its
  absence — correctly, since the product does not use it.
- **Evidence obligations:** (1) the first UDisks2-invoking package enters
  the tool floor with a probe and a row (the package record or the
  launched `udisksctl --version`, its choice, measured); (2) WP-L100's
  `Undetermined` arm keeps its remaining shapes (unparsable, unreadable,
  unlisted `ID`), each still held by a test.

## Verification

- When WP-L100's floor lands on this ADR: the Debian/Ubuntu conjunction
  is distribution and kernel; DR16/DR17's and DR19's bytes compose to
  `MeetsFloor`; a shortfall in either is `BelowFloor`; the unmeasured
  shapes are `Undetermined`; no conjunct, field or remediation text
  names UDisks2 on a measured host; a mutation re-adding a UDisks2
  conjunct as met, or composing met over an undetermined kernel, is
  killed.
- `docs/capabilities/tool-version-floors.json` stays empty until a
  package invokes UDisks2; when it does, the entry's basis names this
  ADR.

## Revisit conditions

- A row showing UDisks2 present by default and client-readable on every
  tier — option A becomes a provenance-and-dependency question only.
- WP-L110 chooses UDisks2 for authorization/mutation — the tool floor
  enters then, and LIN-001's second half is decided there, not here.
- A product surface that needs UDisks2-mediated mounts for INV-006's
  "never auto-mount" discipline — the client route is re-examined on
  that surface's own round.
