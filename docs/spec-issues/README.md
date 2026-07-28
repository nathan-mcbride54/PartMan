# Specification issues

Section 1.11 and Section 0.2 of `AGENT_BUILD_SPEC.md` require that when two
requirements conflict, work stops and the conflict is filed, rather than an
implementer silently picking a side. This directory is where those filings live.

Each entry states the requirements that disagree, why the disagreement cannot be
settled by reading harder, the options, and what the choice costs. **None of
them proposes an answer as though it were decided.**

## How these were found

Seven parallel readers extracted the Section 5 domain-type requirements from the
specification during WP-010 increment 3, and three adversarial reviewers checked
the resulting design for completeness, safety, and encoding fidelity. Thirty
conflict reports and twenty-five design findings were deduplicated into the
twenty-six issues below.

## Why this blocked increment 3

Ten of these are **hash-visible**: the choice changes the canonical bytes of a
plan or topology snapshot. Under MODEL-005 and ADR-C1, changing a hashed
artifact's shape later is not a refactor — it invalidates every hash the product
has ever issued, and there is no migration for an authorization token that no
longer matches. Guessing now and correcting later is the one option with no
cheap exit.

## Legend

- **Blocks 3** — must be decided before WP-010 increment 3 writes the type.
- **Hash-visible** — the decision changes canonical bytes.
- **Later** — decidable before the work package named, not before increment 3.

---

# Part 1 — Blocking WP-010 increment 3

**Six resolved, five remain.** SI-03, SI-05, and SI-06 were one question — what
the hash authenticates — answered by ADR-C2 in spec 3.0.0. SI-01, SI-02, and
SI-04 are answered by ADR-C3 and ADR-C4 in spec 3.1.0.

Still blocking increment 3: **SI-07 through SI-11**, plus **SI-27**, discovered
during the second protection attempt. Round one is recorded in Part 4; round two
and its result are in Part 5.

The approach is settled — protection is proven by computation and the verdict is
frozen into the hashed body — and round two established that the platform
asymmetry which threatened it does not bite. What remains is mechanism.

## SI-01 Identity strength is not computable at discovery time

> **Resolved in spec 3.1.0 by ADR-C3** — strength is a property of one record; matching is a separate helper-side verdict.

**Requirements:** SAFE-003, INV-002, Section 6 · **Blocks 3, hash-visible**

SAFE-003 defines strength as a comparison outcome — a stable hardware identifier
"plus size, sector geometry, and partition-table checksum all **match**". But it
also requires that "each identity record MUST be classified", INV-002 requires
reporting strength at discovery, and Section 6 requires plans to carry bound
identities with strength. In all three there is no counterpart to match against.

Either strength at discovery means "a stable hardware identifier and the other
fields are **present**", and the matching clause describes only the helper's
re-probe; or the field is meaningless before re-probe. The two readings produce
different plan bytes for the same device.

**Options:** (a) presence-based at discovery, match-based at re-probe, with the
two named distinctly; (b) strength is re-probe-only and absent from discovery
and plan bytes — which contradicts INV-002 and Section 6 as written.

## SI-02 Blank and table-less media can never be Strong

> **Resolved in spec 3.1.0 by ADR-C3** — option (b): partition-table state is three-valued, so a positively observed absence is determined and a blank device can be Strong, while an unreadable table cannot. The proposal to also exempt blank-media initialization from severity 4 was **rejected**; see the ADR.

**Requirements:** SAFE-003, INV-003, PART-001 · **Blocks 3**

Strong requires a partition-table checksum match. INV-003 requires representing
missing tables and PART-001 requires initializing blank media, where no checksum
exists. Read literally, a factory-blank NVMe exposing both serial and WWN is
Weak, so every first-initialization takes the weak-identity path: typed
device-name confirmation (UI-009) and refusal of unattended apply.

**Options:** (a) an absent table is a vacuous match, so blank media can be
Strong; (b) it is a distinct third case with its own policy; (c) the literal
reading stands and first-initialization is deliberately high-friction.

