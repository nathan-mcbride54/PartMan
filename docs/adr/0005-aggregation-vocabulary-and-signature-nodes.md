# ADR-C5: One aggregation node, on-disk signatures as nodes, and the scope of `StorageSnapshot`

- Status: **Proposed** — needs the decision owner's acceptance
- Date: 2026-07-28
- Spec version: 3.1.0
- Work packages blocked: WP-010 increment 3 (**partially; see Scope**)
- Requirement IDs: Section 5, MODEL-002, MODEL-003, MODEL-005, MAC-002,
  MAC-003, MAC-009, MAC-010, FS-003, FS-004, LIN-004, LIN-005, LIN-006,
  PART-014, PART-015, INV-004, INV-008, CAP-001, CAP-002, RPC-002, Section 20
- Resolves: SI-07, SI-08, SI-09, SI-10
- Does **not** resolve: SI-11, SI-27. **WP-010 increment 3 remains blocked.**
- Decision owners: repository CODEOWNERS

Acceptance basis: filed under Section 1.11. Round one's answer to these four was
rejected and recorded in Part 4 of `docs/spec-issues/README.md`. Round three's
design was put to five adversarial lenses, which raised fifty-eight objections;
the review destroyed the protection closure and the node-naming scheme and did
not refute the vocabulary. Everything the review landed on the vocabulary is
folded into the decision below, and what was rejected is recorded so the next
round starts from the objection rather than from scratch.

Unlike ADR-C2, ADR-C3, and ADR-C4, this ADR carries no delegation from the
decision owner. It is submitted as a proposal.

## Context

Four filed conflicts turn out to be one question: **which node types express
MODEL-002's non-linear relationships, and where do their boundaries fall.**
Section 5 names types it never defines, and MODEL-002 names a layering that it
then requires be broken.

**SI-07.** Section 5 lists `StorageContainer`, `StoragePool`, and `RaidSet` and
defines none of them. MODEL-002 lumps "Storage Spaces, LVM, RAID, APFS
containers, and Btrfs multi-device file systems" together as non-linear
relationships without saying which is which, so the same membership edge can be
modelled twice and the choice is hash-visible. MAC-003 additionally requires
APFS *physical stores* — plural, and many-to-one for Fusion under MAC-010 —
which a one-to-one `StorageContainer` cannot express at all.

**SI-08.** MODEL-002 places file system strictly above volume, and in the same
sentence requires representing Btrfs multi-device file systems, where one file
system spans several devices and performs the aggregation role a container
performs for APFS. APFS gets an explicit container type; Btrfs gets none.

**SI-09.** FS-004 requires detecting "LVM PV, Linux RAID, LUKS, BitLocker, ZFS
pool members, Storage Spaces, LDM metadata" under file-system operations, while
MODEL-002 puts encryption and containers on layers distinct from file system.
Either `FileSystemKind` enumerates non-file-system signatures and breaks the
mandated layering, or FS-004's results are materialized as nodes. The answer
changes every snapshot hash.

**SI-10.** Section 5 requires a `Snapshot` type; Section 20 defines only
"Snapshot (topology)", which is `TopologySnapshot`. The storage-level type is
named and never specified. Explicit requirements exist for APFS (MAC-003), LVM2
(LIN-004), and Apple signed system snapshots (MAC-009, PART-014). Windows VSS is
implied by PART-015 naming a VSS store as a shrink-limit cause; Btrfs snapshots
are implied by nothing, despite FS-003 and LIN-006 requiring Btrfs support.

Round one answered all four and was rejected. Its load-bearing justification was
that detect-only status would be a total function over a closed technology-kind
enum. It is not, on the first non-goal it maps: an Apple Fusion container
(MAC-010, detect only) and an ordinary mutable APFS container (MAC-002, ADR-M1)
carry the same kind and differ only in member-set shape. Part 4 recorded what the
next attempt needed: **a detect-only predicate defined over kind and membership
rather than kind alone.**

## Safety analysis

This ADR fixes vocabulary, not verdicts, so its safety effect is indirect but not
zero.

The load-bearing safety property is that **membership becomes a first-class,
observable field rather than something inferred from which nodes happen to be
present.** Round one's failure was a protection answer that could not see member
shape; a vocabulary that cannot carry member shape guarantees the same failure in
every future round. `Aggregate.reported_backings` exists for that reason and is
specified as the aggregate's *self-report*, never a count of present nodes,
because a Fusion Mac with its HDD detached presents one present store — so a
count of present backing nodes would classify a degraded Fusion set as an
ordinary mutable container, reaching a Section 2.1 MUST NOT by unplugging a
cable.

