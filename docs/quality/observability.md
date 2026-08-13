# Unprivileged observability

- Spec version: 4.1.0
- Requirement IDs: SAFE-002, SAFE-003, HLP-002, MODEL-005, INV-002, INV-003
- Status: **Windows hardware-scoped rows established. Linux real-hardware
  rows established 2026-08-04 by the increment 6 matrix on explicitly
  authorized passthrough fixture media; the earlier WSL2 virtual-SCSI rows
  (no partitions) and the synthetic loop run (non-qualifying) keep their
  own records' caveats. macOS established 2026-08-05 on Apple Silicon —
  M1–M8 taken and valid on the second sitting, the first being void on two
  recorded harness defects, and **M10 taken the same day** in an ephemeral
  hosted runner, where the helper reads at byte level what the client is
  denied. Only M9 remains `not established`, Apple Silicon having no Fusion
  Drive. **No preregistered cell on any platform is now `not yet taken`.**
  The macOS second-reader readback was discharged 2026-08-08 by an
  independent reader session: both sitting 2 transcripts and the M10
  transcript retrieved through their locators and rehashed, every digest
  matching, with the caveat each record carries stated there rather than
  erased.** WP-035's three instruments have operator-run records dated
  2026-08-02, but their limits are material: SI-33 did not establish the full
  close-before-event/reopen liveness sequence; the SI-35 Windows wrapper did
  not retain every queried surface; and the SI-35 loop run was taken while
  repository issue #94 was open. **#94 closed 2026-08-03** — WP-020 increment
  2e's descriptor-bound mechanism landed and passed a real acceptance — so the
  hardened non-WSL protocol below became **runnable rather than blocked**, and
  on 2026-08-03 it was **taken and passed as valid** on its third sitting (two
  void sittings and their recorded instrument amendments precede it): the
  named candidate client projection is **non-separating** for the decisive
  healthy/conflicting pair on real non-WSL Linux, the WSL2 promotion hold is
  lifted — the historical negative is confirmed through the handle-bound
  protocol — and **M0.5's loop criterion is satisfied**; the record's
  second-reader obligation is discharged (the designated readback recorded
  on the result pull request, then an independent retrieve-and-rehash of
  both artifacts on 2026-08-04, digests matching). The run chooses no SI-35
  option, supplies no chosen-option refusal demonstration, and refutes no
  existential H-separation hypothesis. **The SI-35 Windows completion rerun
  was taken 2026-08-04 and is valid**: all three of its gates held, W-H1,
  W-H2, and W-H3 are refuted and W-Q4 answered, and its four artifacts
  likewise passed a designated readback and an independent 2026-08-04
  rehash. The successor protocol's S4 collision test has two sittings:
  2026-08-03 measured the then-attached pair as cross-model
  (`not established` for that pair); 2026-08-04, on a same-model pair,
  **observed the preregistered collision** — one identical placeholder
  serial from both units at every serial-bearing layer, the second unit
  re-keyed by port with its unique-id capability cleared — and the
  empty-slot rider with it; a third sitting the same day completed the
  card-move rider: the exchange is invisible at every serial surface, and
  on a shared-constant pair unit continuity across it is unverifiable
  unprivileged. **Every S4 arm is executed.** A same-day S1/S2/S2b/S3
  sitting on the reattached parent-record reader closed SI-33's remaining
  measurement gaps on that apparatus: close-before-event/reopen survival
  `moved` in all three trials across true no-handle windows, L4 completed
  at its three trials, the boundary-1 counter reset measured (five events
  above the epoch floor before a reader re-arrival, at the floor after
  it), and the storage-node PDO name qualified as an unprivileged epoch
  signal while ContainerId and the USB-node PDO name were refuted. **The
  successor protocol is fully executed.** **The increment 6
  real-partitioned-Linux matrix was taken 2026-08-04** — every row
  executed on a disposable Proxmox VM with authorized passthrough fixture
  media: the client/helper signature asymmetry measured in both
  directions, the LVM2 member-independent designator helper-only, the
  SI-34 stale-signature finding established on a real device tree, and
  the L9 collision resolving by silent last-writer-wins on every
  UUID-keyed surface. **Increment 6's macOS matrix was taken 2026-08-05**,
  valid on its second sitting after a void first: the decisive SI-35 pair is
  **non-separating** on macOS too, making it the third platform to answer that
  way; every non-native signature — live ext4 with a stale mdraid superblock,
  an mdraid member, a LUKS2 container, an LVM2 orphan — projects
  **byte-identically to a blank disk**; APFS container membership and its UUID
  are client-readable and the UUID is stable across a verified reboot; and the
  unprivileged raw device read is denied. **M10, the privileged comparison leg, was taken the same day** in an
  ephemeral hosted `macos-15` runner: the client's raw read is denied while
  root reads the true bytes, and **the decisive pair separates for the helper**
  — identical head digests, differing tail digests, so the disagreement lives
  in the backup table no client interface reports. The four signatures the
  client called byte-identical to blank each carry a distinct helper digest.

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

Read-only block-device and hardware queries only. Declared setup steps may
write regular scratch files and virtual-container files; the historical loop
scratch step could overwrite a prior same-named scratch copy, as its section
records. No experiment row writes to a block device. The increment 6
real-partitioned-Linux protocol's provisioning writes — declared there
before they existed on disk, and widened into this sentence by the same
change that landed that protocol's first executed run — are performed only
by its separately declared privileged setup actor, only onto explicitly
authorized disposable fixture media passed through to a disposable VM, and
are digest-bracketed before and after every layout's measurements.
Elevation state was asserted
before each run rather than assumed.

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

**Headline: no retained field from `MSFT_Disk`/`MSFT_Partition`,
`MSFT_PhysicalDisk` row presence, or the layout IOCTL separated the
conflicting, damaged-primary, or missing-backup fixture from healthy GPT.**
The executed wrapper discarded queried `MSFT_PhysicalDisk` property values, so
the hypotheses' declared “every status surface” refutation conditions cannot
be evaluated; the broader existential claims remain inconclusive rather than
universally refuted. Two fixtures could not be measured through `MSFT_Disk` at all, for a
reason that is itself a finding and is recorded below rather than as an
absence. This
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
the *state* — that the disk's description is ambiguous, damaged-primary, or
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
succeeded, and the executed wrapper reported every post-detach digest as
`UNCHANGED`. The underlying before/after digest pairs were not retained, so
that wrapper verdict is not independently auditable. W3's control row was
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
the analogue of `/sys/block/loopN/loop/backing_file`. The intended digest
bracket (S2's read-back before, S4's re-hash after) would bound the file's
content on both sides of the attach window. This run retained only the
wrapper's `UNCHANGED` verdict, not the digest pairs, so its durable record
cannot independently establish even those endpoint equalities; nothing in
this protocol asserts the kernel's binding *during* the window. The loop-backed
half of SI-35 was gated on #94, which **closed 2026-08-03**; this
Windows half never was — the gate is loop-specific — and the same class of
residual is still declared rather than assumed away. Closing #94 lifts the
loop-side gate only by supplying a descriptor-bound attach; it does not
retire this Windows residual, which `Mount-DiskImage` still has.

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

# Independent of MSFT_Disk enumeration: query before any candidate-count return.
'--- MSFT_PhysicalDisk (does the virtual disk appear here at all?) ---'
$physical = @(Get-CimInstance -Namespace $ns -ClassName MSFT_PhysicalDisk |
    Where-Object { $_.BusType -eq 15 -and $_.Size -eq $fixtureSize })
"rows: $($physical.Count)"
$physical | Select-Object BusType, MediaType, HealthStatus, Size | Format-List

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

'--- MSFT_Partition ---'
$partitions = @(Get-CimInstance -Namespace $ns -ClassName MSFT_Partition |
    Where-Object { $_.DiskNumber -eq $disk.Number })
"rows: $($partitions.Count)"
$partitions | Sort-Object PartitionNumber | Select-Object PartitionNumber, Offset, Size,
    MbrType, GptType, Guid, IsActive, IsHidden, IsReadOnly, IsOffline | Format-List
```

The 2026-08-02 execution used an earlier rendezvous wrapper: its
`MSFT_PhysicalDisk` query followed the zero/ambiguous-`MSFT_Disk` returns, and
its logger retained only the PhysicalDisk row count. The corrected script
above moves the independent query before those returns and prints the selected
properties, but **has not been rerun**. Tables W1–W3 record the executed
wrapper, not the corrected text.

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
- **No operator paths, usernames, or drive letters** appear in the record.
  Session-local disk numbers do appear where they are the evidence that an
  otherwise invisible fixture remained reachable (index 7, alongside roster
  indices 0–6). Errors are recorded as Win32/HRESULT codes plus a message with
  any embedded path elided.
- The protocol required run metadata, both elevation assertions, per-fixture
  digest pairs, post-detach verdicts, and an incident log. The durable result
  retained the run/build/elevation facts and `UNCHANGED` verdicts, but **not**
  the raw/converted digest pairs or incident-log contents. Their absence is a
  run-record defect; it is not evidence that no prompt or dialog occurred.

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
with the executed wrapper reporting every post-detach digest **UNCHANGED**.
The underlying digest pairs were not retained, so that verdict is not
independently auditable and does not prove the bytes were never transiently
altered during the attach window. Retained fixture values are recorded
verbatim; the bytes are synthetic, deterministic and public. `not retained`
below is an audit state, not an ADR-C4 outcome: the interface was queried but
the executed logger discarded the property values.

| Fixture | ADR-C3 state / fixture fact | `MSFT_Disk` enumeration | `PartitionStyle` | `Guid` | `Signature` | `IsOffline` / `OfflineReason` | `IsReadOnly` | `OperationalStatus` / `HealthStatus` | Partition rows (count) | `MSFT_PhysicalDisk` retained result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `mbr-basic-512` | Present (MBR control) | `observed(absent from MSFT_Disk)` — see the enumeration gap below | `unavailable(no MSFT_Disk row)` | `unavailable` | `unavailable` | `unavailable` | `unavailable` | `unavailable` | `not queried` | `not queried (executed wrapper returned first)` |
| `blank-512` | Absent | `observed(present)` | `0` (unknown/uninitialized) | `''` | `''` | `False` / `0` | `True` | `53264` / `0` | `0` | `rows=1`; BusType/MediaType/HealthStatus/Size `not retained (queried; logger discarded values)` |
| `gpt-basic-512` | Present | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` | `''` | `False` / `0` | `True` | `53264` / `0` | `2` | `rows=1`; properties `not retained` |
| `gpt-conflicting-tables-512` | **Indeterminate** | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` — **identical to basic's** | `''` | `False` / `0` — **identical** | `True` — **identical** | `53264` / `0` — **identical** | `2` — **identical** | `rows=1` — **identical**; properties `not retained` |
| `gpt-invalid-primary-valid-backup-512` | Present; primary-header CRC invalid | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` | `''` | `False` / `0` — **identical** | `True` — **identical** | `53264` / `0` — **identical** | `2` — **identical** | `rows=1` — **identical**; properties `not retained` |
| `gpt-missing-backup-512` | Present, inconsistent | `observed(present)` | `2` (GPT) | `{7a1e9153-…-8c23898f2cbf}` | `''` | `False` / `0` — **identical** | `True` — **identical** | `53264` / `0` — **identical** | `2` — **identical** | `rows=1` — **identical**; properties `not retained` |
| `hybrid-mbr-gpt-512` | Present, hybrid | `observed(absent from MSFT_Disk)` — see the enumeration gap below | `unavailable(no MSFT_Disk row)` | `unavailable` | `unavailable` | `unavailable` | `unavailable` | `unavailable` | `not queried` | `not queried (executed wrapper returned first)` |

The shared disk GUID across the four GPT fixtures is **by construction** —
they are one disk in different states — and is not a finding. Every retained
`MSFT_Disk` field, partition-row count, and PhysicalDisk row count is equal
across those four; equality of discarded PhysicalDisk properties is unproved.

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
| `mbr-basic-512` | `not queried (executed wrapper returned before MSFT_Partition)` | — | — |
| `blank-512` | 0 | none | `none` |
| `gpt-basic-512` | 2 | offset 1048576 size 1048576 type `{c12a7328-…}` (ESP); offset 2097152 size 2080256 type `{0fc63daf-…}` (Linux FS) | `primary/backup (indistinguishable by content)` — this fixture's backup agrees with its primary, so its rows cannot identify the copy parsed either |
| `gpt-conflicting-tables-512` | 2 | **equal to `gpt-basic-512` on every retained field**: offset, size, type GUID, and partition GUID | `primary` — the valid primary's content was presented; the disagreeing backup is not represented in the retained rows |
| `gpt-invalid-primary-valid-backup-512` | 2 | equal on every retained field | `primary/backup (indistinguishable by content)` — both copies describe the same partitions, so retained row content cannot identify the copy |
| `gpt-missing-backup-512` | 2 | equal on every retained field | `primary` |
| `hybrid-mbr-gpt-512` | `not queried (executed wrapper returned before MSFT_Partition)` | — | — |

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
- **It reproduced** — twice for each fixture, in two runs about three minutes
  apart within one session. The host device set changed between the runs, so
  these are not independent sittings under an identical host state.

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
alone, or nothing at all. The two non-protective-MBR configurations were
invisible while the five protective-only/blank configurations were visible.
That is a correlation across fixture configurations, **not seven independent
samples** — four GPT variants share one base image — and not a mechanism:
nothing here establishes why, no discriminating variant was constructed, and
one build was measured.

**What it costs the measurement.** INV-003's hybrid-detection question and the
MBR control both fall in the gap: `hybrid-mbr-gpt-512` is the fixture that
would have answered which scheme Windows privileges, and `mbr-basic-512` was
its control. Both were absent from `MSFT_Disk`; the executed wrapper then
returned before asking `MSFT_Partition` or `MSFT_PhysicalDisk`, and the reachable
layout IOCTL was not run. The hybrid question therefore stays open on this
platform — as it does on Linux, where libblkid reports plain `gpt` for the same
image.

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
- **W-H2, damaged primary.** Some surface flags
  `gpt-invalid-primary-valid-backup-512` as damaged. Refuted if its W1 row
  equals `gpt-basic-512`'s and its partitions appear ordinarily. Which GPT copy
  was used, and whether the primary CRC was validated, are not observable from
  equal row content.
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

**What the aggregate outcomes feed.** These hypotheses are preserved as
pre-registered, including their existential word *surface*. The retained
projection supplies no separating field, but the executed wrapper queried and
discarded `MSFT_PhysicalDisk` properties. Because the declared refutation
conditions name every W1 status surface, those conditions cannot be evaluated
from this record. The equal retained fields give SI-35 no support for option
(b) on that named projection, while they do not establish that every
client-readable surface collapses the states.

#### Outcomes, 2026-08-02

Each antecedent checked against the recorded cells, not against a reading of
them:

| Hypothesis | Outcome |
| --- | --- |
| **W-H1**, the decisive pair | **Inconclusive; the declared refutation condition is unevaluable from the retained record.** `IsOffline`/`OfflineReason`, `IsReadOnly`, retained `OperationalStatus`/`HealthStatus`, PhysicalDisk row count, and every retained partition field were equal for conflict and basic; the separate layout IOCTL was equal too. The rows match the valid primary's content and retain no trace of the disagreeing backup. But queried PhysicalDisk property values, including its health value, were discarded, so the pre-registered “every status surface” condition cannot be checked |
| **W-H2**, damaged primary | **Inconclusive; the declared refutation condition is unevaluable.** The retained fields equal basic and two partitions appear ordinarily, but queried PhysicalDisk properties are absent. **Which copy was used is unmeasured** — both GPT copies describe the same partitions, so the run cannot distinguish backup use from primary parsing without CRC validation |
| **W-H3**, missing backup | **Inconclusive; the declared refutation condition is unevaluable.** The retained fields equal basic and partitions are present, but discarded PhysicalDisk properties leave both the named condition and the broader surface claim open |
| **W-Q4**, hybrid | **Not attempted**, which is weaker than unanswerable and is the honest word. The fixture produced no `MSFT_Disk` row, so the CIM route was closed — but `Win32_DiskDrive` supplied a device index for the same attached disk in the same session, and W3 establishes that the zero-access layout IOCTL is readable unprivileged at such an index. That probe would have answered which scheme the stack privileged and it was simply not run. The gap here is in the execution, not the platform |
| **W-Q5**, blank | **Answered: distinct.** `blank-512` reports `PartitionStyle=0`, no partitions, empty GUID and signature — distinguishable from every GPT fixture and from the `unavailable` rows. Whether `0` maps to ADR-C3's positively-observed `Absent` or to an unreadable unknown is a register question the value alone does not settle, exactly as the format said it would not |

**What this feeds, stated no wider than the run supports.** On this build and
file-backed bus type, the **retained Windows projection** collapses `Present`
and `Indeterminate` onto one description. It supplies no retained
client-readable fact for SI-35 option (b), but the PhysicalDisk capture gap
prevents a claim about every unprivileged surface. The WSL2 loop negative is
separately withheld. Options (a) and (c) are untouched; the register weighs
these bounded observations rather than a platform-wide refutation.

#### Non-answers, each with a defined recording

- **Setup refusal** (digest mismatch, conversion verification failure): the
  fixture's rows stay `not yet taken`; the refusal goes in the run record.
  A refused setup is not a failed measurement — the measurement never began.
- **Attach fails**: that fixture's measurement cells become
  `unavailable(attach failed: <code>)`. If the footer constants are the
  suspected cause, that is recorded as a suspicion, not a diagnosis.
- **No `MSFT_Disk` row appears**: that column records `observed(absent from
  MSFT_Disk)`. The independent PhysicalDisk query still runs; MSFT_Partition is
  `not queried` without a disk number; and a layout probe is attempted only if
  another unprivileged roster supplies a device index. This is an
  interface-enumeration finding, not proof that the whole non-elevated session
  cannot see the disk, and it must not become "no separation observed".
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
  databases, and raise dialogs. The incident-log contents were not retained,
  so durable evidence does not establish which of those occurred. The wrapper
  reported `UNCHANGED` after detach, but the missing digest pairs make even
  that content verdict independently unauditable; it proves nothing about
  host behaviour or transient changes during the window.
- **The custody residual** recorded in the setup section: the attach is
  by-path and the reported backing path is by-name evidence. The intended
  digest bracket would have bounded endpoint content, but the missing pairs
  mean this record does not independently establish even that; it never bound
  the attach window.

#### Completion rerun — taken 2026-08-04; valid; all three hypotheses refuted

Protocol recorded 2026-08-02 under WP-035's SI-35 share, preregistered as
an instrument. Status: **taken 2026-08-04; valid** — the sitting record
below is the evidence. It reruns exactly what the 2026-08-02
execution left unevaluable — nothing else, and executing it may not add
hypotheses. W-H1, W-H2, and W-H3 stand as pre-registered above, existential
*surface* wording included; W-Q5 is answered and is not rerun. Every
mechanic of the parent protocol is inherited unchanged: one-at-a-time
attach, the declared elevated S3 attach in its separate console, the
non-elevated measurement session with its recorded assertion, the
non-answer recordings, and every declared scope limit (file-backed bus
type, one build, 512-byte sectors, 4Kn out of reach). This subsection adds
three gates and the cells they make evaluable:

- **R1 — total retention.** Every property value any step queries is
  retained verbatim in the transcript; query-and-discard — the recorded
  defect that made all three hypotheses unevaluable — voids the sitting.
  The capture script's digest is recorded before the first attach, and the
  script must be reviewed against this gate before privilege.
- **R2 — digest bracket restored.** The before-attach and after-detach
  fixture digest pairs are retained for every fixture; a missing pair voids
  that fixture's rows. Wrapper prose such as `UNCHANGED` is not a digest
  and satisfies nothing.
- **R3 — mandatory index-fallback probe.** The parent's "a layout probe is
  attempted only if another unprivileged roster supplies a device index"
  becomes a MUST: for any fixture without an `MSFT_Disk` row, the
  zero-access layout IOCTL runs at every `Win32_DiskDrive`-supplied index
  for that disk in the same session. An index available and unprobed —
  the recorded W-Q4 execution gap — voids that fixture's cells.

| Cell | Result |
| --- | --- |
| W-H1 refutation condition evaluated over complete retained surfaces | **`refuted`** — every W1 status surface equal and the partition rows identical; see the sitting record below |
| W-H2 same, with the which-copy limit below | **`refuted`** — W1 row equals `gpt-basic-512`'s and its partitions appear ordinarily |
| W-H3 same | **`refuted`** — same condition, same result |
| W-Q4: layout-IOCTL scheme answer for `hybrid-mbr-gpt-512` against the `mbr-basic-512` control | **`other (verbatim)`** — no scheme is reported for either: both are absent from `MSFT_Disk` and their layout IOCTL fails `ERROR_IO_DEVICE` (Win32 1117) through a succeeding zero-access open. Nothing flags the aliasing |
| Complete `MSFT_PhysicalDisk` / `IsReadOnly` surface retention for all seven fixtures | **`observed`** — full property bags retained for all seven; `MSFT_PhysicalDisk` reports one fixture-matched row for **every** fixture, including the two absent from `MSFT_Disk`, and `IsReadOnly` is `True` on every enumerated disk |

