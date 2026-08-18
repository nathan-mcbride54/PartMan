# Issue #1004 round — mounts, `NamingFields`, and what "unrepresentable" means here

**Date:** 2026-08-18. **Base:** `625b329` (main), spec 17.2.0.
**Directive:** Nate — "draft the #1004 recommendation round".
**Issue:** gitea#1004, "WP-010: WP-L100's increment-3 scope names mounts, and
`NamingFields` carries no mount variant", filed by WP-L100 increment 3a.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §4 is for the decision owner.

## 0. The filing's premise, and the half of it that does not survive

#1004 offers two branches: **(1)** a mount kind in the domain model with its
naming map and endpoint-pair rows, or **(2)** a recorded decision that
mounts are not topology nodes in v1, with WP-L100's scope statement
corrected. It grounds the gap on §5 listing `Mount` among the domain's
concepts, "which is what makes this a gap rather than an intended
omission".

Measured, the choice between the branches was made in **4.0.0**, and the
§5 listing does not point where the filing reads it as pointing.

**The premise that survives:** WP-L100's increment-3b sentence names
mounts among things "assembled into a `Topology`", `NamingFields` has
eleven variants and none is a mount, `EdgeKind` has five and none is a
mount edge, and `crates/domain` contains **no** occurrence of "mount" at
all. Every one of those measurements holds at `625b329` (§1). The scope
statement does name something the model cannot carry.

**The premise that does not:** that this is an *open* modelling question
WP-010 owes an answer to. It is a **decided** one, and the answer is
branch (2) — with a different reason from "v1", and with a corrected
account of what §5's `Mount` entry requires.

## 0.1 What the adversarial pass changed, kept rather than erased

