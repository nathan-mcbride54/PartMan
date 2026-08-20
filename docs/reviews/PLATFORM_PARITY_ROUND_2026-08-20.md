# The platform-parity round — what must be identical across Windows, Linux and macOS

**Date:** 2026-08-20. **Base:** `aa84400` (main), spec 20.0.0.
**Directive:** Nate — take the next slice, on the Windows-parity
assessment accepted the same day.
**Question:** issue #597. No requirement governs which properties MUST
be identical across the three platforms and which are legitimately
platform-specific qualifications. The Codex review of 2026-08-20 lists
it as cross-cutting item 8, *"they need explicit decisions before
storage mutation"*; it has no SI number, no ADR, and no scheduled round.
This is that round.

> Committed session record. `docs/reviews/**` is in WP-000's
> `owned-paths` block and lands in its own `Work-Package: WP-000`
> commit, never bundled with code. Nothing below is decided; §5 is for
> the decision owner. The recommendation prices as **no spec change and
> no ADR yet**, and §5.4 says why that is the finding rather than a
> deferral.
>
> **Not a §1.11 item.** The register's scope test is
> requirement-versus-requirement. This is the *absence* of a
> requirement, not a conflict between two — the same test ADR-0054's
> changelog row states verbatim: *"No §1.11 item: no
> requirement-versus-requirement pair conflicts; the tension was
> requirement-versus-delivery, which is what the round and this ADR are
> for."*

## 0. The texts the round works under

- **§13's exit rule** already carries its own escape hatch: a milestone
  exits only when its criteria pass on all three platforms *"(or the
  milestone explicitly scopes fewer)"*. **M1** requires inventory
  correct against fixtures on all platforms with zero elevation
  anywhere; **M3** requires helpers on all platforms with HLP-003
  demonstrated on each OS.
- **HLP-003** is the spec's own template for a cross-platform
  obligation, and it is one requirement rather than three: *"Linux —
  polkit `auth_admin` without retained grants; macOS — documented
  authorization APIs with a per-apply prompt; Windows — a fresh
  administrative consent bound to the plan hash (mechanism fixed by
  ADR-W1)."* One invariant obligation, three named mechanisms, one ID.
- **PLAN-009** is the spec's one worked parity rule, and it is worth
  quoting for its *shape*: *"The dry run refuses exactly where and how
  apply would; the pipeline's internal gate order is the
  implementation's, and sameness of the refusal pair is the tested
  property."* Pin the observable pair; leave the internals to the
  implementation.
- **INV-003** (`AGENT_BUILD_SPEC.md:553`) already levels down: *"The
  unprivileged layer emits no partition-table state **on any
  platform**"* — uniformity reached by withdrawing a capability
  everywhere because it was unreachable somewhere.
- **INV-003's bound on declination** (`:554`): *"An unprivileged layer
  MUST NOT refuse a write on the ground that its own contract cannot
  reach a state, and MUST NOT represent its inability as a
  determination either way."*
- **MODEL-005's envelope rule** (`:428`): *"A field belongs in the
  envelope only if it is the hash itself, or the privileged helper
  independently re-derives it and treats the client's copy as an
  untrusted hint (HLP-002). Every other field belongs in the body."*
- **§9's floors** license runtime narrowing and already ship *unequal*
  guarantees per platform; **§0.2** holds that an ADR *"MUST NOT weaken
  any MUST."*
- **§7.5/7.6/7.7** are **not** parallel blocks. WIN has 11 entries, LIN
  10, MAC 10, and they do not pair up: LVM2, mdraid, APFS stores, Boot
  Camp and Fast Startup exist on one platform only, and the nearest
  triple (WIN-005 / LIN-003 / MAC-004) demands materially different
  things of each. There is no obligation-by-obligation correspondence
  to make identical.

## 1. What is measured

**The motivating hard case is already decided, and it is not a parity
defect.** This round was opened partly on the strength of a specific
worry, carried in issue #597 and in the assessment that produced it:
unprivileged Windows can read `MSFT_Disk.PartitionStyle`, the disk GUID
and the full `MSFT_Partition` list, while unprivileged Linux can read
no partition table at all; therefore Windows client records would be
Weak, unattended apply would refuse on every Windows machine, and
client and helper would disagree on a body field for unchanged
hardware — the PLAN-006 failure ADR-C2 exists to prevent.

Two of those three limbs dissolve against `:553`. The unprivileged
layer emits no partition-table state **on any platform**, so:

- the client authors no table state anywhere, and therefore there is no
  body field for client and helper to disagree about, and no PLAN-006
  unsatisfiability; and
- Windows-Weak, to whatever extent it holds, is a uniform consequence
  of ADR-0014 across all three platforms, not a Windows asymmetry.

What remains is real but much narrower: a Windows adapter **could**
reintroduce the problem by authoring table state the Linux adapter
cannot, and nothing currently forbids it beyond `:553` itself. That is a
constraint on WP-W100, not an open architectural question.