#### Rerun sitting, 2026-08-04 — valid; all three gates satisfied

Taken 2026-08-04 on Windows build 10.0.26200.0 at repository revision
`6b57a53`, seven fixtures attached strictly one at a time. Two consoles with
opposite, separately recorded privilege assertions, as the parent protocol
requires: the attach console asserted `IsInRole(Administrator) = True` and did
only `Mount-DiskImage -Access ReadOnly -NoDriveLetter` and
`Dismount-DiskImage`; the measurement console asserted
`IsInRole(Administrator) = False` **and** `token carries Administrators group:
False` — the two checks recorded separately, because a filtered token can
carry the group. The measurement console was launched ordinarily, not as a
child of the elevated one.

**The three added gates, satisfied.** *R1* — every queried property value is
retained verbatim; the record is a 48 KB property-bag dump, not a summary,
and the 2026-08-02 query-and-discard defect does not recur. *R2* — every
fixture's pre-attach and post-detach VHD digests were taken and compared
individually, and all seven post-detach digests equal their pre-attach pair,
so "the bytes were not altered" is measured rather than documented. *R3* —
both fixtures absent from `MSFT_Disk` had the zero-access layout IOCTL run at
every `Win32_DiskDrive`-supplied index in the same session; no index was
available and left unprobed.

**W-H1, W-H2, W-H3 — all three refuted.** The refutation was evaluated
mechanically rather than by eye: 76 retained `MSFT_Disk` and `MSFT_Partition`
fields were compared field-by-field between the healthy control and each of
the conflicting, damaged-primary, and missing-backup fixtures, excluding a
named list of session-local addressing fields. **Exactly one field differs in
each comparison, and it is `Location` — the backing file's path**, which is
by-name provenance of which file was mounted and not a property of the disk.
Every state surface the hypotheses name is equal: `IsOffline` `False`,
`OfflineReason` `0`, `OperationalStatus` `53264`, `HealthStatus` `0`,
`IsReadOnly` `True`, `PartitionStyle` `2`, `NumberOfPartitions` `2`, and the
disk `Guid` `{7a1e9153-bef6-4752-9460-8c23898f2cbf}` identical across all
four. The partition rows are identical too — same `GptType`s, same partition
GUIDs, same offsets, same sizes — which under W-H1's own wording means **the
primary table was parsed** and presented without complaint. The layout IOCTL
agrees: `PartitionStyle=1 PartitionCount=2 bytes=336` for all four.

**A surface the parent run discarded, now retained.** `MSFT_PhysicalDisk`
reports one fixture-matched row for **all seven** fixtures — including
`mbr-basic-512` and `hybrid-mbr-gpt-512`, which have no `MSFT_Disk` row at
all. The enumeration gap is therefore specific to `MSFT_Disk`; the layer
beneath it sees every fixture. The pattern is exact: the two fixtures
carrying an MBR partition table are absent from `MSFT_Disk` and their layout
IOCTL fails, while `blank-512` and the four GPT fixtures enumerate and their
IOCTL succeeds.