Two further safety-relevant effects:

- `consumer: Null` gives an orphaned member (an exported zpool, an unassembled
  mdraid member, a Storage Spaces disk whose pool is offline) a representation.
  INV-008 requires these be represented without flattening or discarding, and
  SAFE-005 needs a node to attach a refusal to. The previous model could only
  drop them, which is the shape of a silent whole-device destruction path.
- The SI-09 routing rule is stated here rather than left to an adapter precisely
  because an unrouted signature is a capability difference: under one reading a
  partition holding an unrecognized superblock would be permanently undeletable.

No MUST is weakened. Three names are removed from Section 5's list, which is a
semantic change to an existing requirement and is priced in Consequences.

## Options considered

### SI-07 — the aggregation boundary

**Option A — keep the three types Section 5 names.** Benefits: no spec edit.
Costs: none of the three has a definition to implement, so an implementer invents
the boundaries and two implementers invent different ones; the same LVM volume
group is a defensible instance of all three; and the one-to-one shape the word
"container" implies cannot express MAC-003's plural physical stores, which is a
MUST. This is SI-07 restated, not answered.

**Option B — one `Aggregate` node carrying a closed technology discriminant,
with membership expressed by an edge of unbounded in-degree.** Benefits: one
logical object has one node and therefore one canonical encoding (MODEL-005);
many-to-one membership is native, so MAC-003 and MAC-010 become expressible; the
discriminant is the technology, which is what adapters actually observe. Costs:
three names in Section 5 are replaced by one, a semantic change to an existing
requirement; and the discriminant alone cannot decide protection, so the type
must additionally carry member-set shape.

**Option C — an open text discriminant.** Benefits: a new technology never needs
a spec edit. Costs: fatal. Two adapters emitting `"lvm2"` and `"LVM2"` are
different values under `schemas/canonical-encoding.md` §4, which applies no
normalization by design and assigns normalization to the producing adapter. That
is a permanent body-hash divergence between client and helper on unchanged
hardware — the PLAN-006 failure ADR-C2 exists to prevent, and the one ADR-C2
warns gets "fixed" by relaxing the comparison.

### SI-08 — Btrfs multi-device

**Option A — a synthetic container node above the member devices.** Costs:
introduces a node whose existence two adapters can disagree about, and duplicates
the container concept for a technology that has no container.

**Option B — a `FileSystem` with an ordered set of backings, single-device being
the cardinality-1 instance of the same shape.** Costs: every file system — ext4,
NTFS, everything — gains a backing set. Hash-visible, and paid now rather than
when Btrfs support lands.

**Option C — as B, but single-device and multi-device are distinct variants.**
Costs: fatal for a supported workflow. `btrfs device add` is a routine capacity
expansion and would change the node's *shape*, so every stored plan, every
PLAN-008 reversal plan, and every journal-referenced snapshot naming that file
system breaks.

### SI-09 — FS-004's non-file-system signatures

**Option A — enumerate them into `FileSystemKind`.** Costs: breaks MODEL-002's
mandated layering, and makes CAP-001 compute `check` and `repair` for a LUKS
header, which is meaningless under CAP-002's per-operation model.

**Option B — materialize them as nodes on the layer MODEL-002 assigns,
separating the on-disk evidence from the consumer that reads it.** Costs: two
nodes where an adapter reports one fact; and a routing rule is required for a
signature matching nothing known, without which the same superblock is a file
system to one adapter and an aggregation signature to another.

### SI-10 — the scope of the storage-level snapshot type

**Option A — cover only what has an explicit requirement (APFS, LVM2, Apple
signed system).** Costs: PART-015 must report a shrink floor's cause with
per-cause remediation and names a VSS store as such a cause; with no type the
cause degrades to free text, which UI-010's structured cause-and-next-step cannot
key off and which is a normalization hazard. INV-004 and INV-008 separately
forbid discarding a Btrfs snapshot that FS-003 and LIN-006 make the product
responsible for.

**Option B — cover VSS and Btrfs, and rename the type.** Costs: a Section 5
rename and a Section 20 glossary edit.

