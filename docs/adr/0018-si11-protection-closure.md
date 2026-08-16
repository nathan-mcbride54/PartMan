# ADR-0018: The protection closure — computed, total, and fail-closed

- Status: Accepted
- Date: 2026-08-09. The round and its resolution chain were accepted
  together by Nate McBride the same day
  (`docs/reviews/SI-11_ROUND_2026-08-09.md`, an untracked session
  artifact; everything load-bearing is restated here). This is round
  four of the register's longest-running direct blocker; rounds one
  through three are recorded in Parts 4 through 6 of
  `docs/spec-issues/README.md`, and this design is built from their
  recorded objections.
- Spec version: 11.0.0 (major under §0.1 — the Storage Spaces boundary
  narrows what Section 2.1/WIN-003's "protect" claims over space
  contents, and Section 2.1's enforcement paragraph is retexted)
- Work packages blocked: WP-010 increment 3 (SI-11, SI-29, SI-30
  resolved; SI-37 reclassified open off the gate; SI-27, SI-28
  unchanged)
- Requirement IDs: Section 2.1, SAFE-005, PART-003, PART-014, CAP-001,
  CAP-002, CAP-003, CAP-005, CAP-007, HLP-002, MODEL-005, MODEL-002,
  PLAN-004, PLAN-006, CONC-001, CONC-005, WIN-003, WIN-004, MAC-009,
  MAC-010, FS-004, FS-005, FS-010, DIA-004, REC-001, PART-013,
  Section 0.2, Section 11.4
- Decision owners: Nate McBride

## Context

SI-11 filed the question round one could not yet see the bottom of: is
non-goal protection a type-level impossibility or a runtime guard, and
— once ADR-0012 fixed the axis at unrepresentability — what closure
decides *which* nodes Section 2.1 reaches. Three rounds failed on
recorded defects: round one on a PART-014/MAC-009 status conflict,
round two on sibling capture through containment, round three on six
defects, two of which destroy data with both defence layers passing —
the missing downward production rule (deleting a partition reaches the
encryption layer on it and never sees the pool below) and a residual
arm that defaulted to permitted.

Three foundations were fixed after round three, and this design stands
on them rather than re-arguing them. ADR-0012: a mutating step whose
target resolves to a Section 2.1 non-goal node is unrepresentable at
construction, with the helper's independent recomputation retained.
ADR-0014: the helper authors what only it can observe, from its own
named instrument, never from "run a privileged tool." ADR-0016: the
derived protection verdict is hashed-body content, helper-authored at
validation, and it must bind to a **named, deterministic helper
evidence contract with measured re-probe stability** — an unnamed
evidence set is round two's refuted universal premise returned.

Part 6's governing finding disciplines every mechanism below:
**fail-closed-by-unencodability is not fail-closed.** Every refusal in
this design is a typed, encodable artifact; no collision, residual, or
refusal surfaces as an encoder error.

## Safety analysis

### The verdict: three-valued, total, inverted default

Per node: `Permitted` | `Refused { class, reason }` |
`Indeterminate { cause, remediation }`. The function is total by an
enumerated match whose **residual arm is `Indeterminate`, never
`Permitted`** — round three's fail-open residual inverted. Enumerated
non-`Permitted` arms exist for: an indeterminate partition table
(ADR-0014's parser output), an unrecognized file system or signature
(SAFE-005's first clause honored, not wildcarded), a locked encryption
layer, an orphan or released signature, an unrecognized or non-local
transport, and every `Unrecognized` enum variant ADR-C5 mandates. A
property test generates nodes across every kind, including
unrecognized variants, and asserts the residual is never `Permitted`.

Placement is ADR-0016's, not re-argued: the verdict is body content,
helper-authored at validation, recomputed at revalidation and before
first write. Verdict inputs are restricted to facts satisfying
MODEL-005's body-stability rule; mount state, active swap, health, and
tool availability feed Regimes B and C — reasons and runtime gates —
never the verdict.

### The evidence contract, named

The verdict binds to a two-layer contract, per platform.

**The byte layer: the helper's own bounded parsers over raw device
bytes** — ADR-0014's architecture generalized from the table to every
on-disk verdict input. `crates/table-parser` exists; the
signature-family parsers join it: ZFS vdev labels (all four positions,
both end pairs, including the label's own recorded state field), mdraid
0.90 and 1.x superblocks, LUKS1/2 headers, LVM2 label and metadata
area, Storage Spaces and LDM markers, APFS container superblock
(self-reported member count and UUID), BitLocker metadata, and the
file-system superblocks FS-004 lists. Each parser is bounded,
`unsafe`-free, and Section 11.4 fuzz-obligated, fixed by family,
offsets, magic, and checksum validation. **The layer is enumerating by
construction: every family is probed at every defined location and
every validated match is reported.** Nothing is priority-resolved, so
the measured intra-helper asymmetry — `wipefs` enumerates both
signatures of the stale pair where root `blkid -p` reports exactly the
stale one (observability rows L5, L10) — is dissolved the way ADR-0014
dissolved the table-state one: there is no precedence question because
nothing is discarded.

**The state layer: named platform APIs for facts that are not on
disk** — transport/bus class, multipath assembly and membership, NVMe
subsystem and shared-namespace capability, device read-only state,
removability. Named per platform: Linux — sysfs attributes and the
device-mapper/holders topology; Windows — Storage Management API
classes (`MSFT_Disk`, `MSFT_PhysicalDisk.BusType`, the pool-membership
class measured readable non-elevated); macOS — named IOKit registry
keys, never `system_profiler` output (Section 16). Each fact carries
ADR-C4's outcome vocabulary; an `unavailable` or `failed` state-layer
fact makes the consuming arm `Indeterminate`.

**The join rule — the only cross-layer precedence, stated once: the
join is protective.** Evidence of a protected technology from either
layer suffices for its refusing arm; `Permitted` requires every arm
input positively determined or positively absent in its layer; a
positive disagreement between layers is `Indeterminate`. The join is
monotone over a finite input set, so the contract is deterministic
given its inputs.

**Stability, at honest scope.** The byte layer is a pure function of
device bytes; its re-probe stability is byte stability, measured where
measured (digest-bracketed double captures L3/L5/L10, M10's
double-read). The state layer's stability is partially measured; the
remainder is a named evidence obligation below, owed before any arm
consuming an unmeasured state-layer fact leaves `Indeterminate`. A
verdict change on a true state change is not instability — it is
ADR-0016's divergence rule rejecting the plan, as designed.

### The effect table and the affected-set closure

Round three's open parameter — which effect classes count as substrate
destruction — closes by enumeration. Every CAP-002 mutating operation
maps normatively to three range sets over host-qualified extents:
*written table extents* (the exact primary/backup GPT, MBR sector, and
EBR extents of the host's table node — never the parent device),
*consumed free ranges* (verified by the constructor to intersect no
existing node's extent — Section 11.2's overlap invariant enforced at
construction), and *destroyed ranges*, where **release is
destruction**: a range freed from its owner — a deleted partition's
extent, a shrink's truncated tail, a move's source extent at commit —
is destroyed even though no byte is overwritten, because its content
ceases to be referenced. Relocation classes (move, copy-then-commit)
MUST either preserve hosted signatures byte-wise or enumerate their
loss explicitly in the plan.

*(Retired in 13.0.1 by ADR-0040, resolving issue #348: this paragraph
also exempted "the relocated target's own subtree from destruction
descent — its content is preserved by contract (PART-005)". That
clause was **void where it stood** under §0.2's rule 4 — an ADR MUST
NOT weaken a spec MUST, and §2.1's enforcement paragraph is a MUST NOT
— and it was never delivered, never cited by any requirement, and not
expressible in the delivered closure, which takes no `Operation`. The
byte-wise-preservation duty above is the half that survives; it is
delivered nowhere and is tracked as its own issue. The availability
gap the exemption was written to name is real and stays open: see
ADR-0040's residuals.)*

**The affected set is a fixpoint over those ranges:**

1. Seed: the target node, the written table extents, the consumed
   ranges.
2. **Downward containment, range-bounded:** every node whose
   host-qualified extent intersects a destroyed range joins the set.
   *(Amended in 13.0.0 by ADR-0039: the range bound is the seed, not the
   whole rule. Descent also runs from a node already in the set into the
   content it carries, bounded per edge target by declared geometry —
   refused only on a positive contradiction, and never out of a node's
   own address space. The range bound alone left a partial destruction
   unable to reach the content it truncates, which is issue #338's
   defect (b).)*
3. **Upward backing:** a `BackingSignature` in the set brings its
   consumer.
4. **Downward production, restricted to destroyed substrate:** a
   producer in the set through a destroyed range brings its products —
   the mapper device an encryption layer produces, the virtual devices
   and volumes an aggregate produces — recursing over the products'
   own hosted content.

A mutating step **constructs only if every node in its affected set is
`Permitted`**, or carries a valid acknowledgment for an
acknowledgment-gated arm (below). `Refused` in the set makes the step
unrepresentable — ADR-0012's axis discharged at the constructor.
`Indeterminate` refuses construction with a typed artifact carrying
cause and remediation.

This is round three's missing downward rule, built in: deleting `sdb1`
on root-on-ZFS-over-LUKS destroys `sdb1`'s range → the LUKS layer is
in the range → production brings the mapper device → its hosted ZFS
member signature → backing brings the pool → `Refused`. `vgremove` on
a VG holding a ZFS-backed LV and `mdadm --zero-superblock` on an array
carrying a pool close the same way.

### The non-interference theorem, proved as a property

**Claim:** a step's affected set contains no node whose extent is
disjoint from the step's destroyed, consumed, and table ranges, unless
reached through backing or production from a node inside them.
*(Amended in 13.0.0 by ADR-0039, which re-proves it in the stronger and
self-contained form its own premise was reaching for: no node whose
declared extent is comparable with its reacher's and lies outside it is
ever in the set. Containment descent now reaches nodes this claim
excluded — that is the fix — while the consequence the claim was written
for, that a sibling is never captured, holds by geometry rather than by
the edge taxonomy alone.)* *(Amended again in 14.0.0 by ADR-0043: a
partition released by the destruction of the table its own name says
describes it is in the set although it lies outside its reacher's
extent — membership there follows the naming relation and the step's
target, never geometry. The consequence is restated with the same care:
a sibling is never captured by a step that destroys another partition; a
step that destroys the table releases every partition it describes,
which is release, not capture.)* *(Amended a third time in 15.0.0 by
ADR-0044: the destroyed table need not be the step's target — destruction
is carried from the target along the same arms and under the same bound
as reach, and a table it reaches releases too; membership follows the
naming relation and the destruction carried from the step's target,
never geometry and never reach alone. The consequence stands: a sibling
is never captured by a step that destroys another partition, because a
range that touches a non-target node establishes no destruction of it.)*
The premise, stated as a named property of the edge taxonomy: **no backing
or production edge targets a physical device** — products are virtual
devices, volumes, or file systems; containment descent is strictly
range-bounded; table writes target table extents, not the parent
device.

Committed regressions on the named layouts: with
`zpool create tank /dev/sda2`, creating a BIOS boot partition on `sda`
**constructs** (affected = table extents + consumed free range) while
initializing `sda` **refuses** (destroys `[0, len)` ⊇ `sda2` → pool);
the ESP at `sda1` is never captured by its sibling's pool and Fusion
`disk0s2` does not freeze `disk0s1` (containment descent is
range-bounded); `lvresize lv_home` never reaches `lv_root` (a volume
is not a producer); Btrfs `device delete` on one backing never reaches
another (a file system is not a producer).

**The theorem is a property test beside the inverted-default test, not
an assertion.** Its premise is quantified over edge kinds, so SI-27's
round inherits a stated obligation: any edge kind it adds — the
multipath membership edge ADR-0011 deferred to it, the
host-backed-virtual-device edge its round-three record requires — must
either preserve the premise or re-prove the theorem under the new edge
set before acceptance. This design depends on that property, not on
the names SI-27 chooses.

### Device scope: the transport arm inverted, and SI-37's home

The device-scope transport arm carries a **closed positive list of
local transports** — NVMe over PCIe, SATA, directly attached SAS, USB
mass storage, SD/MMC, and the paravirtualized local classes.
Everything else is `Indeterminate` at device scope (`blocked`), or
`Refused` with the network-block-device non-goal cited where the class
is recognized as remote. Round three's inverse enum — three named
network transports, everything else mutable, NVMe-over-TCP included —
is the recorded counterexample this inversion ends.

**SI-37's fail-closed home.** The filed population — two simultaneous
paths to one LUN, no platform-assembled node, unequal identifiers — is
covered by construction for every non-local transport. For the
positive-local population, the state layer adds **per-transport
path-multiplicity contracts**: NVMe — the namespace's own
shared-capability report (CMIC/NMIC) and the platform's subsystem
grouping; SAS/SCSI — the device-reported WWN, whose cross-path
equality fires ADR-0011's existing SAFE-005 ambiguity rule; SATA, USB,
SD — point-to-point by transport construction, recorded as such. Where
the contract's answer is `unavailable`, or reports multi-path
capability without a platform-assembled node, mutation is `blocked`. A
single-controller NVMe namespace reporting no shared capability is
positively single-path by the device's own report and `Permitted` at
this arm.

**SI-37 is not resolved.** Its evidence clause requires the dual-path
matrix and negative controls before any option is accepted, and none
exists. This ADR does what the filing asked of SI-11's round: names
the case and gives it a fail-closed rule over observable predicates.
SI-37 stays open, stops gating increment 3 (its population is typed
and blocked, so the type is writable), and its matrix becomes the
acceptance evidence for any future arm moving that population to
`Permitted` — the SI-28-floor pattern applied to multipath.

### Node-local inheritance for device-scope refusals

A node's effective verdict is the worst of its own arm and **its own
root device's** device-scope verdict — a node-local rule, not a
closure rule. `mkfs.ext4` on an iSCSI LUN's partition is refused
through inheritance; an ESP on a SATA disk carrying a ZFS member is
not, because the refusal it would wrongly inherit lives on the
sibling's backing signature, not on the device. Inheritance cannot
re-derive sibling capture because it never traverses an edge.

### Per-operation status: regimes, operation classes, the canonical step

- **Regime A** — `Refused { NonGoal }`: mutating operations report
  `unsupported`, the reason citing the exact Section 2.1 clause and
  technology.
- **Regime A′** — SAFE-005 ambiguity and every `Indeterminate`:
  mutating operations report `blocked` with cause and remediation.
  Ambiguous identity routes here, never to A.
- **Regime B** — PART-014 protected objects: **status unchanged**,
  reason and confirmation strength attached.
- **Regime C** — runtime preconditions (missing tool, version, lock,
  dirty volume, Recovery-only, pool health): `blocked`. MAC-009's
  Recovery-only rule lives here, intact.

**The operation-class dimension:** CAP-002 operations partition into
mutating classes and source classes (`detect`, `read`, `check` in
read-only mode, `copy`-as-source). A refused or indeterminate verdict
suppresses only the mutating classes. Source classes are never
suppressed — WIN-004's copy-off-LDM stays advertised, `detect` on a
Storage Spaces pool stays honest per WIN-003 — under the
**source-access predicate**: a refused or indeterminate node may be an
operand only of steps that are source steps in their entirety, with
the read-only invocation mode helper-verified (FS-005's health check
on a dirty NTFS inside an LDM partition runs read-only or not at all).

**CAP-005 agreement by construction:** the capability engine computes
a mutating operation's status by running the same closure on the
operation's **canonical step** — each CAP-002 operation defines its
canonical effect-table entry over the target. One computation, two
callers; the planner and the capability engine cannot disagree on a
target/operation pair.

### PART-014, exhaustively classified, outside the body

The class function is normative here, over all nine enumerated kinds,
from helper-side live discovery:

| PART-014 object | Discriminant, per the contract |
| --- | --- |
| EFI System | partition type GUID (byte layer) |
| Microsoft Reserved | partition type GUID (byte layer) |
| Windows Recovery | partition type GUID plus BCD/ReAgent references (state layer) |
| Apple recovery | APFS volume role (byte layer / IOKit) |
| Apple sealed system volume | APFS role plus seal state (IOKit) |
| Signed system snapshots | the flag ADR-C5 placed on the carrying file system |
| Linux boot | mount path `/boot`, `/boot/efi` (state layer), plus fstab-declared purpose where readable (LIN-010's read) — Debian's generic-GUID `/boot` is identified by mount, not GUID |
| Active swap | `/proc/swaps` / platform equivalent (state layer) |
| Current boot/root volumes | mount table root plus EFI boot entries (state layer) |

Regime B never enters a verdict, so none of this is body content —
which also keeps mounts and swap state out of the hashed body,
consistent with MODEL-005's body-stability rule. **The bounded miss is
named:** an unmounted, fstab-invisible generic-GUID `/boot` is
unidentifiable; because Regime B never flips status, the miss costs a
warning, never a safety property.

### The acknowledgment vocabulary: the escape that is not a bypass

Three acknowledgment-gated arms sit between `Permitted` and `Refused`.
Their closed vocabulary: **release acknowledgment**,
**opaque-destruction acknowledgment**, **identity-bound-restore**. Each
is a typed, hash-bound plan value recorded at plan creation under
UI-009 typed confirmation, and re-derived by the helper at validation.

- **Orphan and released signatures.** A validated signature whose
  consumer is not observed — an exported ZFS label (its own state
  field), a stale superblock in a shrink's released tail, an
  unassembled member — is `Indeterminate`: `blocked`, remediation
  "import/assemble to confirm, or record the release acknowledgment"
  naming the technology, the parsed designator, and the consequence.
  With the acknowledgment, the step constructs.
- **A consumed member has no acknowledgment.** Where the consumer *is*
  observed — pool imported, array assembled, space online, or the
  label's own state saying active — the arm is `Refused` and **its
  constructor has no acknowledgment parameter**. The distinction from
  PART-014's bypassable "without an explicit supported plan" gloss is
  representational, per ADR-0012: the consumed case has no sentence,
  and an acknowledgment authored against an orphan that validation
  finds consumed is a divergence and rejects.
- **Locked encryption layers.** A locked layer's contents are
  unenumerable by anyone, helper included. Destructive operations on
  it are `Indeterminate`: `blocked`, remediation "unlock, or record
  the opaque-destruction acknowledgment" — FS-010's recovery-material
  acknowledgment plus a typed statement that the contents cannot be
  verified. Round three's defect — a locked container holding a pool
  destroyed at `supported` — is closed: never silent, never default,
  never `supported` without the recorded acknowledgment. **The
  residual is stated, not rounded away:** a hidden non-goal inside a
  locked container is destroyed blind if the user acknowledges;
  opacity is physical, the guarantee's scope is observable topology
  (ADR-0012's own scoping), and this design makes the blind spot a
  typed, confirmed act rather than a silent default.
- **Indeterminate tables.** Against an `Indeterminate` table state,
  exactly one write family is permitted: restore from the product's
  own identity-bound table backup (PART-013's artifact, REC-001's
  validation). Free-form table writes are `blocked`; PART-001
  initialization stays governed by its categorical invariant, which an
  `Indeterminate` table never satisfies.

### SI-29, resolved: the narrow reading with a geometry line

The protected objects are the pool, the spaces as structural objects,
and the member-disk substrates — **not the file systems inside a
space**. An NTFS resize within a space's existing provisioned block
interface, through WIN-001's documented API against the virtual disk,
mutates no pool or space metadata and is an ordinary target.
Everything changing the space's own geometry or membership — resize or
delete the space, add or remove members, retire a disk — is pool/space
mutation: `Refused`. Member-disk substrates refuse through the
ordinary closure. Two gates travel with the permission: mutation
inside a space is Regime C `blocked` when the pool is degraded or a
thin-provisioned space's allocation headroom cannot be verified, and
the write path is the platform's own supported API, never a raw write
to the backing. The broad alternative was rejected on its measured
cost: every Storage Spaces user loses NTFS resize for a protection
Section 2.1's own words ("pools/spaces") do not claim.

### SI-30, resolved: deletion severed and routed

The sealed system volume and signed system snapshots, **as direct
targets**, are `Refused` for every mutating class, in every
environment, with no acknowledgment arm — Section 2.1's "never
modified," absolute. A whole-container or whole-device destructive
step that reaches the sealed volume **through the substrate closure**
is governed not by the NonGoal arm but by a named, closed step family:
boot-volume work on documented supported paths (Section 2.1's own
second clause), gated by MAC-009 — `blocked` with the exact
Recovery-only reason wherever macOS so gates it. **In v1 that family
is empty:** no such step is implemented, every container erase refuses
through the closure like any other reached non-goal and reports
`unsupported` as any unimplemented operation does, and the model
carries the slot so that ADR-M1's eventual documented path becomes a
Regime C matter, not a closure amendment. The absolute alternative —
the sealed volume's arm refusing the container erase forever — was
rejected as round one's error re-shipped: it hard-codes `unsupported`
for an operation MAC-009's text contemplates as `blocked`.

### Construction, decode, and the single engine

The only way to obtain a mutating `PlanStep` is a constructor taking
the snapshot body, the target, the effect-table entry, and the
structural effects; it computes the affected set and verdicts and
returns either the step — carrying its affected set and the
host-qualified estimated ranges Section 6 requires — or a typed
refusal. The compile-fail proof lands in the pattern ADR-0012 names.
**Decode re-runs the closure:** the schema-validation pass — the named
sole decode boundary, with its own error type — recomputes affected
sets and verdicts from the decoded artifact and rejects disagreement,
mutation-verified. The closure is implemented once, in the Rust
engine; TypeScript carries canonical encoding and hashing parity
(golden-tested) plus structural validation, and no TypeScript surface
is trusted for a verdict it displays (CAP-007; CAP-005/CLI-003 already
require the one engine). The alternative Part 6 priced — dropping the
affected set from the body — was rejected on ADR-0016's ground: the
affected ranges are what the user authorizes.

### Bind set, extents, free extents, rulesets

**The bind set** (CONC-001) is the transitive closure of the affected
set under reverse backing, reverse production, and ancestor
containment, down to and including every device node — shrinking
`sda1` binds `sda`, closing round three's concurrent-GPT lost-update
path. The rule is stated over edge semantics ("the bytes of A live
within or derive from B"), so SI-27's host-backed-virtual-device edge
is traversed by the same rule with no restatement when it lands.

**The extent accessor is total by domain restriction**: extents exist
on the extent-bearing kinds — device, table (explicit primary/backup/
MBR/EBR extents), partition, signature (primary offset), file system
(superblock offset) — and the extent clause is vacuous elsewhere by
stated rule. Every range is host-qualified; one address space per
containment-forest root. **Free extents leave the hashed body**: they
carry no verdict input, PART-009/PART-012 compute placement from the
planner's own policy, and the constructor's consumed-range check works
on ranges, not `FreeExtent` nodes.

**Rulesets.** The closure and arms are versioned and ship compiled
into the helper (SAFE-008). No plan is ever evaluated under the
ruleset it declares — HLP-002, CAP-007, and SAFE-008 already forbid
it; the prohibition is restated at the constructor and validator with
a negative test. A ruleset change that alters any verdict within a
bound target set is ADR-0016's divergence: it rejects before the first
write, invalidates Draft/Validated plans per CONC-003, and on
resume-after-update the plan passes through `Revalidating` under the
new rules — a changed verdict strands into `RecoveryRequired` with
recovery actions re-planned under the current ruleset, and REC-010
stops advertising a rollback whose reversal plan no longer validates.
The stranded-machine cost is intrinsic to authenticated verdicts;
lifecycle machinery stays with SI-20 through SI-22.

## Options considered

### Runtime-guard reach (round three's drift)

Rejected: ADR-0012 already fixed the axis, and a closure consulted
only at validation is the option the register recorded as not
surviving its own bug.

### A permitted residual arm

Rejected: round three's recorded fail-open defect. The residual is
`Indeterminate`, property-tested.

### Unconditionally refused orphan signatures

Rejected, as round three's review directed: a bench-tested disk pulled
from a pool would be `unsupported` for initialization, DIA-004, and
ACC-010 forever, with the only escape an unsupervised external
`zpool labelclear` — the hazard the product exists to prevent. The
acknowledgment arm replaces it.

### The broad SI-29 reading

Rejected on measured cost, as stated in the SI-29 section.

### The absolute SI-30 reading

Rejected as round one's error re-shipped, as stated in the SI-30
section; recorded here so the severance is a decision, not an
omission.

### Verdict out of body

Already priced and rejected by ADR-0016; the fork's conditions govern
any future out-of-body retreat.

## Decision

**SI-11 moves to Resolved.** The protection verdict is three-valued,
total, and fail-closed with an `Indeterminate` residual, computed from
the named two-layer helper evidence contract under the protective join
rule. A mutating step's affected set closes over destroyed substrate —
downward containment range-bounded, upward backing, downward
production — with release counted as destruction *(amended in 13.0.0 by
ADR-0039: it closes over the content the target carries as well, and
containment descent is bounded per edge target rather than by the
destroyed ranges)*; a step whose
affected set reaches a `Refused` node is unrepresentable, and
`Indeterminate` refuses construction with a typed artifact. Device-
scope refusals inherit node-locally; the transport arm is a closed
positive local list; capability status is computed from canonical
steps by the same closure. PART-014 classification is exhaustive,
Regime B, and outside the body. The acknowledgment vocabulary is
closed at three entries. **SI-29 and SI-30 are resolved within this
decision** as specified above. **SI-37 is reclassified**: open, off
the increment-3 gate, its dual-path matrix the acceptance evidence for
any future relaxation of the populations this design blocks.

## Consequences

The normative amendments landing with this ADR in spec 11.0.0, each
inside the reservation's grant:

- **Section 2.1's enforcement paragraph** gains the closure
  commitment: verdicts total with an `Indeterminate` residual, the
  affected-set reach with release-as-destruction, construction refusal
  as the discharge of unrepresentability.
- **Section 2.1's Storage Spaces entry and WIN-003** gain the SI-29
  boundary (the narrowing that makes this major).
- **Section 2.1's sealed-volume entry and MAC-009** gain the SI-30
  severance and routing.
- The document-control version row and §0.3 changelog row.

Other consequences:

- **Positive:** increment 3 can write the verdict, step, and
  capability types against a decided shape; every recorded
  data-destruction case from rounds two and three has a named
  counter-mechanism and a committed regression.
- **Negative, accepted knowingly:** the locked-container residual
  (stated in terms above); availability costs on unmeasured
  populations — non-local and unrecognized transports, multipath-
  capable NVMe, orphan-signature hosts — which fail closed until
  their named evidence exists; and the stranded-machine cost of
  ruleset changes mid-flight, routed fail-closed.
- **Obligations forward (evidence, under the standing sitting
  discipline, recorded here rather than in a review memory):**
  (1) state-layer double-capture stability rows per platform for each
  fact the arms consume; (2) fabric-versus-local transport
  discrimination rows per platform for each listed local transport;
  (3) NVMe shared-capability rows (CMIC/subsystem), Linux and
  Windows; (4) the consumed-versus-released discriminants measured,
  not recalled — the ZFS label state field on the L-E fixture family,
  and the assembled-state facts for mdraid and Storage Spaces.
- **Obligations forward (increment 3):** the compile-fail
  construction proof, the no-sibling-capture property test, the
  inverted-default property test, the decode-recompute mutation
  tests, and the root-on-ZFS regression pair.
- **Obligations forward (first write-capable increment, beside the
  SI-33/SI-34/SI-35 banner obligations):** end-to-end refusals on the
  fixture families — the consumed-member refusal, the release-
  acknowledgment path on the stale-tail shrink, the locked-layer
  acknowledgment path, and the `gpt-conflicting-tables-512`
  restore-only rule — each mutation-verified.
- **Handed to SI-27:** the theorem premise (no backing or production
  edge targets a physical device) to preserve or re-prove under any
  new edge kind, and the bind-set edge-semantics rule its naming must
  slot into.

## Verification

- When increment 3's types land: the property tests and compile-fail
  proofs named above; the canonical-step capability computation shared
  with the constructor (CAP-005 agreement by construction).
- When a write path exists: the four end-to-end refusal families
  above, mutation-verified.
- Register: SI-11, SI-29, SI-30 read Resolved; SI-37 reads open,
  reclassified, no longer gating increment 3; any text implying SI-37
  is resolved, or that SI-27 or SI-28 moved, is an error against this
  ADR.

## Revisit conditions

- SI-27's round finds the theorem premise unpreservable under its
  edge kinds — the theorem must be re-proved, not patched, before its
  round is accepted.
- A verdict component is found that cannot be derived
  deterministically from the named contract — ADR-0016's revisit
  condition, restated here where the contract now has content.
- SI-37's dual-path matrix is taken: the blocked multipath-capable
  populations deserve re-examination against measured reality, in
  their own round.
- Apple documents a supported whole-container path (ADR-M1's
  territory): the empty step family gains its first member and
  MAC-009's blocked routing activates — foreseen, not a model change.
- Any proposal to add an acknowledgment arm to a consumed-member
  case, which this ADR makes unrepresentable deliberately.
