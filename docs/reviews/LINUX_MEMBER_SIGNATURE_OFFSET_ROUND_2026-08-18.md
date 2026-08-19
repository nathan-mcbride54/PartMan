# The Linux member-signature offset round — whether the client may build a `BackingSignature` it cannot read, and what carries a member into the draft instead

**Date:** 2026-08-18. **Base:** `1ba2702` (main), spec 17.4.0.
**Directive:** Nate — "draft the member-signature offset round".
**Question:** WP-L100 increment 4b's remainder — member `BackingSignature`s
and the `Backing` edges they would carry, and the `EncryptionLayer` over a
whole-device LUKS signature — needs, per member, a `NamingFields::BackingSignature
{ host, family, primary_offset }`. DR14 (`docs/quality/observability.md`,
the DR11–DR14 sitting, VMID 9471) measured that the **family** is
client-readable through two interfaces and that **no client interface
reports a signature's primary offset**. May the client draft build the
node anyway — authoring the offset from the family, or from a fixed
table — and if not, what representation carries "this whole disk is a
member of something" into the client draft so that a member is not
proposed as a plain operand?

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §4 is for the decision owner. Where
> the recommendation revises a plan item already accepted (§3.5 of
> `docs/reviews/WP-L100_INCREMENT_4_PLAN_2026-08-18.md`), it says so.

## 0. The premise, and the texts the round works under

Four texts fix what a signature node may rest on, and one fixes what
happens to a member without one:

- **ADR-0018, the two-layer contract** (`docs/adr/0018-si11-protection-closure.md:79-117`).
  Signatures are the **byte layer's**: "the helper's own bounded parsers
  over raw device bytes … mdraid 0.90 and 1.x superblocks, LUKS1/2
  headers, LVM2 label and metadata area … Each parser is bounded …
  fixed by family, offsets, magic, and checksum validation. **The layer
  is enumerating by construction: every family is probed at every
  defined location and every validated match is reported.**" The state
  layer is "named platform APIs for facts that are not on disk" and
  names, for Linux, "sysfs attributes and **the device-mapper/holders
  topology**" (`:100-104`; its example list — transport, multipath
  assembly, NVMe capability, read-only, removability — does not itself
  spell out md/LVM/LUKS assembly, so the holders topology is the Linux
  source and "assembly" is what it reports). And the join rule
  (`:111-117`): "evidence of a protected technology from **either**
  layer suffices for its refusing arm; `Permitted` requires every arm
  input positively determined or positively absent in its layer".
- **ADR-0019, the naming map** (`docs/adr/0019-si27-node-naming.md:78`,
  `:252-256`): a backing signature names from *host id, family, primary
  signature offset*, and "naming's (family, primary offset) fields are
  read from **that same contract**" — ADR-0018's byte layer — "No
  separate probe_tag artifact exists to drift." ADR-0019 is silent on
  who builds the node in a client draft (no occurrence of "draft" or
  "unprivileged" in it). The type carries the rule:
  `NamingFields::BackingSignature { host: NodeId, family: SignatureFamily,
  primary_offset: u64 }` (`crates/domain/src/model/naming.rs:239-246`),
  the offset a required unsigned field at decode (`require_unsigned(map,
  "primary_offset")`, `:453`). There is no offset-absent form.
- **ADR-0034, the designation discipline** (`:76-77`, `:132-140`):
  "nothing here designates by prediction"; the udev database refused as a
  naming source on three grounds — a cached third-party computation, a
  second source able to diverge from the direct read, a worse
  availability class.
