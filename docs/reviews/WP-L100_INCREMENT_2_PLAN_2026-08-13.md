# WP-L100 increment 2 — the arc plan, written before the first line of code

**Session:** Nate directed "start increment 2" after #314 merged.
**Follows:** `WP-L100_ASSIGNMENT_PLAN_2026-08-12.md` (the assignment) and
increment 1 as merged at `7d314d8`.
**Grounding:** an 8-agent design pass — five readers over ADR-0018, the
Linux observability record, SI-28, INV-001/002, and the reach
re-decision; then three adversarial lenses (invention, fail-open,
scope). The lenses overturned four of the readers' own recommendations,
including two labelled `[GROUNDED]` that were not.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block (`docs/work-packages/WP-000.md`) and lands in its own `Work-Package:
> WP-000` commit, never bundled with code. As first written this document
> carried the banner "untracked local artifact, docs/reviews convention:
> never stage into a commit; `verify-change-ownership` refuses it". That is
> false — the gate refuses `docs/reviews` bundled into a code change under
> another package, not the path itself — measured in
> `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md` §6.1 and swept 2026-08-18.

## 0. The finding that shapes the increment

**No Linux transport classification can be grounded, and that is the
answer rather than a gap.**

- No value of `ID_BUS` is recorded anywhere in this repository, on any
  Linux host. The one row naming the key records that the WSL2 udev
  database *carries* it, never what it said.
- Five of ADR-0018's six positive-local classes have zero Linux
  measurement. The only real Linux hardware ever measured is two USB
  flash sticks (2026-08-04); the only other device tree is WSL2 virtual
  SCSI, which the record explicitly disclaims for real trees: "Real
  NVMe, SATA, USB, and SD/MMC device trees expose different files under
  `device/`."
- ADR-0018's own evidence obligation (2), "fabric-versus-local transport
  discrimination rows per platform for each listed local transport", is
  **outstanding** — on every platform.
- ADR-0018 states that an arm consuming an unmeasured state-layer fact
  does not leave `Indeterminate`.

So increment 2 answers `TransportClass::Unrecognized` for every device,
unconditionally, and records that **as the discharge** of imported
obligation 6 rather than as a shortfall. ADR-0018 already prices the
cost under "Negative, accepted knowingly": availability costs on
unmeasured populations that fail closed until their named evidence
exists. Writing an `ID_BUS`-value-to-class table could only come from
udev documentation — the failure mode this repo refuses.

## 1. What the adversarial lenses overturned

Recorded because each would have shipped as a defect.

1. **Do not construct `Facts`, a snapshot, or any `NodeId`-keyed map.**
   `Facts.transports` is keyed by `NodeId`; `NodeId` is an ADR-0019
   derived address; ADR-0019 addressing is imported obligation 7, owned
   by increments 3 and 4. The assignment's phrase "SI-28's floor inputs
   are reported as facts" reads as an instruction to build a `Facts`,
   and doing so would import increment 3's whole naming layer.
   Increment 2's output is a **crate-local per-device record** carrying
   `PropertyObservations` per property.
2. **"Port WP-035's whole-device admission rule" is not grounded.** A
   code precedent is not a measurement. The platform claim underneath
   it — that a whole device positively lacks the `partition` attribute —
   has no observability row; the only trace is instrument code inside
   the non-qualifying 2026-08-02 WSL2 loop record.
3. **"Keep the field roster identical to WP-035's" is not grounded**
   either: `removable` has no row on any Linux host, and
   `physical_block_size` has none on real hardware.
4. **The PLAN-006 hazard was overstated.** Section 6 says the client's
   draft snapshot is a *proposal* and the bound snapshot is the one
   HLP-002's re-discovery produces at validate-plan, so a client/helper
   transport disagreement does not by itself break PLAN-006. The real
   and weaker exposure is CAP-007 advisory divergence. Increment 2 must
   not repeat the stronger claim.
5. **"ID_BUS on real hardware is entirely unmeasured" is imprecise.**
   L2's interface column includes `/run/udev/data/b<maj>:<min>`, so the
   udev entry *was* read on real hardware; only its keys and values went
   unrecorded. That makes the missing row a transcription gap in a
   completed capture — materially cheaper than an untaken measurement,
   and the obligation should say so.

## 2. Decisions this increment makes, each to be recorded