**Where the asymmetry genuinely lives.** Not in observation, which the
spec already licenses to differ, but in three delivered places:

1. The helper wire types the authorizing principal as `u32`
   (`services/helper-linux/src/lib.rs:672`, `validate.rs:183,304`),
   which no Windows SID can inhabit. Filed as issue #598.
2. Roughly 1,600 lines of platform-neutral policy — SEC-002's admission
   arms, ADR-0021's ladder, the two-phase apply — sit inside a crate
   named for Linux, and the portable RPC framing sits in
   `crates/transport-linux`. Filed as issue #599.
3. The domain already carries Windows vocabulary no producer can reach:
   `IdentityClaim::WindowsPipeSddl` is the **first** arm of the shared
   identity enum; `StorageSpaces`, `Ldm`, `BitLocker`, `Ntfs` and `Refs`
   are first-class arms; `windows-11` and `windows-10` are the first two
   `PLATFORM_LABELS`. By this project's own standard a closed-enum arm
   no producer can falsify is an unverified claim.

**What is not measured, and matters.** There is exactly one discovery
adapter behind the model boundary (`crates/adapter-linux`, whose only
consumer is the helper). `apps/cli` carries two more with a different
seam shape. No cross-OS gate exists anywhere; every gate this project
calls a parity proof compares Rust against TypeScript.

## 2. What this round decides, and what it leaves alone

It decides whether a cross-platform parity **requirement** should be
authored now, and if so in what shape.