- **ADR-0013 / SI-38** (`docs/adr/0013-…:74-84`): where the unprivileged
  layer's reach does not cover a table state, "the privileged
  re-discovery HLP-002 already requires before the first write"
  determines it, and the unprivileged layer is forbidden "both from
  refusing on the ground of its own blindness and from representing that
  blindness as a determination". **That sentence is scoped to INV-003's
  table states** (`:74-78`, `:91-95`); this round uses it as an
  *analogy* for signature nodes, and says so wherever it does. The
  8.0.0 precedent: the client draft carries no table-state entry, and
  the closure fails closed at the position the helper's authored value
  occupies (`WP-L100.md`, 3b's paragraph); ADR-0014 `:126-140` — "Drafting
  proceeds against the client's view as the proposal; validate-plan
  re-discovers under HLP-002, produces the authoritative snapshot … and
  binds its hash", and "client-only artifacts … are never plan-bound".
- **ADR-0018's member arms** (`:384-398`; `:523-529`): a validated
  signature whose consumer is *not* observed — "an unassembled member" —
  is `Indeterminate`, `blocked`, with a **release-acknowledgment** arm
  under which the step constructs; a member whose consumer *is* observed
  ("array assembled") is `Refused` and "its constructor has no
  acknowledgment parameter". And the *closure-verdict* option
  "unconditionally refused orphan signatures" was **rejected**: "a
  bench-tested disk pulled from a pool would be `unsupported` for
  initialization … forever, with the only escape an unsupervised
  external `zpool labelclear` — the hazard the product exists to
  prevent." Applying that rejection to a *draft-time withdrawal* is this
  round's analogy, marked as such in §2.

Two structural facts bound the question. The endpoint-pair table admits a
`Backing` edge **only** from a `backing-signature`
(`crates/domain/src/model/topology.rs:367-370`), so without a signature
node there is no membership edge, full stop; and `EncryptionLayer` names
from its backing signature's id (`naming.rs:257-260`), so without one
there is no encryption layer either. Whatever this round decides about
the node, it decides about the edges and the layer.

## 0.1 What the adversarial pass changed, kept rather than erased

An independent verifier attacked the draft's load-bearing claims against
the sources before it was landed. What did not survive as first drafted:

| draft said | measured |
| --- | --- |
| Option C costs "one round trip, no capability lost": the helper finds the orphan signature, refuses `Indeterminate` with the release remediation, and "the draft is re-planned with the acknowledgment" | **Refuted as stated.** The delivered acknowledgment is `Acknowledgment::Release { signature: NodeId }` (`crates/domain/src/model/step.rs:130-141`), lawful only when the covered node is *in the planning snapshot* with an `Indeterminate { OrphanSignature }` verdict (`step.rs:338-343`, `:448-462`); a node absent from the topology yields `Indeterminate { Unrecognized }` (`protection.rs:671-674`) and the step refuses `UnlawfulAcknowledgment`. So the acknowledgment **cannot** be recorded against a client draft that carries no signature node. The path that exists in text is a re-plan **against the helper's capture** — spec `:531` "refused at validation is re-planned against a fresh capture", ADR-0014 `:126-129` (validate-plan "produces the authoritative snapshot") — described for the reversal case and generically, **exercised nowhere yet**. §2 (Option C) and §5.1 now say exactly that, and Option A's one honest advantage (an authored node that happens to match lets the acknowledgment be recorded at first plan creation) is priced rather than omitted. |
| "the closure's own verdict for a consumed member is `Refused` with no acknowledgment (`0018:391-393`)", used to justify D2's held-is-not-an-operand | **True of the ADR, false of the delivered closure.** A `BackingSignature`'s own arm folds `worst` over its consumers' own verdicts (`protection.rs:1244-1263`), and a designated `Lvm2 \| Mdraid` aggregate's own arm is `Permitted` (`:1226`); `RefusalGround` (`:586-603`) has no consumed-member ground and `Facts` (`:80-99`) carries no assembled-state fact. ADR-0018 lists both as **forward obligations**: "(4) the consumed-versus-released discriminants measured, not recalled — … the assembled-state facts for mdraid" (`:601-603`) and "the consumed-member refusal" end-to-end at the first write-capable increment (`:610`). Consequence: a held member in the client draft is `Permitted` by today's closure at draft *and* validation; D2 delivers the Linux **input** for that arm (the holders topology ADR-0018 names), and the arm itself is a WP-010 act on an already-recorded obligation — §3.5 and D6. |
| "`slaves/` was read on the arrays each phase (transcript), `holders/` per member was not re-captured" | **The gap is both sides.** The DR11–DR14 client instrument reads neither `holders/` nor `slaves/` in any phase (transcript: zero hits); the DR1–DR10 instrument read both only in its baseline captures, not after the DR10 re-assembly. Neither side of the relation was captured after any re-assembly or reboot. §1's row and §3.1 (DR15) now say so, and DR15 captures both sides. |
| The stale pair L-F is "a live-ext4-over-stale-mdraid **disk**" | Partition-hosted: L-F is written to the partition of a GPT medium (`observability.md:4058-4064`). The finding stands for a whole-device host by the same mechanism (one `ID_FS_TYPE` per record); the row is now cited at its measured shape. |
| DR14 swept "every key of an md member's and a LUKS disk's udev record; sysfs" for an offset | Only the udev record was swept for `*OFFSET*` keys; sysfs contributed `md/metadata_version` alone. Corrected. |
| mdraid 1.2 → 4096, 1.1 → 0, 1.0/0.90 end-anchored; the LVM2 label "in any of the first four sectors" | In-repo for 1.2 (`crates/fixtures/src/signature.rs:19-20`), 0.90 (`:223-228`, `:270` "by the kernel's own formula") and the LVM2 label sectors (`:92`); **1.1 and 1.0 are external knowledge**, marked so. |
| ADR-0013's blindness sentence, cited in §0 as if general | Scoped to INV-003's table states (`0013:74-78`, `:91-95`); marked as analogy above and in Option A. |
| "no spec text" — LIN-006 as the detection duty | LIN-006 (`AGENT_BUILD_SPEC.md:622`) names dm, multipath, loop, Btrfs and root/boot/swap dependencies — **not** LVM/mdraid/LUKS members. The unscoped MUST-detect texts for those are **FS-004** (`:593`) and **INV-004** (`:552`); only INV-003 carries ADR-0013's privilege scoping. Nothing places FS-004 on the unprivileged layer (spec `:405`: signatures materialize "on the layer this chain assigns", layer-neutral), so Option C narrows no MUST — but the pricing now says why, and §5.4 asks whether the owner wants that stated normatively. |
| D2's "held: `PhysicalDevice` kept, `operand_eligible: false`" as if that flag reached anything | `operand_eligible` is set only from ADR-0034's failed-serial rule (`adapter-linux/naming.rs:338`), **dropped by `absorb_devices`** (`:368-376`), and consumed nowhere outside the crate. As typed, D2 would have no observable effect. D2 is re-shaped: the held standing is a **state-layer observation** on the adapter's reporting surface (the mount precedent, 4a) plus a distinct standing ground in the naming outcome — the input the closure's arm will consume — not a flag nothing reads. |

## 0.2 Post-landing correction (2026-08-19): the consumed-member arm was misread

Recorded as an addendum rather than edited into the text above, so the
round reads as it was taken. The round (§0.1's second row, §2 Option C,
§3.5, D2, D6) and gitea#1008 asserted that ADR-0018 `:391-398` *decides*
`Refused` for a consumed member and that the delivered closure fails to
implement it. Measured before shaping #1008 (2026-08-19), that premise
does not hold: the delivered signature arm — a member's verdict folds
`worst` over its consumers' own verdicts (`protection.rs:1244-1263`) — is a
**deliberate** increment-3 delivery, pinned by the reviewed,
requirement-tagged test `signature_arms_follow_the_consumer`
(`protection_tests.rs:518-563`, MODEL-002/SAFE-005: "a member consumed by
a supported aggregate is Permitted, and a member consumed by a non-goal
aggregate refuses"), and `RefusalGround::InheritedFromConsumerOrProducer`
is documented as "a consumed member of a **refused** consumer" (`:600`).
ADR-0018 admits two readings of the bullet: **(a)** consumed ⇒ `Refused`
(the literal reading the round took); **(b)** consumed ⇒ the consumer's
verdict, and what is refused is the *acknowledgment* route — the closure's
own examples derive refusal from the pool, never from consumption
(`:189-196`); the bullet's own last sentence ("an acknowledgment authored
against an orphan that validation finds consumed is a divergence and
rejects", `:397-398`) is delivered as `UnlawfulAcknowledgment`
(`step.rs:338-343`), which is `:610`'s "consumed-member refusal"; and the
product supports mdraid/LVM2 writes (LIN-005 member replacement, WP-L120's
M4 scope), which (a) would make unrepresentable on every live member.
Obligation (4) (`:601-603`) is about the consumed-versus-released
*discriminant* being measured, not the verdict; the delivered code decides
it by edge presence, DR15 is that measurement for Linux mdraid, and its
consumer is the **helper's capture (WP-L110)** deciding whether to emit
the aggregate node and edge — not a `Facts` field.

**Decision owner's call (Nate, 2026-08-19): reading (b).** Consequences
for this round: D6 is withdrawn and gitea#1008 closed with the finding;
D2's held report has no closure consumer to wait for — its consumer is the
capture; §5.1's fail-open-at-draft point stands unchanged (an unheld
orphan member is `Indeterminate { OrphanSignature }` at the helper, with
the release-acknowledgment re-plan as before); everything else in §4
stands. The WP-L100 and WP-035 records are corrected in the same act;
`held.rs`/`lib.rs` module docs carry the same sentence and are corrected
with the next Rust slice, which owes its own sitting.

Kept on the verifier's confirmation: ADR-0018 `:79-117`, `:384-398`,
`:523-529` as quoted; ADR-0019 `:78`, `:252-256`; the domain types at
the cited lines; ADR-0034 `:76-77`, `:132-140`; DR4, DR6, DR14, L1, L4,
L5, L6, L10 as quoted; the plan's §3.5 sentence; spec `:70` and `:459`;
ADR-0013 `:86-90` and SAFE-002 `:153`.

## 1. What is measured, per input the node would need

Every value is from a cited run — the DR11–DR14 sitting (VMID 9471,
transcript `20c0cee8…`), the DR1–DR10 sitting (VMID 9468, `89ce59ac…`),
or the increment 6 Linux matrix (L1–L10, real passthrough medium) — client
baseline, double-captured.

| Input | Interface / method | Measured | Cell |
| --- | --- | --- | --- |
| **family**, mdraid | udev record `ID_FS_VERSION` (cached, heuristic) | `1.2` on every md member | DR14 |
| **family**, mdraid | sysfs `md/metadata_version` on the **array** (direct) | `1.2` on both arrays — a property of the assembled array, readable only while it is assembled, and not on the member | DR14 |
| **family**, LUKS | udev `ID_FS_VERSION` on the disk (cached, heuristic) | `2` on both LUKS disks | DR14 |
| **family**, LVM2 | udev `ID_FS_TYPE=LVM2_member` (cached, heuristic) | on the three PVs | DR6 |
| **primary offset**, any family | every key of an md member's and a LUKS disk's udev record | **no key names an offset** — zero `*OFFSET*` keys; the full key lists are in the transcript. (Sysfs was not swept for one; the round knows of no sysfs attribute that carries one.) | DR14 |
| the client's **direct read** of device bytes | `dd` one sector; `blkid -p` | **denied** at the client baseline — stock `brw-rw---- root:disk` — and the same operations succeed for the `disk`-group user | L1, L6 |
| the cached view over a **stale pair** (live ext4 over an end-anchored stale mdraid 0.90 superblock, on a partition of a real medium) | udev `ID_FS_TYPE` | the single answer is **exactly the stale `linux_raid_member`**, the live ext4 absent; `ID_FS_AMBIVALENT` fired nowhere | L4, L10 |
| the helper's enumerating view over the same bytes | `wipefs -n` | **both** signatures; root `blkid -p` exactly the stale one — the intra-helper asymmetry ADR-0018 dissolves by enumerating | L5, L10 |
| a plain disk's cached view | udev `ID_FS_TYPE` | present with `ID_FS_TYPE=` **empty** — a positively determined absence | DR6 |
| **held-by**, a whole-disk member | sysfs `holders/` on the member (direct) | each md member's `holders/` names its array; each LUKS disk's names its `dm-N`; symmetric with the assembled node's `slaves/`; a PV that no active LV currently maps has **no** holder while its VG is active; Btrfs members have no holder | DR4 |
| **held-by** across re-assembly and reboot, either side | — | **not measured**: neither `holders/` on a member nor `slaves/` on an assembled node was captured after any re-assembly or the reboot, in either sitting (the DR11–DR14 client instrument reads neither; the DR1–DR10 instrument read both at baseline only) | (gap; DR15, §3) |

Two things follow before any argument. There is **no client source for the
offset** — not a cached one, not a direct one — so any `primary_offset`
the client wrote would be authored: from a family/version table (mdraid
1.2 → 4096, `crates/fixtures/src/signature.rs:19-20`; LUKS at the
device's start as the LUKS2 fixture writer places it; mdraid 1.1 at 0 and
1.0 end-anchored — the last two external knowledge, not in this
repository), or from arithmetic over the device size (0.90 is
end-anchored "by the kernel's own formula", `:223-228`, `:270`), or fixed
by fiat. And the LVM2 label is not at one offset by specification — "The
label lives in one of the first four sectors; sector 1 is conventional"
(`signature.rs:92`) — so for one of the three families no table exists to
author from. Second, the family source that *is* client-readable per
member is the cache (`ID_FS_VERSION`, `ID_FS_TYPE`); the direct one
(`md/metadata_version`) lives on the assembled array, is a fact about the
array's superblock format rather than a per-member read, and vanishes
with the assembly.

## 2. Each option against the texts

**Option A — build the node, authoring the offset.** The client draft
carries `BackingSignature { host: <the member's PhysicalDevice>, family:
<from ID_FS_TYPE/ID_FS_VERSION>, primary_offset: <authored> }` and a
`Backing` edge to the aggregate (or an `EncryptionLayer` over it). This
is what the increment 4 plan wrote in its §3.5 ("member
`BackingSignature`s from DR6's cached view (heuristic, single-valued —
the ADR-C3 finding stands)", `:184-187`), before DR14 asked the offset
question. Against the texts:

1. *The offset is not a read; it is a prediction of what the helper's
   parser will find*, and ADR-0019 says the field is "read from that same
   contract". Where the prediction is wrong — an mdraid 1.0 member, an
   LVM label in sector 2, a LUKS header the parser validates at a
   different copy — the client's node has a **different address** from
   the helper's node for the same on-disk signature (the offset is a
   naming field), so the draft carries a node that does not exist and
   omits the one that does. That is not "a proposal the helper
   corrects"; it is a fabricated identity in a hashed body.