## Decision

### SI-07 — Option B

One `Aggregate` node type. `AggregateTechnology` is a **closed** enum with a
total catch-all:

`Apfs | Zfs | Lvm2 | MdRaid | StorageSpaces | Ldm | CoreStorage | Unrecognized { probe_tag }`

Membership is expressed by a `Backs` edge from an on-disk signature to its
consumer, with **unbounded in-degree**. That is what makes MAC-003's plural
physical stores and MAC-010's two-store Fusion container expressible, and it is
precisely what SI-07 complained a one-to-one `StorageContainer` could not do.
Adding a named variant is a MODEL-003 breaking change refused by the RPC-002
handshake; `Unrecognized { probe_tag }` keeps the enum total without making it
open text, and the payload separates two different unknowns so they do not
silently merge.

`Aggregate` carries `reported_backings` — **the container's own self-reported
physical-store count, never a count of present nodes.** This field is what
discharges Part 4's requirement that detect-only be a function of kind *and*
membership. A single-store APFS container (`reported_backings == 1`) is mutable
under MAC-002 and ADR-M1; a Fusion container (`reported_backings == 2`) is out of
scope under MAC-010. Same discriminant, opposite answer, decided by membership.

The self-report is load-bearing and is decided here rather than left to an
adapter, for the reason given in Safety analysis: counting present backing nodes
classifies a degraded Fusion set as an ordinary mutable container.

### SI-08 — Option B

Btrfs multi-device is a `FileSystem` with an ordered set of n ≥ 1 backings, and
**single-device is the cardinality-1 instance of the same shape.** No synthetic
container node is introduced, so no node exists whose existence two adapters can
dispute. There is exactly one node per logical file system, so MODEL-005's
single-canonical-encoding guarantee holds; a literal-nesting model would have to
duplicate the node under each member device and the copies could disagree.

### SI-09 — Option B, with the routing rule stated

FS-004's non-file-system signatures become `BackingSignature` nodes — "these
bytes at these offsets in this host say it is a member of X" — nested inside the
host's own extent, which is physically accurate for the ZFS vdev label, the LVM2
PV label, the mdraid superblock, the LUKS header, BitLocker FVE metadata, LDM
metadata, and Storage Spaces metadata. The consumer (`Aggregate`,
`EncryptionLayer`, or `FileSystem`) is a separate node and the `Backs` edge is
**optional**: `consumer: Null` means "member of an aggregate that is not
observed", which is exactly an exported zpool, an orphaned mdraid member, or a
Storage Spaces disk whose pool is offline.

`FileSystemKind` stays purely file systems and carries `Unrecognized { probe_tag }`.
MODEL-002's encryption → volume → file-system chain stays expressible, and
CAP-001 never computes `check` or `repair` for a LUKS header.

**The routing rule is part of this decision and is not left to an adapter.** An
unrecognized signature becomes a `BackingSignature` **only** when it matches a
known aggregation or encryption magic family. Everything else is
`FileSystem { Unrecognized { probe_tag } }`, with SAFE-005 governing writes
*into* it while leaving whole-partition destruction available.

Without this rule a ReiserFS superblock — FS-004 requires detecting "common
legacy file systems", and ReiserFS, JFS, UFS, HPFS, Minix, and VMFS appear in no
enumerated kind — is a `FileSystem` to one adapter and a `BackingSignature` to
another: different node kind, different body hash, opposite capability, on
entirely ordinary media. Under the second reading the partition would
additionally be permanently undeletable, since the stated escape for a signature
(clear it outside the product) has no analogue for a file system.

`probe_tag` MUST be defined normatively — which prober, which offset, which magic
bytes — before the type is written, because two adapters producing different tags
for the same bytes is the same divergence by another route.

### SI-10 — Option B

The type is `StorageSnapshot`, resolving the Section 20 conflict by an explicit
rename rather than by convention.

`StorageSnapshotKind = Apfs | Lvm2 | AppleSignedSystem | Vss | Btrfs | Unrecognized { probe_tag }`

VSS is in because PART-015 requires the true minimum shrink size **and its
cause** with per-cause remediation and names a VSS store as such a cause, and
because INV-004 and INV-008 require the structure be represented rather than
discarded once WIN-006 makes the product use VSS. Btrfs is in because FS-003 and
LIN-006 make the product responsible for Btrfs and INV-008 forbids dropping its
snapshots. `StorageSnapshot` is a node kind, not a hashed artifact, so it carries
no schema identifier and canonical-encoding §5 domain separation is not at risk.

