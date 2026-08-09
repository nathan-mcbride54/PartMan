# ADR-0019: Node names are derived addresses, and collisions produce artifacts

- Status: Accepted
- Date: 2026-08-09. The round and its resolution chain were accepted
  together by Nate McBride the same day
  (`docs/reviews/SI-27_ROUND_2026-08-09.md`, an untracked session
  artifact; everything load-bearing is restated here). Rounds two and
  three are recorded in Parts 5 and 6 of `docs/spec-issues/README.md`;
  this design is built from round three's nine recorded defects and
  its governing finding.
- Spec version: 11.1.0 (minor under §0.1 — Section 5 and MODEL-002
  gain additions, LIN-006's deferred-edge-kind clause gains its
  promised pointer, and no existing requirement's claim narrows)
- Work packages blocked: WP-010 increment 3 (SI-27 resolved; SI-28 is
  the gate's sole remaining item)
- Requirement IDs: Section 5, MODEL-002, MODEL-005, MODEL-006, ADR-C2,
  ADR-C5, SAFE-003, SAFE-005, SAFE-006, ADR-C3, ADR-0011, ADR-0018,
  CONC-001, CONC-003, INV-007, INV-008, LIN-006, FS-008, PART-005,
  PART-008, REC-003, Section 2.1, Section 11.4
- Decision owners: Nate McBride

## Context

Round two discovered the gap: any protection closure puts edges in the
hashed body, and edges need a `NodeId` that is stable across re-probes
of unchanged hardware and collision-safe across simultaneously present
devices — while every obvious scheme fails (paths and indices rename
on replug and churn; identity-derived names collide for exactly the
population ADR-C3 calls Weak). Round three's attempt drew nine
recorded defects, and the register's governing finding disciplines
everything here: **fail-closed-by-unencodability is not fail-closed** —
a collision that surfaces as an encoder error produces nothing to
display, journal, or refuse with, and turns a two-device ambiguity
into denial of discovery for the whole host.

What survived round three and is kept: the decomposition (a node
identifier is a **document-local address**, not a device identity —
SAFE-003's record carries identity separately); derived-never-stored,
with the decoder recomputing every id and rejecting unknown referents;
naming by position relative to already-named nodes, including nodes a
plan creates; two schema identifiers for captured and simulated
topologies; and the exclusion list — connection path, OS instance id,
positional indices and counters, table state and checksum, length,
regenerable identifiers, partition type, adapter-formatted text.

What changed since: the bound snapshot is helper-produced at
validation (ADR-0014/0016), so naming inputs are facts ADR-0018's
named evidence contract reads; and ADR-0018 handed this round two
requirements — the no-sibling-capture premise quantified over edge
kinds, and semantics classes on every edge kind for the bind set's
reverse traversal.

The filing's boundary, honored throughout: when two devices are
indistinguishable, the snapshot must still exist; the behavior may
neither *silently* merge them nor refuse discovery for the host; and
any fail-closed limitation attaches to the affected targets.

## Safety analysis

### The derivation: kind-discriminated positional addresses from contract-read fields

A `NodeId` is the recursive pair of the node's kind discriminant and
its per-kind naming fields, one of which is its parent's id — a
positional address rooted at physical devices, derived and never
stored. The schema-validation pass — the named sole decode boundary,
with its own error type, the same boundary ADR-0018's
decode-recompute uses — recomputes every id and rejects any edge
naming an unknown referent; the `pce/1` codec stays schema-agnostic
(round three's defect 7 placed correctly).

The per-kind naming maps:

| Kind | Naming fields |
| --- | --- |
| Physical device | canonicalized serial bytes, canonicalized WWN bytes, total bytes — nothing else; vendor/model excluded as transport-influenced text; both sector sizes excluded |
| Partition table | parent device id, table role discriminant |
| Partition | parent **table** id, start offset |
| Backing signature | host id, family, primary signature offset |
| File system | host id, kind, primary superblock offset (invariant under `btrfs device add`) |
| Encryption layer | its backing signature's id |
| Aggregate | technology, canonicalized native designator bytes |
| Volume / produced virtual device | producer id, the technology's own volume name and role bytes — never a volume UUID, which is regenerable and excluded; same-name-same-producer duplicates (legal in APFS) fall to the collision group |
| `BackingExtent` (new) | host file-system id plus canonicalized path bytes, or host id plus byte range |
| Multipath node | canonicalized platform-reported LUN designator bytes |
| `ConflictingTableEntry` (new) | host table id, view role, entry start offset |
| Collision group (new) | the shared name it absorbs |

Created and simulated nodes name positionally: a created partition
from its table and declared start offset, an initialized table from
its parent, a new file system from its host and declared kind and
offset, a new encryption layer from its backing. The residue that has
no position until it exists — volumes minted inside a container, LVM
snapshots — remains SI-19's, as round three assigned. A moved
partition renames: its new address is its new position, computed in
the simulated topology; addresses are not identities, and PLAN-006
compares bodies that a move legitimately changed.

### Canonicalization by source, not transformation

Round three's defect 6 was unnormalized serial text at the recursion
root — `naa.…`, `0x…`, and bare uppercase as three values for one
drive. The repair rides ADR-0018's contract instead of inventing a
normalization algebra: **an identifier used in naming is the byte
string returned by the one named source the evidence contract
designates for it, per platform, verbatim** — no case folding, no
prefix stripping, no re-encoding, non-UTF-8 bytes legal. Choosing the
single source is the normative act, and it makes divergence — round
three's own "worse than collision" — structurally absent: two
transformations cannot disagree when there is no transformation and no
second source. Where the named source is `unavailable`, the field is
absent, the name is weaker, and any resulting collision fails closed
through the group below. The per-platform source designations are
versioned with the contract; changing one is hash-visible by
construction and lands only with a spec change.

### Collision groups: the artifact a collision produces

When two or more same-kind nodes derive equal naming bytes, the body
contains — by domain-layer construction rule, before any encoding —
one **collision group** node carrying the shared name, the observed
count, and an indeterminate marking; the colliding nodes' children
attach under the shared name, recursively subject to the same rule.
The group is the address; no member is individually addressable; every
operand inside the group is `Indeterminate` under ADR-0018's closure —
`blocked`, the collision as cause, detach-or-disambiguate as
remediation — while detection, display, and diagnostics continue, the
envelope carrying per-OS-node attribution (INV-007) that the body
honestly cannot.

- **This is not the forbidden merge.** The filing forbids *silent*
  merging. The group preserves two-ness in the count, declares
  indeterminacy, blocks every covered operand, and attaches the
  limitation to exactly the colliding targets. The governing finding's
  whole-host denial cannot occur, because the body always encodes.
- **It aligns with rules already normative.** Equal stable identifiers
  with no platform-assembled node are already SAFE-005 ambiguity,
  `blocked` (Section 2.1, ADR-0011). The measured byte-identical L9
  pair — every UUID-keyed symlink farm collapsing to silent
  last-writer-wins — is this population; the group is that ambiguity's
  representation, where the platform's own surfaces measured silent.
- **Individually distinct addresses for byte-identical simultaneous
  devices require an excluded input** — connection path, instance id,
  or arrival order — and each is excluded because it renames on replug
  or churn. This is the wall rounds two and three each hit from a
  different side; the group is the design that stops pretending
  otherwise. Accepting it was the round's named decision, and it was
  accepted.
- **MODEL-006's duplicate rejection never fires.** The domain layer
  absorbs equal names before encoding, so set encoding is total. A
  Section 11.4 fuzz target asserts the whole property: **no on-disk
  byte sequence can make snapshot encoding fail.**

The duplicate-designator case — a cloned pool or VG deriving one
designator — forms a group flagged duplicate: **nothing
re-designates** (defect 8 closed), children keep their addresses
because the group carries the same shared name, and operands block
with the platform's own disambiguation as remediation
(`vgimportclone` and kin) — the honest FS-008 duplicate-UUID posture.
An aggregate whose native designator is unreadable derives a
designator-absent name, is `Indeterminate`, and is not a plan operand
— round four's recorded precondition-2 consequence, adopted; no
member-derived naming exists anywhere (defect 4's withdrawal
honored), so an EIO on one member renames nothing.

### The address property, stated and tested

**A node's address depends only on that node and its ancestors, never
on the presence of other nodes.** Round three violated this twice and
declared neither. Derivation is per-node; grouping is a deterministic
construction over the derived multiset whose output address equals
what each member derived alone — an arriving clone changes a count
and a flag, never an address. A property test generates topologies
and asserts that adding or removing any non-ancestor node changes no
existing address.

### The host-backing edge and the `BackingExtent` node

Host-backed virtual devices — loop, dm-linear, plain dm-crypt,
VHD/VHDX, attached images — have no on-disk signature and had no
legal edge, which is also why CONC-001's bind set was empty for a
loop device. The new **host-backing** edge (semantics class: "the
bytes of A live within B") runs from a `BackingExtent` node — a file,
named from its host file system plus canonicalized path bytes, or a
byte range, named from its host plus range — to the virtual device it
backs. Path bytes in the body are body-stability-clean: a rename is a
directory write, a storage change, and it invalidates dependent plans
through CONC-003 rather than silently re-binding — priced
deliberately, the honest alternative to an inode-shaped regenerable
input. The bind set's reverse traversal picks the edge up by
semantics class with no restatement, so a plan writing to a loop
device binds through file to file system to partition to physical
device, and the concurrent-with-imaging hole closes. This is also
what makes the design testable by its own fixtures — defect 2:
equal-size loop devices collided under round three's device
projection; here each names from its distinct backing file, so the
T1/T2 tiers' multi-member fixtures address cleanly.

### The table, its views, and conflicting entries

The partition table is one node per device at MODEL-002's chain
position, carrying ADR-0014's helper-authored state, ADR-0018's
explicit primary/backup/MBR/EBR extents, and a **role discriminant**
for views. Partitions re-parent onto the table, which restores
parent-plus-offset injectivity: the hybrid case's aliased extent is
two entries under two roles — two addresses, no collision (defect 5
closed). Entries that alias or contradict across views materialize as
`ConflictingTableEntry` nodes holding the entries verbatim, marked
indeterminate — INV-008's representation and REC-003's home — and
ADR-0018's closure scopes the consequence: a step whose affected set
reaches conflicting evidence refuses; steps elsewhere on the same
body construct. Recovery-scan candidates never enter a captured
topology body; they belong to REC-003's preview artifact. The
partition type field takes a form able to carry APM's 32-byte ASCII
type string; type remains excluded from naming, so PART-008 renames
nothing.

### The platform-membership edge, typed — and only typed

ADR-0011's deferred edge is **platform-membership** (semantics class:
platform-asserted composition; detection-only). It runs from the
multipath node — named from the platform-reported LUN designator via
the canonicalization rule — to its member representation: a counted
member group under the multipath node, with the kernel's per-path
detail in the envelope. Individual path addressing is the path-set
encoding ADR-0011 defers to the spec change that first makes
multipath writable; this ADR does not preempt it. A member path as a
capability target resolves to the member group — `unsupported` with
the multipath reason, ADR-0011's rule falling out of addressability.
The class is closure-inert and bind-inert in v1, stated so its later
activation is a decision.

### The theorem, re-proved under the extended edge set

The no-sibling-capture premise — no backing or production edge
targets a physical device — extends: host-backing targets virtual
devices only; platform-membership targets the member group and is
traversed by neither the closure nor the v1 bind set. **The theorem
is re-proved as a property test whose topology generator includes
both new edge kinds** — ADR-0018's first handover discharged as a
test, not an argument.

### Preconditions 2 through 4

- **The native designator table (precondition 2):** the Linux rows
  exist (L7: mdraid array UUID and LUKS2 UUID client-readable; LVM2
  VG id and ZFS pool GUID helper-only). The Windows rows (Storage
  Spaces pool object id, LDM group GUID) and the macOS APFS container
  UUID row are evidence obligations below; until each exists,
  aggregates of that technology on that platform derive
  designator-absent names and are `Indeterminate` non-operands —
  fail-closed, nothing blocked from typing.
- **`probe_tag` (precondition 3):** discharged by ADR-0018's byte
  layer — each parser is fixed by family, offsets, magic, and
  validation, and naming's (family, primary offset) fields are read
  from that same contract. No separate probe_tag artifact exists to
  drift.
- **Preserved-unknown budgets (precondition 4):** depth capped at 4;
  size capped at 32 KiB (two LUKS2 16-KiB copies, the recorded
  floor-setter); the over-cap outcome is normative — truncation
  recorded as truncation, with original length and digest, so INV-008
  stays auditable; and the SAFE-006 redaction rule is versioned, so a
  redaction change is a visible body-hash event.

## Options considered

### Individually distinct addresses for indistinguishable devices

Rejected as physically unavailable: every distinguishing input —
connection path, OS instance id, arrival order — is excluded because
it renames on replug or churn, which is the recorded failure of both
prior rounds. The collision group replaces the pretense.

### Member-derived aggregate naming

Withdrawn by round three (defect 4) and not readmitted: an EIO on one
member renamed the aggregate, every volume, and every mount, on the
failing-disk population the product exists for.

### Round three's fold

Rejected as round three's review directed: not closed under the
naming relation, dependent on other nodes, blind to three of the four
collision families.

### Volume naming by UUID

Rejected: volume UUIDs are regenerable (PART-016, FS-008) and the
exclusion list already bars regenerable identifiers; the technology's
own name-and-role bytes are stable-under-reprobe and rename only on
an explicit storage change, with the group absorbing legal duplicates.

### Deferring the `BackingExtent` node to SI-19 or WP-R100

Rejected: without it, CONC-001's bind set stays empty for the T1/T2
fixture population — the tiers the product tests on — and round
three's record already named the addition as required, with ADR-0018
handing over the semantics-class slot it fills.

## Decision

**SI-27 moves to Resolved.** Node identifiers are derived,
kind-discriminated positional addresses computed from
contract-read fields per the naming maps above, canonicalized by
named source, recomputed at every decode by the schema-validation
pass. Equal derived addresses collapse into counted, flagged,
indeterminate collision groups that always encode, whose operands are
blocked pairwise and whose construction never changes an address.
Section 5 gains `BackingExtent`, `ConflictingTableEntry`, and the
collision-group construction rule; MODEL-002 gains the host-backing
and platform-membership edge kinds with their semantics classes; the
theorem is re-proved under the extended edge set as a property test.
The multipath path-set encoding remains deferred exactly as ADR-0011
left it. SI-19's created-node residue and SI-28 are untouched.

## Consequences

The normative amendments landing with this ADR in spec 11.1.0, each
inside the reservation's grant:

- **Section 5's type list** gains `BackingExtent` and
  `ConflictingTableEntry`, with the collision-group construction rule
  noted beside them (additions).
- **MODEL-002's modelling list** gains the two edge kinds with their
  semantics classes and the theorem-premise note (additions).
- **LIN-006's** deferred-edge-kind clause gains its promised pointer
  to this ADR.
- The document-control version row and §0.3 changelog row.

Other consequences:

- **Positive:** increment 3 can write `NodeId`, the edge types, and
  the snapshot schema against a decided rule; every recorded
  collision family has a named, encodable representation; CONC-001's
  loop-device hole closes.
- **Negative, accepted knowingly:** body-level member attribution is
  lost inside a collision group when contents diverge below equal
  device names (the envelope carries it for display; no covered node
  is an operand); a backing-file rename invalidates dependent plans
  through CONC-003; serial-less same-size devices block each other
  while simultaneously attached — priced against SI-28's own finding
  that typed confirmation is no discriminator for that pair; and a
  crafted equal-identity device blocks its twin pairwise, which is
  SEC-002's refusal working, with detach as remediation.
- **Evidence obligations (operator sitting work, recorded here):**
  (1) the Windows designator rows — Storage Spaces pool object id and
  LDM group GUID, helper-side; (2) the macOS APFS container UUID row
  (byte layer / named IOKit keys); (3) per-platform
  backing-designator rows for host-backed virtual devices — which
  API, what bytes, double-capture stability, behavior under rename
  and re-attach; (4) per-platform named-single-source rows for serial
  and WWN bytes with replug/reboot stability — L2/L8 cover Linux;
  Windows and macOS owed.
- **Obligations forward (increment 3):** the ancestor-only address
  property test, the collision-totality fuzz target (no on-disk byte
  sequence fails snapshot encoding), the theorem re-proof with both
  new edge kinds in the generator, and the stale-pair two-address
  regression (ext4 at `0x438` and end-anchored mdraid 0.90 as two
  addresses on one host).

## Verification

- When increment 3's types land: the property tests and fuzz target
  above; decoder rejection of unknown referents and of any stored
  identifier; the collision-group construction exercised on the L9
  byte-identical shape and the duplicate-designator clone shape.
- Register: SI-27 reads Resolved; the increment-3 gate holds SI-28
  alone; any text implying the path-set encoding was decided, or that
  a sameness inference exists anywhere, is an error against this ADR.

## Revisit conditions

- The spec change that first makes multipath a supported write target
  — the path-set encoding and the platform-membership class's
  closure/bind activation land there, per ADR-0011.
- A platform is measured where the named identifier source is
  systematically absent for a mainstream device population — the
  collision-group cost would then bind the common case, and the
  source designation deserves re-examination.
- SI-19's round finds a created-node class the positional residue
  rule cannot name — its filing already owns that residue.
- Any proposal to store a node identifier, admit an excluded input to
  naming, or let a collision surface as an encoder error.
