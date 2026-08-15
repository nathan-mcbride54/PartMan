# WP-L100 — the arc plan, written before the assignment

**Session:** Nate directed "pick up where the last agent left off", then
chose the read-only adapter front and Linux as its first platform.
**Follows:** `HANDOFF_2026-08-12_FABLE_REGISTER_RESIDUE_TO_NEXT.md`
(main at #312, spec 12.10.0, nothing in flight).

> Untracked local artifact, docs/reviews convention: never stage into a
> commit; `verify-change-ownership` refuses it.

## 0. Why this package, and what it unblocks

Every layer above discovery is delivered and has no facts source. WP-010
carries the domain types, WP-050's engine judges `RuntimeFacts` and a
`TopologySnapshot` it cannot gather, WP-060 plans over snapshots nobody
produces, WP-070 journals applies nobody can run. The Section 14 row
(`WP-L100 | Linux read-only inventory and capability adapter | WP-010,
WP-020, WP-050 | M1`) has every prerequisite met.

`crates/capability/src/engine.rs` states the seam in its own words:
runtime facts are "supplied by the caller (WP-035's doctor today; **the
platform adapters tomorrow**)". This package is that caller.

## 1. What the search established, and what changed because of it

Four findings shaped the assignment. Each was cheaper to find now than
mid-increment.

1. **WP-035's CLI already reads the Linux client contract — and
   deliberately interprets nothing.** `apps/cli/src/devices.rs` reads
   sysfs attributes and the udev database through a `DeviceSource` seam,
   prints raw `(interface, native property, value)` triples, and refuses
   to normalize, elect a canonical identifier, group devices, or read
   `ID_FS_*`/`ID_PART_ENTRY_*`. Its own charter says partition
   enumeration is INV-004 and belongs elsewhere. So this package is the
   modeling layer, not a second reader — and it owns its own seam in its
   own crate, because a crate depending on an app would be backwards.

2. **The client's snapshot is normative, not a transgression.** The
   CLI's `TOPOLOGY_REFUSAL` says a valid snapshot is the helper's to
   produce. Section 6 settles the apparent tension in terms: "the
   client's draft snapshot is a **proposal**, and the snapshot whose
   hash the authorized plan binds is the one HLP-002's re-discovery
   produces during validate-plan." The adapter emits the proposal.
   `TopologySnapshot::assemble` accepts it, and `protection::Facts`
   treats a missing fact as "honest absence [that] fails closed" — so a
   client capture with no `table_states` is constructible and refuses
   exactly where the helper's authored values belong.

3. **Identity strength has a structural conservative answer.** SAFE-003
   makes `Strong` require a positively determined partition-table state;
   INV-003 (ADR-0014) forbids this contract emitting one on any
   platform. Every client-derived record is therefore `Weak` **by
   SAFE-003's own terms** — not by policy. That is also the conservative
   answer SI-28's open gate requires, so the gate is honored by
   construction: `Strong` gets no constructor in this crate. The CLI's
   `identity-strength: not-established (SI-28)` and this package's
   always-`Weak` are the same refusal at two altitudes.

4. **The multipath arm is evidence-gated, and its evidence does not
   exist.** ADR-0011's Verification section says: "When WP-L100's
   adapter lands: T1 tests over fixture-backed replay of **recorded**
   multipath enumeration data … The fixture's shape is gated on the
   multipath observability rows this ADR's Consequences demand: a replay
   fixture authored from specification text alone would be the failure
   mode this ADR's own Context names." `docs/quality/observability.md`
   contains **zero** multipath rows, and that file is WP-035's. This is
   recorded in the assignment as a named gate on one increment's one
   arm, plus an obligation on WP-035 — not discovered in increment 4.

## 2. Imported obligations the creation cannot omit

The founding-duty pattern WP-070 established. Enumerated in the
assignment with the increment that owns each:

| Source | Obligation | Increment |
| --- | --- | --- |
| ADR-0033 §Verification 2 | INV-004 free extents and alignment presented as derivations — inputs named, no observation set, refused over an `unavailable`/`conflicting` input with the input's state surfaced — **a fixture for each arm** | 3 |
| ADR-0011 §Verification | Fixture-backed replay of recorded multipath data; one dm node, member paths, kernel-reported membership, mutating capability `unsupported` on all; equal-identifier pair without an assembled node → `blocked` on both | 4, evidence-gated |
| ADR-0013 / INV-003 | The reach declaration published per state, from the contract, never per device, never omitted when "no" | 1 |
| ADR-0014 / ADR-0016 | No client-emitted table state, no client-authored protection verdict — structurally | 1–3 |
| ADR-0018 | The closed positive-local `TransportClass` list, `Unrecognized` failing closed | 2 |
| ADR-0019 | Derived positional addresses, collision groups, host-backing edges | 3–4 |
| ADR-C4 | A positively observed absence is a value, not an unavailability | 1 |
| ADR-C5 | An `Aggregate` carries its **self-reported** member count | 4 |

## 3. Register gates recorded at creation

Not an empty list — WP-070's was the exception, not the rule.

- **SI-28 (Mitigated-open)** — gates any strength answer above `Weak`.
  Conservative answer structural (no `Strong` constructor). The floor's
  inputs — transport class, removability, identifier attribution — are
  this package's duty to report truthfully.
- **SI-37 (Open, Later)** — gates any relaxation of the unassembled
  unequal-identifier population. Detection-only holds; no cross-path
  sameness inference anywhere.
- **SI-13** — pinned beyond this package (WP-L110's validate-plan
  surface), verified accurate by the 2026-08-12 residue sweep.

## 4. The route decision this package will face

LIN-001 names UDisks2 as the Linux discovery interface. The delivered
contract reads sysfs and the udev database as ordinary files with an
empty dependency closure. Adopting UDisks2 buys LIN-001's named
interface at the price of a D-Bus client dependency and an IPC surface.
That is the WP-035 increment-10 / WP-040 transport triangle again, so
LIN-001 is **claimed increment-gated** behind its own recorded route
decision, and the assignment says so rather than letting the sysfs route
become LIN-001 by drift.

## 5. Sequence

1. `Governance:` PR creating `docs/work-packages/WP-L100.md` — only that
   file, per AGENTS.md, landing before any work that needs its paths.
   Born `hand-maintained` (the generated file cannot exist before the
   first increment); no Delivery status section until increment 1, the
   WP-070 birth shape.
2. Increments 1–5 under `Work-Package: WP-L100`, each with `cargo xtask
   ci`, `cargo xtask test --tier 1`, `verify-change-ownership --base
   origin/main`, real exit codes checked, and mutants killed by named
   tests before proposal.

No Rust lands in step 1, so no WP-020 sitting is owed by the governance
PR (the #311/#312 precedent: Markdown-only, no sitting).

## 6. Evidence-sourcing rule the assignment sets

Structural properties (bounds, refusals, closure, unconstructibility)
may be tested over authored trees — WP-035's `FakeDeviceSource` shape.
Every **representational** claim about what a real Linux host exposes
rests on a recorded capture committed as a text fixture. Where no
recording exists, the increment says so and delivers the fail-closed
answer instead of authoring a fixture from specification text. ADR-0011
makes this explicit for multipath; the assignment generalizes it,
because the failure mode is not multipath-specific.
