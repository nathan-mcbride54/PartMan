# Unprivileged observability

- Spec version: 4.1.0
- Requirement IDs: SAFE-002, SAFE-003, HLP-002, MODEL-005, INV-002, INV-003
- Status: **Windows established. Linux partly established (one distro, virtual
  disks, no partitions). macOS not established.** Two of increment 5's three
  measurements were taken on 2026-08-02 and are recorded in the Windows
  section: the SI-33 media-change-counter liveness experiment, and the SI-35
  Windows partition-list measurement. The **SI-35 loop-device measurement was
  taken read-only on 2026-08-02** with repository issue #94 still open, and its
  binding gap travels with every table it filled; its decisive-pair result is a
  negative recorded under WSL2 and is not available to a register decision
  until a non-WSL distro-kernel run confirms it.

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

### Removable media — SI-28 measured, and confirmed

Established 2026-07-28 on the same host, same non-elevated session, with a USB
SD card reader (one card) and two USB flash drives attached. Read-only; nothing
was formatted, because nothing needed to be.

**The reader's serial is the reader's, and an empty slot proves it.** The reader
enumerates as **two LUNs** — one holding the card, one with no medium at all:

| LUN | Medium present | Size | Partitions | Disk-reported serial |
| --- | --- | --- | --- | --- |
| `…&0` | No | *(none)* | 0 | `2012…5300` |
| `…&1` | Yes | 255863784960 | 1 | `2012…5300` **(identical)** |

A slot containing no medium reports the same serial as a slot holding a 256 GB
card. **The serial cannot be a property of the medium.** This is a stronger
proof than the two-card comparison SI-28 was filed on, and it needed only one
card.

The two LUNs differ solely by the trailing `&0` / `&1` in the PnP instance path —
an enumeration artifact, and Section 16 forbids accepting a device path as
identity.

So for the card, ADR-C3 computes: a stable hardware identifier is present, total
size and both sector sizes are present, and the partition-table state is
positively determined (MBR). **The record is Strong**, and any other card of the
same capacity in that slot produces a byte-identical one. SI-28 is not a
hypothesis.

Two further observations:

- **The reader reports a different serial at the storage layer than in its USB
  descriptor** (`2012…5300` versus `2015…1013`), so the value is synthesized by
  the bridge rather than passed through. Its form — a 16-digit decimal string
  resembling a timestamp — is consistent with a firmware constant, which would
  make it collide across *different readers of the same model*, not merely across
  cards. Not established; worth measuring with a second reader.
- **Windows exposes no card register at all.** `MSFT_PhysicalDisk` carries no
  property matching CID, PSN, manufacturer id, or OEM id. There is no
  medium-attributable identifier to fall back on, so on this platform the honest
  answer for a card in a reader is that none exists.

### Bearing on the identifier question

The two USB flash drives are the control case, and they produce a second finding.

Both are the same model and **exactly the same capacity** (125155860480 bytes,
512/512), so they are the population SI-27 calls indistinguishable — except that
both report distinct serials, so they are in fact distinguishable, and Strong.

But each device offers **two different identifier strings from two layers**: the
storage-layer `SerialNumber` (12 characters) and the USB descriptor serial in the
`USBSTOR` instance path (16 hex characters). They are not the same value for the
same device. A plan binding one and a re-probe reading the other would not match.
That is Part 6 item 4's canonicalization gap with a concrete instance: the
question is not only how to normalize a serial, but **which** serial is the
bound one.

`MSFT_PhysicalDisk.BusType` does distinguish a native SD host (12) and MMC (13)
from USB mass storage (7). It does not help here: the reader and the flash drives
are all bus type 7 and need opposite answers.

### SI-35 on Windows: the partition-list interface against the table states — measured 2026-08-02

**Status: taken 2026-08-02.** Protocol and results both recorded under spec
version 4.2.0 by WP-035 — the header's spec-version line describes the
earlier established measurements, not this subsection.

**Headline: no unprivileged surface separated an ambiguous, silently
recovered, or inconsistent table from a healthy one.** W-H1, W-H2 and W-H3
were each refuted on this build. Two fixtures could not be measured through
`MSFT_Disk` at all, for a reason that is itself a finding and is recorded
below rather than as an absence. This
subsection extends this file's rule of use to its own vocabulary: an entry
marked `not yet taken` MUST NOT be relied on, cited, or paraphrased as a
finding — by anything, not only by an ADR that freezes canonical bytes.
Executing this protocol and filling the tables is what M0.5's exit criterion
means by the Windows partition-list measurement being "taken and recorded";
the protocol's existence satisfies nothing.

#### The question

SI-35's register entry requires, before any of its options is accepted, "the
same measurement on Windows, whose partition-list interface exposes per-
partition detail that Linux's whole-disk probe does not, and may therefore
separate the states where Linux cannot."

What the established Windows table above supports is narrower than that hope:
unprivileged CIM against `root/Microsoft/Windows/Storage` read the complete
logical partition list — on two real, healthy, GPT-partitioned NVMe disks.
Nothing yet establishes what those same classes report when the storage stack
meets a conflicting, damaged, hybrid, or blank table, nor whether the
established "Yes" rows even hold for a virtual disk attached from a file.
Both transfers are exactly the kind of assumption this document exists to
replace with a measurement.

One distinction governs the whole experiment, because it is where an executed
run would be easiest to over-read. The Linux section's fixture table, below,
could ask "are the outputs byte-identical?" because a whole-disk probe of the
healthy and the conflicting image yields the same bytes. Windows's interface reports
partition *content*, and the conflicting fixture's two tables are not equally
distant from the healthy fixture's: its primary set is row-identical to
`gpt-basic-512`'s by construction, and only its backup set differs. Whichever
copy the stack parses, the recorded list matches one table's content —
indistinguishable from the healthy disk's if the primary is parsed, visibly
different if the backup is — and **neither outcome is separation of ADR-C3
states**. A differing list is one table's content, presented; an identical
list is the same collapse the Linux projection records, unless some status
surface says otherwise. Separation means some unprivileged surface reports
the *state* — that the disk's description is ambiguous, recovered, or
inconsistent — distinctly from a healthy disk's. The hypotheses in this
protocol are phrased against status surfaces and like-for-like comparisons so
that a content difference cannot be recorded as a state difference, and a
content match cannot be recorded as its absence.

#### The fixtures, and what each of their tables describes

The instrument is the WP-020 catalogue (`crates/fixtures/src/catalogue.rs`),
whose generator is deterministic and whose per-image SHA-256 digests land in
`tests/generated/MANIFEST`. The images are synthetic, deterministic, public
bytes: everything a fixture disk reports is recordable, per the SEC-006
posture at the bottom of this file. Interpreting an executed run requires
knowing what each on-disk table says, so that is recorded here from the
generator's source, not from any probe:

| Fixture | Primary GPT describes | Backup GPT describes | MBR describes |
| --- | --- | --- | --- |
| `blank-512.img` | *(nothing — all zeros)* | *(nothing)* | *(nothing)* |
| `mbr-basic-512.img` | — | — | `0x0C` active, LBA 2048, 2048 sectors; `0x83`, LBA 4096, 4000 sectors |
| `gpt-basic-512.img` | ESP "EFI System" LBA 2048–4095; Linux-FS "Data" LBA 4096–8158 | same (agreeing) | protective `0xEE` |
| `gpt-conflicting-tables-512.img` | same two as basic | **one** MS-basic-data "Disagreeing", LBA 2048–8158 | protective `0xEE` |
| `gpt-invalid-primary-valid-backup-512.img` | present but fails its CRC | same two as basic, valid | protective `0xEE` |
| `gpt-missing-backup-512.img` | same two as basic, valid | *(zeroed, header and entry array)* | protective `0xEE` |
| `hybrid-mbr-gpt-512.img` | same two as basic | same (agreeing) | `0xEE` LBA 1, 2047 sectors; `0x0C` LBA 2048, 2048 sectors (aliasing the ESP) |

`mbr-basic-512.img` is included as the MBR control the hybrid row needs.
`gpt-basic-4kn.img` is **excluded**, and the reason is a format fact, not a
choice: the VHD container has no logical-sector-size field — its payload is
512-byte-logical by construction — so attaching the 4Kn image through a VHD
would reproduce the "4Kn is not observable from a file at all" result
recorded in the Linux section below with extra steps, measuring the
container rather than the stack. The 4Kn
equivalent is recorded as out of reach for this protocol; see the scope
limits at the end.

Two structural facts an executed run must be read against: the derived GPT
fixtures deliberately share `gpt-basic`'s disk GUID (they are one disk in
different states), so **fixtures are attached one at a time** — attaching two
at once would measure Windows's GUID-collision handling, not table-state
observability. And whichever partition set an executed run records reveals
*which copy* the stack parsed — itself an observation, and still not
separation.

#### Setup: getting fixture bytes in front of the storage stack, read-only

Everything in this sub-part is **setup, not measurement**. It writes new
regular files in operator scratch space — never under the repository, whose
generated fixtures are already ignored by git and never committed — and it
performs one privileged attach, declared as such below. The measurement is
solely what the unprivileged interfaces report afterward.

**Step S1 — provenance.** Generate the catalogue (`cargo xtask fixtures`),
copy the fixture image to a scratch directory, and record the copy's SHA-256.
The conversion function refuses to proceed unless that digest equals the
MANIFEST's — the digest is the fixture's identity, and everything downstream
inherits it.

**Step S2 — conversion.** A raw image becomes a fixed VHD by appending one
512-byte footer; the payload bytes are untouched, which is why this container
was chosen: the attached bytes are *the fixture's bytes plus a footer and
nothing else*, provable by hashing. The footer layout follows Microsoft's VHD
Image Format Specification, cross-checked against the libvhdi project's
independent format documentation. All multi-byte integers are big-endian.
Every field that the specification leaves free is pinned, so the converted
VHD is byte-deterministic and its digest is comparable across operators and
machines:

| Offset | Size | Field | Pinned value |
| --- | --- | --- | --- |
| 0 | 8 | Cookie | `conectix` |
| 8 | 4 | Features | `0x00000002` (the reserved bit, which the spec requires always set) |
| 12 | 4 | Format version | `0x00010000` |
| 16 | 8 | Data offset | `0xFFFFFFFFFFFFFFFF` (fixed disk: none) |
| 24 | 4 | Timestamp | `0` = 2000-01-01T00:00:00Z — determinism chosen over plausibility; the field is informational |
| 28 | 4 | Creator application | `pman` |
| 32 | 4 | Creator version | `0x00010000` |
| 36 | 4 | Creator host OS | `Wi2k` |
| 40 | 8 | Original size | payload length in bytes |
| 48 | 8 | Current size | payload length in bytes |
| 56 | 4 | Geometry C/H/S | the spec appendix's CHS algorithm (4 MiB → 120/4/17); advisory — the size fields are authoritative |
| 60 | 4 | Disk type | `2` = fixed |
| 64 | 4 | Checksum | one's complement of the footer's byte sum with this field zeroed |
| 68 | 16 | Unique id | first 16 bytes of SHA-256(`PartMan VHD footer: <file name>`) |
| 84 | 1 | Saved state | `0` |
| 85 | 427 | Reserved | zeros |

The conversion script, for copy-paste reproduction. It refuses a digest
mismatch, verifies its own output by reading it back (length, payload hash,
cookie, fixed-disk sentinel, size field, checksum), and emits the digests the
run record needs.

**Script provenance, in two stages.** *Before* the run, on 2026-08-02, the
three scripts that can execute without an attach were run to prove they work
as pasted: this conversion against a synthetic 4 MiB payload (it parses,
refuses a wrong digest, converts deterministically, and computes the pinned
120/4/17 geometry), and the measurement query and the layout probe in a
non-elevated console. Those pre-run executions attached no fixture and took no
measurement; the layout probe's exercised one zero-access open of a real fixed
disk, readability only, and filled no cell. *Then* the experiment itself ran:
S3 and S4 were executed for seven fixtures across two sittings, every attach
succeeded and **every post-detach digest matched**, and W3's control row was
taken during that run so its answer shares a session and build with the
fixture rows.

```powershell
# Conversion: raw fixture image -> fixed VHD, in operator scratch space.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-VhdGeometry {
    param([Parameter(Mandatory)][uint64] $SizeBytes)
    # The CHS algorithm from the VHD specification's appendix. Integer
    # division throughout; exact at fixture scale (4 MiB).
    $total = [uint64][math]::Floor($SizeBytes / 512)
    $max = [uint64]65535 * 16 * 255
    if ($total -gt $max) { $total = $max }
    if ($total -ge [uint64]65535 * 16 * 63) {
        $spt = [uint64]255; $heads = [uint64]16
        $cth = [uint64][math]::Floor($total / $spt)
    } else {
        $spt = [uint64]17
        $cth = [uint64][math]::Floor($total / $spt)
        $heads = [uint64][math]::Floor(($cth + 1023) / 1024)
        if ($heads -lt 4) { $heads = [uint64]4 }
        if ($cth -ge ($heads * 1024) -or $heads -gt 16) {
            $spt = [uint64]31; $heads = [uint64]16
            $cth = [uint64][math]::Floor($total / $spt)
        }
        if ($cth -ge ($heads * 1024)) {
            $spt = [uint64]63; $heads = [uint64]16
            $cth = [uint64][math]::Floor($total / $spt)
        }
    }
    [pscustomobject]@{
        Cylinders       = [uint16][math]::Floor($cth / $heads)
        Heads           = [byte]$heads
        SectorsPerTrack = [byte]$spt
    }
}

function Write-BigEndian {
    param([byte[]] $Buffer, [int] $Offset, [uint64] $Value, [int] $Count)
    for ($i = $Count - 1; $i -ge 0; $i--) {
        $Buffer[$Offset + $i] = [byte]($Value -band 0xFF)
        $Value = $Value -shr 8
    }
}

function New-FixedVhdFooter {
    param(
        [Parameter(Mandatory)][uint64] $SizeBytes,
        [Parameter(Mandatory)][byte[]] $UniqueId
    )
    if ($UniqueId.Count -ne 16) { throw 'UniqueId must be exactly 16 bytes' }
    if ($SizeBytes -eq 0 -or $SizeBytes % 512 -ne 0) {
        throw 'size must be a positive whole number of 512-byte sectors'
    }
    $footer = [byte[]]::new(512)
    $ascii = [System.Text.Encoding]::ASCII
    $ascii.GetBytes('conectix').CopyTo($footer, 0)          # Cookie
    Write-BigEndian $footer  8 2 4                          # Features: reserved bit, always set
    Write-BigEndian $footer 12 0x00010000 4                 # File format version 1.0
    Write-BigEndian $footer 16 ([uint64]::MaxValue) 8       # Data offset: none, fixed disk
    # ([uint64]::MaxValue, not a hex literal: PowerShell parses
    # 0xFFFFFFFFFFFFFFFF as the signed value -1, which does not convert.)
    Write-BigEndian $footer 24 0 4                          # Timestamp: pinned, 2000-01-01T00:00:00Z
    $ascii.GetBytes('pman').CopyTo($footer, 28)             # Creator application, pinned
    Write-BigEndian $footer 32 0x00010000 4                 # Creator version, pinned
    $ascii.GetBytes('Wi2k').CopyTo($footer, 36)             # Creator host OS
    Write-BigEndian $footer 40 $SizeBytes 8                 # Original size
    Write-BigEndian $footer 48 $SizeBytes 8                 # Current size
    $geometry = Get-VhdGeometry -SizeBytes $SizeBytes
    Write-BigEndian $footer 56 $geometry.Cylinders 2
    $footer[58] = $geometry.Heads
    $footer[59] = $geometry.SectorsPerTrack
    Write-BigEndian $footer 60 2 4                          # Disk type: 2, fixed
    $UniqueId.CopyTo($footer, 68)                           # Unique id, deterministic
    $footer[84] = 0                                         # Saved state
    # Checksum: one's complement of the byte sum with the checksum field
    # zeroed — it is still zero here, so sum now. 4294967295 in decimal: the
    # hex literal 0xFFFFFFFF is a signed Int32 -1 in PowerShell.
    $sum = [int64]0
    foreach ($b in $footer) { $sum += $b }
    Write-BigEndian $footer 64 ([uint64]((-bnot $sum) -band 4294967295)) 4
    ,$footer
}

function ConvertTo-FixedVhd {
    param(
        [Parameter(Mandatory)][string] $RawImagePath,
        [Parameter(Mandatory)][string] $VhdPath,
        [Parameter(Mandatory)][string] $ExpectedRawSha256
    )
    $raw = [System.IO.File]::ReadAllBytes($RawImagePath)
    if ($raw.LongLength -gt 256MB) { throw 'not a catalogue-sized fixture; refusing' }
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $rawDigest = ([System.BitConverter]::ToString($sha.ComputeHash($raw)) -replace '-', '').ToLowerInvariant()
    if ($rawDigest -ne $ExpectedRawSha256.ToLowerInvariant()) {
        throw "raw digest $rawDigest does not match the MANIFEST digest; refusing to convert"
    }
    $name = [System.IO.Path]::GetFileName($RawImagePath)
    $uniqueId = [byte[]]($sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes("PartMan VHD footer: $name"))[0..15])
    $footer = New-FixedVhdFooter -SizeBytes ([uint64]$raw.LongLength) -UniqueId $uniqueId
    $vhd = [byte[]]::new($raw.Length + 512)
    [System.Buffer]::BlockCopy($raw, 0, $vhd, 0, $raw.Length)
    [System.Buffer]::BlockCopy($footer, 0, $vhd, $raw.Length, 512)
    [System.IO.File]::WriteAllBytes($VhdPath, $vhd)

    # Verify by reading back what was written, not by trusting what was meant.
    $check = [System.IO.File]::ReadAllBytes($VhdPath)
    if ($check.LongLength -ne $raw.LongLength + 512) { throw 'converted file is not raw + 512' }
    $prefix = [byte[]]::new($raw.Length)
    [System.Buffer]::BlockCopy($check, 0, $prefix, 0, $raw.Length)
    $prefixDigest = ([System.BitConverter]::ToString($sha.ComputeHash($prefix)) -replace '-', '').ToLowerInvariant()
    if ($prefixDigest -ne $rawDigest) { throw 'the payload is no longer the fixture bytes' }
    $tail = [byte[]]::new(512)
    [System.Buffer]::BlockCopy($check, $raw.Length, $tail, 0, 512)
    if ([System.Text.Encoding]::ASCII.GetString($tail, 0, 8) -ne 'conectix') { throw 'footer cookie missing' }
    for ($i = 16; $i -lt 24; $i++) { if ($tail[$i] -ne 0xFF) { throw 'data-offset field is not the fixed-disk sentinel' } }
    if ($tail[60] -ne 0 -or $tail[61] -ne 0 -or $tail[62] -ne 0 -or $tail[63] -ne 2) { throw 'disk type is not fixed' }
    $sizeField = [uint64]0
    for ($i = 0; $i -lt 8; $i++) { $sizeField = ($sizeField -shl 8) -bor $tail[48 + $i] }
    if ($sizeField -ne [uint64]$raw.LongLength) { throw 'current-size field does not equal the payload length' }
    $stored = [uint64]0
    for ($i = 0; $i -lt 4; $i++) { $stored = ($stored -shl 8) -bor $tail[64 + $i] }
    $tail[64] = 0; $tail[65] = 0; $tail[66] = 0; $tail[67] = 0
    $sum = [int64]0
    foreach ($b in $tail) { $sum += $b }
    if (([uint64]((-bnot $sum) -band 4294967295)) -ne $stored) { throw 'footer checksum does not verify' }

    [pscustomobject]@{
        Name      = $name
        RawSha256 = $rawDigest
        VhdSha256 = (Get-FileHash -Algorithm SHA256 -Path $VhdPath).Hash.ToLowerInvariant()
        Length    = $check.LongLength
    }
}
```

