# The Linux contract's field roster and its evidence

- Spec version: source of truth is `AGENT_BUILD_SPEC.md` §7.1 (INV-001,
  INV-002) and MODEL-004
- Owner: WP-L100 (`docs/work-packages/WP-L100.md`), increments 2, 3a and 4a
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
| `ID_FS_TYPE` (4b, third slice; reported, consulted by nothing) | **DR6** — `LVM2_member`, `linux_raid_member`, `crypto_LUKS`, `btrfs`, `ext4` on the respective provisioned devices, and positively **empty** on a plain disk; **L4/L10** — over a live-ext4-over-stale-mdraid host the single-answer cache reports exactly the stale `linux_raid_member` and no ext4, which is why this key enters no name, kind or standing | VM (DR); real-hardware (L-F) |
| `ID_FS_USAGE` (4b, third slice; reported, consulted by nothing) | **DR6** — `raid` on PVs and md members, `crypto` on LUKS disks, `filesystem` on Btrfs and ext4; absent on the plain disk | VM (DR) |
| `ID_FS_VERSION` (4b, third slice; reported, consulted by nothing) | **DR14** — `1.2` on every md member, `2` on both LUKS disks: the family, cache-only per member | VM (DR2) |
| `ID_PATH` | **R4** — two values for one physical unit within one sitting (`pci-0000:00:1d.7-usb-…` before a controller reattachment, `pci-0000:01:1b.0-usb-…` after), measured evidence that this key names attachment topology and not the unit | real-hardware |

Two udev-version findings from the first Arch guest (**DR18**, udev 261,
2026-08-19), recorded here because they bear on how the database's keys
are read across the two Linux tiers: (1) **the database's serial election
differs by version** — on udev 261 a QEMU virtio-scsi disk's `ID_SERIAL_SHORT`
is the SCSI device-id designator (`drive-scsi1`) and the configured unit
serial (`DR01`) sits in `ID_SCSI_SERIAL`, so `/dev/disk/by-id` reads
`scsi-0QEMU_QEMU_HARDDISK_drive-scsiN`, where udev 249 (every jammy
sitting) put the unit serial in `ID_SERIAL_SHORT` and named the link
`…_DRnn`; the kernel's own `vpd_pg80` carries `DR01` on both — which is
ADR-0034's three grounds against the database as a naming source, measured
a second way; and (2) **a blank disk's `ID_FS_TYPE` key is absent on udev
261 where jammy's DR6 measured it present and empty** — two spellings of
one absence, both a positively determined absence to this adapter (§7's
cached signature view).

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
- **Every interface outside the four the contract closed**: no kernel
  partition list, no firmware directory, no symlink farm, no D-Bus. (The
  mount and swap tables entered as the third interface with increment 4a,
  on the DR1/DR2 rows; the OS release record as the fourth with increment
  5b, on the DR16/DR18 rows — §7.) The symlink farms are excluded on measured grounds
  as well as scope: the collision row observed `by-uuid`, `by-partuuid`
  and `by-label` all collapsing silently to the last-arriving device.
- **Boot and system role.** INV-002 names it; its Linux route runs
  through mounts, swap and firmware state. The mount and swap halves are
  now read (§7); the firmware half is not, and no derivation of a *role*
  from any of them is made — that is a consumer's, on the state facts.
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
  the transport-discrimination protocol row no home in that grant, and
  since 2026-08-17 names **WP-010** as its sponsor. As accepted the clause
  housed it with "whichever package first records a transport route
  decision" — a phrase that denotes WP-040's IPC transports everywhere
  else in this repository, which never consume this row. That was issue
  #366, and the decision owner answered it by naming the sponsor rather
  than describing it by role: WP-010 owns `TransportClass` and the closure
  that reads it, and the ADR-0034-pattern designation extension the
  deferring round named in the same sentence is sponsored from that
  assignment. **This package was not a candidate**, and the reason is the
  sentence three lines above: the protocol's only source is vendor
  documentation, which this package's evidence rule forbids. The *rows* of
  evidence obligation (2) remain measurable here — two of the six
  positive-local classes already carry a real-hardware Linux measurement —
  and the *protocol* is not. They are two obligations, not one. Recorded
  in the obligations sections of `docs/work-packages/WP-010.md` and
  `docs/work-packages/WP-L100.md`.
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

## 6. The naming roster (increment 3a)

The rosters above are the **observation** rosters: every field there is
read for its own sake, published as an attributed observation, and
elected for nothing. This section records a second, disjoint roster —
the reads that feed ADR-0019 node addressing — because the two answer to
different rules and conflating them is how a value with no evidence
behind it ends up inside a hash.

Three differences hold for every row here.

- **They travel the bytes path.** ADR-0034 records that the delivered
  `read_attribute` "is therefore **not a lawful naming-input path**",
  because it validates UTF-8, refuses non-text, and strips one trailing
  newline. Naming reads go through `read_naming_source`, which does none
  of those, and a source-text guard holds that the naming module calls
  neither the text path nor its result type.
