# Unprivileged observability

- Spec version: 3.1.0
- Requirement IDs: SAFE-002, SAFE-003, HLP-002, MODEL-005, INV-002, INV-003
- Status: **Windows established. Linux partly established (one distro, virtual
  disks, no partitions). macOS not established.**

## Why this document exists

SAFE-002 requires the GUI, CLI, and discovery layer to run without elevation,
while HLP-002 has the privileged helper independently re-discover topology. Both
produce an identity record, and under ADR-C2 identity strength is **body**
content — so both must produce the *same* body for the same hardware, or PLAN-006
can never pass.

That only works if every input to a body field is readable by the unprivileged
side. A field the client cannot observe is a field the two sides will disagree
about on unchanged hardware, which is the failure ADR-C2 exists to prevent.

Two design rounds were rejected partly for asserting an unprivileged projection
nobody established (Parts 5 and 6 of `docs/spec-issues/README.md`), so this
document records only what was measured, and says so where nothing was.

**Rule of use:** an entry marked `not established` MUST NOT be relied on by an
ADR that freezes canonical bytes.

## Method

Read-only queries only. Nothing here writes to storage, and no command in this
document may. Elevation state was asserted before each run rather than assumed.

## Windows

Established 2026-07-28 on Windows 11 Pro 26200, in a session verified
non-elevated (`WindowsPrincipal.IsInRole(Administrator)` returned `False`, and
the token carried no Administrators group). Hardware present: two NVMe SSDs,
GPT, 512e (512-byte logical, 4096-byte physical).

| Fact | Interface | Unprivileged | Notes |
| --- | --- | --- | --- |
| Serial number | `MSFT_PhysicalDisk.SerialNumber` | **Yes** | Non-empty on both NVMe devices |
| Stable unique id | `MSFT_PhysicalDisk.UniqueId` + `UniqueIdFormat` | **Yes** | Format 8 (SCSI Name String) on NVMe |
| Total bytes | `MSFT_PhysicalDisk.Size` | **Yes** | |
| Logical + physical sector size | `MSFT_PhysicalDisk`, `MSFT_Disk` | **Yes** | Both classes agree |
| Bus type | `MSFT_PhysicalDisk.BusType` | **Yes** | Distinguishes USB (7), SD (12), MMC (13), NVMe (17), Storage Spaces (16), virtual (14/15) |
| Media type, removability | `MSFT_PhysicalDisk.MediaType` | **Yes** | |
| Read-only, system, boot role | `MSFT_Disk.IsReadOnly/IsSystem/IsBoot` | **Yes** | |
| Partition style (GPT/MBR) | `MSFT_Disk.PartitionStyle` | **Yes** | |
| Disk GUID | `MSFT_Disk.Guid` | **Yes** | A GPT header field, exposed through the API |
| Partition list: offset, size, type, GUID, flags | `MSFT_Partition` | **Yes** | Complete for every partition on every disk |
| Storage Spaces pool membership | `MSFT_StoragePoolToPhysicalDisk` | **Yes** | Independently confirms the round-two finding |
| **Raw partition-table bytes** | `CreateFile("\\\\.\\PhysicalDrive0")`, read-only | **No** | `ERROR_ACCESS_DENIED`, consistent with the documented Administrator requirement for physical-drive handles |

### The consequence that matters

**An unprivileged Windows client cannot read the bytes of a partition table.**
It can read the table's entire logical content — disk GUID, partition style, and
every partition's offset, size, type, and GUID — through the storage API, but not
the sectors those values were parsed from.

SAFE-003 requires a partition-table state distinguishing `Present { checksum }`,
`Absent`, and `Indeterminate`, and ADR-C3 makes a positively determined state a
condition of Strong identity. **ADR-C3 never says what the checksum is computed
over**, and the two available readings are not equivalent:

