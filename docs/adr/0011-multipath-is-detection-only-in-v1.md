# ADR-0011: Multipath is detection-only in v1

- Status: Accepted
- Date: 2026-08-02
- Spec version: 4.3.0
- Work packages blocked: WP-010 increment 3 (via SI-27), WP-050, WP-L100
- Requirement IDs: SAFE-003, SAFE-005, LIN-006, INV-001, CAP-001, CAP-003,
  MODEL-003, MODEL-005, PLAN-006, Section 2.1
- Decision owners: Nate McBride

## Context

Enterprise storage can reach one physical device through several concurrent
paths. The operating system then exposes multiple block devices that are one
device, and — where device-mapper multipath is configured — a combined `dm`
node on top of them. Three requirements meet this class: SAFE-003 models **one
connection path per identity record** and treats a path change as the special
case for removable replug; LIN-006 requires **detecting** device mapper and
multipath; INV-001 requires **discovering** hardware RAID LUNs, which
legitimately present several paths.

SI-12 filed the conflict: whether a bound record holds one canonical path, an
ordered set, or an unordered set is unstated, and a differing path count or
order would make an unchanged device compare unequal at re-probe — the
PLAN-006 failure. Round three reclassified it as a blocker on SI-27, because a
node-naming scheme that mints two nodes for one multipath device is wrong the
first time it meets a SAN: a multipath pair is one device seen twice, the
opposite of two ambiguous devices, and needs the opposite treatment.

The decision cannot be left implicit because the path representation is
**hash-visible if it lands in body position**: round three's reclassification
note directed "the path set in the envelope" — envelope content is not
hashed — while Section 6's identity-record placement would put it in the
body, and MODEL-005's body-stability rule (4.0.0) argues an
enumeration-derived path set may not qualify as body content at all. So the
body-versus-envelope placement is itself part of what SI-12 left undecided,
which strengthens the case for deferral rather than weakening it: choosing
an encoding now would mean choosing its placement too, from specification
text alone, and the register's standing rule is that guessing a
hash-visible encoding is the one option with no cheap exit.

No measurement of any multipath system exists in
`docs/quality/observability.md`. Every observability row to date is
single-path hardware or virtual disks. An encoding chosen today would be
chosen from specification text alone — the exact failure mode that rejected
three design rounds.

## Safety analysis

- **Device identity.** No multipath device is ever a bound target in v1, so no
  path field enters any hashed body, and the PLAN-006 instability SI-12
  describes cannot occur. This is consistent with MODEL-005's body-stability
  rule (4.0.0): a hashed body may carry only facts invariant under re-probe of
  unchanged hardware, and a path set derived from enumeration is not yet
  proven to be such a fact.
- **The undetected-multipath residual, stated at its real width.** The
  refusal is only as strong as the detection, and the unassembled case is
  wider than a first reading suggests. When no multipath node is assembled,
  LIN-006's device-mapper detection **contributes nothing** — there is
  nothing for it to detect — so the entire unassembled population rests on
  the second mitigation: **equal stable identifiers across two block devices
  with no assembled multipath node are treated as SAFE-005 ambiguity** — the
  round-three Regime A′ mapping, reporting `blocked` — which fails closed
  without claiming the two are one device. That check fires only when both
  paths are enumerated simultaneously **and** present bytewise-equal
  identifiers from the same layer, and the repository's own measurements
  show that premise cannot be assumed: a USB bridge synthesizes a
  storage-layer serial different from its descriptor serial, one device
  offers different identifier strings from different layers, and serial/WWN
  forms (`naa.…`, `0x…`, bare) are uncanonicalized until the round-four
  repair lands. Two paths through different HBAs or bridges can therefore
  present unequal identifier bytes and classify as two Strong, fully mutable
  plain disks. That population is protected by neither mitigation, and this
  ADR says so rather than rounding the residual up to covered. A refusal on
  ambiguity is not a same-device claim; it is a refusal to proceed while the
  question is open.
- **Privilege boundaries, journaling, recovery, secrets, hostile inputs:**
  unchanged. Nothing here adds a privileged surface or a parser.
- **No MUST is weakened.** LIN-006's detection MUST is satisfied by
  representation; INV-001's discovery MUST is satisfied because detection-only
  devices are still discovered and represented; SAFE-003 is untouched — its
  single-path identity record continues to describe every device v1 can bind,
  which after this decision is every supported write target.

## Options considered

### Option A — one canonical path per record, chosen by rule

Pick a canonical path (lowest-sorting, first-enumerated, or preferred-path).
Rejected: every candidate rule is unstable under re-probe, boot order, or path
failure, so an unchanged device compares unequal at PLAN-006 time — the
failure SI-12 exists to prevent — and the rule would be chosen with zero
multipath measurements on record.

### Option B — encode a path set now (ordered or unordered)

Decide the full representation today. Rejected: the choice is hash-visible,
irreversible without invalidating issued hashes, and would be made from
specification text alone against a device population nobody has measured.
This is the register's named worst case. The ordered/unordered sub-question
alone (does path order carry meaning?) has no evidence either way.

### Option C — detection-only representation, refusal as a write target (accepted)