**Artifacts and a recording-hygiene note.** The raw retained transcript
(`31-measure-full.txt`, SHA-256
`7135b19ab7efa05c6ef87640a95793fe27d771acf49340b9283e2ec3fa9ab970`) contains
operator path elements, because `MSFT_Disk.Location` is one of the values R1
requires retaining. That is a **conflict between R1 and this file's
no-operator-paths recording rule**, and it is resolved the way the protocol
resolves embedded paths elsewhere — by elision, not by dropping the field: a
redacted citation copy (`32-measure-redacted.txt`, SHA-256
`b46b27f22cd035e4d02a4ff03311799e29d26ed2e679129391c14b47e0dea71c`) carries
the same content with the profile prefix and username elided, and the raw copy
stays local: its digest above binds it, and no quotation or derived value in
this record is drawn from it rather than from the redacted copy. The
measurement script now elides at capture time so
a future sitting produces no such conflict. Attach transcript
`65b7183cf0ec0c74d533186f125113fc9a56d7510b524914b9e3885fa38cfa58`; fixture
digest table `7d3ac9e6d1e94cd6d6c3f599a66d5ca762b812542e5135aa661e718339c1112d`.
The four artifacts are archived at
`%USERPROFILE%\partman-evidence\SI-35-windows-rerun\` on the operator
workstation, custodian Nate McBride. Second-reader readback, required
before this record is relied on: performed on the result pull request
under the operator's recorded designation (by the producing session, so
not independent, and recorded as such there), and again on 2026-08-04 by
an independent reader session that retrieved all four artifacts through
the locator and rehashed each to its recorded digest, with the redacted
copy spot-checked to contain no operator username. Both readbacks
matched; the requirement is discharged.

**What this sitting does not do.** It decides no SI-35 option and supplies no
chosen-option refusal proof. It refutes three *existential surface*
hypotheses over the enumerated interfaces only — no claim is made about
interfaces not enumerated here. W-H2's which-GPT-copy question stays
unmeasured, as the declared limit below already states.

Declared limits: W-H2's which-GPT-copy question remains unmeasured even by
a valid rerun — both copies of that fixture describe identical partitions,
and the discriminating fixture variant that would separate backup use from
primary parsing belongs to WP-020's catalogue, recorded here as a boundary
rather than preregistered across a package boundary. Transcript custody
and second-reader readback are as the increment 6 matrices define. A valid
rerun makes the three declared refutation conditions evaluable and answers
W-Q4; it decides no SI-35 option and supplies no chosen-option refusal
proof.

### SI-33 media-change-counter liveness — measured 2026-08-02

Protocol recorded 2026-08-02 under spec version 4.2.0 by WP-035 and taken the
same day, in two runs on the hardware described under Apparatus.

**Status: the register's full liveness sequence is not established.** Immediate
and idle-gap exchanges moved in every recorded trial, but the useful
fresh-handle leg kept the original handle open across the physical event. The
required close-before-event/reopen survival arm was not taken. A lower reading
in the later run, across an interval containing a PnP arrival, separately shows
that global monotonicity cannot be assumed; the counter epoch and reset cause
were not characterized.

The register's sequence is three-part: exchange the medium and assert the
immediate re-read moved; repeat with a sixty-second idle gap to detect
poll-driven behaviour; close the handle **before** the event, reopen afterward,
and assert the value survives. The retained results are:

| Part | Retained result |
| --- | --- |
| immediate re-read moved | L1: **3/3** in run β |
| sixty-second idle repeat | L2: **3/3** in run β |
| close-before-event/reopen survival | **unmeasured**. L5b established only that a fresh handle could read the moved value once while the original handle had remained open across the event; L5a was floor-to-floor and uninterpretable |

Run α had one card, so L1/L2 were `not runnable — single medium`; run β's L5a
was uninterpretable. The sequence was therefore neither executed end to end
nor established by composition. L4 moved in its **sole retained trial (1/1
taken; 3 requested, 2 unrun)**, so its replication requirement is unmet; the
one observation remains a liveness warning recorded in its cell.

Two further things travel with the result and may not be dropped from any
retelling of it:

- **The ceiling this protocol declared before any data existed.** Prompt
  movement **cannot** be attributed to exchange-synchronous detection,
  because a background poll could equally have produced it. The strongest
  recordable positive is *no staleness observed under these conditions* — for
  the slot-exchange family, on one reader, on one bridge, on one build.
- **The exposed reading is not globally monotone.** Run β read a value below
  one run α had already passed, across a boundary containing a timestamped PnP
  arrival. That measured decrease makes an equality-only witness unsafe across
  such a boundary. The documented “since the driver started” baseline makes a
  reset plausible, but the run did not characterize a driver-incarnation token
  or establish causation. See “The decrease, and why it matters”.

Cells still reading `not yet taken` are unrun legs, and this section extends
this file's rule of use to that vocabulary: such a cell MUST NOT be relied on,
cited, or paraphrased as a finding — by anything, not only by an ADR that
freezes canonical bytes.

#### The sittings

All non-elevated: completion past the scripts' guard establishes
`IsInRole(Administrator)=False`. The executed split scripts did **not** log
Administrators-group membership. Windows build 10.0.26200.0, one host;
timestamps are the transcripts' own headers except where stated.

| Sitting | Run label / counter context | Legs taken | Media available |
| --- | --- | --- | --- |
| 1a, before 10:25:33 | α | H matrix, L5a, L6a — operator-reported as taken interactively, but no transcript was retained; its floor claim is unavailable for evidence | one card |
| 1b, from 10:25:33 | α | L3, L4, L5b, L6b, L7; L1 and L2 recorded `not runnable — single medium` | one card |
| 2, from 10:37:59 | β — lower exposed reading after a PnP arrival; driver epoch uncharacterized | L5a re-run, L1 ×3, L2 ×3 | **two cards** |

Because 1a has no retained transcript, it supplies no evidentiary continuity
claim with 1b. The **one-card/two-card asymmetry is why the leg table looks as
it does**: every exchange leg belongs to sitting 2 and every retained
same-medium leg to sitting 1b.

Sittings 1 and 2 are **not** one series. The later exposed reading was lower;
no delta spans the boundary, and “counter restarted” remains an inference
rather than a measured epoch identity.

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
| F2 | the second identical-model USB flash drive, added to the H notes and used for no liveness leg |
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

#### H — which handle can even ask (operator report; no durable transcript)

For each combination below: attempt the open; on success issue V2 once, then
V1 once, in that order. V2-before-V1 is a discipline, not a convenience: V1's
access class marks it as the device-reaching variant, so probing it first
could refresh the very state V2 is suspected of serving stale.

The operator reported taking this matrix on 2026-08-02 after the
`IsInRole(Administrator)=False` guard, on Windows build 10.0.26200.0 and the
reader/flash drives under Apparatus. **No transcript was retained**, and the
split script did not record Administrators-group membership. Every cell below
is therefore an operator report preserved as run history and is **unavailable
for durable evidence**. `F2` is an operator-reported addition to the protocol's
matrix because two identical-model drives were present.

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

Three narrower facts can be separated from those unavailable matrix notes:

- **The retained liveness transcripts independently show one zero-access data
  source.** L1/L2 sampled V2 successfully on the chosen zero-access reader
  handle. They do not reproduce the matrix's per-subject access claims, so the
  operator-reported “every subject holding a medium” and paired V1 results are
  unavailable as evidence.
- **A read-access handle exists, but only by the volume path.** `GENERIC_READ`
  on the removable volume and V1 success are retained in L3. The matrix's
  reported refusals on every `PhysicalDrive` are not durably evidenced here;
  whether the volume result generalizes is unmeasured.
- **The retained empty-slot fact comes from L6b, not this matrix.** The reader
  LUN after card removal returned `ERROR_NOT_READY` with zero count bytes. The
  LUN observed medium-less and kept empty for this run has H/L6a details only
  in operator notes; no transcript was retained.

**An operator-reported side observation, unavailable for durable evidence.**
The notes say the medium-less LUN appeared in `MSFT_Disk` (size 0, partition
style 0) but had no `MSFT_PhysicalDisk` row. With no H transcript, this cannot
correct the roster fallback or support a platform claim; a future retained run
must ask it again.

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
| Elevation assertion | `IsInRole(Administrator)=False`, established by successful completion past the guard; group membership not recorded | same |
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
that one trial cannot exclude a background poll having moved the counter, so
the reason stands for L4 without relying on an unretained observation.

| Leg | Trials asked / taken | Record | Result |
| --- | --- | --- | --- |
| L1 immediate exchange | 3 / 3 (sitting 2) | Δ across exchange at immediate re-read, per trial | `count Δ=+1` in **all three trials**, every swap fingerprint-validated as a genuinely different card. In sitting 1b: `not runnable — single medium` |
| L1 | 3 / 3 (sitting 2) | status of first post-exchange sample, per trial | success with a count in all three; the documented withheld-count path never fired, and each bracket at +5 s held the same Δ |
| L2 sixty-second idle | 3 / 3 (sitting 2) | Δ across exchange after 60 s hands-off | `count Δ=+1` in **all three trials**, each from a single sample with no probe in the preceding minute. In sitting 1b: `not runnable — single medium` |
| L3 own bracket | ≥1 / 1 | Δ step 1 (pre) vs step 3, per round — the "moved only after" antecedent's own cell | `count Δ=+1` — moved **before** any forced I/O |
| L3 forced-I/O, filesystem arm | ≥1 / 1 | Δ step 3 vs step 5a | `count Δ=0` |
| L3 forced-I/O, V1 arm | ≥1 / 1 attempted, but not as specified | Δ step 3 vs step 5b, and whether a read handle existed | `count Δ=0`; a volume-path read handle existed and V1 succeeded, but the transcript ran V1 **after** filesystem I/O and step 5a in the same round instead of replacing them in a second round. The value did not move at either sequential sample, but this does not independently de-confound the V1 arm; rerun as specified before relying on it |
| L4 same-medium out-and-back | 3 / **1** | Δ across removal and reinsertion at immediate re-read | `count Δ=+1`, already final at the immediate re-read; `Δ=+1` unchanged at +5 s and after 60 s idle |
| L5a reopen, quiescent | ≥1 / **1 retained** | Δ across close/reopen, with sign | sitting 2 was `count Δ=0` at the floor, where “survived” and “reset” are indistinguishable, so it carries no information. Sitting 1a was operator-reported floor-to-floor but has no retained transcript and supplies no evidence |
| L5b reopen, across a same-medium out-and-back | ≥1 / 1 | Δ visible from the fresh handle, with sign | one fresh-handle sample, `Δ=+1` versus L4's pre-removal sample, sign positive: the value the held handle had reported was still visible from a newly opened handle, so the count is **not per-handle state**. Two deviations recorded rather than smoothed: there was **no A→B exchange** — sitting 1b had one card, and the physical action was L4's same-medium out-and-back — and the leg's specified order (sample, **close**, physical action, reopen, sample) was not followed: a handle stayed open across the action and the fresh handle was opened afterwards. Survival with no handle open across the event, and survival across a true exchange, are both **unmeasured**. |
| L6a empty LUN | 1 / 1 operator-reported; no transcript | open and probe statuses; count bytes present or not | **unavailable for durable evidence**. Operator notes say opens at `0x00000000` and `FILE_READ_ATTRIBUTES`; `GENERIC_READ` `open refused 5`; V2 `error 21`, no count bytes; V1 `error 5` |
| L6b card LUN while empty | 1 / 1 | probe status with no medium | V2 `error 21`, 0 bytes. The operator reported the same for R-empty, but L6a has no retained transcript and is not evidence |
| L7 surprise removal | 1 / 1 | held-handle status after removal; after reinsertion; fresh-handle reading | held handle `handle dead 55` after removal and **still dead after reinsertion**; the pre- and fresh-handle readings were both at the floor. No delta is formed across the PnP arrival because the counter epochs may differ. `DEVPKEY_Device_LastArrivalDate` moved to 10:28:03, establishing a PnP arrival; the PNPDeviceID persisted. Neither fact characterizes the counter epoch, so whether detach ended the counting entity remains unmeasured |

**An unretained operator observation does not bound L4.** The operator reported
that the displayed count advanced between sitting 1a and 1b, but 1a has no
retained transcript and no sample brackets the interval. It is historical run
context only: no within-instance or causal claim is formed from it. L4 remains
one trial rather than three on the simpler, pre-declared ground that one trial
cannot exclude a coincident background poll.

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

The final conditional is preserved as the pre-run map, but the actual L5
execution did not instantiate its close-before-event/reopen antecedent and L4
did not reach the requested replication. It is therefore superseded as a pass
criterion for this run and cannot be read as satisfied by “every trial taken”.

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
| H yields no counting handle | **no, for the selected reader path only** — retained L1/L2 transcripts show V2 returned a count through that zero-access handle. The unretained H matrix cannot establish the other subjects |
| counts only via V1 | **no, for the selected reader path only** — retained liveness legs used V2 at zero access. The operator-reported paired V1 matrix is unavailable for durable evidence |
| L1 unchanged, L2 moved | **no** — L1 moved in all three trials |
| L1 and L2 unchanged, L3 moved only after forced I/O | **no** — L3 moved on its own bracket **before** any forced I/O. The later filesystem and V1 samples each stayed flat, but the V1 step was not run as the specified independent arm |
| L1, L2, L3 all unchanged | **no** |
| L4 unchanged | **not observed in the sole retained trial** — L4 moved on that same-medium return, but 1/1 taken of 3 requested does not satisfy the replication requirement |
| L5 delta negative on reopen | **not evaluated in the specified order** — L5b's fresh handle saw the moved value, but the original handle stayed open across the event; L5a's retained run sat at the floor |
| L7 handle dies and fresh-handle delta negative | **not evaluated by a valid delta** — the handle died and stayed dead, both readings sat at the floor, and a PnP arrival lies between them. LastArrival establishes arrival, not the counter epoch or whether detach ended its counting entity |
| **pass row**: L1, L2, L4 moved and L5 stable | **no** — L1/L2 moved 3/3 and L4 moved 1/1, but the required close-before-event/reopen conjunct is unmeasured. The register's full liveness sequence did not pass |

The one thing no row anticipated is the lower later reading, and it matters
even though the pass row did not fire.

#### The decrease, and why it matters

**Measured, and stated at the strength the evidence carries.** Sitting 1b
ended with the counter at a recorded value. Sitting 2's first sample, on the
same reader, read **lower** — back at its floor. The reader's
`DEVPKEY_Device_LastArrivalDate` is timestamped 10:32:09, between 1b's last
recorded device event (10:28:03) and sitting 2's first sample (10:37:59), so a
device arrival falls inside the interval that also contains the drop.

LastArrival supplies an event marker independent of the count, so the ordering
is not derived from the decrease. It is **not** a driver-incarnation token and
does not causally attribute the decrease. The data support one timestamped PnP
arrival and one lower later reading in the same interval (n=1); the trigger was
not recorded and no leg varied it. A driver reset is plausible from the WDK's
“since the driver started” baseline, but remains an inference.

**Two PnP arrivals were timestamped.** Besides the reader's interval, the L7
drive's LastArrival moved to 10:28:03 across deliberate surprise removal. Only
the reader interval also contains a lower later count; L7 sat at the floor on
both sides. Neither observation identifies a driver epoch.

**Why it matters more than the pass.** A witness exists to answer one question
at apply time: *was the medium exchanged since the plan was made?* The
proposed test is a comparison between a recorded reading and a fresh one, and
that test is sound only if readings from different epochs are rejected or the
value is globally monotone. The measured decrease means global monotonicity
cannot be assumed.

**Constructed scenario, built from one measured property.** Two of its four
steps correspond to measured legs; the other two describe a design that does
not exist and were not exercised — no plan, no apply, and no equality test
appears anywhere in this run:

| Step | Status |
| --- | --- |
| a plan is made in a hypothetical fresh counter epoch, at the floor | **constructed** — no plan-time reading or epoch identification was performed |
| the medium is exchanged and the counter moves | **measured** (L1, L2, L4) |
| a PnP arrival occurs and a later sample is lower | **two co-occurring measured facts**; a causal reset/epoch link is inferred, not measured |
| apply reads the floor, compares equal, and concludes the medium never moved | **constructed** — no apply-time comparison was performed, and that a post-boundary reading lands on the same value a plan recorded is assumed, not observed |

So the claim is not that this happened. It is that **a design of the proposed
shape can fail open in the constructed scenario**, silently, while carrying a
field implying the check was made — the shape of harm SI-33's filing warns
about, *"worse than no witness, because it converts an admitted gap into a
false assurance"*. The run establishes one decrease across an uncharacterized
boundary; it does not establish a repeated floor equality across plan/apply or
an actual witness failure.

**Stated exactly, because the difference is load-bearing.** The decrease does
not show the counter is a bad *detector* — it moved in every retained exchange
leg. It shows exposed readings are not globally monotone, so equality alone is
not evidence of non-interruption across an uncharacterized boundary. A
surviving design needs a separately characterized epoch/incarnation signal so
cross-epoch readings are **incomparable**, or another witness entirely.

**No instance-distinguishing signal was characterized.** Run 2's
`same-driver-instance=False` value was computed from the lower counter itself,
so it cannot independently explain that decrease. L7's PNPDeviceID persisted
across replug, while LastArrival marks an arrival but does not identify a driver
incarnation. Finding and characterizing an unprivileged, stable epoch signal —
not merely naming one of these fields — is the successor experiment.

**A gap in the protocol, recorded against the protocol.** L5b changed handles
but left one handle open across the event; quiescent L5a was at the floor. The
required no-handle-across-event boundary and a characterized device/driver
epoch were both missed. A future revision must pre-register both rather than
derive continuity from bookkeeping between runs.

#### What this protocol cannot establish, declared now

- Anything beyond this one reader and bridge. The SI-28 finding that the
  bridge synthesizes the storage-layer serial stands as a warning here: the
  counter may equally be bridge behaviour, so neither a pass nor a refutation
  generalizes to a second reader, a native SD host, or any other device class
  until one is measured. The bridge measured on 2026-08-02 enumerates as USB
  vendor `0BDA` behind a whitelabel model string. That VID supports only a
  same-vendor/different-vendor comparison; many bridge models and units share
  it, so it cannot establish same-bridge model or same unit. **This is a
  declared exception to the role-labels-only rule below**, taken because a USB
  vendor id names a chip vendor rather than a unit: it distinguishes no two
  devices, identifies no host, and carries none of the session history the
  deltas rule exists to keep out. A future cross-reader protocol may record the
  non-unit-identifying VID:PID pair to compare bridge models; product and
  unit-level identifiers were not retained in this run.
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
- **Whether a driver-instance identity exists that would make an epoch change
  detectable.** The lower later reading makes that the successor question;
  this run characterized no such identity, and makes no claim that one is
  available, stable, or readable unprivileged.
- **What the counter does across suspend, hub reset, or a reboot.** Two
  PnP arrivals were timestamped — the reader's between the sittings, and the
  L7 drive's from its surprise-removal cycle — but only the reader interval
  also contains a lower later reading, and the trigger is unrecorded.
  Suspend, hub reset, and reboot are untested.

### SI-33/SI-28 successor protocol — preregistered 2026-08-02

Protocol recorded 2026-08-02 under WP-035's SI-33 and SI-28 shares. Status:
**every arm is executed** — S4 across sittings 2 and 3 (2026-08-04), and
S1, S2, S2b, and S3 in the same day's S1–S3 sitting recorded below them;
sitting 1's cross-model `not established` stands for that sitting's pair.
The preregistered text remains the instrument the results bind to; the
dated sitting records are the evidence. It preregisters exactly the
successor questions the records above filed — the close-before-event/reopen
arm the 2026-08-02 run missed, the epoch-signal characterization its decrease
demands, L4's unmet replication, and the second-reader collision question the
SI-28 record marked "worth measuring" — and nothing else; executing it may
not add arms. The collision test was previously pre-registered only in
conversation (recorded in the 2026-08-02 handoff with its hypothesis,
refutation condition, live-comparison requirement, and
enumeration-failure-is-data rule); this section is its repository
preregistration, and the conversational one confers no standing of its own.

Everything the parent protocol declared travels unchanged: non-elevated
throughout with the elevation assertion recorded per sitting; the ceiling
declaration (prompt movement cannot be attributed to exchange-synchronous
detection; the strongest recordable positive remains *no staleness observed
under these conditions*, per reader, bridge, and build); role labels only,
with the parent's declared non-unit VID:PID exception for bridge-model
comparison; the mounted-disk-image non-substitution rule; and this file's
rule of use extended to `not yet taken` cells. Result vocabulary is the
closed set the increment 6 matrices define, plus this record's own
`moved` / `unchanged` / `count-absent` counter outcomes. Transcript custody
is identical to the increment 6 matrices: outside-repository archive with
locator and custodian, hash algorithm/digest/byte length, capture-script
digest recorded before the first sample, per-command exit statuses, and
second-reader retrieve-and-rehash before any cell leaves `not yet taken`.
Gate failures make cells `void(<gate>)`, never negatives. New legs carry
S-numbers so no result can be conflated with the parent run's L-legs.

**S1 — close-before-event/reopen survival, with a true no-handle window.**
Read the counter, then close **every handle this process holds on the
device** and assert process-local closure before the event; the assertion's
scope is process-local and is recorded as such — system-wide handle absence
is not claimable unprivileged and is not claimed. Exchange the medium with
the empty-slot assertion between removal and reinsertion (the discarded-L4
audit rule), reopen fresh, and read. Three trials. Bracket every trial with
the reader's `DEVPKEY_Device_LastArrivalDate`: if the reader's own arrival
timestamp moves inside a trial, that trial is `void (epoch boundary)` —
survival and reset are indistinguishable across one. Permitted outcomes per
trial: `moved` (the count survived the no-handle window and registered the
exchange), `unchanged` (**the fail-open signal** — the returning medium
reads as never exchanged; this outcome is the experiment's reason to exist
and must be recorded at full strength), `count-absent`, or void.

**S2 — epoch/instance signal characterization.** The parent run's
`same-driver-instance=False` was computed from the lower counter itself and
is circular; the counter value is therefore **excluded as a candidate**.
Preregistered candidates, each read non-elevated: `DEVPKEY_Device_PDOName`,
`DEVPKEY_Device_ContainerId`, and `DEVPKEY_Device_LastArrivalDate` (the
last already known to mark arrival without identifying an incarnation — it
participates as a bracketing control, not as a proposed token). A candidate
**qualifies** only if all four hold: readable non-elevated; stable across
quiescence and every same-instance sample; changed across **every** induced
re-arrival; and computed independently of the counter. Induce three reader
re-arrival boundaries (deliberate surprise-removal and reattach of the
reader, the run 1/2 boundary reproduced on purpose, trigger recorded this
time), sampling every candidate and the counter immediately before and
after each. One additional reboot boundary is preregistered as S2b with the
same samples. Extending the candidate list is a protocol revision, not a
run-time choice. A qualifying signal makes cross-epoch readings
incomparable by construction; none qualifying is itself a recordable
result and narrows SI-33 toward its another-witness-entirely arm. Nothing
here decides SI-33's design.

**S3 — L4 replication completion.** Two further trials of the L4
same-medium out-and-back leg exactly as defined above — the definition is
not restated here, so it cannot drift — bringing the leg to its originally
requested three trials, each with the empty-slot assertion and idle
re-reads the definition requires.

**S4 — second-reader collision test (SI-28 share; conditional).** Runs only
when a second reader of the same model is available, with model sameness
established through the parent record's declared non-unit VID:PID
exception plus model string, and unit distinctness through the USB
descriptor serial. **Hypothesis under test:** the storage-layer serial is a
bridge firmware constant, predicting the same `2012…5300`-form value from
two distinct units. **Refutation condition:** the second unit reports a
different storage-layer serial. **Live comparison is required:** both
readers attached simultaneously, serials read in one sitting — a recalled
or transcribed value from another session refutes nothing.
**Enumeration failure is data:** if simultaneous attachment makes either
reader fail to enumerate, merge, or re-identify, that outcome is recorded
as the collision behavior it is, not as a failed run. Two further rows
ride only if the primary comparison completes: the empty-slot serial
comparison (the parent record's strongest single-reader form, repeated
across units), and one card moved between readers to record whether the
storage-layer record follows the card or the reader. Without the second
unit every S4 cell stays `not established`, not approximated with a
different model.

| Arm | Result |
| --- | --- |
| S1 close-before-event/reopen survival, 3 trials | `moved` in **all three trials** — `count Δ=+1` per exchange, each read from a fresh handle after a process-local no-handle window, every empty-slot assertion and arrival bracket clean; the fail-open `unchanged` outcome never appeared (sitting of 2026-08-04, below) |
| S2 epoch-signal candidates across 3 induced re-arrivals | storage-node PDO name **qualifies** — changed across every boundary, stable between them, readable non-elevated, counter-independent; USB-node PDO name and ContainerId **refuted** (unchanged across all three); the boundary-1 counter stood five events above the epoch floor before the re-arrival and read at the floor after it — the reset measured |
| S2b epoch-signal candidates across one reboot | storage-node PDO name changed across the reboot as well; ContainerId constant even across it; the USB-node PDO name changed here but was already refuted by the replug boundaries; boot-time bracketed in-transcript |
| S3 L4 trials 2 and 3 | both `count Δ=+1`, already final at the first post-reinsertion read, unchanged at +5 s and after 60 s idle, empty-slot assertion per trial — L4 reaches its originally requested three trials together with 2026-08-02's trial 1 |
| S4 two-unit storage-layer serial live comparison | `observed(identical constant)` — sitting 2, 2026-08-04: two same-model units, live; one storage-layer serial value on every LUN of both — the hypothesis's predicted collision. Sitting 1 (2026-08-03): `not established` for that cross-model pair |
| S4 empty-slot serial across units; card moved between readers | empty-slot arm `observed(identical constant)` — each unit's medium-less LUN reports its loaded LUN's serial, one constant across units; card-move arm `observed(serial invariant under exchange)` — sitting 3, 2026-08-04: the exchange is visible only as media facts; on this shared-constant pair follows-card versus follows-reader is undecidable by value and unit continuity is unverifiable (see the sitting-3 record) |

A valid run can establish only these arms. It decides no SI-33 design
option, does not move SI-28's disposition (the register owns status), and
generalizes to no other reader, bridge, host, or device class than the
units measured.

#### S4 sitting, 2026-08-03 — same-model precondition unmet

A custody-complete sitting attempted S4's primary row against the two
simultaneously attached reader units. It could not run the comparison: the
units are different bridge models, and S4's own rule — `not established`,
never approximated with a different model — governs. No part of the
collision hypothesis is evaluated; the recorded fact is that the same-model
second unit S4 requires does not yet exist in the kit, established by
measuring the delivered unit rather than by its absence. The arm stays
takeable exactly as preregistered the day a second unit of either present
model is attached.

Read non-elevated in one session, both units enumerated, each presenting
two LUNs, no problem codes:

- One unit reports the non-unit VID:PID `0BDA:0306`, model string
  `Generic- USB3.0 CRW -SD` (interior whitespace collapsed here; the
  transcript holds it verbatim), and one storage-layer serial value (elided
  `2012…5300`) on both LUNs, USB descriptor serial elided `2015…1013`,
  holding the 256 GB-class card. Every one of those retained forms and the
  card's exact byte size match the parent record — but the parent run kept
  no unit-level identifiers, so continuity with the 2026-08-02 unit holds
  only at elided-form strength and no full-string identity is claimed.
- The other unit — operator-reported as an outwardly identical enclosure,
  a claim no software surface here can check, holding the
  64 GB-class card — reports VID:PID `2537:1081`, per-LUN model strings
  `1081CS0`/`1081CS1`, bus-reported description `NS1081`. Model sameness
  fails on both of S4's declared establishment surfaces at once.
- Context, not an S4 result: the second unit's USB descriptor serial and
  storage-layer serial are the same 15-character ascending-hexadecimal
  placeholder (elided `0123…BCDE`), identical across its two LUNs and
  across both layers. That is the SI-28 bridge-constant form appearing on
  a second, different bridge family. Whether two units of that family
  collide is precisely S4's question, and it needs two units of it.

Custody: capture script and complete transcript archived outside the
repository at
`%USERPROFILE%\partman-evidence\2026-08-03-si33si28-s4-sitting1\` on the
operator workstation; custodian Nate McBride. SHA-256 throughout.
Capture-script digest, recorded in the transcript header before the first
device query:
`80dd4b49f79b595e69250cb9c0b689f721227e377e9c08f75b046547ac20a6ec`
(8742 bytes). Transcript:
`421a3c09d15d2313f940287bb8a94b1a74e94a86375b970b2910b711a9ade980`
(10254 bytes); all eight steps `OK`, script exit 0; elevation assertion
in-transcript (`IsInRole(Administrator)=False`, token carries no
Administrators SID); OS build 10.0.26200; PowerShell 7.6.4. An independent
second reader retrieved the transcript through the locator and rehashed it
to the same digest and byte length before either cell left
`not yet taken`.

#### S4 sitting 2, 2026-08-04 — collision observed on a same-model pair

The operator installed the ordered second unit and designated the pair for
S4. Both attached units are the NS1081 bridge model — VID:PID `2537:1081`,
per-LUN model strings `1081CS0`/`1081CS1`, bus-reported description
`NS1081` on both — so model sameness holds on both declared establishment
surfaces. The parent-record `0BDA:0306` unit is absent this sitting, and
sitting 1's recorded rule (a second unit of either present model restores
the arm) is the standing under which the arm ran on this pair. The
hypothesis's `2012…5300`-form exemplar was the departed anchor's own
constant; what this pair evaluates is the hypothesis's structural claim —
one bridge-firmware constant shared by two distinct units of one model —
and, per this protocol's closing rule, the result generalizes to no other
model.

**Primary row — the predicted collision, observed.** In one non-elevated
session with both units attached, every serial-bearing surface returned
the same value: `Win32_DiskDrive.SerialNumber`, `MSFT_Disk.SerialNumber`,
and `MSFT_PhysicalDisk.SerialNumber` on all four LUNs of both units carry
the model's 15-character ascending-hexadecimal placeholder (elided
`0123…BCDE`), and the same string appears at the USB-descriptor layer of
both units. The refutation condition — the second unit reports a
different storage-layer serial — did not occur.

**Unit distinctness, and the named instrument's failure as the finding.**
S4 names the USB descriptor serial as the distinctness instrument. It is
unavailable on this pair for the reason under test: both units present
the same serial, and Windows's duplicate handling is itself visible on
three surfaces — the first-arrived unit's USB node is keyed by the serial
while the second, arriving nine seconds later, is keyed by the
port-derived fallback form; the second unit's USBSTOR children carry a
generated uniquification prefix ahead of the same serial string; and
`CM_DEVCAP_UNIQUEID` (cfgmgr32.h 10.0.26100.0, `0x10`) is set on the
first-arrived node and clear on the second. Distinctness is instead
established by simultaneous enumeration: two present USB nodes on
distinct ports of one hub, distinct addresses, distinct container ids,
arrival timestamps nine seconds apart, each owning its own two-LUN
storage stack with different-size media — the 64 GB-class card on the
first-arrived unit, the 256 GB-class on the second, bound to operator
drive letters in-transcript. This substitution is declared rather than
smoothed: reading the establishment clause as requiring distinct serials
before the comparison may run would make the collision outcome
unrecordable by construction, and the preregistration's
enumeration-failure-is-data rule states the opposite intent — merged or
re-keyed identity under simultaneous attachment is the collision
behavior, recorded as such.

**Empty-slot rider — observed.** Each unit's medium-less LUN (`MSFT_Disk`
size 0, no partitions) reports the same serial as its card-bearing LUN —
the parent record's single-reader form, reproduced on each unit — and the
value is the one constant across units.

**Card-move rider — not taken.** It requires physically moving one card
between the units within one sitting. Context recorded without filling
it: across sittings 1 and 2 a 256 GB-class card of byte-identical size
moved, operator-reported, from the `0BDA:0306` unit to an NS1081 unit,
and the storage-layer serial reported for it changed from that bridge's
constant to this one's — consistent with the record following the reader
— but Windows exposes no medium-attributable identifier (the SI-28 record
above), so cross-sitting medium identity rests on size alone and this
fills no cell.

Nothing here moves SI-28's disposition; the register owns status.

**Corrections registry for this sitting, each caught before any cell
moved.** (1) The transcript header mis-dates the sitting 2026-08-03; the
capture script's own clock and both units'
`DEVPKEY_Device_LastArrivalDate` establish 2026-08-04, and an appended
in-transcript correction governs. The archive was staged under the
mis-dated name and relocated, the move recorded in-transcript before the
final digest. (2) A supplementary distinctness query's first version
derived its unique-id line from a recalled constant — `0x40` is
`CM_DEVCAP_RAWDEVICEOK` — and is instrument failure. (3) Its second
version printed a decode contradicting its own retained raw value
(`0x94` reported without `UNIQUEID`), arithmetically impossible, cause
unestablished: instrument failure again. Raw property values from both
stand; the qualifying derivation is version 3's, which decodes with
per-bit literals verified against the installed `cfgmgr32.h` and refuses
to observe unless its decoder passes a self-test on the two values in
question.

Custody: capture script (byte-identical to sitting 1, same digest
`80dd4b49f79b595e69250cb9c0b689f721227e377e9c08f75b046547ac20a6ec`,
8742 bytes, recorded in the transcript header before any device query),
orientation script
`aad092f163ec70d21b37abcbab0d22cb3fbd52836f570e44fce22143678a3950`
(932 bytes), distinctness v3
`e475bc6b38268db284bfb8b827619bc3c44fc7ca9dd7986ebc9d18ded612dec7`
(4066 bytes; v1 and v2 retained in the archive with digests
in-transcript), and complete transcript
`aec8a3d3e9cfee42c15ac635dd1a2a6d9d7c7037a43d77f85f4771c1fbcb671f`
(16479 bytes) archived at
`%USERPROFILE%\partman-evidence\2026-08-04-si33si28-s4-sitting2\` on the
operator workstation, custodian Nate McBride; SHA-256 throughout;
per-step statuses and per-script exit codes in-transcript; elevation
asserted in-transcript (`IsInRole(Administrator)=False`, token carries no
Administrators SID); OS build 10.0.26200; PowerShell 7.6.4. An
independent second reader retrieved the transcript through the locator,
rehashed it to the same digest and byte length, and confirmed the
archive's file inventory before any cell left `not yet taken`.

#### S4 sitting 3, 2026-08-04 — card-move rider: the exchange is invisible except as media

The operator exchanged the two cards between the attached units — a swap,
which contains the preregistered one-card move in both directions and is
declared as the executed form — and reported the readers stayed attached
throughout. Instruments byte-identical to sitting 2, digests recorded in
the transcript header before any device query.

Observed, in one non-elevated session:

- Every serial-bearing surface on every LUN of both units reports the
  same constant as sittings 1 and 2: the storage-layer record is
  **invariant under the exchange**. The exchange is visible only as media
  facts — the by-port card mapping swapped (the port that held the
  64 GB-class card now holds the 256 GB-class and vice versa), sizes
  travelling with the cards, and each card's volume followed its card to
  the other unit, bound in-transcript.
- On a pair sharing one constant at every layer, **follows-the-card
  versus follows-the-reader is undecidable by value**. The attribution
  that the serial is the reader's rests on the empty-slot form, which
  held again this sitting on both units: both medium-less LUNs still
  report the constant.
- **Both units re-arrived inside the exchange window** — their arrival
  timestamps moved from the sitting-2 values to two fresh times twelve
  seconds apart — although the operator reports the readers stayed
  attached. Whether this bridge re-enumerates on media change or the
  connections were disturbed by handling is not attributable
  unprivileged; the fact is recorded at full strength because of what
  follows from it.
- Across those re-arrivals the **serial-derived instance identity
  migrated to the other port**: the serial-keyed USB instance, on one
  port at sitting 2, is on the other port at sitting 3 holding the other
  card, while the second unit was re-keyed for its port with a fresh
  port-derived container id. The serial-keyed badge is a first-arrival
  artifact, not a unit identity, and physical-unit continuity across the
  exchange is **not establishable from any surface this sitting reads**:
  readers-stationary-with-cards-swapped and readers-swapped-with-cards
  produce byte-identical observations on this pair. Recorded per the
  enumeration-failure-is-data rule as the collision behavior's fullest
  measured reach.
- Context for S1's epoch rule, touching no S1 cell (S1 belongs to the
  parent apparatus): on this model a media exchange can coincide with a
  USB re-arrival — exactly the boundary S1's bracketing voids a trial
  for.

Nothing here moves SI-28's disposition; the register owns status.

Custody: capture, orientation, and distinctness-v3 instruments
byte-identical to sitting 2 (same digests, re-recorded in the header
before any query); complete transcript
`54459295fd2d97250bf0797186b7662d0d49b2c6b5219d6013673a73df4a76b1`
(11824 bytes) archived at
`%USERPROFILE%\partman-evidence\2026-08-04-si33si28-s4-sitting3\` on the
operator workstation, custodian Nate McBride; SHA-256 throughout;
per-step statuses and per-script exit codes in-transcript; elevation
asserted in-transcript (`IsInRole(Administrator)=False`, token carries
no Administrators SID); OS build 10.0.26200; PowerShell 7.6.4. An
independent second reader retrieved the transcript through the locator,
rehashed it to the same digest and byte length, and confirmed the
archive's file inventory before the cell left `not yet taken`.

#### S1–S3 sitting, 2026-08-04 — survival established, the reset measured, an epoch signal qualified

The operator reattached the parent-record reader — VID:PID `0BDA:0306`,
USB-descriptor and storage-layer serials matching the parent record's
elided forms (`2015…1013`, `2012…5300`), the same identification strength
sitting 1 declared — with the 64 GB-class card (Card A) seated, the second
slot kept empty, both NS1081 units detached, and the two flash drives
attached (byte sizes matching the parent apparatus's F1/F2). The sitting
was turn-based: every physical step is operator-reported in chat and
validated by a probe before the leg proceeds — the discarded-L4 audit
rule enforced at every removal — and "immediate" post-action samples
carry tens of seconds of turn latency, declared once in the transcript
header; the parent ceiling already forbids attributing promptness, and no
outcome below rests on it. The counter instrument is the parent
protocol's preregistered P/Invoke block run one short-lived process per
sample, so process-local closure between samples holds by process exit;
IOCTL constants were re-verified against `winioctl.h` 10.0.26100.0 at
staging. Candidate reads sample the three named S2 candidates on the
reader's USB node and each USBSTOR child, labelled per node — the named
list read on the apparatus's nodes, not an extension of it. Disk-number
remapping after each re-arrival is folded into the post-boundary sample,
keyed by instance id.

**S3 — L4 trials 2 and 3, both `count Δ=+1`.** Same-medium out-and-back:
the delta was already final at the first post-reinsertion read and held
at +5 s and after a 60-second hands-off idle, in both trials; the
empty-slot assertion returned the no-medium error mid-trial each time,
and the arrival bracket never moved. With 2026-08-02's trial 1, the leg
stands at its originally requested three trials, all `Δ=+1`.

**S1 — `moved` in all three trials; the fail-open signal never
appeared.** Each trial: a pre-exchange sample whose process exited before
the event (the true no-handle window the parent run lacked), the
empty-slot assertion mid-window (a transient probe handle, the
preregistered mechanic), a genuine A↔B exchange validated by size class
at the post-read, and a fresh-handle reading of `count Δ=+1`. No trial's
arrival timestamp moved, so no trial is `void (epoch boundary)`. On this
reader, bridge, and build, the counter survives a no-handle window and
registers the exchange — the register's close-before-event/reopen
survival sequence, established at three trials.

**S2 — the reset measured, and one candidate qualifies.** At boundary 1
the counter stood five exchange events above the epoch floor before the
reader's surprise-removal and reattach, and read at the floor immediately
after: the re-arrival reset is measured, giving the parent record's
unexplained lower-later-reading a demonstrated mechanism. Across the
three induced boundaries: the USB-node PDO name never changed (a
port-slot name — refuted), ContainerId never changed (serial-derived —
refuted), and the storage-node PDO name changed across **every** boundary
while staying stable across quiescence and every same-instance sample —
readable non-elevated and computed independently of the counter, so it
**qualifies** on this sitting's boundaries. Per the preregistration,
a qualifying signal makes cross-epoch counter readings incomparable by
construction; it also supplies a boundary-detection token on this
apparatus. One property is recorded as a limit rather than smoothed: the
storage-node PDO name is a kernel object name whose allocation sequence
restarted after the reboot, so a later epoch's value can in principle
equal an earlier epoch's — it is a change-detection token when sampled
across a boundary, not a globally unique epoch id, and a coincidental
cross-boundary equality is not excluded by construction.

**S2b — the reboot boundary agrees.** Boot time bracketed in-transcript
(the pre- and post-reboot `LastBootUpTime` differ by the operator's
restart); the storage-node PDO name changed across the reboot too;
ContainerId stayed constant even across it; the USB-node PDO name
changed here, but its refutation by the replug boundaries stands. The
counter read at the floor on both sides of the reboot; the reset
evidence is boundary 1's.

Also observed, recorded as apparatus context: this reader's arrival
timestamps never moved across any S1/S3 media exchange — this bridge
does not re-enumerate on media change, in measured contrast to the
NS1081 pair's behavior in the S4 card-move sitting.

Nothing here decides SI-33's design; the register owns status.

Custody: instruments archived with the transcript at
`%USERPROFILE%\partman-evidence\2026-08-04-si33-s1s2s3-sitting1\` on the
operator workstation, custodian Nate McBride; SHA-256 throughout, all
digests recorded in the transcript header before any device query —
probe instrument
`0f996271c4716ace046f5ba82aa2a9a2a3956a3658f66c097f530e2a082e4e1f`
(3170 bytes), candidate/bracket instrument
`67d60efb8073bf5eeee863d3978709dac29f587a3c62f4754146a665ba99d7f9`
(2747 bytes), roster instrument
`b85df3beb672af0242c6e09b9ca9055cd7e7cb432551faba15af46d800c083aa`
(2076 bytes); complete transcript
`64d08b48663ae9b7089ac709c0f0d72797893dc940aafc28631dfc6aead6d383`
(24091 bytes) with per-command exit codes, raw counter values (this file
carries deltas only, per the parent rule), and the elevation assertion
(`IsInRole(Administrator)=False`, token carries no Administrators SID);
OS build 10.0.26200; PowerShell 7.6.4. An independent second reader
retrieved the transcript through the locator, rehashed it to the same
digest and byte length, and confirmed the archive's file inventory
before any cell left `not yet taken`.

#### S5 — medium-register identity on a native MMC controller — preregistered 2026-08-13; taken the same day; valid

A new arm of this successor protocol, preregistered before execution.
It touches SI-28's record only as a preregistered instrument and never
its established results or its register disposition; the register owns
status. Its subject is the measurement Part 7 requirement 1 names as
round five's prerequisite: whether a **medium-attributable identifier**
— the SD card's own CID register — is client-readable on a native
(non-bridged) controller, whether it is stable and per-medium distinct,
and what the *same medium* presents when moved behind USB bridges,
including a same-model bridge pair (the S4 collision population's
Linux face).

Apparatus, recorded as it is rather than idealized: a personal
ThinkPad (Debian 13, kernel 6.12.x, GNOME), its built-in micro SD slot
on a PCIe `rtsx_pci`-class controller (`mmc_host/mmc0`), operator Nate
McBride present throughout with a remote driver over SSH as the
unprivileged user; that user is in `sudo` but not `disk` and holds no
ambient capability — the baseline-denial cell records the posture
rather than asserting it. Media: the two authorized SanDisk micro SD
cards (64 GB, 256 GB). Readers for the contrast legs: three Anker
USB-C card readers — two of one model, one older. GNOME automount is
disabled for the sitting (`org.gnome.desktop.media-handling automount`
and `automount-open` set false, restored after) and the mount table is
captured at every leg — no measured object may be mounted at any
capture; a leg that catches a mount is `void(mount)`, never a
negative. Double capture with `udevadm settle` for every value set;
per-command exit statuses; instrument digests recorded in-transcript
before first capture.

| # | Cell | Command / API | Distinguishing condition | Invalidation conditions | Result |
| --- | --- | --- | --- | --- | --- |
| S5a | Baseline denial, native slot | `stat` on `/dev/mmcblk0`; `dd` one sector; both as the unprivileged user | The client-baseline posture holds on this host for the mmc node class | mode nonstandard without being recorded; `dd` succeeding | `observed(denied)` — `brw-rw---- root:disk`, `dd` refused `Permission denied` rc 1, `CapEff` all zeros |
| S5b | The CID value set, card A (64 GB) | `cat` of `cid`, `serial`, `manfid`, `oemid`, `name`, `date`, `type` on the card's `/sys/bus/mmc/devices/` node, double capture | Whether the medium's own register — the identifier SI-28's filing records as unavailable through bridges — is world-readable on a native controller, and its exact byte values | any value unstable across the double capture | `observed` — the full register, world-readable, rc 0 throughout, byte-stable: `cid` `035344535236344786b7ec3aeb018c00`, `serial` `0xb7ec3aeb` (the medium's PSN), `manfid` `0x000003`, `oemid` `0x5344`, `name` `SR64G`, `date` `12/2024`, `type` `SD` — **every field SI-28's filing records Windows as exposing nothing of** |
| S5c | Block-device linkage | `readlink -f /sys/block/mmcblk0/device`; the mmc bus address recorded | Whether a naming traversal exists from the block device to the CID-bearing node — the structural analogue of ADR-0034's USB-ancestor rule | the link resolving outside `mmc_host/mmc0` | `observed` — resolves to `…/mmc_host/mmc0/mmc0:aaaa`, the CID-bearing node; the readable `rca` attribute (`0xaaaa`) confirms the bus address is the host-assigned RCA — an excluded input for naming, never identity |
| S5d | Reinsertion stability | Operator removes and reinserts card A; recapture S5b/S5c after settle | Whether the CID survives reinsertion byte-identical, and whether the bus address changes while the CID does not | automount fires and mounts before the recapture completes without being caught by the gate | `observed` — CID and every derived field byte-identical; the RCA reassigned to the same value on this host; the mount gate held (automount disabled; no mount fired) |
| S5e | Reboot stability | Full reboot; recapture S5b/S5c | The same, across the deepest boundary this apparatus offers | recapture taken before settle | `observed` — fresh boot bracketed by `uptime -s`; CID, serial, and linkage byte-identical; the gate held through the fresh desktop session. The card also rode an unplanned suspend/resume before any capture (the apparatus incident below) and enumerated identically |
| S5f | Per-medium distinctness | Card B (256 GB) in the same slot; full S5b value set | Two media, one slot, one controller: whether the CID/serial differs per medium — where the S4 pair's bridge serial collapsed | either card's values unstable | `observed(distinct)` — `cid` `035344535232353686454a552b018c00`, `serial` `0x454a552b`, `name` `SR256`: distinct CID and PSN on media sharing `manfid`, `oemid`, and even the manufacture date, while the RCA was the same `0xaaaa` for both — the discriminant is the medium register and only the medium register |
| S5g | The bridge contrast, same medium | Card A in Anker reader R1 on the same host: enumeration class, the ADR-0034 serial traversal, the udev identity keys, and a search for any CID-bearing surface | The same physical medium, native versus bridged: what identity the bridge presents and whether any medium register survives the bridge | the reader enumerating as anything but USB mass storage unrecorded | `observed` — the bridge is a **NORELSYS NS1081-family** part (the S4 sittings' own chip family), enumerating **two LUNs** (`1081CS0`/`1081CS1`) that both carry the USB descriptor serial **`0123456789ABCDE` — the canonical placeholder constant**; the ADR-0034 traversal resolves to the bridge's USB node and returns that constant; no CID-bearing surface exists anywhere under either LUN; a driver supplement (labelled in-transcript, not an instrument capture) pinned the medium to LUN CS0 by its sector count — `124735488`, byte-equal to the native slot's count for the same card — with CS1 the empty slot at size 0. **One constant serial covering a verifiable medium and an empty slot: the filing's original observation, reproduced on Linux** |
| S5h | The same-model bridge pair | Card A in R1, then in R2 (same model): each bridge's presented serial and udev identity | Whether the identical-model pair presents distinct per-unit serials or a shared constant — the S4 Windows finding's Linux face, on this hardware | either leg missing; readers not confirmed same model | `observed(shared constant)` — R2 presents the byte-identical `0123456789ABCDE` and identical `NORELSYS` identity strings: the pair is indistinguishable at every client-visible surface, the S4 collapse on the same chip family, measured on Linux with a medium whose native register discriminates perfectly |
| S5i | The older bridge | Card A in R3: the same captures as S5g | One more bridge data point, cheap, for the population claim's breadth | — | `observed` — a different bridge (`Generic- USB3.0 CRW -SD`), serial `201506301013` — a date-shaped firmware constant, not a unit serial — two LUNs again, no CID surface again |

What this arm deliberately does not do: no write, no format, no mount,
no layout; no register-status or ADR text change; no designation — if
its cells establish a medium-attributable identifier, the designation
extension is its own ADR-0034-pattern act on these rows.

**The sitting, 2026-08-13 (UTC), all nine cells observed.** Apparatus
corrections and incidents, recorded rather than smoothed: (1) the
preregistration guessed the controller `rtsx_pci`-class from the
reconnaissance's bare PCI path; the environment record measured the
driver as **`sdhci-pci`** — a native SDHCI part, which strengthens
rather than weakens the apparatus claim, corrected here against the
preregistration. (2) The host suspended (GNOME idle default) after
reconnaissance and before any capture; no capture was affected, the
card enumerated identically on resume, and idle suspend was disabled
for the sitting. (3) The host's WiFi dropped once between the R1 and
R2 legs during reader handling and was operator-cycled; no capture
was in flight, both legs' captures are complete and byte-stable, and
the interruption bracket is visible in the leg timestamps. (4) One
labelled driver supplement (LUN sector counts) was appended outside
the digested instruments and is marked as such in-transcript.

**What the cells jointly establish.** On one host, in one transcript:
the same physical medium is perfectly identifiable through a native
controller — full CID register, stable across suspend, reinsertion,
and reboot, distinct from its same-manufacturer sibling — and
identity-invisible through every one of three USB bridges, two of
which (a same-model pair on the S4 sittings' own NS1081 chip family)
share the placeholder constant `0123456789ABCDE` covering both the
medium and an adjacent empty slot. Part 7 requirement 1's Linux
measurement now exists; what SI-28's round five does with it — the
Linux attribution rule, any mmc-node designation extension under
ADR-0034's pattern, and the filing's general-predicate amendment —
is register work these rows enable and do not decide.

Custody: transcript SHA-256
`b8cb899539bb5f9782bf9c93edef0695a5d4ec4992475e46faa2ad1d60e7b58e`
(18814 bytes), computed on the ThinkPad before the file moved and
recomputed on the operator workstation — two independent
recomputations as preregistered, both agreeing; instruments
(`s5-capture.sh`, `s5-env.sh`) archived beside it at
`%USERPROFILE%\PartMan-evidence\2026-08-13-s5-thinkpad\`, custodian
Nate McBride, their digests recorded in-transcript before any
capture (the capture instrument's digest covers the S5a patch made
and restaged before the environment record ran).

## macOS

**Established 2026-08-05.** The client rows came from the increment 6 matrix
on an Apple Silicon host — IOKit / `IOMedia` property availability without
elevation, `diskutil` structured fields for an APFS container and its physical
stores, and the raw `/dev/rdiskN` read policy. **The privileged comparison leg
(M10) was taken the same day** in an ephemeral hosted runner, and it reads at
byte level what the client is denied. **Only M9, the Fusion shape, remains
`not established`**, because Apple Silicon has no Fusion Drive and the cell's
own text forbids inferring it from a nearby topology.

### Increment 6 macOS matrix — taken 2026-08-05; valid on the second sitting

Protocol recorded 2026-08-02 under WP-035 increment 6. Status: **taken
2026-08-05; M1–M8 executed, M9 `not established`, M10 `not yet taken`** — the
sitting records below the table are the evidence, and sitting 1's void is
retained above sitting 2 rather than discarded. The remainder of this
paragraph and the environment, setup, custody, and vocabulary text that
follows are preregistration wording kept verbatim; they state the conditions
the valid sitting met, not open preconditions. As preregistered, it records no
measurement beyond its own rows, changes no register disposition, decides no
option, and satisfies no platform-adapter criterion.
Its rows are fixed to the evidence the register names: Part 6 precondition 1's
separate client and helper signature/membership projections for the
platform-applicable technologies, precondition 2's native designators, and
SI-34's client-versus-direct-probe freshness projection. Anything this matrix
does not name is out of its scope, and executing it may not add rows.

**Environment and privilege rules.** Unprivileged rows may run on the
available Mac. No privileged comparison leg may run on that ordinary host;
privileged legs run only in a disposable macOS VM or hosted macOS test
environment satisfying SAFE-001/SAFE-002, and until one exists every
privileged-leg cell stays `not yet taken` rather than being approximated.
Every run first records: macOS product and build version, SIP state
(`/usr/bin/csrutil status`), session type (console or SSH — DiskArbitration
behavior differs and the session type is part of the record, not a nuisance),
`id` output (uid, gid, groups — note whether the user is in `operator`), and
the version and existence of each predeclared executable, invoked by trusted
absolute path with structured argv, bounded output, and a timeout:
`/usr/sbin/diskutil`, `/usr/bin/hdiutil`, `/usr/sbin/ioreg`,
`/usr/bin/plutil`, `/usr/bin/stat`, `/bin/dd`. A missing or
version-unrecordable tool voids the cells that name it, never the run.

**Setup boundary.** The only setup action is `hdiutil attach` of a WP-020
fixture image copied to a fresh `mktemp -d` scratch directory, with
`-imagekey diskimage-class=CRawDiskImage -nomount -readonly`. `-nomount`
keeps INV-006's no-auto-mount rule true by construction; `-readonly` means
the image bytes cannot change, and the image's manifest digest is verified
before attach and re-verified after detach. No experiment row writes to any
block device, and no step mounts a file system. Detach is explicit
(`hdiutil detach` of the exact attached device), and a failed detach is
recorded, voids nothing retroactively, and blocks further attach rows.
Results over attached images (M6, M7, M8) are projections of the DiskImages
device class and may not be represented as real-media evidence — the same
virtual/real boundary the Linux matrix enforces. Real-media macOS rows would
need their own explicitly authorized fixture media and are deliberately not
preregistered here.

Result vocabulary, closed: `observed(<value-class>)`, `observed(absent)`,
`denied(<errno or error-class>)`, `not-recognized`, `not-client-readable`,
`not-applicable(platform)`, `mechanism-unavailable(<which>)`,
`inconclusive(<gate>)`, `void(<gate>)`, `not established`, `not yet taken`.
A cell may not invent vocabulary at execution time. Failure of a validity
gate makes affected cells `void(<gate>)`, never a negative result.

| # | Cell | Command / API | Privilege | Distinguishing condition | Invalidation conditions | Result |
| --- | --- | --- | --- | --- | --- | --- |
| M1 | Raw whole-device read policy, boot disk | `stat` on `/dev/disk0` and `/dev/rdisk0`; `dd if=/dev/disk0 bs=512 count=1 of=/dev/null` and the `rdisk0` equivalent | unprivileged | Whether macOS denies an unprivileged raw read as Windows and Linux do, or the `operator` group changes the answer | ambiguity about which device is the boot disk; any write-intent open | **`denied(EPERM)`** — boot disk derived, not assumed: `/` → APFS container `disk3` → physical store `disk0s2` → whole disk `disk0`. Both nodes are `root:operator`, `brw-r-----` and `crw-r-----`; the measuring user holds `admin` but **not** `operator`. `dd` of one sector returned `Operation not permitted` on both `/dev/disk0` and `/dev/rdisk0`. macOS therefore denies the unprivileged raw read as Windows and Linux do. The `operator` half of the distinguishing condition is **not established**: the mode bits show that group is the route, and no operator-group user was measured. `EPERM` rather than `EACCES` is recorded as observed and unexplained — with the user outside the owning group `EACCES` was the expected errno, so something beyond mode bits is refusing |
| M2 | IOMedia property availability | `ioreg -a -r -c IOMedia` parsed as plist | unprivileged | Which of UUID, Content, Content Hint, Size, Preferred Block Size, Whole, Leaf, Removable, BSD Name are present per media object | unparseable plist; truncated output | **`observed`** — all nine named properties present per media object: UUID, Content, Content Hint, Size, Preferred Block Size, Whole, Leaf, Removable, BSD Name. Also present: Ejectable, Encrypted, Writable, Open, Removable, Role, GPT Attributes, Partition ID, Logical/Physical Block Size. **Limitation:** the capture is a whole-registry dump, so per-object scoping was applied off-host after the run. The presence set above is a direct read and needs no value comparison; any *separation* claim from this interface would need a normalizer declared before capture, and none was — see the sitting record |
| M3 | diskutil structured projection | `diskutil list -plist`; `diskutil info -plist <BSD name>` per whole disk | unprivileged | Which identity and geometry fields the structured interface carries, against the M2 set — same facts or a different projection | non-plist output; interactive prompt | **`observed`** — a **different** projection from M2, not the same facts renamed. It carries scheme, size, block sizes, writability, ejectability, whole/leaf, and per-partition entries, but does not surface M2's IOKit registry identifiers. Captured unnormalized and compared byte-for-byte between fixtures, which is what makes the M6–M8 rows below citable without a normalizer |
| M4 | APFS container and physical-store membership (precondition 1, APFS row) | `diskutil apfs list -plist` | unprivileged | Whether container UUID, physical-store references, and volume roles are client-readable without elevation | no APFS container present (impossible on a modern boot volume — record why if hit) | **`observed`** — container UUID, physical-store references, designated physical store, volume roles, and per-volume UUIDs are all readable without elevation (`APFSContainerUUID`, `PhysicalStores`, `DesignatedPhysicalStore`, `Roles`, `Volumes`, `APFSVolumeUUID`). Part 6 precondition 1's APFS row is **client-readable on this platform** |
| M5 | APFS container UUID as native designator (precondition 2) | fields from M2 and M4, plus a second sitting after reboot | unprivileged | Same UUID from both interfaces and across a reboot — source, stability; collision behavior is out of reach without duplicated hardware and stays `not established` | either source missing; reboot not performed | **`observed(stable)`** — the same container UUID is carried by both interfaces (M4's `APFSContainerUUID` and M2's registry `UUID`), and is **identical across a verified reboot**. Form: RFC-4122, uppercase, 8-4-4-4-12; the value is machine-specific and stays in the retained transcript. The reboot is evidence, not operator recollection: `kern.boottime` was captured in both phases and differs. Collision behaviour remains **`not established`** — it needs duplicated hardware, exactly as the cell's own text says |
| M6 | Foreign-signature fixture projection | attach each of `gpt-basic-512`, `mbr-basic-512`, `apm-basic-512`, `blank-512`; then M2/M3 against the attached device | unprivileged | What macOS reports for GPT, MBR, and its own historical APM, and whether blank and foreign are distinguishable — `not-recognized` is an expected honest outcome, not a failure | attach denied (record as `denied`, cell complete); wrong device targeted; automount observed | **`observed`, and it splits in two.** macOS **distinguishes the three schemes**: `gpt-basic-512` → `GUID_partition_scheme`, `mbr-basic-512` → `FDisk_partition_scheme`, `apm-basic-512` → `Apple_partition_scheme`, each with its partitions materialized as child BSD nodes. But **blank and foreign are `not-recognized` and mutually indistinguishable** — `blank-512` reports no `Content` and no partitions, and so does every non-native signature in M8. Attach succeeded for all ten fixtures, nothing automounted, and every digest bracket matched |
| M7 | SI-34 freshness projection, stale-signature case | attach `ext4-with-stale-mdraid-090-512.img`; capture M2/M3 projection | unprivileged | Whether any macOS client interface reports any signature fact for bytes carrying a live ext4 and a stale mdraid superblock — the platform's contribution to the freshness-projection question is which facts exist here at all | same as M6 | **`not-recognized`** — `ext4-with-stale-mdraid-090-512`'s `diskutil info` and `diskutil list` projections are **byte-identical to `blank-512`'s**. macOS's contribution to SI-34's freshness question is therefore that **no signature fact exists at this layer at all**: the client cannot see the live ext4, the stale mdraid, or the conflict between them. The platform does not report a stale signature in preference to a live one, as Linux does — it reports neither |
| M8 | mdraid / LUKS2 / LVM2 / ZFS on macOS (precondition 1 non-native rows) | outcome of M6/M7 interfaces for those signatures | unprivileged | Stock macOS ships no prober for these; the expected result is `not-recognized` or `not-applicable(platform)`, recorded rather than assumed — OpenZFS, if installed, would be a separately labelled non-stock projection | third-party storage kexts/extensions present and undeclared | **`not-recognized`** for mdraid, LUKS2 and LVM2 — each byte-identical to `blank-512` on both interfaces, as expected of a platform shipping no prober for them. **ZFS is `not-applicable(platform)`**: stock macOS ships no prober and the fixture catalogue contains no ZFS image, so nothing was attached for it. That is recorded rather than folded into the other three. No third-party storage extension was declared or observed |
| M9 | Fusion membership and shape | `diskutil apfs list -plist` on a Fusion container; the one-store-absent shape | unprivileged | What a Fusion container reports intact, and with one store absent | **conditional on representative hardware**; without it this cell remains `not established` per the increment's own rule | **`not established`** — the host is Apple Silicon, which has no Fusion Drive, so the representative hardware this cell is conditional on cannot exist here. Not approximated from a two-device APFS container; degraded Fusion behaviour is not inferred from a nearby topology |
| M10 | Privileged comparison leg | raw header reads at the fixture offsets through `/dev/rdiskN` | privileged, disposable VM only | Whether the helper-side view can read what the client cannot, on the same attached fixture | any attempt on the ordinary host; no disposable macOS environment available | **`observed` — the helper reads what the client cannot, on the same attachment, on every fixture tried. Taken 2026-08-05 in a GitHub-hosted `macos-15` runner (`RELEASE_ARM64_VMAPPLE`, macOS 15.7.7) — an ephemeral Apple Virtualization Framework guest destroyed at job end. For all seven fixtures the unprivileged client's raw read was **denied `EACCES`** while root read the bytes: every helper byte-range digest equals the source image's. **The decisive pair separates for the helper and not the client**: `gpt-basic-512` and `gpt-conflicting-tables-512` have identical first-64-KiB digests and **different last-64-KiB digests**, so the disagreement lives in the backup table, which no client interface here reports. The four signatures the client called byte-identical to blank each carry a distinct helper head digest. Scope: this leg is a different machine and a different macOS from the M1–M8 sitting, which is why it re-ran the client half itself rather than comparing across hosts |

Custody, per executed run: complete transcript retained outside the
repository with archive locator and custodian named; hash algorithm, digest,
and byte length recorded; capture-script digest recorded before the first
attach; OS/build, tool versions, and every exit status recorded; fixture
digests from the generated manifest; a second reader retrieves and rehashes
the transcript before any cell leaves `not yet taken`. Raw diagnostic output,
machine-specific identifiers, and secrets stay out of this file; only
normalized observations and limitations enter it. A cell that cannot meet a
custody requirement stays `not yet taken` — custody failure is not a result.

#### First sitting, 2026-08-05 — VOID (two instrument defects); amended twice

The instrument's first execution, on the available Apple Silicon Mac at
console, unprivileged (uid 501, non-root asserted in-transcript), macOS 26.3.2
build 25D2140, SIP enabled. It is void on two defects, **both in the harness
rather than in the operator's conduct or the platform**, and it is retained
here for the same reason the SI-35 loop protocol retains its two void
sittings.

**Defect 1 — tool versions were never recorded.** The harness invoked
`diskutil version` and `hdiutil version`. Neither verb exists; both exited 1
with `did not recognize verb` and `verb not recognized`, and no version was
captured for either tool. This subsection's custody rule requires tool
versions, and the instrument's own environment rule makes a
version-unrecordable tool void the cells that name it — which is most of the
matrix. A favourable reading was available and is recorded as **rejected**:
these are OS-bundled tools carrying no independent version string, and the OS
build *was* recorded, so one could argue the version requirement was met by
`sw_vers`. It is not adopted, because a stronger record was available the
whole time and the project's rule is that a gate voids rather than gets
argued past.

**Amendment 1.** Tool identity is now a SHA-256 over each declared binary,
recorded alongside the OS build. That is reproducible and stronger than any
version verb would have produced, and it applies to every declared tool rather
than the two that happened to be asked.

**Defect 2 — the post phase ran without a reboot.** M5's invalidation
condition is `reboot not performed`, and the machine did not reboot. Nothing
in the instrument detected this. It was caught after the fact from disk
numbering: the fixture-bearing USB volume was `disk4` in the pre phase,
attaches consumed `disk5`, and the same volume reappeared as `disk6` in the
post phase — a counter that never reset, where a reboot resets it. The run
timestamps, 3 minutes 4 seconds apart, agree. **That reasoning is inference,
not evidence**, which is itself the defect: the instrument asked the operator
to remember a precondition instead of proving it.

**Amendment 2.** `kern.boottime` is now captured in both phases and compared
in a hard gate at the top of the post phase. An unchanged boot time announces
`M5 → void(reboot-not-performed)` in the transcript; an absent pre-phase
capture announces `void(reboot-unverifiable)`. The gate was tested by forcing
each outcome before the amended harness shipped.

**What the void sitting nonetheless observed**, recorded as history and not
cited by any cell above: the M1 denial and every attach row behaved as
sitting 2 later reproduced. Nothing in it is relied on.

**Artifacts.** Retained in the operator evidence store,
`2026-08-05-macos-increment6-sitting1` (custodian Nate McBride), 195 files
with a SHA-256 inventory. Pre-phase transcript
`731253f9e6f03cc12cf10a0af979bc0611d9b5de3c14b181959b344d3ff15c3a`;
post-phase transcript
`3fbcb584d6b64af7ec376bbcab12db129d8eab2e19c96e8519b878664bcc0bbe`.

#### Second sitting, 2026-08-05 — VALID; both amendments held

Same host and posture: Apple Silicon, console session, uid 501 with `admin`
but not `operator`, macOS 26.3.2 build 25D2140, SIP enabled, capture script
`f46500f02b3eeaf26331f11c80f18582e4a8e0d84368eea0f360fe4f3f5a7505` digested
before the first attach.

**Both amendments held.** All 24 declared binaries were SHA-256'd, including
`diskutil` and `hdiutil`. The reboot gate passed on boot times 19 minutes
apart, so M5's across-reboot criterion rests on a captured fact rather than on
recollection.

**Setup integrity.** The transferred `MANIFEST` was byte-identical to the one
generated from repository revision `a6d48cc`, which also confirms fixture
generation is deterministic across Windows and macOS. All ten attach rows
attached, captured, and detached with the detachment confirmed; every
before-attach and after-detach digest pair matched; nothing mounted at any
point, so INV-006 held by construction rather than by inspection.

**The result that matters for SI-35.** For the decisive pair —
`gpt-basic-512` against `gpt-conflicting-tables-512` — `diskutil info -plist`
and `diskutil list -plist` are **byte-identical**, unnormalized. Both fixtures
materialize the same child nodes. **macOS is the third platform on which the
enumerated unprivileged client projection does not separate a healthy GPT from
one whose two tables describe different partitions**, after Linux
(2026-08-03) and Windows (2026-08-04).

**The result that matters for SI-34 and ADR-C3.** Every non-native signature —
the live-ext4-plus-stale-mdraid fixture, the mdraid member, the LUKS2
container, the LVM2 orphan — produces a projection **byte-identical to a
blank disk** on both interfaces. A macOS client cannot distinguish a disk
holding a file system from one holding nothing. This is the platform's
strongest statement about the client/helper asymmetry precondition 1 asks
about, and it is a fact about macOS rather than a defect in the run.

**Limitations, stated rather than left to inference.**

- The `ioreg` capture is a **whole-registry dump**, and no normalizer was
  declared before capture. M2's property-presence result is a direct read and
  needs none. **No separation claim is made from the `ioreg` interface**: a
  raw diff of the decisive pair shows 516 differing lines, every one an
  ambient APFS statistics counter and none naming the fixture device, and
  scoping that away after seeing output would be exactly the post-hoc
  normalization that disqualified the historical WSL2 loop record. The M3
  byte-identity carries the finding instead, unnormalized.
- Results over attached images are projections of the **DiskImages device
  class** and are not real-media evidence. Real-media macOS rows would need
  their own authorized fixture media and are deliberately not preregistered.
- One host, one macOS build, one user, console session only. The SSH
  projection is unmeasured and DiskArbitration is documented to differ.
- M9 and M10 are not taken; nothing above substitutes for either.

**What this sitting does not do.** It chooses no SI-35 option, supplies no
chosen-option refusal demonstration, refutes no existential H-separation
hypothesis, and decides no register disposition. Its non-separation results
cover the enumerated projections only.

**Artifacts.** Retained in the operator evidence store,
`partman-macos-sitting-2` (custodian Nate McBride), alongside sitting 1; 206
files. SHA-256 and byte length per transcript, **recorded 2026-08-05 after the
omission below**:

- `out-pre/00-transcript.txt` —
  `da5506e97d75e889b0e74c78c747912051707566f450de1d701fee789590f94d`,
  18 647 bytes.
- `out-post/00-transcript.txt` —
  `4f6e8916c87477869c617e28aeaf15cfd7f47cb571e7c44ae721b8f1027081cc`,
  5 320 bytes.

**The omission, recorded rather than quietly repaired.** This paragraph
originally named the locator and custodian and no digests, while this
subsection's custody rule requires "hash algorithm, digest, and byte length
recorded". Sitting 1's record carries its digests; sitting 2's did not. The
gap surfaced only when the second-reader readback was being prepared, because
a reader cannot rehash a transcript "to its recorded digest" when no digest is
recorded — so the obligation was not merely outstanding, it was
**unperformable as specified**. The digests above are computed from the
retained capture, which has not been modified since retention; that is weaker
than a digest recorded at retention time and is stated as such rather than
presented as equivalent.

**Second-reader obligation discharged 2026-08-08.** An independent reader
session — not the session that produced this record or computed the digests
above — retrieved both transcripts through the locator, rehashed each, and
confirmed the archive readable: `out-pre/00-transcript.txt` matched its
recorded digest at 18 647 bytes, `out-post/00-transcript.txt` matched at
5 320 bytes, and the store holds the 206 files this paragraph counts. The
discharge does not launder the omission above into the stronger property:
a matching rehash confirms the retained copy is unchanged since the digests
were computed on 2026-08-05, not that the digests were taken at retention.
Sitting 2's custody remains what the omission paragraph says it is —
locator and custodian recorded at the time, digests recorded later from an
unmodified copy.

#### M10 sitting, 2026-08-05 — VALID; the helper reads what the client cannot

Taken in a **GitHub-hosted `macos-15` runner**, which the cell's environment
rule admits as a "hosted macOS test environment": `RELEASE_ARM64_VMAPPLE`,
macOS 15.7.7 build 24G720, an ephemeral Apple Virtualization Framework guest
on Apple silicon, destroyed when the job ended. Fixtures were generated on the
runner from the checked-out revision, so no transfer step needed verifying.
Harness digest recorded before the first attach; all seven fixtures attached,
captured and detached with confirmation; every before/after digest bracket
matched; nothing mounted.

**Why the client half was re-run here.** M10 asks whether the helper view
reads what the client cannot **on the same attached fixture**. This is a
different machine and a different macOS from the M1–M8 sitting, so comparing
across hosts would not have answered that. The harness therefore captures both
halves of every attachment itself: the client interfaces as an unprivileged
user, the byte reads as root, on one attachment.

**The asymmetry, at byte level.** On every fixture the unprivileged client's
raw read was **denied `EACCES`** while root read the device. Every helper
byte-range digest equals the corresponding range of the source image, so the
helper reads the true bytes rather than something the platform reconstructed.

**The decisive pair separates for the helper.** `gpt-basic-512` and
`gpt-conflicting-tables-512` produce **identical first-64-KiB digests** and
**different last-64-KiB digests**. The disagreement between the two tables
lives in the backup, at the tail — which is exactly what the fixture is for,
and which no client interface on any measured platform reports. The client
halves of this same sitting reproduce that blindness: `diskutil info -plist`
and `diskutil list -plist` are byte-identical across the pair here too.

**The blank-versus-foreign collapse breaks the same way.** The four signatures
the unprivileged client reported as byte-identical to a blank disk each carry
a **distinct helper head digest** — the mdraid member, the LUKS2 container,
and the LVM2 orphan differ from blank's, and the live-ext4-plus-stale-mdraid
fixture differs at both head and tail. Four disks a macOS client calls empty
are immediately distinguishable to a privileged reader.

**Two incidental observations, recorded because they contradict or refine
the earlier sitting.**

- **`EACCES` here, `EPERM` on the laptop.** The M1–M8 sitting recorded the
  raw-read denial as `EPERM` and flagged it as unexplained, since `EACCES`
  was the expected errno for a user outside the owning group. This runner
  gives the expected `EACCES` with the same `root:operator` ownership. The
  difference between a physical host and a VM guest is now a **contrast**
  rather than a lone oddity, but nothing here identifies its cause and no
  attribution to SIP or any other mechanism is made.
- **Node mode differs.** The runner's nodes are `br--r-----` / `cr--r-----`,
  where the laptop's were `brw-r-----` / `crw-r-----`.

**A generalization the earlier record could not claim.** The M1–M8 sitting
declared "one host, one macOS build, one user" as a limitation. This sitting's
client half ran on different hardware and a different macOS major version
(15.7.7 against 26.3.2) and reproduced the decisive-pair byte-identity. That
weakens the limitation for that specific finding; it does not lift it for the
rows this sitting did not re-run.

**Limitations.** A GitHub-hosted runner is a shared-infrastructure VM rather
than a machine under this project's control, and its image is not a
checksum-pinned artifact this repository owns. The byte ranges are the first
and last 64 KiB only, chosen to bracket partition tables and the signature
offsets in these fixtures; nothing is claimed about bytes between them. This
leg used `dd` through `/dev/rdiskN` and no other privileged interface.

**What this does not do.** It decides no SI-34 option and does not satisfy
SI-34's freshness-projection element, which names a projection that does not
yet exist. It chooses no SI-35 option and supplies no chosen-option refusal
demonstration. It refutes no existential hypothesis.

**Artifacts.** CI run 31020018982, capture retained in the operator evidence
store as `2026-08-05-macos-m10-ci-run31020018982` (custodian Nate McBride),
172 files with a SHA-256 inventory; transcript
`259b1046e1d80b40fb92fcfd99ef018af86f11b7f5086aca3e5c239a15436256`. The
workflow artifact itself is public and expires; the evidence-store copy is the
durable one.

**An omission of sitting 2's class, recorded at readback.** This paragraph
recorded the transcript's digest and no byte length, where the custody rule
requires both. The length is 23 516 bytes, measured 2026-08-08 from the
retained capture; like sitting 2's digests it is a value taken later from an
unmodified copy, not a retention-time record, and is stated as such. The
digest itself was recorded at retention, so the transcript's custody is
stronger than sitting 2's on that axis and weaker on none.

**Second-reader obligation discharged 2026-08-08.** An independent reader
session retrieved the transcript through the locator and rehashed it to its
recorded digest, matching — and because M10's digest was recorded at
retention, this rehash carries the property sitting 2's cannot. The reader
also rehashed every entry of the capture's SHA-256 inventory: all 172 match,
and the store holds those 172 files plus the inventory itself, readable.

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
| `gpt-invalid-primary-valid-backup-512` | Present; primary-header CRC invalid | `ID_PART_TABLE_TYPE=gpt` | gpt `0x3ffe00`, PMBR `0x1fe` |
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
- **A damaged-primary image is still labelled `gpt`.** `blkid` reports `gpt`
  for `gpt-invalid-primary-valid-backup-512`, while only `wipefs`'s retained
  offset list shows that the primary signature is absent. The observation does
  not reveal whether libblkid used the backup or accepted other bytes without
  validating the primary CRC; the recovery mechanism is unmeasured. A client
  reading this udev projection cannot distinguish this fixture from the healthy
  fixture by table type alone.

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

This subsection scopes only the 2026-07-28 file/virtual-SCSI run immediately
above. The later 2026-08-02 loop output is recorded in its own section below;
it remains historical and non-qualifying because issue #94 was open and its
final normalizer was extended after inspecting output.

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
  partition scanning can populate `ID_PART_ENTRY_*` — and at the time this row
  was written the loop measurement was still open. The narrow claim, that
  libblkid produces identical output for the healthy and conflicting image, is
  established for files. The later non-qualifying loop record did not separate
  them in its retained post-hoc-normalized views; the hardened rerun below was
  **taken 2026-08-03 and is valid on its third sitting**, and it found the
  named candidate client projection `non-separating` for the decisive pair on
  a real loop device whose partitions were materialized — so this bullet's
  "probing a loop device may differ" caveat is answered for that enumerated
  projection and for no other interface. The rerun's separately labelled
  privileged leg found `blkid -p -o udev` likewise identical across the pair;
  that is the libblkid-specific half of this bullet, and being privileged it
  may not be merged into any client claim.
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
- **The environment was not §11.3's.** T2 is defined as "disposable VMs per
  OS"; this ran on the operator's working WSL2 instance, which is not
  disposable. Recorded as a second respect in which the run sat outside the
  arrangement its subject matter belongs to, independent of the #94 block.

**Two project documents characterize this activity differently, and the
specification text neither of them cites is the one that matters.** WP-035
increment 5 calls these measurements "operator-run, read-only experiments...
not tests and not repository commands"; issue #94 calls the same SI-35 loop
probe "Tier-2 work that cannot yet be made". **§11.3** of the specification —
cited by neither — places `loop` under T2, *"Privileged, disposable VMs per
OS"*, while stating its restriction more narrowly than either document
assumes: **"Destructive suites run only here or in T3, gated by SAFE-007."**
A read-only loop attach is not a destructive suite, so §11.3 does not forbid
this run outright; what it does establish is that loop work belongs to a tier
whose environment is a disposable VM, and this ran on the operator's working
WSL2 instance, which is not disposable.

**This is not a §1.11 filing, and an earlier version of this record said it
was.** The spec-issue register exists for conflicts *between requirements* —
its own rule is that each entry "states the requirements that disagree" — and
here the disagreement is between two project documents, with the one genuine
requirement (§11.3) silent on the read-only case rather than in conflict with
anything. Filing it in the register would have miscategorized a documentation
inconsistency as a specification conflict and inflated the register's counts,
which that document names as its own characteristic failure. The
reconciliation belongs with the two documents that disagree, and is recorded
on issue #94 where the stricter characterization lives.

These sections extend this file's rule of use to their own vocabulary — a row
marked `not yet taken` MUST NOT be relied on, cited, or paraphrased as a
finding, by anything, not only by an ADR that freezes canonical bytes. What
the hypotheses and their logical predicates were fixed before anyone saw a
result. The projection normalizer was not: it was extended after the first
diffs, which voids the resulting comparison as pre-registered evidence. The
historical outcomes are recorded under "What the historical run recorded"
below.

**What it answers.** The table-state section above establishes that, probed as
regular files, `gpt-basic-512` and `gpt-conflicting-tables-512` produce
byte-identical output from both interfaces, that `ID_PART_ENTRY_*` is absent
for whole-disk file probes, and that whether a loop device separates the two
images is **open**; SI-35's register entry names the loop measurement so the
file-probing limitation is not mistaken for a kernel limitation. The 4Kn
section above establishes that IMG-011 evidence cannot come from file probing
at all, and names a loop device with an explicit 4096-byte sector size as the
route. Two hypotheses were recorded before the run. H-separation's positive
criterion was sound, but its negative criterion was not: equality of one finite
projection cannot refute a claim about *some* client-readable fact. The
historical rule is preserved here with that defect explicitly withdrawn:

- **H-separation** — *once the kernel has parsed both images, some
  client-readable fact separates `gpt-conflicting-tables-512` from
  `gpt-basic-512`.* Any retained difference supports that existential claim on
  the run's environment. Byte equality establishes only that Phase 7's named,
  finite projection did not separate the pair; it does **not** refute the
  existential claim. The original preregistration called equality a
  refutation, which was a logical error independent of the later normalizer
  change. Either measured sign is recorded with versions beside it and
  generalized no further — the mdraid section above is this file's own proof
  that this toolchain changes answers between versions.
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

**What it is not.** It was an operator-executed, read-only script, not a test,
repository command, or `xtask`; but that does **not** make it a conforming T2
run. Loop work belongs in §11.3's disposable-VM environment, and this working
WSL2 instance was not disposable. The blocks below preserve the historical
instrument and the final post-hoc normalization; they are not authorization to
rerun it. It also does not close the real-partitioned-Linux row: a loop device
stands in for “a device this kernel parsed” and says nothing about real
hardware.

**The #94 gate, carried structurally.** `losetup` resolves its path argument
in userspace and hands the kernel a descriptor it opened itself, so the object
attached is whatever the name resolved to at attach time, and
`/sys/block/loopN/loop/backing_file` is by-name evidence only — nothing binds
`/dev/loopN` to a verified handle (repository issue #94). WP-035's rule was
this section's rule: anything loop-backed was blocked until #94 closed, and if
a read-only measurement was taken before then — the issue records the
read-only blast radius as a wrong measurement, not a write — the gap was
recorded beside the numbers. **#94 closed 2026-08-03.** The block is lifted,
and what lifted it is narrow and worth stating exactly: a descriptor-bound
attach now exists and has been proven, so a measurement that configures the
loop device *from a verified descriptor* no longer carries this gap. A
measurement that reaches the device by pathname — including anything built on
plain `losetup <file>` — carries it undiminished. The run recorded in this
section is the latter, and closing #94 does not improve it. The recording format makes that a template
obligation rather than a judgment call. Phase 1's digest check is accident
friction on the manifest-token model: it compares a same-named file, by name,
before attach, and closes nothing.

**Read-only posture, declared.** Storage-device access was read-only. The script
did write regular scratch copies and capture files; because Phase 1 used a
fixed directory plus `cp`, it could overwrite a prior scratch run, contradicting
the original “new files only” claim. That is a protocol defect, not a device
write. Every attach used `--read-only`, read back from `/sys` for disk and
partitions; no phase mounted a file system.

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
        | grep -Ev '^(USEC_INITIALIZED|ID_PART_ENTRY_DISK|ID_LOOP_BACKING_FILENAME|ID_LOOP_BACKING_FILENAME_ENC|ID_LOOP_BACKING_INODE|ID_LOOP_BACKING_DEVICE)=' \
        | LC_ALL=C sort
    else
      echo "UDEV-DB:absent"   # observed absence — a value, not a failure (ADR-C4)
    fi
    echo --
  done > "$1"
}
capture "$SCRATCH/proj-gpt-basic-512.txt"   # substitute the fixture under measurement
awk -v b="$BASE" '$4 == b || $4 ~ ("^" b "p[0-9]+$")' /proc/partitions   # cross-check row count
```

