# The Linux mutation-toolset round — LIN-001's mutation half (route b)

**Date:** 2026-08-20. **Base:** `20e2e49` (main), spec 19.0.0, with the
DR25 record (PR #555) beside it.
**Directive:** Nate — take the toolset round, on the 4a-closing
recommendation accepted the same day.
**Question:** WP-L110's route (b), owed before increment 4b: *"LIN-001's
authorization/mutation half (ADR-0054: UDisks2, libblockdev or
authoritative native tools; the UDisks2 ≥ 2.9 tool floor enters the
CAP-006 store with the first invoker)."* Concretely: **what performs the
helper's mutations** — the first GPT/MBR table write and the
file-system operations increment 4b executes — a daemon, a linked
library, launched binaries, or the product's own code.

> Committed session record. `docs/reviews/**` is in WP-000's
> `owned-paths` block and lands in its own `Work-Package: WP-000`
> commit, never bundled with code. Nothing below is decided; §5 is for
> the decision owner. The recommendation prices as a **spec change under
> ADR** and says so.

## 0. The texts the round works under

- **LIN-001** (`AGENT_BUILD_SPEC.md:619`, as ADR-0054 revised it):
  discovery is decided for the measured file route; *"UDisks2,
  libblockdev, or authoritative native tools for authorization and
  mutations, behind the helper's own recorded route decision (WP-L110;
  HLP-002, LIN-009)."* This round is that recorded decision — for the
  **mutation** half. The authorization *mechanism* (how the helper asks
  polkit: `pkcheck` through a launcher, or the D-Bus authority) is the
  apply-ceremony round's own follow-up, on DR22–DR24's rows
  (`docs/reviews/LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md:25` keeps
  routes b and c WP-L110's; its R1/R2 stay its follow-up's), and this
  round does not take it.
- **LIN-002** (`:620`): "Support GPT/MBR, ext2/3/4, Btrfs, XFS, F2FS,
  FAT/exFAT, NTFS, and swap **according to installed capabilities**. The
  NTFS write stack (kernel `ntfs3` vs `ntfs-3g`) is selected and
  version-gated per ADR-L1." No ADR-L1 exists; the NTFS stack choice is
  its own later act and is not taken here.
- **SAFE-004** (`:190-192`): external tools invoked with structured
  argument arrays, a fixed executable allow-list, **verified executable
  identity/version**, bounded output, timeout, sanitized environment,
  trusted absolute locations; "versions outside the tested range make
  the dependent capability `blocked` (ACC-009)".
- **SAFE-005** (`:194-196`): missing dependencies disable the affected
  write operation. **CAP-004**: tool availability and version confirmed
  at runtime.
- **Section 9** (`:791-793`): the Arch row is "Full advertised matrix,
  **tool-version-gated**"; "Per-tool version floors live with the
  capability fixtures (CAP-006) in `docs/capabilities/`, not in this
  spec. UDisks2 ≥ 2.9 is such a floor (ADR-0054 …): entered in that
  store by the first package that invokes UDisks2."
  `docs/capabilities/format.md` §2: the floors list is empty; "a tool's
  floor arrives with the first package that invokes it, under review,
  with its basis stated."
- **HLP-002 and the delivered increments 2–4a**: the helper re-discovers
  independently, authors the table state and protection verdict itself,
  re-plans over its own capture, and authorizes **a plan hash over its
  own authored snapshot** (`AdmittedPlan`, the SEC-002 arms, the
  journal-borne apply). Whatever mutates must be bound by that hash, or
  the binding ends where the mutation begins.
- **WP-L110 obligation 11** (the founding duty): "Every tool through
  the launcher's fixed absolute allow-list with verified identity and
  version, structured argv, bounded output, timeout, sanitized
  environment; tool floors entered in the CAP-006 store with the first
  invocation; absence fails closed; file-system operations per installed
  capability, never assumed (increment 4b …)."
- **WP-L110's boundary**: "No journal, planner, engine or adapter
  re-implementation — the helper calls what is delivered." And ADR-0054's
  reservation: WP-L110 may choose UDisks2 for mutations "without
  contradiction" — the option is genuinely open, not pre-rejected.