This decides whether `PartitionTable.checksum == None` is identity-degrading.

## SI-03 Is provenance inside the hashed identity bytes?

> **Resolved in spec 3.0.0 by ADR-C2** (`docs/adr/0002-hashed-artifact-body-and-envelope.md`).

**Requirements:** MODEL-004, MODEL-005, SAFE-003, HLP-003 · **Blocks 3, hash-visible**

MODEL-004 requires *every* discovered property to record source adapter and
confidence. Identity fields are discovered properties, and they are covered by
the plan hash. The specification never says whether the provenance metadata is
inside the hashed bytes.

If it is, a plan authorized while one adapter reported the device rehashes
differently after an adapter or version change on unchanged hardware, breaking
HLP-003's binding. If it is not, a `conflicting` identity observation sits
outside the authorization boundary that SAFE-003 and SEC-001/002 establish.

**Must be decided before the identity encoding is frozen.** Changing it later
invalidates every hash previously issued.

## SI-04 MODEL-004 cannot express its own `conflicting` value

> **Resolved in spec 3.1.0 by ADR-C4** — option (a): provenance is a set of observations held in the envelope, with the four confidence values derived rather than stored. The proposal to also collapse disputed body values to a single resolution bit was **rejected**; see the ADR.

**Requirements:** MODEL-004, INV-007, SAFE-005, UI-010 · **Blocks 3, hash-visible**

MODEL-004 requires each discovered property to record "its source adapter"
(singular) while permitting the confidence value `conflicting`, which by
definition means two or more adapters disagreed. A single-source envelope cannot
say which sources disagreed or what each reported. SAFE-005 requires ambiguous
identity to disable the affected write, which needs to know what disagreed, and
INV-007 requires raw evidence to be inspectable.

**Options:** (a) the provenance record holds multiple `(adapter, value)`
observations, changing the shape MODEL-004 states; (b) `conflicting` is an
unexplained flag and losing observations live only in the diagnostic bundle,
outside the hash.

## SI-05 A plan cannot contain its own hash

> **Resolved in spec 3.0.0 by ADR-C2** (`docs/adr/0002-hashed-artifact-body-and-envelope.md`).

**Requirements:** Section 6, MODEL-005, ADR-C1, HLP-001, SEC-001 · **Blocks 3, hash-visible**

Section 6 requires `OperationPlan` to contain the "Cryptographic plan hash",
while MODEL-005 defines that hash as SHA-256 over the plan's canonical bytes. A
field cannot simultaneously be inside the bytes and be the hash of those bytes.

The model needs an envelope/body split — hash outside, everything else inside —
that neither Section 6 nor ADR-C1 states. **Which Section 6 items belong to the
body versus the envelope is undecided and load-bearing**, because HLP-001
applies by plan hash and SEC-001 authorizes only exact hashes.

## SI-06 What is inside a TopologySnapshot's hash?

> **Resolved in spec 3.0.0 by ADR-C2** (`docs/adr/0002-hashed-artifact-body-and-envelope.md`).

**Requirements:** MODEL-005, PLAN-006, HLP-004, CONC-004, MODEL-004 · **Blocks 3, hash-visible**

PLAN-006 requires the helper to re-discover topology and reject a mismatch,
which implies comparing a fresh capture's digest against the recorded one. But
MODEL-005 hashes the whole artifact's canonical bytes, and that artifact
includes CONC-004's transitional marking, MODEL-004 provenance, and any capture
timestamp.

If those are inside the hash, two captures of a physically identical topology
never produce equal digests and the freshness check can never pass. If they are
outside it, the hash does not cover the whole artifact and MODEL-005's
single-canonical-encoding guarantee weakens.

The specification does not separate topological content from capture metadata.

## SI-07 StorageContainer, StoragePool, and RaidSet have no boundary

**Requirements:** Section 5, MODEL-002, MAC-003, MAC-010 · **Blocks 3, hash-visible**