**Step S3 — the privileged attach, declared as privileged.** Mounting a VHD
requires administrator privilege — that is Microsoft's documented behaviour
for `Mount-DiskImage`, not an inference — and the default access mode for a
VHD is **read-write**, so `-Access ReadOnly` must be explicit, never
defaulted. The attach therefore runs in an elevated console whose elevation
state is asserted and recorded, mirroring (in the opposite direction) the
non-elevation assertion the established rows above record. The elevated
console does exactly this and nothing else — no `Set-Disk`, no
`Initialize-Disk`, no onlining, no writes of any kind:

```powershell
# PRIVILEGED SETUP — elevated console. Set $vhdPath to the converted artifact.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'the attach requires an elevated console'
}
'setup elevation assertion: IsInRole(Administrator) = True'   # record this line

Mount-DiskImage -ImagePath $vhdPath -StorageType VHD -Access ReadOnly -NoDriveLetter | Out-Null
# Orientation only; the path and disk number are not recorded:
Get-DiskImage -ImagePath $vhdPath | Format-List Attached, DevicePath, Size
```

`-NoDriveLetter` reduces shell interaction with the RAW volumes these
fixtures produce (their partitions contain no file systems). Equivalent
privileged routes exist — Hyper-V's `Mount-VHD -ReadOnly`, `diskpart`'s
`attach vdisk readonly` — and an operator may substitute one, recording
which; `Mount-DiskImage` is specified because the Storage module is present
without enabling Hyper-V.

**Step S4 — detach, and prove read-only rather than assert it.** After the
unprivileged measurement, in the elevated console:

```powershell
Dismount-DiskImage -ImagePath $vhdPath | Out-Null
$after = (Get-FileHash -Algorithm SHA256 -Path $vhdPath).Hash.ToLowerInvariant()
if ($after -ne $recordedVhdSha256) {
    throw 'the VHD changed across the attach: the read-only claim failed; record this'
}
'post-detach digest equals pre-attach digest'                 # record this line
```

`-Access ReadOnly` is a claim about the disk object; the digest comparison is
what makes "the fixture bytes were not altered" a measured statement instead
of a documented one. A mismatch is itself a first-class recordable outcome
about the attach mechanism, and it voids that fixture's rows.

**The custody gap, recorded beside the protocol.** Repository issue #94
records that on Linux nothing binds `/dev/loopN` to a handle the interlock
verified, and asks that any measurement taken before it closes carry the gap
beside the numbers. This protocol's Windows mechanism has the parallel gap
and records it the same way: `Mount-DiskImage` resolves a *path* at attach
time, and `Get-DiskImage`'s `ImagePath`/`Location` is by-name evidence only —
the analogue of `/sys/block/loopN/loop/backing_file`. The digest bracket
(S2's read-back before, S4's re-hash after) bounds the file's content on both
sides of the attach window; nothing in this protocol asserts the kernel's
binding *during* it. The loop-backed half of SI-35 is gated on #94; this
Windows half is not — the gate is loop-specific — but the same class of
residual is declared rather than assumed away.

#### The measurement: what the unprivileged interfaces report

The measurement runs in a **separate, ordinarily launched, non-elevated
console** — not the elevated console, and not a de-elevated child of it. Its
elevation state is asserted first and recorded, per this file's precedent:
`IsInRole(Administrator)` must return `False`, and the token's
Administrators-group membership is recorded alongside it — a filtered token
can carry the group with deny-only attributes even in a properly non-elevated
session, which is why the established rows record the two checks separately,
and so does this. The script refuses to run elevated, refuses to guess
between zero or several candidate disks, and reports the fixture disk's rows:

```powershell
# UNPRIVILEGED MEASUREMENT — ordinary console.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
$elevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
"elevation assertion: IsInRole(Administrator) = $elevated"    # record this line
if ($elevated) { throw 'this console is elevated; the unprivileged measurement is void' }
$hasAdminSid = [bool]($identity.Groups | Where-Object { $_.Value -eq 'S-1-5-32-544' })
"token carries Administrators group: $hasAdminSid"            # record this line

$ns = 'root/Microsoft/Windows/Storage'
$fixtureSize = [uint64]4MB   # every catalogue image is 4 MiB
$candidates = @(Get-CimInstance -Namespace $ns -ClassName MSFT_Disk |
    Where-Object { $_.BusType -eq 15 -and $_.Size -eq $fixtureSize })

if ($candidates.Count -eq 0) {
    'outcome: unavailable - no file-backed virtual disk of fixture size is visible from this session'
    return
}
if ($candidates.Count -gt 1) {
    'outcome: unavailable - more than one candidate disk; attach exactly one fixture at a time'
    return
}
$disk = $candidates[0]

'--- MSFT_Disk ---'
$disk | Select-Object PartitionStyle, Guid, Signature, IsOffline, OfflineReason,
    IsReadOnly, OperationalStatus, HealthStatus, BusType, Size,
    LogicalSectorSize, PhysicalSectorSize, NumberOfPartitions | Format-List

'--- MSFT_PhysicalDisk (does the virtual disk appear here at all?) ---'
$physical = @(Get-CimInstance -Namespace $ns -ClassName MSFT_PhysicalDisk |
    Where-Object { $_.BusType -eq 15 -and $_.Size -eq $fixtureSize })
"rows: $($physical.Count)"
$physical | Select-Object BusType, MediaType, HealthStatus, Size | Format-List

'--- MSFT_Partition ---'
$partitions = @(Get-CimInstance -Namespace $ns -ClassName MSFT_Partition |
    Where-Object { $_.DiskNumber -eq $disk.Number })
"rows: $($partitions.Count)"
$partitions | Sort-Object PartitionNumber | Select-Object PartitionNumber, Offset, Size,
    MbrType, GptType, Guid, IsActive, IsHidden, IsReadOnly, IsOffline | Format-List
```

The selection predicate — bus type 15 (file-backed virtual) and the catalogue
size — exists for redaction as much as identification: it keeps every other
disk's values out of the recorded output entirely.

**The WinAPI row.** If CIM collapses states, the layer beneath it gets one
row before the collapse is recorded as Windows's answer.
`IOCTL_DISK_GET_DRIVE_LAYOUT_EX` is defined with `FILE_ANY_ACCESS`
(`CTL_CODE(0x7, 0x0014, METHOD_BUFFERED, FILE_ANY_ACCESS)` = `0x00070050`),
and `CreateFileW` documents that a zero-access open permits device-attribute
queries without read access — so the *documented* gate is the open and the
driver's own checks, not the ACCESS_MASK. What is established above is only
that a `GENERIC_READ` open of a physical drive is denied unprivileged. Three
things are deliberately **to be measured, not assumed**: whether a
zero-access open of the attached virtual disk succeeds unprivileged, whether
the IOCTL then succeeds, and whether either answer differs on a real physical
disk (where only success or the error code is recordable — never content,
which is real-hardware data under SEC-006):

```powershell
# UNPRIVILEGED MEASUREMENT — layout IOCTL through a zero-access open.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class DiskLayoutProbe
{
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern IntPtr CreateFileW(string name, uint access, uint share,
        IntPtr security, uint disposition, uint flags, IntPtr template);

    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool DeviceIoControl(IntPtr handle, uint code, IntPtr inBuffer,
        uint inLength, byte[] outBuffer, uint outLength, out uint returned, IntPtr overlapped);

    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr handle);

    const uint IoctlDiskGetDriveLayoutEx = 0x00070050;

    public static string Probe(int diskNumber, bool contentRecordable)
    {
        const uint shareReadWrite = 0x1 | 0x2;
        const uint openExisting = 3;
        IntPtr handle = CreateFileW(@"\\.\PhysicalDrive" + diskNumber, 0,
            shareReadWrite, IntPtr.Zero, openExisting, 0, IntPtr.Zero);
        if (handle == (IntPtr)(-1))
            return "open(access=0) failed: Win32 error " + Marshal.GetLastWin32Error();
        try
        {
            byte[] buffer = new byte[65536];
            uint returned;
            if (!DeviceIoControl(handle, IoctlDiskGetDriveLayoutEx, IntPtr.Zero, 0,
                    buffer, (uint)buffer.Length, out returned, IntPtr.Zero))
                return "open(access=0) succeeded; ioctl failed: Win32 error "
                    + Marshal.GetLastWin32Error();
            if (!contentRecordable)
                return "open(access=0) succeeded; ioctl succeeded (content not recordable: real hardware)";
            uint style = BitConverter.ToUInt32(buffer, 0);
            uint count = BitConverter.ToUInt32(buffer, 4);
            return "open(access=0) succeeded; ioctl succeeded: PartitionStyle=" + style
                + " PartitionCount=" + count + " bytes=" + returned;
        }
        finally { CloseHandle(handle); }
    }
}
'@

# Fixture disk: pass the MSFT_Disk Number from the query above, content recordable.
# Physical control: pass a real disk's number with $false — readability only.
[DiskLayoutProbe]::Probe($fixtureDiskNumber, $true)
```

#### Recording rules

- Everything the **fixture disk** reports is recordable — the bytes are
  synthetic, deterministic, and public — including partition GUIDs, the disk
  GUID, the MBR signature, and the raw and converted digests from S1/S2.
- From **every other disk** enumerated in passing, and from the physical
  control row: presence and readability only, never values. This is the same
  posture as the established table above.
- **No operator paths, usernames, drive letters, or disk numbers** appear in
  the record. Errors are recorded as Win32/HRESULT codes plus a message with
  any embedded path elided.
- The record carries: run date, OS edition and build, both elevation
  assertions (setup `True`, measurement `False` with the group check), the
  per-fixture digest pairs, the post-detach digest verdicts, and an incident
  log (every dialog or prompt the OS raised, each cancelled, each recorded).

#### Recording format

Result-cell vocabulary, from ADR-C4 as WP-035 binds this package to it:
`observed(<value>)` — including `observed(absent)`, because positively
observed absence is a value; `unavailable(<reason>)` — the interface could
not be asked; `failed(<error>)` — it was asked and errored. Before execution
every cell is `not yet taken`, which is none of the three: it means the
experiment has not run, and a blank cell is not a permitted value.

**Table W1 — disk-level surfaces, one fixture attached at a time.**

Taken 2026-08-02, non-elevated (`IsInRole(Administrator)` `False`, no
Administrators group), build 10.0.26200.0, one fixture attached at a time,
every post-detach digest **unchanged** so the read-only attach altered no
fixture's bytes. Values from the fixture disks are recorded verbatim: the
bytes are synthetic, deterministic and public.

| Fixture | ADR-C3 state | Visible unpriv. | `PartitionStyle` | `Guid` | `Signature` | `IsOffline` / `OfflineReason` | `OperationalStatus` / `HealthStatus` | Partition rows (count) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `mbr-basic-512` | Present (MBR control) | `observed(absent from MSFT_Disk)` — see the enumeration gap below | `unavailable(no MSFT_Disk row)` | `unavailable` | `unavailable` | `unavailable` | `unavailable` | `unavailable` |
| `blank-512` | Absent | `observed(present)` | `0` (unknown/uninitialized) | `''` | `''` | `False` / `0` | `53264` / `0` | `0` |
| `gpt-basic-512` | Present | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` | `''` | `False` / `0` | `53264` / `0` | `2` |
| `gpt-conflicting-tables-512` | **Indeterminate** | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` — **identical to basic's** | `''` | `False` / `0` — **identical** | `53264` / `0` — **identical** | `2` — **identical** |
| `gpt-invalid-primary-valid-backup-512` | Present, recovered | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` | `''` | `False` / `0` — **identical** | `53264` / `0` — **identical** | `2` — **identical** |
| `gpt-missing-backup-512` | Present, inconsistent | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` | `''` | `False` / `0` — **identical** | `53264` / `0` — **identical** | `2` — **identical** |
| `hybrid-mbr-gpt-512` | Present, hybrid | `observed(absent from MSFT_Disk)` — see the enumeration gap below | `unavailable(no MSFT_Disk row)` | `unavailable` | `unavailable` | `unavailable` | `unavailable` | `unavailable` |

The shared disk GUID across the four GPT fixtures is **by construction** —
they are one disk in different states — and is not a finding. What is a
finding is that every other cell is identical too.

`PartitionStyle`'s documented value space is 0 (unknown/uninitialized),
1 (MBR), 2 (GPT). Whatever `blank-512` records, the mapping of that value
onto ADR-C3's positively-observed `Absent` versus an unreadable unknown is a
register question the value alone does not answer; the cell records the
value, and the register argues its meaning.

**Table W2 — partition rows, and which on-disk description they match.**
"Matches" takes one of: `primary`, `backup`, `mbr`, `gpt`, `neither`,
`none`, judged against the fixture-description table above. For
`gpt-invalid-primary-valid-backup-512` both GPT copies describe the same
partitions, so row content *cannot* identify the copy read — only status
surfaces can separate that fixture — and its Matches cell says
`primary/backup (indistinguishable by content)` if rows appear.