| # | Decision | Why, and what would revisit it |
| --- | --- | --- |
| D1 | Transport is `Unrecognized` for every device | §0. Revisited by the first fabric-versus-local discrimination row |
| D2 | Output is a crate-local per-device record, not `Facts` | §1.1; increment 3 owns addressing |
| D3 | Read `removable` and `queue/physical_block_size` despite having no row, and derive **no** classification from either | Declining makes an SI-28 floor input structurally `Unavailable`; an ADR-C4 read that can answer `NotPresent` is not a claim the platform exposes the field. `lib.rs`'s standing per-interface evidence sentence must be qualified in the same change, or it ships false |
| D4 | Whole-device admission on positively-absent `partition` (`NotFound` only), with its unmeasured basis recorded and the row filed | Fail-closed and honest; `is_ok()` fails open and promotes a partition into the device list |
| D5 | Exclude `ID_PART_TABLE_TYPE` and `ID_PART_TABLE_UUID` as increment-3 material | A partition-table identifier is `NamingFields::PartitionTable` + `TableRole`, which the topology increment owns |
| D6 | Reach: contract word moves to `implemented-reaches-no-table-state`; all six cells stay `not-measured` with per-cell reasons | The citation vocabulary is observability headings, and no heading exists for `mbr` or `apple-partition-map` — "move to measured" is unexecutable for at least two cells |
| D7 | Correct increment 1's `ID_PART_TABLE_TYPE` sentence in `reach.rs` and `reach.md` | It attributed a `blkid -p` output (measured **denied** to the client) taken over **regular files** to the client-readable udev database. Recorded as a correction, not silently edited |

## 3. Scope boundaries this increment must not cross

- No `/proc/partitions`, `/proc/self/mounts`, `/proc/swaps`,
  `/sys/firmware/efi`, `/dev/disk/by-*`. Boot and system role is
  increment 4's; widening the roster re-opens the published reach. The
  `/dev/` needle already enforces the `by-*` half structurally.
- No CAP-003 `Reason` variant and no `Facts` field proposed, even
  conditionally — WP-050's and WP-010's design.
- No statement about what the privileged helper reads — WP-L110's.
- No edit to `docs/quality/observability.md` or `AGENT_BUILD_SPEC.md` —
  WP-035's paths; `verify-change-ownership` refuses.

## 4. Claims recorded as partial, so no row reads as delivered

- **INV-001** names loop devices (increment 4's), NBD (detection-only),
  and eMMC, SD media, virtual disks and hardware RAID LUNs — device
  classes with zero Linux measurement. No reader examined INV-001's text
  until the scope lens did; its evidence line must not read as complete.
- **INV-002** is partial on two distinct axes: *deferred* (system/boot
  role → increment 4) and *permanently out of scope* (identity record
  and strength → the helper at validation).

## 5. Obligations to file on other packages

Executed only as entries in WP-L100.md's "Obligations on other
packages" section — never as an edit to another package's paths.

1. **WP-035**: the `ID_BUS`/`ID_PATH` values per device class — a
   transcription gap in the completed 2026-08-04 capture, not an
   untaken measurement.
2. **WP-035**: a `removable` row (none exists on any Linux host) and a
   real-hardware `queue/physical_block_size` row.
3. **WP-035**: the whole-device `partition`-attribute discriminator, and
   the `size` 512-byte-unit convention — the latter **blocks increment
   3**, because `NamingFields::PhysicalDevice.total_bytes` is required
   and non-optional and ADR-0033 closes derivations at exactly two, so
   no third can be minted by an adapter.
4. Record that WP-035's own `observability.md` share is an enumerated
   grant that does not visibly cover a transport-discrimination
   protocol — so these may need their own governance step. Worth
   discovering now rather than at increment 4.

## 6. Mechanics the design pass nearly omitted

- **Extend `shipped_sources()`** for every new module in the same
  change, and mutation-verify it: removing the new entry must fail a
  named test. Both SAFE-002 scans iterate that fixed array, so a
  forgotten entry yields two green tests asserting nothing about the
  new code.
- **Update the crate doc**: `lib.rs` currently describes increment 2's
  scope as future, and its standing per-interface evidence sentence
  becomes a per-field claim once a roster exists.
- **Full record sweep in one change**: the increment-2 delivery-status
  row, the README status row, the CHANGELOG entry.
- **Mark imported obligation 6 discharged**, in this increment, in
  WP-L100.md — never anywhere else.
- Traceability annotations with bare IDs, `cargo xtask traceability
  --write` run last, and real exit codes checked for `ci`,
  `test --tier 1`, and `verify-change-ownership --base origin/main`.
- Mutants applied and killed by named tests before proposal, with the
  count stated only after the pass runs.

## 7. Open questions this increment records rather than answers

Each is genuinely undecided and none is a §1.11 requirement-vs-requirement
conflict, so each belongs in the increment's record, not the register.

1. Whether a `udev`-derived (`Method::Heuristic`, `inferred`) value may
   license a positive-local class at all. Nothing in ADR-0018, ADR-C4,
   or the register addresses it. Moot while D1 holds, and it becomes
   live the moment a discrimination row lands.
2. Whether `TransportClass` is an observed INV-002 property with its own
   MODEL-004 observation set, or a normalization over raw fields.
   ADR-0033 closes derivations at exactly two, so a closed-enum
   classification over raw strings fits neither category.
3. What a sysfs/udev disagreement produces. `Confidence::Conflicting`
   exists and ADR-0018's join rule makes a positive cross-layer
   disagreement `Indeterminate`, but `Facts::transports` holds one value
   per device and cannot represent the disagreement.
4. Whether a `PhysicalDevice` node must carry a transport fact at all:
   omitting it and saying `Unrecognized` are both structurally legal and
   produce **different canonical bytes**.