Section 5 lists all three and defines none. MODEL-002 lumps "Storage Spaces,
LVM, RAID, APFS containers, and Btrfs multi-device file systems" together as
non-linear relationships. Nothing says whether an LVM volume group, an mdraid
array, or a Storage Spaces pool is a container, a pool, or a RAID set — so the
same membership edges can be modelled twice, and the choice is hash-visible.

MAC-003 additionally requires APFS physical stores (plural, many-to-one for
Fusion per MAC-010), which a one-to-one `StorageContainer` cannot express.

## SI-08 Btrfs multi-device: container, or file system with many backings?

**Requirements:** MODEL-002, FS-003, LIN-006, MAC-003 · **Blocks 3, hash-visible**

MODEL-002 places file system strictly above volume, yet requires representing
Btrfs multi-device file systems, where one file system spans several devices and
performs the aggregation role a container performs for APFS. APFS gets an
explicit container type; Btrfs gets none.

## SI-09 FS-004 detects things that are not file systems

**Requirements:** FS-004, MODEL-002 · **Blocks 3, hash-visible**

FS-004 requires detecting "LVM PV, Linux RAID, LUKS, BitLocker, ZFS pool
members, Storage Spaces, LDM metadata" under file-system operations, while
MODEL-002 places encryption and containers on layers distinct from file system.

Either `FileSystemKind` enumerates non-file-system signatures, breaking the
mandated layering, or FS-004 results are materialized as
`StorageContainer`/`EncryptionLayer` nodes. The answer changes both the schema
and every snapshot hash.

## SI-10 The `Snapshot` type has no defined scope

**Requirements:** Section 5, Section 20, MAC-003, LIN-004, PART-015, FS-003 · **Blocks 3**

Section 5 requires a `Snapshot` type, but Section 20 defines only "Snapshot
(topology)", which is `TopologySnapshot`. The storage-level type is named and
never specified. Explicit requirements exist for APFS (MAC-003), LVM2 (LIN-004),
and Apple signed system snapshots. Windows VSS is implied by PART-015 naming a
VSS store as a shrink-limit cause; Btrfs snapshots are implied by nothing,
despite FS-003 and LIN-006 requiring Btrfs support.

Whether `SnapshotKind` must cover VSS and Btrfs is a specification question.

## SI-11 Is non-goal protection a type-level impossibility or a runtime guard?

**Requirements:** Section 2.1, PART-014, Section 20, Section 0.2 · **Blocks 3**

Section 2.1 says the product MUST NOT mutate ZFS, Storage Spaces, LDM, or
Fusion — absolute. The mechanism it supplies, PART-014 protected objects, is
defined in the glossary as refusal "without an explicit supported plan", which is
bypassable by construction; and PART-014's enumerated list does not include pool
members, ZFS, Storage Spaces, or LDM at all. Section 0.2 grants override
authority to Section 3, and Section 2.1 is not in Section 3.

This decides whether a detect-only marking makes a plan step naming such a node
**unrepresentable**, or merely rejected at runtime. The two are not
interchangeable, and only the first survives a bug in the guard.

---

# Part 2 — Blocking later work packages

## SI-12 Multipath devices and the single connection path

**Requirements:** SAFE-003, LIN-006, INV-001 · **Later (WP-L100), hash-visible**

SAFE-003 models one connection path per identity record and makes a path change
the special case for removable replug. LIN-006 requires detecting multipath and
device mapper, and INV-001 requires hardware RAID LUNs — devices that
legitimately present several concurrent paths. Whether the bound record holds
one canonical path, an ordered set, or an unordered set is unstated, and a
differing path count or order would make an unchanged device compare unequal at
re-probe.

## SI-13 Identity binding for pool and array write targets

**Requirements:** SAFE-003, LIN-004, LIN-005, UI-009, ACC-014 · **Later (WP-L110)**

SAFE-003 enumerates device-level identity fields only. LIN-004 and LIN-005 make
pools and arrays direct write targets possessing none of them. Whether such a
plan binds the union of member identities, the pool UUID, or both — and how
strength is classified for an aggregate whose members differ — governs whether
the weak-identity path applies to an mdraid grow.