- *Checksum over raw table sectors.* Not computable unprivileged on Windows, so
  every record a client produces would be `Indeterminate` and therefore **Weak**.
  Every destructive whole-device operation on every Windows machine would demand
  typed device-name confirmation (UI-009), and unattended apply would be refused
  everywhere (SAFE-003). Worse, the helper *can* read the sectors, so client and
  helper would disagree on a body field for unchanged hardware — the PLAN-006
  failure ADR-C2 was written to prevent.
- *Checksum over the kernel-exposed table content*, canonically encoded. Readable
  by both sides on Windows, and it still serves the purpose SAFE-003's replug
  clause names, which is detecting that the table changed.

This repository does not decide that here. It is filed as a required amendment to
ADR-C3 in Part 6 of `docs/spec-issues/README.md`, and this document supplies the
measurement that makes it unavoidable rather than theoretical.

### Bearing on SI-28

`MSFT_PhysicalDisk.BusType` distinguishes a native SD host (12) and MMC (13) from
USB mass storage (7). That is an observable signal, and it is *not* the same
question as SI-28's: bus type says how the device is attached, not whether the
serial it reports belongs to the medium or to the enclosure. A USB flash drive
(bus 7, medium soldered in) and a USB card reader (bus 7, medium removable)
report the same bus type and need opposite answers.

No card reader or removable medium was attached during this run, so **SI-28's
central claim is not established by this document.** What a reader reports with
and without a card inserted, and whether it differs between two cards, must be
measured on hardware before any ADR relies on it.

## macOS

**Not established.** Needs: IOKit / `IOMedia` property availability without
elevation, `diskutil info -plist` fields for an APFS container and its physical
stores, whether raw `/dev/rdiskN` reads require elevation, and what a Fusion
container reports when one store is absent.

## Linux

**Partly established** 2026-07-28 on Debian under WSL2, kernel 6.6.114.1,
systemd as PID 1 with udev running, as a normal user in groups
`adm cdrom sudo dip plugdev users` — **not** `disk`, which is stock Debian.

Read the scope limits below before relying on any row.

| Fact | Interface | Unprivileged | Notes |
| --- | --- | --- | --- |
| Device-node permissions | `/dev/sd*` | n/a | `brw-rw---- root:disk`, and the default user is not in `disk` |
| **Raw sector read** | `dd if=/dev/sdX bs=512 count=1` | **No** | Denied, as on Windows |
| **Direct signature probe** | `blkid -p /dev/sdX` | **No** | `Permission denied` |
| Kernel block-device list | `/proc/partitions` | **Yes** | `-r--r--r--` |
| Device list | `/sys/class/block/` | **Yes** | |
| Total size, read-only flag | `/sys/class/block/*/size`, `/ro` | **Yes** | |
| Logical + physical sector size | `/sys/class/block/*/queue/*_block_size` | **Yes** | |
| Vendor, model, WWID | `/sys/class/block/*/device/{vendor,model,wwid}` | **Yes** | SCSI VPD pages 0x83/0xb0-b2 present as files |
| Serial, WWN, bus, path | `/run/udev/data/b<major>:<minor>` | **Yes** | Directory `drwxr-xr-x`, entries `-rw-r--r--`; carries `ID_SERIAL`, `ID_SERIAL_SHORT`, `ID_WWN`, `ID_WWN_WITH_EXTENSION`, `ID_BUS`, `ID_PATH` |
| File-system type and UUID | udev database, e.g. via `lsblk -f` | **Yes** | **Cached, not probed — see below** |
| `blkid` with no arguments | cache file | n/a | Returned nothing; neither `/run/blkid/` nor `/etc/blkid.tab` existed |

This confirms, rather than assumes, two claims the earlier rounds asserted:
`/dev/sdX` really is `brw-rw---- root:disk` with the default user outside the
`disk` group, and the udev database really is world-readable.

### The finding: an unprivileged client's view of signatures is second-hand