2. *The family would enter a name from the rejected source class.*
   `ID_FS_TYPE`/`ID_FS_VERSION` are `udevd`'s cached run of `blkid` at
   event time — ADR-0034's first ground verbatim — and the L4/L10 rows
   measured the failure mode on a real medium: over a live-ext4-over-stale-mdraid
   partition the cache reports **only the stale member signature and no
   ext4 at all**. A client node built from it would name that host an
   mdraid member and hide its live file system — the SI-34 stale-pair
   hazard re-shipped at the draft, on the one interface that is
   single-answer by construction, where the helper's layer exists
   precisely to enumerate both.
3. *INV-008's enumeration is lost.* The byte layer reports "every
   validated match"; the cache reports one type per device. A client
   signature node is at most one per host, so a stale pair, a second
   LUKS copy, or a ZFS label set collapses to a single authored node or
   to nothing (`ID_FS_TYPE` empty for ZFS, L4).
4. *"Representing blindness as a determination"* — ADR-0013's sentence,
   by analogy (it is written for table states). The client cannot read
   the bytes (L1: `dd` and `blkid -p` refused). Emitting a node whose two
   naming fields it did not read is the same representation for a
   different fact; the honest analogue of 8.0.0's table-state rule is
   that the client emits no signature node and the helper's byte layer
   authors the ones that exist.