| Fixture | Rows returned | Extents and types observed | Matches |
| --- | --- | --- | --- |
| `mbr-basic-512` | `unavailable(no MSFT_Disk row)` | — | — |
| `blank-512` | 0 | none | `none` |
| `gpt-basic-512` | 2 | offset 1048576 size 1048576 type `{c12a7328-…}` (ESP); offset 2097152 size 2080256 type `{0fc63daf-…}` (Linux FS) | `primary/backup (indistinguishable by content)` — this fixture's backup agrees with its primary, so its rows cannot identify the copy parsed either |
| `gpt-conflicting-tables-512` | 2 | **byte-identical to `gpt-basic-512`'s rows**, including both partition GUIDs | `primary` — the valid primary was parsed; the disagreeing backup is not represented anywhere |
| `gpt-invalid-primary-valid-backup-512` | 2 | identical rows | `primary/backup (indistinguishable by content)` — both copies describe the same partitions, so only a status surface could have separated this fixture, and none did |
| `gpt-missing-backup-512` | 2 | identical rows | `primary` |
| `hybrid-mbr-gpt-512` | `unavailable(no MSFT_Disk row)` | — | — |

**Table W3 — the zero-access layout IOCTL.**

| Target | `CreateFileW(access=0)` | `IOCTL_DISK_GET_DRIVE_LAYOUT_EX` | Style / count (fixtures only) |
| --- | --- | --- | --- |
| `blank-512` | succeeded | succeeded, 48 bytes | style `0` = MBR, count `0` |
| `gpt-basic-512` | succeeded | succeeded, 336 bytes | style `1` = GPT, count `2` |
| `gpt-conflicting-tables-512` | succeeded | succeeded, 336 bytes | style `1` = GPT, count `2` — identical |
| `gpt-invalid-primary-valid-backup-512` | succeeded | succeeded, 336 bytes | style `1` = GPT, count `2` — identical |
| `gpt-missing-backup-512` | succeeded | succeeded, 336 bytes | style `1` = GPT, count `2` — identical |
| `mbr-basic-512`, `hybrid-mbr-gpt-512` | not attempted — see below; a device index **was** available | — | — |
| one real physical disk (control — readability only, no content) | **succeeded** | **succeeded** | n/a — real hardware |

**The two interfaces use different enumerations for the same answer, and this
record originally misread that as a disagreement.** `winioctl.h`'s
`PARTITION_STYLE` is `MBR=0, GPT=1, RAW=2`; `MSFT_Disk.PartitionStyle` is
`Unknown=0, MBR=1, GPT=2`. So the IOCTL's `1` and CIM's `2` are both **GPT**
and the interfaces **agree**. The output sizes corroborate the GPT parse
independently: 336 bytes is a 48-byte `DRIVE_LAYOUT_INFORMATION_EX` header
(its union sized by the 40-byte GPT member) plus two 144-byte
`PARTITION_INFORMATION_EX` entries, and `blank-512`'s 48 bytes is that header
with no entries. An earlier version of this section claimed the two
interfaces reported different partitioning schemes, derived by comparing raw
integers across two enumerations without checking either. That claim was
false, is withdrawn, and the enum values are stated here so no future reader
re-derives it.

One cell in this table is a real cross-interface difference and is **not**
resolved by the enum mapping: for `blank-512`, CIM reports `0` = *unknown or
uninitialized* while the IOCTL reports `0` = *MBR*. Whether that is a
meaningful answer or merely each enum's zero value is **not established
here**; it is recorded because ADR-C3's `Absent` is exactly the state a blank
disk should map to, and two interfaces naming it differently is the register's
business, not this record's.

**The zero-access layout IOCTL is readable unprivileged**, on the fixture
disks and on a real physical disk alike — while `GENERIC_READ` on a physical
drive is denied, as the established table above records. The control row
records readability only; no content from real hardware is recorded.

#### The enumeration gap: two unprivileged interfaces disagree about whether a disk exists

`mbr-basic-512` and `hybrid-mbr-gpt-512` produced no `MSFT_Disk` row. Before
that could be recorded as anything, the obvious instrument faults were ruled
out by measurement rather than by argument:

- **The attach succeeded.** The elevated half logged `attach=ok` for both, and
  `Get-DiskImage` reported `Attached=True` with `Size=4194304` from the
  *unprivileged* session.
- **The disk existed at the device layer.** `Win32_DiskDrive` listed it at
  index 7 with the correct size, read from the same unprivileged session.
- **The selection predicate was not the cause.** The re-run dumped every
  `MSFT_Disk` row unfiltered; disks 0–6 were present and there was no row for
  the attached image at all.
- **`MSFT_Disk` was not simply stale.** The re-run's roster contained a USB
  disk that had not been present in the earlier roster — the host's device set
  changed between sittings. So the class enumerated a newly arrived device in
  that same window while still not enumerating the attached image. That also
  bounds any "reproduced under identical conditions" reading: the conditions
  were not identical, and the reproduction is of the absence, not of the
  whole host state.
- **It was not a settling race.** The first run allowed 800 ms, the re-run
  3 seconds; both produced no row. A longer wait than 3 s is untested.
- **It reproduced** — twice for each of the two fixtures, in two sittings.

So the record is not "the disk was absent". It is that **the disk was present
and one unprivileged interface could not see it**: `Win32_DiskDrive` and
`Get-DiskImage` enumerated it, `MSFT_Disk` — the class the established Windows
table above is built on, and the class this protocol measures through — did
not. `Get-DiskImage`'s own `Number` field was correspondingly empty.

This is not a privilege asymmetry. Both views were taken from the same
non-elevated session, so nothing here says a helper sees what a client cannot;
it says two interfaces available to the *same* unprivileged client disagree
about the existence of a disk.

**A correlation, offered as a correlation.** The two invisible fixtures are
exactly the two whose MBR carries a non-protective partition entry —
`mbr-basic-512` (`0x0C` and `0x83`) and `hybrid-mbr-gpt-512` (`0xEE` plus a
`0x0C` aliasing the ESP). The five that enumerated carry a protective `0xEE`
alone, or nothing at all. That is a clean split over seven fixtures and it is
**not a mechanism**: nothing here establishes why, no variant was constructed
to test it, and one build was measured. Whoever pursues it should build the
discriminating fixture rather than reason from this row.

**What it costs the measurement.** INV-003's hybrid-detection question and the
MBR control both fall in the gap: `hybrid-mbr-gpt-512` is the fixture that
would have answered which scheme Windows privileges, and `mbr-basic-512` was
its control. Both are recorded `unavailable`, and the hybrid question stays
open on this platform — as it does on Linux, where libblkid reports plain
`gpt` for the same image.

#### Hypotheses, and what refutes each

None of these is a prediction. Three hypotheses each name the observation
that would refute them, and two degraded-state questions ride along as
questions — the loop protocol's device, adopted here for the same reason —
with every legal answer, including every refutation and every negative,
recordable in the tables above.

- **W-H1, the decisive pair.** Some unprivileged surface distinguishes
  `gpt-conflicting-tables-512` from `gpt-basic-512` *as a state*. Refuted if
  every status surface in W1 (`IsOffline`/`OfflineReason`,
  `OperationalStatus`/`HealthStatus`) records equal values for the two
  fixtures and W2 shows the conflicting disk's rows are simply one table's
  content presented without complaint. **Not** refuted — and not supported —
  by the shape of the partition rows alone: matching rows mean the primary
  was parsed, differing rows mean the backup was. Which copy the stack chose
  is recorded either way; it is an answer about parsing priority, not about
  state separation.
- **W-H2, silent recovery.** Some surface flags
  `gpt-invalid-primary-valid-backup-512` as damaged or recovered. Refuted if
  its W1 row equals `gpt-basic-512`'s and its partitions appear ordinarily —
  which would be the Windows analogue of libblkid's silent backup recovery
  recorded in the Linux section below, on this build.
- **W-H3, missing backup.** Some surface flags `gpt-missing-backup-512` as
  inconsistent. Refuted the same way.
- **W-Q4, hybrid — a ride-along question, not a hypothesis.** Which scheme
  did the stack privilege for `hybrid-mbr-gpt-512`, judged against the
  `mbr-basic-512` control, and does any surface flag the aliasing? Legal
  answers: `gpt, flagged` / `gpt, unflagged` / `mbr` / `other (verbatim)` —
  whatever appears is the answer, recorded without a hypothesis to defend.
- **W-Q5, blank — a ride-along question.** How is an uninitialized disk
  represented, and is that representation distinguishable from any error or
  unknown state the other fixtures produce? Legal answers:
  `distinct representation` / `indistinguishable from <fixture> (verbatim)`
  / `other (verbatim)`.

**What the aggregate outcomes feed.** If every hypothesis is refuted — no
recorded surface separates any damaged state from healthy on this build —
then the unprivileged interface collapses `Present` and `Indeterminate` for
a file-backed virtual disk on this build, just as the Linux udev projection
does for files, and SI-35's option (b) is left with no measured platform
whose client-readable projection separates the states; that lands in SI-35's
evidence list beside the libblkid result, carrying the same qualifiers. If
any surface separates any pair, the record names the property and its
values, and whether that surface is client-readable and body-encodable
enough to carry option (b) is argued in the register, not here. Either
outcome is register evidence; neither is written in advance.

#### Outcomes, 2026-08-02

Each antecedent checked against the recorded cells, not against a reading of
them:

| Hypothesis | Outcome |
| --- | --- |
| **W-H1**, the decisive pair | **Refuted.** Every status surface in W1 — `IsOffline`/`OfflineReason`, `OperationalStatus`/`HealthStatus` — recorded equal values for `gpt-conflicting-tables-512` and `gpt-basic-512`, and W2 shows the conflicting disk's rows are one table's content presented without complaint. The valid primary was parsed; the disagreeing backup appears nowhere. Both branches the protocol named in advance were covered, and this is the matching-rows branch: the list is indistinguishable from the healthy disk's, and no status surface says otherwise |
| **W-H2**, damaged primary | **Refuted.** `gpt-invalid-primary-valid-backup-512`'s W1 row equals `gpt-basic-512`'s and its partitions appear ordinarily: no surface flags the damage. **Which copy was used is unmeasured** — both GPT copies of this fixture describe the same partitions, so row content cannot identify the copy read, and the run distinguishes "recovered from the backup" from "parsed the primary without validating its CRC" not at all. An earlier version of this row called it silent *recovery*; that attributed a mechanism the measurement cannot see, and the loop run cuts against it, since the same image materialized **zero** partitions under the kernel's parser |
| **W-H3**, missing backup | **Refuted**, the same way: identical W1 row, partitions present, nothing flagged |
| **W-Q4**, hybrid | **Not attempted**, which is weaker than unanswerable and is the honest word. The fixture produced no `MSFT_Disk` row, so the CIM route was closed — but `Win32_DiskDrive` supplied a device index for the same attached disk in the same session, and W3 establishes that the zero-access layout IOCTL is readable unprivileged at such an index. That probe would have answered which scheme the stack privileged and it was simply not run. The gap here is in the execution, not the platform |
| **W-Q5**, blank | **Answered: distinct.** `blank-512` reports `PartitionStyle=0`, no partitions, empty GUID and signature — distinguishable from every GPT fixture and from the `unavailable` rows. Whether `0` maps to ADR-C3's positively-observed `Absent` or to an unreadable unknown is a register question the value alone does not settle, exactly as the format said it would not |

**What this feeds, stated no wider than the run supports.** On this build, for
a file-backed virtual disk, the unprivileged Windows interface collapses
`Present` and `Indeterminate` onto one indistinguishable description — the
same collapse the Linux udev projection produces for files, reached through a
different interface on a different platform. SI-35's option (b) requires
establishing that some client-readable fact separates a conflicting table from
a healthy one; **neither measured platform now supplies one**, and this record
lands beside the libblkid result in SI-35's evidence list carrying its own
qualifiers: one build, one virtual-disk bus type, seven fixtures, two of them
unmeasurable. Options (a) and (c) are untouched by this run. The record lands
here; the register weighs it.

#### Non-answers, each with a defined recording

- **Setup refusal** (digest mismatch, conversion verification failure): the
  fixture's rows stay `not yet taken`; the refusal goes in the run record.
  A refused setup is not a failed measurement — the measurement never began.
- **Attach fails**: that fixture's measurement cells become
  `unavailable(attach failed: <code>)`. If the footer constants are the
  suspected cause, that is recorded as a suspicion, not a diagnosis.
- **The disk is invisible to the non-elevated session**: `Visible unpriv.`
  records `observed(absent)` and the remaining cells `unavailable(disk not
  visible unprivileged)`. This outcome is a finding in its own right — a
  client that cannot see what a helper attached is a client/helper asymmetry
  this file exists to catch — and it must not be recorded as "no separation
  observed".
- **The disk arrives offline** (the OS may keep a read-only disk offline,
  for example on a signature or GUID collision it cannot resolve by
  rewriting): record `IsOffline`/`OfflineReason` as observed; **do not
  online it** — `Set-Disk` is a mutation and on collision would attempt a
  write this protocol forbids. Whether CIM still returns partition rows for
  an offline disk is itself part of the record, not an assumption.
- **A query errors**: `failed(<code>)` in that cell; the others stand.
- **The post-detach digest mismatches**: the read-only claim failed for that
  attach; the fixture's rows are recorded but flagged as taken during an
  attach whose non-interference could not be proven, and the mismatch is
  recorded as its own finding about the mechanism.

#### Scope limits this record will carry, declared before execution

- **The disk is a file-backed virtual disk** (`BusType` 15). The established
  table above already distinguishes bus types for good reason; nothing
  measured here transfers to NVMe, SATA, USB, or SD paths, where the driver
  stack differs. What this run establishes, it establishes for the storage
  stack's handling of the *table states*, reached through the one read-only
  vehicle available.
- **One OS build.** Whether the stack surfaces a damaged or ambiguous state
  is implementation behaviour, not a specified interface — the same caveat
  the Linux section records for libblkid's priority resolution — and the
  build number in the run record bounds the claim.
- **512-byte logical sectors only.** The VHD container cannot present
  anything else; the 4Kn equivalent needs a virtual disk with 4096-byte
  logical sectors, which the VHDX format supports and VHD does not. Both
  known routes are recorded and neither is taken here: creating one with
  Hyper-V's `New-VHD -LogicalSectorSizeBytes 4096` and then writing fixture
  bytes *into the attached disk* is a block-device write this protocol's
  posture forbids; constructing a fixed VHDX container around the fixture
  bytes as a regular-file write is permitted by the posture but requires
  implementing the VHDX header, log, BAT, and metadata region, which this
  protocol does not specify. The Windows 4Kn measurement is therefore **out
  of reach for this protocol**, and saying so is the record.
- **Read-only bounds the fixture's bytes, not the host's behaviour.** The
  OS may still mount RAW volumes, record the disk and its GUIDs in its own
  databases, and raise dialogs; the incident log captures what it did. The
  digest bracket proves the file came through unaltered; it proves nothing
  else.
- **The custody residual** recorded in the setup section: the attach is
  by-path, the reported backing path is by-name evidence, and the digest
  bracket bounds content only outside the attach window.

### SI-33 media-change-counter liveness — measured 2026-08-02

Protocol recorded 2026-08-02 under spec version 4.2.0 by WP-035 and taken the
same day, in two runs on the hardware described under Apparatus.

**Status: every part of the register's liveness sequence was satisfied, but no
single driver instance carried the whole sequence — and a separate measured
property, which the falsification map did not anticipate, constrains whether
the witness can be used at all.**

The register's sequence is three-part: exchange the medium and assert the
immediate re-read moved; repeat with a sixty-second idle gap to detect
poll-driven behaviour; close and reopen the handle and assert the value
survives. Each part was satisfied — but **on two different driver instances**,
and that split is a qualification on the pass, not a footnote to it:

| Part | Where it was satisfied |
| --- | --- |
| immediate re-read moved | **instance β** (L1, three trials of three) |
| sixty-second idle repeat | **instance β** (L2, three trials of three) |
| value survives close/reopen | **instance α** (L5b, once, with the ordering deviation recorded in its cell) |

On α only one card was available, so L1 and L2 were `not runnable — single
medium`. On β the only close/reopen leg attempted (L5a) returned nothing
interpretable. Reading the three parts as one passed sequence therefore
assumes two instances of the same driver on the same reader behave alike —
reasonable, and **unmeasured**. This record does not assemble deltas across
that boundary and does not claim the sequence was executed end to end.

Two further things travel with the result and may not be dropped from any
retelling of it:

- **The ceiling this protocol declared before any data existed.** Prompt
  movement **cannot** be attributed to exchange-synchronous detection,
  because a background poll could equally have produced it. The strongest
  recordable positive is *no staleness observed under these conditions* — for
  the slot-exchange family, on one reader, on one bridge, on one build.
- **The counter is not monotone across driver instances.** A fresh instance
  reads a value the previous instance had already passed, so a later reading
  can equal an earlier one. An equality test between a plan-time and an
  apply-time reading therefore cannot establish non-interruption. The reading
  is measured; the attribution to re-enumeration rests on one timestamped
  co-occurrence plus the documented baseline, and is **not** a row in the
  falsification map. See "The reset, and why it outranks the pass".