Represent what the kernel itself materializes: the device-mapper multipath
node and its member path devices, connected by the **kernel-reported
membership relation** — with the product inferring no cross-path device
sameness of its own. The relation's **edge kind is deliberately not typed
here**: round three's record states that host-assembled devices with no
on-disk signature (loop, dm-linear, plain dm-crypt) have no legal edge under
the surviving containment/backing/production taxonomy and need a new edge
kind, with the no-sibling-capture theorem re-proved under the extended set —
and a dm multipath node, assembled from device-reported WWIDs with no
on-disk multipath signature, is exactly that class. Typing the edge is
SI-27's naming round's work; committing it here would be the same silent
hash-visible guess this ADR rejects in Option B. Any mutating operation on a
multipath device or a recognized member reports CAP-003 **`unsupported`**
with a multipath reason from CAP-003's reason vocabulary — a closed,
versioned enum delivered with the capability engine (WP-050): truthful,
because v1 does not implement the operation for this target, and not a
masked open decision. The in-spec precedent is INV-001's own sentence:
*"Network block devices are represented detection-only"* — and that
precedent's non-goal entry lives in Section 2.1, so this decision adds a
matching platform-neutral Section 2.1 entry rather than leaving the rule in
a Linux-only requirement while the harm it prevents exists on every
platform (a Windows host reaching a SAN LUN through two HBAs without MPIO
is the same case).

## Decision

Option C. v1 represents multipath detection-only, refuses it as a write
target via CAP-003 `unsupported` with a reason, and **defers the path-set
encoding** — including its body-versus-envelope placement — to the
specification change that first makes a multipath device a supported write
target, where it will land behind a MODEL-003 schema version bump rather
than as a silent widening, informed by measurements of real multipath
systems that do not yet exist. The normative home is a **Section 2.1
non-goal entry** (platform-neutral, and after ADR-0012 enforced by
unrepresentability like every other Section 2.1 entry), with LIN-006
carrying the Linux-specific detection mechanics.

This resolves SI-12 and removes its transitive block on SI-27. SI-27's
naming round proceeds **without a deduplication question** — the pair is two
kernel-materialized nodes — while the *collision-behaviour* question that
pair poses (two simultaneously present devices with equal identity fields
and only excluded-from-naming connection paths to tell apart) is assigned to
SI-27's scope, where the equal-identifier family already lives.
Deduplication across paths is the kernel's (device-mapper's), never
inferred by the product.

## Consequences

- Positive: SI-27's naming round unblocks; no hash-visible guess is made; the
  refusal surface is honest (`unsupported` is the true state); the kernel's
  own multipath assembly is represented rather than second-guessed.
- Negative: v1 cannot repartition SAN or multipath storage. For the desktop
  population this is a near-zero cost; for enterprise use it is a real
  deferral, and it is deliberate.
- The eventual path-set decision inherits an evidence obligation: multipath
  observability rows — which interfaces expose which path facts, at which
  privilege, **and whether two paths to one device present bytewise-equal
  identifiers, per layer and per form** — before any encoding is chosen.
- ACC/UI surfaces show a typed refusal for multipath targets rather than an
  absent device — consistent with the inspect chassis's refusal discipline.
- **A named follow-up owned by WP-035**, which this ADR cannot edit under
  WP-010's grant: the inspect chassis's in-band gated list currently emits
  `same-device-claims` gated on SI-12 as an open question, its pinned tests
  pin that string, README restates the list twice, and WP-035's boundary
  cites SI-12 in two lines. Those surfaces re-attribute from SI-12 (open
  question) to this ADR (standing decision — the prohibition on inferring
  sameness survives; its authority changes), and WP-035's boundary lines
  re-cite SI-27 alone for handles and this ADR for the sameness rule. Until
  that lands, the repository's live surfaces disagree with the register's
  sole-authority table, which is the drift its 2026-07-28 rule exists to
  prevent — so the follow-up is part of this resolution, not optional
  cleanup.

## Verification

- When WP-L100's adapter lands: T1 tests over **fixture-backed replay of
  recorded multipath enumeration data as regular files** — the WP-035 replay
  pattern; Section 11.3 keeps live dm assembly at T2/T3, and no Tier-1 test
  opens a block device. The fixture's shape is **gated on the multipath
  observability rows this ADR's Consequences demand**: a replay fixture
  authored from specification text alone would be the failure mode this
  ADR's own Context names. The tests assert: one dm node, member path
  nodes, the kernel-reported membership relation, and `unsupported` (with
  the multipath reason) for every mutating capability on all of them; and
  two recorded devices presenting equal stable identifiers without an
  assembled dm node yield SAFE-005 `blocked` on both.
- When WP-050's engine lands: the multipath reason exists in CAP-003's
  closed reason vocabulary, and CAP-001 computes the refusal per exact
  target with no plan in scope.
- Register: SI-12 recorded Resolved by this ADR and spec 4.3.0; SI-27's
  blocked-by-SI-12 state cleared, with the equal-identifier collision family
  assigned to SI-27's scope.
- The WP-035 follow-up named in Consequences lands, so no live surface
  cites SI-12 as an open gate.

## Revisit conditions

- Multipath or SAN storage becomes a supported write-target requirement.
- Evidence that hosts the product runs on meaningfully contain unassembled
  multipath members — any host, not only desktops: SAFE-005 applies
  wherever v1 actually runs, including a SAN-attached host without an
  assembled multipath framework.
- **Any measurement showing two paths to one device presenting unequal
  identifier strings** — by layer, by form, or by bridge synthesis — which
  would confirm the residual's uncovered population exists in practice.
- Any measurement showing the kernel's dm assembly is not a reliable
  deduplication boundary on a supported platform.
