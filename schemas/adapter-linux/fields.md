# The Linux contract's field roster and its evidence

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` §7.1 (INV-001,
  INV-002) and MODEL-004
- Owner: WP-L100 (`docs/work-packages/WP-L100.md`), increment 2
- Decided semantics carried: ADR-C4 (a positively observed absence is a
  value, not an unavailability), ADR-0018 (the device-scope transport
  arm's closed positive-local list), ADR-0014 (the partition-table state
  is helper-authored, so no client emits one)
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
read through them — and the difference is load-bearing, because two
fields on the roster have no measured row behind them at all. This
document is where that gap is stated per field rather than implied away.

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
  with authorized passthrough fixture media.
- **WSL2-only** — the 2026-07-28 table, whose own scope limits record
  that real device trees expose different files under `device/`, and
  that a WSL2-only absence must not be relied on until confirmed on a
  non-WSL distro kernel.

## 2. The sysfs roster

Paths are relative to the device's directory under the block class.

| Native property | Path | Evidence | Strength |
| --- | --- | --- | --- |
| `size` | `size` | Total size row | WSL2-only; the partition-level figure is real-hardware |
| `ro` | `ro` | Read-only flag row; device `ro` read back at every real-hardware capture | real-hardware |
| `removable` | `removable` | **none** | — |
| `logical_block_size` | `queue/logical_block_size` | Sector-size row, and the non-WSL frozen projection names this one | WSL2-only, corroborated |
| `physical_block_size` | `queue/physical_block_size` | Sector-size row only; **absent from the non-WSL projection** | WSL2-only |
| `device/vendor` | `device/vendor` | Vendor/model/WWID row; vendor and model strings observed on real hardware | real-hardware |
| `device/model` | `device/model` | Vendor/model/WWID row; observed on real hardware | real-hardware |
| `device/wwid` | `device/wwid` | Present on WSL2 virtual SCSI; **positively absent** on real usb-storage | real-hardware (as an absence) |
| `device/serial` | `device/serial` | Absent on WSL2 virtual SCSI. A serial **was** observed through sysfs on real hardware and was stable across replug and reboot — but see §5: the row bundles four attributes and attributes the value to sysfs generically, so which one returned it is not transcribed | real-hardware, attribution not transcribed |

Two of these carry a recorded decision rather than a row:

- **`removable` has no observability row on any Linux host.** It is read
  anyway, and nothing is derived from it. Declining to read it would make
  one of SI-28's three named floor inputs — transport class, removability,
  identifier presence — structurally `Unavailable` for every device,
  which trades a recorded gap for a silent one. The row is filed as an
  obligation on WP-035.
- **`physical_block_size` has no real-hardware row.** The one non-WSL
  measurement of client-readable geometry names the logical size only. It
  is read on the same terms, and its row is filed too.

Note also that the `size` unit is a kernel convention (512-byte sectors)
confirmed only inside a record marked non-qualifying. This adapter
therefore reports `size` **raw and uninterpreted**. That gap is not
cosmetic: `NamingFields::PhysicalDevice` carries a required `total_bytes`,
so increment 3 cannot address a device without a byte figure, and
ADR-0033 closes derivations at exactly two — so no third may be minted
here. The unit row is filed as blocking increment 3.

## 3. The database roster

| Key | Evidence | Strength |
| --- | --- | --- |
| `ID_SERIAL` | Identifier row names the key as carried | WSL2-only |
| `ID_SERIAL_SHORT` | Identifier row names the key as carried | WSL2-only |
| `ID_WWN` | Identifier row names the key as carried | WSL2-only |
| `ID_WWN_WITH_EXTENSION` | Identifier row names the key as carried | WSL2-only |
| `ID_BUS` | Identifier row names the key as carried; **no value is recorded anywhere, on any host** | WSL2-only, key only |
| `ID_PATH` | Identifier row names the key as carried; no value recorded | WSL2-only, key only |

The real-hardware sitting read the database entry — its interface column
names the record path — but transcribed only the file-system and
signature keys, not these. So the missing rows are a **transcription gap
in a completed capture**, not an untaken measurement, and the obligation
filed on WP-035 says so: it is materially cheaper than a new sitting.

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
  strength. Each is either another increment's or another package's, and
  the two that are nobody's yet are filed rather than invented.

## 5. What the roster's evidence does not establish

- **Which sysfs attribute returned the serial observed on real hardware.**
  Corrected here rather than edited away: this document's first version
  gave `device/serial` the strength `real-hardware` on the strength of a
  row that bundles `{vendor, model, wwid, serial}` as one read set and
  attributes the observed value to "sysfs" generically. That a serial came
  from sysfs is established; that *this attribute* produced it is a
  natural reading, not a transcription. The distinction is ordinarily
  academic and is not academic here: ADR-0019 makes the choice of a single
  named source per platform a normative, hash-visible act, so a naming
  designation cannot rest on a bundled row. The 2026-08-04 transcript is
  archived with its digest recorded and holds the instrument's exact
  paths, so closing this is a readback, not a new sitting — filed as an
  obligation on WP-035 with the others.
- That a whole device positively lacks the `partition` attribute. That is
  the admission rule this adapter uses, and **no qualifying row
  establishes it** — the only trace is instrument code inside a
  non-qualifying record. The rule is written fail-closed so the unmeasured
  direction is the safe one: an unreadable attribute admits nothing.
- That any value of any key names a transport class. Nothing does, on any
  Linux host, which is why the transport answer is `Unrecognized` for
  every device.
- That the block class directory contains only whole devices and
  partitions. No row records its population, so the entry bound is a
  fail-closed constant rather than a measured headroom figure.