Its one honest advantage, priced (§0.1): where the authored node happens
to match the helper's exactly — the common mdraid-1.2-at-4096 and
LUKS-at-0 cases — a release acknowledgment could be recorded at *first*
plan creation, saving the round trip Option C costs. That advantage is
bought with a fabricated identity in every case that does not match and
with the stale-pair inversion in the case that matters most. Not
permitted on decided text; and the plan's §3.5 item is withdrawn by this
reading, recorded rather than quietly dropped.

**Option B — a new offset-less signature form** (a spec change:
`BackingSignature` with `primary_offset: Option<u64>`, or a
"signature-family-only" node kind). Rejected without a sitting: the
offset is what makes two signatures on one host two nodes (the stale
pair, both LUKS2 copies), so an offset-less node breaks the injectivity
ADR-0019 proves and cannot coexist with the helper's offset-bearing node
for the same bytes — the draft and the capture would carry different
node sets for one disk with no correspondence rule. And it would be
authoring the *family* from the cache still (point 2 above). The
"unrecognized" arms exist for values a platform ships that the product
does not know, not for facts the client did not read.

**Option C — no client signature node; membership as a state-layer
report; a held member's standing reported.** The client draft carries no
`BackingSignature`, no `Backing` edge, and no `EncryptionLayer` on
Linux; those enter the inventory only from the helper's byte layer at
HLP-002's re-discovery, which is where ADR-0018 already places them.
What the client carries instead is what it *does* read:

