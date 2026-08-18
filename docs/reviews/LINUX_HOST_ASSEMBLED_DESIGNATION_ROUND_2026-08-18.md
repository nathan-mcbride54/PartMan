# The Linux host-assembled designation round — what may name an aggregate, a volume, or a loop on Linux, on the DR rows

**Date:** 2026-08-18. **Base:** `1bd53da` (main), spec 17.3.0.
**Directive:** Nate — "draft the designation round".
**Question:** WP-L100 increment 4b — the topology half of LIN-006's
detection layer — needs a Linux naming source for each host-assembled
kind it would build. ADR-0034's table designates one cell (USB-attached
serial) and leaves every other Linux cell undesignated. The 2026-08-18
detection-rows sitting (DR1–DR10, `docs/quality/observability.md`)
measured candidate sources. Which of them, if any, may be designated
under the discipline ADR-0034 fixed and ADR-0035 followed?

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §4 is for the decision owner. The
> designation itself is a normative act landing only through an ADR and a
> spec change (ADR-0019; ADR-0034's "changing any cell is hash-visible by
> construction and lands only with a spec change").

## 0. The premise, and the discipline the round works under

Three texts fix what a designation may rest on:

- **ADR-0019** (`docs/adr/0019-si27-node-naming.md:98-116`, `:106-108`): "an identifier
  used in naming is the byte string returned by the **one named source** the
  evidence contract designates for it, per platform, verbatim — no case
  folding, no prefix stripping, no re-encoding". The per-kind maps
  (`:73-86`): Aggregate names from *technology, canonicalized native
  designator bytes* (`:81`) — "never from its members" (`naming.rs:261-262`;
  member-derived naming withdrawn at `:273-277`); Volume from *producer id,
  the technology's own volume name and role bytes — never a volume UUID,
  which is regenerable and excluded* (`:82`; the option rejected at
  `:285-290`); `BackingExtent` from *host file-system id plus canonicalized
  path bytes* (`:83`), and a loop device "names from its distinct backing
  file" (`:195`); and the designator-absent aggregate rule at `:159-161`.
- **ADR-0034** (`:59-79`, `:132-165`): a cell is designated only with
  "measured value, stability, and per-unit distinctness behind it";
  "nothing here designates by prediction". The udev database was rejected
  as a naming source on **three grounds** — a cached third-party
  computation (`Method::Heuristic`, `inferred`), a second source able to
  diverge from the direct read, and a measured worse availability class —
  and holding until every class is measured was rejected because
  undesignated cells are already fail-closed.
- **ADR-0035**: the precedent shape — a qualifying sitting measures a source
  for an undesignated class, the table extension lands with its rows, minor
  under §0.1.

And one fact the round first took as delivered bounds the cost of not
designating — **it is decided, not delivered** (§0.1): ADR-0019 decides
that "an aggregate whose native designator is unreadable derives a
designator-absent name, is `Indeterminate`, and is not a plan operand"
(`docs/adr/0019-…:159-161`), the type admits the absence
(`Aggregate.designator: Option<Vec<u8>>`, `naming.rs:263-269`, whose
doc-comment restates the rule), and a `Volume` cannot be built without a
name (`name: Vec<u8>` at `:281`, `require_bytes` at `:469`).

## 0.1 What the adversarial pass changed, kept rather than erased

An independent verifier attacked the round's load-bearing claims against
the sources before it was landed. What did not survive as first drafted:

| draft said | measured |
| --- | --- |
| "a designator-absent aggregate … is an `Indeterminate` non-operand under ADR-0018's closure" — cited as a delivered fact from `naming.rs:266-268`, and load-bearing for D1's "bounded cost" and D3's interim | **Refuted as delivered.** `crates/domain/src/model/protection.rs` contains no occurrence of `designator`; the aggregate own-arm at `:1187-1210` returns **`Verdict::Permitted`** for `Lvm2 \| Mdraid` regardless of designator (`:1206`). A *lone* designator-absent aggregate is a plan operand under the delivered closure; only two or more of one technology reach `Indeterminate`, through the collision group (`naming.rs:707-729`, `protection.rs:674-677`). No test constructs `designator: None` on a verdict path (`tests.rs:266` tests grouping only), and the adapter's `operand_eligible` flag has no consumer outside `crates/adapter-linux`. The doc-comment is decided text (ADR-0019 `:159-161`) restated over a closure that does not implement it — the "structural claims come from types" rule tripping. **Consequence:** the gap is a defect to close before any designator-absent aggregate enters a draft, and D3 is re-conditioned on it (§4). |
| Every example value "from the cited run (VMID 9468, `89ce59ac…`)" | The literal `MD_UUID`/`ID_FS_UUID` examples were copied from the **void first invocation's** transcript. The *form* claims hold in both runs; the values are now the cited run's (`27642121:0f8e15dd:…` beside `27642121-0f8e-15dd-…`; PV UUIDs `uWGkdS-J2OL-…`, `824JMN-fmkJ-…`). |
| §3.4 "a client-readable VG UUID" as an open question, and D4 as a new negative to record | Already decided and already measured: ADR-0019 `:245-246` ("LVM2 VG id and ZFS pool GUID helper-only") and the increment 6 Linux matrix's **L7** row (`observability.md:4080`: "LVM2 VG id: **`not-client-readable`** — the client carries only the PV UUID … root `pvs` only"). §3.4 and D4 are withdrawn; the round cites the row. L7 also records the mdraid array UUID as client-readable through udev `ID_FS_UUID`, and **L8** (`:4081`) measured it surviving a passthrough replug and a full VM reboot — cited now in the mdraid paragraph, where the stability half of the question was already answered before DR. |
| Btrfs "not an aggregate under decided text (ADR-0019 `:78`)" | The deciding text is ADR-C5 SI-08 Option B, `docs/adr/0005-…:203-208` ("a `FileSystem` with an ordered set of n ≥ 1 backings … No synthetic container node"); ADR-0019 `:79` carries only the naming invariance. Re-cited. |
| Line citations `:73-88` for the naming table | The table runs `:73-86`; the FS row is `:79`, Aggregate `:81`, Volume `:82`, `BackingExtent` `:83`. Corrected. |
| D3's "no count for LVM2 … absence is its answer" | True of the fact map (`member_counts: BTreeMap<NodeId, u64>`, `protection.rs:93-95`), and the closure consults it only on the APFS arm — but ADR-C5 lists "a platform stops self-reporting an aggregate's member count" as a **revisit trigger** (`0005-…:478-480`). Recorded beside D3 rather than treated as free. |

Added on the verifier's prompting: `observability.md`'s DR10 cell says
"`ID_FS_UUID` distinct per member", while the transcript shows the two
members of one md array carrying **equal** `ID_FS_UUID` (the array's UUID
re-spelled) — the cell is looser than its transcript, and a one-line
WP-035 correction is named in §7 so it is not read as a per-member
identifier.

## 1. What DR measured, per candidate source

Every value below is from the cited run (VMID 9468, transcript
`89ce59ac…`), client baseline, double-captured; **stability** means
byte-equal across DR10's full deactivate/re-assemble cycle
(`vgchange -an/-ay`, `cryptsetup close/open`, `mdadm --stop/--assemble
--scan`, unmount/remount); **distinctness** means distinct across the two
units of a kind the sitting provisioned.

| Candidate | Interface / method | Value | Stability | Distinctness | Cell |
| --- | --- | --- | --- | --- | --- |
| `dm/uuid` on a logical volume | sysfs, direct | `LVM-<vg-uuid><lv-uuid>` (64 hex chars after the prefix) | **equal** across re-assembly, though the minor moved (dm-0 ↔ dm-1) | distinct per LV | DR3, DR10 |
| `dm/uuid` on an opened LUKS container | sysfs, direct | `CRYPT-LUKS2-<uuid>-<name>` | equal | distinct per container | DR3, DR10 |
| `dm/name` | sysfs, direct | the mapper name (`vg_dr_a-lv_a`, `cr_a`) | **not measured across re-assembly** — DR10's per-unit capture read `dm/uuid` only | distinct per unit as provisioned | DR3 |
| `MD_UUID` in the array node's udev record | udev database, heuristic | `27642121:0f8e15dd:ff2155a8:c2414550` form (colon-quartet) | equal, while the record's other bytes changed | distinct per array | DR5, DR10 |
| `md/uuid` under the array's sysfs `md/` | sysfs, direct | **not measured** — DR5 read `level`, `raid_disks`, `array_state` | — | — | (none) |
| `ID_FS_UUID` on an md member | udev database, heuristic | the array UUID **re-spelled** (`27642121-0f8e-15dd-ff21-…`) — not the bytes `MD_UUID` carries | equal | equal across the two members of one array | DR6, DR10 |
| `ID_FS_UUID` on an LVM member | udev database, heuristic | the **PV** UUID (`uWGkdS-J2OL-…`, `824JMN-fmkJ-…`), not the VG's | equal | distinct per PV — including the two PVs of one VG | DR6, DR10 |
| a client-readable **VG UUID** | — | **no interface measured carries it**: `dm/uuid` embeds it inside an LV mapping's bytes, member records carry PV UUIDs — the increment 6 matrix's **L7** row already records the VG id as `not-client-readable` (root `pvs` only) | — | — | L7 (negative) |
| `loop/backing_file` | sysfs, direct | the attached path verbatim | equal (no re-attach was performed) | not measured across two loops on one file | DR7 |
| `/sys/fs/btrfs/<uuid>` and each member's `ID_FS_UUID` | sysfs / udev | one UUID per file system, equal on both members | equal | one file system provisioned | DR6, DR8 |
| `slaves/`, `holders/` | sysfs, direct | per **mapping**, not per aggregate — the LV's `slaves/` names one PV of a two-PV VG | — | — | DR4 |

Two DR findings bear on naming without being candidates: `slaves/` and
`holders/` do not report aggregate membership (DR4), so nothing here can
derive a VG's member set from sysfs edges — the cached member view (DR6)
is what names a member; and a whole-record digest is **not** a stable
identity across re-assembly (DR10), which is why any designation names a
key's value, never a record.

## 2. Each kind against the discipline

**Aggregate — mdraid.** The native designator is the array UUID. The one
measured source is `MD_UUID` in the array node's own udev record: value,
stability and distinctness all hold — and the increment 6 matrix had
already recorded the array UUID as client-readable through the cache
(**L7**) and surviving a passthrough replug and a full VM reboot (**L8**),
so the *stability* half was answered before DR asked it. But it is exactly the source class
ADR-0034 rejected: a cached third-party computation (`udevd` ran
`mdadm --detail --export` at event time; the delivered adapter classifies
the interface `Heuristic`, deriving `inferred`), and a second source
beside a direct one that may exist — sysfs `md/uuid` — which the sitting
**did not read**. Designating the cache while a direct source is
unmeasured would be designating by prediction in reverse: predicting the
direct source does not exist. Not designatable on these rows; one cell
short.

**Aggregate — LVM2.** The native designator is the VG UUID, and **no
client-readable interface reports it** — decided in ADR-0019 (`:245-246`,
"LVM2 VG id … helper-only") and measured in the increment 6 matrix's
**L7** row (`not-client-readable`, root `pvs` only), which DR6 confirms. `dm/uuid` on an LV
carries it as the first 32 characters after `LVM-` — but that source
names a mapping, exists only while an LV is active, and extracting the
prefix is the transformation class ADR-0019 excludes wholesale ("no prefix
stripping"). Member records carry PV UUIDs, which name members, and
member-derived aggregate naming is withdrawn. Not designatable, and not one cell short: **the record already says no
candidate exists**. The consequence is bounded by decided text — an LVM2
`Aggregate` with an absent designator is representable and is, per
ADR-0019 `:159-161`, indeterminate and not an operand — but that rule
is **not delivered by the closure** (§0.1), so the honest 4b shape for
LVM2 waits on the closure arm before any designator-absent aggregate
enters a draft.

**Volume — LV, opened container, and any dm target.** ADR-0019 wants
"the technology's own volume name and role bytes — never a volume UUID".
`dm/uuid` is a technology-assigned volume UUID and falls in the excluded
class by category; whether LVM can regenerate an LV UUID in place is not
measured and the exclusion does not turn on it. `dm/name` is the right
*kind* of source — the mapper name, sysfs, direct, the bytes the
technology itself chose — and its stability across re-assembly and reboot
is **not measured**: DR10 recorded `dm/uuid` per unit and the setup
actor's own name-based resolution, which is not a client capture. Not
designatable on these rows; one cell short. Role bytes: Linux dm defines
none; the field is `Option` and stays absent.

**Encryption layer.** Names from its backing signature's id, so no
designation is needed; what 4b needs is a `BackingSignature` node
(host, `Luks2`, primary offset), which is a 3b/4b construction question —
the cached view names the family (DR6, `crypto_LUKS`), and the primary
offset of a whole-device LUKS header is the parser's fact, not the
client's — recorded here as 4b's, not this round's.

**Loop.** The `BackingExtent` names from its host file-system id plus
the path bytes; `loop/backing_file` is the direct sysfs source (DR7),
verbatim, and ADR-0019 already decided the semantics — path bytes are the
identity, a re-attach from another path is a storage change. What is not
measured is per-unit distinctness (two loops on one file — a legal
configuration) and stability across detach/re-attach; and the host
file-system node does not exist until 3b lands. Designatable in kind;
one cell short on distinctness; and unusable until 3b regardless.

**Btrfs multi-device.** Not an aggregate under decided text — ADR-C5
SI-08 Option B, "a `FileSystem` with an ordered set of n ≥ 1 backings … No
synthetic container node" (`docs/adr/0005-…:203-208`) — naming from host
id, kind, superblock offset (ADR-0019 `:79`, "invariant under `btrfs
device add`"). Which
member is "the host" of a multi-device file system is a 3b/4b question
the record has not asked; `/sys/fs/btrfs/<uuid>/devices/` (DR8) answers
membership, not naming. Outside this round.

**Multipath.** Gated on WP-L100 obligation 3; no measurement, no
candidate.

## 3. What is genuinely open

1. **Whether a direct mdraid source exists.** sysfs `md/uuid` under the
   array's `md/` directory, on the guest kernel — one `cat`, double
   captured, across re-assembly, on both arrays. If it exists and holds,
   mdraid is designatable on the ADR-0034 shape; if it does not, the
   record faces the same three-grounds question ADR-0034 answered for
   serials, now for a kind whose only source is the cache.
2. **`dm/name` stability** across re-assembly and across a reboot, both
   arrays and both containers — one cell.
3. **Loop distinctness** — two loop devices attached to one file, and one
   loop detached and re-attached, `backing_file` on each — one cell.
4. **The closure's designator-absent arm** — not a measurement but a
   delivered-code gap (§0.1), filed as **gitea#1006** on WP-010: ADR-0019 `:159-161` decides that a
   designator-absent aggregate is `Indeterminate` and not an operand;
   `protection.rs:1206` permits it. A WP-010 fix — the aggregate own-arm
   returning `Indeterminate { MissingFact }` when `designator` is `None`
   for the technologies that name by designator, with a test that
   constructs `designator: None` and a mutation that flips it — closes
   the gap; until it lands, no client draft may carry a designator-absent
   aggregate, because the draft would propose an operand.

The three cells fit one short sitting on the DR apparatus (the `dr-*` scripts,
`PartMan-evidence/2026-08-18-dr-vmid9467-9468`, VMID 9470 next) — cells
DR11–DR14, preregistered before the guest exists, WP-035's second act on
a WP-L100 filing as before.

## 4. The recommendation

**Designate nothing on today's rows, and do not hold 4b for it.**
Concretely:

- **D1. No designation ADR from this round.** Every measured candidate
  fails one criterion of decided text: `MD_UUID` is the rejected source
  class while its direct sibling is unmeasured; `dm/uuid` is the excluded
  volume-UUID class and, for the VG, a prefix of a mapping's bytes;
  `dm/name` and `loop/backing_file` lack the stability or distinctness
  cell. Designating any of them now would be the "designation by
  prediction" ADR-0034 refuses, and the cost of not designating is
  bounded by the delivered types (an aggregate may be designator-absent
  and indeterminate; a volume simply is not built).
- **D2. File DR11–DR14 on WP-035** (§3), the same two-act bracket as
  #1005, and take them on the DR apparatus. If `md/uuid` holds, an
  ADR-0035-shaped extension designates **mdraid: sysfs `md/uuid`, bytes
  verbatim**; if `dm/name` holds, **volume name bytes: sysfs `dm/name`,
  verbatim, role absent**; if the two-loop cell holds, **loop backing
  path: sysfs `loop/backing_file`, verbatim**. LVM2 aggregates stay
  undesignated on the negative row unless the row surprises.
- **D3. Close the closure gap first, then let 4b start on what needs no
  designation.** The gap (§0.1, §3.4) is a WP-010 act on its own: the
  aggregate own-arm in `protection.rs` returns `Indeterminate` for a
  designator-absent aggregate of a designator-named technology, as ADR-0019
  `:159-161` decides and `naming.rs:266-268` already claims — a test that
  constructs `designator: None`, a mutation that flips the arm, no spec
  text (the sentence exists; the code catches up), Rust so it owes a
  sitting. **Only after it lands** may 4b's first slice build what needs no
  designation: member `BackingSignature`s from DR6's cached view where a
  host node exists (whole-device members only, 3a's `PhysicalDevice`s);
  mdraid and LVM2 `Aggregate`s with **absent designators** — then
  representable *and* indeterminate non-operands, the fail-closed posture
  decided for this case — carrying DR5's self-reported count
  (`md/raid_disks`, direct) for mdraid and no count for LVM2 (none is
  client-readable; ADR-C5 lists a platform ceasing to self-report as a
  revisit trigger, `0005-…:478-480`, recorded here rather than treated as
  free); no `Volume` and no loop `BackingExtent` until their cells and 3b.
  **Reversible in the right direction:** when a designation lands, the
  aggregate's name gains a field and strengthens; nothing built under D3
  has to be withdrawn.
- **D4. Withdrawn** — the LVM2 negative is already decided (ADR-0019
  `:245-246`) and measured (L7); WP-L100's 4b record cites the row rather
  than restating it.

**Pricing.** No spec text and no ADR from this round; two documentation
PRs (the WP-L100 filing, the WP-035 preregistration) and a sitting; the
closure arm is Rust under WP-010 (owes a sitting; no spec text, the
sentence is ADR-0019's) and 4b's first slice is Rust under WP-L100 (owes
a sitting) — one arc, one sitting at its head on the r11 precedent, the
domain act first. If
D2's rows land as hoped, one ADR on the ADR-0035 shape, minor under §0.1
(cells gain designations; no text narrows).

## 5. Open questions for the decision owner

1. **Is D3 wanted before the rows, or should 4b wait for D2's answer so
   aggregates land named once?** The round prefers D3 *with its closure
   arm landed first*: designator-absent aggregates are the decided
   fail-closed representation, they let the detection layer report what a
   host has, and a later designation only strengthens them — but the
   closure must refuse them before a draft may carry one.
2. **If `md/uuid` does not exist on the measured kernel, is `MD_UUID`
   acceptable as a designation with ADR-0034's three grounds recorded
   against it — or does mdraid stay designator-absent until a direct
   source appears?** The round leans to the latter; the cost is
   indeterminate mdraid aggregates on Linux, which is where they are today.
3. **Should the DR11–DR14 sitting add a ThinkPad leg** for `dm/name`
   stability across a *real* reboot (the VM protocol never reboots by
   design)? The round says no — a `qm reboot` of a disposable guest, done
   as a declared step after all other cells, measures the same kernel
   contract, and the "never reboot" rule exists for the WP-020 acceptance's
   kernel pin, not for this sitting.

## 6. What would change this round's mind

- A decided text this reading missed that admits a udev-database source
  for naming — none was found: ADR-0034's three grounds are the only text
  on the question and they refuse it.
- A reading of ADR-0019's "no prefix stripping" under which the first 32
  characters of an LV's `dm/uuid` are "the byte string returned by the
  named source" — the round reads that sentence as excluding exactly this,
  and the source as naming the mapping in any case.
- `dm/uuid` turning out *not* to be in the volume-UUID class ADR-0019
  excludes — but the exclusion is by category ("regenerable and excluded"),
  and moving it would be a change to that ADR, not a designation.

## 7. Next acts, in order

1. Decision-owner call on D1–D4 and §5.
2. WP-L100 filing of DR11–DR14 (`Work-Package: WP-L100`; Gitea issue) and
   the WP-035 preregistration, before the guest exists.
3. The sitting on VMID 9470; the record — and, in the same WP-035 act, the
   one-line DR10 correction (md members of one array carry *equal*
   `ID_FS_UUID`, the array's re-spelled; distinctness is per array).
4. If D3: the WP-010 closure arm (gitea#1006; its own PR), then
   WP-L100 increment 4b's first slice — member signatures on whole-device
   hosts, designator-absent aggregates with mdraid's self-reported count —
   one arc, r43 at its head, named in both PR bodies before the first
   merge.
5. If D2's rows hold: the designation ADR (ADR-0053-shaped on ADR-0035),
   then 4b's second slice naming what it may.