## 1. What is measured

Every fact here is a recorded row, a delivered type read off the tree,
or is flagged as archive knowledge. Nothing about a tool's behaviour is
asserted from memory.

1. **DR25** (`docs/quality/observability.md`, taken 2026-08-20 on all
   three tiers, valid on the second invocation): **`sgdisk` and `sfdisk`
   are present by default on every tier** — sgdisk 1.0.8/1.0.9/1.0.10;
   sfdisk from util-linux **2.37.2 / 2.38.1 / 2.42.2, three distinct
   feature generations of the same tool**. **The mkfs family is
   tier-gapped**: ext2/3/4 everywhere; Btrfs absent on Debian 12; XFS
   and NTFS on jammy only; FAT absent on Debian 12; **F2FS and exFAT
   have no maker on any default image**. **libblockdev exists on jammy
   only** (2.26, riding in with the image's udisks2, which every
   acceptance guest purges) and is absent by the loader and by name on
   Debian 12 and Arch. **A launched version query does not cover the
   family**: `mkfs.fat`/`mkfs.vfat` (dosfstools 4.2) refuse both
   `--version` and `-V` on both tiers that ship them; `mkfs.ntfs
   --version` exits 0 with an empty first line; `mke2fs` answers only
   `-V`. The package manager's record carries every version the launch
   could not.
2. **The one delivered launcher** (`apps/cli/src/doctor.rs`): the
   `ToolLauncher` trait's `launch(&self, path, arguments, output_limit)`
   (`:176`) has **no deadline parameter** — `LAUNCH_TIME_LIMIT` is a
   private 5-second constant (`:118`) — and the Linux `ROSTER` (`:78`)
   carries exactly `blkid` and `wipefs`, read-only, tested at
   "util-linux 2.41" — a family **none of the three measured tiers
   ships** (2.37/2.38/2.42). Its home is route (c)'s question, not this
   round's; its shape is what obligation 11 will inherit.
3. **The CAP-004 seams are delivered and pinned empty**
   (`crates/adapter-linux/src/runtime.rs:37-196`): `ToolRequirement`,
   `REQUIREMENTS` ("no served operation launches a tool"), `ToolProbe`,
   `ToolFloor`, `tool_state` — `Missing` and `OutOfRange` fail closed.
   The store (`docs/capabilities/tool-version-floors.json`) is empty.
4. **The repository already authors GPT/MBR bytes, and checks them
   against the native tools** — as test code. `crates/fixtures`
   (`layout.rs`, `catalogue.rs`) writes primary and backup GPTs, entry
   arrays, CRC32s, protective MBRs, a hybrid, conflicting copies, a 4Kn
   geometry; `cargo xtask probe-fixtures`
   (`tools/xtask/src/main.rs:603`) runs the real probers over them in
   CI (the "Real-prober acceptance (FS-004)" job), and its doc comment
   records that **two signature writers were undetectable until their
   checksums were reproduced** — the probers catching what the format
   documentation and the crate's own tests could not. The crate is
   test-tier, WP-020-owned, and stays so; it is cited as proof the
   encoding knowledge is tractable and prober-verified in-tree, not as
   a component to link.
5. **The read half is already product code**: the bounded, enumerating,
   fuzz-obligated table parser (`crates/table-parser`, ADR-0014's
   architecture), and the helper authors `TableState` from it over the
   byte layer (increment 2). There is no encoder anywhere in product
   code.
6. **UDisks2's substrate is unchanged** since ADR-0054: not installed by
   default on two tiers (DR18, DR19), purged on the third's acceptance
   guests, zero rows for its interfaces; the 2.9 floor is parked in
   ADR-0054 awaiting a first invoker.
7. **Archive knowledge, flagged as such (not rows):** the current
   libblockdev line is 3.x with a breaking API change against the 2.26
   jammy ships; and upstream libblockdev 2.x's part plugin has executed
   external partitioning utilities itself. Neither is measured here;
   both would need measuring if option B were pursued.