**MAC-009's signed system snapshots do not require this node to be a
protection-graph participant.** The verdict input relocates to a boolean on the
`FileSystem` that carries the snapshot. This is what makes SI-10 answerable
independently of SI-27, which round three's design assumed it was not.

### Two rules binding every type this ADR fixes

Stated once here rather than per enum, because the review found the same defect
in six places.

**Rule 1 — a closed enum over externally observed values MUST carry
`Unrecognized { probe_tag }`.** This binds `AggregateTechnology`,
`FileSystemKind`, `StorageSnapshotKind`, `BackingTechnology`,
`EncryptionTechnology`, `Transport`, `Virtualization`, and `VolumeRole`. Without
it, INV-008 is unsatisfiable the first time a platform ships a value the product
does not know: a macOS release adding an APFS volume role leaves only silent-drop
(an INV-008 violation) or refuse-to-snapshot (denial of discovery on an OS
upgrade), and Section 2.1 requires ZFS snapshots be detected and represented
while `StorageSnapshotKind` has no home for them. It is hash-visible, so adding
the variant later is a MODEL-003 breaking change refused by RPC-002.

**Rule 2 — a hashed body may carry a fact only if it is invariant under re-probe
of unchanged hardware.** This is a *narrowing* of MODEL-005's envelope rule, not
a replacement: the envelope rule decides what is authenticated, this rule decides
whether PLAN-006 is satisfiable, and both must hold. Consequently
`FileSystem.size`/`free`, `Volume.size`/`reserve`/`quota`, the `Mount` set, and
the `StorageSnapshot` set are **envelope** content. Technology discriminants,
`reported_backings`, the APFS sealed flag, and backing-set membership are
invariant and remain body content. A fact that a verdict needs but that fails
this rule is a signal that the wrong fact was chosen, not a reason to relax the
rule.

## What was rejected, and why

**Rejected: amending MODEL-005's envelope rule.** Round three proposed replacing
"the privileged helper independently re-derives it" with "no helper decision
reads the client's copy of it". **Rejected as fatal.** Under HLP-002 the helper
decides on almost everything, so the amended rule sweeps every descriptive field
into the body — including `FileSystem.free`. On a stock Ubuntu host journald
writes between capture and revalidation and the body hash changes; on macOS
`tmutil` mints an APFS local snapshot on the hour; on Windows VSS creates a
shadow copy before every update; on any systemd host, autofs and snap mount units
add and remove mounts with no storage change at all. That is ADR-C2's Option A
reintroduced verbatim, and ADR-C2 rejected Option A because "an over-inclusive
body makes PLAN-006 unsatisfiable, and a freshness check that can never pass gets
'fixed' by relaxing the comparison." Worse, the relaxation would then have to
decide whether the frozen protection verdict is inside the compared subset,
turning a fail-closed property back into a code-path property.

The tension the amendment was reaching for is real, and it belongs to SI-11: the
settled approach requires the derived protection verdict inside the hashed body,
while the literal envelope rule sends a helper-re-derived value to the envelope.
Its resolution is a **narrow, named over-inclusion in the ADR-C2 tradition, not a
rule rewrite.** ADR-C2 did exactly this for CONC-004's transitional marking — "By
the rule it could sit in the envelope... Putting it in the body makes [it] a
property of the encoding rather than a check that some code path might skip."
Rewriting the rule to admit one field moved every other field with it.

**Rejected: three disjoint aggregation types as Section 5 names them.** No
requirement supplies a boundary between them, the one-to-one reading fails
MAC-003 immediately, and keeping the names to avoid a spec edit ships SI-07
unresolved under the appearance of compliance.

**Rejected: an open text discriminant.** `schemas/canonical-encoding.md` §4
applies no normalization by design, so two spellings are two values and one
permanent divergence.

**Rejected: distinguishing single-device from multi-device Btrfs at the type
level.** `btrfs device add` would change node shape on a routine capacity
expansion, breaking every stored plan and reversal plan naming that file system.
This is the load-bearing half of the SI-08 answer.

**Rejected: enumerating FS-004's signatures into `FileSystemKind`.** Breaks
MODEL-002's mandated layering and makes CAP-001 compute file-system operations
against an encryption header.