Cells still reading `not yet taken` are unrun legs, and this section extends
this file's rule of use to that vocabulary: such a cell MUST NOT be relied on,
cited, or paraphrased as a finding — by anything, not only by an ADR that
freezes canonical bytes.

#### The sittings

All non-elevated (`IsInRole(Administrator)` `False`, token carrying no
Administrators group), Windows build 10.0.26200.0, one host. Timestamps are
the transcripts' own headers except where stated.

| Sitting | Driver instance | Legs taken | Media available |
| --- | --- | --- | --- |
| 1a, before 10:25:33 | α | H matrix, L5a, L6a — taken interactively, no transcript file; counter at its floor throughout | one card |
| 1b, from 10:25:33 | α | L3, L4, L5b, L6b, L7; L1 and L2 recorded `not runnable — single medium` | one card |
| 2, from 10:37:59 | β — a different instance | L5a re-run, L1 ×3, L2 ×3 | **two cards** |

1a and 1b are one driver instance and one continuous counter; they are
separated here only because 1a has no transcript file and its provenance
should not be overstated. The **one-card/two-card asymmetry is why the leg
table looks as it does**: every exchange leg belongs to sitting 2 and every
same-medium leg to sitting 1b.

Sittings 1 and 2 are **not** one series. The counter restarted between them,
so no delta spans that boundary and none is recorded across it.

**The reader's arrival is timestamped 10:32:09** — after sitting 1b's last
recorded device event (10:28:03, the L7 replug) and before sitting 2's first
sample (10:37:59). The ordering is therefore sourced rather than inferred; an
earlier draft of this record derived a run-2 start time *from* the arrival
timestamp and then used it to order the two, which was circular and is
recorded here as the error it was.

#### Why this experiment exists

SI-33 (`docs/spec-issues/README.md`, Part 1) proposes binding a plan to a
witness that the medium was never exchanged between plan creation and apply,
and names Windows' media-change counter as the one concrete signal anyone has
produced. The register makes liveness a precondition on any design, not a
detail: a witness that is evaluable but stale fails open in exactly the vector
it exists for — plan, swap, apply, seconds apart, within one attach session —
while the plan carries a field implying the check was made, which is worse
than no witness. Until the sequence below runs on real hardware, the witness
is a hypothesis, and this file is where the pass or the refutation must land.
M0.5's exit criteria name this experiment; it exits on recorded results, not
on this protocol existing.

#### What the interfaces are documented to do — and the one thing that is not documented

Checked against the vendor's pages and this machine's SDK headers rather than
recalled:

- `IOCTL_STORAGE_CHECK_VERIFY` (below: **V1**) and
  `IOCTL_STORAGE_CHECK_VERIFY2` (**V2**) are the same function code behind
  different access gates. `winioctl.h` 10.0.26100.0 (lines 350–351) defines
  V1 as `CTL_CODE(IOCTL_STORAGE_BASE, 0x0200, METHOD_BUFFERED,
  FILE_READ_ACCESS)` = `0x002D4800`, and V2 identically except
  `FILE_ANY_ACCESS` = `0x002D0800`. The I/O manager enforces the access class
  against the handle's granted access: V1 needs a read-access handle; V2
  passes on any handle. That gate difference is the entire reason V2 is the
  candidate witness interface and the entire reason the register worries
  about it.
- The [WDK page for V1](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddstor/ni-ntddstor-ioctl_storage_check_verify)
  documents an optional output: for disk and CD-ROM devices, a buffer of at
  least `sizeof(ULONG)` receives the media change count — a ULONG counting
  media changes "since the driver started" — filled only when the buffer was
  supplied and the request succeeds. The
  [V2 page](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddstor/ni-ntddstor-ioctl_storage_check_verify2)
  defines V2's input, output, and status as identical to V1's. Optional means
  optional: a driver that never fills the buffer is inside the documented
  contract, so a missing counter is a documented possibility this protocol
  must be able to record, not a failure of the run.
- The same status block documents that on a detected change with a mounted
  volume the request completes unsuccessfully (STATUS_VERIFY_REQUIRED) with
  zero output — **the count is withheld at exactly the moment of change** —
  and completes with a device-error status when no volume is mounted. A
  failed probe is therefore a first-class observation; the recording format
  brackets errors with a follow-up sample rather than treating them as broken
  runs. The NTSTATUS-to-Win32 mapping for these paths is not documented on
  those pages and is not asserted here; whatever `GetLastError` value appears
  is recorded as it appears.
- The [CreateFileW page](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)
  documents that a zero `dwDesiredAccess` can query device attributes without
  accessing the device, even where GENERIC_READ would have been denied; that
  direct read/write access to disks and volumes requires administrative
  privilege; and that volume opens must include write sharing. This is
  consistent with — not contradicted by — the `ERROR_ACCESS_DENIED` row in
  the Windows table above: that attempt requested read access; this
  protocol's primary handle requests none. Whether the zero-access open
  succeeds on this hardware is a matrix cell below, not an assumption.