- **`holders/`** on each admitted whole device (sysfs, direct, DR4;
  named by ADR-0018 as the Linux state-layer membership source,
  `:103-104`). A whole device whose `holders/` positively names an
  assembled node is **held**: it stays a `PhysicalDevice` node under its
  designated name (it exists, and it is the host of whatever the helper
  will find), its holder is reported by selector as a state-layer
  observation keyed to its address — the shape 4a gave mounts, and for
  the same MODEL-005 reason: assembly changes under re-probe of unchanged
  hardware, so it is envelope, never body — and its naming outcome
  carries a `Held` standing distinct from ADR-0034's failed-read ground.
  A `holders/` listing that did not answer refuses the device, the
  `partition` discipline again; an empty listing is positively unheld.
  **What that standing does today**: it is reported. The closure's
  consumed-member refusal is not delivered — a designated mdraid or LVM2
  aggregate's own arm is `Permitted` and a consumed member follows it
  (`protection.rs:1226`, `:1244-1263`) because `Facts` carries no
  assembled-state fact — and ADR-0018 already lists the assembled-state
  measurement (`:601-603`) and the consumed-member refusal (`:610`) as
  forward obligations. The held observation is the Linux input for that
  arm; the arm is WP-010's (D6, §7).
- **The cached signature view** (`ID_FS_TYPE`, `ID_FS_USAGE`,
  `ID_FS_VERSION`, `ID_FS_UUID` where present) on each admitted device,
  **reported as an attributed observation** on the udev interface the
  adapter already classifies `Heuristic`/`inferred` — never a name,
  never a node, never a withdrawal. It is the client's best available
  early warning ("the cache says this unheld disk carries a
  `linux_raid_member` signature") and, per L4/L10, it is single-answer
  and can be wrong in the dangerous direction, which is why it decides
  nothing.
- **An unheld device stays operand-eligible in the draft**, whatever the
  cache says. This is the deliberate choice. Withdrawing an unheld disk
  on a cached `linux_raid_member` would remove it from every plan with
  no acknowledgment path — the client draft has no signature node to
  hang the release acknowledgment on — which is, by analogy at the draft,
  the closure option ADR-0018 rejected by name ("unconditionally refused
  orphan signatures", `:523-529`). Under Option C the plan names the
  disk; the helper's byte layer finds the orphan signature; the verdict
  is `Indeterminate { OrphanSignature }` with the remediation "record the
  release acknowledgment" naming technology, designator and consequence;
  and — **the correction from §0.1** — the acknowledgment is recorded on
  a re-plan **against the helper's fresh capture**, because
  `Acknowledgment::Release { signature }` is lawful only against a
  snapshot that carries the node (`step.rs:338-343`, `:448-462`). That
  re-plan is the path spec `:531` and ADR-0014 `:126-129` describe and
  nothing has yet exercised; it costs one round trip, and it is the same
  round trip Option A would cost in every case its authored offset
  missed.

Permitted on decided text at every point the round can find, and it
strengthens in the right direction: when the helper's capture arrives,
signature nodes and edges are *added*; nothing the client built has to be
withdrawn or re-addressed.

**Option D — hold 4b's remainder until a client byte source exists.**
There is none to wait for: SAFE-002 places discovery at no elevation
(spec `:153`), L1/L6 measured raw reads denied to the baseline and granted
only to the `disk` group, and ADR-0013 rejected requiring elevation for
an inventory (`:86-90`). Waiting is deciding Option C without saying so.

## 3. What is genuinely open

1. **`holders/` and `slaves/` stability and the deactivation edge, both
   sides.** DR4 measured the relation once, both directions, at baseline;
   no cell in either sitting captured `holders/` on a member or `slaves/`
   on an assembled node across `vgchange -an/-ay`, `cryptsetup
   close/open`, `mdadm --stop / --assemble` and the reboot, nor captured
   the **transition** — that a member's `holders/` is positively empty
   the moment its consumer is stopped and names the consumer again after
   re-assembly. Option C's held/unheld standing rests on that being a
   live kernel fact and not a cached one; it is one cell (**DR15**),
   capturing both sides each phase, including the stopped state and the
   active-VG-with-unmapped-PV case DR4 saw in passing (§5.3).
2. **The partial-VG case as a standing question, not a measurement.**
   DR4 already measured that a PV no active LV currently maps has no
   holder while its VG is active. Under Option C that PV is *unheld* in
   the draft although its VG is assembled — the helper's byte layer will
   find the LVM2 label and its consumer observed. Is a draft-time
   proposal that the helper refuses acceptable here, or does the LVM2
   case need the cache to demote eligibility? The round's answer is in §4
   (D3): the cache does not decide, because the alternative is the
   rejected option by analogy, and the helper closes it — but the
   decision owner should see the case named.
3. **The reach statement.** INV-003's published reach
   (`crates/adapter-linux/src/reach.rs`) is table-state only. Nothing
   requires the adapter to *publish* "signature nodes are helper-only on
   this platform" — FS-004 and INV-004 (`AGENT_BUILD_SPEC.md:593`,
   `:552`) carry the detect duty for LVM/mdraid/LUKS members and are not
   privilege-scoped the way INV-003 is; spec `:405` places signature
   nodes "on the layer this chain assigns", layer-neutral — but
   ADR-0013's spirit does, and `schemas/adapter-linux/fields.md` §4/§7 is
   where the roster says what the client carries. A sentence there and in
   the module docs is enough; a change to the reach payload's schema is
   not warranted (§5.2).
4. **What the second-slice `slaves/` report becomes.** `arrays.rs`
   reports each array's `slaves/` listing as an observation, "not an
   edge". Under Option C that stays true (no signature node, no edge) and
   the same fact is now visible from the member's side as its holder —
   the two reports should agree by construction (DR4: symmetric where a
   mapping exists), and a test should say so.
5. **The consumed-member arm.** Delivered code permits a consumed
   mdraid/LVM2 member because the aggregate's own arm has no
   assembled-state input; ADR-0018 decided `Refused`, and recorded the
   measurement (`:601-603`) and the arm (`:610`) as obligations. Whether
   the arm lands before the first write-capable increment, and what fact
   shape `Facts` gains for it (an assembled/held state fact keyed by
   address, in the state-layer half), is WP-010's to decide; the round
   filed it as **gitea#1008** so it is a numbered obligation and not a
   paragraph (D6).

## 4. The recommendation

**Option C: no client-built signature node on Linux; a held whole device
is a physical device whose held standing is reported; the cache reports
and decides nothing.** Concretely:

- **D1. Withdraw the plan's §3.5 member-signature item and record why.**
  The client cannot read the two naming fields a `BackingSignature`
  needs (offset: no interface, DR14; family: cache-only per member, and
  measured wrong on the stale pair, L4/L10), the type has no
  offset-absent form, and ADR-0019 reads both fields from the helper's
  byte layer. **No `BackingSignature`, no `Backing` edge, no
  `EncryptionLayer` is built by `adapter-linux`**; the module docs and
  `fields.md` say so in terms, and the WP-L100 record's "what still
  waits" for these three moves from "the next round" to "the helper's
  capture (WP-L110)". No spec text and no ADR: FS-004/INV-004's detect
  duty is the product's and is discharged on the layer ADR-0018 assigns;
  no text places it on the unprivileged layer, so nothing narrows — a
  plan item is corrected on measurement.
- **D2. The held standing, on `holders/`, as a state-layer report.** 4b's
  third slice reads `holders/` on every admitted plain whole device
  through the bounded listing seam. A device whose listing positively
  names an entry is **held**: `PhysicalDevice` under its designated name;
  a `Held { holders }` standing in the naming outcome, distinct from
  ADR-0034's failed-read ground; and an attributed observation on the
  sysfs interface — "held by `<selector>`" — keyed to the device's
  address on the same reporting surface 4a gave mounts. A listing that
  did not answer refuses the device; empty is unheld. Distinct from 4a's
  *withdrawal* (a dm/md/loop node is not a physical device and gets no
  node) — a held member *is* a physical device and keeps its node, so
  the helper's signature nodes have their host. Said plainly: the
  standing is reported, and it changes no verdict until WP-010's arm
  consumes it (D6). Rust, so it owes a sitting; one test over an
  authored tree carrying DR4's shape (an md member held by its array, a
  LUKS disk held by its `dm-N`, an unmapped PV unheld, the plain disk
  unheld, an unanswered listing refused; the member-side and array-side
  reports agreeing); mutations: a held device reported unheld; an
  unanswered listing read as empty; the holder authored from the entry
  name.