This is the **final recomputation**, not the pre-registered normalizer. The
initial run dropped only `USEC_INITIALIZED` and `ID_PART_ENTRY_DISK`; discovering
the backing-object keys after inspecting its diffs voided that computation.
Changing a normalizer after seeing results is why the WSL2 negative remains a
historical observation rather than promotable evidence.

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
printing nothing and `IDENTICAL` records non-separation by this named finite
projection. A difference supports H-separation on this environment; equality
does not refute the existential hypothesis. Both are exactly recordable.

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
a healthy one. A difference in this instrument's named projection supports
that proposition and would make option (b) viable. Equality supplies only a
bounded negative about that projection; neither the register's earlier
file-projection equality nor this loop projection can establish that *no*
client-readable fact exists. The loop question therefore narrows only on a
positive separation result, not on either sign.
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

**A content check was added at run time, and is not reproducible from Phases
0–7.** After each attach, `--partscan`, and udev settle, the privileged half
hashed the whole loop device and retained only a `matched` verdict; all seven
matched. The exact command was not preserved in the instrument, so this record
does not pretend otherwise. Timing matters: even a correct post-attach digest
cannot bind the pathname resolution or the earlier partition-scan/udev events,
and it says nothing about later rebinding. It is a content snapshot and a
historical mitigation, neither #94's inode/handle closure nor reproducible
protocol evidence.

