# The Linux contract's field roster and its evidence

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` §7.1 (INV-001,
  INV-002) and MODEL-004
- Owner: WP-L100 (`docs/work-packages/WP-L100.md`), increment 2
- Decided semantics carried: ADR-C4 (a positively observed absence is a
  value, not an unavailability), ADR-0018 (the device-scope transport
  arm's closed positive-local list), ADR-0014 (the partition-table state
  is helper-authored, so no client emits one), ADR-0034 (the Linux
  naming-source designations — whose designated serial source is a USB
  device node's `serial`, reached by traversal, and *not* the
  `device/serial` path this roster reads)
- Implementation: `crates/adapter-linux`'s `devices` module; the rosters
  below are the `SYSFS_FIELDS` and `UDEV_KEYS` constants, held in
  agreement with this document by the
  `the_published_field_roster_matches_this_crates_constants` test
- Underlying byte profile: none — this document records a read roster and
  its evidence, not a wire format

This document records a delivered roster. It decides nothing: a field
appears here because `crates/adapter-linux` reads it, and that module's
tests are the authority wherever a sentence could be read two ways.

Its purpose is narrower and more specific than the roster itself. The
crate's own documentation claims that the two **interfaces** it reads are
the ones `docs/quality/observability.md` establishes as client-readable
on real hardware. That claim is about interfaces, not about every field
read through them — and the difference is load-bearing, because a field
can be read through a measured interface at a path no row measures. This
document is where that gap is stated per field rather than implied away.

**Revised 2026-08-14 against the rows issue #318 filed and the two
2026-08-13 records that landed them.** The first version of this document
stated four gaps that are now measured — `removable`, a real-hardware
`physical_block_size`, the whole-device `size` unit, and the
`partition`-attribute admission rule — and stated two things above their
evidence, which is the defect this document exists to catch and which it
has now made twice. Both are corrected in place rather than edited away:
`device/wwid`'s real-hardware result was **not** an observed absence but a
failed read, and `device/serial` was credited with a real-hardware serial
that was read from a different node entirely. Each correction names the
row that establishes it.

## 1. What each column means

`Evidence` names the observability row that establishes the field as
client-readable, or says **none** where no row does. A field with no row
is not thereby forbidden: reading it is not a claim that the platform
exposes it, because ADR-C4's outcome vocabulary lets the answer come back
`ObservedAbsent` or `Unavailable` honestly. What a missing row does
forbid is *relying* on the field — deriving a classification from it, or
describing it as measured.

`Strength` distinguishes the two measured tiers this record holds:

- **real-hardware** — the 2026-08-04 sitting on a disposable non-WSL VM
  with authorized passthrough fixture media, together with the two
  2026-08-13 records that read it back and extended it on the same
  measured unit: the **readback rows R1–R4**, transcribed from that
  sitting's archived transcript under re-verified custody and taking no
  new measurement, and the **floor-rows sitting FR1–FR5**, a preregistered
  sitting on VM 9437.
- **WSL2-only** — the 2026-07-28 table, whose own scope limits record
  that real device trees expose different files under `device/`, and
  that a WSL2-only absence must not be relied on until confirmed on a
  non-WSL distro kernel.

## 2. The sysfs roster

Paths are relative to the device's directory under the block class.

| Native property | Path | Evidence | Strength |
| --- | --- | --- | --- |
| `size` | `size` | Total size row; **FR5** read the whole-device attribute as `244457472`, and 244457472 × 512 = `blockdev --getsize64`'s `125162225664` **exactly** — the 512-byte unit measured on the whole-device node itself. R3 measures the same unit on the partition node against a declared byte extent | real-hardware, unit measured on both nodes |
| `ro` | `ro` | Read-only flag row; device `ro` read back at every real-hardware capture | real-hardware |
| `removable` | `removable` | **FR1** — `1`, rc 0, byte-stable across the double capture: the first qualifying value on any Linux host, and the SI-28 floor input this field exists to supply | real-hardware |
| `logical_block_size` | `queue/logical_block_size` | Sector-size row, the non-WSL frozen projection, and **FR2** — `512` on real hardware | real-hardware |
| `physical_block_size` | `queue/physical_block_size` | **FR2** — `512` on real hardware, rc 0, byte-stable: the first real-hardware row for this attribute, which the non-WSL frozen projection omits | real-hardware |
| `device/vendor` | `device/vendor` | Vendor/model/WWID row; vendor and model strings observed on real hardware | real-hardware |
| `device/model` | `device/model` | Vendor/model/WWID row; observed on real hardware | real-hardware |
| `device/wwid` | `device/wwid` | Present on WSL2 virtual SCSI. On real usb-storage the read **failed `ENXIO`** at every capture across every layout and both stability legs (**R2**) — a failed read, not the absence this document previously recorded | real-hardware (as a **failed read**) |
| `device/serial` | `device/serial` | **none** — **R1** establishes that this path was never read in the 2026-08-04 sitting, and no qualifying record reads it on any Linux host. The serial that sitting observed came from a different node; see the first note below | — |

Two of these carry a recorded decision rather than a plain row:

- **`device/serial` is read at a path no record measures, and it is not
  the designated source.** The serial the 2026-08-04 sitting observed was
  read by parent traversal from the block device's SCSI node four levels
  up to the **USB device sysfs node** —
  `/sys/block/sdb/device/../../../../serial`, whose `serial` attribute is
  the USB descriptor's iSerialNumber (**R1**; **FR4** later resolved that
  traversal to `/sys/devices/…/usb10/10-1/serial` and read the identical
  value by both paths, discharging ADR-0034's evidence obligation 1).
  `/sys/block/sdX/device/serial` — the SCSI-node attribute this roster
  reads, which the earlier bundled spelling
  `device/{vendor,model,wwid,serial}` invited a reader to assume — was
  never read in that sitting and is unobserved on every Linux host.
  **ADR-0034 designates the traversal,
  not this path**, as the Linux serial source for USB-attached block
  devices. Reading it stays legitimate on §1's terms — reading is not
  claiming, and the answer comes back honestly through ADR-C4's
  vocabulary — but nothing may rely on it, and reconciling the roster with
  the designation belongs to increment 3, where ADR-0019's derived
  addresses land and the designated bytes acquire a consumer.
- **`device/wwid`'s real-hardware result is a failed read, not an
  absence.** `cat /sys/block/sdb/device/wwid` failed `ENXIO` ("No such
  device or address") at every capture; `ENOENT` would have printed "No
  such file or directory" (**R2**). The distinction is ADR-C4's own and it
  is load-bearing here: an `ObservedAbsent` is a value, and this is not
  one. The delivered code already draws the line correctly — only
  `ErrorKind::NotFound` reaches `AttributeRead::NotPresent`, and every
  other error becomes `AttributeRead::Failed` — so this was a defect in
  this document, not in the contract. ADR-0034 leaves the WWN identifier
  **undesignated** for every Linux attachment class on exactly this
  evidence: no WWN-class value has ever been observed on Linux.

The `size` unit is no longer a convention. **FR5** measured the
whole-device sysfs `size` against a byte-denominated interface in the same
sitting on the same unit, and **R3** measured it on the partition node
against a declared byte extent, with `/proc/partitions` agreeing at its own
1 KiB unit in both. This adapter still reports `size` **raw and
uninterpreted**, which is unchanged and correct:
`NamingFields::PhysicalDevice` carries a required `total_bytes`, ADR-0033
closes derivations at exactly two so no third may be minted here, and the
acceptance that turns 512-byte sectors into that byte figure is increment
3's recorded act. What has changed is that the act now cites a measured row
rather than a convention.

## 3. The database roster

| Key | Evidence | Strength |
| --- | --- | --- |
| `ID_SERIAL` | Identifier row names the key as carried | WSL2-only |
| `ID_SERIAL_SHORT` | **R1** — carries the USB-node serial's value (`A20036CA8695D921`) at every database capture, on the disk and partition nodes alike, as a database-side derivation of udev's own probing rather than an independent client observation | real-hardware |
| `ID_WWN` | Identifier row names the key as carried; no value recorded on any Linux host, consistent with `device/wwid`'s failed read | WSL2-only, key only |
| `ID_WWN_WITH_EXTENSION` | Identifier row names the key as carried; no value recorded | WSL2-only, key only |
| `ID_BUS` | **R4** — `usb` on every database capture of the real USB mass-storage unit, alongside `ID_USB_DRIVER=usb-storage`, `ID_USB_INTERFACES=:080650:` and `ID_TYPE=disk` (none of which this roster reads) | real-hardware |
| `ID_PATH` | **R4** — two values for one physical unit within one sitting (`pci-0000:00:1d.7-usb-…` before a controller reattachment, `pci-0000:01:1b.0-usb-…` after), measured evidence that this key names attachment topology and not the unit | real-hardware |

The transcription gap this section previously recorded is closed. The
2026-08-04 sitting read the database entry — its interface column names
the record path — but transcribed only the file-system and signature
keys; the 2026-08-13 readback went back to the archived transcript, under
custody re-verified by rehash before anything was read, and transcribed
the rest. Nothing here is a new measurement.

`ID_PATH`'s two-values-in-one-sitting result deserves its own sentence,
because it is the only key on this roster with *measured* evidence about
what it names: the same physical unit re-pathed when the passthrough moved
between emulated controllers. That is confirmation on real hardware of the
exclusion ADR-0019's path-shaped-identifier rule already asserts, on
exactly the hardware class where the distinction matters.

## 4. What the roster deliberately omits

- **Every partition-table key**, including the table type and the table
  identifier. A table identifier is topology material — increment 3's —
  and the table *state* is authored by the privileged helper under
  ADR-0014 in any case. Their absence is what lets the published reach
  declaration say this contract carries no partition-table key.
- **Every interface outside the two the contract closed**: no kernel
  partition list, no mount table, no swap table, no firmware directory,
  no symlink farm. The symlink farms are excluded on measured grounds as
  well as scope: the collision row observed `by-uuid`, `by-partuuid` and
  `by-label` all collapsing silently to the last-arriving device.
- **Boot and system role.** INV-002 names it; its Linux route runs
  through mounts, swap and firmware state, which is increment 4's
  detection layer. No row measures it on Linux at all.
- **Any derived value.** No removability boolean, no transport
  classification from a key's value, no byte capacity, no identity
  strength. Each is either another increment's or another package's. Three
  of the four now have their measured input in the record and wait only on
  the increment that consumes them; the fourth — a transport
  classification — waits on a protocol, not a value, and §5 says where
  that protocol now lives.

## 5. What the roster's evidence does not establish

- That any value of any key **names a transport class**. This is the one
  gap of the six issue #318 filed that measurement did not close, and its
  shape has changed rather than shrunk. Values are now recorded —
  `ID_BUS=usb` on a real USB mass-storage unit (**R4**) — so the earlier
  statement that no classifying value exists on any Linux host is no
  longer the reason. The reason now is that **no protocol establishes the
  fabric-versus-local discrimination** those values would have to feed:
  ADR-0018's evidence obligation (2), "fabric-versus-local transport
  discrimination rows per platform for each listed local transport", is
  outstanding on every platform, and a mapping from interface strings to
  the closed positive-local list could otherwise come only from vendor
  documentation, which this package's evidence rule forbids. So the
  transport answer stays `Unrecognized` for every device, which ADR-0018
  prices knowingly and which resolves to `Indeterminate` at the closure,
  never `Permitted`. **The protocol's home is decided and is not here:**
  WP-035's observability share, amended 2026-08-13, deliberately gives
  the transport-discrimination protocol row no home in that grant and
  houses it with whichever package first records a transport route
  decision. That phrase needs a reading before anyone can act on it —
  everywhere else in this repository it denotes WP-040's IPC transports,
  which never consume this row — which is issue #366, recorded in the
  obligations section of `docs/work-packages/WP-L100.md`.
- That the block class directory contains only whole devices and
  partitions. No row records its population, so the entry bound is a
  fail-closed constant rather than a measured headroom figure.

Three statements that stood here in the first version have since been
measured, and are recorded as closed rather than deleted, so that a reader
of the earlier text can find what answered it:

- **Which sysfs attribute returned the serial** — answered by **R1**, and
  the answer was that it was not the attribute this roster reads. Moved
  into §2's first note, where it now bears on the roster itself.
- **That a whole device positively lacks the `partition` attribute** —
  answered by **FR3**: the read fails `ENOENT` on the whole device with the
  attribute absent from the directory listing (a measured absence, the
  `ObservedAbsent` shape) and reads `1` on the partition, both byte-stable.
  The admission rule this adapter uses now rests on a qualifying
  measurement rather than on instrument code inside a non-qualifying
  record. It remains written fail-closed, which is unchanged and
  independent of the measurement: an unreadable attribute admits nothing.
- **`removable` and a real-hardware `physical_block_size`** — answered by
  **FR1** and **FR2**, and moved into §2's table.
