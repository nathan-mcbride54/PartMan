# ADR-0019 Linux naming-source designation — recommendation round, 2026-08-13

Untracked session artifact, docs/reviews convention. Everything
load-bearing is restated in ADR-0034; this document records the round's
routes, its adversarial pass, and what was rejected, so the decision can
be audited against the alternatives it declined.

**What is being decided.** ADR-0019's canonicalization rule: "an
identifier used in naming is the byte string returned by the one named
source the evidence contract designates for it, per platform, verbatim
… Choosing the single source is the normative act … lands only with a
spec change." No Linux designation has ever been made. Without it,
`NamingFields::PhysicalDevice` has no referent for canonicalized serial
bytes and WP-L100 increment 3 cannot address a node. A prior round (the
WP-L100 arc, 2026-08-13) defeated its own proposal because the L2
observability row bundled four attributes and credited "sysfs"
generically; the 2026-08-13 readback rows (PR #321) un-bundled it and
changed the evidence picture materially.

## The evidence base (all citations: `docs/quality/observability.md`, the increment-6 Linux matrix and its 2026-08-13 readback rows)

1. **R1**: the only serial value ever observed by a qualifying Linux
   record came from the **USB device sysfs node's `serial` attribute**
   (the USB descriptor iSerialNumber), read as
   `/sys/block/sdb/device/../../../../serial`. Value stable across the
   L8 replug and reboot; per-unit distinct on the two-unit SanDisk pair.
   `/sys/block/sdX/device/serial` — the SCSI-node attribute — was
   **never read in the sitting** and has no observation on any Linux
   host.
2. **R2**: `device/wwid` failed **ENXIO** at every capture — a failed
   read, not a measured absence. No WWN-class value has ever been
   observed on Linux.
3. **R4**: `ID_PATH` took two values for one physical unit in one
   sitting (controller reattachment) — measured confirmation that
   path-shaped identifiers name attachment topology, which ADR-0019's
   exclusion list already bars.
4. The S4 sittings (Windows record): a same-model card-reader pair
   shares one constant serial at every layer — the population for which
   ADR-0019's collision group exists.
5. The delivered contract's `read_attribute`
   (`crates/adapter-linux/src/contract.rs`) decodes to `String`,
   refuses non-UTF-8 as `NotText`, and strips one trailing newline —
   all three are transformations ADR-0019's verbatim rule forbids for
   naming inputs.

## Routes

- **A (recommended, adopted).** Designate, for Linux devices attached
  through USB: the `serial` attribute of the device's nearest sysfs
  ancestor that is a USB device node — bytes verbatim as read,
  trailing newline included. Every other Linux attachment class and
  the WWN field: **undesignated**, fields absent, fail-closed through
  ADR-0019's existing weak-name/collision-group machinery. Close the
  measured-absent and failed-read seams in the same act, and state
  the bytes-preserving read-path requirement.
- **B.** Designate udev `ID_SERIAL_SHORT` (uniform across transports).
- **C.** Designate sysfs `device/serial` (the defeated round's shape).
- **D.** Designate `device/wwid` for the WWN field.
- **E.** Hold the designation until SATA/NVMe/virtio sittings exist and
  designate all attachment classes at once.

## The adversarial pass on route A

Each attack was run to kill the route; dispositions recorded.

1. **"Per platform" does not license a per-attachment-class table.**
   Sustained as a wording matter, dissolved as a substance matter: the
   rule's purpose is *exactly one source per identifier per device* so
   divergence is structurally absent, and a table keyed by (platform,
   attachment class) preserves that — each device resolves to at most
   one source. The ADR states the key extension explicitly as an
   addition rather than pretending Linux has one transport. No existing
   designation narrows, because none exists.
2. **The measured read was a fixed `../../../../` traversal, not the
   structural rule being designated.** True, and the transcript holds
   no `realpath`. Disposition: the structural spelling ("nearest USB
   device-node ancestor") resolves to the same directory the traversal
   reached on the measured topology — corroborated in-transcript by
   `ID_PATH`'s `…usb-0:1:1.0-scsi-0:0:0:0` nesting, which shows the
   interface and SCSI levels between the block device and the USB
   device node — and the *value and its stability* are measured facts.
   The canonical-path capture is filed as an evidence obligation on the
   next Linux sitting, not a blocker: what the designation needs
   measured (which byte string, from which interface, stable under
   replug/reboot, per-unit distinct) is measured.
3. **Verbatim-including-newline will surprise consumers.** The
   attribute read returns `…21\n`; stripping the newline is a
   transformation with an undecidable edge (a value legitimately ending
   in `0x0a`), and ADR-0019 kills the whole transformation class on
   purpose. Disposition: naming bytes are the read's bytes, newline and
   all; the rendered forms consumers see are display concerns.
   Consequence stated in the ADR so nobody "fixes" it.
4. **A same-model USB pair with a shared constant iSerialNumber derives
   equal names.** That is the S4-measured population, and equal names
   are what the collision group exists for: counted, flagged, blocked
   pairwise, detach as remediation. The design working, not a defect.
5. **Non-USB devices become serial-absent and same-size pairs group.**
   Priced and accepted: no qualifying non-USB Linux identity row
   exists, the delivered fixture population is USB, and ADR-0019's
   revisit condition (a mainstream population where the named source is
   systematically absent) is the pressure valve for extending the
   table. Designating an unmeasured source to avoid this cost is the
   exact defect the prior round died of.
6. **Failed-read handling could destabilize addresses across probes.**
   A transient failure yields a weaker-named capture than a successful
   one — two bodies differ. Disposition: that is honest; CONC-003
   staleness and PLAN-006 comparison exist for exactly this. The rule
   adopted mirrors the ADR's existing unreadable-aggregate-designator
   posture: on a failed read of a designated source the device names
   from its remaining fields, is marked indeterminate, and is not a
   plan operand — detection continues, nothing is silently omitted,
   and the whole-host denial the governing finding forbids cannot
   occur. Measured absence (`ObservedAbsent`) is the stable-truth case
   and joins `unavailable`'s existing consequence: field absent, name
   weaker, collisions group, the device remains an operand.
7. **Does this preempt SI-28?** No: SI-28's identifier-presence floor
   is untouched; the designation supplies naming bytes, not the floor
   decision, and the increment-3 register gate still holds SI-28 as
   recorded.

## Rejected routes, recorded

- **B, udev `ID_SERIAL_SHORT`.** Rejected on three grounds. It is a
  cached third-party computation — root's `udevd` computed it at
  event time; the WP-L100 record already classifies database values
  `Method::Heuristic`, deriving `inferred` — and naming from it makes
  the address rest on another actor's transformation. It is a *second
  source* for the same underlying descriptor, able to diverge from the
  direct read — the exact divergence ADR-0019's one-source rule exists
  to make structurally absent. And its availability class is worse:
  the sitting itself measured a database entry miss (the `b0:0` first
  capture) where sysfs answered.
- **C, sysfs `device/serial`.** Rejected: zero observations on any
  Linux host — the readback established the sitting never read it —
  so the designation would rest on nothing, which is the defeated
  round's defect resurrected in a narrower spelling.
- **D, `device/wwid` for WWN.** Rejected: no WWN-class value has ever
  been observed on Linux and the one candidate's read fails ENXIO on
  the measured class; the designation would be born firing ADR-0019's
  own revisit condition. WWN stays undesignated and absent on Linux.
- **E, hold for a complete table.** Rejected: undesignated arms are
  already fail-closed by ADR-0019's existing machinery, so partial
  designation loses no safety; holding blocks increment 3 for no
  measured gain; and the per-class table extension is exactly what the
  revisit condition schedules.

## Decision carried to ADR-0034

Route A, plus: `ObservedAbsent` joins `unavailable` (field absent, name
weaker, operand-eligible, collisions group); a failed read of a
designated source names from remaining fields, marks the device
indeterminate, and blocks it as an operand; naming inputs are read
through a bytes-preserving path — a text-decoded, newline-stripped
projection (the delivered `read_attribute`) is not a lawful naming
input; WWN undesignated on Linux; every non-USB Linux attachment class
undesignated. Spec version 12.11.0, minor: every change is an addition
to previously undefined territory, and no existing requirement's claim
narrows.