Two independent verifiers attacked the round's load-bearing claims
against the delivered code and the spec text before it was landed.
**The central claim — that a mount cannot be a topology node without
amending MODEL-005 — stands under nine attacks**, including the closest
one (MODEL-002's "→ mount" layer) and every spec line and ADR that names
a mount. What did not survive as first drafted:

| draft said | measured |
| --- | --- |
| D2 "entered under Governance, so should leave under one" | A preference, not an AGENTS.md rule. Precedent runs both ways: `02f34f6` corrected the assignment's identity half under `Governance:`; `33be9e1` rewrote this very paragraph under `Work-Package: WP-L100`. Reworded as a preference with the reason. |
| D3 "the two envelope-side §5 entries are tracked nowhere" — as if singular | True, but `DeviceHealth`, `RecoveryAction`, `ExecutionJournal`, `ExecutionResult`, `OperationRequest` are equally untracked and equally absent from `crates/`. D3 now names the whole pending list and says why the envelope pair is the part worth pinning. |
| §3.3 "ADR-0005's mount/unmount obligation is satisfied vacuously" | Understated: **none** of Rule 2's three evidence arms has a test; `envelope_edits_never_move_the_body_hash` edits only timestamp and provenance. Now stated at full scope as a WP-010 residue. |
| §6 "the model distinguishes a mount … by refusing it node status" | Wrong reading of MODEL-002's "MUST distinguish"; the layer is distinguished by the envelope-side `Mount` type, and until it lands the layer is *undelivered*, not misplaced. Reworded. |
| `FreeExtent` cited as "delivered as a derivation (ADR-0033)" | It has no type at all; the delivery is `free_extents()` in the planner and the Linux adapter's derivation module. Precedent restated as "no `NamingFields` variant", which is the property that matters. |
| Six line citations | `EdgeKind` `:20-33` → `:24-53`; endpoint pairs `:353-373` → `:351-375`; `snapshot.rs` range missed `assemble` at `:97-104`, the line that proves nodes are `NamingFields`; ADR-0018 `:70-74` → `:72-75`; two WP-L100 spans. Corrected. |

Added on the verifiers' prompting: CONC-003's mount-change invalidation
cannot ride PLAN-006 (§3.1) — named now so nobody files it as a §1.11
conflict later.

## 1. What is delivered and decided, measured at `625b329`

| fact | where |
| --- | --- |
| `NamingFields` has exactly eleven variants: `PhysicalDevice`, `PartitionTable`, `Partition`, `BackingSignature`, `FileSystem`, `EncryptionLayer`, `Aggregate`, `Volume`, `BackingExtent`, `MultipathNode`, `ConflictingTableEntry` | `crates/domain/src/model/naming.rs:210-301` |
| `EdgeKind` has exactly five: `Containment`, `Backing`, `Production`, `HostBacking`, `PlatformMembership` | `crates/domain/src/model/topology.rs:24-53` |
| The endpoint-pair table admits no pair whose either end is a mount | `topology.rs:351-375` |
| `grep -rn -i mount crates/domain/src` returns **nothing** — no type, no field, no test, no comment | measured |
| Every topology node is a `NamingFields` value, and the node set is hashed body content; the envelope carries only `capture_timestamp` and MODEL-004 provenance, and the one envelope test holds only that editing those two never moves the body hash | `crates/domain/src/model/snapshot.rs:60-80`, `:97-104` (`assemble(nodes: Vec<NamingFields>, …)`); `snapshot_tests.rs:163-185` |
| **MODEL-005's body-stability rule:** "Occupancy figures, **mount sets**, and storage-snapshot sets therefore belong to the **envelope**: they change without any storage change, through ordinary background activity." | `AGENT_BUILD_SPEC.md:427` |
| ADR-0005 (ADR-C5, 4.0.0), Rule 2: "`FileSystem.size`/`free`, `Volume.size`/`reserve`/`quota`, **the `Mount` set**, and the `StorageSnapshot` set are **envelope** content" | `docs/adr/0005-…:280-286` |
| ADR-0005's rejection record: the PART-014 class is computed by the helper "from live discovery including mount path, partition flags, and label, none of which need be body content. **This is also what removes `Mount` and the active-swap flag from the hashed body**" | `docs/adr/0005-…:345-350` |
| ADR-0005's own evidence obligation: "Two probes of one fixture separated by … a mount/unmount cycle produce **equal** body hashes" | `docs/adr/0005-…:463-466` |
| ADR-0018: verdict inputs are restricted to body-stable facts; "mount state, active swap, health, and tool availability feed Regimes B and C — reasons and runtime gates — never the verdict"; the evidence contract puts Linux boot identification on "mount path `/boot`, `/boot/efi` (**state layer**)"; "Regime B never enters a verdict, so none of this is body content — which also keeps mounts and swap state out of the hashed body" | `docs/adr/0018-…:72-75`, `:359-365` |
| ADR-0019 names nodes by **derived positional address**, never stored; a mount path is not a position and moves under autofs/snap units with no storage change (ADR-0005's own example) | `AGENT_BUILD_SPEC.md:367-369`; `docs/adr/0005-…:301-302` |
| §5 requires "serializable, versioned types for" a list that includes `Mount` — **and also** `FreeExtent`, `StorageSnapshot`, `Capability`, `PlanRisk`, `ExecutionJournal`, `RecoveryAction` | `AGENT_BUILD_SPEC.md:327-353` |
| `FreeExtent` is a §5 entry with **no type and no `NamingFields` variant**, delivered as a derivation (ADR-0033) at `free_extents()` and the Linux adapter's derivation module; `StorageSnapshot` — the other Rule 2 envelope set — is, like `Mount`, absent from `crates/` and `schemas/` entirely | `crates/planner/src/solve.rs:614`; `crates/adapter-linux/src/derivation.rs:119`; measured |
| WP-050's engine names mount state as a CAP-001 conditioning input and deliberately carries no field for it: "No decided text consumes them yet … a field the engine reads nothing from would represent conditioning that does not happen. Each arrives with the text that decides its rule" | `crates/capability/src/engine.rs:33-39`; `docs/work-packages/WP-050.md:84`, `:165` |
| WP-L100's scope sentence entered with the assignment itself (`af4a889`, Governance, 2026-08-12), reading INV-004's detection list — "volumes, file systems, encryption, **mounts**" — into a `Topology` sentence; increment 3a (`33be9e1`, `Work-Package: WP-L100`) restated it as "3b, not started" with the mount clause intact, and filed this issue against it | `git log -S`; `docs/work-packages/WP-L100.md:566-567`, `:419-424` |
| WP-L100 increment 4 already owns "active root/boot/swap dependencies" as its **detection layer**; the adapter contract's published exclusion list names "no mount table, no swap table" as increment 4's material | `WP-L100.md:577-584`; `schemas/adapter-linux/fields.md:162-167` |
| WP-010's own record has **no row** for `Mount` or `StorageSnapshot` — nor for `DeviceHealth`, `RecoveryAction`, `ExecutionJournal`, `ExecutionResult`, `OperationRequest`; of §5's list, only the delivered types appear. Undelivered §5 entries are tracked nowhere in the package record | `grep -in 'mount\|storagesnapshot' docs/work-packages/WP-010.md` → nothing; measured |

## 2. What the record already decides

**A mount is not a topology node, and cannot be made one without amending
MODEL-005.** Nodes are hashed body; the body-stability rule puts mount
sets in the envelope; ADR-0005 removed `Mount` from the hashed body in so
many words and made "a mount/unmount cycle produces equal body hashes" a
verification obligation. A `NamingFields::Mount` with a naming map and
endpoint-pair rows would put the mount set back into the body and make
that obligation unsatisfiable on any systemd host. **Branch (1) is refused
by decided text**, and it would owe a *major* spec change (a MODEL-005
sentence becoming false), not a WP-010 increment.

**§5's `Mount` entry does not say otherwise.** §5 is a list of
serializable, versioned *types*, not of topology *node kinds*: it names
`FreeExtent` (a derivation), `PlanRisk`, `Capability`, `ExecutionJournal`.
`Mount` sits in that list beside `StorageSnapshot`, its Rule 2 sibling.
What §5 requires is a typed `Mount` record — undelivered, like
`StorageSnapshot` — whose *placement* ADR-0005 has already fixed as
envelope/state-layer content. The filing's inference "§5 lists it,
therefore a node kind is missing" is the same shape as #371's "the grep
found no delivery, therefore no vehicle exists": true premise, stronger
conclusion than it supports.

**What "unrepresentable" therefore means.** WP-L100's sentence is wrong
not because the model lacks a type it should have, but because it puts a
state-layer fact into the body's node set. The correction is to the
sentence, and the sentence's own package already has the right home for
the fact — increment 4's detection layer.

## 3. What is genuinely open

Three things, none of them "should mounts be nodes":

1. **The carrier for the §5 `Mount` type.** Two non-hashed places exist:
   `SnapshotEnvelope` (ADR-0005's literal placement — "the `Mount` set is
   envelope content") and WP-050's `RuntimeFacts` (CAP-001's "mount
   state" input, WP-050:84). They are not exclusive — the envelope carries
   what discovery observed; `RuntimeFacts` carries what the engine
   conditions on — but each is a field nothing reads today, and
   `engine.rs:33-39`'s discipline applies to both: it arrives with the
   text that consumes it. The consumers on record are CAP-001 mount-state
   conditioning (WP-050), ADR-0018's Regime B Linux-boot identification
   (state layer, WP-L100 increment 4 → WP-050 reasons), CONC-003/INV-005
   invalidation on mount change, PART-008's mount-point change, and
   UI-003's display. Whichever lands first fixes the shape. One
   consequence is worth stating now so it is not later mistaken for a
   §1.11 conflict: CONC-003's "mount changes" invalidation **cannot ride
   PLAN-006** — body hashes are equal across a mount cycle by ADR-0005's
   own obligation — so it lands through INV-005's event path, which
   is where INV-005 lists "mount, unmount" separately from "topology
   changes".
2. **The reference from a mount to what it mounts.** A mount names a
   file system (or a volume/partition acting as one) by its ADR-0019
   address, and that reference is one-way — envelope pointing into body,
   never body pointing out. This is the recompute-at-decode discipline
   ADR-0022 uses for step-output references, and it is unowned text
   today.
3. **ADR-0005's Rule 2 evidence obligation is unwritten, not just its
   mount arm.** None of its three arms — file-system writes, a new local
   snapshot, a mount/unmount cycle — has a test in `crates/`; the nearest
   is `envelope_edits_never_move_the_body_hash`
   (`snapshot_tests.rs:163-185`), which edits only the timestamp and
   provenance. The mount arm is satisfied vacuously today because no
   mount fact exists to leave out; when the `Mount` type lands, that arm
   must be written with it, or the delivery is exactly the "shell of a
   discipline" the WP-035 rule refuses. The other two arms are a WP-010
   residue independent of this issue, named here so it is not
   rediscovered.

## 4. The recommendation

**Branch (2), on ADR-0005's authority rather than as a new v1 decision.**
Concretely, and in this order:

- **D1. No new ADR, no spec change, no `NamingFields` or `EdgeKind`
  variant.** The decision exists (ADR-0005 Rule 2, MODEL-005:427,
  ADR-0018:70-74). Restating it in a fresh ADR would be the "decision
  already made" duplication this repository's ADR practice avoids; the
  round record and the issue's closing comment cite the sentences.
- **D2. Correct WP-L100's 3b sentence** to "Partitions, volumes, file
  systems, and encryption layers assembled into a `Topology`", and add
  mounts to increment 4's detection layer beside "active root/boot/swap
  dependencies", stated as **state-layer facts under MODEL-005's
  body-stability rule, never topology nodes**, reported through the
  adapter's observation surface and consumed by WP-050 once the §5
  `Mount` type exists. Either trailer is legal: `WP-L100.md` is in
  WP-L100's `owned-paths`, and the record has precedent both ways —
  `02f34f6` corrected the assignment's identity half under `Governance:`,
  while `33be9e1` rewrote this very paragraph under `Work-Package:
  WP-L100`. The round's preference is `Governance:`, because the change
  moves a deliverable between increments rather than reporting status;
  either way, its own PR before any increment-3b or -4 work relies on the
  corrected reading.
- **D3. Give WP-010's record a row for the undelivered §5 entries**,
  with `Mount` and `StorageSnapshot` marked *envelope-placed by ADR-0005,
  arriving with their first consumer*. Today no undelivered §5 entry is
  tracked in the package record — `Mount`, `StorageSnapshot`,
  `DeviceHealth`, `RecoveryAction`, `ExecutionJournal`,
  `ExecutionResult`, `OperationRequest` alike — which is how a package
  could name one as body content without anything noticing. The
  envelope-side pair is the one that carries a *placement* decision worth
  pinning; the rest is a plain pending list. Docs-only, WP-010's own
  file.
- **D4. Close #1004 as answered by ADR-0005**, with the correction
  landing under D2, and the residues in §3 named on the issue rather than
  left to be rediscovered. Item 3.1 (the carrier) is not filed as its own
  issue: it has no consumer to file against, and filing it would schedule
  a field nothing reads.

**Pricing.** No spec text moves; no ADR; two documentation PRs (one
Governance, one WP-010) and an issue closure. No Rust, so no WP-020
sitting is owed by anything here.

## 5. Open questions for the decision owner

1. **Governance or Work-Package for D2?** Both are legal and both have
   precedent (`02f34f6` vs `33be9e1`); the round prefers `Governance:`
   because a deliverable moves between increments, but that is a
   preference, not an AGENTS.md rule.
2. **Is D3's row wanted, or is "absent from `crates/domain`" the record
   the house prefers** for a type whose consumer does not exist? The
   `engine.rs` discipline argues for naming the absence; the row is that
   naming.
3. **Should the closing comment on #1004 pin the carrier now** (envelope
   for observed mounts, `RuntimeFacts` for conditioned mount state), or
   leave it to the first consumer as D4 proposes? Pinning it costs
   nothing today and could be wrong; leaving it costs a future round.

## 6. What would change this round's mind

- A decided text, missed here, that requires a mount **in the hashed
  body** — that would make MODEL-005:427 and ADR-0005 Rule 2 a genuine
  §1.11 conflict rather than a scope-sentence error. Every occurrence of
  "mount" in `AGENT_BUILD_SPEC.md` (lines 95, 147, 293, 341, 396, 427,
  497, 550-552, 559, 576, 596, 603, 628, 649, 681) and in `docs/adr` was
  read for that shape; none has it (`:972`, WP-035's "mounts nothing",
  is the one line outside that list, and is about the instrument).
  MODEL-002's chain "→ mount" lists layers the model must *distinguish*;
  the mount layer is distinguished by the envelope-side §5 `Mount` type
  referencing its file system (§3.2), not by a node kind — ADR-0005
  materialised signature nodes on MODEL-002's layers in the same act that
  put the mount set in the envelope, so decided text already reads that
  layer as non-body. Until the §5 type lands, MODEL-002's mount layer is
  undelivered, not misplaced.
- A consumer landing that needs the mount set *authenticated* — which
  would be evidence, under Rule 2's own last sentence, that "the wrong
  fact was chosen", not grounds to move it into the body.

## 7. Next acts, in order

1. Decision-owner call on D1–D4 and §5.
2. Governance PR correcting WP-L100's 3b/4 text (D2).
3. WP-010 docs PR adding the envelope-side §5 row (D3).
4. Close #1004 citing this round; leave §3's residues on it.