- **A miss is not a gap to be filled later.** An undesignated cell means
  the field is absent, the name is weaker, and any resulting collision
  fails closed through ADR-0019's group. Nothing here is read
  speculatively against a source the designation does not name.
- **They are not published as observations.** A naming input is consumed
  by the address derivation, not reported. Where the same underlying
  attribute is *also* an observation row it appears in §2 as well, read
  separately through the text path, and the two readings are deliberately
  not shared.

| Naming input | Path | Evidence | Strength |
| --- | --- | --- | --- |
| serial (USB-attached) | the `serial` attribute of the nearest sysfs ancestor that is a USB device node | **R1** — the only serial any qualifying Linux record has observed, `A20036CA8695D921`, stable across replug and reboot (L8) and per-unit distinct on the measured pair (L9); **FR4** resolved the traversal to `/sys/devices/…/usb10/10-1/serial` and discharged ADR-0034's evidence obligation 1 | real-hardware |
| the USB-device-node predicate | `idVendor` and `idProduct` on a candidate ancestor | **none** — see the note below | none |
| `size`, as the `total_bytes` input | `size` | **FR5** on the whole-device node: `244457472` × 512 = `blockdev --getsize64`'s `125162225664` exactly | real-hardware |
| WWN | *no path* | ADR-0034 leaves the cell **undesignated** on Linux for every attachment class; **R2** records `device/wwid`'s `ENXIO` failed read, and no WWN-class value has ever been observed on Linux | not read |

**The predicate row is the gap this increment adds, and it is stated
rather than implied.** ADR-0034's rule is structural — "the nearest
ancestor sysfs node that is a USB device node" — and **FR4** establishes
that the measured traversal *reaches* one. No row establishes what a
client may read to *recognize* one, which is a different claim and the
one the predicate makes. The contrast is ADR-0035's mmc cell, whose
structural rule **S5c** measured directly (`/sys/block/mmcblk0/device`
resolving under `mmc_host/*`), which is why that designation needs no
ancestor search at all.

The rule is therefore written the way increment 2 wrote the
`partition`-attribute admission under the same shortfall, and discharged
the same way afterwards: fail-closed, so the unmeasured direction is the
safe one. Recognition requires **both** markers to answer with a value;
an unreadable marker recognizes nothing and the walk continues; and a
device whose USB ancestor is never identified yields an absent serial and
a weaker name, never a guessed one. Nothing relies on the predicate being
right — it can only ever *lose* a name — which is what keeps the
shortfall priced. The row is filed as an obligation on WP-035, which owns
`docs/quality/observability.md`.

## 7. The state-layer and kind-marker roster (increment 4a)