- **D3. The cached signature view is reported, not consulted.** The same
  slice reads `ID_FS_TYPE`/`ID_FS_USAGE`/`ID_FS_VERSION` from the record
  half the adapter already has and attaches them as observations on the
  udev interface (`Heuristic`, `inferred`), verbatim, per device — so a
  consumer can show "cache says `linux_raid_member`" beside an unheld
  disk — and **nothing reads them for standing, naming, or kind**. A
  mutation that demotes an unheld device on `ID_FS_USAGE=raid` must be
  killed by a test asserting it stays unheld: the rejected-option guard.
- **D4. File DR15 on WP-035** — `holders/` per member **and** `slaves/`
  per assembled node, both sides, at baseline, after each stop, after
  each re-assembly, and after the reboot, including the
  active-VG-with-unmapped-PV case — the same two-act bracket, taken on
  the DR apparatus before D2 merges (the standing is a claim about a
  live fact; the sitting is what makes it measured rather than assumed).
  DR15 is also, for Linux mdraid, ADR-0018's measurement obligation (4)
  — "the assembled-state facts for mdraid" — and the record should say
  so. If DR15 shows `holders/` lagging or surviving a stop, D2 does not
  land as drafted.
- **D5. Nothing partition-hosted, nothing multipath, nothing Btrfs**
  moves here — 3b and obligation 3 as before. 3b inherits D2's rule for
  partitions (`holders/` on `sda1`) when it builds them.