## SI-14 Derived properties have no confidence rule

**Requirements:** MODEL-004, INV-004 · **Later (WP-050)**

INV-004 lists free extents and alignment among properties to detect, bringing
them under MODEL-004's provenance requirement, but both are computed from other
provenanced properties. None of the four confidence values describes a derived
value, and no rule composes a derived property's confidence from its inputs.

## SI-15 Pre-existing misaligned partitions

**Requirements:** PART-009, PART-004, PART-005, Section 11.2 · **Later (WP-060)**

PART-009 permits alignment deviation only when published geometry requires it or
the user explicitly overrides. A legacy MBR partition at a non-1 MiB offset
grown at its tail matches neither cause, yet realigning it forces a data move the
user did not request.

## SI-16 Backup-before-first-write on blank or corrupt media

**Requirements:** PART-013, PART-001, REC-001, INV-003, SAFE-005 · **Later (WP-060)**

PART-013 requires backing up table metadata before the first table write, and
Section 8 routes backup failure to Failed. On blank media there is nothing to
back up; when restoring a damaged table the backup source is precisely what is
unsound. Whether an absent or corrupt prior table satisfies PART-013 vacuously,
requires a journaled acknowledgement, or blocks decides whether the operations
intended to repair a table are fail-closed against themselves.

## SI-17 Severity 1 versus the `irreversible-after-start` flag

**Requirements:** PLAN-004, PLAN-005, UI-009, HLP-003 · **Later (WP-060)**

PLAN-004 declares the flags orthogonal to severity, but severity 1 is "fully
undoable ... via an emitted reversal plan", which `irreversible-after-start`
directly negates. The flag is never defined, nor is its relationship to
PLAN-005's `non-cancellable` class — cannot-stop and cannot-undo may or may not
be the same thing. Since UI-009 and HLP-003 key off severity plus flags, the
model cannot silently decide whether the combination is legal.

## SI-18 Does a severity-1 plan need fresh authorization?

**Requirements:** SAFE-002, HLP-003, Section 0.2 · **Later (WP-040)**

SAFE-002 confines privileged behavior to a helper "executing a validated plan
after fresh, explicit user authorization", which reads as every privileged
execution. HLP-003 requires fresh interactive authorization only for severity
≥ 2. A severity-1 plan still writes storage and still needs privilege. Section
0.2 gives Section 3 precedence, pointing to SAFE-002, but the two are written in
contradiction, and the answer decides whether the plan carries an
authorization-requirement field distinct from severity.

## SI-19 A reversal plan has no snapshot to bind to

**Requirements:** PLAN-008, PLAN-002, PLAN-006, HLP-004, Section 6 · **Later (WP-060)**

PLAN-008 requires the planner to emit a reversal plan at planning time. Section 6
requires every plan to carry a source topology snapshot hash, and PLAN-006
requires the helper to reject a mismatch — but the topology a reversal plan runs
against does not exist yet and has only a simulated snapshot. Whether a reversal
plan binds the simulated final topology, is emitted unbound and re-planned after
apply, or is exempt decides whether `OperationPlan` is recursive.

## SI-20 RecoveryRequired has no exit in the transition table

**Requirements:** Section 8, REC-009 · **Later (WP-070)**

Section 8 states recovery actions "are themselves plans under this same
contract", but the table moves the *original* plan directly
`RecoveryRequired → Executing`. A recovery action that is its own plan has its
own lifecycle and hash; if a second plan executes, the original stays in
RecoveryRequired, for which the table provides no exit — there is no
`RecoveryRequired → Completed` or `→ Cancelled`.

## SI-21 Resume and roll-forward reuse an authorization HLP-003 forbids

**Requirements:** HLP-003, HLP-005, Section 8, WIN-009 · **Later (WP-070)**

The table reaches `Executing` from `RecoveryRequired`, and `Protecting` from
`Revalidating` after `RebootPending`, without passing `AwaitingAuthorization`.
So a roll-forward or post-reboot resume writes storage under an authorization
granted before the interruption — possibly after the helper exited, which
HLP-005 permits. WIN-009 suggests reuse is intended; that is exactly the
retained grant HLP-003 forbids.