Because the gap line must travel with any excerpt carrying a number, a
self-contained short form appears beneath every filled table below.

**Disk-level record** — one row per fixture. The wrapper reported every attach
succeeded, every `/sys` `ro` read back `1`, and every run-time content check
`matched`; the commands and digests behind the content verdicts were not
retained:

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

**Separation record** — the historical Phase 7 diff after the final,
post-hoc normalization. It is not a pre-registered answer to SI-35.

**The drop-list was extended after seeing the first diffs.** It began with
`USEC_INITIALIZED` and `ID_PART_ENTRY_DISK`. The void computation exposed
backing filename, encoded filename, and inode values; three pairs otherwise
differed only by backing-object plumbing, while the damaged-primary pair also
retained its real missing-partition difference. The executed recomputation
dropped all four declared `ID_LOOP_BACKING_*` keys, including DEVICE, but the
retained summary does not establish that every key differed in every pair.
The first computation occupies no result cell.

The self-audit was also too narrow: it scanned key names for `loopN`, then
proposed scanning values only for the scratch path. Neither catches a backing
inode/device or an unknown plumbing key. The future protocol must freeze the
normalizer before the run and void on any undeclared backing/session value;
post-hoc extension is not a valid measurement method.

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

| Fixture | Retained `wipefs` offsets separate it from healthy? | Client projection differs from healthy? |
| --- | --- | --- |
| `gpt-conflicting-tables-512` | **no** — identical offsets | **no** — identical projection |
| `gpt-invalid-primary-valid-backup-512` | **yes, by content** — primary-copy offset absent | **yes, by content** — no partitions materialized; no explicit damaged-state marker |
| `gpt-missing-backup-512` | **yes** — backup copy absent | **no** — projection identical to healthy |
| `hybrid-mbr-gpt-512` | **unmeasured as a hybrid classification** — only offsets, not signature types, were retained | **no** — GPT rows carry no trace of the aliasing MBR entry |