It does not take: ADR-W1's consent mechanism; the Windows discovery
route (WP-035's deferral of 2026-08-08 stands with its recorded revisit
condition); the principal representation on the wire (#598); or where
the neutral policy modules should live (#599). Each has its own owner
and its own act.

## 3. The options, each against the texts

Three candidate rules were developed independently, from deliberately
different angles, and each was asked to state its own weaknesses before
being judged.

**A — Refusal parity (proposed SAFE-010).** Observation may differ;
permission may not weaken. Every capability answer and plan outcome
projects onto a three-valued permission through one platform-neutral
function; platform identity may not be an argument to any function
computing a verdict; and — the operative clause — deleting observations
may never raise a permission cell or shrink an affected set.

**B — Model parity (proposed MODEL-007).** The observer is never in the
body. For one storage object in one state, the hashed body is the same
body on every platform; observation differences are absorbed below the
model boundary, into the envelope.

**C — Declared-contract parity (proposed PLAT-001…006).** The
*declaration* is what must match, not the answers. Every platform
publishes a total, schema-versioned contract declaration over an
identical question set with identical vocabularies; the answers may
differ freely, and every declination carries one of four closed classes
(`unreachable`, `deferred`, `pending`, `refused`), each with its own
evidentiary burden.

## 4. The adversarial pass

Three judges scored the candidates independently, on conformance to the
existing texts, on checkability, and on the hard case alone. They split
two-to-one — and, more usefully, each found a defect in the candidate
the others preferred. Both defects were re-verified against the source
before being recorded here.

**A collides with a MUST.** Clause 4 — a narrower contract refuses more,
never less — runs directly into `:554`: *"An unprivileged layer MUST
NOT refuse a write on the ground that its own contract cannot reach a
state."* Linux's blindness to the partition table is precisely such a
ground. A rule that needs a prior decision to resolve its own central
case cannot be the answer to that case.

**B inverts a MUST.** MODEL-007 §1 routes every non-shared-derived,
non-designation-covered field into the envelope. MODEL-005 says the
opposite in terms — envelope membership requires that *the helper
re-derives the field*, and *"every other field belongs in the body"* —
and §0.2 forbids an ADR from weakening a MUST. B also has no legal
state for `total_bytes`, the one unconditional field a physical
device's name carries, whose cross-platform byte-identity no sitting has
established.

**C is nobody's winner and the source of the best material.** Two judges
independently called its enforcement layer the highest-value graft on
the board, for a reason worth recording: each of its four decline
classes is *already instantiated in delivered code* —
`apps/cli/src/reach.rs:186` is a real `deferred` with an owner and a
2026-08-08 date, macOS's `Content` omission is a real `refused`, Linux's
empty table roster is a real `unreachable`. C's own stated weakness is
the honest one: a declaration can be total, versioned, diffable and
false, because the gate checks it against this repository rather than
against the platform.

**The finding that outlived all three candidates.** A's clause 3 —
*deleting an observation must never yield a more permissive answer* — is
not a parity rule at all. It is a safety-monotonicity property that
happened to be discovered while writing one, it is single-platform, and
it is mechanically checkable with machinery already delivered: `Facts`
is exactly four maps (`protection.rs:87`), `TopologySnapshot::assemble`
is the one constructor and validates internally, and no `FactError` arm
fires on absence — so every ablation yields a legal snapshot to
re-measure.

**And the class has already bitten once.** `crates/domain/src/model/capability.rs:189-199`
records a live fail-open of exactly this shape, found by hand: a volume,
for which `may_carry_extent` is false, *"had no range here at all, so
the closure never saw it destroyed… and `Wipe(volume)` gated `Clear`
over a live pool."* It was fixed by substituting the node's own frame.
A standing ablation gate would have caught it; nothing stops the next
one.

Three further sites share the shape — `crates/planner/src/lib.rs:707-721`
(an absent `table_state` makes `indeterminate_table_guard` pass rather
than refuse), `:735-745` (an absent `table_state` yields *no* protection
obligation), and `crates/domain/src/model/protection.rs:945-948` (a node
with no extent is skipped in the destroyed closure). **None is asserted
here as a defect.** `may_carry_extent` is false for `Aggregate`,
`MultipathNode`, `EncryptionLayer` and `Volume`, so absence is
legitimate for those kinds and the skip is correct. Whether absence is
reachable for a kind where it matters is exactly what the gate would
adjudicate, and is the argument for building it.

## 5. The recommendation, and the decisions for the owner

### 5.1 — Do not author a parity family yet

**Recommended: reject A, B and C as spec text, for now.** A and B each
weaken a MUST. C is adoptable and cheap, but it would be authored
against **one** adapter behind the model boundary, and a parity rule
validated by a single implementation is precisely the shape this project
refuses elsewhere: a claim no producer can falsify. C's teeth are in
forced writing-down, and with one adapter there is nothing to write down
that diverges.

### 5.2 — Build the monotonicity gate instead

**Recommended: file and build the ablation gate as its own act**, single
platform, Tier-1, no VM, independent of any parity decision. Build a
snapshot, compute the full (operation × node) permission matrix, delete
each deletable fact, rebuild through `TopologySnapshot::assemble`,
assert no permission cell rose and no affected set shrank. Count the
ablations `validate_facts` refuses and treat a growing skip count as a
signal rather than a pass.

This is the round's most valuable output and it is not about platforms
at all. It converts a class of bug the project has already suffered, and
found only by hand, into a standing property.

### 5.3 — Extend the declaration machinery, cheaply

**Recommended: adopt C's four decline classes into the existing INV-003
reach declaration**, without a new requirement family. Every class is
already instantiated in delivered code; today the schema collapses them
into a `distinguished`/`basis` pair, so macOS's deliberate `refused` and
Linux's structural `unreachable` are byte-identical cells. That
collapse is a real loss of information, it is cheap to repair, and
repairing it is what makes a later parity rule *derivable* rather than
authored.

### 5.4 — Why this is a finding and not a deferral

The honest answer to #597 is that the parity question is **less open
than it looked**. §13 fixes the milestone rule and its escape hatch,
HLP-003 fixes the shape for a cross-platform obligation, PLAN-009 fixes
the shape for a parity property, and `:553` already decided the case
everyone pointed at. What is missing is not a rule but a **second
implementation to write it against**. Author it when WP-W100 exists, in
HLP-003's shape — one obligation, three named mechanisms, one ID — and
it will be derived from measurement rather than asserted ahead of it.

### 5.5 — The decisions

1. Reject A, B and C as spec text now, or take one anyway.
2. Build the monotonicity gate as its own WP-010/WP-060 act.
3. Extend the reach declaration with the four decline classes.
4. Defer the parity requirement to WP-W100's arrival, written in
   HLP-003's shape.
5. Confirm that #597 is amended rather than closed: its premise about
   Windows reading more is corrected by `:553`, and its remaining
   content is the WP-W100 constraint in §1.

## 6. What would change this round's mind

- **A second adapter landing behind the model boundary.** The moment
  WP-W100 exists, C becomes derivable rather than authored and should
  be revisited immediately.
- **A measured case where two platforms' designated sources disagree on
  a body field for the same object.** That would make B's problem real
  and urgent rather than prospective, and would force the designation
  question ahead of the parity question.
- **The ablation gate finding a live violation.** If deleting an
  observation does raise a permission cell somewhere reachable, A's
  clause 3 stops being a nice property and becomes a defect report,
  and its collision with `:554` must then be resolved rather than
  noted.
- **A decision on #598 that keeps a platform-shaped principal on the
  wire.** That would make the asymmetry structural rather than
  incidental, and would change what a parity rule has to cover.

## 7. Next acts, in order

1. This record (WP-000), and the amendment to issue #597.
2. The monotonicity gate, filed and built — the only act here that does
   not wait on anything.
3. The reach declaration's decline classes (WP-L100 owns
   `schemas/adapter-linux/reach.md`; the CLI's roster is WP-035's, so
   this is two acts).
4. #598 and #599, in that order, before increment 4b hardens the wire.
5. WP-W100, and the parity requirement derived from it.
