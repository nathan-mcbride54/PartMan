# The detection-layer arc — WP-L100 increment 4, 2026-08-18

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Written before the first line of code, per house convention;
> its base is `35149b0` (main), spec 17.3.0, both forges at that SHA.

**Directive:** Nate — "start the increment-4 arc, plan doc first".
**What it delivers:** LIN-006's detection layer on Linux as WP-L100's
increment 4 states it after the #1004 decision (`WP-L100.md:580-595`) —
device mapper, loop devices, Btrfs multi-device, LVM/mdraid/LUKS members,
active root/boot/swap dependencies, and **mounts as state-layer facts**,
with the multipath arm still gated.

## 0. What is already decided, so this plan decides nothing

- **The scope sentence** (Governance PR #452, on the #1004 round): mounts
  are detected in increment 4 "as state-layer facts and never topology
  nodes … reported through the adapter's observation surface, keyed to the
  mounted node's ADR-0019 address, and consumed by WP-050 once the Section 5
  `Mount` type exists". The §5 `Mount` type is WP-010's and **arrives with
  its first consumer** (`WP-010.md:2037`; #1004's closing comment, residue
  1). This adapter is a producer, so **no WP-010 slice rides this arc**.
- **The evidence rule** (`WP-L100.md:210-224`): every representational
  claim about what a real Linux host exposes rests on a capture recorded
  from a real host; where no recording exists, "the increment says so and
  delivers the fail-closed answer rather than authoring a fixture from
  specification text". ADR-0011 states the same for multipath in terms
  (`WP-L100.md:132-136`, obligation 3).
- **The interfaces are closed at two** — sysfs block-class attributes and
  the udev database — because those are the two `observability.md`
  establishes as client-readable on real hardware
  (`crates/adapter-linux/src/lib.rs:14-19`; `observation.rs:26-39`).
  A third interface enters the contract the way the first two did: by a
  row.
- **Naming** (ADR-0019, ADR-0034): a body node names from the contract's
  one designated source per platform; the Linux table designates exactly
  one cell (USB-attached serial) and every other cell is **undesignated**
  (`docs/adr/0034-…:66-71`); an aggregate whose native designator is
  unreadable is indeterminate and not an operand (`:108-112`); "nothing
  here designates by prediction" (`:73-77`). Loop and plain-dm devices
  name through the host-backing edge from a `BackingExtent` on their host
  file system (`docs/adr/0019-…:176-197`).
- **ADR-C5, obligation 9:** an `Aggregate` carries its self-reported
  member count, never a count of members observed (`WP-L100.md:207-208`).
- **The state layer** (ADR-0018 `:359-366`; ADR-0005 Rule 2): mount path
  and active swap feed Regimes B and C, never the verdict, and are not
  body content; a mount/unmount cycle must leave the body hash equal
  (ADR-0005's own evidence obligation).
- **WP-035 increment 6** may preregister and take read-only, primarily
  non-elevated measurements on "a disposable, non-WSL distro Linux VM"
  (`WP-035.md:415-430`); the floor-rows sitting on VM 9437 is the
  precedent for a preregistered VM sitting whose cells become rows
  (`observability.md:4276-4346`).
- **WP-020's stopping condition:** any Rust in the arc owes a sitting at
  the arc's head, named in the PR bodies before the first merge.

## 1. Measured at `35149b0`

| fact | where |
| --- | --- |
| The Linux observability table has **no row** for `/proc/self/mountinfo`, `/proc/swaps`, `/sys/class/block/*/{slaves,holders}`, `dm-*/dm/{name,uuid}`, `md*/md/*`, `loop*/loop/backing_file`, or `/sys/fs/btrfs/*`; the only mount mentions are the validity gate "no mount of any measured object (asserted via `/proc/self/mounts`)" and the only `backing_file` mention is the SI-35 loop record's "by-name evidence only" | `observability.md:2684-2698`, `:4091`, `:4306`, `:3048-3049`; `grep -n mountinfo\|/proc/swaps\|slaves\|holders\|dm/name\|dm/uuid\|backing_file\|md/level` → nothing else |
| The adapter's own exclusion list says so: "no kernel partition list, **no mount table, no swap table**, no firmware directory, no symlink farm"; boot/system role's Linux route "runs through mounts, swap and firmware state, which is increment 4's detection layer. **No row measures it on Linux at all**" | `schemas/adapter-linux/fields.md:158-167` |
| `enumerate` admits every block-class entry that positively lacks `partition` — which on any systemd host includes `loop*`, `dm-*`, `md*` — and `name_device` names each admitted entry `NamingFields::PhysicalDevice { serial, wwn: None, total_bytes }`; a non-USB entry is `Undesignated`, which is **operand-eligible** with a weaker name. So at HEAD a snapd loop device is an operand-eligible physical device, and two equal-sized ones collide into a group | `devices.rs:182-188`; `naming.rs:280-315`, `:117-128`, `:140-147` |
| No `NamingFields` kind for a host-assembled device has a designated Linux source: `Aggregate { technology, designator }`, `Volume { producer, name, role }`, `EncryptionLayer { backing_signature }`, `BackingExtent { host, locator }`, `MultipathNode { lun_designator }` all name from sources ADR-0034's table leaves undesignated | `crates/domain/src/model/naming.rs:263-297`; ADR-0034 `:67-71` |
| Every kind but the loop/dm device itself hangs on a node this adapter cannot build until 3b: `EncryptionLayer` needs a `BackingSignature` on a host, `BackingExtent` needs a host `FileSystem`, member signatures need their partition or device host — whole-device hosts exist (3a), partition hosts do not (3b, blocked on the table-role route) | `WP-L100.md:554-579`, `:621` |
| The multipath arm's fixture is gated on WP-035 rows that do not exist (obligations 2 and 3), and the adapter answers that population with increment 2's fail-closed classification | `WP-L100.md:124-136`, `:228-235` |
| The INV-003 reach declaration is a statement about table states and names the two interfaces in its contract statement; a third interface changes the statement's `detail`, not any cell | `schemas/adapter-linux/reach.md:1-60`; `reach.rs:88-99` |
| ADR-0005 Rule 2's evidence obligation is unwritten for all three arms; the mount arm lands with the `Mount` type, which is not this arc's | `WP-010.md:2037`; #1004 round §3.3 |

## 2. The finding: the arc's first act is evidence, not code

Increment 4 as scoped makes representational claims on **every** arm —
what a mount table line carries, what marks a block node as dm, md, or
loop, where a member's technology and the array's self-reported count are
read, what a Btrfs multi-device file system exposes — and **no row
supports any of them**. Authoring the fixtures from `proc(5)` and the
kernel's sysfs ABI documents is exactly the failure mode the evidence rule
names, and the increment-2/3a precedent is that the rows come first
(#318 → R1–R4, FR1–FR5) and the code cites them. So the arc opens with a
preregistered sitting, not a Rust PR.

There is a second, independent gap on the topology half only. Even with
rows, a host-assembled device becomes a **body node** only through a
designated naming source, and the Linux table has none for aggregates,
volumes, or produced virtual devices. ADR-0034's discipline is
measurement — value, stability, per-unit distinctness — before
designation (ADR-0035 is the precedent: rows first, then the table
extension). So the topology half needs a designation round after the
rows, and until it lands the adapter **detects and reports** host-assembled
devices without naming them as nodes.

The state-layer half has no such dependency: a mount and an active swap
are envelope facts by decided text, carry no name, and key to whatever
the adapter can already address (a whole device, 3a) or to the kernel's
own source identity where it cannot.

**One consequence for HEAD, stated so it is not read as hidden:** until
increment 4 lands, a loop, dm, or md block node is an operand-eligible
`PhysicalDevice` in this adapter's client draft. The exposure is bounded —
the draft is a proposal the helper re-discovers under HLP-002/ADR-0014,
and no apply path exists — but it is a misclassification of kind, and the
first Rust act below withdraws it in the fail-closed direction.

## 3. The shape

**3.1 The DR sitting (WP-035, `observability.md`).** One disposable Proxmox
guest (fresh jammy against the pinned digest, VMID 9467 next), no fixture
media — the claims are about kernel interfaces, not a medium — with root
as the setup actor and `muser1` as the client baseline. Root provisions,
on attached virtual disks and files: an LVM VG over two members with two
LVs; an mdraid RAID1 over two members; a LUKS container over a whole
virtual disk, opened; a Btrfs file system over two members; a loop device
from a file (plain `losetup`, the #94 by-name caveat recorded beside the
row, since the row's claim is what a client sees and not what the device
is bound to); one ext4 file system mounted, then unmounted; one swap
enabled. Cells, each client-baseline, double-captured for byte stability,
per-command exit statuses in-transcript, ADR-C4's three answers kept
apart:

| # | Cell | What it establishes |
| --- | --- | --- |
| DR1 | `/proc/self/mountinfo` readability, one line per mount, field shape (mount id, parent, `major:minor`, root, mount point, options, optional fields, `-`, fs type, source, super options) | The state-layer interface for INV-004/INV-005 mounts, and its keying field |
| DR2 | `/proc/swaps` readability and shape | ADR-0018's active-swap state-layer source |
| DR3 | Kind markers: which of `dm/`, `md/`, `loop/` exist under a dm, md, loop, and plain whole device; `dm/name`, `dm/uuid` (LVM `LVM-…`, crypt `CRYPT-…` prefixes) values | What positively marks a host-assembled block node, and what the client can read of its technology |
| DR4 | `slaves/` and `holders/` listings on members and on the assembled device, both directions | The kernel-reported membership relation, and whether it is symmetric as read |
| DR5 | `md/level`, `md/raid_disks`, `md/array_state`, and the udev record's `MD_LEVEL`, `MD_DEVICES`, `MD_UUID` | The self-reported member count (obligation 9) and its interface |
| DR6 | udev `ID_FS_TYPE`, `ID_FS_USAGE`, `ID_FS_UUID` on an LVM member, an md member, the LUKS device, each Btrfs member, and the ext4 device | Whether the cached signature view names the member technology, and whether Btrfs members share one UUID |
| DR7 | `loop/backing_file`, `loop/offset`, `loop/autoclear` on the file-backed loop and on a snapd loop | The host-backing input, and its by-name standing |
| DR8 | `/sys/fs/btrfs/<uuid>/devices/` listing | The multi-device layout as one file system over n backings |
| DR9 | Byte stability of every sysfs attribute and udev record of the ext4 device across mount → unmount → mount | ADR-0005 Rule 2's mount arm, at the input layer |
| DR10 | Per-unit distinctness: two VGs, two arrays, two LUKS containers — are `dm/uuid`, `MD_UUID`, `ID_FS_UUID` distinct per unit and stable across a re-assembly | The designation round's inputs, on ADR-0034's three criteria |

What the sitting deliberately does not do: no multipath (obligation 3
stays gated; two paths to one LUN are not this apparatus); no partition
hosts (3b's block stands); no root/boot on the measured objects (the
guest's own root and boot mounts are captured as they are, DR1); no
transport-discrimination protocol.

**3.2 The filing.** As #318 was: WP-L100 records "Filed by increment 4 —
the Linux rows this increment waits on" (`Work-Package: WP-L100`), the
Gitea issue names the ten cells, and WP-035 takes them as one
preregistration (`Work-Package: WP-035`) before the guest is created.
The record PR after the sitting carries the rows and the custody.

**3.3 The designation round (WP-000; a Governance PR only if a new ADR
path is minted).** On DR3/DR5/DR6/DR10: which source, if any, names a
Linux `Aggregate` (LVM VG, md array, Btrfs), a `Volume` (LV, dm target),
and a loop/plain-dm device — or the finding that none qualifies and the
kinds stay indeterminate on Linux. Its own round, adversarially passed,
then an ADR on the ADR-0035 shape (a table extension; minor if any cell
gains a designation, no text narrows). **The topology half of the code
waits on this; the state-layer half does not.**

**3.4 Increment 4a — the state layer and the withdrawal (Rust; owes a
sitting).** In `crates/adapter-linux`:
- `Interface::Procfs` (`linux-procfs`, `Method::Direct`), read through the
  existing bounded seam (`read_record` for the two tables); the reach
  document's contract statement gains the interface in its `detail`, no
  cell moves.
- `mounts` module: `/proc/self/mountinfo` parsed as DR1 recorded it into
  attributed observations keyed by the source's `major:minor` — resolved
  to the adapter's own device address where the source is an admitted
  whole device, and left on the kernel key where it is not (a partition or
  dm source, until 3b and 3.3) — never a topology node, never body
  content, and refusing rather than guessing on a line whose field count
  is not the recorded shape.
- `swap` module: `/proc/swaps` likewise (DR2).
- The withdrawal: an admitted block node carrying a DR3 kind marker is
  reported as **host-assembled** with its marker and its `slaves/`
  listing (DR4) as observations, and is **not** named `PhysicalDevice` —
  it stays out of the operand set (indeterminate) until 3.3 gives it a
  kind and a name. Fail-closed: an unreadable marker withdraws, a
  positively absent marker admits, exactly the `partition` discipline.
- Tests over authored trees carrying the recorded values, citing the
  rows, one per arm; mutations proven applied and killed: the field-count
  refusal dropped, the kind-marker admission flipped to admit on failure,
  the address resolution keyed by name instead of `major:minor`, the
  interface method set `Heuristic`.
- `fields.md` §4 loses "no mount table, no swap table" and gains the
  procfs rows; `WP-L100.md` increment 4 splits into 4a/4b with the
  delivery table updated; README row; CHANGELOG; generated traceability.

**3.5 Increment 4b — the host-assembled topology (Rust; owes a sitting;
after 3.3).** Node kinds for what 3.3 designates, hosted on whole devices
only: member `BackingSignature`s from DR6's cached view (heuristic, single-
valued — the ADR-C3 finding stands), `Aggregate` with DR5's self-reported
count, `Volume`s produced by it, `EncryptionLayer` over a whole-device
LUKS signature, `BackingExtent` + host-backing edge for the file-backed
loop **only where the host file system is a node this adapter has** —
which today is none, so the loop arm reports and does not name until 3b.
Partition-hosted arms and the multipath arm stay gated, each answered
fail-closed and said so.

## 4. Sequencing

1. This plan (WP-000).
2. The filing (WP-L100) and the DR preregistration (WP-035), one PR each,
   before the guest exists.
3. The DR sitting on VMID 9467; the record PR (WP-035) with rows, custody,
   teardown; captures archived beside the r-series evidence.
4. Increment 4a (WP-L100), its PR body naming **r42 at its head**; sitting;
   re-pin.
5. The designation round and its ADR (WP-000), decision-owner call.
6. Increment 4b (WP-L100), **r43** at its head; sitting; re-pin.

Steps 4 and 5 are independent and may interleave; 6 waits on both.

## 5. Pricing

No spec text moves anywhere in the arc: INV-004, INV-005, LIN-006 stand
verbatim, and the mount decision was priced at #1004 (none). The rows are
WP-035 record. The designation ADR, if a cell qualifies, is minor on the
ADR-0035 precedent; if none qualifies, it is a recorded finding and no
version moves. Two Rust increments, two sittings.

## 6. What would change this plan

- A row already recorded somewhere this reading missed, establishing any
  DR cell — then that cell drops from the sitting; the search above was
  `observability.md`, `docs/adr`, `docs/work-packages`, and the WP-020
  record, and found only the incidental mentions cited.
- The DR sitting finding a marker or table unreadable to the client
  baseline on jammy — then that arm is delivered as the fail-closed
  `unavailable` answer with the row saying so, and nothing is authored
  around it.
- A decision that host-assembled devices should be admitted as operands
  under a weak name rather than withdrawn — the plan reads ADR-0034's
  "not a plan operand" posture for the unreadable-designator aggregate as
  the nearer analogue and withdraws; the decision owner may read it the
  other way, and the mutation battery is written so either reading is a
  one-line flip with a failing test.
- 3b unblocking (a table-role route) before 4b — then 4b's partition-
  hosted arms open in the same increment rather than a later one.

## 7. Open for the decision owner

1. **Run the DR sitting on the VM alone, or add a ThinkPad leg?** The
   cells are kernel-interface claims; a VM is a real host for them, on
   the FR precedent. A ThinkPad leg would add nothing but a second kernel
   (6.12) — worth it only if jammy's 5.15 is a shape this arc should not
   pin to.
2. **Designation round now or after 4a?** They are independent; running
   the round while 4a is in flight shortens the arc by one sitting's
   worth of calendar, at the cost of a decision taken on rows less than a
   day old.
3. **Is the withdrawal in 4a acceptable as an interim** — a loop or dm
   device that is today an operand-eligible physical device becomes a
   reported, indeterminate non-operand until 3.3 names it — or should 4a
   wait for the round so the classification lands once? The plan
   prefers the interim: it is the fail-closed direction and it is what a
   sitting can measure.