## SI-22 Journal retention can delete what recovery depends on

**Requirements:** JRN-001, JRN-003, JRN-004, SEC-009, Section 8, SAFE-005 · **Later (WP-070)**

JRN-004 requires bounded journals with retention controls; JRN-003 requires
recovery state to derive solely from the journal; Section 8 requires
RecoveryRequired to persist until the user acts, which is unbounded in time.
Nothing exempts records belonging to a non-terminal plan, so retention can delete
the records recovery needs and SAFE-005 then fails closed on a plan the product
itself is holding open. How rotation preserves JRN-001's monotonic sequence and
torn-tail semantics is also unstated.

## SI-23 The encryption-metadata backup artifact has no protection owner

**Requirements:** REC-011, SAFE-006, JRN-004, JRN-005, REC-001 · **Later (WP-R100)**

REC-011 requires backing up encryption-layer metadata — explicitly the LUKS
header, which contains key slots — before mutating that layer. SAFE-006 and
JRN-005 forbid key material in logs, plans, journals, and UI state, but nothing
says where this artifact lives, how it is protected, or whether it inherits
JRN-004's admin-protected location. It cannot be discarded, because RecoveryAction
must reach it.

## SI-24 CAP-003 `preview` versus PLAN-009 dry-run parity

**Requirements:** CAP-003, PLAN-009, HLP-002, CAP-007 · **Later (WP-050)**

CAP-003 says preview permits "planning and simulation" while apply is refused.
PLAN-009 says a dry run traverses the identical pipeline including helper
revalidation, and that success means only physical outcomes remain uncertain.
A dry run of a preview-backed plan must therefore either fail (contradicting
"simulation permitted") or succeed while apply is still guaranteed to be refused
for a non-physical reason (contradicting PLAN-009).

## SI-25 CAP-002's operation list does not span the operation surface

**Requirements:** CAP-002, DIA-004, DIA-005, PART-007, PART-010, PART-011 · **Later (WP-050)**

CAP-002 enumerates fourteen operations including a single `wipe`, but DIA-005
requires overwrite, crypto-erase, sanitize, format, discard, and file deletion to
be distinguished and "never call them equivalent". Modelling them as one `wipe`
violates DIA-005; adding variants exceeds CAP-002's list. Separately PART-007 and
PART-010 map to no CAP-002 operation at all. Whether the list is a closed
enumeration or a required minimum is unstated.

## SI-26 `Stable` is not a capability status

**Requirements:** Section 16, CAP-003 · **Later (WP-050)**

Section 16 forbids marking a capability "Stable" without matrix fixture and
acceptance evidence, but `Stable` is not one of CAP-003's four values
(`supported`, `preview`, `unsupported`, `blocked`) and appears nowhere else.
Either it is a stale synonym for `supported`, in which case Section 16's evidence
rule attaches there, or `Capability` needs a maturity axis orthogonal to status.

---

# Part 3 — Design findings, not specification conflicts

These are decisions within the implementer's authority. They are recorded so
they are not rediscovered, and will be resolved when increment 3 is written.
They do **not** require a specification change.

- **Set ordering must be over encoded bytes, not a discriminant.** A
  discriminant is a total order only if it injectively determines the element,
  which is false for several proposed sets. Sorting each element's fully
  canonical encoding and rejecting a non-increasing successor is total by
  construction. *(MODEL-005)*
- **Node identifiers must be a pure function of stable content**, not of
  discovery arrival order, or a re-probe of unchanged topology produces
  different bytes and every Draft plan spuriously invalidates. *(MODEL-005,
  CONC-003)*
- **A label that is valid UTF-8 must have exactly one representation.** A
  `Utf8 | Raw` pair where both are legal for the same bytes lets two adapters
  disagree on an unchanged label. *(MODEL-005)*