- **Not documented anywhere consulted: whether V2's answer comes from the
  device or from state the storage class driver already holds.** The V2 page
  attributes its speed to the FILE_READ_ATTRIBUTES-style open avoiding a
  filesystem mount, and Microsoft's published
  [class-driver reference source](https://learn.microsoft.com/en-us/samples/microsoft/windows-driver-samples/classpnp-storage-class-driver-library/)
  carries its own media-change-detection machinery that probes devices on a
  timer — so the driver plausibly holds an answer it can return without
  touching hardware, which is precisely the register's staleness worry. No
  documentation consulted settles it. That is why this is an experiment and
  not a citation.

#### Method, and one asymmetry declared before any result exists

- Entirely unprivileged and entirely read-only. The medium exchange is
  physical; every probe is a query; **no privileged setup step exists in this
  protocol**, unlike its SI-35 siblings. An access-denied is a result to
  record, never an obstacle to elevate past: SAFE-002 fixes the privilege
  level at which a witness must be evaluable, so what an elevated handle
  would report is deliberately out of scope.
- Elevation state is asserted first and recorded, per this file's precedent;
  the script refuses to continue in an elevated session.
- "No intervening I/O" can only mean **no operator-initiated I/O**. The host
  cannot be quiesced: the shell, automatic mounting, and the class driver's
  own media-change machinery may touch the device at any time. The evidential
  force is therefore asymmetric, and this is declared before any result
  exists: a counter that fails to move refutes; a counter that moves promptly
  cannot be attributed to exchange-synchronous detection, because a
  background poll may have moved it. The strongest positive this protocol can
  record is "no staleness observed under these conditions".
- Each exchange leg is run three times where media permit. One trial is an
  anecdote; a lucky background poll cannot carry three.
- The roster query runs before the legs and never between designated samples
  — a storage-service refresh is itself I/O.

#### Apparatus

The hardware population is the one the SI-28 measurement above established,
addressed by role, never by model or serial:

| Label | Role |
| --- | --- |
| R-card | the reader LUN whose slot holds and exchanges the card, as `\\.\PhysicalDrive<n>` |
| R-vol | the card's mounted volume, as `\\.\<X>:`, when a drive letter exists |
| R-empty | the reader LUN observed medium-less in the SI-28 run, kept empty for this run, as `\\.\PhysicalDrive<m>` |
| F1 | one of the two identical-model USB flash drives |
| Card A, Card B | two SD media. Card B may not exist in the kit; every leg needing it then records `not runnable — single medium`, and the falsification map states what that leaves open. |

#### The script

One copy-paste block, PowerShell 7, non-elevated. Its output stays on the
operator's screen; nothing from it enters this file except deltas and
statuses, per the recording rules below.

```powershell
# SI-33 media-change-counter liveness probe. Read-only. Refuses elevation.
$ErrorActionPreference = 'Stop'

# 1. Elevation-state assertion — recorded in the run header, per this file's precedent.
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw 'Session is elevated. The experiment measures the unprivileged interface; use a normal session.'
}
'non-elevated: confirmed'                                     # record this line
$hasAdminSid = [bool]($identity.Groups | Where-Object { $_.Value -eq 'S-1-5-32-544' })
"token carries Administrators group: $hasAdminSid"            # record this line — a filtered
# token can carry the group deny-only in a properly non-elevated session, which is
# why the established rows record this check separately from IsInRole.

# 2. P/Invoke surface.
Add-Type -Namespace Si33 -Name Native -MemberDefinition @'
[DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
public static extern IntPtr CreateFileW(
    string lpFileName, uint dwDesiredAccess, uint dwShareMode,
    IntPtr lpSecurityAttributes, uint dwCreationDisposition,
    uint dwFlagsAndAttributes, IntPtr hTemplateFile);
[DllImport("kernel32.dll", SetLastError=true)]
public static extern bool DeviceIoControl(
    IntPtr hDevice, uint dwIoControlCode,
    IntPtr lpInBuffer, uint nInBufferSize,
    out uint lpOutBuffer, uint nOutBufferSize,
    out uint lpBytesReturned, IntPtr lpOverlapped);
[DllImport("kernel32.dll", SetLastError=true)]
public static extern bool CloseHandle(IntPtr hObject);
'@

# 3. Constants — verified against winioctl.h 10.0.26100.0, not recalled.
$V1 = [uint32]0x002D4800    # IOCTL_STORAGE_CHECK_VERIFY   (FILE_READ_ACCESS class)
$V2 = [uint32]0x002D0800    # IOCTL_STORAGE_CHECK_VERIFY2  (FILE_ANY_ACCESS class)
$ACCESS_ZERO = [uint32]0x00000000
$ACCESS_ATTR = [uint32]0x00000080    # FILE_READ_ATTRIBUTES
$ACCESS_READ = [uint32]2147483648    # GENERIC_READ (0x80000000; that hex literal is
                                     # Int32 -2147483648 in PowerShell and kills the cast)
$SHARE_RW    = [uint32]3             # FILE_SHARE_READ|FILE_SHARE_WRITE; volume opens require the write share
$OPEN_EXISTING = [uint32]3

function Open-DeviceHandle([string]$Path, [uint32]$Access) {
  $h = [Si33.Native]::CreateFileW($Path, $Access, $SHARE_RW, [IntPtr]::Zero,
                                  $OPEN_EXISTING, 0, [IntPtr]::Zero)
  $err = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
  [pscustomobject]@{ Path = $Path; Access = ('0x{0:X8}' -f $Access)
                     Ok = ($h -ne [IntPtr]::new(-1)); Err = $err; Handle = $h }
}

function Probe([pscustomobject]$H, [uint32]$Code) {
  $count = [uint32]0; $bytes = [uint32]0
  $ok  = [Si33.Native]::DeviceIoControl($H.Handle, $Code, [IntPtr]::Zero, 0,
                                        [ref]$count, 4, [ref]$bytes, [IntPtr]::Zero)
  $err = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
  [pscustomobject]@{
    T     = [DateTime]::Now.ToString('HH:mm:ss.fff')
    Ioctl = if ($Code -eq $V2) { 'V2' } else { 'V1' }
    Ok    = $ok
    Err   = if ($ok) { 0 } else { $err }
    Bytes = $bytes
    Count = if ($ok -and $bytes -ge 4) { $count } else { $null }
  }
}

function Sample([pscustomobject]$H)     { Probe $H $V2 }   # liveness legs use this and nothing else
function SampleBoth([pscustomobject]$H) { Probe $H $V2; Probe $H $V1 }  # H matrix only — L3's V1 arm calls Probe directly

function Watch([pscustomobject]$H, [int]$Seconds = 30, [int]$IntervalMs = 1000) {
  # Exploration only. A Watch IS intervening I/O at V2's layer: never run one
  # during a leg; any leg sample taken while a Watch ran is invalid.
  $until = [DateTime]::Now.AddSeconds($Seconds)
  while ([DateTime]::Now -lt $until) { Sample $H; Start-Sleep -Milliseconds $IntervalMs }
}

# 4. Roster — read-only CIM, run BEFORE the legs, never between samples.
#    BusType 7 = USB. The reader's LUNs are the USB disks whose Size is empty
#    (no medium) and nonzero (card present); the flash drives are the other two.
Get-CimInstance -Namespace root/Microsoft/Windows/Storage -ClassName MSFT_Disk |
  Sort-Object Number | Format-Table Number, BusType, Size, IsBoot, IsSystem -AutoSize
# If the medium-less LUN does not appear as MSFT_Disk, fall back to
# MSFT_PhysicalDisk (DeviceId, BusType, Size) to find its disk number.
```

This block was executed once on 2026-08-02, non-elevated, to prove it runs as
pasted: the elevation assertion, the type compile, the roster query, and one
zero-access V2-then-V1 marshaling probe of a fixed system disk — a subject
outside this protocol's population. That run confirmed only that the calls
marshal and that the access-class gate behaves as `winioctl.h` declares on
that one handle: V2 completed on a zero-access handle and returned a
four-byte count, and V1 on the same handle failed with `ERROR_ACCESS_DENIED`.
No cell below is filled from this paragraph — the H matrix belongs to the
removable-media subjects. This paragraph is instrument validation, not
observation: this section's rule of use extends to it, so nothing in it may
be cited as any device's gate behaviour — the H matrix, on the protocol's own
subjects, is where that answer landed.

Per-handle usage, repeated for each matrix cell:

```powershell
$h = Open-DeviceHandle '\\.\PhysicalDrive<n>' $ACCESS_ZERO; $h
if ($h.Ok) { SampleBoth $h }
# hold or close per the leg's instruction; close with:
#   [Si33.Native]::CloseHandle($h.Handle) | Out-Null
```

#### H — which handle can even ask (run first)

For each combination below: attempt the open; on success issue V2 once, then
V1 once, in that order. V2-before-V1 is a discipline, not a convenience: V1's
access class marks it as the device-reaching variant, so probing it first
could refresh the very state V2 is suspected of serving stale.

Taken 2026-08-02, non-elevated (`IsInRole(Administrator)` `False`, token
carrying no Administrators group), Windows build 10.0.26200.0, on the reader
and flash drives described under Apparatus. `F2` is an addition to the
protocol's matrix: two identical-model drives were present, and the second
costs one row.

| Subject | Path form | Access | Open | V2 | V1 |
| --- | --- | --- | --- | --- | --- |
| R-card | `\\.\PhysicalDrive<n>` | `0x00000000` | ok | ok, count returned | `error 5` |
| R-card | `\\.\PhysicalDrive<n>` | `FILE_READ_ATTRIBUTES` | ok | ok, count returned | `error 5` |
| R-card | `\\.\PhysicalDrive<n>` | `GENERIC_READ` | `open refused 5` | — | — |
| R-vol | `\\.\<X>:` | `0x00000000` | ok | ok, count returned | `error 5` |
| R-vol | `\\.\<X>:` | `FILE_READ_ATTRIBUTES` | ok | ok, count returned | `error 5` |
| R-vol | `\\.\<X>:` | `GENERIC_READ` | **ok** | ok, count returned | **ok, count returned** |
| R-empty | `\\.\PhysicalDrive<m>` | `0x00000000` | ok | `error 21`, 0 bytes | `error 5` |
| R-empty | `\\.\PhysicalDrive<m>` | `FILE_READ_ATTRIBUTES` | ok | `error 21`, 0 bytes | `error 5` |
| R-empty | `\\.\PhysicalDrive<m>` | `GENERIC_READ` | `open refused 5` | — | — |
| F1 | `\\.\PhysicalDrive<k>` | `0x00000000` | ok | ok, count returned | `error 5` |
| F2 | `\\.\PhysicalDrive<j>` | `0x00000000` | ok | ok, count returned | `error 5` |
| F1, F2 | `\\.\PhysicalDrive<…>` | `FILE_READ_ATTRIBUTES` | ok | ok, count returned | `error 5` |
| F1, F2 | `\\.\PhysicalDrive<…>` | `GENERIC_READ` | `open refused 5` | — | — |

Three results here, each narrower than it may look:

- **The zero-access design has a data source on this hardware.** V2 returned
  a four-byte count through a handle requesting no access at all, on every
  subject holding a medium, while V1 was denied on those same handles. That
  is the access-class gate `winioctl.h` declares, observed rather than cited.
- **A read-access handle exists, but only by the volume path.** `GENERIC_READ`
  was refused on every `PhysicalDrive` — consistent with the established
  `ERROR_ACCESS_DENIED` row above — and granted on the removable volume,
  where V1 then succeeded. This is what makes L3's V1 arm runnable at all.
  Whether it generalizes past a removable volume on this build is unmeasured.
- **An empty slot is not merely countless; the probe fails.** `error 21`
  (`ERROR_NOT_READY`) with zero output bytes. Recorded as L6a below.

**A side observation, recorded because the roster step's fallback assumed the
opposite.** The medium-less LUN appears in `MSFT_Disk` (size 0, partition
style 0) but has **no `MSFT_PhysicalDisk` row at all**. The protocol's roster
step anticipated needing `MSFT_PhysicalDisk` as the fallback for finding an
empty LUN; on this host that class is where the LUN is missing. One host, one
reader — the correction to the fallback is the finding, not a general claim.

**Instrument rule.** The liveness legs use the least-privileged combination
whose V2 returned a count: access `0x00000000` preferred over
`FILE_READ_ATTRIBUTES` over `GENERIC_READ`, and PhysicalDrive preferred over
volume, because the slot's disk object is what persists across a medium
exchange while a volume's lifetime is the mounted filesystem's — part of what
changes. If no V2 returns a count but some V1 does, the legs run against that
handle and the substitution is recorded — it is itself a finding (see the
falsification map). If nothing returns a count, every leg records
`not runnable — no counting handle` and counter absence is the recorded
outcome of the experiment.

#### L — the liveness legs

Global rules. A `Sample` is one V2 probe on the held instrument handle.
Between a leg's designated samples the operator performs nothing except the
stated physical action — no Explorer window on the volume, no enumeration, no
Watch. "Moved" means a nonzero delta between bracketing counts on one driver
instance. When a designated sample errors, take one immediate follow-up
`Sample` and record both: the documented change path withholds the count
while signaling the change, so the error is evidence, not noise.

**L1 — immediate exchange** (needs Card B; three trials, alternating
direction).
1. `Sample` → pre. 2. Eject the card, insert the other. Nothing else.
3. `Sample` immediately — target within two seconds of insertion.
4. `Sample` once more about five seconds later, to bracket if step 3 errored.

**L2 — sixty-second idle** (needs Card B; three trials). As L1, but after
the exchange keep hands off for sixty seconds — no probes — then take a
single `Sample`. The pairing with L1 is the point: L1 asks whether the first
immediate answer is fresh; L2 asks whether an answer becomes fresh with time
and no explicit I/O, which is the poll-driven signature the register asks
this leg to look for.

**L3 — forced-I/O control** (single-card variant permitted: use same-medium
removal as the disturbance). 1. `Sample` → pre. 2. Exchange or eject/reinsert.
3. `Sample` → record. 4a. Force filesystem I/O: `Get-ChildItem '<X>:\' |
Out-Null` — the one operator-initiated I/O this protocol permits, declared
here as its own step. 5a. `Sample`. If H produced a read-capable handle, run
a second round replacing steps 4a/5a with: 4b. `Probe $h $V1` on that handle,
so the filesystem is not a confound; 5b. `Sample`. This leg exists to
separate "the counter moved" from "the counter moved because something
finally asked the device".

**L4 — same-medium out-and-back** (single card suffices; the core continuity
leg; three trials). 1. `Sample` → pre. 2. Eject the card; wait about five
seconds. 3. Optionally `Sample` while the slot is empty — recorded under
L6b. 4. Reinsert the same card. 5. `Sample` immediately. 6. If unchanged,
`Sample` again after sixty seconds of idle. A continuity witness must move
here: SI-33's vector includes exchange to an identical-looking medium, and
the same medium returning is that vector's limiting case.

**L5 — handle close/reopen survival.**
a. Quiescent: with the instrument handle still open after a completed leg,
`Sample`; close; reopen the same path at the same access; `Sample`. Record
the delta across the boundary and its sign.
b. Across an exchange: `Sample` on the held handle; close; eject and
reinsert (or exchange); reopen; `Sample`. The witness's real use crosses
both a handle boundary and an exchange, so the movement must still be
visible from the fresh handle.

**L6 — empty-slot behaviour.**
a. R-empty: the H rows above, plus one `Sample` on any handle that opens —
recording the status and whether any count bytes come back for a slot
holding no medium for the duration of this protocol.
b. R-card while empty: the L4 step-3 sample — status and count presence with
no medium in a slot that had one.

**L7 — flash-drive surprise removal** (secondary subject; scoping, not
liveness). 1. Open F1's H row and hold the handle. 2. `Sample` → pre.
3. Confirm nothing else is using the drive, then remove it without the eject
flow — the surprise is the subject, and this protocol has nothing in flight
because it writes nothing. 4. `Sample` on the held handle → record the
status. 5. Reinsert. 6. `Sample` on the held handle → does it recover or
stay dead? 7. Open a fresh handle; `Sample`; record the delta against step 2
and its sign. Purpose: the counter counts "since the driver started", and a
whole-device detach may end the counting entity. Whether this mechanism can
address the two-identical-flash-drives population at all — SI-28's population
— is what this leg establishes, in either direction.

#### Recording format

Run header, one per execution:

| Field | Sitting 1a / 1b | Sitting 2 |
| --- | --- | --- |
| Date, OS build | 2026-08-02, 10.0.26200.0 | 2026-08-02, 10.0.26200.0 |
| Elevation assertion | `non-elevated: confirmed`; token carries Administrators group: `False` | same, both lines recorded |
| Media available | **one card** | **two cards** |
| Instrument handle chosen (per the H rule) | `\\.\PhysicalDrive<n>` at access `0x00000000` — the least-privileged combination whose V2 returned a count | same |

Result cells use a closed vocabulary, and every negative outcome has a
spelling. The vocabulary deliberately departs from the ADR-C4 spelling the
SI-35 sections use, because deltas-not-values (below) makes an
`observed(<value>)` form unusable here; the mapping is stated so the
departure cannot read as drift: `count Δ=…` and `ok, no count` are
observations (absent count bytes are a value), `error` / `open refused` /
`handle dead` are failures recorded verbatim, and `not runnable — <reason>`
is the step being unavailable.

- `count Δ=+n` — success with a count; only the delta is recorded.
- `count Δ=0` — success; count unchanged.
- `ok, no count` — success with zero output bytes (the documented
  optionality).
- `error <decimal>` — failed probe; raw `GetLastError`, annotated with the
  symbol below when it matches one.
- `open refused <decimal>` — the open itself failed.
- `handle dead <decimal>` — a previously working handle now errors (L7).
- `not runnable — <reason>` — the step could not be performed; the reason is
  part of the record. Foreseeable reasons, named now: `single medium`,
  `no mounted volume`, `no counting handle`, `device absent`.
- `not yet taken` — the experiment has not run. After the 2026-08-02 run this
  spelling means the specific leg is still unrun, not that the section is.

Instrument failure — an `Add-Type` error, a typo, a crashed shell — is not
an observation and must not occupy a cell; fix the instrument and run the leg
cleanly.

**A discarded leg, recorded so the discard is auditable.** An earlier L4
attempt on 2026-08-02 used a fixed-window script — sample, wait, sample —
that announced its timing in advance rather than prompting. The physical
action was never performed inside the window, and every sample it took
returned a present medium and an unchanged count. Read as a result it would
have said *the counter did not move across a same-medium exchange*, which is
the L4 refutation, from a leg where no exchange occurred. The empty-slot
assertion is what caught it: the mid-leg sample returned a present medium
where an ejected card must return `error 21`. Under the rule above the attempt
is instrument failure, so it occupies no cell and its numbers appear nowhere
in this record. It is named here because the assertion that caught it is now
load-bearing for every leg, and the operator scripts were rewritten to prompt
and to validate each physical step before proceeding.

Symbols for annotation, verified against `winerror.h` 10.0.26100.0:
`ERROR_ACCESS_DENIED` 5, `ERROR_NOT_READY` 21, `ERROR_UNRECOGNIZED_VOLUME`
1005, `ERROR_MEDIA_CHANGED` 1110, `ERROR_NO_MEDIA_IN_DRIVE` 1112,
`ERROR_IO_DEVICE` 1117, `ERROR_DEVICE_NOT_CONNECTED` 1167. These name codes
the tables may contain; no claim is made about which will appear. The
2026-08-02 run produced one code this list did not anticipate —
`ERROR_DEV_NOT_EXIST` 55, on a held handle after surprise removal (L7) —
added here because it appeared, which is the only reason any code belongs.

**Why deltas, never values.** The counter is a tally of media events since
the driver instance started, so an absolute value is a fragment of the real
machine's session history — operator behaviour, which is exactly the class of
real-hardware values this file's SEC-006 posture keeps out (see "Reproducing
this" below). Everything the hypothesis needs is carried by deltas between
this protocol's own designated samples, by signs (L5, L7), and by status
transitions — properties of the protocol's actions and of Windows constants,
not of the hardware or its history. Timestamps are recorded as offsets from
each leg's first sample for the same reason. Devices are recorded by role
label only.

Leg results, 2026-08-02, both runs. The exchange legs reached their three
trials; **L4 did not**, and that shortfall is carried in the table rather than
noted once and forgotten. This protocol's own reason for demanding three is
that one trial cannot exclude a background poll having moved the counter — and
the inter-sample advance recorded below is this run's own demonstration that
the counter does move unobserved, so the reason stands for L4.

| Leg | Trials asked / taken | Record | Result |
| --- | --- | --- | --- |
| L1 immediate exchange | 3 / 3 (sitting 2) | Δ across exchange at immediate re-read, per trial | `count Δ=+1` in **all three trials**, every swap fingerprint-validated as a genuinely different card. In sitting 1b: `not runnable — single medium` |
| L1 | 3 / 3 (sitting 2) | status of first post-exchange sample, per trial | success with a count in all three; the documented withheld-count path never fired, and each bracket at +5 s held the same Δ |
| L2 sixty-second idle | 3 / 3 (sitting 2) | Δ across exchange after 60 s hands-off | `count Δ=+1` in **all three trials**, each from a single sample with no probe in the preceding minute. In sitting 1b: `not runnable — single medium` |
| L3 own bracket | ≥1 / 1 | Δ step 1 (pre) vs step 3, per round — the "moved only after" antecedent's own cell | `count Δ=+1` — moved **before** any forced I/O |
| L3 forced-I/O, filesystem arm | ≥1 / 1 | Δ step 3 vs step 5a | `count Δ=0` |
| L3 forced-I/O, V1 arm | ≥1 / 1 | Δ step 3 vs step 5b, and whether a read handle existed | `count Δ=0`; a read handle existed, by the volume path, and V1 succeeded on it |
| L4 same-medium out-and-back | 3 / **1** | Δ across removal and reinsertion at immediate re-read | `count Δ=+1`, already final at the immediate re-read; `Δ=+1` unchanged at +5 s and after 60 s idle |
| L5a reopen, quiescent | ≥1 / 2 | Δ across close/reopen, with sign | `count Δ=0` both times, and **uninterpretable both times**: the counter was at its floor in sitting 1a and had returned to the floor in sitting 2, and at the floor "survived" and "reset" are indistinguishable. The sitting-2 re-run was taken precisely to escape the floor and did not. L5a carries **no information in either direction**; L5b is the only interpretable evidence for this part of the sequence. |
| L5b reopen, across a same-medium out-and-back | ≥1 / 1 | Δ visible from the fresh handle, with sign | one fresh-handle sample, `Δ=+1` versus L4's pre-removal sample, sign positive: the value the held handle had reported was still visible from a newly opened handle, so the count is **not per-handle state**. Two deviations recorded rather than smoothed: there was **no A→B exchange** — sitting 1b had one card, and the physical action was L4's same-medium out-and-back — and the leg's specified order (sample, **close**, physical action, reopen, sample) was not followed: a handle stayed open across the action and the fresh handle was opened afterwards. Survival with no handle open across the event, and survival across a true exchange, are both **unmeasured**. |
| L6a empty LUN | 1 / 1 | open and probe statuses; count bytes present or not | opens at `0x00000000` and `FILE_READ_ATTRIBUTES`; `GENERIC_READ` `open refused 5`; V2 `error 21`, **no count bytes**; V1 `error 5` |
| L6b card LUN while empty | 1 / 1 | probe status with no medium | V2 `error 21`, 0 bytes — the same answer the never-occupied LUN gives |
| L7 surprise removal | 1 / 1 | held-handle status after removal; after reinsertion; fresh-handle Δ sign | held handle `handle dead 55` after removal and **still dead after reinsertion**; fresh handle `count Δ=0` versus pre. The **counter** cannot settle the row — both readings sat at the floor, where a reset and a never-counted event are indistinguishable — but the drive's own `DEVPKEY_Device_LastArrivalDate` moved to 10:28:03 across the cycle, so the device demonstrably **re-enumerated**, and the row's question (does a whole-device detach end the counting entity?) is answered by that independent instrument rather than by the count. The drive was re-identified across the cycle by its instance id. |

**One reading between legs, recorded because it bounds the single-trial
result.** Between sitting 1a's last designated sample and sitting 1b's L4
pre-sample — both readings on instance α — the counter advanced by 2, with no
samples taken in the interval. The interval contained
operator handling of the medium, so the advance is consistent with two
handling cycles outside any bracket; the cause is **not established**, and no
sample brackets it. It is recorded because it is the run's own demonstration
that the counter moves while unobserved, which is exactly why one L4 trial is
not three. It is a within-instance advance and is unrelated to the reset
below, which is a different phenomenon in the opposite direction.

#### Falsification map — what each observation would establish

No row is a prediction. Each is a conditional, written before the fact so no
conclusion can be fitted to the data afterward:

| If observed | It establishes | It feeds back into SI-33 as |
| --- | --- | --- |
| H yields no handle whose V2 returns a count | the witness has no data source at the privilege SAFE-002 fixes, on this hardware | the only proposed mechanism loses its Windows anchor; the option space needs a different non-interruption signal before any witness field may exist |
| counts arrive only via V1 on a read-access handle | the zero-access design is dead; evaluating the witness requires read access | whether a read-access device handle is grantable non-elevated becomes the successor question — H's open cells already record the answer for this hardware |
| L1 unchanged, L2 moved | the value is refreshed by something other than the exchange, within a window this run bounds only as ≤ 60 s | a staleness window exists, and plan/swap/apply operates in seconds — inside it; a witness read is trustworthy only behind a proven wait-or-probe discipline |
| L1 and L2 unchanged, L3 moved only after forced I/O | evaluable-but-stale — the register's exact worry — confirmed | the witness as proposed converts an admitted gap into false assurance; any surviving design must couple the read to device-reaching I/O and prove the coupling |
| L1, L2, and L3 all unchanged | the counter does not witness exchange on this device at all | same terminus as row 1, reached behaviourally |
| L4 unchanged | the counter misses an interruption when the returning medium is identical — the limiting case of SI-28's population | whatever it counts, it is not interruption; not a continuity witness |
| L5 delta negative on reopen | the value does not survive the handle boundary | plan-time and apply-time reads from different handles compare across epochs; only a single held handle spanning plan→apply could use it, a constraint recorded rather than assumed satisfiable |
| L7 handle dies and the fresh-handle delta is negative | whole-device detach ends the counting entity | the mechanism scopes at most to media-in-slot exchange; the identical-flash-drives population needs something else |
| L1 moved, L2 moved, L4 moved, L5 stable in every trial taken | the cheapest refutations failed on this hardware | the register's liveness sequence — immediate re-read, sixty-second idle, close/reopen — has passed **on this reader**, with the standing qualifier that movement cannot be attributed to exchange-synchronous detection; the recordable ceiling is "no staleness observed under these conditions", for the slot-exchange family only |

If only one card is available, the rows requiring L1/L2 true exchange stay
unevaluated and this narrower question stays open with them: a counter that
moves on same-medium return but is identity-derived rather than
event-counting would behave identically under L4 alone. `not runnable —
single medium` therefore blocks any claim about A→B exchange, and the section
status cannot advance past the legs actually run.

**Which rows fired, 2026-08-02.** Every refutation row's antecedent was
checked against the recorded cells rather than against a reading of them:

| Row | Fired? |
| --- | --- |
| H yields no counting handle | **no** — V2 returned a count at zero access on every subject holding a medium |
| counts only via V1 | **no** — V2 counted at zero access; V1 was denied there |
| L1 unchanged, L2 moved | **no** — L1 moved in all three trials |
| L1 and L2 unchanged, L3 moved only after forced I/O | **no**, and the opposite is recorded: L3 moved on its own bracket **before** any forced I/O, and neither filesystem I/O nor a V1 probe moved it further |
| L1, L2, L3 all unchanged | **no** |
| L4 unchanged | **no** — L4 moved on same-medium return |
| L5 delta negative on reopen | **no** — L5b showed the post-exchange value present from a fresh handle |
| L7 handle dies and fresh-handle delta negative | **not by the counter** — the handle died and stayed dead, but both readings sat at the floor, so a reset and a never-counted event are indistinguishable there. The row's conclusion is nonetheless supported by an independent instrument: the drive's last-arrival moved across the cycle, so the detach did end the counting entity |
| **pass row**: L1, L2, L4 moved and L5 stable | **yes, with the conjunction assembled across two driver instances** — L1 and L2 on β, L4 and L5b on α, and the L5 conjunct resting on **L5b alone**, since L5a is uninterpretable in both directions. Hence the status above, with its declared ceiling and its split |

The one thing no row anticipated is the reset, and it is the reason the pass
row is not the end of the matter.

#### The reset, and why it outranks the pass

**Measured, and stated at the strength the evidence carries.** Sitting 1b
ended with the counter at a recorded value. Sitting 2's first sample, on the
same reader, read **lower** — back at its floor. The reader's
`DEVPKEY_Device_LastArrivalDate` is timestamped 10:32:09, between 1b's last
recorded device event (10:28:03) and sitting 2's first sample (10:37:59), so a
device arrival falls inside the interval that also contains the drop.

Reading the instance change from a property other than the count keeps the
inference **non-circular** — detecting a new instance *by* the count being
lower would prove nothing. Non-circular is not the same as causally
established, and the record should not trade on the difference. What the data
supports is a single timestamped co-occurrence on n=1, with the *trigger* of
that arrival nowhere recorded and no leg varying it. Attributing the reset to
the re-enumeration rests on that co-occurrence plus the documented "since the
driver started" baseline. It was not reproduced, and no boundary other than
this one was tested.

**Two re-enumerations were observed, not one.** Besides the reader's, the L7
drive's last-arrival moved to 10:28:03 across its surprise-removal cycle — a
second re-enumeration, deliberately induced, with a different cause. Only the
reader's produced an interpretable counter reading; the L7 drive's counter sat
at its floor on both sides.

**Why it matters more than the pass.** A witness exists to answer one question
at apply time: *was the medium exchanged since the plan was made?* The
proposed test is a comparison between a recorded reading and a fresh one, and
that test is sound only if the value cannot revisit a value it already held.
This one can.

**Constructed scenario, built from one measured property.** Two of its four
steps correspond to measured legs; the other two describe a design that does
not exist and were not exercised — no plan, no apply, and no equality test
appears anywhere in this run:

| Step | Status |
| --- | --- |
| a plan is made on a fresh driver instance, at the floor | **constructed** — no plan-time reading was performed |
| the medium is exchanged and the counter moves | **measured** (L1, L2, L4) |
| the device re-enumerates and the counter returns to the floor | **measured once**, as the co-occurrence above |
| apply reads the floor, compares equal, and concludes the medium never moved | **constructed** — no apply-time comparison was performed, and that the post-reset reading lands on the same value a plan recorded is assumed, not observed |

So the claim is not that this happened. It is that **a design of the proposed
shape would fail open** here, silently, while carrying a field implying the
check was made — the shape of harm SI-33's filing warns about, *"worse than no
witness, because it converts an admitted gap into a false assurance"*, reached
by a route the filing did not name: its concern was staleness within one
attach session, and this is a reset across instances.

**Stated exactly, because the difference is load-bearing.** The reset does not
show the counter is a bad *detector* — as a detector it moved in every leg
that could see it. It shows the counter is not a *monotone* quantity, and that
equality of two readings is therefore not evidence of non-interruption. A
design that compares readings needs something the reset cannot forge: a
driver-instance identity recorded alongside the count, so a reading from a
different instance is **incomparable** rather than equal.

**An instance-distinguishing signal was in fact read, twice, and the earlier
draft of this record wrongly said none had been looked for** — an understatement
in the direction that made the problem look less tractable than the run
showed. Sitting 2 separated instance β from α unprivileged, before taking any
leg; and L7 re-identified a drive across a surprise removal and reinsertion by
its instance id, with the disk number stable across the cycle. What is **not**
characterized is the part that matters for a witness: the signal's stability
across other boundaries, its behaviour under adversarial conditions, and its
suitability as a hashed or compared field. That characterization — not the
existence of the signal — is the open successor question.

**A gap in the protocol, recorded against the protocol.** L5 covered the
*handle* boundary and found the value survives it. Nothing in the protocol
covered the *device* boundary, and the fact surfaced only from a continuity
check added between runs to decide whether two records could be compared —
not from any designated leg. A future revision should make device
re-enumeration a designated leg with its own pre-registered refutation
condition, rather than leaving the most consequential finding of this run to a
bookkeeping step.

#### What this protocol cannot establish, declared now

- Anything beyond this one reader and bridge. The SI-28 finding that the
  bridge synthesizes the storage-layer serial stands as a warning here: the
  counter may equally be bridge behaviour, so neither a pass nor a refutation
  generalizes to a second reader, a native SD host, or any other device class
  until one is measured. The bridge measured on 2026-08-02 enumerates as USB
  vendor `0BDA` behind a whitelabel model string, recorded so a second reader
  can be compared as same-bridge or different-bridge rather than by brand —
  a brand name does not determine the bridge, and comparing by brand would
  answer the wrong question. **This is a declared exception to the
  role-labels-only rule below**, taken because a USB vendor id names a chip
  vendor rather than a unit: it distinguishes no two devices, identifies no
  host, and carries none of the session history the deltas rule exists to
  keep out. Product and unit-level identifiers stay out of the record.
- Event-driven versus poll-driven semantics, conclusively — the asymmetry
  declared under Method.
- Anything at elevated privilege — deliberately unmeasured.
- Placement of a witness field (plan body versus compared-at-apply) — SI-33's
  open design question; L5 informs it, nothing here decides it.
- macOS or Linux equivalents. The register says comparable signals exist
  elsewhere; nothing in this section measures one, and no equivalence may be
  inferred from this section's existence.
- A mounted disk image is not a substitute subject: the hypothesis concerns
  the removable-media class stack's view of a physical slot, and a virtual
  disk's medium never changes.
- **Whether a driver-instance identity exists that would make the reset
  detectable.** The reset finding names that as the successor question; this
  run did not look for such an identity, and no claim is made that one is
  available, stable, or readable unprivileged.
- **What the counter does across suspend, hub reset, or a reboot.** Two
  re-enumerations were observed — the reader's between the sittings, and the
  L7 drive's from its surprise-removal cycle — but only the reader's produced
  an interpretable counter reading, and the reader's trigger is unrecorded.
  Suspend, hub reset, and reboot are untested.

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

**Established, on the third attempt: a device does carry two signatures at once,
and the single-answer interface reports the *stale* one.**

Two earlier attempts failed, each instructively. Writing a bare `LABELONE` magic
produced nothing, because libblkid validates an LVM2 label's checksum — that
tested the forgery, not the prober. Then `mkfs.ext4` followed by `mkswap -f` left
only the swap signature, because current util-linux and e2fsprogs erase competing
signatures when they format. That second result stands and narrows round three's
collision-family list: **a partition reformatted by a current tool does not keep
its old file-system signature.**

What survives is *end-of-device* metadata, which start-of-device formatting never
reaches. WP-020's generator now builds exactly that case — a live ext4 superblock
at `0x438` and an obsolete mdraid 0.90 superblock in the last 64 KiB-aligned
block — and the two interfaces disagree:

| Interface | Reports |
| --- | --- |
| `wipefs -n` | **both**: `linux_raid_member` at `0x3f0000` *and* `ext4` at `0x438` |
| `blkid -p -o udev` (the form udev's builtin uses) | **one**: `ID_FS_TYPE=linux_raid_member`, `ID_FS_USAGE=raid` |

Two consequences, and the second is worse than the arity gap alone:

- `ID_FS_AMBIVALENT` **did not fire.** libblkid resolved the conflict by priority
  rather than flagging ambiguity, so the earlier hypothesis that a client would
  at least see "ambiguous" is wrong. It sees a confident, single, wrong-ish
  answer.
- The answer it settles on is the **stale** signature, not the live file system.
  A client reading udev's cache calls this device a RAID member; its actual
  content is an ext4 file system. The privileged helper, which can probe
  directly, can see both.

So the asymmetry is not merely that the client sees less. On this device the
client and the helper would describe the same bytes differently, and the client's
description is the one that is out of date.

**A qualification round two did not test.** Part 5 concluded that every fact
asymmetric between client and helper is a *roster-identity* fact, and that no
protection verdict needs roster identity. The arity difference above is not
roster identity, and signature presence does feed a verdict. Whether that
difference ever produces two different bodies for one unchanged device is exactly
what remains unestablished, so Part 5's conclusion needs re-checking against it
rather than either discarding or assuming.

### ADR-C3's `Present` and `Indeterminate` are not distinguishable through either interface

Measured 2026-07-28, same environment, by probing **every** fixture in the
WP-020 catalogue rather than only the multi-signature one. This is the result
that should change a design round, and it was found by asking the cheap question
— "what does a real prober say about all thirteen?" — rather than about the one
fixture a question had already been asked of.

Six of the fixtures carry a GPT in a different state. Through the two interfaces
an unprivileged client and a `blkid`-using helper actually read:

| Fixture | ADR-C3 state | `blkid -p -o udev` | `wipefs -n` |
| --- | --- | --- | --- |
| `blank-512` | Absent | *nothing* | *nothing* |
| `gpt-basic-512` | Present | `ID_PART_TABLE_TYPE=gpt` | gpt `0x200`, gpt `0x3ffe00`, PMBR `0x1fe` |
| `gpt-conflicting-tables-512` | **Indeterminate** | `ID_PART_TABLE_TYPE=gpt` | gpt `0x200`, gpt `0x3ffe00`, PMBR `0x1fe` |
| `gpt-invalid-primary-valid-backup-512` | Present, recovered | `ID_PART_TABLE_TYPE=gpt` | gpt `0x3ffe00`, PMBR `0x1fe` |
| `gpt-missing-backup-512` | Present, inconsistent | `ID_PART_TABLE_TYPE=gpt` | gpt `0x200`, PMBR `0x1fe` |
| `hybrid-mbr-gpt-512` | Present, hybrid | `ID_PART_TABLE_TYPE=gpt` | gpt `0x200`, gpt `0x3ffe00`, PMBR `0x1fe` |

**`gpt-basic-512` and `gpt-conflicting-tables-512` produce byte-identical output
from both tools.** They are different images: one has two agreeing tables, the
other has two independently valid tables describing *different* partitions, which
is precisely what ADR-C3 means by a table that parses ambiguously. Neither
interface says so. `ID_PART_ENTRY_*` is absent for whole-disk probes and
`ID_FS_AMBIVALENT` does not fire here either.

(The shared `ID_PART_TABLE_UUID` between them is by construction — the
conflicting fixture deliberately keeps one disk GUID, because it is one disk
described twice. The finding is the state collapse, not the UUID.)

Three consequences, stated as narrowly as the measurement supports:

- **`Indeterminate` cannot be computed from the udev projection.** Part 6's
  precondition already requires an ADR-C3 amendment fixing what
  `Present { checksum }` is computed over. This adds a constraint the amendment
  must satisfy rather than choose: whatever that projection is, it cannot be
  the `blkid`/udev view, because that view has no representation for the state
  at all. Distinguishing the two images requires reading *both* tables and
  comparing them — raw sector access, which §Linux above measures as **denied**
  unprivileged and which Windows also denies.
- **The hybrid case is invisible too.** `hybrid-mbr-gpt-512` carries an ordinary
  `0x0c` MBR entry aliasing the ESP's exact extent. libblkid sees the `0xEE`
  entry first, calls the MBR protective, and reports plain `gpt`. So INV-003's
  hybrid-table detection cannot be delegated to libblkid; SI-27 files this as a
  node-naming collision family, and the collision is not observable through the
  interface the client reads.
- **A damaged primary is silently recovered.** `blkid` reports `gpt` for
  `gpt-invalid-primary-valid-backup-512`. Only `wipefs`'s offset list shows the
  primary copy is missing. A client reading udev cannot tell a healthy disk from
  one running on its backup header.

### Two libblkid versions disagree about the mdraid 1.2 fixture

Measured 2026-07-29 by the first run of `cargo xtask probe` in CI, which is the
whole reason that check was automated. Increment 1 verified the signature
fixtures by hand on one machine and recorded the result as though it held
generally. It does not.

| Fixture | util-linux 2.41.0 (Debian, WSL2) | util-linux 2.39.3 (`ubuntu-24.04`) |
| --- | --- | --- |
| `mdraid-1.2-member-512.img` | `blkid -p` names it: `linux_raid_member`, UUID `62fc041a-…`, label `pm:0` | **`blkid -p` reports nothing at all** |
| — same fixture, `wipefs -n` | `linux_raid_member` at `0x1000` | `linux_raid_member` at `0x1000` — **unchanged** |
| `ext4-with-stale-mdraid-090-512.img` (0.90 superblock) | named | named — **unchanged** |

So the disagreement is specific to the **1.x** superblock, and it is the
validating interface that differs while the enumerating one agrees. Every other
fixture in the catalogue produces identical answers on both versions.

**What this means for the requirement.** FS-004 Linux RAID and LIN-005 are
**not** established on util-linux 2.39.3, which is what a stock Ubuntu 24.04
ships. A client on that platform reading udev's cache sees nothing where this
project's record said it would see an array member.

**The cause is unestablished, and should not be guessed at.** Ruled out by
reading both versions' sources: the checksum routines are arithmetically
identical — 2.39 zeroes the `sb_csum` field before summing, 2.41 subtracts its
value, which is the same sum — and the fixture satisfies the magic,
`major_version`, and `super_offset` checks 2.39 documents. Establishing which
condition it fails needs a 2.39 environment to bisect against, which this project
does not currently have; the development machine has 2.41 only.

That is the same posture as the ZFS writer above: the negative result is recorded
so it is not rediscovered, and no mechanism is asserted without measuring it.

### 4Kn is not observable from a file at all

`gpt-basic-4kn.img` reports `ID_PART_TABLE_TYPE=PMBR` — not `gpt`. libblkid
probing a regular file assumes 512-byte logical sectors, looks for `EFI PART` at
`0x200`, finds the zero padding of the 4096-byte protective-MBR sector, and falls
back to the protective MBR it did find. The 4Kn table at `0x1000` is never read.

The fixture is not wrong; its bytes are a valid 4Kn GPT, and
`crates/fixtures/src/evidence.rs` proves that structurally. What this establishes
is that **IMG-011 evidence cannot come from file-based probing**, because a file
carries no logical sector size to communicate. A prober-based check for 4Kn needs
a loop device configured with an explicit sector size, which is privileged and
therefore Tier 2. Recorded so the fixture is not "fixed" into a 512-byte table by
someone who reads `PMBR` as a defect.

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
- **The table-state rows above were probed as regular files, not devices.**
  SAFE-001 permits only that at Tier 1. Probing a loop device may differ —
  partition scanning would populate `ID_PART_ENTRY_*` — and that is a Tier-2
  measurement this project cannot yet make. The narrow claim, that libblkid
  produces identical output for the healthy and the conflicting image, is
  established for files; whether a loop device separates them is **open**.
- One libblkid version, 2.41.0. The priority resolution and the PMBR fallback
  are implementation behaviour, not specified interfaces, and may change.

### The per-user problem, now concrete

Linux is the one platform where the answer differs between two users of the same
machine running the same build: adding a user to `disk` grants both raw reads and
`blkid -p`. Whatever projection is chosen, it must be a **clamping** obligation on
the client — deliberately declining to look at what a privileged user happens to
be able to see — not merely a discard obligation on the helper. Otherwise the
same build produces different bodies for two users on one host, and PLAN-006
fails for one of them.

### The SI-35 loop-device measurement — taken 2026-08-02, read-only, across an open #94 block

Defined under spec version 4.2.0 by WP-035; the header's spec-version line
describes the established measurements, not these three sections.

**Taken 2026-08-02, read-only, while repository issue #94 was open — which
means this measurement was taken across a block that had not lifted.** That is
stated first because an earlier version of this section stated it wrongly, and
the wrong version is the kind this file exists to prevent.

**What the authorities actually say.** WP-035 increment 5: *"Anything
loop-backed is **blocked** until repository issue #94 closes; if a read-only
measurement is ever taken before then, the gap is recorded beside the
numbers."* That is a block plus a rule for what to record if it is breached —
a contingency, not a permission. Issue #94, in the paragraph this record
previously quoted for its "worst outcome is a wrong measurement" clause,
continues in the same sentence: *"that measurement is itself recorded as
Tier-2 work that cannot yet be made; this issue does not change that, and it
does **not** propose a manual, out-of-tier loop attach. `docs/quality/
test-tiers.md`'s tier boundaries stand."* The earlier version of this section
quoted the first half and omitted the second, then argued from the half it had
kept that the gate covered only destructive use. It does not, and that
argument is withdrawn.

**What happened, and on whose authority.** The run was performed at the
repository operator's explicit instruction on 2026-08-02, after the gate was
raised — but it was raised with the over-favourable reading above, so the
decision to proceed was taken on a mischaracterization of what the documents
said. The measurement was read-only throughout and no device was written; the
harm is to the record's accuracy, not to any storage.

**Consequences, recorded rather than softened.**

- **M0.5's loop-backed exit criterion is NOT satisfied by this run.** The
  specification conditions it on the loop-backed portion being "gated on
  repository issue #94", and #94 is open. These numbers exist; they do not
  discharge that criterion.
- The contingency clause **was** honoured: the binding-gap line travels
  beneath every table this run filled.
- WP-020's increment-2 row stays **Blocked** and #94's Requested recordings
  are otherwise untouched — verified, not assumed.
- `docs/quality/test-tiers.md`'s sentences remain true: nothing in the
  repository opened a block device, and the scripts that did are operator-run
  and live outside it.

**A conflict between two authorities, filed rather than resolved here.**
WP-035 increment 5 calls these measurements "operator-run, read-only
experiments... not tests and not repository commands"; issue #94 calls the
same SI-35 loop probe "Tier-2 work that cannot yet be made". Those are
different characterizations of one activity, and which governs decides whether
an operator-run loop attach is available at all before a destructive suite
exists. This record does not choose between them — that belongs under §1.11
in the spec-issue register, which is not this package's to edit — and it is
named here so the next person meets the conflict rather than one side of it.

These sections extend this file's rule of use to their own vocabulary — a row
marked `not yet taken` MUST NOT be relied on, cited, or paraphrased as a
finding, by anything, not only by an ADR that freezes canonical bytes. What
gets recorded, and what would count as refutation, was fixed before anyone saw
a result; the outcomes against those pre-registered conditions are in "What
the run established" below.

**What it answers.** The table-state section above establishes that, probed as
regular files, `gpt-basic-512` and `gpt-conflicting-tables-512` produce
byte-identical output from both interfaces, that `ID_PART_ENTRY_*` is absent
for whole-disk file probes, and that whether a loop device separates the two
images is **open**; SI-35's register entry names the loop measurement so the
file-probing limitation is not mistaken for a kernel limitation. The 4Kn
section above establishes that IMG-011 evidence cannot come from file probing
at all, and names a loop device with an explicit 4096-byte sector size as the
route. Two hypotheses, each with its refutation condition fixed before any run:

- **H-separation** — *once the kernel has parsed both images, some
  client-readable fact separates `gpt-conflicting-tables-512` from
  `gpt-basic-512`.* Refuted, on the run's environment, if the two normalized
  client projections (Phase 7) are identical; supported if any retained line
  differs. Either outcome is recorded with versions beside it and generalized
  no further — the mdraid section above is this file's own proof that this
  toolchain changes answers between versions.
- **H-4Kn** — *a loop device attached with an explicit 4096-byte logical
  sector size makes `gpt-basic-4kn`'s GPT observable, where file probing
  reported `PMBR`.* Refuted only like-for-like: if, in the same run,
  `gpt-basic-512`'s attach materialized partitions or carried
  `ID_PART_TABLE_TYPE=gpt` while the 4Kn attach shows neither. If the
  512-byte control also shows neither, the mechanism failed globally on that
  host and the cell records `failed (mechanism: <verbatim>)`, not `refuted`.

The degraded fixtures ride along with questions rather than hypotheses — which
table's view does the kernel materialize, and does any client-readable fact
mark the degradation? — and the format below enumerates the answers each
question can take, including every negative one this protocol anticipates.
Anything unanticipated is recorded verbatim under `other`, path-normalized.

**What it is not.** An operator-run, read-only experiment on the SI-28
hardware-confirmation precedent: not a test, not a repository command, not an
`xtask`. The scripts below are embedded for copy-paste reproduction and
nothing else runs them; `docs/quality/test-tiers.md` holds that device reads
are operator-run or Tier-2 work, never Tier-1 tests, and this is the
operator-run kind. It also does not close the real-partitioned-Linux row in
the scope limits above: a loop device stands in for "a device this kernel has
parsed", which answers attribution — kernel limitation or file-probing
limitation — and says nothing about real hardware.

**The #94 gate, carried structurally.** `losetup` resolves its path argument
in userspace and hands the kernel a descriptor it opened itself, so the object
attached is whatever the name resolved to at attach time, and
`/sys/block/loopN/loop/backing_file` is by-name evidence only — nothing binds
`/dev/loopN` to a verified handle (repository issue #94). WP-035's rule is
this section's rule: anything loop-backed is blocked until #94 closes, and if
a read-only measurement is ever taken before then — the issue records the
read-only blast radius as a wrong measurement, not a write — the gap is
recorded beside the numbers. The recording format makes that a template
obligation rather than a judgment call. Phase 1's digest check is accident
friction on the manifest-token model: it compares a same-named file, by name,
before attach, and closes nothing.

**Read-only posture, declared.** The only writes any phase performs are new
regular files under the operator's scratch directory — Phase 1's copies and
the Phase 4 and 7 capture files, declared here as the setup they are. Every
attach is `--read-only`, and read-only is then read back from `/sys` for the
disk and for every materialized partition rather than trusted from the flag.
No phase mounts a file system. The privileged phases are setup and a labelled
comparison view; the measurement itself is what the unprivileged interfaces
report afterward.

**Recording discipline (SEC-006).** Every value in this instrument derives
from synthetic fixture bytes — public, deterministic, generated by
`cargo xtask fixtures` — so fixture-derived values are recorded verbatim.
Host-session artifacts are not: paths are written `$SCRATCH/<name>`, the
device is written `LOOP`, loop numbers and major:minor pairs are dropped, and
identity checks are recorded as booleans because raw `id` output can carry a
username (a primary group is typically named after the user).

Phases 2–6 run once per fixture — attach, measure, detach before the next —
for `blank-512`, `gpt-basic-512`, `gpt-conflicting-tables-512`,
`gpt-invalid-primary-valid-backup-512`, `gpt-missing-backup-512`, and
`hybrid-mbr-gpt-512`, then once more for the 4Kn annex.

**Phase 0 — preflight, unprivileged.** Each line feeds the environment block.

```sh
uname -r
grep ^PRETTY_NAME= /etc/os-release
losetup --version; blkid --version; wipefs --version; udevadm --version
test -e /dev/loop-control && echo loop-control:present || echo loop-control:absent
sysctl kernel.dmesg_restrict
```

**Phase 1 — scratch, unprivileged.** The declared writes: new regular files on
a native Linux filesystem (not `/mnt/*` — a 9p-backed file is an untested loop
backing and the working copy stays off the WSL filesystem regardless). Run
`cargo xtask fixtures` in the working copy first; then compare every digest
printed here against the `image` lines of `tests/generated/MANIFEST` (SHA-256,
by name, before attach — see the gate above for what this does not bind).

```sh
SCRATCH="$HOME/partman-si35"; mkdir -p "$SCRATCH"
for f in blank-512 gpt-basic-512 gpt-conflicting-tables-512 \
         gpt-invalid-primary-valid-backup-512 gpt-missing-backup-512 \
         hybrid-mbr-gpt-512 gpt-basic-4kn; do
  cp tests/generated/$f.img "$SCRATCH/"
done
( cd "$SCRATCH" && sha256sum *.img )
```

**Phase 2 — attach. PRIVILEGED SETUP, declared as such.** This is the elevated
half; nothing it prints is the measurement. Flags verified against
`losetup`(8), util-linux: `--read-only` sets up a read-only loop device;
`--partscan` forces the kernel to scan the partition table on the new device;
`--find`/`--show` allocate the first unused device and print its name. The man
page documents no interaction between read-only and partition scanning, so
whether a read-only attach still materializes partitions is observed below,
not assumed.

```sh
sudo id -u    # must print 0 — the elevated-half assertion, recorded
LOOP=$(sudo losetup --find --show --read-only --partscan "$SCRATCH/gpt-basic-512.img")
udevadm settle --timeout=15   # best-effort wait; presence is checked in Phase 4, not assumed
BASE=${LOOP#/dev/}
cat /sys/block/$BASE/ro                    # read-only read back, per fixture
cat /sys/block/$BASE/queue/logical_block_size
cat /sys/block/$BASE/loop/backing_file     # BY-NAME diagnostic only — issue #94
sudo dmesg | tail -n 40                    # note the pre-attach tail first; record only
                                           # lines new since attach, normalized (privileged channel)
```

**Phase 3 — elevation assertion for the measuring shell**, per this file's
method line: asserted before each run rather than assumed. Booleans only.

```sh
test "$(id -u)" -ne 0 && echo uid-nonzero:yes || echo uid-nonzero:no
id -nG | grep -qw disk && echo disk-group:yes || echo disk-group:no
dd if=$LOOP of=/dev/null count=1 2>&1 | tail -n1   # a DENIAL here is the pass
```

If `uid-nonzero:no`, or `disk-group:yes`, or the `dd` read succeeds, the
measuring identity is not the unprivileged one this instrument is about: the
run still records its environment block, every client-projection cell is
marked `unavailable (measuring identity was not unprivileged)`, and only the
privileged tables may be filled from it, labelled as such.

**Phase 4 — the measurement, unprivileged: the client-readable projection.**
The udev database and `/sys`, for the disk and every materialized partition.
`/sys` reports `start` and `size` in 512-byte units by kernel convention
regardless of logical sector size — recorded here as a reading aid, and
confirmed at run time by arithmetic against the fixture layout rather than
trusted from this sentence.

```sh
BASE=${LOOP#/dev/}
capture() {  # capture <outfile> — normalized projection of $LOOP and its partitions
  for d in /sys/block/$BASE /sys/block/$BASE/${BASE}p*; do
    [ -e "$d/dev" ] || continue
    if [ -f "$d/partition" ]; then
      echo "PART $(cat $d/partition) start=$(cat $d/start) size=$(cat $d/size) ro=$(cat $d/ro)"
    else
      echo "DISK sectors=$(cat $d/size) lbs=$(cat $d/queue/logical_block_size) ro=$(cat $d/ro)"
    fi
    db=/run/udev/data/b$(cat $d/dev)
    if [ -f "$db" ]; then
      sed -n 's/^E://p' "$db" \
        | grep -Ev '^(USEC_INITIALIZED|ID_PART_ENTRY_DISK)=' | LC_ALL=C sort
    else
      echo "UDEV-DB:absent"   # observed absence — a value, not a failure (ADR-C4)
    fi
    echo --
  done > "$1"
}
capture "$SCRATCH/proj-gpt-basic-512.txt"   # substitute the fixture under measurement
awk -v b="$BASE" '$4 == b || $4 ~ ("^" b "p[0-9]+$")' /proc/partitions   # cross-check row count
```

**Phase 5 — privileged COMPARISON view, labelled.** The helper-side
interfaces the register's file measurement used, now against the device. This
never merges into Phase 4's capture; it exists so the client projection can be
compared against what a privileged prober reports from the same bytes.

```sh
sudo blkid -p -o udev "$LOOP"
sudo wipefs -n "$LOOP"
for d in /sys/block/$BASE/${BASE}p*; do
  [ -e "$d/dev" ] || continue
  sudo blkid -p -o udev "/dev/$(basename "$d")"
done
```

**Phase 6 — detach. Privileged teardown.**

```sh
sudo losetup -d "$LOOP"
```

**Phase 7 — comparison, unprivileged**, over the captured projections. `diff`
printing nothing and `IDENTICAL` is the refutation outcome for H-separation
and is exactly as recordable as a difference.

```sh
cd "$SCRATCH"
for f in gpt-conflicting-tables-512 gpt-invalid-primary-valid-backup-512 \
         gpt-missing-backup-512 hybrid-mbr-gpt-512; do
  echo "== gpt-basic-512 vs $f"
  diff proj-gpt-basic-512.txt proj-$f.txt && echo IDENTICAL
done
```

**4Kn annex.** Same phases, one change at the attach. `losetup`(8):
`--sector-size` sets the loop device's logical sector size (since Linux 4.14 —
the kernel under test satisfies this on paper; a refused attach is recorded as
`failed` with the verbatim error, not explained away), and partition-table
parsing depends on the sector size, with the man page directing that
`--sector-size` be used together with `--partscan` for non-512 tables.

```sh
LOOP=$(sudo losetup --find --show --read-only --partscan --sector-size 4096 \
       "$SCRATCH/gpt-basic-4kn.img")
```

**Bearing on SI-35's options, as the register wrote it.** Option (b) requires
establishing that some client-readable fact separates a conflicting table from
a healthy one; the register records the file measurement as evidence that none
does, for files under libblkid 2.41, leaves the loop question open, and states
that if a loop device does separate them, option (b) becomes viable and the
issue narrows sharply. This instrument produces that evidence in either sign.
Options (a) and (c) are decided by neither sign: the register's stated costs —
(a)'s observation basis becoming hash-visible body content, (c)'s inherited
unproven monotonicity obligation — are untouched by this measurement, which
changes only the measured content of the projection each option would have to
carry or clamp to. The record lands here; the register weighs it.

### The SI-35 loop-device record — filled 2026-08-02

One environment block per run, then per-fixture tables. Outcome vocabulary is
ADR-C4's as WP-035 carries it: **observed** (with absence as a value —
`UDEV-DB:absent`, `none materialized`), **unavailable** (the interface cannot
be reached from the measuring identity — e.g. the kernel log when
`dmesg_restrict` is `1`, which is why that channel is privileged-labelled),
**failed** (attempted and errored — verbatim error, path-normalized), or the
cell stays `not yet taken`. A run that stops at any phase still records the
environment block and every outcome up to the stop.

| Environment | Value |
| --- | --- |
| Run date | 2026-08-02 |
| Kernel (`uname -r`) | `6.6.114.1-microsoft-standard-WSL2` |
| Distribution | Debian 13 (trixie), under WSL2 |
| util-linux (`losetup --version`) | 2.41 |
| udev (`udevadm --version`) | 257 |
| `/dev/loop-control` present | yes |
| `kernel.dmesg_restrict` | `0` |
| Elevated half asserted (`id -u` = 0) | yes |
| Measuring shell: uid non-zero | **yes** |
| Measuring shell: in `disk` | **no** — as required |
| Measuring shell: direct read of `LOOP` | **denied**, on every fixture — the denial is the pass |
| Scratch filesystem | ext2/ext3, native — **not** 9p/`/mnt/*` |
| **Binding status (issue #94)** | **open at run time** — the first form below |

The binding-status field has exactly two legal forms, and the first is a line
that must appear **beneath every table filled by the run**, so any excerpt
that carries a number carries it too:

> **Binding gap (repository issue #94, open at run time):** nothing bound
> `/dev/loopN` to a verified handle. The attach resolved a path in userspace;
> the object the kernel attached is whatever that name resolved to at attach
> time. The pre-attach SHA-256 check and the `backing_file` read-back are
> by-name evidence only. A wrong-measurement possibility travels with every
> number in this record.

or, only if #94 is closed at run time: *"#94 closed; attach performed through
the closure's mechanism, with its binding assertion's output recorded here
verbatim"* — the mechanism named from the closure itself, not presumed from
the issue's candidate.

**A content check was performed, and it is not that closure.** After each
attach the privileged half hashed the whole loop device and compared it to the
fixture's digest; all seven matched. This binds *bytes*, not an inode: #94 is
about binding `/dev/loopN` to a verified handle, and a digest cannot do that.
What it does bound is the read-only blast radius the issue names — a wrong
backing object would have to reproduce the fixture's full-device digest to
yield a wrong measurement. It is a snapshot taken at one instant and says
nothing about the binding afterwards, and it is recorded as a mitigation, not
a closure.

Because the gap line must travel with any excerpt carrying a number, a
self-contained short form appears beneath every filled table below.

**Disk-level record** — one row per fixture. Every attach succeeded, every
`/sys` `ro` read back `1`, every content check matched:

| Fixture | Attach | `/sys` `ro` | Partitions materialized | `ID_PART_TABLE_TYPE` (udev db) | udev db entry |
| --- | --- | --- | --- | --- | --- |
| `blank-512` | ok | `1` | **0** | absent | present |
| `gpt-basic-512` | ok | `1` | **2** | `gpt` | present |
| `gpt-conflicting-tables-512` | ok | `1` | **2** | `gpt` | present |
| `gpt-invalid-primary-valid-backup-512` | ok | `1` | **0** | `gpt` | present |
| `gpt-missing-backup-512` | ok | `1` | **2** | `gpt` | present |
| `hybrid-mbr-gpt-512` | ok | `1` | **2** | `gpt` | present |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle; the attach resolved a path in userspace. A
> wrong-measurement possibility travels with these numbers.

`udevd` did process loop-attach events on this host — no `UDEV-DB:absent` was
recorded for any fixture, which the protocol had left as a measured row rather
than an assumption in either direction.

**Per-partition record** — one row per materialized partition, as many rows as
materialize; a fixture that materializes none records the literal
`none materialized` in place of rows, which is an answer, not a blank.
Fixture-derived values (`ID_PART_ENTRY_SCHEME`, `NUMBER`, `OFFSET`, `SIZE`,
`TYPE`, `UUID`, `NAME`) are recorded verbatim; a property absent from the udev
db entry is recorded as `absent`, which is observed absence, not a failure.

| Fixture | Partition | `/sys` start/size (512-B units) | `ro` | `ID_PART_ENTRY_*` present | Values |
| --- | --- | --- | --- | --- | --- |
| `blank-512` | *none materialized* | — | — | — | — |
| `gpt-basic-512` | p1 | 2048 / 2048 | `1` | yes | `NAME=EFI\x20System` `SCHEME=gpt` `OFFSET=2048` `SIZE=2048` `TYPE=c12a7328-…` `UUID=e0dfdbfc-…` |
| `gpt-basic-512` | p2 | 4096 / 4063 | `1` | yes | `NAME=Data` `SCHEME=gpt` `OFFSET=4096` `SIZE=4063` `TYPE=0fc63daf-…` `UUID=28e70b48-…` |
| `gpt-conflicting-tables-512` | p1, p2 | 2048/2048, 4096/4063 | `1` | yes | **byte-identical to `gpt-basic-512`'s**, every key and value |
| `gpt-invalid-primary-valid-backup-512` | *none materialized* | — | — | — | — |
| `gpt-missing-backup-512` | p1, p2 | 2048/2048, 4096/4063 | `1` | yes | **byte-identical to `gpt-basic-512`'s** |
| `hybrid-mbr-gpt-512` | p1, p2 | 2048/2048, 4096/4063 | `1` | yes | **byte-identical to `gpt-basic-512`'s**; `SCHEME=gpt` |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle. A wrong-measurement possibility travels with these numbers.

`/sys` reports `start` and `size` in 512-byte units regardless of logical
sector size — confirmed by arithmetic against the fixture layout rather than
trusted from the protocol's sentence: the 4Kn annex's ESP sits at LBA 256 of
4096 bytes and `/sys` reported `start=2048`, which is 256 × 8.

**View classification** — for the fixtures where two descriptions of the disk
exist, which one the kernel materialized. The candidate descriptions are
public source facts of `crates/fixtures/src/catalogue.rs`, restated here so
classification is mechanical: `gpt-conflicting-tables-512`'s primary set is
"EFI System" at LBA 2048–4095 plus "Data" at 4096–8158; its backup set is one
partition, "Disagreeing", at 2048–8158. Classification is by start/size/name
match; anything else is `other` with the verbatim rows.

| Fixture | Question | Legal answers | Result |
| --- | --- | --- | --- |
| `gpt-conflicting-tables-512` | Which view materialized? | `primary set` / `backup set` / `neither` / `other (verbatim)` | **`primary set`** — ESP at 2048–4095 plus Data at 4096–8158, matching the catalogue's primary exactly. The backup's single "Disagreeing" partition appears nowhere |
| `gpt-invalid-primary-valid-backup-512` | Did partitions materialize, and does any client-readable fact differ from `gpt-basic-512`'s projection? | `materialized, marked` / `materialized, unmarked` / `none materialized` / `other` | **`none materialized`** — and the whole-disk entry still carries `ID_PART_TABLE_TYPE=gpt`. The kernel materialized nothing while libblkid labelled the disk `gpt` |
| `gpt-missing-backup-512` | Same question, for the missing backup | same four | **`materialized, unmarked`** — two partitions, projection identical to healthy, nothing flagged |
| `hybrid-mbr-gpt-512` | Which scheme won, and does any client-readable fact carry a trace of the aliasing `0x0c` entry? | `gpt, traced` / `gpt, untraced` / `dos` / `other` | **`gpt, untraced`** — `SCHEME=gpt`, the GPT partitions materialized, and the aliasing `0x0c` MBR entry left no trace in the client projection |
| `blank-512` | Control: zero partitions and no table type? | `confirmed` / `other (verbatim)` | **`confirmed`** |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle. A wrong-measurement possibility travels with these numbers.

**Separation record** — the SI-35 question itself, answered only by the
Phase 7 diff over declared normalizations.

**The drop-list had to be extended at run time, and the rule that forced it
worked.** The list was closed at `USEC_INITIALIZED` and `ID_PART_ENTRY_DISK`,
with the standing condition that any retained key carrying a device name joins
it with its reason "or the diff is not this measurement". The first run
returned every pair as DIFFERS — because a loop device's udev entry carries
`ID_LOOP_BACKING_FILENAME`, `ID_LOOP_BACKING_FILENAME_ENC`,
`ID_LOOP_BACKING_INODE` and `ID_LOOP_BACKING_DEVICE`, which name **the backing
file**, so each fixture differed from every other by its own filename and by
nothing else. That is a property of the loop plumbing, not of the partition
table. The four keys join the drop-list with that reason and the diff was
recomputed; the first computation is void and its output occupies no cell.
The device-name check as written scanned for `loop[0-9]` and did not catch a
key whose *value* was a path — a real hole in the check, recorded so the next
revision widens it to any key whose value contains the scratch path.

Final drop-list: `USEC_INITIALIZED`, `ID_PART_ENTRY_DISK`, and the four
`ID_LOOP_BACKING_*` keys.

| Pair (normalized client projections) | Differ? | Differing lines |
| --- | --- | --- |
| `gpt-basic-512` vs `gpt-conflicting-tables-512` | **IDENTICAL** | none |
| `gpt-basic-512` vs `gpt-invalid-primary-valid-backup-512` | **DIFFERS** | the entire partition half: `gpt-basic-512` carries two `PART` entries with their full `ID_PART_ENTRY_*` sets; the damaged fixture carries none. The whole-disk `ID_PART_TABLE_TYPE=gpt` line is common to both |
| `gpt-basic-512` vs `gpt-missing-backup-512` | **IDENTICAL** | none |
| `gpt-basic-512` vs `hybrid-mbr-gpt-512` | **IDENTICAL** | none |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle. A wrong-measurement possibility travels with these numbers.

**Privileged comparison record** — labelled, never merged with the rows above:

| Fixture | `blkid -p -o udev` (device) | `wipefs -n` offsets (device) | `blkid -p -o udev` (each partition) |
| --- | --- | --- | --- |
| `blank-512` | *(nothing)* | *(nothing)* | none |
| `gpt-basic-512` | `TABLE_TYPE=gpt` `TABLE_UUID=7a1e9153-…` | `0x200`, `0x3ffe00`, `0x1fe` | p1 ESP, p2 Data, full entries |
| `gpt-conflicting-tables-512` | **identical to basic** | **`0x200`, `0x3ffe00`, `0x1fe` — identical to basic** | **identical to basic** |
| `gpt-invalid-primary-valid-backup-512` | `TABLE_TYPE=gpt` — same as basic | **`0x3ffe00`, `0x1fe` — the primary at `0x200` is absent** | none — no partitions materialized |
| `gpt-missing-backup-512` | `TABLE_TYPE=gpt` — same as basic | **`0x200`, `0x1fe` — the backup at `0x3ffe00` is absent** | p1, p2, identical to basic |
| `hybrid-mbr-gpt-512` | `TABLE_TYPE=gpt` — same as basic | `0x200`, `0x3ffe00`, `0x1fe` — identical to basic | identical to basic |
| `gpt-basic-4kn` | `TABLE_TYPE=gpt` `TABLE_UUID=1ac207e4-…` | `0x1000`, `0x3ff000`, `0x1fe` — the 4Kn offsets | p1 ESP, p2 Data (`TYPE=ebd0a0a2-…`) |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle. A wrong-measurement possibility travels with these numbers.

**The asymmetry, quantified.** Putting the privileged view beside the client
projection gives the client/helper comparison this file exists to make, and it
is not uniform across the damage cases:

| Fixture | Helper (`wipefs` offsets) separates it from healthy? | Client projection separates it? |
| --- | --- | --- |
| `gpt-conflicting-tables-512` | **no** — identical offsets | **no** — identical projection |
| `gpt-invalid-primary-valid-backup-512` | **yes** — primary copy absent | **yes** — no partitions materialized |
| `gpt-missing-backup-512` | **yes** — backup copy absent | **no** — projection identical to healthy |
| `hybrid-mbr-gpt-512` | **no** | **no** |

So the client is not uniformly weaker: it separates the damaged-primary case
the helper also separates, is blind where the helper sees the missing backup,
and both are blind to the conflicting and hybrid tables. The ambiguous
table — the one ADR-C3's `Indeterminate` exists for — is invisible to **both**.

**4Kn annex record:**

| Row | Value |
| --- | --- |
| Attach with `--sector-size 4096` | ok |
| `logical_block_size` read-back | **4096** |
| `ID_PART_TABLE_TYPE` (udev db) | `gpt` |
| Partitions materialized | **2** — ESP at `/sys` start 2048, Data at 4096 |
| H-4Kn on this environment | **`supported`** |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle. A wrong-measurement possibility travels with these numbers.

**H-4Kn is supported, and the like-for-like condition was met properly.** The
refutation condition required the same-run `gpt-basic-512` control to
materialize partitions or carry `ID_PART_TABLE_TYPE=gpt` while the 4Kn attach
showed neither; the control did materialize, and so did the 4Kn attach, so the
mechanism was working and the annex's answer is a real one rather than a
global failure misread as a refutation.

**This is the IMG-011 route the file said would be needed.** The Linux section
above records that `gpt-basic-4kn.img` probes as `PMBR` from a regular file,
because a file carries no logical sector size, and that a prober-based check
for 4Kn "needs a loop device configured with an explicit sector size, which is
privileged and therefore Tier 2". That device was configured and the 4Kn GPT
is observable through it: `wipefs` finds the tables at `0x1000` and `0x3ff000`
rather than the 512-byte offsets, and the partition extents match the
catalogue's 4Kn layout under the 512-byte-unit convention. The fixture's
`PMBR` result from file probing is confirmed as an artifact of file probing,
not a defect in the fixture — which is exactly what that record asked a future
measurement to settle.

### What the run established

> **Scope line, travelling with every claim below:** these results are one
> kernel build, one util-linux, one udev, under WSL2. The decisive-pair result
> is an **absence** claim, and this section's own rule withholds such a claim
> from any register decision until a non-WSL distro-kernel run confirms it.

**H-separation is refuted on this environment.** Its condition was fixed
before the run: refuted if the two normalized client projections are
identical. They are — byte for byte, once the backing-object keys are dropped.
The kernel parsed both images, materialized the **primary** set for the
conflicting fixture, and produced a projection indistinguishable from the
healthy disk's. The backup's disagreeing partition appears nowhere, and no
client-readable fact marks the disagreement.

**On SI-35's attribution question, the answer is provisional and points one
way.** The register asked for this measurement "so the file-probing limitation
is not mistaken for a kernel limitation". On this environment it is not a
file-probing artifact: a device the kernel has fully parsed, with partition
scanning on, yields the same collapse, and the privileged view is no better —
`wipefs` reports identical offsets for both images. So here **neither the
client nor the helper can distinguish an ambiguous table from a healthy one.**
The word is *provisional*, not *settled*: this file's own mdraid section is
the proof that this toolchain changes its answers between versions, and a
result cannot be settled and simultaneously withheld from register use. An
earlier version of this paragraph said "settles", which contradicted the
withholding rule four screens below it.

**Option (b) does not become viable.** The register states that if a loop
device separates the two, "option (b) becomes viable and this issue narrows
sharply". It does not separate them. Combined with the Windows partition-list
measurement recorded above and the original file probing, the decisive pair is
now indistinguishable through **three interfaces on two platforms**. Options
(a) and (c) are untouched by this run: their recorded costs — (a)'s
observation basis becoming hash-visible body content, (c)'s inherited unproven
monotonicity obligation — are unchanged. The record lands here; the register
weighs it.

**Two findings the hypotheses did not ask for.**

- **A damaged primary is separated, by partition count rather than by any
  table property.** `gpt-invalid-primary-valid-backup-512` materialized **no**
  partitions while its udev entry still read `ID_PART_TABLE_TYPE=gpt`. So the
  kernel's partition parser and libblkid disagree about the same device: one
  declines the table, the other labels it. A client reading only
  `ID_PART_TABLE_TYPE` sees a healthy-looking `gpt`; a client that also counts
  materialized partitions sees the difference. This is a **second** instance
  of the two-interfaces-disagree pattern this file has recorded, after the
  `blkid`-versus-`wipefs` arity finding on Linux. It is not a third: the
  `MSFT_Disk`-versus-layout-IOCTL "disagreement" this record briefly claimed
  was an artifact of comparing two enumerations, and is withdrawn in the
  Windows subsection above.
- **A missing backup is helper-only.** `wipefs` shows the backup copy absent;
  the client projection is identical to healthy. That is a clean instance of
  the asymmetry Part 5's conclusion needs re-checking against, and it is
  recorded for that purpose without deciding it.

**The hybrid table is invisible here too.** `gpt, untraced`: the aliasing
`0x0c` MBR entry leaves no trace in the client projection, matching libblkid's
file result. INV-003's hybrid-detection requirement therefore cannot be
delegated to this projection on Linux — and the Windows equivalent fell into
that platform's enumeration gap, so **the hybrid question is unanswered on
both platforms**.

### What a WSL2 run of this protocol does and does not establish

The available machine is the WSL2 Debian environment recorded at the top of
this Linux section (kernel 6.6.x Microsoft build, util-linux 2.41, systemd
with udev running). Unlike the hardware-scoped rows above — measured on
particular disks whose device trees bound their claims — no row in this
instrument depends on hardware: the objects under test — the loop driver, the
kernel's partition parser, `udevd`, libblkid — are stock software operating on
synthetic bytes, so the virtual-disk scope limit does not carry over. Three
limits do:

- **One environment.** A Microsoft kernel build is not a distro kernel config,
  the udev rule set that decides whether loop devices get database entries at
  all is a distro-shipped file, and the mdraid section above is this file's
  proof that one util-linux version's answer is not another's. Whether `udevd`
  processes loop-attach events on any given host is therefore a measured row —
  `UDEV-DB:absent` is a finding about the platform, not a broken run.
- **Negatives demand a second environment, and this run produced the negative
  that matters.** A row recorded only under WSL2 that asserts an absence —
  projections identical, no udev entry, no partitions — must not be relied on
  by any register decision until confirmed on one non-WSL, distro-kernel
  environment, because an absence claim generalizes worse than an existence
  claim. **H-separation's refutation is exactly such a negative**, and this
  rule binds it: the decisive-pair result stands as measured on this
  environment and is **not yet available to a register decision** until a
  non-WSL distro-kernel run confirms it. The positive rows — H-4Kn supported,
  the damaged-primary separation — carry the ordinary environment scoping
  every row in this file carries.

  **What remains outstanding on SI-35's evidence list, counted honestly.** The
  register requires three things before any option is accepted: the
  loop-device measurement, the same measurement on Windows, and *"a
  demonstration that whichever option is chosen still refuses rather than
  proceeds on `gpt-conflicting-tables-512.img`"*. This run supplies the first
  and the Windows subsection supplies the second. **Two** remain: the non-WSL
  confirmation this rule demands of the negative, and the register's third
  item — which **no measurement can supply**, because it depends on an option
  being chosen and the register has not chosen one. An earlier version of this
  section said only one piece was outstanding, which made SI-35 read as one
  run from closure. It is not, and this package may not make it look so.
- **The attach is privileged everywhere.** A separation finding here is a fact
  about what the kernel's parse leaves client-readable, not about what an
  unprivileged client can cause to be parsed. Real disks are parsed at
  device-add without anyone elevating; whether their projection matches this
  one is exactly the real-partitioned-Linux row that stays open above.

## Reproducing this

The Windows facts above come from read-only CIM queries against
`root/Microsoft/Windows/Storage` plus one read-only `CreateFile` attempt on a
physical-drive path. No device layout, serial, or unique id from the measured
machine is recorded here; only whether each property was present and readable,
per SEC-006's redaction posture.