## 2. What this round decides, and what it leaves alone

Decided here: **the mutating actor for increment 4b's two operation
families** — partition-table writes, and file-system operations
(mkfs-class, per installed capability).

Left alone, each with its owner: the ceremony mechanism (the
apply-ceremony round's follow-up, on DR22–DR24); the launcher's home
(route c, WP-035 in the room); the NTFS write stack (ADR-L1, unwritten);
LUKS/LVM/mdraid/multipath/GRUB/fstab tooling (LIN-003…007, LIN-010,
behind their own rounds); packaging dependencies (LIN-008,
`packaging/`); and the first CAP-006 qualification rows (WP-050's
reviewed act, never authored by the invoking package).

## 3. The options, each against the texts

**A. UDisks2 as the mutating actor.** The helper hands each admitted
step to `udisksd` over D-Bus. *Against, on rows and delivered types:*
(i) the daemon is absent by default on two of three tiers and purged on
the third's guests — the write path would not exist on any default or
measured machine without a package install; (ii) **the plan-hash binding
ends at the bus**: the helper authorizes a hash over its own authored
snapshot (HLP-002, `AdmittedPlan`), but udisksd derives and acts on its
*own* view of the device — the executed mutation is whatever the daemon
computes, a structural TOCTOU no journal entry can witness; (iii)
SAFE-004's identity, allow-list and bounded-output discipline cannot see
a bus-activated daemon (ADR-0054 §2's provoked-launch argument, applied
to the write path this time); (iv) udisksd runs its own polkit checks —
and DR23 measured that a root subject is authorized for `auth_admin`
with no agent and no prompt (rc 0 on both polkit tiers), so for a root
helper-client those checks authorize trivially: the product's ladder
(ADR-0021, delivered in increment 3) would be followed by a second,
vacuous authorization theatre. ADR-0006's "IPC is not linking" keeps
licensing out of the costs, as before. **Rejected**, and with it the
parked UDisks2 ≥ 2.9 floor never gains an invoker: the ADR should say
the number stays parked in ADR-0054's record and enters no store.

**B. libblockdev linked into the helper.** *Against:* (i) C FFI in the
one privileged process — SAFE-009 admits reviewed `unsafe` in a helper
crate, but WP-L110's assignment says "none is planned", and this would
be the plan's first and largest; (ii) present by default on **one tier
of three**, at 2.26, while the current line is 3.x with a breaking API
change (§1.7, flagged) — the dependency would be install-everywhere,
version-forked at its major; (iii) it is a plugin loader — `dlopen`ed
`libbd_*.so` resolved at runtime, which is SAFE-004's verified-identity
question re-asked one level down, unanswerable by an allow-list of
paths the loader does not consult; (iv) upstream 2.x's part plugin has
itself executed partitioning utilities (§1.7, flagged) — the "library"
route may reduce to option C executed by code we neither wrote nor
launched. **Rejected.**

**C. Authoritative native tools for both halves** — `sfdisk`/`sgdisk`
write the tables; `mkfs.*` makes the file systems; everything through
the SAFE-004 launcher. *For:* the table writers are present by default
on **every** tier (DR25 — the only option with universal measured
substrate besides D); the launcher discipline exists in shape;
util-linux is about as reviewed as software gets. *Against, for the
table half specifically:* (i) **the executed artifact is a
translation**: the helper's authorized plan binds byte-level facts over
its authored snapshot, but what runs is an `sfdisk` script or `sgdisk`
argv — a second language, at **three measured feature generations** —
and the tool then applies its own policy (re-reading the device,
recomputing CRCs, rewriting the protective MBR, relocating the backup)
rather than the plan's resolved bytes; (ii) the refusal arms become
prose: the typed, explained refusals every increment has shipped would
end at a launched process whose errors are text to parse, per
generation; (iii) the write path's core acquires per-tier version
floors and dialect testing for the *authoring* side of the very format
the product already parses bindingly. Viable and honest — but the
correctness burden moves into translation and prose-parsing, where the
delivered architecture is weakest.

**D. The split: the product's own table encoder; native tools for file
systems.** The first GPT/MBR table writer is product Rust —
`#![forbid(unsafe_code)]`, the inverse of ADR-0014's parser
architecture, golden vectors, round-trip fuzz against the delivered
parser (Section 11.4's harness already runs), proven on fixture images
in disposable environments only — writing, under the journal and
CONC-001's lock, through a read-write sibling of the byte layer the
helper already owns. The native tools' role on the table path is the
one `probe-fixtures` already demonstrates: **independent verifiers of
our bytes**, never the author. File-system operations are authoritative
native tools through the SAFE-004 launcher, per installed capability:
`mkfs.*` launched with structured argv from the fixed absolute
allow-list, floors entered in the CAP-006 store at first invocation
(format.md's rule, WP-050's reviewed rows), absence failing closed
(LIN-002, SAFE-005) — F2FS and exFAT thereby advertised on no default
tier until packaging (LIN-008) or the operator installs their makers,
which is what "according to installed capabilities" says. *For (the
table half):* the thing reviewed, fuzzed and hashed is the thing
written — the plan-hash binding runs unbroken from validation to the
device byte; refusals stay typed; no tool dialect, no per-generation
floor on the write path's core; and the encoding knowledge is already
in-tree and prober-verified (§1.4), so the risk is engineering, not
research. *Against, stated:* the product owns GPT/MBR correctness end
to end — backup placement, CRC discipline, protective MBR, alignment,
4Kn — and a defect writes a bad table; the mitigations (golden vectors,
round-trip fuzz, prober CI, SAFE-001 disposable-only proving, review)
are the same regime that already guards the parser, plus the verifier
role for the native tools. It is more code than shelling out, and it is
new product surface in the most dangerous place the product has.

*Version verification, under C and D alike:* DR25 measured that a
launched version query does not cover the family (dosfstools answers no
spelling at all), and the delivered probe semantics make that bite:
`ToolProbe::Present { version }` carries "the parsed version, or `None`
where the banner did not parse" (`runtime.rs:119-127`), and `tool_state`
fail-closes an unparsed version to `OutOfRange` — so `mkfs.fat`, probed
by launch, would be blocked at **every** installed version, forever.
SAFE-004's "verified executable identity/version" must therefore rest on
the package manager's record and/or a content digest recorded beside the
launch, the launched query never the sole source; the ADR should fix
that discipline before the first floor is entered.

## 4. The adversarial pass

1. **Is D a re-implementation the boundary forbids?** The boundary
   forbids re-implementing what is *delivered below the helper*
   (journal, planner, engine, adapter). No encoder is delivered
   anywhere in product code (§1.5); the fixtures crate is test-tier and
   stays so. Not a re-implementation — the first implementation.
2. **Is D drift dressed as a decision** (the ADR-0054 §3.1 test —
   would it be recommended if `crates/fixtures` did not exist)? Yes:
   the decisive ground is the plan-hash binding argument, which stands
   on delivered helper types (HLP-002's authored snapshot,
   `AdmittedPlan`, the journal) with no reference to the fixtures
   crate. §1.4 lowers D's cost; it does not supply its reason.
3. **Does "authoritative native tools" make C the spec's own
   preference?** The sentence offers a menu behind "the helper's own
   recorded route decision" — it defers, it does not rank. And
   "authoritative" cuts the other way for tables: the kernel does not
   arbitrate GPT contents on write (`BLKRRPART` re-reads whatever bytes
   landed), so authority for the written bytes is the UEFI
   specification — which the product's parser already encodes bindingly
   and its fixtures already exercise against the probers. For file
   systems the tools *are* the authority (mkfs.ext4's bytes are what
   e2fsprogs says they are), which is exactly where D keeps them.
4. **Obligation 11 and the package objective under D.** Obligation 11
   says "every tool through the launcher" — D routes every tool through
   it; the encoder is not a tool. The WP-L110.md objective sentence
   ("GPT/MBR table writes and file-system operations per installed
   capability, through a SAFE-004 launcher") **would need its own
   consequential edit** — named here so it lands with the route's
   acceptance, not by drift. Section 9's Arch row ("tool-version-gated")
   survives: the advertised matrix stays capability- and tool-gated
   where tools are used.
5. **What would break D that does not break C?** An own-encoder defect
   writes a corrupt table on real hardware. Priced: the same class of
   defect under C is a translation defect — mis-spelled `sfdisk` script,
   dialect drift across 2.37→2.42 — which writes an *honest but wrong*
   table, harder to catch because the tool "succeeded". Neither option
   removes the burden; D puts it where the round-trip fuzz and the
   prober CI already look.
6. **Spec pricing.** Under D, LIN-001's mutation sentence ("UDisks2,
   libblockdev, or authoritative native tools …") becomes false for the
   table half — the route taken is none of the named three. **Major**,
   by the pricing rule this repository uses and by ADR-0054's own
   precedent on the same sentence: **20.0.0**, one ADR, revising the
   mutation half to name the decided split and reserving the ceremony
   mechanism to its own round. Under C the sentence stays true and the
   ADR records the choice with no normative text change. The pass
   looked for a reading making D minor ("our encoder is an
   'authoritative native tool'") and rejected it as the kind of
   stretched reading the pricing memory warns about.
7. **Is this a §1.11 item?** LIN-001-as-revised against SAFE-004 and
   SAFE-009 was checked: no requirement-versus-requirement conflict —
   LIN-001 defers to a decision, SAFE-004 governs tools when used,
   SAFE-009 permits reviewed unsafe and D declines it. Not filed.

## 5. The recommendation, and the decisions for the owner

**Take D.** One ADR (ADR-0056, "the Linux mutation toolset: the
product's own table encoder; native tools, launched and floored, for
file systems"), one spec change (20.0.0) revising LIN-001's mutation
half; the UDisks2 ≥ 2.9 floor recorded as staying parked (no invoker on
the taken route); the version-verification discipline (package record
and/or content digest beside the launch) fixed in the ADR; the
consequential WP-L110.md objective edit named in the ADR and landed by
that package.

The decisions:

1. **The route: D, C, or defer?** (A and B are recommended for
   rejection on §3's grounds either way, so the register does not
   re-litigate them.)
2. **If D: accept the 20.0.0 pricing** (major; LIN-001's mutation
   sentence changes meaning), with ADR-0056 landing before any 4b code
   and its Governance catalogue PR before it, per the standing
   two-PR rule.
3. **The version-verification discipline** — SAFE-004's "verified
   identity/version" read as: fixed absolute path, plus version from
   the package manager's record or a recorded content digest, the
   launched query never the sole source. Accept as the discipline
   obligation 11's launcher inherits?
4. **The absence policy stated for the record**: F2FS and exFAT (and on
   Debian 12, Btrfs/XFS/FAT/NTFS) advertise no capability on default
   images until their makers are installed — the fail-closed reading of
   LIN-002 the engine already implements. Confirm, so no future
   increment reads absence as a defect to route around.

## 6. What would change this round's mind

- A row showing a native table writer that takes **bytes** rather than a
  dialect — then C's translation cost collapses for the table half and
  C strengthens materially.
- The decision owner weighing the own-encoder maintenance burden above
  the binding argument — then C, with the translation layer named as
  reviewed, fuzzed product surface and per-tier dialect floors accepted
  into CAP-006 from the start.
- A measured need for LUKS/LVM/mdraid operations *before* 4b — those
  rounds may reopen the library option for their own domains (the
  cryptsetup/LVM ecosystems are library-shaped in a way partitioning is
  not); nothing here pre-decides them.

## 7. Next acts, in order

1. This round (WP-000, `docs/reviews/`). Decision.
2. Governance PR adding `docs/adr/0056-…` to the catalogue (ownership
   reads the catalogue from the base; two PRs, never both trailers on
   one commit).
3. ADR-0056 + `AGENT_BUILD_SPEC.md` (LIN-001's mutation half; document
   control 20.0.0; changelog row), `spec-change` label.
4. WP-L110's consequential objective edit, its own act.
5. The launcher-home round (route c) — now the only gate left before
   increment 4b.