**Rejected: `StorageSnapshot` nodes as hashed body content.** Their set is
volatile on every supported platform for reasons unrelated to storage change —
Time Machine hourly, VSS before every Windows update, snapper and timeshift on
Linux — and PART-015's shrink-floor cause is a capability reason, not a node.

**Rejected: deriving the PART-014 protected-object class from `Volume` fields.**
A `Volume` requires an incoming edge from an `Aggregate` or `EncryptionLayer`, so
a basic-disk `C:` and a plain `/dev/sda2` root have no `Volume` node at all, and
`is_current_boot`/`is_current_root` have nowhere to live on the primary
platform's commonest layout — which would leave ACC-001, a Section 19
release-gate workflow, unable to demonstrate its safeguard. The PART-014 class
never enters a protection verdict, so it is computed by the helper from live
discovery including mount path, partition flags, and label, none of which need be
body content. This is also what removes `Mount` and the active-swap flag from the
hashed body, so the two defects resolve together.

**Not decided here, deliberately: everything downstream of the verdict.** The
protection predicate, the affected-set closure, and the node-naming scheme belong
to SI-11 and SI-27 and remain filed. This ADR fixes what the types *mean*; it
does not fix their complete field list, because naming inputs are hash-visible
and SI-27 owns them.

## Consequences

Positive:

- SI-07's complaint is answered structurally rather than by convention:
  many-to-one membership is native, so MAC-003 and MAC-010 are expressible, and
  one logical object has one node and one canonical encoding.
- What Part 4 asked for is delivered. `reported_backings` makes detect-only a
  function over kind **and** membership, and round one's counterexample — a
  Fusion container versus an ordinary APFS container — produces opposite answers
  from the same discriminant.
- `consumer: Null` gives INV-008 a representation for an orphaned member, which
  the previous model could only flatten or discard, and gives SAFE-005 a node to
  attach a refusal to.
- `FileSystemKind` stops carrying things that are not file systems, so CAP-002's
  fourteen operations stay meaningful per target.
- The two general rules each close a defect *class* rather than the instances a
  review happened to find.
- SI-10 is decoupled from SI-27, which round three's design believed impossible.

Negative and to be managed:

- **Every file system gains an ordered backing set**, including ext4 and NTFS
  that will never have more than one. Hash-visible, decided now.
- **Three names disappear from Section 5** — `StorageContainer`, `StoragePool`,
  `RaidSet` — a semantic change to an existing requirement. They were listed and
  never defined, which is SI-07's own complaint, so nothing implemented is
  invalidated: `crates/domain/src/lib.rs` exposes only `pub mod canonical` and no
  Section 5 type exists yet.
- **`probe_tag` is now load-bearing in eight enums and is a naming input for
  `BackingSignature`.** Its normative definition is a precondition on writing any
  of these types and is owned by whichever ADR lands the naming scheme.
- **Recognizing eight aggregation and encryption technologies commits to eight
  parsers, eight `cargo-fuzz` targets, and eight CAP-006 fixtures.** SEC-003 and
  Section 11.4 require the fuzz targets and already name LVM/LUKS/mdraid
  metadata; Section 16 forbids claiming a capability without its fixture and
  acceptance evidence; Section 11.7 fails a work package claiming a requirement
  without linked evidence. This is a large cost committed on day one and easily
  under-budgeted.
- **The version-bump reading needs an owner.** Section 0.1 makes a semantic change
  to an existing requirement a major bump, so accepting this ADR implies 4.0.0.
  That rule is already being applied inconsistently: 3.1.0 shipped as a minor bump
  while changing SAFE-003's definition of identity strength. Nothing in this ADR
  depends on the answer, but it should be settled deliberately rather than
  inherited.
- **WP-010 increment 3 remains blocked.** These four decisions fix the
  vocabulary; the types cannot be written until SI-11 supplies the verdict and
  SI-27 supplies the naming inputs, both of which add hash-visible fields to the
  very types fixed here.

## Verification

Test tier T1 throughout, unprivileged. Fixture images are generated by WP-020's
script; no binary images are committed (Section 16, 11.3). Every item links to a
requirement ID in `docs/traceability/` (Section 11.7).