Every row here rests on the **detection-rows sitting** of 2026-08-18
(`docs/quality/observability.md`, DR1–DR10; filed by this package as
gitea#1005), taken on a disposable Proxmox guest — a real host for a
kernel-interface claim, on the floor-rows precedent — by an ordinary
client with no `disk` membership and no capability. Strength is
therefore **VM (DR)**: real kernel, virtual disks; nothing here is a
claim about a medium.

| Surface | Path | Evidence | Strength |
| --- | --- | --- | --- |
| The mount table | `<procfs>/self/mountinfo` | **DR1** — readable, one line per mount, the documented shape (mount id, parent id, `major:minor`, root, mount point, options, optional fields, `-`, fs type, source, super options); keyed by `major:minor` for a whole-disk, loop and LVM mount and for the root; **a Btrfs mount's `major:minor` is anonymous** and names its member only in the source field | VM (DR) |
| The swap table | `<procfs>/swaps` | **DR2** — readable; header, then one row per active swap (path, type, size, used, priority) | VM (DR) |
| Kind markers | `dm/`, `md/`, `loop/` under the block-class node | **DR3** — present on a device-mapper node, an mdraid array, and a loop device respectively, absent on a plain disk, readable to the client. Read as directory listings; a listing that fails for any reason other than not-found leaves the kind **indeterminate**, and the node is refused rather than admitted as plain | VM (DR) |
| The device number, as a key | `dev` | Read since increment 2 to locate the database record; since 4a also the key the mount table resolves against. **DR9** — byte-equal across a mount cycle with the rest of the roster and the record | VM (DR) |

| The array's self-reported member count (4b, first slice) | `md/raid_disks` on an `md/`-marked device | **DR5** — `2` on both arrays, direct, agreeing with the database's `MD_DEVICES`; ADR-C5's self-reported count, never a count of members observed. A value that is not a decimal count is refused, never guessed | VM (DR) |
| The kernel's membership listing (4b, first slice) | `slaves/` on an `md/`-marked device | **DR4** — names the array's members as the kernel reports them; a **per-mapping** relation (an LV over a two-PV VG listed one PV), so it is reported as an observation and never turned into an edge | VM (DR) |

| The mdraid designator (4b, second slice) | `md/uuid` on an `md/`-marked device | **DR11** — present on the measured kernel, client-readable, byte-equal across re-assembly and a reboot, distinct per array; **ADR-0053** designates it, bytes verbatim, trailing newline included, through the bytes-preserving path. The udev cache's `MD_UUID` (the same bits, colon-quartet) is not read for naming | VM (DR2) |
| The dm classification (4b, second slice) | `dm/uuid` on a `dm/`-marked device | **DR3** — `LVM-` on a logical volume, `CRYPT-LUKS2-` on an opened container; read as a classification input, never a name; the 32 bytes after `LVM-` partition logical volumes into volume-group classes and enter no name (ADR-0053) | VM (DR) |
| The LVM logical-volume name (4b, second slice) | `dm/name` on an `LVM-`-classified dm node | **DR12** — byte-equal across `vgchange -an/-ay` and a reboot with automatic activation; **ADR-0053** designates it, verbatim. For a dm-crypt mapping the same attribute is the opener's argument (DR12) and is **not** a name | VM (DR2) |
| The loop backing path (4b, second slice; reported, no node) | `loop/backing_file` on a `loop/`-marked device | **DR7**, **DR13** — the attached path verbatim; two loops on one file report equal bytes; **ADR-0053** designates it for the `BackingExtent` 3b's host node will let a loop have. By-name evidence on #94's terms | VM (DR, DR2) |
| The Section 9 floor's distribution conjunct (5b) | `os-release` under the OS-release root — **the fourth interface** — keys `ID` and `VERSION_ID`, read through the bounded record seam, one pair of double quotes stripped where the file carries them | **DR16** (jammy: a client-readable symlink to `/usr/lib/os-release`, `ID=ubuntu` unquoted, `VERSION_ID="22.04"` quoted, `ID_LIKE=debian`, byte-equal across the pinned reboot), **DR18** (the first Arch guest: `ID=arch`, `BUILD_ID=rolling`, **no** `VERSION_ID`/`ID_LIKE`, so the Arch arm reads `ID` alone). Ubuntu is compared numerically as `major.minor` against `22.04` (a later release is above the floor); Debian is **undetermined** — no Debian guest in the record, its release shape unmeasured; an unlisted `ID` is undetermined; nothing is assumed | VM (DR4, both tiers) |
| The Section 9 floor's kernel conjunct (5b) | `sys/kernel/osrelease` under the procfs root, `major.minor` parsed against `5.15` | **DR17** (`5.15.0-186-generic` plus one newline, equal to `uname -r`, equal across the pinned reboot — the acceptance environment sits exactly on the floor), **DR18** (`7.1.8-arch1-3`; not in the Arch row). A string that does not parse is undetermined | VM (DR4, both tiers) |
| The Section 9 floor's UDisks2 conjunct (5b) | **none** — no file under the four interfaces carries the daemon's version; LIN-001's route is undecided | **DR18** measured the Arch tier shipping without `udisks2` at all; every jammy acceptance guest ran with it purged. Reported `Undetermined` by construction (WP-050 increment 5) on the Debian/Ubuntu row, so every such host is `Undetermined` today — the honest answer, never a widening | VM (DR4) |
| The held standing (4b, third slice; a state-layer observation, never a name) | `holders/` on every admitted plain whole device; the holder's own `md/uuid` or `dm/uuid` as its key | **DR4**, **DR15** — live from both ends: positively empty on every member the moment its consumer is stopped, naming the consumer again after re-assembly and after a reboot; agreeing with the assembled node's `slaves/` by identity in every phase while entry names moved (`dm-0`/`dm-1`, `md126`/`md127`); the unmapped PV of an active VG, a Btrfs member, a plain disk and a live-but-unopened LUKS disk unheld. Held / unheld / undetermined; a listing that did not answer is undetermined, never unheld | VM (DR, DR3) |

What this roster deliberately does **not** carry: `/sys/fs/btrfs`,
`md/metadata_version` — measured (DR8, DR14) and waiting on 3b; and no
derived boot or system role.

**No signature node from this client, on any row.** The member-signature
offset round (`docs/reviews/LINUX_MEMBER_SIGNATURE_OFFSET_ROUND_2026-08-18.md`)
found that no client interface reports a signature's primary offset (DR14)
and that the family is client-readable per member only from the udev cache,
which reports exactly the stale signature on a stale pair (L4/L10). A
`BackingSignature`'s two naming fields are the helper's byte layer's
(ADR-0018; ADR-0019 `:252-256`), so this adapter builds no
`BackingSignature`, no `Backing` edge and no `EncryptionLayer`; they enter
the Linux inventory at HLP-002's re-discovery. `holders/` is what the client
reads instead — a state-layer fact, never a name — and DR15 measured its
liveness, so the third slice reports it (§7).
