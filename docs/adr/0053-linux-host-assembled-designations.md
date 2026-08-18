# ADR-0053: The Linux host-assembled naming designations

- Status: Accepted
- Date: 2026-08-18. Made on the adversarially reviewed recommendation
  round of the same day
  (`docs/reviews/LINUX_HOST_ASSEMBLED_DESIGNATION_ROUND_2026-08-18.md`,
  a committed session record; its D2 asked for the cells this ADR rests
  on and its D3 built what needed no designation meanwhile), under
  ADR-0034's revisit condition fired in its sanctioned direction: two
  qualifying sittings (DR1–DR10 and DR11–DR14, `docs/quality/observability.md`)
  measured naming sources for currently undesignated Linux kinds, and the
  table extension lands with its rows. Recorded before its first consumer
  is written — merging is not acceptance.
- Spec version: 17.4.0 (minor under §0.1 — previously undesignated cells
  gain designations; no existing text narrows; LIN-006, INV-004 and
  Section 5 stand verbatim)
- Work packages blocked: none (the first consumer is WP-L100 increment
  4b's second slice, which waits on this and on a further round for
  member signature nodes)
- Requirement IDs: Section 5, LIN-006, INV-004, MODEL-004, SAFE-005,
  ADR-C4, ADR-C5, ADR-0018, ADR-0019, ADR-0034, ADR-0035
- Decision owners: Nate McBride

## Context

ADR-0019 makes naming a matter of *one named source per identifier per
platform, verbatim* — "no case folding, no prefix stripping, no
re-encoding" — and gives each kind its naming map: an aggregate names
from *technology, canonicalized native designator bytes*, never from its
members; a volume from *producer id, the technology's own volume name
and role bytes — never a volume UUID*; a `BackingExtent` from *host
file-system id plus canonicalized path bytes*, and a loop device "names
from its distinct backing file". ADR-0034 fixed the discipline for
designating a source — measured value, stability, and per-unit
distinctness; a direct source over the udev cache, which it rejected on
three grounds; nothing designated by prediction — and left every Linux
cell but USB-attached serial undesignated. ADR-0035 extended the table
for the mmc class the day a sitting measured it.

Until 2026-08-18 no Linux source was designated for any host-assembled
kind, so WP-L100 increment 4b could name nothing: the Linux
host-assembled designation round measured every candidate the DR1–DR10
sitting had produced against the discipline and found each one cell
short — the udev cache's `MD_UUID` while sysfs `md/uuid` was unread;
`dm/name` without its stability cell; `loop/backing_file` without its
distinctness cell — and the LVM2 volume-group id already recorded
`not-client-readable` (the increment 6 matrix's L7 row; ADR-0019
`:245-246`). It recommended designating nothing on those rows, filing
the missing cells, and building what needed no designation once the
closure enforced ADR-0019's designator-absent rule (WP-010 slice 3q,
gitea#1006). The cells were filed (gitea#1007), preregistered, and taken
the same day on the DR apparatus with one declared reboot leg:

- **DR11** — sysfs `md/uuid` **exists** under each array's `md/` on the
  measured kernel and is client-readable: hyphenated form where the udev
  record's `MD_UUID` is colon-quartet (the same 128 bits, two spellings —
  the divergence one named source exists to exclude); byte-equal across
  `mdadm --stop`/`--assemble` and across a reboot; distinct per array.
- **DR12** — `dm/name`, keyed by `dm/uuid`, is byte-equal for LVM logical
  volumes across `vgchange -an/-ay` and across a reboot with automatic
  activation. For dm-crypt mappings the sitting measured that the name is
  **the opener's argument, not a stored property**: its own mis-addressed
  post-reboot re-open put container A under the name `cr_b`, and
  `dm/name` followed the opener.
- **DR13** — two loop devices attached to one file each report the same
  `backing_file` bytes; a loop detached and re-attached from the same
  path reports the path verbatim again.
- **DR14** — the member-signature family is client-readable
  (`ID_FS_VERSION`, `md/metadata_version`), and **no interface reports a
  signature's offset**.