The retained client projection differs on damaged-primary content and is blind
where retained wipefs offsets expose the missing backup. Both named readings
fail to distinguish conflict from healthy. Hybrid typing on the privileged
side was not retained, so no helper-blindness claim is available there. A
content difference is not automatically a reusable damaged-state marker; an
undamaged empty-GPT control was not included.

**4Kn annex record:**

| Row | Value |
| --- | --- |
| Attach with `--sector-size 4096` | ok |
| `logical_block_size` read-back | **4096** |
| `ID_PART_TABLE_TYPE` (udev db) | `gpt` |
| Partitions materialized | **2** — ESP at `/sys` start 2048, Data at 4096 |
| H-4Kn wrapper observation on this environment | **`supported` (historical, non-qualifying while #94 was open)** |

> **Binding gap (issue #94, open at run time):** nothing bound `/dev/loopN` to
> a verified handle. A wrong-measurement possibility travels with these numbers.

**The historical wrapper output is consistent with H-4Kn support, and its
like-for-like control also produced the expected rows.** The same-run
`gpt-basic-512` control and the 4Kn attach both materialized partitions. That
rules out one global no-partition mechanism inside the reported output, but
issue #94's wrong-object possibility makes this a non-qualifying observation
until the descriptor-bound rerun reproduces it.

**This is consistent with the IMG-011 route the file said would be needed.**
The Linux section
above records that `gpt-basic-4kn.img` probes as `PMBR` from a regular file,
because a file carries no logical sector size, and that a prober-based check
for 4Kn "needs a loop device configured with an explicit sector size, which is
privileged and therefore Tier 2". The wrapper reported a 4096-byte loop, GPT
signatures at `0x1000` and `0x3ff000`, and partition extents matching the
catalogue's 4Kn layout under the 512-byte-unit convention. Those readings are
consistent with the regular-file `PMBR` result being a probing artifact rather
than a fixture defect; they do not confirm that explanation until #94-bound
identity is established.

### What the historical run recorded

> **Scope line, travelling with every claim below:** these results are one
> kernel build, one util-linux, one udev, under WSL2. The decisive-pair result
> is a **named-projection equality** claim, and this section's own rule
> withholds such a negative
> from any register decision until a non-WSL distro-kernel run confirms it.
> Issue #94's open binding gap separately makes **every** loop row a possible
> wrong-object measurement until a descriptor-bound rerun reproduces it.

**The retained named projection was equal after post-hoc normalization; that
is not an H-separation refutation.** After backing-object keys were dropped,
the retained projections were byte for byte identical: the kernel materialized
the primary partition set for the conflicting fixture, and the disagreeing
backup partition appeared nowhere in that projection. Equality of those named
fields says nothing exhaustive about other client-readable facts. Moreover,
the normalizer was amended after the run exposed undeclared plumbing fields,
and issue #94's binding requirement was not yet satisfied. This is historical,
hypothesis-forming evidence only, not valid preregistered evidence available to
a register decision.

**On SI-35's attribution question, the retained views point one way but do not
settle it.** The register asked for this measurement "so the file-probing
limitation is not mistaken for a kernel limitation". On this environment the
post-hoc-normalized client projection and the retained privileged `wipefs`
offset list both collapse the decisive pair. That is narrower than saying
either the client or a future helper is blind: other client fields were not
exhausted, `wipefs` offsets are not the helper's eventual classification, and
the run itself is not qualifying evidence. Toolchain drift in the mdraid rows
above is an additional reason not to generalize the observation.

**Option (b) gains no promotable support from this run; its viability remains
open pending a valid confirmation.** The register states that if a loop device
separates the pair, "option (b) becomes viable and this issue narrows sharply".
The named retained readings did not separate it: the Windows partition-row
projection, this WSL client projection, and this WSL `wipefs` offset list.
Those are related projections with documented coverage gaps, not three
independent exhaustive interfaces. Options (a) and (c) are untouched: their
recorded costs — (a)'s observation basis becoming hash-visible body content,
(c)'s inherited unproven monotonicity obligation — are unchanged. The register
can weigh only a future qualifying run.

**Two findings the hypotheses did not ask for.**

- **The damaged-primary fixture differs in retained partition count, not in an
  explicit table-health property.** It materialized no partitions while its
  udev entry still read `ID_PART_TABLE_TYPE=gpt`; the healthy fixture
  materialized two. That distinguishes these two images in this run, but it is
  not a general damaged-primary classifier: a valid empty GPT can also have no
  partitions, and no such control was included. Which GPT copy libblkid used
  also remains unmeasured.
- **Only the retained privileged offset view exposes the missing backup.** The
  `wipefs` offset list omits the backup while the retained client projection is
  identical to healthy. Calling that "helper-only" would overstate both the
  future helper's design and the fields captured here.

**The retained Linux client projection does not expose the hybrid entry.** It
records `gpt, untraced`; the aliasing `0x0c` MBR entry leaves no trace there,
matching libblkid's file result. The run retained `wipefs` offsets rather than
a privileged type classification, so helper visibility was not measured. The
Windows layout probe was reachable by its `Win32_DiskDrive` index but was not
run for this fixture. The broader hybrid question is therefore unanswered on
both platforms.

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
  proof that one util-linux version's answer is not another's. In the historical
  wrapper, whether `udevd` processed loop-attach events was a measured row:
  a successfully established `UDEV-DB:absent` was `observed(absent)`, not a
  broken capture. The hardened confirmation below deliberately requires a
  database entry as a coverage gate; a confirmed absence remains an observation
  but makes that pair inconclusive for its named projection.
- **Every row needs binding; negatives additionally need a second
  environment.** Issue #94 makes every historical loop row — positive or
  negative — unavailable for promotion until a descriptor-bound rerun. A row
  recorded only under WSL2 that asserts an absence —
  projections identical, no udev entry, no partitions — must not be relied on
  by any register decision until confirmed on one non-WSL, distro-kernel
  environment, because an absence claim generalizes worse than an existence
  claim. The post-hoc-normalized decisive-pair equality is such a negative, so
  its valid successor must be both descriptor-bound and non-WSL. The positive
  H-4Kn and fixture-specific damaged-primary readings do not need a second
  environment merely because they are positive, but they still need #94-bound
  reproduction before promotion.

  **What remains outstanding, counted by the authority that requires it.** The
  register lists three evidence categories before accepting an option: the
  loop-device measurement, the Windows equivalent, and *"a demonstration that
  whichever option is chosen still refuses rather than proceeds on
  `gpt-conflicting-tables-512.img`"*. **Two are now satisfied and the third is
  not.** The loop category is discharged by the descriptor-bound non-WSL
  protocol below, taken 2026-08-03 and valid on its third sitting; the
  historical WSL2 negative it was written to confirm is confirmed, and the #94
  non-qualification no longer applies. The Windows category is discharged by
  the completion rerun of 2026-08-04, which is valid: the original procedure
  omitted reachable layout rows and retained incomplete status surfaces, and
  the rerun made all three declared refutation conditions evaluable over
  complete retained surfaces, refuted them, and answered W-Q4. **The
  chosen-option refusal proof remains outstanding, and cannot exist before an
  option and an implementation exist** — no SI-35 option has been chosen, so
  this category is blocked on a decision rather than on measurement. Neither
  discharged category resolves SI-35, and neither refutes the existential
  H-separation hypothesis: both cover their enumerated projections only.
- **The attach is privileged everywhere.** A separation finding here is a fact
  about what the kernel's parse leaves client-readable, not about what an
  unprivileged client can cause to be parsed. Real disks are parsed at
  device-add without anyone elevating; whether their projection matches this
  one is exactly the real-partitioned-Linux row that stays open above.

### Hardened non-WSL confirmation protocol — taken 2026-08-03; valid; non-separating

Protocol recorded 2026-08-02. Status: **taken 2026-08-03; valid on the third
sitting; non-separating** — the confirmation-row table and the sitting records
below the eligibility text are the evidence, and the two void sittings and
their recorded instrument amendments precede the valid one and are retained
rather than discarded. The remainder of this paragraph and the eligibility,
actor, custody, and gate text that follows are preregistration wording kept
verbatim; they state the conditions the third sitting met, not open
preconditions. It may run only in a fresh disposable non-WSL distro
VM — the available Proxmox host is one suitable T2 route — from a
checksum-pinned base image, with a stock distro kernel, after issue #94
closes **and its descriptor-bound implementation has landed and been
reviewed**. This protocol does not authorize improvising a pathname-based
substitute or treating a candidate described on the issue as the closure.

The measurement shell remains unprivileged and must record that its UID is
non-root, it has no `disk` or other block-read group, it has no ambient or
effective capability, and a direct loop read is denied. A separately declared,
narrow privileged setup actor may create and remove only the descriptor-bound,
read-only loop object for the approved fixture inside the disposable VM; it
accepts no path or general command from the measurement shell and grants that
shell no block-read privilege. A second narrowly scoped privileged comparison
actor may run only the predeclared read-only `blkid -p -o udev` and
`wipefs -n` probes against the closure-verified held loop descriptor (or a
node identity-bracketed immediately around the call if a tool cannot consume
that descriptor), with fixed arguments, sanitized environment, bounded output,
and a timeout. Its output is labelled privileged and never merged into the
client projection. Every external executable used by the measurement shell or
either privileged actor — including the descriptor-bound setup program,
`udevadm`, `blkid`, and `wipefs` — is predeclared by a trusted absolute path
from the checksum-pinned distro package set, validated and version-recorded
before privilege, and launched directly with structured argv, no shell and no
`PATH` lookup. The fixture set is generated afresh from the checked-out
revision. Raw transcripts stay outside the repository as a hashed review
artifact referenced by the result's pull request. Before any result enters
this file, a second reader must retrieve the artifact through that identifier,
rehash it, and confirm that it is readable; the evidence store must retain it
through SI-35's resolution and migration into release evidence. A failed
readback, missing transcript, or shorter retention commitment voids every
result. This file receives bounded facts, artifact digests, and pass/fail
binding predicates only.

The run has nine validity gates:

1. **Private, fresh scratch.** Create a new `mktemp -d` directory, require mode
   `0700`, and refuse a non-empty or reused directory. Cleanup targets only
   objects created in that directory and the loop object held by the setup
   process; it never discovers a cleanup target from a glob or a later path
   lookup.
2. **Descriptor-bound identity and rebinding exclusion.** Hold the verified
   backing-file descriptor and loop-device descriptor for the whole
   attach/probe/detach lifetime. Issue #94's mechanism must exclude or detect
   `LOOP_CHANGE_FD`: a held descriptor alone does not stop another privileged
   actor rebinding the loop. The attach actor drops its setup authority or
   becomes unreachable before measurement begins; any later detach actor has
   only the closure's exact held-object authority. After configuration, after
   partition scan and udev settlement, immediately before and after every
   external probe, and immediately before descriptor-bound detach, compare
   `LOOP_GET_STATUS64`'s backing device, inode, offset, size limit, and all
   flags with `fstat` on the held backing descriptor and declared fixture. Any
   mismatch voids the entire run; no result row may survive it.
3. **Byte continuity.** Hash the fixture through the held backing descriptor
   before attach, the complete loop device through the held loop descriptor
   before and after all probes, and the held backing descriptor after detach.
   All four readings must equal the generated manifest digest. This binds the
   measurement interval; the historical post-attach pathname check did not.
4. **Frozen projection.** Hash and record the exact probe script and normalizer
   before the first fixture is attached. The normalizer may drop only the six
   predeclared plumbing keys `USEC_INITIALIZED`, `ID_PART_ENTRY_DISK`,
   `ID_LOOP_BACKING_FILENAME`, `ID_LOOP_BACKING_FILENAME_ENC`,
   `ID_LOOP_BACKING_INODE`, and `ID_LOOP_BACKING_DEVICE`. It retains every
   other property. A newly discovered scratch path, loop name, major/minor,
   inode, device number, or other session-dependent field voids the run; the
   normalizer is not amended after output is seen.
5. **Negative controls before the decisive comparison.** Attach two distinct
   copies of `gpt-basic-512` under different scratch names and inodes, then
   repeat one copy in a fresh attach. Their normalized projections must be
   identical. These controls test plumbing removal and repetition before the
   healthy/conflicting comparison.
6. **Descriptor-derived observation identity.** Derive the sysfs/udev root from
   the held loop descriptor's major/minor at `/sys/dev/block/<major>:<minor>`,
   not from a remembered `/dev/loopN` name. Invoke probers through the held
   descriptor where their interface permits. Where a tool requires a device
   name, require that node's `st_rdev` to equal the held descriptor immediately
   before and after the call and repeat the status binding around it. Read back
   logical sector size through the block-device interface, require the whole
   loop and every materialized partition to remain read-only, and require that
   none is mounted. The candidate client projection is finite and frozen: all
   `E:` properties from the udev database entry for the loop and each
   materialized partition, plus disk `size`, `ro`, and
   `queue/logical_block_size`, and partition `partition`, `start`, `size`, and
   `ro` from those descriptor-derived sysfs roots. For a successfully captured
   complete udev database entry, a property absent from that entry is the
   comparison value `observed(absent)`, not a failed capture; compare the union
   of retained property keys across the pair with that explicit sentinel. An
   unrecognized retained `E:` key is data. It voids the run only if the
   negative controls establish that it is undeclared session plumbing. No
   result is generalized to an unenumerated client interface.
7. **Deterministic event completion.** Capture the expected add/change uevents,
   require `udevadm settle`, and require the healthy control's expected
   partitions and udev database entries before measuring a pair. Capture the
   complete client projection twice and require byte stability. A successfully
   established no-entry state is recorded `observed(absent)` and makes the
   decisive pair `inconclusive (udev coverage gate)`; inability to determine
   whether an entry exists is a capture failure and voids the pair. A missing
   expected event, an unexpected later event, an unstable or incomplete
   capture, or a missing fixed sysfs field also voids it. Absence of an optional
   udev property inside a successfully captured complete entry remains
   `observed(absent)` under gate 6.
8. **Replicated, order-balanced trials.** Run at least three fresh
   attach/probe/detach cycles for `gpt-basic-512` and
   `gpt-conflicting-tables-512`, alternating or pre-randomizing their order and
   bracketing the sequence with healthy controls. Before the first attach,
   commit the exact schedule — or the PRNG algorithm and seed that generates
   it — to the hashed protocol and transcript header; retain the actual
   per-trial order, and never reshuffle after a result is visible. Capture raw
   udev data, the exact named sysfs projection above, structured `blkid`
   properties, and
   structured `wipefs` signature type plus offset; do not reduce a privileged
   reading to offsets alone.
9. **Exact teardown.** Install cleanup for success, error, and signal paths.
   Detach only the exact held loop object after one final binding check, verify
   the backing association is gone, close the held descriptors, and only then
   allow VM rollback. A cleanup failure voids the run and is retained.

This confirmation protocol deliberately answers only SI-35's decisive
healthy/conflicting pair. The historical damaged-primary, missing-backup,
hybrid, and H-4Kn ride-alongs are **not** confirmation rows here: each needs its
own predeclared controls, at least three fresh order-balanced trials, and exact
outcome rules before a future run may promote it. In particular,
damaged-primary needs a valid-empty-GPT control, hybrid needs typed MBR/GPT
signature comparison, and H-4Kn needs a same-run 512-byte control plus an
asserted and read-back 4096-byte loop configuration. Until such instruments
are recorded, their historical observations remain bounded as above.

The environment record includes distro release, kernel configuration/build,
udev ruleset digest, util-linux versions, repository revision, generated
manifest digest, probe-script digest, normalizer digest, setup implementation
digest, VM base-image digest, preregistered schedule or PRNG algorithm/seed,
actual per-trial order, raw-transcript digest and retrieval identifier,
elevation assertions, and every binding-gate Boolean. A failed or
missing gate makes every hypothesis row `void (<gate>)`, never `refuted`,
`supported`, or `failed (mechanism)`.

The **named candidate projection** is recorded as non-separating only if its
predeclared fields are byte-identical in every valid trial, all controls and
bindings pass, and every required database entry, fixed sysfs field, and
capture is present and stable. A confirmed missing database entry is retained
as `observed(absent)` but makes the pair inconclusive under the coverage gate;
inability to establish entry presence or absence voids it. Optional
udev-property absence is compared as `observed(absent)`, and an otherwise
unknown retained key is data. That result
does not refute the existential H-separation hypothesis or prove that no other
Linux client-readable surface exists. Any normalizer change, identity mismatch,
undeclared session-dependent key, unexpected event, or incomplete trial is
`void` or `inconclusive`, never a negative. Privileged `wipefs` classification
is recorded separately and can neither repair nor replace the client result.

| Confirmation row | Result |
| --- | --- |
| Fresh disposable non-WSL VM and environment record | `established` — 2026-08-03, VM 9423, commit `b231e0f`, third sitting |
| Issue #94 descriptor-bound setup used; all binding checks pass | `established` — every session's crate bindings and node re-stats passed; sessions refuse on any mismatch and none refused |
| Manifest/loop/backing byte-continuity hashes agree | `established` — backing before/after and whole-device before/after all equal the compiled catalogue digest, in every session |
| Duplicate-name/inode and repeated-attach normalization controls | `established` — NC2 (distinct root and inodes) and NC3 (repeat attach) both byte-equal the NC1 baseline after the frozen normalizer |
| Named candidate projection: healthy versus conflicting | **`non-separating`** — byte-identical normalized projections in every valid trial |

A valid run can lift only this record's WSL2 promotion hold and satisfy the
loop-measurement part of M0.5. It cannot choose SI-35's option, supply the
chosen-option refusal proof, or substitute for SI-34's real-partitioned-Linux
and macOS observations.

#### First sitting, 2026-08-03 — void (gates 4 and 7); instrument amended

The instrument's first execution, in fresh disposable Proxmox VM 9421 (stock
Ubuntu 22.04.5, kernel 5.15.0-186-generic, snapd/udisks2 purged, repository
commit `491d10f`, base image and environment digests in the retained record).
The privileged capture half completed all ten scheduled sessions with exit 0 —
every session's crate bindings, whole-device hashes, and confirmed teardown
passed, and `losetup -a` was empty afterwards. The unprivileged projection
half then **voided the run**, exactly as the protocol requires:

- **Gate 4 void — an undeclared session-dependent field.** `E: DISKSEQ`, the
  kernel-assigned monotone disk sequence counter, is present on this kernel
  and increments per attach (`22` on control NC1 against `26` on the same
  root's repeat NC3). It is not among the six preregistered droppable keys,
  so every control and trial-coherence comparison failed and the run is void.
- **Gate 7 void, rendering instability.** Two back-to-back `udevadm info`
  captures of the same partition rendered `E: DEVLINKS` with the **same
  token set in different orders** (`by-partuuid … by-partlabel` against
  `by-partlabel … by-partuuid`), failing byte-stability on several partition
  subjects.
- **Gate 7 void, event capture.** Every session's passive monitor recorded 2
  udev block `add` events where 1 + partitions = 3 were required: the
  listener was spawned but its netlink subscription raced the attach's first
  event burst.

Every confirmation row for this run is therefore `void (gate 4; gate 7)` —
never `refuted` or `supported` — and the decisive-pair output the projection
printed is **not usable and is deliberately not quoted here**: with an
undeclared varying field in every projection, "separates" is contaminated by
plumbing. No hypothesis moves. The raw capture, full transcript, and
environment record are retained outside the repository
(transcript SHA-256
`2c2528d7fabdb66da77af172a0395a306c328fe92d2e775e4d044e14ca71d067`), and the
VM was reverted to its pre-sitting snapshot afterwards.

**Amendment recorded 2026-08-03, before any subsequent run's output.** The
three defects are instrument defects, justified by the validity failures
alone:

1. `DISKSEQ` joins the normalizer's droppable keys. Reasoning: it is a
   kernel-assigned monotone attach counter carrying no content information —
   the same session-plumbing class as `USEC_INITIALIZED` and the loop backing
   keys; leaving it retained makes every fresh attach trivially distinct,
   which is exactly what the drop list exists to prevent. The list is now
   seven keys and remains exact.
2. One declared rendering canonicalization: within an `E: DEVLINKS` line the
   space-separated tokens are sorted before comparison and projection. Every
   token is preserved — a genuinely different symlink set still differs —
   only udevadm's set-rendering order is removed.
3. The monitor waits for `udevadm monitor`'s readiness banner before the
   session may configure, and refuses if none appears, so the first uevent
   burst can no longer race the subscription.

Gate texts, the schedule, trial counts, hypotheses, and outcome rules are
untouched. The next sitting runs the amended instrument from a pre-sitting
snapshot revert.

#### Second sitting, 2026-08-03 — void (gate 7); instrument amended again

Run in fresh disposable VM 9422 at the amended commit `bc922f0` (same base
image, same purge deviation; VM 9421's crash-consistent live snapshot proved
unusable for revert — a lesson recorded for operations, not protocol — so the
sitting used a freshly provisioned VM, which is what the protocol prefers
anyway). The capture completed all ten sessions with exit 0. The first
amendment held: **every control and trial-coherence comparison passed** with
`DISKSEQ` dropped. The projection still voided the run, on gate 7 alone, for
two narrower instrument defects:

- **Stability**: the comparison was implemented over udevadm's whole
  rendering, and the `S:` symlink block renders its set in varying order
  between back-to-back queries — the same set-order nondeterminism `DEVLINKS`
  shows, in a section that is not part of gate 6's projection at all. The
  canonicalized `E:` sequences were identical.
- **Events**: the requirement demanded 1 + partitions udev `add` events, but
  a **preallocated loop node emits no disk `add`** — this kernel pre-creates
  `loop0`–`loop7`, so attach produces disk `change` events plus one `add` per
  partition. Captured event streams show exactly that shape, listener
  readiness confirmed.

The decisive-pair line the void run printed is again unusable and not
evidence. **Amendment recorded 2026-08-03, before any subsequent run's
output:** the stability comparison is over the projection gate 6 defines —
the `E:` property sequence with the declared `DEVLINKS` canonicalization —
never udevadm's addressing or symlink-presentation lines; and the event gate
requires udev adds ≥ partitions observed plus at least one disk `change`,
matching what a preallocated node can emit. Gate texts, schedule, trials,
hypotheses, and outcome rules remained untouched. Raw capture and transcript
retained externally (transcript SHA-256
`f435cf5b6e63a68100e39d8425985436eec3a1acde44b4cd4de12bd486be5974`).

#### Third sitting, 2026-08-03 — VALID; every gate passed; non-separating

Run in fresh disposable Proxmox VM 9423 at the twice-amended commit
`b231e0f99826867c25c13194752808ac6c21aec6` — stock Ubuntu 22.04.5 cloud image
(base digest in the retained host record), kernel 5.15.0-186-generic, snapd
and udisks2 purged as the recorded deviation, no USB or PCI passthrough, VM
destroyed with post-destroy verification afterwards. `cargo xtask ci` passed
inside the guest before the instrument ran; the no-token negative control and
the projection-as-root negative control both refused as required.

**Every validity gate passed.** All ten scheduled sessions completed: crate
bindings, node re-stats, whole-device and backing byte-continuity hashes
(every reading equal to the compiled catalogue digest), confirmed detach and
partition teardown per session, `losetup -a` empty afterwards. Stability:
both udev captures of every subject projection-identical. Controls: NC2
(distinct root and inodes) and NC3 (repeat attach) byte-equal the NC1
baseline; the closing healthy control equal again after six intervening
trials. Trials: the three basic projections equal the baseline; the three
conflicting projections equal each other. Events: each session's passive
monitor shows the preallocated-node shape exactly — one udev `add` per
materialized partition plus disk `change` events, listener readiness
confirmed. The unprivileged projection half ran as uid 1001 with no `disk`
group, all-zero `CapEff`, and denied direct reads of `/dev/loop-control` and
`/dev/loop0`, recorded in-transcript.

**Result — the named candidate projection is `non-separating`.** In every
valid trial, the frozen client projection (all retained `E:` properties for
the disk and both partitions under the amended normalizer, plus the named
sysfs facts) was byte-identical between `gpt-basic-512` and
`gpt-conflicting-tables-512`. The conflicting-primary-versus-backup condition
is invisible to this projection on this platform. Separately and labelled
privileged, never merged into the client result: the retained
`blkid -p -o udev` and `wipefs -n` outputs for the decisive pair were also
identical — the same negative the historical Windows and WSL2 records
reported for their retained finite projections, now on qualifying ground.

**What this run establishes and lifts.** Per the protocol's own scope: it
confirms the historical WSL2 decisive-pair negative through the
descriptor-bound, handle-verified mechanism, **lifting the WSL2 promotion
hold**, and it **satisfies the loop-measurement part of M0.5**. What it does
not do, in the protocol's words: it does not choose SI-35's option, does not
supply the chosen-option refusal demonstration, does not substitute for
SI-34's real-partitioned-Linux and macOS observations, and does not refute
the existential H-separation hypothesis — no claim is made about client
interfaces outside the enumerated projection.

**A coverage gate added after the result, and why that is defensible here.**
Reviewing the instrument against gate 7 after this sitting found a latent
false-pass path: the projection compared normalized strings, so two
*successfully captured but empty* udev entries would have compared equal and
printed `non-separating` — a negative produced by measuring nothing, where
gate 7 requires `inconclusive (udev coverage gate)`. The instrument now counts
retained properties per subject and reports `observed(absent)`, failing the
run rather than the pair. The reachable shape is narrow, because a udev query
exiting outside its allowed set is already refused at capture time; what was
missing was the exit-0-but-empty case.

Adding a gate after seeing output is exactly the move this protocol distrusts,
so the claim is bounded rather than asserted. The added gate is **strictly
stricter** — it can turn a pass into inconclusive and never the reverse — and
the verdict was **re-derived rather than assumed**: the amended projection
half was re-run over the retained raw capture (digest verified first), off the
sitting host, on a different Linux machine, as an unprivileged user, and
reported the identical verdict with every gate passing. Coverage was
substantial, not marginal: 12 retained properties for the disk and 23 for each
partition, in every session. Had this sitting's entries been empty, this run
would have been `inconclusive`, not a negative — and they were not.

**Artifacts.** Transcript SHA-256
`76bbd9e122d6d672e153b7d522f801ec9d5c9e668b741ec9b2223e22ce52b994`; raw
capture SHA-256
`8af58b262bca69695b886519033a5dfebebf4929cabacdbc8e44a6b111c7700a`; retained
with the void runs' records in the operator evidence store
(`%USERPROFILE%\partman-evidence\SI-35-sitting-2026-08-03\`, custodian
Nate McBride), alongside the
host environment and teardown proofs for VMs 9421–9423. Per the protocol's
custody rule, a second reader must retrieve and rehash the transcript before
this record is relied on; the result pull request carries that obligation.
The obligation is discharged twice over: the readback recorded on the
result pull request under the operator's designation (performed by the
producing session and recorded there as not independent), and an
independent reader session on 2026-08-04 that retrieved both artifacts
through the locator and rehashed each to its recorded digest, both
matching.

#### Mechanism amendment, 2026-08-03 — recorded before any output exists

No measurement has been taken and every result cell above stays exactly as
preregistered. This subsection amends the protocol's **mechanism** — who
performs which step — because the preregistered actor arrangement turned out
not to be implementable as written, and WP-020's increment 2f boundary
requires the substitution recorded here, with reasoning, before any
measurement output exists. The gates, hypotheses, normalizer rules, trial
counts, and outcome vocabulary are untouched.

**What was found.** Repository issue #130: the descriptor-bound setup program
this protocol names did not exist, and no distro ships one — `losetup`
resolves its path argument in userspace, the exact gap issue #94 named. The
owner chose to extend `crates/ffi-linux-loop`, and WP-020's increment 2f
authorization (merged 2026-08-03) grants the crate a hold-open session with a
**crate-owned** prober launch. The implemented entry point is
`run_probed_session`.

**Why the actors merge.** A design in which the setup actor lends the
measurement side the held loop descriptor, a borrowed view of it, or the node
name was evaluated and rejected on a live measurement:
`BorrowedFd::try_clone_to_owned` is safe, stable Rust whose `OwnedFd`
outlives the borrow, the supervised window, and the detach, and the session's
loop node is opened `O_RDWR`. A lent identity therefore cannot be confined by
construction, and a boundary that pretended otherwise would be the
declared-not-computed safety this repository refuses. The consequence:

- The **setup actor and the privileged comparison actor become one
  process** — the crate session configures the loop from the held verified
  descriptor and itself launches the predeclared `blkid -p -o udev` and
  `wipefs -n`, under the same launch controls the protocol already required
  (compiled absolute paths, fixed argv, sanitized environment, bounded
  output, a timeout). Their outputs remain labelled privileged and are still
  never merged into the client projection.
- The **live client-projection capture also moves into the session's
  predeclared launches** (`udevadm settle`, `udevadm info --query=all`
  against the disk and each materialized partition), because the increment 2f
  boundary forbids disclosing the live device identity to any caller — the
  unprivileged shell included — while the device is bound. Validity
  reasoning, stated rather than assumed: the udev database and the sysfs
  attributes this protocol names are world-readable state whose content does
  not depend on the reader's privilege, so root-launched capture changes who
  reads, not what exists to be read. What the unprivileged measurement shell
  loses is only *live addressing* of the device, which is exactly what the
  boundary is for.
- The **unprivileged measurement shell keeps** everything else it was for:
  recording its non-root UID, absent `disk`/block-read groups, empty
  capability sets, and a denied direct read against the loop-device class;
  and performing all post-release analysis and normalization over the
  quarantine-released records. The session releases captured output only
  after `ENXIO`-confirmed detach and partition teardown, so nothing the
  shell analyzes existed for it while the device was bound.

**One preregistered mitigation is substituted, and the substitute is
weaker.** Gate 2 asked the attach actor to drop its setup authority or become
unreachable before measurement begins. The merged session cannot: its
authority *is* the held descriptors, and verification and confirmed detach
need them. What replaces authority-dropping is the bracket the crate enforces
as control flow — node `lstat` identity plus the full `LOOP_GET_STATUS64`
backing/flags/geometry binding re-verified immediately before and after every
external launch, any mismatch voiding the session with nothing published.
Per the increment 2f boundary this **detects a rebind that happened rather
than preventing one**, and no evidence produced under it may be described as
carrying the authority-dropped design's strength. The session itself contains
no `LOOP_CHANGE_FD` call; only increment 2e's adversarial leg exercises one.

**What this amendment does not do.** It takes no measurement, moves no cell,
and does not touch gate 4's frozen normalizer, gate 5's negative controls,
gate 7's event-completion rules, or gate 8's replicated order-balanced
trials — those are obligations on the future instrument run, which will
invoke one session per attach/probe/detach cycle. The instrument's own
runner, schedule commitment, and transcript custody remain WP-035 work that
does not exist yet.

### Increment 6 real-partitioned-Linux matrix — preregistered 2026-08-02

(The heading above is restored: the 2026-08-03 mechanism-amendment commit
deleted exactly this heading line while inserting its own subsection,
leaving the matrix body orphaned under the amendment. The body below is
byte-unchanged from its preregistration except the status line and result
cells, which the 2026-08-04 sitting fills.)

Protocol recorded 2026-08-02 under WP-035 increment 6. Status: **taken
2026-08-04; every row executed** — the sitting record below the table is
the evidence. It is a sibling of, not a substitute for, the
SI-35 confirmation protocol above: that one answers the loop decisive pair
behind issue #94; this one takes the real-device-tree and per-technology rows
the register names, which issue #94 does not gate. It records no measurement,
changes no register disposition, decides no option, and satisfies no
platform-adapter criterion.

**Environment.** A fresh disposable non-WSL distro VM from a checksum-pinned
base image with a stock distro kernel — the Proxmox host is the intended
route — plus **explicitly provisioned, explicitly disposable physical USB
fixture media passed through to the VM**. A virtual disk may not be
represented as real-hardware evidence; rows below that say *real medium* are
valid only against the passthrough device. The environment record includes
distro release, kernel version and build, udev version and ruleset digest,
util-linux/mdadm/cryptsetup/lvm2/zfs tool versions, repository revision and
generated manifest digest, VM base-image digest, and — per row — effective
UID, effective capabilities, relevant group memberships, and the tested
device node's ownership, mode, and ACLs. Every external executable is
predeclared by trusted absolute path from the pinned distro package set,
launched with structured argv, sanitized environment, bounded output, and a
timeout.

**Provisioning boundary, stated against this file's Method section.** The
Method section's sentence — no experiment writes to a block device — remains
true here: **no experiment row writes**. Provisioning the fixture media
(writing a partition table and one technology structure per declared layout
onto the disposable USB medium, then setting the device read-only with
`blockdev --setro` and re-verifying the provisioned digest) is performed by a
separately declared privileged setup actor inside the disposable VM, before
measurement, with its own transcript. It accepts no path or command from the
measurement shell. This is a wider setup class than Method's current
"regular scratch files and virtual-container files" wording; the PR that
lands this protocol's first executed run must widen that sentence explicitly
rather than let it silently lapse — recorded here so the conflict is filed
before it exists on disk.

**Baseline discipline.** The ordinary-client baseline must lack raw block
access: non-root, no `disk` or other block-read group, no ambient or
effective capability, and a recorded denial of `dd` and `blkid -p` against
the device node. A `disk`-group leg and a root helper leg are separately
labelled projections and may not substitute for, or be merged into, the
baseline. The result vocabulary is the closed set defined by the macOS matrix
above, and gate failures make cells `void(<gate>)`, never negatives.

Declared layouts, provisioned one at a time on the same medium, each with its
digest recorded: **L-A** GPT with one ext4 partition (identity/designator
baseline); **L-B** GPT with one mdraid 1.2 member partition; **L-C** GPT with
one LUKS2 partition (`luksFormat` only — never opened); **L-D** GPT with one
LVM2 PV carrying one VG (metadata area present, no LV activated); **L-E** GPT
with one ZFS vdev label pair (pool exported before measurement); **L-F** the
`ext4-with-stale-mdraid-090` byte pattern written to the partition (SI-34's
stale-signature case on a real device tree). APFS has no Linux row:
`not-applicable(platform)`. Precondition 2's Windows designators — the
Storage Spaces pool object id and LDM group GUID — are not this matrix's
rows; they belong to the Windows record and L7 is not the complete
designator table without them.

| # | Cell | Command / API | Privilege | Distinguishing condition | Invalidation conditions | Result |
| --- | --- | --- | --- | --- | --- | --- |
| L1 | Baseline denial, real medium | `stat`/`getfacl` on the node; `dd` one sector; `blkid -p` | client baseline | The stock-permission claim (`brw-rw---- root:disk`) holds for a real passthrough device in this VM, and the baseline truly lacks raw access | node ACL-granted or mode nonstandard without being recorded | `observed(denied)` — stock `brw-rw---- root:disk`, base ACL only; the baseline's `dd` and `blkid -p` both refused; the identical operations succeed for the `disk`-group user (L6) |
| L2 | Real-device identity rows | `/run/udev/data/b<maj>:<min>`; `/sys/.../device/{vendor,model,wwid}` on the SCSI device node and the USB device node's `serial` via `device/../../../../serial`; by-id symlink set | client baseline | Which identity facts (serial, WWN, bus, path) a real USB-attached device of this class actually exposes to the client — the WSL2 rows could not answer this | udev entry absent or unsettled (`udevadm settle` required); device re-enumerated mid-capture | `observed` — USB descriptor serial via the USB device node (exact path in the 2026-08-13 readback below; `device/serial` on the SCSI node was never read), vendor/model strings, serial-derived `by-id` and path-derived `by-path` symlinks, populated udev db entries; `wwid`: the read failed `ENXIO` at every capture (previously stated `observed(absent)`; restated by the readback below) |
| L3 | Kernel table view per layout | `/proc/partitions`; `/sys/.../start,size,ro,partition` for each materialized partition | client baseline | Kernel-materialized partition set for a real medium per layout L-A…L-F | partition count unstable across two captures; medium not read-only at capture | `observed` — every layout materialized its declared one-partition set with exact start/size, byte-stable across the double capture, device and partition `ro=1` at every capture |
| L4 | Client signature view per layout (precondition 1, client half) | udev `E:` properties (`ID_FS_TYPE`, `ID_FS_UUID`, `ID_FS_USAGE`, raid metadata keys) for L-B/L-C/L-D/L-E/L-F | client baseline | What the cached, event-time udev projection says per technology — including which single answer it gives for L-F's live-plus-stale pair and whether `ID_FS_AMBIVALENT` fires on a real device | incomplete udev DB capture (absence of an entry is `observed(absent)` only when established); event after capture | `observed` per technology — mdraid: type, array UUID, member sub-UUID, host-qualified name; LUKS2: type and UUID; LVM2: type and PV UUID (`ID_FS_USAGE=raid`); ZFS: `observed(absent)` — `ID_FS_TYPE` empty in both stability captures, the event-time-cache mechanism in the sitting record; L-F: the single answer is exactly the stale `linux_raid_member`, the live ext4 absent; `ID_FS_AMBIVALENT` fired nowhere in the sitting |
| L5 | Helper signature view per layout (precondition 1, helper half) | `blkid -p -o udev`, `wipefs -n`, `mdadm --examine`, `cryptsetup luksDump`, `pvs --readonly -o pv_uuid,vg_uuid,vg_name`, `zdb -l` — all read-only forms, fixed argv | root, VM only | The direct-probe projection per technology over the same bytes, to stand against L4 — establishing or refuting client/helper signature agreement per precondition 1's instruction to establish rather than assume | any tool invocation not on the predeclared list; output unbounded; device writable | `observed` per technology — every predeclared probe returned its structure read-only; where the client projection was empty for ZFS, `blkid -p` reports `zfs_member` with pool and vdev GUIDs and `wipefs -n` all four labels; for L-F `wipefs -n` enumerates **both** signatures (end-anchored stale raid and live ext4) while root `blkid -p` reports exactly the stale one; `pvs` carries the VG UUID no client surface has |
| L6 | `disk`-group projection | L4's and L5's client-executable subset re-run as a `disk`-group user | disk group, VM only, separately labelled | Whether group membership alone changes the observable set — the clamping obligation's concrete form: two users, one host, one build, different views | leg merged into baseline; group state not recorded per row | `observed` — group membership alone flips both raw-access denials to success and grants the direct-probe subset; the cached udev/sysfs projection is identical between the two users; separately labelled per capture |
| L7 | Native designators (precondition 2) | from L4/L5: mdraid array UUID; LUKS UUID; LVM2 **VG id** (not PV UUID) and which interface, if any, yields it clientside; ZFS pool GUID | per source row | For each designator: raw byte form, source, privilege needed, and specifically whether a member-independent designator is client-readable at all — where none is, the register's indeterminate-aggregate consequence is the recorded fact | conflating PV UUID with VG id; reducing a designator to its rendered string without its source | mdraid array UUID: **client-readable** (udev `ID_FS_UUID`); LUKS2 UUID: **client-readable**; LVM2 VG id: **`not-client-readable`** — the client carries only the PV UUID and no client surface carried the VG UUID (root `pvs` only): the indeterminate-aggregate consequence, instantiated; ZFS pool GUID: helper-only in this sitting's cached projection, with L4's event-time boundary declared |
| L8 | Designator stability | detach passthrough, reattach, recapture L2/L4/L7; one full VM reboot, recapture | client baseline | Which identity facts and designators survive replug and reboot unchanged on real hardware | fewer than two sittings; medium reprovisioned between sittings | `observed` — with L-F provisioned throughout, the array UUID, USB descriptor serial, and `by-id` set survived a passthrough replug and a full VM reboot unchanged; `wwid` stayed absent; the replug's plug event visibly re-probed the udev db to the same single answer |
| L9 | Designator collision | second physical medium provisioned byte-identical to L-A; both attached | client baseline | Client-visible collision semantics: `by-uuid`/`by-id` symlink behavior, udev entries, and whether either interface signals the duplicate | **conditional on a second authorized medium**; without it `not established`; media not byte-identical by digest | `observed(silent last-writer-wins)` — both byte-identical media (digest-verified over both declared windows) enumerate fully with identical identity records; `by-uuid`, `by-partuuid`, and `by-label` each collapse to the later-arriving device; no surface signals the duplicate; only the bus-serial-derived `by-id` remains distinct on this hardware |
| L10 | SI-34 freshness projection, real medium | L4 versus L5 over L-F, captured in the same sitting | both, separately labelled | The regular-file finding — enumerating interface reports both signatures, single-answer interface reports exactly the stale one — established or refuted on a real device tree, which is the projection input SI-34 names | either half missing; bytes not digest-verified before and after | `observed` — **established on a real device tree**: both single-answer interfaces (the client's cached projection and root `blkid -p`) report exactly the stale mdraid signature; the enumerating `wipefs -n` reports both; bytes digest-verified before and after the captures |

Validity gates, all required per sitting: fresh VM and recorded environment;
provisioned-digest verification before and after each layout's measurements,
performed by the setup actor through the read-only device — the client
baseline cannot read the device and never verifies digests itself;
`udevadm settle` plus double-capture byte
stability for every udev read; no mount of any measured object at any point
(asserted, not assumed — `/proc/self/mounts` captured before and after);
automount and repair services confirmed absent or disabled; explicit
per-layout teardown by the setup actor only. Transcript custody is identical
to the macOS matrix: outside-repository archive with locator and custodian,
hash algorithm/digest/byte length, capture-script digest recorded before
first capture, per-command exit statuses, anonymized device-role mapping
rather than machine identifiers, and second-reader retrieve-and-rehash before
any cell leaves `not yet taken`.

A valid run establishes only the rows above. It cannot decide SI-34 (the
projection choice is a register decision over the evidence, not the
evidence), cannot lift the loop protocol's own holds, and does not touch
issue #94 in either direction.

#### Sitting, 2026-08-04 — every row taken

**Environment.** Disposable VM 9424 on the authorized Proxmox host, fresh
from the digest-verified jammy cloud image (base digest as recorded in the
transcript header, equal to the WP-020 acceptance's), Ubuntu 22.04.5,
kernel 5.15.0-186-generic, pre-sitting snapshot as revert boundary; udev
249.11 with the ruleset digest recorded; util-linux 2.37.2, mdadm 4.2,
cryptsetup 2.4.3, lvm2 2.03.11, zfsutils 2.1.5, and the acl package added
from the pinned distro set mid-sitting (amendment recorded before use);
udisks2 absent, no automount unit, snapd present and recorded rather than
purged — this protocol names no loop-administrator exclusion. Fixture
media: the two operator-authorized SanDisk sticks, byte-equal capacities,
passed through as USB; roles medium-1 (all layouts, L8) and medium-2 (L9
only); the operator's hard boundary — the host NVMe drives are never
touched — held structurally, every destructive action confined to the VM's
virtual disk and the passthrough media. The L-F byte pattern is the
generated `ext4-with-stale-mdraid-090-512` fixture, digest equal to the
generated MANIFEST's at the recorded repository revision, written to a
partition sized exactly to the image so its end-anchored stale superblock
stays end-anchored. Actor separation as declared: root setup/helper,
`muser1` baseline (no groups, empty capability set, denials recorded),
`muser2` disk-group leg, all captures separately labelled.

**Corrections and incidents registry, every one caught by a declared gate
before any cell was derived.** (1) A carriage-return transfer artifact on
the instrument scripts: recorded, scripts amended, digests re-recorded,
environment record retaken. (2) `getfacl` absent from the image: the acl
package installed and versioned before the row needing it. (3) The client
instrument's maj:min derivation produced `0:0` under mawk; amended to read
sysfs, prior capture superseded in place. (4) The first L-E attempt wedged
in `zpool export` for ~35 minutes on the emulated USB2 controller the
passthrough had landed on (kernel hung-task trace retained); the
passthrough was detached — releasing the wedge — reattached onto XHCI, the
VM rebooted, and the layout re-run; the first attempt filled no cell.
(5) A CR-corrupted runner invocation made the provision dispatch refuse
mid-layout, leaving a bare partition whose interleaved captures fill no
cell; re-pushed via scp and re-run. (6) The L9 head-window copy passed a
block count where a byte count was expected, moving 129 bytes; the
byte-identity digest gate refused the state and the corrected copy
re-verified identical. Transcript blocks interleave near incidents (4) and
(5); every block carries its own UTC timestamps, and ordering is
reconstructible from them.

**Findings the cells compress.** The client/helper signature asymmetry is
now measured on real hardware in both directions: for ZFS the cached
client projection is empty while the helper reads the full label set —
the event-time-cache mechanism, since neither pool creation nor export
re-triggers a partition uevent, with the replug leg separately showing a
plug event does re-probe; for the stale-plus-live pair both single-answer
interfaces (client cache and root `blkid -p`) give exactly the stale
answer while only the enumerating probe reveals the pair. The LVM2
member-independent designator is helper-only. And the collision row's
silent last-writer-wins on every UUID-keyed symlink farm — resolved by
arrival order, signalled nowhere — is the Linux face of the identity
collapse the Windows S4 sittings measured on same-model readers: there
the bus-serial layer collapsed too, here it is the one surface that held.

**Custody.** Complete transcript and all instruments archived at
`%USERPROFILE%\partman-evidence\2026-08-04-lmx-sitting1\` on the operator
workstation, custodian Nate McBride; SHA-256 throughout; instrument
digests recorded in the transcript header before any capture and
re-recorded at each amendment before first use; per-command exit statuses
and per-layout provisioned-digest brackets in-transcript, verified through
the read-only device by the setup actor before and after every layout's
captures; anonymized device-role mapping (media addressed by role; unit
serials stay in the transcript); post-sitting no-mount assertion captured
(no measured object was ever mounted). Transcript
`6da1db67d58fb49f47a42614d00343b60ad07b7c52493a9e198c34a57030df71`
(164843 bytes). An independent second reader retrieved the transcript
through the locator, rehashed it to the same digest and byte length, and
confirmed the archive's file inventory before any cell left
`not yet taken`. The VM was destroyed with post-destroy verification
(config, volumes, and snapshot absent); the media remain provisioned with
the L9 pair and are re-writable fixture stock.

### Readback rows, 2026-08-13 — transcribed from the archived transcript; no new sitting

Issue #318 items 1 and 2 (both blocking WP-L100 increment 3) plus item 3's
value-transcription half, closed by reading the archived 2026-08-04
transcript back into this record. Nothing here is a new measurement: every
value below sits in the transcript bound above, and this section names its
exact source. Custody re-verified before transcription: the archived
`transcript.txt` rehashed 2026-08-13 to
`6da1db67d58fb49f47a42614d00343b60ad07b7c52493a9e198c34a57030df71` at
164843 bytes — both matching the recorded values.

**R1 — the serial's exact source.** The L2 serial was read by the client
instrument (`l-client.sh`, its digest in the transcript header) as
`cat /sys/block/sdb/device/../../../../serial` — a parent traversal from
the SCSI device node four levels up to the **USB device sysfs node**, whose
`serial` attribute is the USB descriptor's iSerialNumber. The transcript
holds no `realpath` of that traversal, so this row names the as-executed
path and its structural resolution, not a canonical absolute path. Value
`A20036CA8695D921`, identical at all five full-scope captures and stable
across the L8 replug and reboot. **`/sys/block/sdb/device/serial` — the
SCSI-device-node attribute the earlier interface column's
`device/{vendor,model,wwid,serial}` spelling suggested — was never read in
this sitting**, and no qualifying record reads it on any Linux host; the
row above is corrected to name both facts rather than license the
misreading. The udev database carries the same value as `ID_SERIAL_SHORT`
(every DB capture, disk and partition nodes), and the `by-id` name embeds
it — both database-side derivations of udev's own probing, not independent
client observations. The L9 destination unit answered the same USB-node
interface with a distinct value (`A2003886B8F0D722`), so on this two-unit
SanDisk pair the interface is per-unit distinguishing — the contrast case
to the SI-28 S4 card-reader pair, whose shared-constant serial collapsed
at every layer.

**R2 — `wwid`: a failed read, not an observed absence.**
`cat /sys/block/sdb/device/wwid` failed with `No such device or address`
— `ENXIO` — at every capture, all five full-scope captures across layouts
and both L8 stability legs. `ENOENT` would have printed
`No such file or directory`: the errno shape says the attribute file
existed and reading it failed, though the transcript holds no directory
listing of `/sys/block/sdb/device/`, so "the file existed" is an
errno-shape inference and is recorded as such. The L2 result cell
previously compressed this to `observed(absent)`; restated, because the
distinction is load-bearing: ADR-0019's naming rules currently define
neither a failed-read outcome nor a measured-absent one, and whoever
closes those seams needs this row at its true shape — on this hardware the
`wwid` source produced a **read failure**, not a clean absence.

**R3 — sizes, and the one unit that is measured.** The setup actor's
provisioning guard read the whole device as
`blockdev --getsize64 /dev/sdb` = **125162225664** bytes (printed as
`guards passed: TRAN=usb SIZE=125162225664`, every provisioning, root
actor). The client-readable `/proc/partitions` whole-disk row reports
`122228736` blocks; 122228736 × 1024 = 125162225664 exactly, establishing
`/proc/partitions`' 1 KiB block unit on a whole device against a
byte-denominated interface. The partition node's sysfs attributes read
`start=2048`, `size=1048576` (client baseline, byte-stable across every
double capture); the setup actor declared that partition
`sgdisk -n 1:2048:+512M` — 512 MiB = 536870912 bytes — and
1048576 × 512 = 536870912 exactly, with `/proc/partitions`' `sdb1`
row (`524288` blocks × 1024) agreeing: **the sysfs block `size`
attribute's unit is measured as 512 bytes on the partition node**,
against a declared byte extent and a second interface. The instrument's
own digest arithmetic corroborates it: the head window
`(start+size)×512 + 1048576 = 538968064` bytes was printed, read with
`head -c`, and digest-verified before and after every layout's captures.
**The whole-device sysfs `size` attribute (`/sys/block/sdb/size`) was
never read in this sitting**; the unit measurement exists on the
partition node alone. Extending the 512-byte convention to the
whole-device node is a decision, not a measurement, and belongs to the
record of whoever consumes it — WP-L100 increment 3's record, per the
directed acceptance — not to this readback.

**R4 — udev identifier values for this device class** (issue #318 item 3's
transcription half; the transport-discrimination *protocol* question is
untouched and keeps its own recorded grant question). On every database
capture of this real USB mass-storage unit: `ID_BUS=usb`,
`ID_USB_DRIVER=usb-storage`, `ID_USB_INTERFACES=:080650:`,
`ID_TYPE=disk`. `ID_PATH` was observed with **two values in one sitting**:
`pci-0000:00:1d.7-usb-0:1:1.0-scsi-0:0:0:0` before incident (4)'s
controller reattachment and `pci-0000:01:1b.0-usb-0:1:1.0-scsi-0:0:0:0`
after — the same physical unit, re-pathed when the passthrough moved from
the emulated USB2 controller to XHCI. That is measured evidence that
`ID_PATH` names attachment topology, not the unit, on exactly the
hardware class where the distinction matters.

What this closes and what it does not: #318 items 1 and 2 asked for
transcription, and these rows are it. The ADR-0019 Linux naming-source
designation remains **unmade** — it is a normative act landing only with a
spec change, and nothing in this section makes it. These rows are what
that act can now rest on. (The designation has since been made: ADR-0034,
spec 12.11.0, on these rows.)

### The floor-rows sitting — preregistered 2026-08-13; taken the same day; valid on its second invocation

Five cells, declared before execution per this document's method. They
close issue #318 items 4, 5, and 6 (three of SI-28's floor inputs and
adjacent unmeasured reads), discharge ADR-0034's evidence obligation 1
(the designated serial source's canonical path), and close the one gap
the 2026-08-13 readback stated plainly: no record reads the
whole-device sysfs `size` attribute.

Apparatus: one disposable Proxmox VM (fresh jammy image against the
pinned digest), **one** of the two authorized SanDisk fixture sticks
passed through on the XHCI controller (the L-E wedge lesson), media
content as the prior sitting left it — no provisioning, the device set
read-only with `blockdev --setro` before any capture. Client-baseline
measurement user (non-root, no `disk` group, no capability), root
driver for the two root-side cells. Instrument derived from the
archived `l-client.sh` lineage with its digest recorded in-transcript
before first capture.

| # | Cell | Command / API | Privilege | Distinguishing condition | Invalidation conditions | Result |
| --- | --- | --- | --- | --- | --- | --- |
| FR1 | `removable` on a real whole device | `cat /sys/class/block/<dev>/removable`, double capture | client baseline | The SI-28 floor input's value for a real USB mass-storage device — no qualifying row exists on any Linux host, though both WP-035's CLI and WP-L100's adapter read it | value unstable across the double capture; device re-enumerated mid-capture | `observed` — `1`, rc 0, byte-stable across both captures: the first qualifying `removable` value on any Linux host, and it is the value the SI-28 floor needs for this device class |
| FR2 | Physical and logical block size, real hardware | `cat /sys/class/block/<dev>/queue/physical_block_size` and `queue/logical_block_size`, double capture | client baseline | Both values on real non-virtual hardware — measured on WSL2 virtual SCSI only, and the one non-WSL frozen projection names the logical size alone | value unstable across the double capture | `observed` — physical `512`, logical `512`, both rc 0, byte-stable: the first real-hardware `physical_block_size` row |
| FR3 | The whole-device discriminator | `ls /sys/class/block/<dev>/` and `ls /sys/class/block/<dev>1/`; `cat` of `partition` on both nodes with exit status recorded | client baseline | Whether a whole device **positively lacks** the `partition` attribute while its partition carries it — the admission rule both delivered implementations use, resting today on a code precedent inside a non-qualifying record | either listing incomplete; the partition node absent at capture | `observed` — on the whole device the read fails `ENOENT` (`No such file or directory`, rc 1) with the attribute absent from the directory listing — a measured absence, the `ObservedAbsent` shape, not a failed read; on the partition the attribute reads `1`, rc 0; both byte-stable. The admission rule now rests on a qualifying measurement |
| FR4 | The designated serial source's canonical path | `realpath /sys/block/<dev>/device/../../../../serial` beside `cat` of the same path, double capture | client baseline | ADR-0034's evidence obligation 1: the as-executed traversal's resolved absolute path, closing the structural-resolution inference the designation currently carries | realpath and cat disagree on target; value differs from the udev-recorded serial | `observed` — the traversal resolves to `/sys/devices/pci0000:00/…/usb10/10-1/serial`, a **USB device node's** `serial` attribute exactly as ADR-0034's structural rule states; the value read via the traversal and via the resolved path is identical (`A20036CA8695D921`, the recorded serial of this unit); byte-stable. The designation's structural-resolution inference is discharged (this apparatus presents QEMU's passthrough topology — the structural claim, not the specific path, is the measured fact) |
| FR5 | Whole-device sysfs `size` against a byte interface | `cat /sys/block/<dev>/size` (client baseline, double capture) and `blockdev --getsize64 /dev/<dev>` (root), same device, same sitting | both, separately labelled | Whether sysfs `size` × 512 equals the byte interface's answer **on the whole-device node** — the 2026-08-13 readback measured the 512-byte unit on the partition node alone and stated the whole-device gap rather than bridging it by convention | either read failing; the two captures unstable | `observed` — sysfs `size` = `244457472` (client, byte-stable), `blockdev --getsize64` = `125162225664` (root), and 244457472 × 512 = 125162225664 **exactly**: the 512-byte unit is measured on the whole-device node itself. The convention WP-L100 increment 3 was directed to accept is now a measured fact on this class; the acceptance decision still lands in that increment's record, citing this row instead of a convention |

Validity gates, all required: fresh VM and recorded environment
(`l-env.sh` lineage); `udevadm settle` plus double-capture byte
stability for every sysfs read; no mount of any measured object
(asserted before and after via `/proc/self/mounts`); the device
read-only at every capture; per-command exit statuses in-transcript;
custody identical to the matrices above — outside-repository archive
with locator and custodian, SHA-256 digest and byte length, instrument
digests recorded before first capture, guest/host/workstation digest
agreement. Gate failures make cells `void(<gate>)`, never negatives.

What this sitting deliberately does not do: no transport-discrimination
protocol row (issue #318's grant question is undecided and this
preregistration does not preempt it); no layout provisioning; no second
stick (FR cells are single-device claims; the pair's distinctness is
already recorded); no CID measurement (no native MMC controller exists
in this apparatus).

**The sitting, 2026-08-13 (UTC).** Disposable VM 9437 on the same
Proxmox host, fresh jammy image against the pinned digest, kernel
`5.15.0-186-generic`, the bus-port `2-3` SanDisk unit (the readback
rows' measured unit, serial `A20036CA8695D921`) passed through with
`usb3=1`; `muser1` created with no supplementary groups as the client
baseline; the device `blockdev --setro` before any capture and `ro=1`
at every read; no mount of the measured object before or after;
`udevadm settle` before captures; every cell byte-stable across the
double capture. **The first invocation was void** — the client
instrument was staged in `/root`, which the unprivileged user cannot
read, and `runuser` refused it before any client capture ran; the void
transcript is retained under the keep-revisions practice, the
instrument was restaged world-readable at `/usr/local/lib`, and the
cited run's transcript records the amended `fr-root.sh` digest. The
guest's snapd loop devices were present and recorded; this protocol
names no loop-administrator exclusion. Teardown verified
2026-08-13T15:57:45Z: no VM config, no storage volume, no LVM volume.

**Custody.** Transcript and the void first invocation archived at
`%USERPROFILE%\PartMan-evidence\2026-08-13-fr-vmid9437\` on the
operator workstation, custodian Nate McBride. Cited transcript SHA-256
`6173cc46f62671d63b0fdaf44a3f218aad03088d884a6f3d0f31d19ab1f340a6`
(4310 bytes), computed in the guest before the file moved, recomputed
on the Proxmox host, recomputed on the workstation — all three
agreeing. Instrument digests recorded in-transcript before any
capture.

## Reproducing this

The Windows facts above come from read-only CIM queries against
`root/Microsoft/Windows/Storage` plus one read-only `CreateFile` attempt on a
physical-drive path. No device layout, serial, or unique id from the measured
machine is recorded here; only whether each property was present and readable,
per SEC-006's redaction posture.
