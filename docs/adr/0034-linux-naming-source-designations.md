# ADR-0034: The Linux naming-source designations

- Status: Accepted
- Date: 2026-08-13. Made on the adversarially reviewed recommendation
  round of the same day
  (`docs/reviews/ADR-0019_LINUX_DESIGNATION_ROUND_2026-08-13.md`, an
  untracked session artifact; everything load-bearing is restated
  here), under the decision owner's directive "complete the ADR-0019
  designation", recorded and implemented in one autonomous arc —
  merging is not acceptance, and every element below is reviewable
  against the round's recorded alternatives.
- Spec version: 12.11.0 (minor under §0.1 — every change is an
  addition to previously undefined territory: no Linux designation
  existed to narrow, ADR-0019 defined no outcome for a measured-absent
  or failed-read source, and no numbered requirement's text changes)
- Work packages blocked: WP-L100 increment 3's designation blocker
  clears (its register gate still holds SI-28 exactly as its
  assignment records)
- Requirement IDs: Section 5, MODEL-004, SAFE-003, SAFE-005, ADR-C4,
  ADR-0018, ADR-0019
- Decision owners: Nate McBride

## Context

ADR-0019's canonicalization rule made choosing each naming identifier's
single per-platform source "the normative act", landing only with a
spec change — and no Linux designation was ever made, so
`NamingFields::PhysicalDevice` had no referent for canonicalized serial
bytes and no Linux node could be addressed. A first recommendation
round (the WP-L100 arc, 2026-08-13) defeated its own proposal: the L2
observability row bundled four sysfs attributes and credited "sysfs"
generically, so no designation could rest on it. The 2026-08-13
readback rows un-bundled that row from the archived 2026-08-04
transcript and changed the evidence picture materially:

- The only serial value a qualifying Linux record has ever observed
  came from the **USB device sysfs node's `serial` attribute** (the
  USB descriptor's iSerialNumber), read by parent traversal from the
  block device's SCSI node. It was stable across replug and reboot
  (L8) and per-unit distinct on the measured two-unit pair (L9).
- `/sys/block/sdX/device/serial` — the SCSI-node attribute a natural
  reading of the bundled row suggested — was **never read in the
  sitting** and has no observation on any Linux host.
- `device/wwid` failed with `ENXIO` at every capture: a **failed
  read**, not a measured absence. No WWN-class value has ever been
  observed on Linux.
- `ID_PATH` took two values for one physical unit within one sitting
  when the passthrough moved controllers — measured confirmation that
  path-shaped identifiers name attachment topology, which the
  exclusion list already bars.

Two seams surfaced by the same work are closed here because the
designation is unusable without them: ADR-0019 defined a naming
outcome only for an `unavailable` source, while the delivered contract
produces two more (a measured absence, ADR-C4's `ObservedAbsent`; and
a failed read), and ADR-0019's verbatim rule is incompatible with the
delivered text-decoding read path.

## The designations

The designation table is keyed by **(platform, attachment class)** —
an addition to ADR-0019's per-platform spelling, stated rather than
implied. The rule's purpose is exactly one source per identifier per
device, so divergence stays structurally absent: each device resolves
to at most one source per identifier.

| Platform | Attachment class | Identifier | Designated source |
| --- | --- | --- | --- |
| Linux | USB-attached block device | serial | The `serial` attribute of the device's nearest sysfs ancestor that is a USB device node, bytes verbatim as read — trailing newline included |
| Linux | USB-attached block device | WWN | **Undesignated** |
| Linux | every other attachment class | serial, WWN | **Undesignated** |

An undesignated (platform, class, identifier) cell means the field is
absent, the name is weaker, and any resulting collision fails closed
through ADR-0019's collision group — the machinery that already
exists. Nothing here designates by prediction: the one designated cell
is the one with measured value, stability, and per-unit distinctness
behind it.

**Verbatim includes the trailing newline.** The attribute read returns
the descriptor bytes followed by `0x0a`; stripping it is a
transformation with an undecidable edge (a value legitimately ending
in `0x0a`), and ADR-0019 excludes the transformation class wholesale.
Naming bytes are the read's bytes. Rendered display forms are a
consumer concern and never feed naming.

**The resolution rule is structural, not a fixed traversal.** The
measured instrument used `device/../../../../serial`; the designation
names the structure that traversal reached — the nearest ancestor
sysfs node that is a USB device node — corroborated in-transcript by
`ID_PATH`'s interface-then-SCSI nesting. Capturing the resolved
canonical path is an evidence obligation on the next Linux sitting,
recorded below; the value, its interface, and its stability are
already measured.

## The two outcome rules, closed in the same act

ADR-0019's sentence "where the named source is `unavailable`, the
field is absent, the name is weaker, and any resulting collision fails
closed through the group" gains two siblings:

- **Measured absence** (`ObservedAbsent` — the source positively
  observed not to exist): the same consequence as `unavailable`. The
  field is absent, the name is weaker, the device remains an operand,
  collisions group. A stable truth about the hardware is a lawful
  weak name.
- **Failed read** (the source exists or may exist, and reading it
  failed — `ENXIO`, `EIO`, denial, or any other failure): **not
  absence.** The device derives its name from its remaining fields,
  is marked indeterminate, and is not a plan operand — the same
  posture ADR-0019 already gives an aggregate whose native designator
  is unreadable. Detection, display, and diagnostics continue;
  nothing is silently omitted; the governing finding's whole-host
  denial cannot occur. A capture taken during a transient failure
  differs from a healthy capture — deliberately: CONC-003 staleness
  and PLAN-006 comparison exist for exactly that divergence.

## The bytes-path requirement

Naming inputs are read through a bytes-preserving path: the byte
string the source returned, verbatim — no UTF-8 validation, no
newline stripping, no re-encoding, non-UTF-8 bytes legal, exactly as
ADR-0019 states. The delivered `read_attribute`
(`crates/adapter-linux/src/contract.rs`) decodes to `String`, refuses
non-UTF-8 as `NotText`, and strips one trailing newline — three
transformations — and is therefore **not a lawful naming-input path**.
It remains correct for its delivered purpose (text-shaped observation
rows). The contract owes a bytes read seam before any naming input is
consumed; that is WP-L100 increment 3's first delivery obligation, not
a change this ADR makes to code.

## Options considered

### udev `ID_SERIAL_SHORT` (uniform across transports)

Rejected on three grounds. It is a cached third-party computation —
root's `udevd` computed it at event time, and the delivered record
already classifies database values `Method::Heuristic`, deriving
`inferred` — so naming from it rests an address on another actor's
transformation. It is a second source for the same descriptor, able to
diverge from the direct read — the divergence ADR-0019's one-source
rule exists to make structurally absent. And its availability class is
measured worse: the sitting itself recorded a database-entry miss
where sysfs answered.

### sysfs `device/serial`

Rejected: zero observations on any Linux host — the readback
established the sitting never read it — so the designation would rest
on nothing: the defeated round's defect resurrected in a narrower
spelling.

### `device/wwid` for the WWN field

Rejected: no WWN-class value has ever been observed on Linux, and the
one candidate's read fails `ENXIO` on the measured class. The
designation would be born firing ADR-0019's own revisit condition.

### Holding until every attachment class is measured

Rejected: undesignated cells are already fail-closed through the
existing weak-name and collision-group machinery, so partial
designation loses no safety; holding blocks increment 3 for no
measured gain; and extending the table per class is exactly what the
revisit condition schedules.

## Decision

The designation table above is normative, versioned with the evidence
contract; changing any cell is hash-visible by construction and lands
only with a spec change, per ADR-0019. The `ObservedAbsent` and
failed-read naming outcomes and the bytes-path requirement land with
it. WWN is undesignated on Linux. Every non-USB Linux attachment class
is undesignated. SI-28 is untouched and remains the increment-3
register gate exactly as recorded.

## Consequences

- **Positive:** WP-L100 increment 3 can address a node: the serial
  referent exists for the measured population, which is the fixture
  population the package tests on. The two contract outcomes that had
  no naming rule now have one each, fail-closed in the direction each
  deserves.
- **Negative, accepted knowingly:** non-USB Linux devices derive
  serial-absent names until their classes are designated, so
  same-size pairs of them collide into blocked groups while
  simultaneously attached; a same-model USB pair sharing a constant
  descriptor serial (the S4-measured population) derives equal names
  and groups — the design's intended representation of that
  ambiguity, with detach as remediation; and naming bytes carry a
  trailing `0x0a`, which will surprise anyone expecting rendered
  text — recorded so nobody "fixes" it into a transformation.
- **Evidence obligations:** (1) the next Linux sitting captures the
  resolved canonical path of the USB-ancestor `serial` attribute
  (`realpath`) beside the value, closing the structural-resolution
  inference; (2) per-class identity rows for SATA, NVMe, and
  virtio-class attachment — each class's designation is a future
  table extension under this ADR's discipline, resting on its own
  rows; (3) ADR-0019's obligation 4 (Windows and macOS
  named-single-source rows) is unchanged.

## Verification

- When increment 3's naming lands: the serial bytes in any
  `NamingFields::PhysicalDevice` equal the designated attribute's
  bytes verbatim (newline included) for USB-attached devices; no
  naming input flows through `read_attribute`; a measured-absent
  source yields an absent field on an operand-eligible node; a failed
  read yields an indeterminate non-operand that still appears in the
  body; an undesignated class yields absent fields with no read
  attempted against an undesignated source.
- Any text implying a Linux WWN source exists, or that udev database
  values feed naming, is an error against this ADR.

## Revisit conditions

- A qualifying sitting measures a serial or WWN source for a
  currently undesignated Linux attachment class — the table extension
  lands with its rows.
- A platform or class is measured where the designated source is
  systematically absent or constant for a mainstream population —
  ADR-0019's own revisit condition, restated here for the USB cell:
  the S4 shared-constant population is already known for card
  readers, and if such hardware becomes the common case the cost
  moves from edge to center.
- The kernel's sysfs contract for the USB `serial` attribute changes
  shape (encoding, termination) — hash-visible by construction, and
  the designation deserves re-examination rather than a compatibility
  shim.