- The reboot **renumbered the disks** (`sdh`/`sdi` swapped roles): kernel
  entry names carry no identity across boots.

## The extension

ADR-0034's designation table gains the following cells, keyed by
(platform, kind, technology) — the kinds ADR-0019's per-kind maps name
from a platform source:

| Platform | Kind | Technology | Naming input | Designated source |
| --- | --- | --- | --- | --- |
| Linux | Aggregate | mdraid | designator | The `md/uuid` attribute under the array's block-class node, bytes verbatim as read — trailing newline included |
| Linux | Aggregate | LVM2 | designator | **Undesignated** — no client-readable interface reports the volume-group id (L7; ADR-0019 `:245-246`); the helper names it from the metadata it can read |
| Linux | Volume | LVM2 logical volume | name | The `dm/name` attribute under the volume's block-class node, bytes verbatim — trailing newline included |
| Linux | Volume | LVM2 logical volume | role | **Undesignated** — device-mapper defines none; the field stays absent |
| Linux | Volume | dm-crypt mapping | name | **Undesignated** — the mapping name is the opener's argument (DR12), not the technology's own; the LUKS2 header's label field is unmeasured |
| Linux | `BackingExtent` | loop device | path | The `loop/backing_file` attribute under the loop's block-class node, bytes verbatim — trailing newline included; the host is the file-system node the path lives in |

**The kind is decided before the source is read.** A block node is
host-assembled by DR3's markers (`md/`, `dm/`, `loop/`; WP-L100 increment
4a's withdrawal), and which dm target it is by `dm/uuid`'s prefix — read
as a *classification* input, never as a name (`LVM-` and `CRYPT-LUKS2-`
were the two DR3 measured; any other prefix is an unrecognized target and
names nothing). The entry name (`md127`, `dm-0`, `loop6`) is a
session-local locator and never a naming input: DR2 measured it
renumbering across one reboot.

**Everything ADR-0034 established applies unchanged**: verbatim includes
the trailing newline; a measured absence (`ObservedAbsent`) of the
designated source yields the designator-absent, indeterminate,
non-operand aggregate ADR-0019 decides and slice 3q enforces (for a
volume, whose name is required, absence means no node); a failed read
yields the same indeterminate non-operand standing; naming inputs flow
through the bytes-preserving path, never through the text-decoding read.

**Two designations that are consequences, not sources.** An LVM logical
volume's *producer* is its volume group's aggregate, which on Linux is
designator-absent by the row above — so every LVM volume group is one
designator-absent `Aggregate`, two or more of them collapse into ADR-0019's
collision group, and each logical volume names under the group's shared
address by its own `dm/name`, all of them indeterminate non-operands
through producer inheritance. That is the decided fail-closed
representation of "an ordinary client sees LVs and PVs but no VG
identity", and it is what the helper's own re-discovery repairs at
HLP-002. A loop device's `BackingExtent` needs its host file-system node,
which the Linux client draft cannot build until WP-L100 3b lands; the
cell is designated now so the source is fixed before the node exists.

## Options considered

### udev `MD_UUID` for the mdraid designator

Rejected on ADR-0034's three grounds, and on a fourth this record adds:
it is a cached third-party computation (`Method::Heuristic`, `inferred`);
it is a second source beside the direct one, and DR11 measured the two
**spelling the same bits differently** — precisely the divergence the
one-source rule exists to make structurally absent; its availability
class is the cache's; and the direct source exists, so choosing the cache
would be designating by prediction in reverse.

### `dm/uuid` for the LVM2 designator or the volume name

Rejected: for the volume group it carries the VG id only as a prefix of
an LV mapping's bytes, exists only while an LV is active, and extracting
it is the transformation ADR-0019 excludes wholesale; for the volume it
is a technology-assigned volume UUID, the class ADR-0019 excludes by
category (`:82`, `:285-290`), whether or not LVM can regenerate one in
place.