A direct probe is denied, but the udev database is world-readable and already
holds the answer udev's own probe produced. So an unprivileged client does get
file-system and signature type — **but it is a cached value that root's `udevd`
computed at device-add time, not something the client observed.** The privileged
helper, by contrast, can probe the device directly.

That matters because FS-004 signature detection is what materializes
`BackingSignature` nodes in the proposed model, and those nodes feed the
protection verdict, which is **body** content.

**Established: the two interfaces have different arities.** On synthetic image
files (SAFE-001 permits these), `blkid -p -o udev` — the form udev's builtin
uses — returns exactly one `ID_FS_TYPE`, while `wipefs -n` prints a table of
signatures with an offset per row. So the udev-cached view a client reads is
single-valued *by construction*, and the enumerating interface is one the client
cannot reach, because probing is denied.

**Not established: whether a real device ever carries two signatures at once.**
Two attempts to construct one both failed, each for an instructive reason:

1. Writing a bare `LABELONE` magic at offset 512 produced nothing. libblkid
   validates an LVM2 label beyond its magic string, so this tested the forgery,
   not the prober.
2. `mkfs.ext4` followed by `mkswap -f` left **only** the swap signature —
   `wipefs -n` showed one row at `0xff6` and the ext4 superblock at `0x438` was
   gone. Current util-linux and e2fsprogs tools erase competing signatures when
   they format.

That second result cuts against part of round three's collision-family list: a
partition reformatted by a current tool does **not** keep its old file-system
signature. The cases that plausibly survive are all *end-of-device* metadata,
which start-of-device formatting never reaches — ZFS labels written at both ends
where repurposing clears only the leading pair (round two's own observation),
mdraid 0.90 and 1.0 superblocks near the end, and a GPT backup header after an
image restore onto a larger disk. None of those was tested here.

**A qualification round two did not test.** Part 5 concluded that every fact
asymmetric between client and helper is a *roster-identity* fact, and that no
protection verdict needs roster identity. The arity difference above is not
roster identity, and signature presence does feed a verdict. Whether that
difference ever produces two different bodies for one unchanged device is exactly
what remains unestablished, so Part 5's conclusion needs re-checking against it
rather than either discarding or assuming.

### Scope limits — what this run does NOT establish

- **The disks were WSL2 virtual SCSI disks.** Real NVMe, SATA, USB, and SD/MMC
  device trees expose different files under `device/`; in particular `serial` was
  absent here and `wwid` present, which is a SCSI-transport property.
- **No partitions existed on any device**, so `ID_PART_ENTRY_*` and
  `/sys/class/block/*/start` were not observed. The Linux equivalent of the
  Windows partition-list finding is therefore **not** established, and it is the
  row that decides the ADR-C3 checksum amendment on this platform.
- **No removable medium and no card reader**, so SI-28 is untouched here, and
  `mmcblk*/device/cid` was not observable.
- `ID_FS_AMBIVALENT` was never observed, because no multi-signature medium could
  be constructed with current tools; see the two failed attempts above. What udev
  records for a genuinely ambivalent device is still unmeasured.
- One distribution, one kernel, one user. Debian's default group set is the
  claim being generalized; Ubuntu, Fedora, and Arch were not measured.

### The per-user problem, now concrete

Linux is the one platform where the answer differs between two users of the same
machine running the same build: adding a user to `disk` grants both raw reads and
`blkid -p`. Whatever projection is chosen, it must be a **clamping** obligation on
the client — deliberately declining to look at what a privileged user happens to
be able to see — not merely a discard obligation on the helper. Otherwise the
same build produces different bodies for two users on one host, and PLAN-006
fails for one of them.

## Reproducing this

The Windows facts above come from read-only CIM queries against
`root/Microsoft/Windows/Storage` plus one read-only `CreateFile` attempt on a
physical-drive path. No device layout, serial, or unique id from the measured
machine is recorded here; only whether each property was present and readable,
per SEC-006's redaction posture.