- **D6. The consumed-member arm on WP-010 — filed as gitea#1008** during
  this round (an issue is cheap and reversible; the designation round
  filed #1006 the same way): the aggregate own-arm's assembled-state
  input and the consumed-member `Refused` ADR-0018 `:391-398` decides,
  against the delivered `Permitted` (`protection.rs:1226`, `:1244-1263`),
  citing `:601-603` and `:610` as the obligations it discharges and
  DR4/DR15 as the Linux measurement. Not this round's to shape beyond
  that; it is the act that gives D2's report a consumer, and whether and
  when WP-010 takes it is the decision owner's.

**Pricing.** D1 and D2/D3's records are Markdown under WP-L100 and
WP-000; D4 is a WP-L100 filing plus a WP-035 preregistration and record;
D2/D3 are one Rust slice under WP-L100 (owes a sitting, r45 at its head);
D6 is an issue now and a WP-010 slice later (Rust; owes a sitting; no
spec text — the sentences are ADR-0018's). No spec text, no ADR — unless
the decision owner wants D1's "signature nodes are helper-only on Linux"
stated normatively, in which case it is a one-sentence ADR-0013-shaped
corollary on FS-004/INV-004 and minor under §0.1 (no MUST narrows; the
client was never required to emit signature nodes).

## 5. Open questions for the decision owner

1. **Is a draft that proposes an unheld orphan member as an operand
   acceptable, on the strength of a re-plan against the helper's
   capture?** The round says yes — it is the path spec `:531` and
   ADR-0014 describe, and the alternative is the option ADR-0018 rejected
   — but it is the one place Option C is fail-open *at the draft* and
   closed only at validation, the re-plan-against-capture path is
   exercised by nothing yet, and it should be chosen with eyes open. The
   mitigating fact is D3: the cache's report is beside the disk, so a UI
   can warn before the round trip.
2. **Should "no signature node from the client" be published in the
   reach payload** (a `partman.adapter-linux.reach/1`), or is `fields.md`
   plus module docs enough? The round says the latter: the reach payload
   answers INV-003's six table states and its schema is versioned for
   that; a second axis is a schema change with no consumer asking for it.
3. **Does DR15 include the LVM2 partial-VG case** — an active VG with an
   unmapped PV, `holders/` on that PV captured — so the record says in a
   cell what DR4 said in passing? The round says yes, cheap and already
   provisioned by the DR layouts.
4. **Should FS-004/INV-004 gain the privilege scoping INV-003 has** — an
   ADR-0013-shaped corollary saying signature detection is discharged on
   the byte layer and the unprivileged layer neither emits nor is
   required to emit signature nodes? The round says not yet: no text
   places the duty on the client, so nothing conflicts; if a later reader
   argues FS-004 binds the client, that is the moment for the corollary,
   and the round's §0 gives it its citations.

## 6. What would change this round's mind

- A client-readable interface that reports a signature's offset — DR14
  looked at every key of the member's and the LUKS disk's udev record and
  found none; sysfs carries none the round knows of. A new kernel or
  udev version shipping one would be a new cell, not a reason to author.
- A decided text under which a naming field may be authored by the layer
  that did not read it — ADR-0019 `:252-256` and ADR-0034 say the
  opposite, and 9.0.0's "no client claim is representable in a bindable
  artifact" (spec `:70`, said of the verdict) is the same posture at the
  artifact level.
- DR15 showing `holders/` is not a live fact (survives a stop, or lags
  re-assembly) — then D2's standing rests on a stale source and would be
  re-drafted; the fallback "held per `holders/` *or* the assembled
  node's `slaves/`" is only available if DR15 measures `slaves/` holding
  where `holders/` does not, which is why DR15 captures both sides.
- A text describing the re-plan-against-capture path differently from
  spec `:531` — then §5.1's cost is not one round trip and Option C's
  fail-open-at-draft would need a different mitigation than D3.

## 7. Next acts, in order

1. Decision-owner call on D1–D6 and §5.
2. WP-L100 filing of DR15 (`Work-Package: WP-L100`; Gitea issue) and the
   WP-035 preregistration, before the guest exists; the WP-L100 record's
   §3.5 correction and "what still waits" edit in the same filing PR
   (gitea#1008 for D6 already exists).
3. The DR15 sitting on the DR apparatus (VMID 9473 next); the record,
   naming ADR-0018 obligation (4) for Linux mdraid where it discharges
   it.
4. If D2/D3: 4b's third slice — the held standing on `holders/` reported
   as state, the cache reported and not consulted, the `slaves/`/`holders/`
   agreement test — one PR, r45 at its head, named in the PR body before
   merge.
5. 4b then closes at its honest scope: aggregates named (mdraid) or
   designator-absent (LVM2), volumes named (LVM), members held or unheld
   with the cache reported beside them, containers and loops reported;
   signatures, edges and encryption layers arriving with the helper's
   capture (WP-L110); the consumed-member verdict with WP-010's arm
   (D6); partitions with 3b; multipath with obligation 3.