- **Normalization belongs to adapters, never to the decoder.** A decoder that
  NFC-normalizes repairs rather than rejects, which is the malleability class
  ADR-C1 forbids. *(`schemas/canonical-encoding.md` §6)*
- **Version expressions must not be free text**, and need a fixed arity, or
  `2.9` and `2.9.0` become one version with two encodings inside the plan hash.
  *(MODEL-005, Section 6)*
- **Preserved unknown structure needs its own depth and size budget**, well
  below the profile limit, since it is the one field with externally controlled
  depth — and must not become a path for key material to enter a hashed
  artifact. *(INV-008, SAFE-006)*
- **No anonymous tuples in hashed types**, and no `Vec` where a set is meant.
  *(MODEL-005)*
- **`Null` must be either banned everywhere or permitted explicitly**; the two
  rules cannot both hold, and Rust and TypeScript will resolve it differently.
  *(MODEL-005)*
- **A secret must be structurally unable to reach a plan.** A reference type
  that cannot carry material, and cannot print it, is the only shape that makes
  SAFE-006 a property rather than a review item. *(SAFE-006, LIN-003, WIN-005,
  MAC-004)*
- **Detect-only must propagate to members, not only upward.** Marking a ZFS pool
  while leaving its member partitions mutable protects nothing. *(PART-014,
  Section 2.1)*
- **Copy-as-source is not mutation.** A binary per-node policy cannot express
  LDM's "migration off dynamic disks only via copy to basic disks". *(WIN-004,
  Section 2.1)*

---

# Part 4 — Analysed, not yet decided

Two of the four remaining decisions were analysed and their proposals rejected
by adversarial review. The *direction* of each survives; the mechanism does not.
Recorded so the next attempt starts from the objection rather than from scratch.

## SI-07 to SI-10, the aggregation vocabulary

**Direction that survived.** One aggregation node type carrying a technology
discriminant, rather than three disjoint types. Btrfs multi-device is a file
system with several backings, not a container. FS-004's non-file-system
signatures are materialized as container and encryption nodes rather than
enumerated into `FileSystemKind`, which preserves MODEL-002's layering.

**Why it was not accepted.** The proposal's load-bearing justification was that
detect-only status would be a total function over the closed kind enum. It is
not, on the first non-goal it maps: an Apple Fusion container and an ordinary
mutable APFS container carry the same kind and differ only by member-set shape
(MAC-010). Detect-only is therefore a function over kind *and* members — the
coarse-kind failure the proposal listed as a future risk, occurring immediately
against a Section 2.1 MUST NOT. Splitting the kind by instance shape contradicts
the proposal's own rule that the discriminant is the technology.

**What the next attempt needs.** Either a detect-only predicate defined over kind
and membership rather than kind alone, or a reason why membership-derived
protection is safe to compute outside the hashed body.

## SI-11, protection strength for non-goals

**Direction that survived.** Detect-only should be enforced structurally rather
than only at runtime, and it must propagate to members, not merely upward:
marking a ZFS pool while leaving its member partitions mutable protects nothing.
Read-as-source must be distinguishable from mutation, so Section 2.1's permitted
copy-based migration off dynamic disks (WIN-004) stays possible.

**Why it was not accepted.** The proposal removed Apple sealed volumes from
PART-014 and required mutating operations on a detect-only target to report
`unsupported` and never `blocked`. That contradicts MAC-009, which makes sealed
volumes protected objects and requires exactly `blocked`, with a stated reason,
for operations macOS permits only in Recovery. The proposal amended neither, so
merging it would have shipped a fresh Section 1.11 conflict on day one. The
reasoning underneath was also wrong: `blocked` means a runtime precondition
failed and the user has a next step, and "boot into Recovery" is precisely that.

**What the next attempt needs.** A status mapping that keeps MAC-009 intact,
distinguishing "this product will never do this" from "this platform will not
permit it right now".

---

# Part 5 — Protection model, round two

The decision owner chose the approach: **protection is proven by computation,
not asserted**, with the derived verdict frozen into the hashed body so it is
authenticated and a client/helper disagreement fails closed.