**SI-07 / MAC-010 — the round-one regression guard.** Two APFS fixtures.
`reported_backings == 1` classifies as an ordinary container and every
MAC-002/ACC-006 operation is offered; `reported_backings == 2` is out of scope.
Same `AggregateTechnology` discriminant, opposite outcome. A model that decides
from the discriminant alone fails this test.

**Degraded Fusion.** A two-store container fixture with the second store's device
absent produces the **same** classification as the intact one. Assert explicitly
that a count of *present* backing nodes would have produced the opposite answer.
This test exists to make that implementation choice unmissable.

**MAC-003 plural stores.** A container with two `Backs` edges encodes as one
`Aggregate`; assert no encoding exists in which it is two, and that `Backs`
in-degree is unbounded by construction rather than by convention.

**SI-08 cardinality.** A single-device and a two-device Btrfs produce the same
node kind and variant; only the backing set differs. Simulate `btrfs device add`
and assert the node's shape is unchanged, so a stored plan naming the file system
does not break.

**SI-09 orphan.** A `zfs_member` signature with no observable pool, and a
`linux_raid_member` with no assembled array, each encode with `consumer: Null`
and are neither dropped (INV-008) nor mapped into `FileSystemKind` (MODEL-002).

**SI-09 routing.** A ReiserFS superblock fixture encodes as
`FileSystem { Unrecognized { probe_tag } }` in both the Rust and the TypeScript
producer, byte-identically. A LUKS2 header encodes as `BackingSignature { Luks }`.
Assert the two can never swap, and that `probe_tag` is byte-identical across
producers for one fixture.

**SI-09 layering.** Assert no `FileSystemKind` variant names an aggregation or
encryption technology, and that CAP-001 offers no `check` or `repair` for any
`BackingSignature` or `EncryptionLayer` target.

**SI-10 scope.** Fixtures carrying a VSS shadow store and a Btrfs snapshot are
represented rather than discarded; the VSS store additionally produces a
structured PART-015 shrink-floor cause keyed for UI-010 rather than free text. A
sealed APFS system volume's signed snapshot sets the boolean on its `FileSystem`
and is identifiable without the snapshot being a hashed node.

**Rule 1 totality.** For every enum bound by Rule 1, a fixture carrying an
unknown value encodes rather than failing, and is neither dropped nor coerced to
a known variant. Include an NVMe-over-TCP transport, an unknown APFS volume role,
and a ZFS snapshot.

**Rule 2 stability — the PLAN-006 property, re-run with the graph present.** Two
probes of one fixture separated by simulated file-system writes, a new local
snapshot, and a mount/unmount cycle produce **equal** body hashes. A transitional
and a stable snapshot of the same topology produce **different** body hashes
(ADR-C2's existing test, re-run with the node graph present).

**Cross-language golden vectors** (MODEL-005) for every type this ADR fixes,
including a `Null` case for every optional field, `u64` values above `2^53` in
every byte-count field, and one vector per `Unrecognized { probe_tag }` variant.

## Revisit conditions

- A technology appears whose membership is neither many-to-one nor one-to-one
  over a single consumer, so an unbounded-in-degree `Backs` edge stops being
  sufficient.
- A platform stops self-reporting an aggregate's member count, so
  `reported_backings` loses its input and the kind-plus-membership predicate
  loses the field Part 4 required.
- A file system appears that must distinguish its single- and multi-device forms
  at the type level, which would reopen SI-08's cardinality decision and the
  plan-stability argument that rests on it.
- `Unrecognized { probe_tag }` proves load-bearing in production for a technology
  that then gains a named variant. The migration is a MODEL-003 breaking change
  and MUST be planned as one, not made in place.
- SI-27 resolves in a way that requires a naming input this vocabulary does not
  carry, at which point the affected type's **field list** — not its meaning —
  reopens.
- SI-11 resolves in a way that requires a verdict input outside the
  body-stability rule, which would be evidence that the wrong input was chosen
  rather than that the rule is wrong.

## Scope

This resolves four of the six conflicts blocking WP-010 increment 3. Increment 3
stays blocked on SI-11 and SI-27, and on the filings this round produced: SI-28
through SI-32 in `docs/spec-issues/README.md`. SI-28 is a data-destruction path
in an already-accepted decision and should be read first.

Accepting this ADR implies a spec change to Section 5, MODEL-002, and Section 20,
which is not included here: the repository's practice is that an ADR is accepted
first and the spec change lands against the accepted decision.
