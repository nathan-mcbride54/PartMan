# Unprivileged observability

- Spec version: 3.1.0
- Requirement IDs: SAFE-002, SAFE-003, HLP-002, MODEL-005, INV-002, INV-003
- Status: **Windows established. macOS and Linux not established.**

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

**Not established.** Needs: whether `/dev/sdX` is readable by the invoking user
(it is `brw-rw---- root:disk` on stock Debian, Ubuntu, and Fedora, so this varies
with `disk` group membership and is therefore a per-user answer on one host —
which is itself a problem, because two users on one machine must not produce
different bodies), which of `/sys/class/block/*` and the udev database carry the
partition list, and what `mmcblk*/device/cid` exposes for a native SD host versus
a USB reader.

Note that Linux is the one platform where the answer can differ between two users
of the same machine running the same build. Whatever the projection turns out to
be, it has to be a **clamping** obligation on the client — deliberately ignoring
what a privileged user happens to be able to see — not merely a discard
obligation on the helper.

## Reproducing this

The Windows facts above come from read-only CIM queries against
`root/Microsoft/Windows/Storage` plus one read-only `CreateFile` attempt on a
physical-drive path. No device layout, serial, or unique id from the measured
machine is recorded here; only whether each property was present and readable,
per SEC-006's redaction posture.