### Member `ID_FS_UUID` for the mdraid designator

Rejected: member-derived aggregate naming is withdrawn (ADR-0019
`:273-277`), and DR6/DR10 measured it as a third spelling of the same
bits on every member — one more source, one more divergence.

### The mapper name for dm-crypt

Rejected on DR12's measurement: the name is whatever `cryptsetup open`
was told, so two hosts opening one container name it differently and one
host names it differently across two opens. A stored name may exist in
the LUKS2 header's label field; it is unmeasured, and this ADR
designates nothing by prediction.

### Kernel entry names as any naming input

Rejected: DR2 measured `sdh`/`sdi` swapping roles across one reboot on
one guest. Entry names are locators.

### Holding until member signatures can be built

Rejected: the cells above are measured on ADR-0034's three criteria and
withholding them buys no safety — undesignated and designator-absent
kinds are already fail-closed — while the signature question is a
different question (DR14: no interface reports an offset, so a client
signature node would author one) and gets its own round.

## Decision

The cells above are normative, versioned with the evidence contract
exactly as ADR-0034's and ADR-0035's; changing any is hash-visible by
construction and lands only with a spec change. LVM2 volume groups and
dm-crypt mapping names stay undesignated on Linux. No member signature
node is designated or authorized here.

## Consequences

- **Positive:** WP-L100 increment 4b's second slice can name an mdraid
  array from its own sysfs designator, an LVM logical volume from its own
  mapper name under its group's aggregate, and a loop device's backing
  extent from the kernel's own path — each from a direct, measured,
  reboot-stable source. The designator-absent mdraid aggregates increment
  4b's first slice builds gain their field and their group dissolves;
  nothing built there is withdrawn.
- **Negative, accepted knowingly:** every LVM volume group on a Linux
  client draft is a designator-absent aggregate, so two or more group and
  every logical volume is an indeterminate non-operand until the helper
  names the group; two loop devices on one file derive one address and
  group; the loop cell is unusable until 3b's file-system node exists;
  and no client draft may carry a member `BackingSignature`, so aggregate
  membership stays a reported listing (DR4, per-mapping) rather than an
  edge until the offset question is decided.
- **Evidence obligations:** (1) the LUKS2 label field — whether it is
  client-readable and stable, the only candidate for a dm-crypt name;
  (2) `md/uuid` on a second kernel line (the record measured
  5.15.0-186 only); (3) a real-hardware capture of the same cells, when
  a sitting next has one — the DR rows are kernel-interface claims on a
  VM, on the floor-rows precedent; (4) the member-signature offset
  question, filed as the next round's premise rather than answered here.

## Verification

- When increment 4b's second slice lands: an mdraid `Aggregate`'s
  designator bytes equal the array's `md/uuid` file content verbatim; an
  LVM `Volume`'s name bytes equal `dm/name` verbatim and its producer is
  its group's designator-absent aggregate; a loop `BackingExtent`'s path
  bytes equal `loop/backing_file` verbatim; no naming input flows through
  the text-decoding read path; `MD_UUID`, `dm/uuid` (beyond
  classification), and member `ID_FS_UUID` are not read for naming; a
  failed read of a designated source yields an indeterminate non-operand;
  a dm-crypt mapping yields no `Volume`. Each with a fixture, each with a
  mutation killed.
- The kind is decided by markers, never by entry name: a fixture whose
  entry names lie (an `md`-marked node called `sdb`) names by its marker.

## Revisit conditions

- The LUKS2 label row lands — the dm-crypt name cell is re-examined on it.
- A qualifying row finds a client-readable LVM2 volume-group id — the LVM2
  designator cell is designated on the ADR-0035 shape.
- `md/uuid`, `dm/name`, or `loop/backing_file` changes shape or
  termination on a measured kernel — hash-visible by construction, and
  the cell deserves re-examination rather than a shim.
- The member-signature round decides how a client draft carries a
  signature it cannot offset — which changes what an aggregate's members
  are in the draft, not how the aggregate is named.