A second design attempt under that constraint **validated the approach and
failed on mechanism**. Both are recorded.

## What the round established, and it is the important part

The open worry was that an unprivileged client and a privileged helper might see
membership differently, making a frozen verdict diverge on unchanged hardware —
the same structural failure that hashing capture metadata caused for PLAN-006.

Per-platform investigation (including empirical, non-elevated testing on
Windows) found the asymmetry is real but **lands where it does not matter**:

> Every asymmetric fact is a *roster-identity* fact — which pool, which group,
> which volume group. No protection verdict in this product needs roster
> identity.

- **Windows LDM.** The group GUID and member roster are Administrator-only. But
  Section 2.1 forbids in-place LDM editing *uniformly*, so the group is not a
  verdict input. The per-disk GPT type that decides it reads unprivileged.
- **Windows Storage Spaces.** Pool membership is readable unprivileged through
  `MSFT_StoragePoolToPhysicalDisk` — verified on this machine from a
  non-elevated session, contradicting the widespread claim that it needs
  Administrator. A hardened host can revoke it by namespace ACL, but the member
  disk also carries its own GPT type, which is not policy-gated.
- **Linux ZFS and LVM.** An exported pool's vdev tree is root-only, and a VG
  roster with no active LV is invisible. Neither is a verdict input: ZFS is
  uniformly detect-only, LVM is uniformly supported. `ID_FS_TYPE=zfs_member` is
  in the world-readable udev database.
- **macOS Fusion.** The *only* case where member-set shape decides a verdict —
  and macOS is the one platform where shape is fully readable unprivileged.

The Fusion counterexample that killed round one and the asymmetry that
threatened round two are **disjoint**. Shape is needed exactly where it is
symmetric. That makes the chosen approach viable, and it is a fact about the
platforms rather than a design choice, so it holds regardless of what is built
on top.

## Why round two was still rejected

**1. Protection propagated through containment into unrelated siblings.** The
closure treated "disk contains partition" and "aggregate has member" as one
relation. On the standard root-on-ZFS layout — `zpool create tank /dev/sda2` —
protection flowed from `sda2` up to `sda` and back down to `sda1`, making the
**EFI System Partition immutable on every such machine**, which kills LIN-007
bootloader repair and REC-006 for that entire population. The same composition
fires on macOS: a Fusion store at `disk0s2` freezes `disk0s1`.

Containment is not membership. Protecting a ZFS member must not protect its
sibling ESP. The propagation rule has to distinguish the two edge kinds.

**2. The hashed body would contain a graph, and nothing names its nodes.** This
design is the first to freeze cross-references — backing edges, containment
edges, divergence reports — into the body. Every obvious naming scheme breaks
ADR-C2's requirement that identical hardware produce equal digests:

- Path-derived names are unstable across reboot, replug, and enumeration race.
- Positional indices are worse under INV-009 device churn.
- Names derived from the identity record inherit SAFE-003's *connection path*,
  so a replug changes the name — breaking the one case SAFE-003 explicitly
  blesses.
- Names derived from a path-free subset **collide** for exactly the population
  ADR-C3 calls Weak: two identical blank USB sticks behind bridges with no
  serial. Two nodes, one name, and the edge relation silently merges them.

That is a new blocking gap, filed below as SI-27.

## SI-27 The hashed body has no node-naming rule

**Requirements:** MODEL-005, ADR-C2, SAFE-003, ADR-C3 · **Blocks 3, hash-visible**

Until now the body was a bag of values with no internal references, so node
identity never had to be canonical. Any protection closure, and any faithful
representation of MODEL-002's layering, puts edges in the body and therefore
needs a `NodeId` that is stable across re-probes of unchanged hardware and
collision-free across simultaneously present devices.

Section 5 lists no such type and neither ADR-C2 nor ADR-C4 addresses naming.
The four obvious schemes each fail, as above. A resolution must state the
derivation, its stability guarantee, and its behaviour when two Weak-identity
devices are indistinguishable — including whether an indistinguishable pair is
an error that fails closed rather than a silent merge.
