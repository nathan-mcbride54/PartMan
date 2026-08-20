# Changelog

All notable implementation changes are recorded here. Specification changes
remain controlled by the changelog in `AGENT_BUILD_SPEC.md`.

## Unreleased

### Added

- **WP-070 slice 3b: record schema v2 — the recorded instant on the
  transition record (JRN-006, MODEL-003; on the WP-L110 increment-4
  shape round's transition-only decision).** `RecordedInstant` (seconds
  since the Unix epoch, caller-authored; the journal crate still reads
  no clock), required on construction and on the wire — a missing
  instant refuses rather than defaults, because a defaulted zero would
  sit below every honest reading and fail the backward-clock bound
  open. v1 is refused at decode with nothing to migrate: no journal
  on-disk home existed while v1 was current, which is why this act
  precedes 4a's on-disk home. Golden vectors regenerated; the closed
  vocabulary and redaction sweep extended over the new position. One
  test added; six mutations killed. Rust: one arc with WP-L110
  increment 4a, its sitting at the arc head.
- **WP-L110 increment 3: ADR-0021's authorization ladder (HLP-003,
  HLP-004, PLAN-004, CAP-007, SEC-009, RPC-002; on the apply-ceremony
  round's R8 decision with S2 recorded).** The helper-computed tier
  (severity-plus-flags, the flags half compared against the empty set so a
  sixth flag escalates by default), the floor act minted without agent or
  terminal, the interactive ceremony behind a seam whose completion value
  is unconstructible in a shipped build, `AdmittedPlan` as the provenance
  type that keeps a forged or cross-user plan away from the computation,
  the tier on the validate-plan response (schema v3), and `apply-plan`
  corrected to increment 4. **Three fail-opens in increments 1–2 fixed
  here**: an unreadable clock rendered as `0` (which made HLP-004's expiry
  unreachable), an idle watchdog that could kill an operation in flight,
  and discarded audit writes. Nine tests; fifteen mutations killed. Two further
  findings from preparing the acceptance, on the same arc: RPC-002's
  remediation sentence carried a fourteen-space gap no gate can see
  (fixed, and pinned as rendered text), and the floor act is **not
  reachable over this build wire** at all - the unsized planner entry
  validate-plan calls has a Disruptive floor, and the request vocabulary
  cannot spell a sized create - so no plan a client can obtain over the
  socket can be applied on any tier. Stated rather than glossed, and
  pinned by a test.
  Rust: the WP-020 sitting is r56.
- **WP-L110 increment 2: HLP-002 re-discovery and validate-plan (HLP-002,
  HLP-004, PLAN-006, PLAN-007, SEC-002, CAP-007, INV-003, SI-13; on
  ADR-0014, ADR-0016, ADR-0018, ADR-0036, ADR-0053 reading (b)).** The
  helper's byte layer (two bounded read-only 64 KiB windows per device,
  bracketed by device number), the capture authoring the table state and
  the table node WP-L100's 3b waited on, `validate-plan` served by
  re-planning with WP-060's `plan()` over the helper's own capture, the
  SEC-002 admission arms for increment 3's apply, the helper's reach
  declaration on the DR21 row, request/response v2 (v1 retired). Nine
  tests; ten mutations killed. Rust: the WP-020 sitting is r55.
- **WP-070 slice 5c: the authorization tier's wire vocabulary made
  public** (`AuthorizationTier::wire_name`), sequenced with WP-L110
  increment 3 so one closed set has one spelling and one owner. Rendering
  only — no parse from a wire word back to a tier exists, because a tier a
  client could name is what CAP-007 makes unrepresentable.
- **WP-010 slice 3r: the ADR-0014 authoring entry and the HLP-004 window
  accessor (SAFE-003, PLAN-007; sequenced with WP-L110 increment 2).**
  `TableState::present(checksum)` — the helper parser's copy-invariant
  digest enters the `Present` state through one documented path — and
  `OperationPlan::validity()` for the helper that enforces the window.
  Two tests; two mutations killed. Rust: one arc with WP-L110
  increment 2, its sitting at the arc head.
- **WP-L110 increment 1: the Linux helper process and its closed surface
  (HLP-001, HLP-005, HLP-006, HLP-007, RPC-005; on ADR-0055 and the launch
  round).** `services/helper-linux` — reached over the Linux transport,
  launched per user through `pkexec` under `org.partman.helper.serve`
  (`allow_active`; the `PKEXEC_UID` launch rule; the `0711` directory and
  `0600` node through the transport; idle exit), HLP-001's six operations
  decoded strictly and closed by test, `status`/`enumerate` served
  (enumeration the adapter's contract as root, a labelled proposal), the
  rest `not-yet-served` naming their increment, an identifier-free audit
  vocabulary; `schemas/helper/operations.md`. Five tests; eight mutations
  killed. Rust: the WP-020 sitting is r54.
- **WP-040 increment 5: the Linux transport (RPC-001, RPC-002, RPC-004,
  SEC-007, SAFE-009; on ADR-0055, spec 19.0.0).** `crates/transport-linux`
  — a Unix-socket endpoint in a root-owned `0711` directory with a `0600`
  node owned by the authorizing user, checked fail-closed and never
  re-moding a path it did not make; `SO_PEERCRED` through `rustix`'s safe
  `socket_peercred`, the connection refused before any byte is read unless
  the peer is the authorizing user; the RPC-002 handshake and RPC-004's
  bounds over a length-prefixed frame, both ends in one crate; no
  first-party `unsafe`, no network type, nothing launched.
  `IdentityClaim::UnixPeerCredentials` now names its verifier;
  `schemas/rpc/transport-linux.md` records the frame, the rules and the
  admission order. Six tests; eight mutations killed. Rust: the WP-020
  sitting is r53.
- **WP-L100: the USB-device-node predicate's evidence moves from none to
  FR6 (ADR-0034; no rule change).** gitea#1002's row, taken 2026-08-19 on
  the FR4 unit in two legs (a real XHCI chain on the Proxmox node; the
  passthrough chain on a jammy guest): `idVendor` and `idProduct` are both
  readable exactly on the ancestors the kernel classes `usb_device` and on
  no interface, SCSI, PCI, virtio or NVMe node; nearest-first selects the
  unit's node over the root hub; the serial is FR4's. `naming.rs`'s doc and
  `fields.md` say so; the predicate itself is unchanged. Rust (a doc
  comment): the WP-020 sitting is r52.
- **WP-L100: the floor finished on ADR-0054 (CAP-004, INV-002; spec
  18.0.0).** The Debian/Ubuntu row composes from its two measured
  conjuncts — distribution and kernel — and the UDisks2 number is a CAP-006
  tool floor, entered by the first package that invokes UDisks2, not this
  adapter's to determine; `compose` takes two; `FloorReport` loses its
  `udisks2` field; every measured guest (DR16–DR19) answers
  `MeetsFloor`; `Undetermined` remains for the shapes no row measured.
  Three tests renamed or revised; five mutations killed. Rust: the
  WP-020 sitting is r51.
- **WP-L100 obligation 4 discharged: the Debian arm of the floor
  determination on DR19 (CAP-004, INV-002; no spec change).** The first
  Debian guest measured `VERSION_ID="12"` — double-quoted, one numeric
  part, no minor, the shape `major.minor` refuses — so `floor.rs` compares
  the leading integer against 12 (a later major above, `11` a measured
  shortfall, a missing or unparsable value undetermined) instead of
  answering undetermined for want of a row. One test over the measured
  bytes; five mutations killed. Rust: the WP-020 sitting is r50.
- **WP-L100 increment 5b: the Section 9 floor determination (CAP-004,
  INV-002; no spec change).** `floor.rs` reads `os-release` — the
  fourth interface, entered by DR16/DR18 — and procfs `osrelease`
  through the bounded record seam and composes the floor fail-closed:
  a measured shortfall is `BelowFloor`, any undetermined conjunct is
  `Undetermined` naming it, only every conjunct met is `MeetsFloor`.
  Ubuntu's release (numeric `major.minor` against 22.04) and kernel
  (against 5.15) are measured; Arch meets its row on `ID` alone; Debian
  is undetermined (no row); the UDisks2 conjunct is undetermined by
  construction, so every Debian/Ubuntu host answers `Undetermined`.
  Three tests over the transcripts' bytes; six mutations killed. Rust:
  the WP-020 sitting is r49.
- **WP-050 increment 5: the undetermined floor arm (CAP-001, CAP-003,
  Section 9; no spec change).** `PlatformFact::Undetermined { conjunct }`
  in `crates/capability`: a Section 9 floor conjunct the producer cannot
  establish is neither met nor below; the engine blocks it under the
  existing `PlatformFloor` reason with the conjunct named in the
  remediation, at the floor's own precedence. Consumer-driven by WP-L100
  increment 5b, landed first. One test; three mutations killed. Rust:
  the WP-020 sitting is r48.
- **WP-L100 increment 5a: the capability seam (CAP-004, INV-006; no
  spec change).** `runtime.rs` produces WP-050's `RuntimeFacts` in the
  engine's own vocabulary (`partman-adapter-linux` now depends on
  `partman-capability`): an empty tool roster for every served
  source-class operation, pinned by test on the plan's finding that no
  read-only operation needs a tool; a typed `NotServed` for mutating
  operations, whose tools are WP-L110's to state; the ACC-009 mapping
  from a caller-supplied structured probe and a store-read floor to the
  engine's tool state, fail-closed on every arm the text leaves open (no
  floor known, unparsed version, failed probe, not probed); and the
  assembly carrying the caller's platform fact unchanged. Nothing
  launches; INV-006 held by the forbidden-tool list and the no-process
  guard. Two tests; five mutations killed. Rust: the WP-020 sitting is
  r47.

### Changed

- **spec-change 19.0.0: SI-41 resolved — RPC-001's Linux clause is revised
  (ADR-0055).** On the Linux transport route round
  (`docs/reviews/LINUX_TRANSPORT_ROUTE_ROUND_2026-08-19.md`) and its
  measurement: a `0700` root-owned directory refuses the SAFE-002 client
  (`EACCES`); the clause now requires a root-owned `0711` directory, a
  socket node owned by the authorizing user and `0600`, and
  peer-credential verification of the connecting process against that
  user — two kernel gates, the Windows SDDL's analog. Route T1 (`std` +
  `rustix`'s safe `socket_peercred`), flat per-user nodes, per-message
  credentials deferred, the transport in its own crate under WP-040.
  Major: a MUST's sentence changes meaning. No code changes here.
- **spec-change 18.0.0: LIN-001's discovery route is decided, and the
  UDisks2 floor moves to the tool it gates (ADR-0054).** On the Linux
  UDisks2 route round (`docs/reviews/LINUX_UDISKS2_ROUTE_ROUND_2026-08-19.md`)
  and the DR16–DR19 rows: LIN-001 now names the measured client-readable
  route (sysfs, the udev database, procfs, `os-release`, each entered by a
  row) and reserves UDisks2/libblockdev/native tools for authorization and
  mutations behind the helper's own route decision; Section 9's
  Debian/Ubuntu row is "Debian 12 / Ubuntu 22.04 LTS; kernel ≥ 5.15", and
  "UDisks2 ≥ 2.9" is a CAP-006 tool floor, entered by the first package
  that invokes UDisks2. Major: two sentences change meaning. No code
  changes here; WP-L100's floor drops the conjunct on this ADR.

- **WP-L100 `adapter-linux`: `held.rs`/`lib.rs` module docs corrected**
  — the held standing's consumer is the helper's capture (WP-L110), not
  a WP-010 consumed-member arm; the delivered closure gives a consumed
  member its consumer's verdict (ADR-0018 reading (b), decided
  2026-08-19; gitea#1008 closed). Doc comments only, no behaviour change;
  Rust, so the WP-020 sitting is r46.

### Added

- **WP-L100 increment 4b, third slice: the held standing, and the cached
  signature view reported and consulted by nothing (LIN-006, INV-004,
  MODEL-004; no spec change).** `held.rs` reads `holders/` on every
  admitted plain whole device and reports its standing — held (each
  holder keyed by its own `md/uuid` or `dm/uuid`, never its entry name),
  unheld, or undetermined where the listing did not answer — as
  MODEL-004 observations on the sysfs interface: a state-layer fact under
  MODEL-005, never a name, the shape 4a gave mounts; DR15 measured the
  relation live from both ends and agreeing by identity while entry names
  moved. `devices.rs` reads `ID_FS_TYPE`/`ID_FS_USAGE`/`ID_FS_VERSION`
  from the same record as the identity keys and reports them
  `Heuristic`/`inferred`; nothing consults them, structurally held. No
  `BackingSignature`, `Backing` edge or `EncryptionLayer` is built by the
  client, and none is waited for (the member-signature offset round). Two
  tests; six mutations killed. Rust: the WP-020 sitting is r45.
- **WP-L100 increment 4b, second slice: naming what ADR-0053 designates
  (LIN-006, INV-004; no spec change beyond 17.4.0's).** `arrays.rs` names
  each mdraid array from sysfs `md/uuid`, bytes verbatim, trailing newline
  included, through the bytes-preserving path — an absent or unreadable
  source keeps the designator-absent name and standing; the udev cache's
  `MD_UUID` is not read for naming. `volumes.rs` classifies device-mapper
  nodes by their `dm/uuid` prefix (`LVM-`, `CRYPT-`, else unrecognized; a
  silent uuid undetermined) as a classification input, never a name, and
  names each LVM logical volume `Volume { producer, name: dm/name verbatim,
  role: None }` under the designator-absent LVM2 aggregate as its producer
  — volume-group classes partition the volumes and set the group count
  without entering any name; a dm-crypt container yields no `Volume`; the
  loop's `loop/backing_file` is reported and no node built until 3b's host
  node exists. Two tests (the second closing under the domain closure:
  a volume of a designator-absent group is indeterminate); five mutations
  killed. Rust: the WP-020 sitting is r44.

- **spec-change 17.4.0: the Linux host-assembled naming designations
  (ADR-0053).** ADR-0034's revisit condition fired in its sanctioned
  direction on the DR1–DR14 rows: the (Linux, Aggregate, mdraid)
  designator is sysfs `md/uuid`, the (Linux, Volume, LVM2 logical volume)
  name is `dm/name`, the (Linux, `BackingExtent`, loop) path is
  `loop/backing_file` — each bytes verbatim, trailing newline included,
  each a direct source measured for value, stability across re-assembly
  and a reboot, and per-unit distinctness. The LVM2 volume-group id stays
  undesignated (not client-readable, L7 — every volume group a
  designator-absent aggregate, its volumes indeterminate non-operands
  until the helper names it) and so does the dm-crypt mapping name (DR12:
  the opener's argument). Rejected and recorded: udev `MD_UUID` (the
  cache, spelling the same bits differently), `dm/uuid` as designator or
  name, member `ID_FS_UUID`, the mapper name for dm-crypt, entry names
  (renumbered across one reboot), and holding for the member-signature
  question, which gets its own round (DR14: no interface reports an
  offset). Minor under §0.1; no requirement text moves.

- **WP-035: the naming-designation cells DR11–DR14, preregistered and
  taken 2026-08-18 (gitea#1007), all four established.** On the DR
  apparatus with one declared reboot leg (kernel pinned by `grub-reboot`,
  -186 on both sides): sysfs `md/uuid` **exists**, is client-readable,
  hyphenated where the udev cache's `MD_UUID` is colon-quartet, and is
  byte-equal across re-assembly and reboot and distinct per array — a
  direct source, so mdraid is designatable on the ADR-0035 shape (DR11);
  `dm/name` is stable for LVM logical volumes and is the **opener's
  argument** for dm-crypt mappings — the sitting's own mis-addressed
  re-open showed container A under the name `cr_b` — so it qualifies for
  LVs and not for opened containers (DR12); two loops on one file report
  the same `backing_file` and a re-attach reports the path verbatim
  (DR13); the member-signature family is client-readable through both
  interfaces and **no interface reports an offset** (DR14). The reboot
  renumbered the disks (`sdh`/`sdi` swapped), which the first phase-2 pass
  did not survive; two instrument amendments (serial- and `md/uuid`-keyed
  re-addressing; containers re-opened under their baseline names) are
  recorded and the mis-addressed captures retained. No designation is
  made; the round's next act is now an ADR.

- **WP-L100 increment 4b, first slice: mdraid arrays as designator-absent
  aggregates (LIN-006, INV-004; no spec change).** The Linux host-assembled
  designation round found no measured source that may name a host-assembled
  kind under ADR-0034's discipline and was taken as recommended — designate
  nothing on today's rows, file the missing cells (DR11–DR14, gitea#1007),
  and start 4b on what needs no designation once the closure enforces the
  designator-absent rule (WP-010 slice 3q, gitea#1006, the domain act first
  in one arc). `crates/adapter-linux/src/arrays.rs`: every `md/`-marked
  device is an `Aggregate { Mdraid, designator: None }` carrying its
  self-reported member count from `md/raid_disks` (DR5; a decimal or a
  refusal) and the kernel's `slaves/` listing reported, not edged (DR4);
  two arrays absorb into one collision group, one alone is indeterminate
  and not an operand through the 3q arm — one test asserts both halves.
  Three mutations killed. `fields.md` §7 rows; the 4b record. Rust: r43 at
  the arc's head.

- **WP-010 slice 3q: a designator-absent aggregate is `Indeterminate` and
  not an operand, as ADR-0019 decides (gitea#1006; no spec change).** The
  closure's aggregate own-arm matched on technology alone and returned
  `Permitted` for a lone designator-absent LVM2 or mdraid aggregate — the
  sentence ADR-0019 decided (`:159-161`) and the naming type's doc-comment
  restated, over an arm that never read the field; found by the adversarial
  pass on the Linux host-assembled designation round. Now a designator-absent
  APFS, LVM2 or mdraid aggregate is `Indeterminate { MissingFact }` before
  any of their own arms run, the class refusals standing above it; a step
  reaching such an aggregate does not construct. One test (alone, through a
  member's signature, and with the class refusal), one mutation killed. Rust:
  one arc with WP-L100 increment 4b's first slice, r43 at its head.

- **WP-L100 increment 4a: the state layer and the withdrawal, on the
  detection rows (LIN-006, INV-004; no spec change).** The kernel's procfs
  mount and swap tables enter `crates/adapter-linux` as its third
  interface (`linux-procfs`, direct), entered the way the first two were —
  by the DR1/DR2 rows WP-035 took the same day — and read through the
  bounded seam under a table bound of their own; every line is an
  attributed observation carrying the kernel's line verbatim, parsed into
  the recorded shape with no transformation, and one line off the shape
  refuses the whole table rather than a partial one. Mounts key to the
  admitted devices by `major:minor` and nothing else, so the whole-disk,
  loop and dm mounts key and the anonymous Btrfs, pseudo and partition
  entries stay unkeyed (DR1's finding). And DR3's `dm/`, `md/`, `loop/`
  markers classify every admitted whole device: a marker positively
  present makes the node host-assembled — reported, named nothing, not an
  operand — every marker positively absent admits a plain disk, and a
  marker whose listing did not answer refuses; increment 3a had admitted
  every such node as an operand-eligible `PhysicalDevice`, and that is
  withdrawn in the fail-closed direction. Four tests over authored trees
  carrying the recorded shapes; six mutations killed. The reach
  declaration names the third interface; `fields.md` §7 carries the
  roster on its rows. Increment 4b — the topology half — waits on a
  naming-designation round over DR3/DR5/DR6/DR10, since ADR-0034's table
  designates no Linux source for any host-assembled kind. Rust: the
  WP-020 sitting is r42.

- **WP-035: the detection-rows sitting DR1–DR10, preregistered and taken
  2026-08-18 (gitea#1005), all ten established.** WP-L100 increment 4 filed
  ten Linux rows it could not author its detection layer without — the
  mount and swap tables, the sysfs kind markers and membership listings of
  host-assembled block devices, the cached signature view of their
  members, the loop and Btrfs surfaces, mount-cycle byte stability, and
  per-unit UUID distinctness across re-assembly. Preregistered on the
  floor-rows precedent (`docs/quality/observability.md`), taken on two
  disposable Proxmox guests with fourteen virtual disks and no
  passthrough, valid on the second invocation (the first void for DR2 on a
  setup-actor `mkswap` flag; retained). Findings the increment must carry:
  a Btrfs mount keys by an anonymous `major:minor` and names its member
  only in the source field (DR1); `slaves/`/`holders/` is a per-mapping
  relation, not aggregate membership (DR4); a plain disk's udev record
  carries `ID_FS_TYPE=` empty as a positive absence (DR6); virtio-scsi
  `device/wwid` is a failed read, not an absence (DR9); minor numbers move
  across re-assembly while `dm/uuid`, `MD_UUID`, `ID_FS_UUID` do not
  (DR10). No designation is made; the record names the inputs a round
  could rest on.

- **WP-060 increment 12: the consequence text is stated into the body
  (Section 6; slice 3p's plan body v5; no spec change).** Every planning
  path assembles through `assemble_linked_stated` with the `Display`
  sentences of its typed consequence facts, so the body's `consequences`
  set is exactly those sentences (canonically sorted, so emission order
  never reaches the hash) and the empty set where the vocabulary is
  silent; the reversal draft carries the empty set. `Planned.consequences`
  stays as the typed form beside the plan. ADR-0052 D6's
  "pending-in-body" is delivered; issue #371's last rider closes. Two
  mutations killed.

- **WP-010 slice 3p: plan body version 5 — Section 6's consequence text
  rides the body (no spec change; ADR-0023's form applied).** The body
  gains a required, set-valued `consequences` array of non-empty
  sentences — sorted by canonical bytes (length-first) and unique,
  empty where the planner has nothing to state, pinned empty in a reversal
  draft — through the fully-stated `OperationPlan::assemble_linked_stated`;
  the delegating `assemble_linked` states none, so every existing emitter
  stays valid. The boundary requires the item and refuses an unsorted or
  repeated set, an empty sentence, a non-text element, and a draft that
  states any. Version 4 is retired on the v2/v3 precedent (one change
  window, no emitter outside it, no surviving artifact) and refuses at
  decode; the plan vectors are regenerated as v5 — the identity-bound
  one now states two sentences so the set form is pinned cross-language
  — and the TypeScript suite reproduces all forty-five unchanged. Seven
  mutations, each proven applied, all killed. Jointly sequenced with
  WP-060 increment 12, which states the planner's sentences into the
  body; until it lands the planner emits version 5 with an empty set.

- **WP-060 increment 11: the move — PART-005's destination vocabulary
  (ADR-0052, spec 17.3.0, on issue #371).** `SizedRequest::Move { target,
  new_start }`; the solver's destination rule with the source counted as
  room and the scoped not-named-within clause; the copy mode derived from
  the ranges; the conservative `destroyed = S`, `consumed = D` declaration;
  the simulation renaming the moved partition and what it names at the
  destination; the `MoveDraft` reversal resolved under the unchanged
  step-output contract; and `Consequence::RelocationReleases` enumerating
  what a move releases without carrying, its negative space bounded. The
  `no_representable_request_relocates_bytes` tripwire is replaced by
  `only_a_move_relocates_a_pre_existing_start`, PART-005's traceability
  rows now tracing to tests that exercise a move. Ten mutations, each
  proven applied, all killed. `Copy` stays source-class and unplanned; the
  unsized `Move` still refuses as not representable.

- **WP-010: `names_within` is public, and the `consumed` doc-comment says
  what is enforced (ADR-0052; no spec change beyond 17.3.0's row).**
  ADR-0052 names `names_within` as the predicate for both halves of a
  relocation — the consumed-class exception and the solver's destination
  rule — so the domain exposes it rather than having the planner re-derive
  it from the naming relation. `StepRanges::consumed`'s comment claimed the
  constructor verifies freeness; `PlanStep::mutating_declared` never has,
  and the comment now says where the judgment lives. Behaviour unchanged;
  no test moves.

- **WP-L100 increment 3a: devices are addressed, and INV-004's presentable
  derivation lands (ADR-0019, ADR-0033, ADR-0034; no spec change).** The
  Linux adapter gains a bytes-preserving naming seam, ADR-0034's designated
  serial resolution, ADR-0019 collision grouping through the domain's own
  `absorb`, and the alignment derivation with its refusal arms.

  **The seam is ADR-0034's own first delivery obligation.** That ADR records
  the delivered `read_attribute` as "not a lawful naming-input path" because
  it validates UTF-8, refuses non-text, and strips one trailing newline.
  `read_naming_source` does none of those. The divergence is measured rather
  than asserted: the suite reads the same fixture files both ways, and a file
  holding a lone newline is a positively determined **absence** through the
  text path and a one-byte **name** through this one.

  **The serial resolves structurally, not at the measured depth.** ADR-0034
  says the rule is "the nearest ancestor sysfs node that is a USB device
  node" and that the instrument's four-step traversal "names the structure
  that traversal reached", not the rule. The fixtures carry the USB node at
  four depths and plant two decoy `serial` attributes on non-USB ancestors,
  one nearer and one farther, so a fixed-depth walk and a first-readable-
  serial walk each fail a named test. ADR-0034's two outcome rules land with
  it: a measured absence leaves an operand with a weaker name, a failed read
  leaves an indeterminate non-operand, and an undesignated cell is read **not
  at all** — held by a source that records its reads, because no assertion
  over return values can establish a negative.

  **One shortfall is shipped fail-closed and recorded, on increment 2's own
  precedent.** FR4 establishes that the measured traversal *reaches* a USB
  device node; no row establishes what a client may read to *recognize* one,
  which is a different claim and the one the delivered predicate makes.
  Recognition requires both `idVendor` and `idProduct` to answer, an
  unreadable marker recognizes nothing, and an unidentified ancestor yields
  an absent serial and a weaker name. The predicate can only lose a name,
  never invent one. Filed as an obligation on WP-035, which owns
  `docs/quality/observability.md`.

  **ADR-0033's imported obligation is discharged with a fixture for each
  arm**: alignment presented over authoritative and inferred inputs — an
  inferred input is fit, because the input's confidence travels by reference
  rather than being copied onto the derivation — withheld with the input's
  own state surfaced over `unavailable` and `conflicting`, and withheld over
  an input fit by confidence that carries no usable value, an arm ADR-0033
  does not name and a positively determined absence reaches. The
  `conflicting` fixture is hand-built and said to be: this adapter keys each
  property by the interface that answered, so production cannot produce a
  plural set, and the arm would otherwise go untested rather than
  unreachable.

  **Free extents are not presented at all, and ADR-0036's choice is
  recorded.** INV-004 forbids presenting the derivation "where the host
  declares a table scheme the build cannot name"; this contract builds no
  partition-table node, so it declares none. ADR-0036's forward obligation
  put a binary choice to this increment, and the second branch is taken: the
  solver reserves nothing on Linux client drafts until HLP-002 re-discovery
  supplies a table node. The first was declined on **measured** grounds
  rather than for want of a value. `ID_PART_TABLE_TYPE` *is* carried in the
  client-readable udev database for loop-attached fixtures, and the record
  measures it wrong on exactly the cases the domain model exists to
  represent: `gpt` on the Indeterminate conflicting-tables fixture whose
  backup view "appears nowhere"; `gpt` on a damaged fixture the kernel
  materialized nothing from; `gpt, untraced` on a hybrid whose aliasing
  `0x0c` entry "left no trace in the client projection"; and `PMBR` rather
  than `gpt` on a 4Kn disk. Designating it would make the adapter assert
  table states the record measures as false.

  The layered topology — partitions, volumes, file systems, encryption
  layers, signatures, and the `Captured` snapshot — is **increment 3b**, not
  started and blocked on that route. Two further gaps are filed rather than
  worked around: a real-hardware table-role row, and the fact that this
  package's scope names **mounts** while `NamingFields` carries no mount
  variant at all, which is WP-010's model rather than this adapter's.

  Fourteen mutants killed by named tests before proposal, including a
  compiling fixed-depth walk, a single-marker predicate, a wrong sector
  unit, a silently overflowing byte product, a signed sector count, an
  absence reported as a failed read and its converse, a value read before
  the confidence gate, and free extents presented as an empty list.

### Fixed

- **WP-035: the transport-discrimination protocol's deferral is re-pointed
  at its real sponsor (issue #366; a decision-owner call — no ADR, no
  version change, no code).** WP-035's `observability.md` share deferred
  the fabric-versus-local protocol row to "whichever package first records
  a transport route decision", the 2026-08-13 grant-question round's route
  (c). Measured at `381c7ec`, every occurrence of that phrase in this
  repository is WP-040's per-OS **IPC** transport — `identity.rs:104`,
  `:107` and `:110`, `schemas/rpc/authentication.md:40-42`, ADR-0021:236,
  and five in `WP-040.md` — and none is a device-transport classification.
  WP-040 has no reference to ADR-0018, no observability share and no
  obligations section, so the deferral had no scheduled author: precisely
  the late discovery that filing obligations exists to prevent. The
  sponsor is now **WP-010**, named rather than described by role.

  Two measurements decided which replacement. The ADR-0034-pattern
  designation extension the deferring round named beside WP-040, in the
  same sentence, is sponsored from WP-010's assignment, whose
  `owned-paths-reserved` block carries that ADR's path — the round kept
  the wrong half of its own sentence. And a platform adapter cannot
  sponsor it: WP-L100's imported obligation 6 records that the protocol's
  only source is vendor documentation, which that package's evidence rule
  forbids. The *rows* of ADR-0018's evidence obligation (2) are measurable
  there and two of the six positive-local classes already are; the
  *protocol* is not. They are two obligations, not one.

  Nothing became owed and no increment in any package gained or lost an
  item. Every adapter's transport answer stays
  `TransportClass::Unrecognized`, which resolves to `Indeterminate` at the
  closure and never to `Permitted` — ADR-0018's own terms for an
  unmeasured transport. Three assignments carry the change: WP-035's
  clause re-pointed with all three rejected readings recorded beside it,
  WP-010's reciprocal obligation added so the sponsorship is scheduled
  rather than asserted only in another package's text, and WP-L100's
  open-reading paragraph answered while keeping the clause as accepted on
  2026-08-13 beside its answer.

- **WP-010: descent admits an unlocated child of a geometric parent
  (issue #319's authorization half; ADR-0051, spec 17.2.0 — minor).**
  The oldest live hole in the tree, and the one three predicates had died
  on. `descends_into` never descended into a containment child that
  declared no extent, so removing a ZFS signature's one extent fact —
  which nothing requires — took every mutating operation on the disk from
  refusing 10 of 10 to `Clear` 10 of 10 over a live pool. The round's
  finding is that the arm's comment described a job it does not do: the
  capture it prevents is a **partition table's**, whose extent is its own
  header bytes rather than the region it governs, and ADR-0041's
  `containment_pair_is_geometric` — the predicate that actually decides
  it — had never been consulted by this arm. Descent now admits an
  unlocated child where the pair is geometric and refuses it where the
  pair is structural. One red, ADR-0048's pinned open limit for this very
  issue, closing deliberately; both sibling pins survive. Two mutations,
  each proven applied, each killed. Issue #319's third measured shape did
  not reproduce on the committed fixture and is recorded as unmeasured
  rather than claimed closed.

- **WP-010: a backing extent is framed on its named host (issue #365;
  ADR-0050, spec 17.1.0 — minor).** ADR-0046 enforced the anchoring rule
  for every kind but one: a backing extent is outside every containment
  forest, so `frame_root` is `None` for it and the frame check never ran,
  and no edge may target one so the edge-versus-extent cross-check never
  saw it. It was the single node whose declared frame nothing
  constrained, and three acts had to pin limits around that hole. The
  model already answered the question — `ExtentLocator::Range` reads "a
  byte range within the host node's own address space" and
  `BackingExtent.host` names that node — so this is enforcement, not a
  new decision. Absence still admits. Measured: three reds, every one a
  repair. ADR-0022's occupancy witness rebuilds on a lawful body and
  still proves the frame arm finds what nothing else does; ADR-0046's
  enumeration is strengthened to no exceptions at all; ADR-0049's pinned
  limit closes. `crates/planner` took the consumer-first pull request its
  own grant required. Two mutations, each proven applied, each killed.
  Issue #365 closes entire: Part 1's two wrong doc comments now state the
  relation rather than a list that can drift from it, Part 2's coverage
  is delivered, and Part 3 was already discharged by ADR-0045.

- **WP-010: reach follows the hosting name (issue #409; ADR-0049, spec
  17.0.0 — major).** A `BackingExtent` is the target of no edge kind —
  the pair table admits it only as the source of `HostBacking`, and
  `Topology::build` refuses `containment(file-system → backing-extent)`
  outright — so the closure, which walks edges, could not traverse the
  relation its hashed name asserts. The entire host-backed class had no
  upward reach: measured on a body that validates, wiping the disk
  holding a loop image gated **`Clear` on 10 of 10** mutating operations
  over a live ZFS pool, and so did wiping its file system, which the
  filing had not recorded. The closure gains a fourth arm — downward
  hosting — bounded by the same declared geometry as containment,
  descending only, and carrying destruction exactly when the host is
  destroyed. It reads the name and never the frame, so #365's frame
  question stays open and untouched. Both alternatives were built and
  run: admitting a containment pair makes the honest body
  unrepresentable and, once reframed to satisfy that, makes a 512 MiB
  image file read as destruction of the whole disk; additionally moving
  the naming rule to `Sources(CONTAINMENT)` costs five reds including
  ADR-0022's occupancy witness and ADR-0046's carve-out pin. The chosen
  route costs zero reds. Four mutations, each proven applied, all killed
  — one only after the round added the regression it was missing.

- **WP-010: an extentless target is destroyed by identity (issue #392;
  ADR-0048, spec 16.0.0 — major).** A `Volume`, `Aggregate`,
  `EncryptionLayer` or `MultipathNode` declares no extent, so
  `canonical_ranges` gave it no destroyed range at all: measured on the
  committed `partitioned_mdraid` fixture, `Wipe(md0)` and `Wipe(array)`
  gated **`Clear` on 10 of 10** mutating operations over a live ZFS pool,
  on a complete ADR-0046-lawful body with nothing omitted. Two rules land
  together, and the measurement establishes that both are needed: an
  extentless target's canonical destroyed entry is its **whole frame**,
  which closes the volume; and the destroyed-target seed gains a second
  source — such a target named by a destroyed range framed on itself is
  destroyed by identity — which is the only thing that closes the
  **aggregate**, since nothing is framed on an aggregate. Both move to
  `Clear` 6/10 (the four destroying operations refuse; the six that write
  and destroy nothing are deliberately unchanged), while the committed
  `Label(sda)` and `Wipe(sda)` controls are byte-identical. Cost: one red
  workspace-wide, ADR-0044's pinned limit rewritten in place. Five
  mutations, each proven applied; three killed, one killed only after the
  round added the missing regression (the seed is frame-equal, not merely
  non-empty), and one recorded as a survivor that is a proof. Editorial,
  corrected in the same pass and not this act's own claim: "the
  operation's minimal invariant ranges" is not ADR-0018 text, and a
  create-writes-and-consumes claim in the same comment had contradicted
  the code beneath it since ADR-0042. Issue #319 is untouched and its
  third measured shape is pinned as an open limit; the planner simulation
  coverage this population lacks is WP-060's own pull request.

- **WP-010: an omitted edge is not an escape from inheritance (issue
  #397; ADR-0047, spec 15.3.0 — minor).** `device_scope_verdict` and
  `producer_verdict` both walked the edge set only, so a body that named
  its host in `FileSystem.host`, `BackingSignature.host`,
  `PartitionTable.parent` or `Partition.parent_table`, or its producer in
  `Volume.producer`, **with the edge omitted, inherited nothing**.
  Measured on ADR-0045's own pinned named limit: an xfs naming a
  multipath node gated `Clear` on all ten mutating operations with the
  edge dropped, against `Unsupported{InheritedDeviceScope}` with it
  present; the same shape held for a `RecognizedRemote` device's
  transport arm and for a volume naming a ZFS aggregate as its producer.
  ADR-0043 closed this class of escape for *release* by reading
  `Partition.parent_table` rather than the edge; the inheritance verdicts
  now follow. The containment parent a node's name declares is ascended
  alongside every incoming containment edge, and the producers its name
  declares are folded alongside the `Production` and `HostBacking` edge
  sources — the qualifying fields read off `naming_referent_rule` rather
  than a second copy of the list. Both fold with `worst`, so an added
  ancestry can only ever **add** refusal and an agreeing body answers
  exactly as before; `affected_set` is untouched, no descent bound moves,
  and no schema version or golden vector moves. Cost: one red across the
  whole workspace, ADR-0045's pin itself. Six mutations, each proven
  applied — three killed, and three recorded as survivors that are
  proofs. **Named limit**: `NamingFields::Aggregate` carries no naming
  referents, so a signature whose `Backing` edge to its aggregate is
  omitted still leaves the aggregate unreached; this does not close the
  omitted-edge escape in general.

- **WP-010: occupancy is read as bytes, not as frame names (issue #401;
  spec version unchanged — a defect fix on ADR-0022's truthfulness
  mechanism, recorded in the ADR that lands issue #333's frame
  enforcement).** `Precondition::violated_by` found an occupant of a
  host only where an extent was *framed on* the host — `extent.host ==
  host` — and ADR-0037's accepted rule (12.14.0) makes a partition never
  a frame: a file system inside a partition carries its extent on the
  device. So on any capture framed as the rule requires, `HostUnoccupied`
  over a created partition and `RegionUnoccupied` over a grow's reclaimed
  tail held vacuously, and the decayed reversal that ADR-0022 exists to
  refuse bound and destroyed. Measured at `43872c0`: re-framing only
  `reversal_worlds`' file system onto the device — the lawful spelling,
  nothing else changed — and `a_decayed_precondition_refuses_at_binding`
  binds; the committed test was green only on the partition-framed
  spelling ADR-0037 calls unlawful. **Occupancy is now read three ways
  and a node found by any of them occupies**: an extent framed on the
  host (the old reading, kept, so nothing found before is lost); an
  extent lying on the host's bytes, compared in the frame the host's own
  extent is expressed in — a region translated through the host's extent
  into that frame, or the host's extent entire — with the host's own
  frame ancestors (its table, its device, read off the naming relation)
  excused and nothing else; and, for the whole-host form, a node whose
  own name positions it inside the host, extent or none. A host whose own
  extent is absent has bytes that cannot be located and is returned
  itself: honest absence fails closed at this arm as at every other. The
  naming walk (`named_position`, `named_ancestry`, `names_within`) reads
  `naming_referent_rule` — the field that names a containment source is
  the hop, a backing extent's open `host` is outside every forest — so
  there is no second roster. Green on the committed population unchanged
  (674 tests); one new regression enumerates the four readings, the
  ancestor exclusion, the byte-exact tail, the unlocated host, and the
  one corner the old reading alone answers; six mutations on this arm
  (each reading dropped, the ancestors made occupants, the region left
  untranslated, the unlocated host admitted), each proven applied, each
  killed. First PR of issue #333's enforcement arc: WP-060's fixtures
  move next, then the frame rule lands with ADR-0046.

- **WP-010: a frame root is never written wholesale, and a target frame
  root reaches what it carries (issue #353, ADR-0042; spec version
  unchanged — a defect fix against §2.1:110).** `canonical_ranges` put
  the target's whole extent in `written_table_extents` for `Create`,
  `Grow`, `Repair`, `Label`, `Uuid` and `Decrypt`; for a device target
  that is "the parent device wholesale" in the sentence's own words, and
  the whole-disk gates refused *because* of it — correct the entry alone
  and six gates open over a live pool with a green suite (the issue's
  table). Now a target whose extent is expressed in its own address
  space declares no written range, and `descends_into` lets
  carried-content descent leave a self-framed extent when, and only
  when, that node is the step's target — the operand is in the set by
  identity, not by intersection, so ADR-0039's sibling-capture guard is
  untouched for every other node and re-asserted on the same disk.
  Measured on five layouts: the whole-disk vdev keeps all ten refusals
  through the hop; a partitioned disk carrying a protected *partition*
  moves six device-target gates from over-refusal to `Clear` (creating
  a partition in free space does not touch sda2) while the four release
  operations still refuse; the ordinary disk, the LUKS chain and every
  partition-target gate are unchanged; ADR-0040's whole-disk pin holds,
  its revisit condition discharged. Below a frame root the entry is
  unchanged and pinned — an over-approximation the record names, kept
  because the planner's touched-device derivation reads it, and whose
  removal survives the domain suite while silently dropping PART-013
  obligations there (WP-060 pins the consumer side first, PR #382).
  Five mutations, each proven applied; four killed in the domain suite,
  the fifth by that planner test. What stays open: the per-kind truthful
  entry, which needs the request or the topology at `canonical_ranges`
  and is a cross-package act.

- **WP-010: the relocation exemption is retired, and the release entry
  stands (issue #348, ADR-0040, spec 13.0.1).** ADR-0018:141-145
  exempted "the relocated target's own subtree from destruction
  descent"; ADR-0038 gave `Move` the whole-target-extent `destroyed`
  entry that captures exactly that subtree. The exemption is retired as
  **void where it stood** — §0.2's rule 4 forbids an ADR weakening a
  spec MUST, and §2.1's enforcement paragraph is a MUST NOT — and it was
  additionally never delivered, never cited by any requirement, and not
  expressible in the delivered closure, which takes no `Operation` at
  either call site.

  **No production line changes.** The issue's own proposed fix —
  reverting ADR-0038's `Move` entry — was rejected on measurement.
  Moving `Move` between range arms changes no verdict on six targets
  with the suite green; deleting its entry outright changes none on a
  partition target and turns a whole-disk `Move` from
  `Unsupported{Zfs}` into **`Clear` over a live ZFS pool**. On a disk
  target a self-framed extent is never a descent source, so
  carried-content reach cannot propagate and reach is entirely
  range-driven — which makes ADR-0038's release entry load-bearing
  after ADR-0039 rather than superseded by it.

  That mutation survived the entire committed suite, so the one
  regression this adds
  (`a_release_over_a_whole_disk_reaches_the_aggregate_it_carries`) is
  the coverage that was missing, and it is **proven to bite**: under
  each mutation it is the only red, once on the gate assertion and once
  on the range-class assertion.

  **The availability gap the exemption named stays open and is filed,
  not settled** — a length-preserving relocation of a protected
  partition refuses although copy-then-commit would preserve every byte.
  The byte-wise-preservation half of ADR-0018's paragraph survives as a
  plan-layer duty and is delivered nowhere; it is filed too.

- **WP-L100: the Linux field record swept onto the rows that answered it,
  and two claims that stood above their evidence corrected (issue #318).**
  Issue #318 filed six unmeasured Linux observability rows; WP-035 took all
  six on 2026-08-13, as the readback rows R1–R4 and the floor-rows sitting
  FR1–FR5. Both records landed in `docs/quality/observability.md` and
  nowhere else, so every consumer still described the rows as missing —
  the stale-record shape a sitting produces by construction, one document
  updated and the rest left behind. `schemas/adapter-linux/fields.md`,
  `docs/work-packages/WP-L100.md` and two comments in
  `crates/adapter-linux` now cite the rows.

  **Two entries were not merely stale but wrong, and both are the same
  defect this roster exists to catch — a claim stated above its evidence.**
  `device/wwid` was recorded as **positively absent** on real usb-storage
  with the strength "real-hardware (as an absence)"; R2 establishes that
  the read failed `ENXIO` at every capture, which is a failed read and not
  ADR-C4's `ObservedAbsent`. The delivered code was already right — only
  `ErrorKind::NotFound` reaches `AttributeRead::NotPresent`, every other
  error becoming `AttributeRead::Failed` — so this was a defect in the
  record alone. And `device/serial` carried a real-hardware serial that R1
  shows was read from a **different node**: the observed value came from the
  USB device node reached by parent traversal, while `device/serial` on the
  SCSI node — the path this roster reads — was never read in the sitting and
  is unobserved on every Linux host. Its evidence is now **none**, which is
  also what ADR-0034 implies: the designated Linux serial source is the
  traversal, not this path. Reconciling the two is increment 3's work.

  **The transport answer is unchanged and its reason is not.** `ID_BUS=usb`
  and two `ID_PATH` values are now recorded (R4), so "no Linux row records a
  classifying value" — which stood in the assignment, in `fields.md`, in a
  shipped doc comment on `transport_class`, and in a test's requirement
  comment — is false. `Unrecognized` still holds for every device because no
  **discrimination protocol** maps a value to a class: ADR-0018's evidence
  obligation 2 is outstanding on every platform, and it is the one item of
  the six that measurement did not close. Its home was decided on
  2026-08-13 and its addressee is filed as **#366** — "whichever package
  first records a transport route decision" denotes WP-040's IPC transports
  everywhere else in this repository, and WP-040 never consumes this row.

- **WP-010/WP-060: a node's own name may no longer reference an address
  nothing carries (issue #354, partially — the kind half is held).**
  Eight node kinds embed a `NodeId` in their hashed name, and no layer
  required any of them to resolve. `absorb` never looks inside a field;
  `Topology::build` validated edge endpoints only, its doc comment about
  rejecting unknown referents being about edges; and
  `TopologySnapshot::from_canonical_body` re-runs that same construction,
  so the decode boundary inherited the blindness. A body whose partition
  named a derived-but-never-absorbed table assembled, encoded, decoded
  and rebuilt with agreeing hashes, with lawful containment edges under
  the real table — the name saying one thing, the edges another, and no
  layer comparing them. `Topology::build` now sweeps the absorbed set
  before reading any edge and refuses with `UnresolvedNamingReferent`,
  naming the node, its kind, the field and the address that did not
  resolve.

  **The sweep is resolve-only, and this does not close #354.**
  ADR-0037:146-150's stated harm is the forbidden *pairing*, which
  resolve-only leaves open. The panel's winning design derived the kind
  check from `endpoint_pair_allowed` — genuinely the right shape, since
  it is the delivered pair table rather than a second authored list — but
  that table lists the pairs the *edge* validator needs and was never a
  catalogue of what a naming field may reference. Deriving a mandatory
  check from it promotes its omissions into refusals: measured against
  current main, it refused a GPT inside a LUKS volume, a partitioned
  mdraid array, and an xfs on a dm-multipath node, all of which build.
  The kind half is held behind **#360**, the pair-table gap itself, and
  three standing controls now fail if it leaks in.

  **The planner's duplicate referent roster is deleted**, both readings
  now sharing `NamingFields::naming_referents`. This is load-bearing
  rather than tidying, and the direction of the risk is the reverse of
  the obvious one: the destruction closure removes everything named
  relative to a removed node, so a referent kind it failed to follow
  would leave a survivor naming a casualty — which this change turns
  from a slightly wrong prediction into a hard `SimulateRefusal`. With
  the `Volume` arm dropped from the roster, the entire planner suite
  stayed green while the domain sweep refused the plan outright; that
  gap is now covered by a test asserting the rebuild *stands*.

  **MODEL-003 discharge, recorded rather than assumed.** The issue names
  this a versioned behaviour change at the decode boundary, and it is:
  bodies that decoded now refuse. It is taken under MODEL-003's
  **explicit-rejection** limb, with `SCHEMA_VERSION` deliberately left at
  1 and no spec bump. The byte format, field shapes and parse rules are
  untouched — `fields_from_map` accepts exactly what it accepted — and
  the refused population is bodies that were never lawful under
  MODEL-002, only unvalidated. Bumping the schema version would instead
  make every existing v1 body undecodable, including the cross-language
  golden vector, which is a migration cost with nothing to migrate.
  Evidence that no conforming artifact changes meaning: the golden vector
  and all 639 previously committed tests are unmoved. No spec text
  changes, because ADR-0037 already records this sweep as owed; §0.1
  bumps for requirement changes, and this adds no requirement.

- **WP-010: the verdict computation no longer picks one edge by sort
  order (issue #355).** Three arms of `node_verdict` selected a single
  edge with `.find()` where the body may present several — a
  signature's consumer, a node's producer, and the containment ascent
  to the device whose scope arm is inherited. Nothing bounds those
  in-degrees: the endpoint-pair table admits `Production` from both an
  encryption layer and an aggregate and containment of a signature or
  file system under both a device and a partition, and `Topology::build`
  enforces no cardinality rule. The selected edge was therefore decided
  by `NodeId` order — a SHA-256 digest over hashed naming fields — so an
  author who adds one lawful edge and grinds one hashed field chose
  which arm was consulted. Measured at `8e03e68`: a file system on a
  `RecognizedRemote` device went `Unsupported{InheritedDeviceScope}` to
  **`Clear`** behind a decoy local containment parent, and a volume
  produced by a live ZFS pool went `Unsupported` to **`Clear` on all ten
  mutating operations** behind a decoy encryption-layer producer, on
  bodies that assemble, encode, decode and rebuild with agreeing hashes.
  Each arm now folds `worst` over **every** matching edge, which is the
  module's own combinator and SAFE-005's posture: an added edge can only
  ever add refusal, and a body presenting one ancestry, one producer and
  one consumer answers exactly as before — no committed test moved. The
  containment ascent became a visited-set graph walk, so its termination
  rests on the walk rather than on the pair table's acyclicity. The
  consumer arm was already covered at the gate by `affected_set`'s own
  enumeration of consumers and is corrected for the verdict it reports.
  **Forbidding the multiplicity outright is deliberately not taken
  here** — that is a decode-boundary rule with its own MODEL-003 debt,
  and MODEL-002 gives membership unbounded in-degree on purpose, so it
  belongs to a decided act rather than to this fix.

- **WP-010: carried-content reach, and a bounded descent (ADR-0039,
  spec 13.0.0, closing issue #338's held half).** A mutating step's
  affected set now closes over the content its target carries, not only
  over the substrate it destroys, and containment descent is bounded per
  edge target by declared geometry rather than by the destroyed ranges.
  The measurement that ended the hold: `PlanStep::mutating_declared` —
  the constructor `parse_step` calls when a recorded plan body is
  re-validated, with no capability gate in that path — accepted a
  declared partial shrink truncating 128 MiB off a live ZFS vdev,
  because the freed tail missed the label's own bytes. The six
  operations that destroy nothing (`Grow`, `Create`, `Repair`, `Label`,
  `Uuid`, `Decrypt`) seeded no propagating class and gated `Clear` over
  a live pool; they refuse now by carrying reach from the target.
  **The bound can never remove reach** — it refuses only on a positive
  geometric contradiction and admits on every absence, mismatch or
  ambiguity — because extents are authored body content that nothing
  authenticates. Four earlier predicates were rejected on measured
  fatals: two could subtract reach on the strength of `extent_host`
  (one turned a live-pool refusal into `Clear` on a body whose node ids
  and hash are unchanged), and two false-refused ordinary disks, on a
  stale end-anchored mdraid superblock and on a sibling that merely
  lacks an extent fact. ADR-0018's theorem premise is generalized past
  the name `physical-device` and enumerated over the endpoint-pair
  table, discharging MODEL-002's standing obligation that it be
  re-proved as a property. Monotonicity in the declared ranges is
  restored, so ADR-0038's per-operation conservatism argument is
  superseded. Issue #338 closes; #347, #348 and #349 record what does
  not.

- **WP-010: release operations seed the protection closure (ADR-0038,
  on issue #338).** `affected_set`'s two entry routes are not
  equivalent — a node intersecting `destroyed` enters
  `range_destroyed` and propagates; one intersecting
  `written_table_extents` or `consumed` enters `affected` and reaches
  nothing further. Two corrections bring the code to ADR-0018's own
  text. **`Shrink` and `Move` now take the conservative entry** (the
  whole target extent in `destroyed`), because ADR-0018 names a
  shrink's truncated tail and a move's source extent as releases and
  only those two of the eight qualify; the entry is conservative
  rather than truthful because `canonical_ranges` takes no request
  parameters, and the truthful range was **measured** to leave a pool
  unreached where the whole-extent entry refuses. **Rule 3's
  membership half is ungated**, since ADR-0018 states it
  route-agnostically. Measured on the LUKS chain: Shrink and Move move
  from `Clear` to a refusal over a live ZFS pool; the six operations
  that destroy nothing stay `Clear`, pinned so the held half cannot
  drift. **Issue #338 stays open** on defect (b) and on those six; no
  spec version changes, since ADR-0018's text is untouched.

- **WP-060: `plan_set` no longer panics on an unsized create (issue
  #341).** `impossibility`'s `unreachable!` rests on the premise that
  every operation reaching reversal emission is one the path can plan.
  That premise was a property of `plan`'s statement order — it settles
  simulatability before emitting statements — and did **not** hold in
  `plan_set`, whose statement loop ran before its simulatability check.
  A single-request set carrying an unsized `Create`, on a target that
  clears the capability gate, aborted the process. Reproduced by
  execution before the fix. `plan_set` now settles simulatability for
  every request after ordering and before any step or statement is
  built, so the input refuses with **the same ground the
  single-request path gives it**, asserted equal by test. No delivered
  refusal's ground changes — graph refusals still precede it. The
  premise is now pinned for the whole operation vocabulary by
  `no_request_set_reaches_an_unplannable_statement`, the guard that
  would have caught this before it was filed.

### Added

- **spec-change 15.2.0: the frame rule is enforced (ADR-0046, resolving
  issue #333 and issue #401).** ADR-0037 decided that a range in a
  containment forest is expressed in that forest's root address space and
  held the enforcement — no green form, and a capture-side referent sweep
  owed first; ADR-0045 delivered the sweep, and this act delivers the
  front-runner ADR-0037 named, in the form it named. **At
  `TopologySnapshot::assemble`, and therefore at every decode, every
  extent's `host` is compared with the containment root the node's own
  name leads to** — `frame_root` walks the one naming field per kind
  `naming_referent_rule` classifies as naming a containment source (a
  partition's `parent_table`, a table's `parent`, a signature's or file
  system's `host`, a conflicting entry's `table`) until a kind that names
  none, so there is no second roster and no edge is consulted — and a
  mismatch refuses with `FactError::ExtentFrameDisagreesWithName { node,
  declared, derived }`, both frames named, the declared host never
  replaced. **A containment edge must nest a node in the parent its name
  embeds** (`ContainmentEdgeDisagreesWithName`, the strength ADR-0045
  held beside this issue), so a node's three positional claims — name,
  edge, extent — are pairwise consistent and ADR-0041's rule 6 collapses
  to its one live branch. A backing extent is carved out (no containment
  pair, the one open naming field, its range in its host's own space) and
  the carve-out is pinned: it assembles framed on any absorbed node
  (issue #365's). Measured: issue #333's flagship defeat — `root_on_zfs`
  with only the signature re-anchored on its member, pool unreached, wipe
  constructing — is unrepresentable on both construction paths, equal by
  value, and with both table edges removed; a body holding a device
  forest, a volume forest and a multipath forest at every depth,
  enumerated over every node × every candidate frame, admits exactly one
  frame per forest node (340 refused, 38 admitted); every containment
  edge re-sourced onto every other node refuses (59 by the name, 245 by
  the pair table first); six committed layouts validate; the golden
  vector is regenerated in the same act — `snapshot-full-captured` and
  `node-entry-backing-signature-7` move by exactly `extent_host` and
  `extent_start`, fourteen entries byte-identical, the TypeScript suite
  unchanged — under MODEL-003's explicit-rejection limb, `SCHEMA_VERSION`
  1, the debt ADR-0037 said travels with the enforcement discharged.
  Fifteen mutations, each proven applied, each killed but one whose
  premise (a child and parent reaching rule 6 in different frames) the
  two new rules make unconstructible, recorded. Third and last PR of the
  arc: issue #401's occupancy reading and WP-060's fixtures landed first.
  Priced and not taken, recorded in the ADR: a root-framed rule on a
  step's declared ranges (zero cost across every committed step; held on
  #365, since a range over a host-backed file's bytes is expressed in its
  file system's space). **Minor under §0.1**: the rule is 12.14.0's; the
  edge-name agreement, the occupancy readings and the Section 5 sentence
  are additions; what a body may say narrows, as under 13.1.0. It still
  does not make the reach sound.

- **spec-change 15.1.0: names are admitted where edges are (ADR-0045,
  resolving issue #354's kind half).** Eight node kinds embed a `NodeId`
  in their hashed name; PR #362 made every such referent resolve, and a
  referent that resolved to the *wrong kind* — `Partition.parent_table`
  naming the physical device, `Volume.producer` naming a partition —
  still built at assembly, at encode→decode→rebuild and in the planner's
  simulated rebuild. That is ADR-0037:146-150's stated harm, and the
  precondition (`:217`) #333's frame enforcement is held on. **Every
  naming referent must now resolve to an entry whose kind
  `endpoint_pair_allowed` admits as the source of the relation the field
  names** — `naming_referent_rule` maps each field to a *relation*
  (containment for a table's `parent`, a partition's `parent_table`, a
  signature's or file system's `host`, a conflicting entry's `table`;
  backing for an encryption layer's `backing_signature`; production or
  host-backing for a volume's `producer`), never to a list of kinds, so a
  row added to the table admits the name in the same act and there is
  no second authored list to drift; a backing extent's `host` is the one
  open field (no edge kind targets a backing extent) and must only
  resolve; an unclassified field admits nothing. `Topology::build`
  refuses with `ForbiddenNamingReferent` naming the node, kind, field,
  referent and the kind it resolved to, and the decode boundary inherits
  it. What held this after ADR-0044 was one population — content hosted
  on a multipath node, which no row admitted — measured to be an
  omission and a fail-open: an xfs naming `/dev/mapper/mpatha` as `host`
  built, no edge could carry it, its device-scope ascent found itself its
  own root, and all ten mutating gates were `Clear` over a device §2.1
  says never to mutate. **The pair table gains `multipath-node →
  {backing-signature, file-system, partition-table}`**, and content on a
  multipath node inherits its detection-only refusal, `Unsupported` ten
  times over. Measured: the workspace green with the check on (the golden
  vector and every planner rebuild included), the only red the test that
  pinned the held half, deliberately replaced; the naming enumeration
  admits 17 pairings and refuses 60; five honest layouts earlier
  candidates false-refused — a GPT in LUKS, a partitioned mdraid array,
  an xfs on multipath, a partitioned multipath node, a loop-backed volume
  — all build with their edges; seven mutations, each proven applied,
  each killed. MODEL-003 under the explicit-rejection limb, schema version
  unchanged, on #362's own reasoning. **Minor**: additions to MODEL-002,
  Section 5 and the §2.1 multipath entry. Two limits pinned and filed:
  device scope ascends the edge set, so a body omitting the multipath
  edge still gates its content `Clear` (the naming relation carries no
  scope — the escape ADR-0043 closed for release, open for scope); and a
  table inside a partition is expressible by no row and refused as a
  name — unrepresentable, not fail-open. ADR-0037:217's precondition is
  now satisfied; #333's enforcement is unblocked and is its own round.

- **spec-change 15.0.0: destruction carries through the cascade, and a
  volume carries a partition table (ADR-0044, resolving issue #360).**
  The endpoint-pair table gains `volume → partition-table` — the missing
  third of a set two-thirds present beside `volume → backing-signature`
  and `volume → file-system` — so a partitioned mdraid array
  (`aggregate → volume → table → md0p1`, over the existing production
  hop; aggregates stay out of the containment forest) and a GPT inside a
  LUKS-mapped volume are representable. ADR-0043 measured what the row
  alone would ship: wiping the array's member disk descended four hops
  and stopped at the table, because reach is not destruction and only a
  step's own target released. **Destruction now carries**: a step whose
  own destroyed ranges reach its target destroys it; destruction is
  carried from there along the same four arms, under the same geometric
  bound, as reach (`destroy`, `carry`); every destroyed node releases
  what its name-roster says it describes — a table its partitions,
  every other kind nothing — and a released partition is destroyed in
  turn, so a table below it releases too. Seeded by the target alone:
  never by a range that merely touches some other node (round 2's L1/L2
  guards hold unmoved), never by reach (`Label` on the member reaches
  the table and releases nothing). Measured: the chain refuses
  `Wipe(member)` through the pool, `CCCCCCCCCC` → `CCRRCCCRCR` on the
  member disk, the array's superblock and the table; the GPT-in-LUKS
  layout refuses the LUKS partition's wipe and keeps the ESP beside it
  10/10 `Clear`; a partitioned array carrying a plain ext4 constructs;
  every existing layout byte-identical; seven mutations, each proven
  applied, each killed. One named limit, pinned as a committed row: an
  extentless target — a volume, an aggregate — declares no destroyed
  range, so its own wipe is not seen destroyed and reaches the table it
  carries as content only; the whole-frame canonical entry that would
  close it was measured green across the workspace and **held**, filed,
  because it moves `canonical_ranges` and the planner's simulation on a
  population no planner test covers. **Major**: §2.1:113's release clause
  and ADR-0018's theorem both said the release follows the step's
  *target*; both now say the target and the destruction carried from it;
  MODEL-002's chain gains the volume-carries-a-table sentence. #354's
  kind half stays held (now on the multipath population alone), the
  `honest_layouts` test re-spelled through the volume with edges.

- **spec-change 14.0.0: a destroyed partition table releases the
  partitions it describes (ADR-0043, resolving issue #347 on its third
  round).** ADR-0018 defines the destroyed class as releases — content
  that "ceases to be referenced" — and the closure reached none of the
  partitions a destroyed table describes: on a disk carrying a live ZFS
  vdev, `Wipe(table)` was 10/10 `Clear` with the pool never consulted.
  Two designs died inside the closure on the table's own authored
  geometry — round 1's coverage test (fail-open on one byte of
  inflation), round 2's intersection test (every sibling captured from a
  partition-target step on the BIOS-boot layout) — and round 2's panel
  measured that no predicate over `Facts.extents` can be repaired into
  shape. **The release is structural now**: a step whose *target* is a
  partition table and whose own destroyed ranges reach it destroys the
  table, and a destroyed table releases every partition whose *name*
  says it describes it (`Partition.parent_table`, `released_by_table`,
  quantified over the naming roster). Never a table some other step's
  range touches, never coverage of the table's extent, never the edge
  set; a `ConflictingTableEntry` names a table and is not released by
  it. Measured on the panel's own fatal shapes: L1's partition-target
  row is 10/10 `Clear`; L2's one-byte inflation no longer refuses
  `Wipe(esp)`; M3's hybrid-MBR wipe stays `Clear`; M5's omitted-edge
  escape is closed; the plain disk still constructs; every
  non-table-target row on five layouts is byte-identical to before. One
  priced limit, pinned beside the honest spelling on `bios_boot_gpt`: a
  table-target step destroying any byte the body attributes to the table
  releases, fail-closed — the closure cannot tell one GPT entry from the
  header, and nothing delivered emits that spelling. Six mutations, each
  proven applied, each killed (release on target identity alone is
  killed by four pre-existing guards). **Major**: ADR-0018's theorem is
  amended again and §2.1:113 changes — including the correction of
  ADR-0042's "never a descent source" mispricing, recorded rather than
  smoothed. The #360 chain needs the release to propagate through the
  cascade; measured to work and cut as an uncovered clause; #360's
  remainder, with `Wipe(volume)`. `Containment`'s doc comment now says
  what the two table pairs are (round 1's §2 finding).

- **spec-change 13.1.0: the body's facts are validated against its
  topology at assembly (ADR-0041, resolving issues #349 and #356 as
  filed).** `TopologySnapshot::assemble` — the one path both the
  in-process constructors and the decode boundary run through — now
  refuses, with the node named: a fact keyed by an address no entry
  carries (`OrphanFact`); a fact on a kind that does not carry it
  (`MisplacedFact`, the decode path's four placement checks moved here
  so `assemble` can no longer accept what `from_canonical_body`
  refuses); an extent framed on an unabsorbed address
  (`UnresolvedExtentHost`); a zero-length extent (`ZeroLengthExtent` —
  it can intersect nothing, so a label declared this way was invisible
  to the byte scan); an extent whose `start + length` overflows
  (`ExtentOverflows`); and a containment child lying outside its parent
  where the pair is geometric and the frames are comparable
  (`ExtentOutsideContainmentParent` — issue #356's measured escape, a
  signature the edge nests in `[0, 100 MiB)` and the fact puts at
  500 MiB, is refused at assembly instead of approving the deletion of
  the partition carrying a live pool's label). `partition-table` →
  `partition` and → `conflicting-table-entry` are structural: the
  table's extent is its own header bytes, not the region it governs, and
  a blanket child-within-parent rule was measured to refuse every
  committed GPT disk and a pre-existing step test. Left alone by design:
  an incomparable frame (ADR-0037's held enforcement), a parent with no
  extent (the golden vector's shape), an absent extent (#356's
  absent-extent spelling was re-measured under the act and **still
  constructs** — issue #319's class, not claimed here), sibling overlap,
  and a device's extent against its own `total_bytes`. Under MODEL-003's
  explicit-rejection limb with `SCHEMA_VERSION` left at 1 (PR #362's
  precedent); `SnapshotSchemaError::MisplacedFact` is retired, the
  boundary's refusal now carrying the constructor's own error, equal by
  value — a committed test. Twelve mutations, each proven applied by
  `git diff`, each killed. The `bios_boot_gpt` overlapping-geometry
  fixture the issue-347 round-2 panel required is committed with its
  `f11`/`f12` assertions lifted as tests. WP-060's occupancy test was
  adjusted first under its own grant (PR #377) so this lands green.
  **The reach is not made sound by this**: validation buys
  self-consistency, and #319, #333 and #347 stay open as recorded in the
  ADR.

- **spec-change 12.14.0: the containment-frame anchoring rule
  (ADR-0037, on issue #333).** A range in a containment forest is
  expressed in that forest's root address space, and `HostRange.host`
  names that root. The anchoring was never validated while
  `HostRange::intersects` opens on it, and three committed fixtures
  disagreed — including the cross-language golden vector. Measured:
  re-anchoring one signature, every extent still present, left a ZFS
  pool unreached and a whole-device wipe constructing. Two rival
  readings and two enforcement designs were rejected on measured
  fatals and recorded in the ADR. **Enforcement is deliberately
  held** — none has been measured green, and the naming-field-derived
  front-runner is recorded as a candidate in derive-and-**compare**
  form only. Priced knowingly: the rule makes coordinates uniform but
  **does not make the reach sound**, and the golden vector plus
  `plan_tests.rs` stay unlawful under it until the enforcement PR
  regenerates them. **Issue #319's authorization half is blocked on
  issue #338**, not on #333 — correcting what ADR-0036 recorded. No
  code changes.

### Changed

- **WP-060: two planner tests re-based on ranges that exist, ahead of
  issue #353's act.** `an_unordered_overlap_refuses_with_both_steps_named`
  now asserts the graph refusal on the ordered chain's two wipes with
  the dependency removed — destroyed ranges that truthfully overlap. It
  used to pair a wipe with an *unsized* create on one device, which
  overlapped only because the create's canonical entry wrote the parent
  device wholesale, the over-claim §2.1 forbids and the act removes; an
  unsized create's honest ground is the simulate refusal it already
  gets. And a new test,
  `a_partition_write_still_touches_its_disk_for_the_protection_arms`,
  pins what nothing pinned: a `Label` on a partition carries its disk's
  PART-013 parse-backup obligation and refuses on Indeterminate media,
  both derived from the step's declared range whose host is the disk.
  The domain suite is blind to that entry (dropping every write entry
  survives there, measured), so the consumer that depends on it holds
  it. Green at HEAD; green with the act merged (661 tests, 0 failed);
  and the new test is what kills the declare-nothing mutation.

- **WP-060: the occupancy ground is a function of the located range.**
  `occupancy_ground(located, host, declared_start)` is extracted from
  `unaccounted_occupant` and every `OccupancyGround` arm is asserted on
  it directly, `RangeIsEmpty` included. The behaviour is unchanged; the
  reason is sequencing. The body-validity act on issues #349 and #356
  (ADR-0041, WP-010, the next PR) refuses at assembly two shapes this
  package's occupancy test built through `TopologySnapshot::assemble` —
  a zero-length extent and an extent framed on an address the snapshot
  does not absorb — so those grounds could no longer be reached through
  a snapshot, and the test would have gone red under the change it does
  not own. The empty-range ground now lives where it can be measured
  regardless of which shapes a snapshot lets through, and the other-host
  case names a device the snapshot absorbs, which is a valid body under
  both regimes. Green at HEAD and green with the act applied, measured
  on a throwaway merge of the two before either landed. The solver's own
  defensive reading of an empty range is kept, deliberately: a
  consumer's guard should not depend on a producer's promise.

- **WP-020: the r14 re-pin.** WP-060 increment 10 (PR #336) landed Rust
  after `b50dd19` and tripped increment 2e's stopping condition for the
  fifteenth time — the ninth from outside the package. One sitting
  (VMID 9437, 2026-08-13 UTC) re-took all three acceptances at the
  arc's head on `1f9f2c7`: 2e (`configured_legs=2`,
  `clean_observation_bytes=4096`, every confirmation true), the 2h
  single-range suite (`ranges_written=1`, 8 bytes), and the 2j
  two-range suite (`ranges_written=2`, 16 bytes, both GPT signatures
  erased and restored) — identical value sets to r13, the full
  eleven-control refusal set, fixture digests equal before and after.
  Custody run 21, transcript digest agreeing across guest, host, and
  workstation; teardown verified 2026-08-13T22:35:21Z with no config,
  volume, LVM volume, or snapshot remaining. The records are re-pinned
  at `1f9f2c7`. Unlike the r13 arc, this arc's plan recorded the
  sitting it would owe **before the first merge**.

### Fixed

- **WP-060 increment 10: the scheme's own regions, and located
  occupancy (ADR-0036, spec 12.13.0, on issue #319).** `free_extents`
  now subtracts the regions a host's declared table schemes claim at
  each end, derived from the table node's own hashed `TableRole` and
  bounded rather than measured, and refuses where a scheme cannot be
  named or where a partition the authenticated names place on the host
  is not one the subtraction removes. The filed defect is closed: a
  create of exactly `DEFAULT_ALIGNMENT` no longer lands at offset 0
  over the protective MBR and the GPT header. Two further §11.2
  defects close with it — a host extent exceeding the size its own
  naming fields declare, and a child extent leaving its host. One
  delivered assertion is overturned:
  `free_extents_are_the_hosts_minus_its_children`'s `(0,
  DEFAULT_ALIGNMENT)` tuple, a committed claim that the fail-open was
  correct; the fixture and the test name survive, and the test now
  asserts the extent-less-ness it had been resting on silently.
  `shrink_reduction` is unaffected. **Issue #319's authorization half
  stays open**, blocked on issue #333.

### Added

- **spec-change 12.13.0: the scheme's own regions, and located
  occupancy (ADR-0036, on issue #319).** A measured fail-open in the
  delivered free-extent derivation: at `ecb3dc6` a `place_create` of
  exactly the 1 MiB default against the delivered solver fixture
  returned `start=0 … Aligned`, placing a partition over the
  protective MBR and the GPT header, because that fixture's
  partition-table node carries a containment edge and no extent.
  INV-004's derivation now withholds the regions a host's declared
  table schemes claim at each end — a **bound, never a measurement**,
  since sector size, entry count, and entry size do not reach it — and
  is not presented at all where the scheme cannot be named or where an
  authenticated partition of the address space is not one the
  derivation subtracts. PART-009 gains a third named structural edge,
  the low boundary of a scheme-claimed tail region. The claim derives
  from the table node's own hashed `TableRole`; occupancy is read from
  the authenticated names, never from containment edges. Rejected on
  measured fatals and recorded in the ADR: keying the guard on extent
  presence, and refusing outright on an unlocated table. Two further
  §11.2 defects close with it — a host extent exceeding its own
  declared size, and a child extent leaving its host. Minor under
  §0.1, with the 13.0.0 counter-argument recorded and declined.
  **Issue #319's authorization half stays open**, blocked on #333;
  the `crates/planner` implementation is WP-060 increment 10, landing
  after.
- **spec-change 12.12.0: the Linux mmc-class designation extension
  (ADR-0035).** ADR-0034's revisit condition fired in its sanctioned
  direction the same day: the S5 sitting measured a serial source for
  an undesignated attachment class, and the extension lands with its
  rows. The (Linux, native MMC-attached block device) serial cell
  designates the linked mmc node's `cid` attribute — the full
  register, verbatim, newline included — with the attachment class
  defined by the S5c-measured structural resolution. The kernel's
  `serial`/PSN projection is the recorded rejection (a transformation
  with strictly weaker collision resistance). WWN undesignated for
  the class; every ADR-0034 rule applies unchanged; eMMC and
  second-host captures are evidence obligations, not scope-outs.
  Linux thereby gains its first medium-attributable naming route,
  sitting opposite the S4/S5-measured bridge-collision families in
  the same table — the shape SI-28's round five needs, which these
  rows enable and do not decide.

- **spec-change 12.11.0: the Linux naming-source designations
  (ADR-0034).** The normative act ADR-0019 anticipated, made on the
  #318 readback rows and nothing else. The per-platform source table
  gains a (platform, attachment class) key with exactly one designated
  cell: a Linux USB-attached block device's serial is the `serial`
  attribute of its nearest USB device-node sysfs ancestor, bytes
  verbatim as read, trailing newline included. WWN and every other
  Linux attachment class are undesignated — fields absent, fail-closed
  through ADR-0019's existing weak-name and collision-group machinery.
  Two previously undefined naming outcomes close in the same act: a
  measured absence (`ObservedAbsent`) joins `unavailable`
  (operand-eligible weak name), and a failed read of a designated
  source (the measured `device/wwid` `ENXIO` shape) yields an
  indeterminate non-operand that still appears in the body. Naming
  inputs must flow through a bytes-preserving path; the delivered
  `read_attribute` is not one, and the bytes seam is WP-L100
  increment 3's first delivery obligation. Rejected and recorded:
  udev `ID_SERIAL_SHORT`, sysfs `device/serial` (zero observations on
  any Linux host), `device/wwid` for WWN, and holding for a complete
  table. Decided on the adversarially reviewed round of 2026-08-13
  under the decision owner's directive; merging is not acceptance.

- **WP-020: the r13 sitting — all three acceptances re-taken on
  `b50dd19`, the stopping condition re-pinned there, closing the
  WP-L100 arc.** The eighth trip from outside the package (the arc's
  three Rust merges, PRs #314, #316, and #317: the Linux adapter's
  contract-and-seam increment, its devices-and-identity increment, and
  its recorded corrections), covered by a single sitting at the arc's
  head. Unlike the two arcs before it, the WP-L100 arc's plan recorded
  no sitting economics — only that its Markdown-only governance step
  owed none — so the one-sitting choice was made at re-take time on
  the r11/r12 precedent and the record states it as this re-pin's own
  decision rather than the arc's plan. One sitting on 2026-08-13
  (UTC), one fresh disposable Proxmox-hosted VM (VMID 9436, kernel
  5.15.0-186-generic), the r12 runbook copied to r13 with header prose
  alone changed: the full eleven-control refusal set refused, 2e
  passed (its thirteenth re-take, identical value set), the 2h suite
  passed (`fixtures_executed=1`, `ranges_written=1`,
  `contracted_bytes_written=8`), and the 2j suite passed
  (`ranges_written=2`, `contracted_bytes_written=16`, both signatures
  restored). Fixtures byte-identical to the catalogue, loop table
  empty, teardown verified with nothing remaining
  (2026-08-13T14:27:20Z), custody run 20 with transcript digests
  agreeing across guest, host, and workstation (`81e009db…6b5b`).

### Fixed

- **WP-L100: two corrections to increment 2, recorded rather than
  edited away.** Both were found by the adversarial round that ran
  before increment 3, and both are the same class — a claim stated at
  more strength than its source licenses.
  First, increment 2's own text said the no-identity-record rule was
  "held by a test over the crate's public surface", and it shipped
  without one: `DeviceIdentity` appeared nowhere but a doc comment. The
  guard now exists — a text scan over every shipped module for the
  construction, derivation, and import spellings, leaving the crate doc
  free to name the type in prose in order to say why it is absent — and
  it is mutation-verified: an identity-strength reference entering the
  crate fails it.
  Second, `schemas/adapter-linux/fields.md` gave the `device/serial`
  attribute the strength `real-hardware`, resting on an observability
  row that bundles `{vendor, model, wwid, serial}` as one read set and
  attributes the observed value to sysfs generically. That a serial came
  from sysfs is established; that *this attribute* produced it is a
  natural reading rather than a transcription. Ordinarily academic —
  not here, because ADR-0019 makes the choice of a single named source
  per platform a normative, hash-visible act, so a naming designation
  cannot rest on a bundled row. The row is restated at the strength the
  record supports, the reason is recorded in the document's own
  what-this-does-not-establish section, and the readback is filed as an
  obligation on WP-035 — the transcript is archived with its digest
  recorded and holds the instrument's exact paths, so closing it is a
  readback rather than a new sitting. **Until it lands, the Linux
  naming-source designation ADR-0019 owes cannot be made, and WP-L100
  increment 3 cannot address a node.** No behaviour changes: the crate
  already constructed no identity record and already read the same
  attribute.

### Added

- **WP-L100 increment 2: devices and their identity material.**
  `crates/adapter-linux`'s `devices` module: whole-device enumeration
  admitted **only** on a positively determined absence of the
  `partition` attribute, so an unreadable attribute admits nothing —
  the successful-read reading fails open, and a read error would then
  promote a partition into the device list where its sector count
  would be reported as a device capacity. Nine sysfs fields and six
  database keys, each an attributed observation under its own
  `interface:native-property` key, electing nothing: the attribute
  layer's serial and the database's serial-shaped key are two
  properties because they are two interfaces' different answers — the
  record shows one device reporting `S3Z9NB0K` through the first and
  `ata-Samsung_S3Z9NB0K` through the second — and merging them would
  manufacture a `conflicting` confidence out of values never in
  conflict. ADR-C4's separation now extends across the database half
  through a second evidence-token producer: a record that does not
  exist makes every key `unavailable`, while a key missing from a
  record that does exist is a positively determined absence, because
  calling the first absent would claim the database answered and said
  nothing.
  **The ADR-0018 transport answer is `Unrecognized` for every device,
  and that is imported obligation 6's own terms rather than a
  shortfall.** That ADR's evidence obligation — "fabric-versus-local
  transport discrimination rows per platform for each listed local
  transport" — is outstanding on every platform; no value of any
  classifying key is recorded anywhere in this repository for any
  Linux host; and five of the six positive-local classes have no Linux
  measurement of any kind. A table mapping interface strings to
  classes could come only from vendor documentation, the one thing
  this package's evidence rule forbids. ADR-0018 prices exactly this
  availability cost under "Negative, accepted knowingly", and the
  answer resolves to `Indeterminate` at the protection closure, never
  `Permitted`; a source-text guard holds that no positive class is
  constructible in the module at all.
  Nothing here builds a `NodeId`, a `protection::Facts`, or a
  snapshot: those are keyed by ADR-0019 derived addresses, whose rules
  are increment 3's imported obligation, so keying a map today would
  be naming without them. The published reach moves to
  `implemented-reaches-no-table-state` — the roster carries no
  partition-table key, pinned by test — with every cell still negative
  on a **re-decided** not-measured basis: a citation's vocabulary is
  observability headings and no Linux heading exists for `mbr` or
  `apple-partition-map`, so `measured` is unexecutable for two of the
  six. **Correcting increment 1**, whose reach text said the `udev`
  database carries `ID_PART_TABLE_TYPE`: that token appears under the
  direct-signature-probe column, an interface measured *denied* to the
  unprivileged client, and those probes ran over regular files rather
  than devices — the conclusion it supported is unchanged and better
  supported without it. `schemas/adapter-linux/fields.md` publishes
  the roster with, per field, the observability row that supports
  reading it or an explicit none; `removable` has no row on any Linux
  host and `queue/physical_block_size` none on real hardware, so both
  are read with nothing derived from them and both rows are filed as
  obligations on WP-035, along with the transport-discriminating row,
  the whole-device discriminator, and the `size` unit convention that
  blocks increment 3. Twenty-two tests and one compile-fail proof,
  none platform-gated. Seven mutants (fail-open admission, a missing
  record reported as absence, a positive transport class, the
  interface qualifier dropped from property keys, a shipped module
  dropped from the structural guards, a partition-table key entering
  the roster, the contract word left stale) were each killed by a
  named test before proposal, two of them by two tests each. This is a
  Rust merge: the WP-020 2e stopping condition pinned at `77b0dd7`
  trips, and the re-pin sitting is run and recorded separately under
  WP-020's ownership.

- **WP-L100 increment 1: the contract, its seam, and its published
  reach.** `crates/adapter-linux` joins the workspace (workspace lints,
  `unsafe_code` denied by inheritance — this adapter takes the denial
  rather than SAFE-009's adapter exception, because it opens nothing
  that would need one), depending on `partman-domain` deliberately:
  MODEL-004's observations are the domain's own ADR-C4 vocabulary, not
  a second one beside it that would drift. The injected read seam
  returns **bytes**, which is a deliberate variation on WP-035's
  precedent and the reason this increment can test what it declares —
  WP-035 enforces its per-value bound inside its production
  implementation, where a Tier-1 fake cannot reach it, so that bound
  has no test. Here every rule is decided above the seam: the entry
  bound and the byte bound each refuse with the count seen rather than
  truncating (a prefix is byte-for-byte indistinguishable from a
  complete read of that length), non-UTF-8 bytes refuse rather than
  being lossily converted, and exactly one trailing newline is
  stripped — trimming all trailing whitespace turns a padded SCSI
  vendor into an empty string, which then reads as a positively
  determined absence, the ADR-C4 violation WP-035 records having made.
  ADR-C4's separation is structural rather than documented: reading an
  attribute requires an `InterfaceAnswered` token that only a
  successful listing produces and that no caller can name into
  existence (compile-fail-proven), so a missing attribute cannot be
  read as an absence by a caller who cannot show the interface
  answered, and an interface that did not answer is `unavailable`,
  never an empty listing. The interface decides MODEL-004's method,
  and that is a decision rather than a transcription: `sysfs` is
  `Direct` and derives `authoritative`, while a `udev` database value —
  computed by root's udevd at device-add time and read here from its
  cache — is `Heuristic` and derives `inferred`, because calling a
  cached third-party computation authoritative would let one stale
  record outrank nothing. The INV-003 reach declaration (ADR-0013)
  publishes one cell per state in INV-003's own order, fixed-size so a
  missing cell is a compile error rather than an omitted `no`, with
  every cell negative on a **deliberately** not-measured basis:
  `docs/quality/observability.md` §Linux does hold measured rows here,
  but a measured basis must be a measurement about *this* contract and
  this contract reads no field yet — the same rows record that the
  `udev` database carries `ID_PART_TABLE_TYPE`, so which fields
  increment 2 lists is what decides reachability, and increment 2
  earns the basis rather than defaulting it. No identity record is
  emitted and no strength derived: `DeviceIdentity` carries a required
  table state, every variant of which is a determination INV-003
  forbids this contract making, so the record binds at validation from
  the helper's own re-discovery. `schemas/adapter-linux/reach.md`
  records the format, pinned to the crate's vocabulary by test.
  Fourteen tests and one compile-fail proof, none platform-gated —
  the crate is pure over the seam, so the whole suite runs on all
  three CI legs rather than only where a defect would be least
  convenient to find; traceability converted to `generated`. Seven
  mutants (the entry bound raised past its own fixture, the byte bound
  doubled, trailing whitespace trimmed instead of one newline,
  non-UTF-8 converted lossily, a cached database value called direct,
  an absent interface listed as empty, a failed read reported as an
  absence) were each killed by a named test before proposal, and the
  fail-open one was killed twice — the evidence token catches it a
  second time. This is a Rust merge: the WP-020 2e stopping condition
  pinned at `77b0dd7` trips, the eighth trip from outside that
  package, and the re-pin sitting is run and recorded separately under
  WP-020's ownership, as every re-take has been.

- **spec-change 12.10.0: SI-14 is resolved — a derived property is a
  derivation, not an observation (ADR-0033).** The register-residue
  arc's one ripe issue: its "Later (WP-050)" gate had been reached and
  passed by delivered work that embodies the answer (ADR-C4's
  never-stored confidence, the WP-060 solver's free extents computed
  from body-carried authenticated extents, ADR-0023's rejected
  duplicate alignment-fact field), so the absence of a
  derived-confidence rule is recorded as the rule itself. INV-004
  gains the scoping clause — free extents and alignment are recomputed
  at use from the detected inputs they name, never stored, no
  observation set or confidence of their own — and one new
  prohibition: a derivation over an input whose observation set
  derives `unavailable` or `conflicting` is not presentable, the
  input's own state surfaced instead. No fifth confidence value, no
  composition algebra; presentation obligations for the future
  WP-W100/WP-L100/WP-M100 inventory surfaces are recorded in the ADR
  and land in those assignments at creation. The companion residue
  sweep verified the other three without edits — SI-13 stays Later
  (WP-L110: identities bind at validation; aggregates are not
  plannable write targets, structurally), and SI-28's floor and
  SI-37's evidence clause stand unmet by any existing measurement,
  their relaxation campaigns deliberate future arcs rather than
  documentation debt. Minor under §0.1; the major counter-argument is
  recorded in the ADR.

- **WP-020: the r12 sitting — all three acceptances re-taken on
  `77b0dd7`, the stopping condition re-pinned there, closing the
  PLAN-005 cancellation arc.** The seventh trip from outside the
  package (the arc's three Rust merges, PRs #307–#309: WP-010
  plan-body slices 3n and 3o with the jointly-sequenced WP-060
  increment 9), covered by a single sitting at the arc's head per the
  arc's own recorded one-sitting economics. One sitting on 2026-08-13
  (UTC), one fresh disposable Proxmox-hosted VM (VMID 9435, kernel
  5.15.0-186-generic), the r11 runbook copied to r12 with header prose
  alone changed: the full eleven-control refusal set refused, 2e
  passed (its twelfth re-take, identical value set), the 2h suite
  passed (`fixtures_executed=1`, `ranges_written=1`,
  `contracted_bytes_written=8`), and the 2j suite passed
  (`ranges_written=2`, `contracted_bytes_written=16`, both signatures
  restored). Fixtures byte-identical to the catalogue, loop table
  empty, teardown verified with nothing remaining
  (2026-08-13T02:36:40Z). One operational deviation recorded rather
  than hidden: the sitting's first invocation went through `sudo`, and
  2e's own Tier-1 redaction sweep refused it on the injected
  `SUDO_USER` value — the sweep doing its job on an operator-injected
  variable — so the void transcript is retained (custody run 18), the
  guest was rolled back to the `pre-acceptance` snapshot, and the
  cited run was root-invoked from the snapshot state (custody run 19:
  transcript digests agreeing across guest, host, and workstation,
  `b42574de…d4d5`).

- **WP-010 slice 3o: the version-1 plan body retired.** The reviewed
  change plan-body.md §0 has promised since slice 3l: with the planner
  on the linked form since WP-060 increment 6 and the version-4
  carriage landed by slice 3n, version 1's last emitters were this
  crate's own tests and its two fixture vectors. Both are migrated —
  the tests assemble linked bodies, and the identity-bound vector's
  SAFE-003 coverage survives as `plan-v4-bound-identity-wipe` (the
  redundant bare-wipe vector is dropped, every other vector
  byte-identical) — so `OperationPlan::assemble` is removed, the
  reversal linkage becomes a required field rather than an `Option` (a
  plan without a linkage is unconstructible, not refused; the
  `reversal()` accessor keeps its `Option` shape for its consumers),
  `PlanError::UncarriedPreconditions` goes with the form that needed
  it, and version 1 refuses at decode like every other retired version
  (MODEL-003's explicit-migration discipline). The committed
  regression pins both spellings: a downgraded version byte and the
  full v1 shape (no reversal, no preconditions, no class, no
  cancellation) refuse as `WrongSchemaVersion` — the version gate
  comes first. Two mutants (the version-1 decode arm restored, the
  retired constructor's linkage-freedom restored via an
  always-backed `reversible_backed`) each failed a named test before
  proposal.

- **WP-060 increment 9: the cancellation vocabulary — PLAN-005
  delivered on the recorded cancellation-class decision.** Every step
  the planner emits now declares exactly one of PLAN-005's three words
  in the hashed version-4 body, on WP-010 slice 3n's jointly-sequenced
  schema change. The class is a per-family stated declaration
  (`cancellation_class`, the interruption-profile precedent) under the
  decision recorded in the WP-060 assignment in the WP-035
  route-decision shape — never a derivation from the interruption
  profile, since spec 12.3.0 records cannot-stop and cannot-unwind as
  independent facts in both directions, and the partition fixture
  exhibits the combinations: the entry write cannot stop yet unwinds
  trivially, the journaled chunk copy stops at its declared
  checkpoints yet is unflagged, the wipe can neither stop nor unwind.
  `Move`/`Copy` are stated `checkpoint-cancellable` on PART-005's
  durable progress map and ACC-012's declared checkpoint — for the
  family, before the planner emits it — and every family the planner
  emits today sits on the `non-cancellable` floor, each earning more
  only through the decision's named revisit conditions (the overwrite
  wipe's measured safe-stop story first among them). The derivation is
  wired explicitly into all four step-construction sites, never left
  to the carriage's default; the emitted body carries the declaration
  end to end and it survives the boundary's recompute; `CancelClaim`
  and ADR-0025's coupling rule are untouched; the UI's must-not-offer
  law and EXE-004's acknowledgment are recorded as the UI and executor
  packages' boundaries. Three mutants (the family statement flipped to
  the floor, a call site bypassing the derivation with an off-floor
  constant, the floor arm flipped off the floor) each failed a named
  test; the floor-constant bypass is semantically equivalent today and
  recorded in the assignment rather than claimed killable. With this,
  every item the WP-060 assignment's beyond-list named is delivered.

- **WP-010 slice 3n: plan body version 4 — PLAN-005's cancellation
  declaration enters the hashed body.** Jointly sequenced with WP-060
  increment 9 under the WP-060 recorded cancellation-class decision
  (2026-08-12): every version-4 step carries a required `cancellation`
  field, closed at PLAN-005's own three words (`cancellable`,
  `checkpoint-cancellable`, `non-cancellable`), typed in the domain as
  `Cancellation` with the fail-closed `non-cancellable` floor as its
  default — the delegating constructors sit on the floor, and only the
  fully-declared form (`PlanStep::mutating_declared`) claims more. An
  unknown spelling refuses, the missing field refuses, and the
  declaration is independent of `irreversible-after-start` in both
  directions (spec 12.3.0), so no coupling law was invented. A draft
  step's declaration is pinned to the floor exactly as its class is
  pinned to `ordinary`: the emitted draft carries `non-cancellable`,
  and a draft body claiming more refuses at decode — a draft family
  off the floor is a future reviewed extension of the recorded
  decision. **Version 3 is retired**: one change window old, no
  emitter outside it, no surviving artifact — its vectors regenerated
  as version 4 (`plan-v4-wipe-impossible`,
  `plan-v4-forward-create-draft-linked`,
  `plan-v4-table-repair-acknowledged`, and the re-encoded
  `draft-create-reversal`) in the same change, reproduced by the
  TypeScript suite unchanged — and refused at decode (MODEL-003's
  explicit-migration discipline; the v2 precedent). Version 1 stays
  emitted and accepted; its retirement remains its own reviewed
  change. Four mutants (default flipped off the floor, the version-3
  refusal dropped, an unknown spelling accepted, the draft floor pin
  dropped) each failed a named test before proposal.

- **WP-020: the r11 sitting — all three acceptances re-taken on
  `667f6aa`, the stopping condition re-pinned there, closing the
  WP-060 unlock arc.** The sixth trip from outside the package, and
  the first covered by a single sitting over multiple merges: the
  WP-060 unlock arc (PRs #299–#304 — increments 5 through 8 with the
  jointly-sequenced WP-010 plan-body slices 3l and 3m) recorded up
  front that its six Rust merges would trip the condition once and be
  re-measured once at the arc's head, and that is what ran. One
  sitting on 2026-08-13 (UTC), one fresh disposable Proxmox-hosted VM
  (VMID 9434, kernel 5.15.0-186-generic, no reboot during
  provisioning), the r10 runbook copied to r11 with header prose and
  two carried-over VMID labels corrected: the full eleven-control
  refusal set refused, 2e passed (its eleventh re-take — fifteen
  passes across twelve guests, identical value set), the 2h suite
  passed (its ninth re-take, `fixtures_executed=1`, `ranges_written=1`,
  `contracted_bytes_written=8`), and the 2j suite passed its sixth
  re-take (`ranges_written=2`, `contracted_bytes_written=16`, both
  signatures restored). Fixtures byte-identical to the catalogue, loop
  table empty, teardown verified with nothing remaining
  (2026-08-13T00:53:24Z). Custody run 17: transcript digests agreeing
  across guest, host, and workstation (`10297fab…4b8d`).

- **WP-060 increment 8: the combination unlock — ADR-0025's criterion
  derived, the contested combination constructing.** The
  `irreversible-after-start` flag is now derived from a typed
  criterion (`InterruptionProfile`) rather than withheld: the wipe,
  the shrink, and the table repair carry it (in-place destruction and
  rewriting — for the repair, the raw capture is a recovery substrate,
  not an unwind), entry-level writes do not, and PART-005's journaled
  chunk copy is stated unflagged for the family before the planner
  emits it — the partition fixtures ADR-0025 names. Severity 1 plus
  the flag assembles through the sole constructors exactly when its
  truthful reversal draft stands beside it — endpoints fully undoable,
  mid-window roll-forward-only — and still refuses with no draft: the
  flag changes nothing about ADR-0022's rule. The coupling rule lands
  unconstructible rather than discouraged (`CancelClaim::no_writes`
  has no path for a flagged step after its first write; cannot-stop
  and cannot-unwind independent both ways), and `plan_flags` delivers
  PLAN-004's plan-level union so the ceremony's inputs are derivable —
  the flagged severity-1 plan's union is nonempty, binding the
  interactive ceremony under ADR-0021's closed rule, with the tier's
  computation and enforcement recorded as the helper packages'
  boundary. Three mutants (disabled flag derivation, disabled coupling
  rule, broken union operator) each failed a named test. With this,
  all four unlock increments the resolved register gates opened are
  delivered.

- **WP-060 increment 7: the backup family — PART-013's planning half,
  state-selected per ADR-0024.** Every plan now carries its derived
  protection obligations, one per table-bearing device it touches, arm
  selected by the helper's authored table state: `Present` → the
  parse-backup obligation; `Absent` → the journaled determination, a
  value not a skip, with no user acknowledgement demanded or carried;
  `Indeterminate` → an ordinary operation refuses typed **before any
  obligation is computed** (SAFE-005's planner half — PART-013 never
  reached), while the typed table-repair family plans over exactly
  that media through its own entry point. `plan_repair`'s step is
  `table-repair` class, its write targets exactly the located table
  regions (fail-closed both ways: no located table refuses with no
  invented regions, and a positively determined state refuses — the
  family exists for `Indeterminate` tables), its raw-capture
  obligation names exactly those regions, its simulation drops the
  stamp, and its reversal is the pre-state-preserved statement. The
  capture-impossible arm proceeds only under the plan-creation
  acknowledgement naming the exact uncapturable regions — riding the
  hashed body, flipping the obligation to acknowledged-unpreserved,
  and unconstructible outside the family by slice 3m's constructor
  law. Obligations are derived at every computation, never stored; the
  journal's protection record is the durable artifact and the REC-001
  byte round-trip stays WP-R100's, both recorded as boundaries. Four
  mutants (disabled guard, swapped Present arm, dropped raw-capture
  exactness, dropped acknowledgement path) each failed a named test.

- **WP-010 slice 3m: plan body version 3 — the ADR-0024
  protection-family schema change, jointly sequenced with WP-060's
  backup-family increment.** Every linked step now carries its typed
  `class` (`ordinary` | `table-repair`): the REC-001 repair family is a
  class, never an intent flag, per the safety-is-computed discipline.
  The acknowledgment vocabulary closes at four:
  `uncapturable-regions` lands as ADR-0024's plan-creation
  capture-impossible acknowledgment, naming exact well-formed regions
  (strictly ascending, non-overlapping, nonzero, on the covered
  device), and `identity-bound-restore`'s arm exists at last — both
  table-state kinds are lawful exactly on a table-repair step over a
  device whose authored table state is `Indeterminate`, and
  unconstructible outside the typed family: on an ordinary step, over a
  Present state, or with malformed regions, the sole constructor
  refuses, and the same law re-runs at the boundary so a forged class
  flip never parses. The impossibility vocabulary gains
  `pre-state-preserved-for-recovery` — the repair's honest reversal
  statement: the raw capture is the substrate, and putting it back is
  REC-001's recovery plan. Version 2, which lived for exactly one
  change window with no surviving artifact, is refused at decode —
  retirement recorded here rather than smoothed over. Vectors
  regenerated as v3, plus the table-repair-acknowledged plan and its
  indeterminate-table snapshot; the TypeScript parity suite reproduces
  all of it unchanged. The mutation pass demanded one new fixture (the
  Present-state identity-bound-restore refusal) whose absence it
  exposed — recorded rather than smoothed over; three mutants killed.

- **WP-060 increment 6: the reversal — PLAN-008 emitted, ADR-0022
  implemented.** Every plan the planner produces now carries PLAN-008's
  output, in the body (the slice-3l linkage) and beside it (the
  `EmittedReversal` value for REC-010's advertisement and UI-005's
  display). The sized create emits a truthful draft deleting the
  created structure through a step-output reference — never an
  address, the node does not exist yet — with the created node's
  emptiness as its precondition, and claims **Reversible** exactly
  because that truthful draft exists (the withheld-claim posture
  becoming ADR-0022's rule). The grow emits a shrink-back draft whose
  reclaimed-tail emptiness is judged in the target's own address
  space, and deliberately keeps its conservative severity — no draft,
  no Reversible is one-directional. Wipes, shrinks, and identity
  writes state per step, machine-readably, why reversal is impossible.
  End-to-end at the planner: the draft is byte-deterministic
  (PLAN-001), binds against a post-apply capture into an ordinary plan
  bound to the capture's hash whose own linkage is the reapply-forward
  statement, refuses against the pre-apply world (nothing to resolve),
  refuses the prediction itself (nobody applies a prediction), and
  refuses by precondition once data lands in the created partition —
  the truth-decay fixture ADR-0022 names. Four mutants (dropped draft
  plan-ID derivation, dropped preconditions, reclaimed tail in the
  wrong address space, swapped impossibility reason) each failed a
  named test before proposal.

- **WP-010 slice 3l: plan body version 2 — the ADR-0022 reversal
  linkage, jointly sequenced with WP-060's reversal increment.**
  Section 6's reversal item becomes body content: `draft` linkage
  carrying the emitted draft's plan ID and body hash, `impossible`
  carrying per-step statements over a closed reason vocabulary
  (`data-destroyed`, `prior-value-not-carried`) that must cover exactly
  the plan's steps, and the draft's own `reapply-forward` naming the
  forward plan by ID alone — the acyclic asymmetry with no mutual-hash
  spelling (a `reapply-forward` map smuggling a `hash` key refuses as an
  undeclared field). Steps gain required `preconditions`
  (`region-unoccupied`, `host-unoccupied`, and the draft-only
  `step-output-unoccupied`), re-checked at every validation boundary —
  ADR-0022's two-time truthfulness, with the volume-with-data decay
  fixture refusing at binding and at the plain boundary. The
  `ReversalDraft` artifact lands with its step-output target spelling
  (a created node has no address to spell), emission-time truthfulness
  judged against the simulated proposal, a decode-recompute boundary of
  its own, and a binding boundary that resolves references against the
  helper's capture (zero or many candidates refuse), re-runs the sole
  constructor, and assembles an ordinary bound plan against the
  capture's hash. Two rules are structural everywhere: **no draft, no
  Reversible** (severity 1 refuses in the unlinked form, under an
  impossibility linkage, and as a forged severity byte at decode), and
  **a prediction never binds** (the plain boundary and the draft's
  binding boundary both refuse simulated snapshots outright). Version 1
  stays emitted and accepted until the planner migrates — its
  retirement is its own reviewed change under MODEL-003's
  explicit-migration discipline. Vectors: four new entries (the
  simulated-created snapshot, the v2 wipe with impossibility
  statements, the v2 forward create with draft linkage, and the
  create-reversal draft), pinned by the Rust constructors and
  reproduced by the TypeScript parity suite with no TypeScript change.
  Four mutants applied and each failed a named test before proposal.

- **WP-060 increment 5: the SI-15 solver unlock — a deviation is an
  act, not a state.** ADR-0023 (spec 12.1.0) implemented in
  `crates/planner`'s extent solver: the `MisalignedLegacyGrowth`
  refusal is replaced by the decided behavior. The filed 63-sector
  grow-at-tail case proceeds, authoring only the aligned new end; the
  untouched misaligned start is an inherited fact, byte-identical
  before and after (held by test against the simulated topology) and
  carried out of the planner as typed consequence material with its
  rendered sentence — planner-layer carriage, since ADR-0023 rejected
  typed hashed carriage and the consequence-text body vocabulary is a
  later jointly-sequenced change. Grow-to-fill against a misaligned
  neighbor is conformant and recorded as coincident naming the edge
  (the coincident-edge rule); the §11.2 authored/inherited split is
  proven on the shrink path; and the no-fourth-state property is swept
  across grow, shrink, and create — authored ends now meet policy on
  every solver path (create and shrink ends were previously unjudged,
  which would have been the fourth state), with the
  `UnalignedAuthoredBoundary` refusal naming the nearest conforming
  values. The deviation-override vocabulary stays deliberately
  inexpressible. Four mutants applied and each failed a named test
  before proposal.

- **WP-020: the r10 sitting — all three acceptances re-taken on
  `59ba1f6`, the stopping condition re-pinned there, closing the
  WP-070 arc.** The fifth trip from outside the package: WP-070
  increment 5 (PR #296) landed the apply-lifecycle module and
  completed that package's assigned increments. One sitting on
  2026-08-12, one fresh disposable Proxmox-hosted VM (VMID 9433,
  kernel 5.15.0-186-generic, no reboot during provisioning), the r9
  runbook copied to r10 with header prose alone changing: the full
  eleven-control refusal set refused, 2e passed (its tenth re-take —
  fourteen passes across eleven guests), the 2h suite passed (its
  eighth re-take, `fixtures_executed=1`, `ranges_written=1`,
  `contracted_bytes_written=8`), and the 2j suite passed its fifth
  re-take (`ranges_written=2`, `contracted_bytes_written=16`, both
  signatures restored). Fixtures byte-identical to the catalogue,
  loop table empty, manifest re-verified independently, teardown
  verified with nothing remaining (2026-08-12T22:37:55Z). Custody run
  16: transcript digests agreeing across guest, host, and workstation
  (`115267d2…6275`). One operational deviation recorded rather than
  hidden: the sitting-launch automation failed to deliver the script
  to the guest, the launch was performed manually about forty minutes
  after provisioning, no invocation occurred before it, and nothing
  measured was affected.

- **WP-070 increment 5: the apply lifecycle, enforced at the library
  layer.** The `lifecycle` module of `crates/journal`, discharging
  imported obligations 2 (the ordering half), 4, 5, 6, 7, and 8
  (ADR-0028, with ADR-0027's ordering and ADR-0021's single-use act)
  over decoded journals — every admission a pure function of the
  bytes, which is the whole point. One act, one apply: `admit_apply`
  requires an unconsumed authorization act for exactly the offered
  plan; another plan's act is no act for this one, an in-flight apply
  refuses a second admission citing the act it consumed, a consumed
  act admits nothing after its terminal, and one act facing a second
  grant refuses at the grant — a fixture the mutation pass itself
  demanded when the acts-never-consumed mutant survived the first
  suite, recorded rather than smoothed over. Disposal before
  recovery: a recovery plan named by a disposal linkage is
  inadmissible while the original's Failed record sits above the
  journal's durable watermark — appended-but-uncommitted refuses,
  one commit through the seam admits; the HLP-005 structural half on
  a shared device set is the platform packages' obligation. Each of
  the three re-entry edges — resume, reboot-resume, roll-forward —
  traces to the original act through an unbroken chain of connected
  Section 8 transitions with the act preceding the grant, and a
  broken chain refuses naming the break. A re-entry past the
  PLAN-007 window rejects; a fresh act journaled after the
  suspension readmits the same apply citing both acts — two acts,
  one apply, journaled as such — and a pre-suspension act does not
  count. Time is an injected `LogicalTime` seam: the truth of "now"
  is the helper's, the comparison is the library's. The roll-forward
  variant carries the `FreshRediscovery` attestation in its type, so
  JRN-003's journal-plus-fresh-re-discovery rule is demanded by the
  signature on exactly the edge ADR-0027 names. And the hand-forged
  in-memory-grant test: a restart recomputes identically from the
  bytes alone, a journal whose act was never written refuses
  admission and every re-entry edge by name, and the admitted types
  have no public constructor for a forged grant to inhabit. Six
  mutants (acts peeked not consumed, plan binding dropped,
  durability check dropped, connectivity dropped, stale fresh act
  accepted, window inverted) each killed by a named test — the first
  surviving until the pass forced the one-act-two-grants fixture.
  **This completes WP-070's five assigned increments**; everything
  beyond is consumer-driven, re-taken by the platform helper
  packages against real transport, privilege, and durability under
  their own assignments. This is a Rust merge: the WP-020 2e
  stopping condition trips, and the r10 re-pin sitting follows under
  WP-020's ownership.

- **WP-020: the r9 sitting — all three acceptances re-taken on
  `d4f61ed`, the stopping condition re-pinned there.** The fourth trip
  from outside the package: WP-070 increment 4 (PR #295) landed the
  journal's retention module, and the condition tripped by design.
  One sitting on 2026-08-12, one fresh disposable Proxmox-hosted VM
  (VMID 9432, kernel 5.15.0-186-generic, no reboot during
  provisioning), the r8 runbook copied to r9 with header prose alone
  changing: the full eleven-control refusal set refused, 2e passed
  (its ninth re-take — thirteen passes across ten guests), the 2h
  suite passed (its seventh re-take, `fixtures_executed=1`,
  `ranges_written=1`, `contracted_bytes_written=8`), and the 2j suite
  passed its fourth re-take (`ranges_written=2`,
  `contracted_bytes_written=16`, both signatures restored). Fixtures
  byte-identical to the catalogue, loop table empty, manifest
  re-verified independently, teardown verified with nothing remaining
  (2026-08-12T21:28:09Z). Custody run 15: transcript digests agreeing
  across guest, host, and workstation (`a16de758…d2a4`).

- **WP-070 increment 4: retention and compaction under ADR-0029's
  liveness rule.** The `retention` module of `crates/journal`,
  discharging imported obligations 9, 10, 11's derivation half, 12,
  and 13's fixture. The liveness-scoped exemption is computed from
  decoded records alone and closes over ADR-0027's linkage graph: a
  terminal apply whose disposal chain has not wholly terminated keeps
  every record pinned — a disposal-named plan that never started
  counts as non-terminal, the conservative reading, fail-closed
  toward retention — and a fully terminated chain ages into ordinary
  history, with a double-terminal journal refusing rather than
  guessing. No code path reclaims a live record, structurally:
  `compact()` is the sole reclamation entry point, computes the
  reclaimable set itself from the journal's own records, and offers no
  parameter by which a caller could name a range; budget exhaustion
  (`over_budget`, the spend measured in encoded frame bytes per apply)
  resolves only to the published `Executing → RecoveryRequired` row —
  an existing Section 8 edge, never a new one, never a reclamation —
  so the writer is stopped and the recoverer is never blinded.
  `decode_journal`'s two-pass replay derives `CoveredRanges` from the
  journal's own durable compaction records and nothing else: an
  absence no record covers refuses as the named mid-chain corruption
  case, and a compaction record cannot hide frame-level damage
  because checksums run before gap classification. Sequence
  monotonicity holds across compaction and continued appends —
  retained frames keep their numbers, compaction records consume the
  continuing positions, recover-and-continue preserves them, and a
  second retention round stays monotonic — and compaction records are
  journal infrastructure, never reclaimed, because reclaiming one
  would orphan the gap it legitimizes. Obligation 13's reconciliation
  fixture passes: the ADR-0028-shaped chain trace — the original's
  Failed-with-linkage terminal, the live recovery's act and records —
  reads identically before and after compacting around the live
  apply, from the bytes alone. Six mutants (unstarted-recovery
  exemption dropped, linkage closure ignored, tolerant second pass,
  sequence reset on compaction, infrastructure reclaimed, exhaustion
  through a wrong edge) were each killed by a named test before
  proposal. This is a Rust merge: the WP-020 2e stopping condition
  pinned at `94bfeba` trips, and the r9 re-pin sitting is run and
  recorded separately under WP-020's ownership, as every re-take has
  been.

- **WP-020: the r8 sitting — all three acceptances re-taken on
  `94bfeba`, the stopping condition re-pinned there.** The third trip
  from outside the package: WP-070 increment 3 (PR #293) landed the
  journal's record vocabulary, and the condition tripped by design.
  One sitting on 2026-08-12, one fresh disposable Proxmox-hosted VM
  (VMID 9431, kernel 5.15.0-186-generic, no reboot during
  provisioning), the r7 runbook copied to r8 with header prose alone
  changing: the full eleven-control refusal set refused, 2e passed
  (its eighth re-take — twelve passes across nine guests), the 2h
  suite passed (its sixth re-take, `fixtures_executed=1`,
  `ranges_written=1`, `contracted_bytes_written=8`), and the 2j suite
  passed its third re-take (`ranges_written=2`,
  `contracted_bytes_written=16`, both signatures restored). Fixtures
  byte-identical to the catalogue, loop table empty, manifest
  re-verified independently, teardown verified with nothing remaining
  (2026-08-12T20:43:53Z). Custody run 14: transcript digests agreeing
  across guest, host, and workstation (`1cbd7e18…4177`). The sweep
  also corrected an overstatement in the r7 record's label paragraph,
  in place: the sitting-script header's trip ordinal continues the
  script lineage's own numbering — one behind the record's running
  count, which includes the original post-merge re-take — and the r7
  record had claimed the lineage numbering was moved to the record's;
  it was not, and the correction names both numberings.

- **WP-070 increment 3: the record vocabulary under JRN-006.** The
  `records` module of `crates/journal` and the `schemas/journal/`
  schema set (`records.md`, plus `framing.md` documenting increment
  2's frame profile): `partman.journal.record` version 1, encoded
  through WP-010's `pce/1` canonical codec — the crate's two
  dependencies (`partman-statemachine`, `partman-domain`) arriving
  with this increment exactly as increment 2's manifest recorded.
  Every imported record class lands with its deciding authority
  carried in its documentation: the authorization-act record with the
  helper-computed tier (ADR-0021/0028 — the journal is the act's only
  home), transition records enforcing Section 8's per-row effect
  constraints at record-write time — the check increment 1's
  `TerminalRecord` deferred here, now taken over every published row
  by test — with ADR-0027's disposal linkage constructible on the
  `failure-accepted` row alone, checkpoint records, the three-variant
  protection record (ADR-0024: verified parse-level backup, positively
  determined absence, verified raw capture with validated regions)
  whose artifact references are content hash plus store identity with
  the artifact's bytes given no field to occupy (ADR-0030's "never its
  bytes" held structurally and proven by a walk over every encoded
  byte-string position), the compaction record and
  `PER_APPLY_JOURNAL_BUDGET_BYTES` (ADR-0029: 256 MiB, landing with
  the schema as the ADR requires; the failure direction was the
  decided part and enforcement is increment 4's), and the dry-run
  refusal class as response-data vocabulary (ADR-0026, its own type so
  pending-qualification can never read as a validation failure). The
  WP-010 joint sequencing each ADR names is discharged hash-only: no
  WP-010 body schema changed. JRN-005 is held structurally: no record
  class has a free-text position, every Text value in every encoded
  record is proven inside the closed transcribed vocabulary, and the
  WP-035/WP-040-shaped gate plants every SEC-006 identifier class in
  every position, proving each refusal echoes nothing back. The strict
  decoder refuses unknown versions (MODEL-003's explicit rejection),
  kinds, fields, tags, mistyped positions, and wrong-length hashes,
  and routes the constructors' own invariants so the wire cannot
  smuggle a refused shape. Golden vectors are pinned in
  `schemas/journal/records.md` and held in doc-code agreement by test.
  Imported obligation 3's increment-3 half is discharged: the disposal
  chain — act, Failed-with-linkage, recovery act — reconstructs from
  journal bytes alone across a torn tail, idempotently. Six mutants
  (constraint dropped, linkage widened, any-version accepted,
  unknown-fields accepted, region overlap accepted, wire-tag drift)
  were each killed by a named test before proposal. This is a Rust
  merge: the WP-020 2e stopping condition pinned at `15e6469` trips,
  and the r8 re-pin sitting is run and recorded separately under
  WP-020's ownership, as every re-take has been.

- **WP-020: the r7 sitting — all three acceptances re-taken on
  `15e6469`, the stopping condition re-pinned there.** The second trip
  from outside the package: WP-070 increment 2 (PR #291) landed
  `crates/journal`, a crate no acceptance executes, and the condition
  tripped by design, exactly as it did for increment 1. One sitting on
  2026-08-12, one fresh disposable Proxmox-hosted VM (VMID 9430, kernel
  5.15.0-186-generic, no reboot during provisioning), the r6 runbook
  copied to r7: the full eleven-control refusal set refused, 2e passed
  (its seventh re-take — eleven passes across eight guests), the 2h
  suite passed (its fifth re-take, `fixtures_executed=1`,
  `ranges_written=1`, `contracted_bytes_written=8`), and the 2j suite
  passed its second re-take (`ranges_written=2`,
  `contracted_bytes_written=16`, `EFI PART` at offsets 512 and
  4,193,792 before, eight zeros each in between, both restored after).
  Both fixtures ended byte-identical to the compiled catalogue, the
  loop table ended empty, the manifest re-verified independently, and
  teardown was verified with nothing remaining (2026-08-12T19:52:37Z).
  Custody run 13: transcript digests agreeing across guest, host, and
  workstation (`bc3fe187…8b48d7`). The r7 copy corrected all three
  carried-over labels the r6 record had noted — the header's trip
  numbering, the 2j transcript section's "FIRST TAKE", and the
  evidence-bundle path — so that debt does not carry forward; no label
  touched a measured value in either lineage.

- **WP-070 increment 2: the journal core.** `crates/journal`
  (workspace lints, `unsafe_code` denied by inheritance per SAFE-009's
  journal-crate rule, dependency closure empty — the
  `partman-statemachine` dependency arrives with increment 3's record
  vocabulary, exactly as increment 1's manifest recorded): JRN-001's
  frame mechanics as a pure library. The byte log is append-only with
  per-record CRC-32/IEEE checksums and strictly monotonic one-based
  sequence numbers — the encoding pinned byte-for-byte against an
  independent bit-by-bit transcription of the format and the standard
  check value, every earlier snapshot a byte prefix of every later
  one. A torn tail is detected and safely truncated, proven by a sweep
  over every byte cut of a three-record log plus the
  damaged-complete-final-frame shape; interior damage refuses rather
  than truncates — a checksum mismatch with bytes behind it, a
  duplicated frame, a forged sequence zero, and a complete over-bound
  frame are each a typed refusal naming the defect and its offset,
  because safe truncation is the tail's rule alone. JRN-002's
  durability rule lands as a typed injected boundary: `commit` offers a
  `DurabilitySeam` exactly the not-yet-durable byte suffix, a refusal
  leaves the watermark and the pending suffix untouched for re-offer,
  and a `WriteClearance` is constructible only behind the watermark —
  so the storage-writing code the M3 helper packages build demands
  proof of prior journal durability instead of a comment, while
  platform fsync truth stays their acceptance obligation, said in the
  trait's documentation rather than implied. JRN-003's replay is a
  pure function of the bytes and the covered ranges: identical inputs
  replay identically, recovery is a fixpoint (replaying the truncated
  valid prefix reproduces the same records with no further
  truncation), and a recovered journal continues the sequence exactly
  where the surviving records end — nothing derives from writer
  memory. Imported obligation 11's core (ADR-0029, shared with
  increment 4) ships as the three-way gap classification: a
  compaction-covered gap proceeds — over `CoveredRanges`, the
  classification's typed input, which increment 4 derives from durable
  compaction records — a torn tail truncates, and any uncovered or
  partially covered gap refuses as the named mid-chain-gap corruption
  case, at the head and in the chain alike. Seven mutants (uncovered
  gaps accepted, clearance without durability, interior damage
  truncating, checksums ignored, regressions accepted, watermark
  advancing without the seam, recovery resetting the sequence) were
  each killed by a named test before proposal. This is a Rust merge:
  the WP-020 2e stopping condition pinned at `a2e6db2` trips, and the
  r7 re-pin sitting is run and recorded separately under WP-020's
  ownership, as every re-take has been.

- **WP-020: the r6 sitting — all three acceptances re-taken on
  `a2e6db2`, the stopping condition re-pinned there.** The first trip
  from outside the package: WP-070 increment 1 (PR #289) landed
  `crates/statemachine`, a crate no acceptance executes, and the
  condition tripped anyway because it binds to the tree, not to a code
  path — its design, not a false positive. One sitting on 2026-08-12,
  one fresh disposable Proxmox-hosted VM (VMID 9429, kernel
  5.15.0-186-generic), the r5 runbook copied to r6 with only VMID,
  candidate commit, and header prose changed: the full eleven-control
  refusal set refused, 2e passed (its sixth re-take — ten passes across
  seven guests), the 2h suite passed (its fourth re-take,
  `fixtures_executed=1`, `ranges_written=1`), and the 2j suite passed
  its first re-take (`ranges_written=2`, `contracted_bytes_written=16`,
  `EFI PART` at offsets 512 and 4,193,792 before, eight zeros each in
  between, both restored after). Both fixtures ended byte-identical to
  the compiled catalogue, the loop table ended empty, the manifest
  re-verified independently, and teardown was verified with nothing
  remaining (2026-08-12T18:48:50Z). Custody run 12:
  transcript digests agreeing across guest, host, and workstation
  (`1277f1b1…2ec0`). Three carried-over labels in the copied artifacts
  are recorded in the WP-020 record rather than hidden (the script
  header's trip numbering, the transcript's "FIRST TAKE" section label
  for the 2j re-take, the teardown bundle's r5 path label); none
  touches a measured value. A pre-existing stale count in
  `docs/quality/test-tiers.md` ("four times" against six custody rows)
  was found in the sweep and corrected with the correction noted in
  place.

- **WP-070 increment 1: the execution state machine, pure.**
  `crates/statemachine` (workspace lints, `unsafe_code` denied by
  inheritance, no dependencies — the journal will depend on it, never
  the reverse): Section 8's thirteen states and its twenty-three-row
  transition table as `Transition` variants, so an undeclared
  transition has no variant to be — unrepresentable at construction,
  Section 11.6's obligation and ADR-0027's imported obligation 1,
  proven over all 169 ordered state pairs against an independent
  transcription of the specification's rows rather than against the
  crate itself. Terminal records carry their effect summaries
  structurally (`TerminalRecord` constructs only for the three
  terminal states; a non-terminal state is a typed refusal naming
  itself), the published per-row effect constraints are encoded and
  pinned (`no-writes` alone on the three no-writes rows,
  `no-writes`-or-`partial` on the honored cancel, `None` where the row
  constrains nothing — the per-journal pause cancel stays the journal
  increment's to determine), no transition leaves a terminal state,
  every non-terminal state has an exit, and ADR-0027's two arms are
  asserted as the exact `RecoveryRequired` exit set. The
  machine-readable table Section 8 requires under `schemas/` lands as
  `schemas/state-machine.md`, rendered by `published_markdown()` from
  the same variants the property tests check and held byte-fresh by
  test — one source, three views — with regeneration via the
  documented `render` example, which performs no repository write
  itself. Converts the traceability mode to `generated`. The re-pin
  sitting this Rust merge owes under the WP-020 stopping condition is
  run and recorded separately, as always.

- **WP-020 increment 2j is Delivered on its first-take acceptance, and
  increment 2 itself is delivered as scoped; the 2e stopping condition is
  re-pinned at `39b59f5`.** One sitting on 2026-08-11, one fresh disposable
  Proxmox-hosted non-WSL VM (VMID 9428, kernel 5.15.0-186-generic), commit
  `39b59f5`: eleven negative controls refused (the nine from the previous
  sittings plus two for the new selector), the 2e read-only acceptance
  passed first on a pristine tree (its fifth same-day re-take), the 2h
  single-range suite passed (its fourth), and then the 2j two-range suite
  passed on its first take — the first real-kernel run of the 2i general
  executor's multi-range chain. `EFI PART` at offset 512 **and** at offset
  4,193,792 before, eight zeros established at each in between, `EFI PART`
  at both after regeneration; `fixtures_executed=1`, `ranges_written=2`,
  `contracted_bytes_written=16`, one attachment, one confirmed detach, the
  kernel's `LOOP_CHANGE_FD` refusal classified from the observed status
  re-read. Both fixtures ended byte-identical to the compiled catalogue and
  the loop table ended empty; teardown verified with nothing remaining. The
  sitting's first invocation was void before any gate ran — the copied
  sitting script lacked the execute bit and `script -c` refused it — and is
  retained in the custody table rather than discarded; the re-invocation is
  the cited run (transcript `a788471b…b0f6`, digests agreeing across guest,
  host, and workstation). With a registered suite exercising the general
  shape under its own boundary and operator-run acceptance, increment 2's
  own delivery bar is met: a Tier-2 destructive suite can exist, two do,
  and no product write path exists or is authorized by any of it. The
  multi-fixture half of the general shape stays Tier-1-proven until a
  contract needs it, as the 2j boundary records.

- **WP-020 increment 2j: the second destructive suite, and the first
  two-range one, implemented and Tier-1-proven; its VM acceptance is not yet
  taken.** `gpt-basic-512-both-signatures-erase` is registered: one fixture,
  two declared eight-byte ranges replaced with zeros — the primary GPT
  header's signature at offset 512, byte-for-byte the range the 2h suite
  writes, and the backup header's signature at offset 4,193,792, the last
  512-byte LBA of the 4 MiB image, measured on the generated image before
  the contract was written. A drifted backup offset fails closed twice
  before any write: admission re-checks the bound against the compiled
  generated length, and the run's differs-before-the-write requirement
  refuses a range that is not carrying a signature. Both edit-detectors
  flipped as the 2g and 2i boundaries provide — the registry shape pin
  became `the_shipped_registry_holds_exactly_the_2h_and_2j_suites` and the
  xtask refusal-count pin moved from one to two, each firing on the edit —
  and every generic-refusal test was re-read, with the re-readings recorded
  in the count pin's comment and on the registering pull request. No
  executor change: 2i's general executor runs the suite as compiled data,
  and the shipped two-range shape is reduced through the real path
  (membership check included) by a new Tier-1 test, so the shape is
  executed before any privileged kernel meets it. The shared status
  documents' availability sentences (`AGENTS.md`, `CONTRIBUTING.md`) are
  repaired in the same change: both still claimed a single runnable
  higher-tier acceptance, stale since the SI-35 selector and the 2h suite
  registered — the exact drift the increment 2e grant was widened to
  prevent. Mutation-checked: a drifted backup offset fails both shape pins.
  The delivery condition is the suite's operator-run VM acceptance:
  one sitting, 2e re-taken first on a pristine tree, the 2h suite re-taken,
  then the two-range suite's first acceptance with `EFI PART` at both
  offsets before, eight zeros in between, and `fixtures_executed=1`,
  `ranges_written=2` reported.

- **WP-020 increment 2i is Delivered: both acceptances re-taken through the
  general executor, and the 2e stopping condition is re-pinned at
  `0625b07`.** The 2i merge replaced the executor both Tier-2 acceptances
  run through, tripping the stopping condition a fourth time by
  construction — its own boundary made the re-take the delivery condition.
  One sitting on 2026-08-11, one fresh disposable Proxmox-hosted non-WSL VM
  (VMID 9427, kernel 5.15.0-186-generic), commit `0625b07`, the same
  runbook and sequencing: nine negative controls refused, the 2e read-only
  acceptance passed first on a pristine tree, then the 2h destructive suite
  passed through the general executor's one-fixture path with the identical
  value set plus the general report's counters (`fixtures_executed=1`,
  `ranges_written=1`). The declared range read `EFI PART` before and after
  with eight zeros established in between, both fixtures ended
  byte-identical to the compiled catalogue, the loop table ended empty, and
  teardown was verified — no VM config, volume, or snapshot remains.
  Transcript `bc46c821…d903`, digests agreeing across guest, host, and
  workstation. The general shape's first real-kernel exercise arrives with
  the suite that uses it, behind that suite's own boundary; what this
  sitting establishes is that the accepted one-fixture chain survives the
  generalization on a real kernel exactly as the containment pin says it
  must at Tier 1.

- **WP-020 increment 2i: the destructive executor becomes general.** The 2g
  registry always compiled the general contract shape — N fixtures, N
  non-overlapping ranges each — but the 2h executor deliberately refused
  everything except one fixture with one range, and its own comment said a
  suite outgrowing that shape gets a new executor rather than a widened one.
  This is that reviewed replacement, and it registers nothing: the registry
  still holds exactly the 2h suite, no selector changes, every
  generic-refusal test keeps its meaning, and nothing new can run. The
  admission reduction now binds each declared fixture to the verified held
  object of exactly its catalogue basename — tested under an authorization
  deliberately ordered opposite to the declaration, with the binding proven
  by reading the held objects' bytes so a positional zip fails by bytes
  rather than by names — and re-states the general preconditions as its own.
  The pure protocol generalizes per fixture to N ranges (every range
  pre-read and required to differ from its replacement, so a suite cannot
  ride one provable change past a range that would prove nothing; one
  bracket over the complement of the union; one write per range between the
  rebind probe and the sync; per-range post-equality after confirmed
  detach), and a suite executor adds the multi-fixture discipline: a
  pre-flight hashes every fixture's held bytes before any fixture is
  attached, so a suite whose second fixture is wrong refuses before its
  first is touched, then complete self-contained chains run in declared
  order. On the one-fixture, one-range shape the general protocol produces
  exactly the call sequence the 2h boundary recorded, and that containment
  is itself a pinned test. The multi-range bracket refuses unsorted or
  overlapping input rather than mis-partitioning the file, with its
  complement arithmetic pinned against independently computed digests
  including chunk-straddling ranges. Every new gate was mutation-verified:
  the removed pre-flight, a skipped second write, a skipped second
  pre-read, the bracket cursor arithmetic, and the positional target zip
  each fail a named test. The report gains `fixtures_executed` and
  `ranges_written` under the same allowlist discipline; the public surface
  gains no entry point. This merge re-opens both acceptances — the 2e
  stopping condition trips on it by construction — and the increment's row
  moves to Delivered only after the sitting re-takes them through the
  general executor.

- **WP-020: both Tier-2 acceptances re-taken on the issue-fix tree; the 2e
  stopping condition is re-pinned at `68298f2`.** The #248/#249/#250 merges
  (PRs #251–#253) landed three non-Markdown paths after `4fbb2f9` — two of
  them the very probe and write lines the 2h suite executes — so both proofs
  went stale by their own terms and were re-measured rather than argued
  forward. One sitting on 2026-08-11, one fresh disposable Proxmox-hosted
  non-WSL VM (VMID 9426, kernel 5.15.0-186-generic — this guest did not
  reboot into -187 as the previous one had), commit `68298f2`, the same
  runbook and sequencing: nine negative controls refused, the 2e read-only
  acceptance passed first on a pristine tree, then the destructive suite
  passed with the identical value set — the kernel's `LOOP_CHANGE_FD`
  refusal now classified from an observed status re-read (#248) and the
  contracted write issued through the helper whose destination Tier 1 now
  measures (#250). The declared range read `EFI PART` before and after with
  eight zeros established in between, both fixtures ended byte-identical to
  the compiled catalogue, and the loop table ended empty. Teardown verified:
  no VM config, volume, or snapshot remains. Transcript `ee330401…c28bc`,
  digests agreeing across guest, host, and workstation. The sitting also
  corrected a stale count the previous round left behind: the 2e
  reproducibility sentence still said "five times across two guests" after
  the custody table had grown to seven runs — the
  sitting-lands-in-more-places-than-one shape, now fixed alongside the new
  row rather than left for the next reader.

### Fixed

- **WP-020: the contracted write's destination is measured at Tier 1
  (#250).** The only line in the repository that writes to storage under a
  Tier-2 gate had no automated coverage: the protocol tests drive a fake
  whose `write_contracted` never involves the offset, and the real method
  needs a loop device, so a transposed field or an offset arithmetic change
  would have passed every unprivileged test on every platform and been
  caught only by a VM sitting's post-conditions — which are checked by the
  same protocol whose write is in question. The write moved into
  `write_contracted_range`, a helper taking the admitted contract and the
  device descriptor, and
  `the_contracted_write_lands_exactly_at_the_contracted_offset` writes
  through it into a regular scratch object and reads the whole object back
  independently: the replacement bytes at exactly the contract's offset,
  every other byte untouched, the reported count exact, and the range
  asserted to differ beforehand so a write that did nothing cannot satisfy
  the read-back. This is the issue's named fallback — the address
  arithmetic measured even though the syscall destination under a real loop
  mapping remains the acceptance's measurement — and the transposed-field
  mutation fails it.

- **WP-020: the rebind probe names only an observed kernel state (#248).**
  Increment 2h's pre-write discipline rests on the loop driver refusing
  `LOOP_CHANGE_FD` on a read-write attachment, and the probe read any
  `EINVAL` from that ioctl as exactly that refusal — naming the result
  after the reason it expected rather than anything it observed, on the
  most overloaded errno the ioctl path has. The direction made it worth
  fixing now: a misclassified `EINVAL` is a false safety pass that
  proceeds into the write, while every other refusal in the protocol
  fails closed. On `EINVAL` the probe now re-reads `LOOP_GET_STATUS64`
  and only an attachment observed without `LO_FLAGS_READ_ONLY` may name
  the answer `KernelRefused`; an `EINVAL` the observation cannot explain
  refuses the run, and a failed observation wins over any
  classification. The mapping itself moved into a pure classifier the
  fake-driven protocol tests could never reach, with
  `the_rebind_probe_names_only_an_observed_kernel_state` pinning every
  arm — including that acceptance and non-`EINVAL` errors classify
  without consulting flags — and both the blind-`EINVAL` and
  inverted-flag mutations fail it.

- **WP-020: admission counts verified handles per fixture instead of
  comparing name sets (#249).** `Admission::admit` compared the authorized
  target names to the declared fixture names as `BTreeSet`s, and a set
  cannot distinguish one verified handle per declared fixture from several
  handles for one fixture — both collapse to the same set, so both were
  admitted. Unreachable for the one registered suite only because two
  downstream checks happened to close it, which is the inherited-check
  shape this package refuses, and the single-target arity check it leaned
  on does not generalize to the multi-fixture suites increment 2's
  remaining scope introduces. The admission now counts handles per target
  name and refuses more than one with a refusal naming the name and the
  count; the `unwrap_or_default()` that silently coerced a nameless target
  path to the empty string on the same path is now an explicit refusal.
  The duplicate-handle shape is constructed for real in the new evidence —
  a `..`-spelled second path to one fixture survives the interlock's
  supplied-path dedup on Unix and verifies into two handles — and the
  writing of that test corrected the issue's own premise: `Path` equality
  normalizes `.` components away, so the `./` spelling the issue cited is
  deduplicated after all, while `..` components compare literally. On
  Windows the shape cannot reach admission — the first verified handle's
  share mode refuses the second write-capable open — and a companion test
  pins that platform split as a measured fact rather than an assumption.

### Added

- **WP-020 increment 2h is Delivered: the first destructive suite passed its
  operator-run acceptance, and the 2e acceptance was re-taken beside it.** One
  sitting on 2026-08-11, one disposable Proxmox-hosted non-WSL VM, one
  transcript, commit `4fbb2f9`. The read-only 2e acceptance ran first on a
  pristine tree — its stopping condition had tripped a second time when
  increments 2g and 2h landed nine non-Markdown paths after `582e6d1` — and
  the destructive suite ran second, because it mutates a fixture and restores
  it. **The kernel refused `LOOP_CHANGE_FD` on the read-write attachment**,
  which had been an assumption about the loop driver when the code was written
  and is now a measurement on kernel 5.15.0-187-generic; the suite keeps that
  leg so a kernel behaving otherwise voids the run rather than passing it. The
  declared range read `EFI PART` before and `EFI PART` after, the harness
  established that it held eight zero bytes in between, the digest bracket
  over every other byte was unchanged, and both fixtures ended byte-identical
  to the compiled catalogue. Nine negative controls refused — the four from 2e
  plus five for the new `--suite` selector — with the fixtures digest-checked
  afterwards, so no refusal path wrote anything. Teardown verified: no VM
  config, volume, or snapshot, and the host-attached USB media unchanged.
  Transcript `ac4a496b…af8b`, digests agreeing across guest, host, and
  workstation. The 2e stopping condition is re-pinned at `4fbb2f9`.

  The sitting also found a defect in the runbook and it is fixed here rather
  than noted: `02-guest-provision.sh` ran `apt-get purge -y snapd 2>/dev/null
  || true`, discarding both the purge's error output and its exit status. The
  purge failed on this guest — snapd's squashfs mounts and their loop bindings
  were live — and the script continued, aborting only incidentally when the
  following `rm -rf` hit a read-only filesystem. Had that `rm` succeeded on a
  partially unmounted tree, provisioning would have reported success with
  snapd installed and the sitting would have recorded the
  no-other-loop-administrator exclusion as established when it was not. The
  script now unmounts first, purges with output visible, and **proves** the
  package is gone with `dpkg -l` before continuing.

- **WP-020 increment 2h: the first destructive suite, implemented and
  Tier-1-proven; its VM acceptance is not yet taken.** The one edit increment
  2g reserved: `gpt-basic-512-signature-erase` is registered, both
  edit-detectors fired, and every generic-refusal test was re-read at that
  edit with each re-reading recorded in the test that changed. The suite
  contracts a single range — eight bytes at offset 512, the primary GPT
  header's signature field, replaced with zeros — and `IntendedChange` gained
  the replacement bytes so "changed exactly as contracted" is byte-checkable
  rather than narrative, with a length disagreement refused at admission.
  `crates/ffi-linux-loop` gained exactly one entry point,
  `run_destructive_suite`, which takes the registry `Admission` rather than an
  `Authorization` or a file, so the compiled contract is unavoidable on the
  write path; the pinned public-surface test widened by that one line as a
  reviewed edit. Its attachment is **read-write**, which is the pre-write
  discipline the 2e record requires a destructive path to establish for
  itself: the kernel's loop driver refuses `LOOP_CHANGE_FD` on a read-write
  attachment, so the rebind is inapplicable rather than detected, and the
  suite attempts it before writing and voids the run if the kernel accepts.
  The protocol brackets every byte outside the contracted range by digest
  before attaching, writes and `fdatasync`s through the held loop descriptor,
  re-verifies the full status binding, and reads both post-conditions only
  after confirmed detach and partition teardown — with a detach failure
  winning over any attached-path result, because a mutated fixture with
  uncertain cleanup must refuse rather than report. The runner then
  regenerates the fixture tree and requires it to equal the compiled
  catalogue. `cargo xtask test --tier 2 --profile destructive --suite <name>`
  resolves only a compiled registry name at exactly Tier 2 with the
  destructive profile; a generic request still selects no suite and still
  refuses. Every new gate was mutation-verified — each disabled in turn, its
  named test failed — and the Linux half was type-checked, clippy-clean, and
  tested under WSL Debian, which is where `linux.rs` actually compiles.

  An adversarial review of the first draft found six defects, all fixed
  before the change was proposed, and they are worth recording because two of
  them were the exact failure this package exists to refuse. **The post-run
  restoration guard could not fail:** it compared `catalogue::generate`'s
  returned manifest to `catalogue::expected()`, which is the same pure
  function of the same compiled data, so it never read a byte from disk while
  reporting `backing_regenerated_to_catalogue=true`. The new
  `catalogue::verify_on_disk` re-reads and re-hashes every image, and its own
  test mutates a fixture and requires the refusal. **Restoration also ran
  only on success**, so every refusal reachable after the contracted write —
  a wrong-length write, the post-write re-verify, either post-condition —
  left the fixture mutated while the message discussed cleanup uncertainty;
  it is now unconditional. The other four: the executor accepted any
  `&'static Suite` rather than a registered one, so the registry gated
  nothing (it now checks membership by address); `digest_outside_range` and
  `read_exact_range` had no tests at all despite being the only code that
  makes the two post-conditions real (both now tested, including the
  chunk-boundary and past-EOF cases); `changed_exactly_as_contracted` named a
  change the protocol never measured (the range is now read before the write
  and required to differ); and the module documentation still described an
  empty registry with no executor.

- **WP-020 increment 2g: the destructive-suite registry becomes a compiled
  type.** "No destructive suite is registered" was load-bearing prose backed
  by refusal tests; it is now `partman_fixtures::registry` — a compiled,
  catalogue-pattern registry where a suite is a value naming its fixture set
  by catalogue basename, its verified target class (a closed one-variant
  vocabulary), its per-fixture intended-change contract (exact byte ranges
  with each range's reason, everything outside them pinned by digest
  bracket), and its teardown proof obligations (a closed vocabulary, pinned
  by exhaustive match). The shipped registry is empty and a test pins the
  emptiness as increment 2h's reviewed edit-detector; the xtask generic
  destructive refusal now cites the registry's count, pinned at zero by its
  own test on the other side. Admission consumes the SAFE-007
  `Authorization` — one admission is one gated run, non-clonable, its
  targets extractable only by consuming it — and refuses a target set that
  is not exactly the declared fixture set, a fixture the catalogue does not
  generate, a duplicate or vacuous contract, and zero-length, out-of-bounds,
  or overlapping ranges. Every refusal gate was mutation-verified: each gate
  was disabled in turn and its named test failed, and a suite smuggled into
  the shipped registry trips both edit-detectors. No executor exists,
  nothing consumes an admission, and every generic destructive Tier-2 and
  Tier-3 request refuses exactly as before. Recorded in the increment 2g
  authorization boundary in `docs/work-packages/WP-020.md`, alongside the
  defined-but-not-started 2h row.

- **WP-020 increment 2e: the acceptance re-taken on current main, discharging
  issue #175.** The reproducibility record's own stopping condition had
  tripped — `git diff --name-only c75b340 HEAD` reported fifteen non-Markdown
  paths, several on the acceptance's code path — so the
  `linux-loop-read-only` acceptance was re-taken rather than argued forward:
  a fresh disposable Proxmox-hosted non-WSL VM (same pinned base image
  digest, kernel, and toolchain as the 2026-08-03 sitting, provisioned by the
  same runbook), current main `582e6d1`, root over a direct login with no
  `sudo` and no injected variables. The retake's first run refused at its
  Tier-1 gate — WP-035's identity sweep found `$USER` (`root`) as a substring
  of the udev caveat's static "root's udevd" prose, a verified coincidence
  collision exempted per the test's own remedy in its own reviewed commit —
  and the second run passed on the merged tree with the identical harness
  value set, byte-identical fixture digests, all four negative controls
  refusing, and an empty loop table afterwards. Both transcripts, including
  the refusal, are digest-bound in the record with guest, host, and
  workstation recomputations agreeing; the VM's verified teardown left no
  config, volume, or snapshot behind. The stopping condition is re-pinned at
  `582e6d1`.

- **WP-040 increment 4: the authentication skeleton and the record.**
  The closed per-transport claim vocabulary RPC-001 implies lands as
  `crates/rpc`'s `identity` module: one identity claim per transport —
  the SDDL a Windows named pipe must restrict access to (SYSTEM and
  the authorizing interactive user; the claim names the restriction,
  not a value), the peer credentials a Unix domain socket must verify,
  the code-signing requirement a macOS XPC connection or its
  equivalently verified socket must check — as **types naming what a
  peer proves, verified by nobody here**. Every transport is
  route-decision-gated (the WP-035 increment-10 triangle, three times
  over), so each claim's verifier arrives with its transport's
  recorded route decision, and the skeleton says which claim waits on
  which route per claim — `waits_on` states "unrecorded" rather than
  letting absence read as oversight, because complete and endpoint-less
  is a truthful state, not a gap. **No authorization vocabulary
  exists, deliberately**: SI-18 holds whether a severity-1 plan needs
  fresh interactive authorization, so the vocabulary names identity
  facts only — nothing about what a peer may do, when a human must
  approve, or when an approval expires — with HLP-003's binding
  WP-070's to implement under whatever SI-18 decides. The closure test
  pins the vocabulary by exhaustive match, so widening it fails the
  suite as a visible reviewed edit. `schemas/rpc/authentication.md`
  records the vocabulary — a type vocabulary, not a wire format, and
  the doc says so. With this, WP-040's four ungated increments are
  delivered; what remains is one transport increment per OS, each
  opening only after its recorded route decision, and whatever field
  SI-18's resolution unlocks.

- **WP-040 increment 3: the redaction boundary.** SEC-006's deny-floor
  lands at the protocol edge as a schema-level rule for which field
  positions may carry identifier-class bytes at all, held the WP-035
  way: an allowlist that needs no knowledge of the denied classes,
  because every position outside it is structurally incapable — a
  pinned constant refuses any other value, an unsigned number cannot
  hold bytes, a closed tag refuses anything outside itself, and the
  strict validator is the mechanism rather than a filter (there is no
  position to invent; an identifier planted even as a field's own key
  refuses by name). The allowlist is exactly two authored positions,
  each with its governing authority named: the envelope `body` (the
  `schemas/`-defined type the bytes encode governs them) and the
  resume token's `execution` handle (opacity is WP-070's minting
  obligation, said so rather than pretended verified). The handshake's
  `build` — the protocol's one free-entry text position — moves to
  RPC-002's own word for it, a *version*: `digits.digits.digits` with
  an optional `+`/`-` suffix over `[A-Za-z0-9._+-]`, ASCII, bounded at
  64 bytes, enforced in both directions so this side cannot emit what
  the peer would refuse, with a refusal that names the rule and never
  echoes the value; `partman.rpc.handshake` moves to schema version 2
  for it, the envelope-v2 reviewed-bump posture exactly. What a
  grammar cannot do is stated rather than hidden: the boundary's reach
  is raw identifier-class values — the gate test plants a serial, two
  path shapes, a spaced label, a username, an armored key, and a file
  name in every non-allowlisted position of every format and each
  refuses — while deliberate shaping inside the admitted alphabet is
  the peer's schema violation, named in `schemas/rpc/redaction.md`.
  Three new tests; the boundary table's per-format field sets pinned
  to the wire's actual key sets as literals so widening the allowlist
  is a visible reviewed edit.

- **WP-040 increment 2: streams and reattach vocabulary.** The
  envelope moves to schema version 2 — a reviewed bump taken while no
  consumer exists, which is exactly what version numbers are for —
  gaining the event stream's `sequence` field with **per-channel
  presence rules held strictly both ways**: an event carries exactly
  one, a request or response carries none, and a violation refuses
  naming the channel and the presence found. Loss tolerance is
  detection, classification, and recovery — never papering over: the
  producer's sequence is monotone from 1 with no gaps, and the
  consumer's total classification processes in-order arrivals,
  discards replays (expected and harmless after reattach), and names a
  gap's missing closed range whose recovery is resynchronization from
  the journal — WP-070's to provide, said so in the schema doc, with
  this layer shipping the anchor and nothing that pretends to replay.
  The resume token (`partman.rpc.resume-token` v1) round-trips
  strictly — execution identifier plus last processed sequence, a
  smuggled `skip_journal` field refusing by name. Timeouts land as
  typed configuration the consumer supplies and enforces: this pure
  layer has no clock, exactly like the planner, so its honest
  contribution is the vocabulary. `schemas/rpc/streams.md` records the
  rules; `envelope.md` moves to v2. Three new tests.

- **WP-040 increment 1: the RPC message layer.** `crates/rpc` joins
  the workspace, depending on `partman-domain` deliberately — the wire
  body encoding is `pce/1`, so both sides of the RPC boundary already
  encode and hash identically under MODEL-005's cross-language proof.
  The versioned envelope lands with RPC-004's 1 MiB bound **binding
  the wire before any parsing touches the bytes** (checked at decode
  entry, at body wrap, and at encode), the body re-proved canonical at
  wrap and decode so an envelope cannot launder bytes the codec would
  refuse, and the three-class channel vocabulary closed now so the
  shape does not move when increment 2's sequence numbering arrives.
  RPC-002's handshake is refuse-never-degrade as a **total function**:
  equal protocol versions are compatible, unequal refuse with both
  versions and a remediation naming the older side and the build to
  update to — exact equality deliberately, because a compatibility
  window is a reviewed decision and the honest rule until one exists
  is the one that cannot admit an untested pairing. RPC-003's
  strictness is one validator for both ends — unknown fields refuse by
  name, mistyped fields by field — so the helper-side strictness the
  requirement demands is structurally also the client's.
  `schemas/rpc/envelope.md` and `handshake.md` record the formats in
  the `schemas/domain` shape. Four tests; traceability converted to
  generated.

- **WP-060 increment 4: the simulated final topology — PLAN-002's
  second half, and the package's last unlocked increment.** Every
  planning entry point now returns `Planned { plan, simulated }`: the
  plan and its predicted final topology together, because **simulation
  is mandatory, not decorative** — PLAN-002 says every valid plan
  produces both, so an effect this model cannot represent produces no
  valid plan at all, and Move/Copy/Repair/Encrypt/Decrypt refuse as
  `NotRepresentable` until their vocabularies arrive rather than
  emitting a prediction that lies. What honestly simulates today: a
  wipe removes everything the facts place on the wiped bytes
  (transitively, with everything named relative to it) while the wiped
  container itself survives empty, and drops the target's table-state
  stamp — post-wipe state is unestablished until a real capture, and
  absence is the honest prediction; a sized create mints a partition
  under the host's single table view (none or two — a hybrid —
  refuses, because creating "somewhere" is not a prediction) at the
  solver's placed extent; sized resizes update the extent length with
  the start never moving; Label/Uuid are identity, exact rather than
  lazy, because this model carries no labels. The simulated snapshot
  is assembled through the real constructors as
  `SnapshotKind::Simulated`, round-trips its own typed boundary, and —
  the 3c property re-asserted at the planner's boundary, held by
  test — **a plan can never revalidate against it**: a prediction is
  not a capture, structurally. The increment-2 chain test moved to a
  wipe-then-wipe chain (signature before device) because the unsized
  create it used is now honestly unplannable — the PLAN-002
  consequence caught by its own enforcement. Four new tests plus the
  moved chain. WP-060's four increments are delivered; the gated
  remainder (cancellation carriage, reversal, backup steps, the SI-17
  combination) waits on the register gates the assignment names.

- **WP-060 increment 3: the extent solver, alignment-conservative.**
  Free space is computed from the snapshot's authenticated extents and
  nothing else — a host's free ranges are its own extent minus the
  extents the facts place on it; where the facts carry no table
  region, the solver does not invent one. Placement is PART-009's
  default and only it: first-fit at the lowest 1 MiB-aligned start
  that holds the full size, with the no-fit refusal naming the largest
  aligned fit so the caller can explain what would have succeeded. The
  two permitted deviation causes — published geometry, explicit user
  override — have no input vocabulary yet, so deviation is
  **inexpressible rather than half-supported**; each arrives with the
  vocabulary that carries it and the body change PART-009's recording
  requires, under WP-010's grant. **SI-15's held case refuses by
  name**: growing a partition whose start is not 1 MiB-aligned matches
  neither deviation cause, so `MisalignedLegacyGrowth` carries the
  target, its actual start, and the gate string `SI-15` — refusing is
  the answer, guessing is what the register exists to prevent.
  `plan_sized` carries solved geometry into the body: a create
  consumes its placed range, a grow consumes its tail extension, a
  shrink destroys its freed tail (bytes beyond the new end are gone,
  and the ranges say so); every sized plan is deterministic to the
  byte and revalidates through the typed boundary. Five new tests.

- **WP-060 increment 2: the step graph.** PLAN-003 lands as explicit
  machinery: request sets carry dependency edges, and `plan_set`
  refuses cycles (with every unorderable member named), duplicate
  requests (before ranges are even compared — a plan that says one
  thing twice is a request error), malformed edges (out-of-range and
  self-dependency each by name), and **dependency-unordered overlaps**
  — the conflict rule this increment commits: two steps whose declared
  effect ranges touch the same bytes of the same host are legitimate
  exactly when a dependency path orders them (a wipe followed by a
  create in the freed space is a chain, and the dependency is its
  explanation), and with no path in either direction no execution
  order makes them deterministic, so the pair refuses naming both
  steps and the host. Every conflict is a typed value that explains
  itself, never a boolean. Ordering is Kahn's with the smallest ready
  index first — deterministic under PLAN-001, held by the byte-equal
  test extended to the two-step chain — and every step of the ordered
  set still constructs individually through `PlanStep::mutating`
  against the capture. Four new tests.

- **WP-060 increment 1: the planner's request vocabulary and pure
  chassis.** `crates/planner` joins the workspace, depending on
  `partman-domain` and `partman-capability` deliberately. `plan()` is
  PLAN-001's computation — deterministic and side-effect free, purity
  structural (no clock: the caller supplies creation time and the
  PLAN-007 window; the 24-hour default is the calling surface's policy
  to apply before this boundary), determinism held by test as
  byte-equal plan bodies. The conditioning rule is ACC-009's planner
  half with CAP-007 both ways: `unsupported`/`blocked` answers refuse
  the request with the engine's answer carried **verbatim** — reason
  and remediation travel, never re-derived — `preview` permits
  planning, `supported` is not a distinct planning state (it differs
  at apply, which does not exist here), and no answer can admit a step
  the closure refuses: every step is `PlanStep::mutating`, every plan
  `OperationPlan::assemble`. Source-class requests refuse as not plan
  material. Severities are conservative-up with the reasoning stated
  in the code: severity 0 never fits a mutating step, and severity 1's
  "fully undoable via an emitted reversal plan" cannot be claimed
  while PLAN-008's emission waits on SI-19 — a Reversible claim
  without the reversal would be the assertion this codebase refuses
  everywhere else. Four tests plus the boundary revalidation;
  traceability converted to generated.

- **WP-050 increment 4: the consumer seams, the multipath arm the
  coverage net caught, and the package record.** The engine's public
  API is documented for its three consumer classes (the CLI rendering
  advisory answers under CAP-007, the planner conditioning planning on
  them with `PlanStep::mutating` staying the sole constructor, the
  adapters producing snapshots and runtime facts and never computing
  verdicts). **The all-reasons coverage requirement did its job before
  it was even written**: `MultipathDetectionOnly` existed in the
  vocabulary with no engine arm producing it — a multipath mutation
  would have fallen through to protection's `RemoteTransport` refusal,
  refusing correctly but reporting the wrong reason, where LIN-006
  requires "a multipath reason from CAP-003's reason vocabulary". The
  arm now exists and **precedes protection deliberately**: LIN-006
  names the reason this population reports, the closure refuses the
  same population anyway (the device-scope transport arm reaches a
  multipath node as not-positively-local), so the precedence moves
  reporting and never permission — the plan constructor still refuses
  these targets on the closure's own ground, held by test with source
  classes passing untouched (detection-only means detection works).
  The coverage test exercises every reachable reason and status over
  integration-shaped fixture topologies and asserts the two
  unreachable members (`supported`, `QualifiedByEvidence`) unreachable
  by proof — increment 1's `compile_fail` doctest — rather than by
  omission. **Recorded narrowing**: the fixture topologies are
  constructed to mirror the WP-020 catalogue's shapes; byte-level
  derivation from the images arrives with the platform adapters.
  Increments 1–4 delivered; the package's remaining obligations are
  consumer-driven (a real qualification row, the evidence loader for
  its first consumer, floors as tools join the roster).

- **WP-050 increment 3: the CAP-006 store, structured and truthfully
  empty.** `docs/capabilities/` exists in the form CAP-006 and
  Section 9 name: `format.md` (the normative format — advertised rows
  per platform/file-system/operation with a closed two-state
  vocabulary, qualified rows required to carry fixture, run, date, and
  transcript digest per Section 16's evidence rule),
  `qualifications.json` (advertised set **empty, the vacuity named**:
  nothing is advertised while no apply path exists; advertising is a
  reviewed act that adds a row unqualified, qualifying is a second
  reviewed act that fills its evidence), and
  `tool-version-floors.json` (empty for the same reason: no storage
  tool is invoked anywhere yet, and a floor for a tool nobody calls is
  an assertion nobody can test). The CI gate is a Tier-1 store test in
  `crates/capability` — the `shared_vectors` pattern, dev-dependency
  only — refusing malformed rows, unknown fields, out-of-vocabulary
  platforms/file-systems/operations, and evidence-less qualified rows,
  with the qualified-row count **pinned at zero** so qualification can
  only arrive as a diff that moves the pin under review. **Delivered
  narrower than the assignment's sentence, and recorded:** the
  evidence token gains no constructor at all — not even crate-internal
  — because both preconditions are vacuous (no row to qualify, no
  consumer that could possess a store at runtime); the loading path
  arrives with the first consumer that embeds qualification evidence
  under its own grant, and the increment-1 `compile_fail` proof holds
  verbatim meanwhile.

- **WP-050 increment 2: the engine core.** `capability()` computes
  CAP-001's conditioning as one entry point — the CAP-003 answer for
  one operation on one exact target over a decoded snapshot,
  caller-supplied immutable limits, and CAP-004-shaped runtime facts —
  composing the decided arms in the assignment's refusal-precedence
  order: the domain's `protection_gate` first (the same closure the
  plan constructor runs), then FS-007's technology limits statused per
  ADR-0020 (`unsupported`, the limit as explicit reason, `NoneExists`
  as an exact remediation), then Section 9's floor and ACC-009's tool
  preconditions as `blocked` with remediations naming the tool, with
  `preview` as the answer no arm refuses — implemented for planning,
  apply refused pending CAP-006 evidence. An address the snapshot does
  not carry is a typed `UnknownTarget` error, not an answer about
  nobody. **The CAP-005 agreement is enumerated, not asserted**: over
  all fourteen operations and six fixture targets (a permitted device,
  a ZFS signature consumed by its pool — refused by inheritance — an
  orphan LUKS2 signature, a transport-less device, an XFS file system,
  and the pool), the engine's protection answer and
  `PlanStep::mutating` agree pair by pair, grounds matching; source
  classes take no protection answer anywhere. The enumeration also
  records a domain semantic the first fixture draft got wrong twice:
  an extent-less target's canonical destructive entry is empty and the
  gate clears at capability time (the plan step's declared ranges
  refuse later), and a consumerless non-goal signature is the orphan
  indeterminacy, not the refusal — the refusal needs the consumer
  chain. CAP-001's mount-state, boot-role, and OS-identity conditioning
  inputs are deliberately not carried as dead fields: each arrives with
  the text that decides its rule (the vacuous-state discipline).

- **WP-010 files SI-40, the FS-007 / CAP-003 conflict**, under Section
  0.2's requirement to file rather than silently pick a side, on the
  grant landed the same day. FS-007 says immutable technical limits
  surface "as explicit blocked reasons"; CAP-003 defines `blocked` as
  "implemented, but a runtime precondition fails" and `unsupported` as
  "the product does not implement the operation for this target". An
  immutable limit is not an implemented operation with a failing
  runtime precondition, so one case draws two statuses from two
  normative texts, and CAP-005 makes the answer product-visible on
  every surface at once. The filing records where it came from: WP-050
  increment 1 building the CAP-003 vocabulary, which deliberately left
  the `TechnologyLimit` reason's status coupling unasserted rather
  than decide this in a constructor. Classified **Later** (Part 2;
  before WP-050 increment 2 composes technology limits — its other
  arms have decided couplings and do not wait). Three readings are
  recorded as options, none recommended, with one classification fact:
  the vocabulary-noun-phrase reading amends no normative text, while
  the literal-`blocked` reading retexts CAP-003's definitions.

- **WP-050 increment 1: the CAP-003 vocabulary, structural.**
  `crates/capability` joins the workspace — depending on
  `partman-domain` deliberately, because composing the domain's
  protection gate is this engine's purpose (the CLI chassis's
  empty-closure guard is that package's rule, not the workspace's).
  CAP-003's four statuses land with their definitions held by
  construction: `supported` is constructible only through
  `QualificationEvidence`, which has **no constructor in this
  increment** — the evidence store is increment 3, no apply path exists
  anywhere in the product, and unreachable is the correct answer,
  proven by a `compile_fail` doctest (the ADR-0012 pattern). The
  closed reason vocabulary (`partman.capability.reason`, MODEL-003
  version 1) carries at birth exactly the reasons decided texts name —
  ADR-0018's protection grounds re-enumerated through exhaustive
  `From` impls so a domain arm added later fails compilation here and
  the version bump becomes a reviewed decision, ADR-0011/LIN-006's
  multipath detection-only reason, ACC-009's two tool arms, FS-007's
  technology limit, Section 9's platform floor, CAP-003's own
  pending-evidence ground, and the evidence-built reason that panics
  in every assertive constructor so an unqualified answer cannot be
  dressed as qualified (CAP-007's no-upgrade rule at the type layer).
  `from_protection_gate` carries 3g's decided coupling — refusals to
  `unsupported`, indeterminacies to `blocked`, `Clear` producing no
  answer at all so the engine keeps composing. **Deliberately
  unasserted: `TechnologyLimit`'s status coupling.** FS-007 says
  immutable limits surface "as explicit blocked reasons"; CAP-003
  defines `blocked` as "implemented, but a runtime precondition
  fails" — an immutable limit is neither, and CAP-003's `unsupported`
  ("the product does not implement the operation for this target")
  reads as that case's home. Two requirements assigning one case
  different statuses is Section 1.11's shape: the conflict is to be
  filed on the register under its own grant before increment 2
  composes technology limits, not decided silently in a constructor.
  Six tests plus the doctest proof; traceability converted to
  generated on the WP-010/WP-035 precedent.

- **WP-010 increment 3k: the body-format record and its cross-language
  proof.** The three body schemas increment 3 delivered get their
  `schemas/domain/` documents — `topology-snapshot-body.md`,
  `plan-body.md`, `node-entry-format.md`, each recording a decided
  format and deciding nothing (a field exists because a slice delivered
  it, never because a document says so) — and the MODEL-005 parity
  proof extends to them through one shared fixture,
  `schemas/domain/body-vectors.json`. The vectors are the Rust
  constructors' own pinned output: four snapshots (minimal, its
  simulated/transitional twin, the full capture with every fact class
  and a collision group, the plan base), two plans over that base (the
  bare destructive wipe and its identity-bound twin, each recording
  the digest of the snapshot vector it binds — the PLAN-006 binding
  held across the fixture itself), and nine node entries pinned
  standalone and required to appear verbatim in their snapshot's
  `nodes` set. `crates/domain/tests/body_vectors.rs` proves the
  constructors reproduce every recorded byte and the typed boundaries
  round-trip them; `packages/canonical/src/body-vectors.test.ts`
  proves the TypeScript codec reproduces the same bytes and digests
  from the same trees, riding the existing required cross-language
  job. No domain constructor exists in TypeScript on purpose: that
  side re-encodes decided trees, it does not build topologies.

- **WP-010 increment 3j: the authoring set, structural.** MODEL-005's
  two authored fields land in the shapes their ADRs decided. ADR-C3's
  table state becomes a snapshot **fact** — body content per device,
  stamped when the helper produces the snapshot at validation
  (ADR-0014's stamp point realized as fact carriage), kind-checked to
  physical devices, round-tripping under the boundary's
  decode-recompute equality. Section 6's bound identities enter the
  plan body keyed by target address with strength still derived, and
  the plan boundary enforces the rule ADR-0014 wrote: **a plan
  identity whose table state disagrees with the snapshot's stamp
  refuses as `AuthoredFieldMismatch`** — the client-authored value
  that never validates, held by test with a forged Present against a
  stamped Indeterminate. The second authored field — the derived
  protection verdict — is committed through its body-carried inputs
  (topology, facts) rather than stored beside them: a pure function
  of authenticated bytes cannot disagree with its inputs, which is
  the same anti-assertion mechanism ADR-C4 chose for confidence and
  3d chose for strength, and it satisfies ADR-0016's substance (the
  verdict a user authorizes is in the bytes they authorize; a client
  cannot author it because there is nothing to author). One test:
  the agreeing identity round-trips, the forged one refuses.

- **WP-010 increment 3i: the plan body and its boundary — the
  hand-forged artifact, refused.** `model::plan` lands PLAN-004's risk
  model (the five ordinal severities, the five orthogonal flags
  mirroring the requirement's own enumeration, plan severity as the
  step maximum) and the Section 6 body skeleton for every item whose
  vocabulary is decided today: schema identity, plan id, creation
  timestamp, the source snapshot's body hash as bound at validation
  (8.0.0's rule), PLAN-007's validity window as body content
  (enforced, never re-derived — ADR-C2's row), and the step graph as
  a semantic dependency array. `from_canonical_body` takes the plan
  bytes and the snapshot they claim to bind, refuses the
  wrong-snapshot presentation (the ACC-007 stale-plan shape at the
  type layer), and re-runs every step through the sole constructor —
  so the hand-forged test's forged bytes, a clean plan retargeted at
  a pool-carrying device by editing the value tree, refuse through
  the same closure that would have refused the honest construction.
  That is ADR-0012's second verification row discharged at the
  boundary this crate owns; the helper's fresh re-discovery supplies
  the snapshot at validation (HLP-002). Remaining Section 6 items
  land as their owning vocabularies (WP-050/WP-060) arrive. Five
  tests: the exact round-trip, the wrong-snapshot refusal, the forged
  step's refusal by recomputation, severity maximization, and strict
  unknown-field refusals.

- **WP-010 increment 3h: the mutating step's sole constructor, and
  ADR-0012's proof (spec 4.4.0's commitment, discharged).**
  `model::step` lands `PlanStep` with private fields and one
  constructor: `PlanStep::mutating` runs ADR-0018's closure over the
  snapshot's own authenticated facts and returns a typed refusal
  instead of a value for any non-permitted reach — a mutating sentence
  naming a protected node has no spelling, and the `compile_fail`
  doctest is the construction-refusal proof in the pattern the CLI
  chassis set, verified by the compiler on every test run. ADR-0018's
  acknowledgment vocabulary lands closed at its decided three:
  `Release` converts exactly the orphan-signature indeterminacy on
  exactly the node it names (recorded at plan creation, re-derived at
  validation where a consumed object diverges and rejects);
  `OpaqueDestruction` and `IdentityBoundRestore` are carried so the
  set is closed but refuse at construction until their arms exist; a
  refused node is coverable by no acknowledgment — the
  consumed-member case is deliberately unrepresentable, which is what
  separates this from PART-014's bypassable gloss. Five tests: the
  refused-reach-even-acknowledged refusal, the orphan's lawful
  release, wrong-node and unmodelled-kind refusals, and the clean
  construct. The helper's independent recomputation at validation is
  the retained second layer, landing with the plan boundary.

- **WP-010 increment 3g: the protection gate on capability (ADR-0018's
  canonical-step rule).** `model::capability` models CAP-002's fourteen
  operations separately with ADR-0018's class partition — detect, read,
  check, and copy-as-source are source class and never suppressed by a
  verdict (WIN-003's detection duty and WIN-004's copy-off escape stay
  advertised); the ten mutating operations are gated. Each mutating
  operation defines a canonical effect-table entry over its target —
  the minimal invariant ranges derivable with no plan in scope, with a
  destructive operation destroying the target's extent and content
  operations deferring to the plan step's authoritative declared
  ranges — and `protection_gate` is the same closure the constructor
  runs, so the capability surface and the planner cannot disagree on a
  target/operation pair: CAP-005 agreement is enumerated over every
  pair in the pool layout rather than argued. Refusals map to
  `unsupported` with the citing ground; indeterminacies map to
  `blocked`, remediable — the orphan-signature host blocks rather than
  refuses forever. `Clear` is deliberately not a CAP-003 `supported`
  claim: WP-050's engine layers tool, version, and evidence gates on
  top, and CAP-007 keeps every client-shown status advisory. Five
  tests, including the enumerated agreement sweep.

- **WP-010 increment 3f: the protection facts are body content.**
  The snapshot body carries, per node, the evidence-contract facts
  the closure consumes — host-qualified extents, the device transport
  class, the aggregate's self-reported member count — extending
  ADR-0016's logic to the verdict's inputs: what the verdict reads,
  the authorization commits to. A fact edit moves the body hash; the
  facts round-trip through the typed boundary and are covered by its
  decode-recompute equality; a fact on a kind that does not carry it
  (a transport on an aggregate, a member count on a device, an extent
  on a volume) is a typed `MisplacedFact` refusal. The full-stack
  regression closes the loop the whole increment built toward:
  encode a body carrying a ZFS member and its pool, decode it at the
  boundary, and the rebuilt snapshot's **own authenticated facts**
  refuse initializing the device through the pool — no out-of-band
  input anywhere. `TopologySnapshot::step_constructs` is the
  convenience that runs ADR-0018's closure directly off a decoded
  body. The plan-step constructor with ADR-0012's compile-fail proof,
  the acknowledgment vocabulary, and the canonical-step capability
  computation remain the next slices.

- **WP-010 increment 3e: ADR-0018's protection layer as pure
  functions.** `model::protection` lands the closure and verdicts the
  register's longest round decided: the three-valued verdict whose
  residual arm is `Indeterminate` — never `Permitted`, round three's
  fail-open default inverted — with the enumerated arms (ZFS, Storage
  Spaces, and LDM refuse; Fusion refuses by ADR-C5's self-reported
  member count and permits at one; the device-scope transport arm is
  the closed positive local list with recognized-remote refusing and
  unrecognized indeterminate; orphan signatures are the remediable
  indeterminate arm; collision groups are never operands), node-local
  inheritance from a node's own producer and own root device only,
  the effect-table range sets with release counted as destruction,
  and the affected-set fixpoint. The fixpoint carries two destruction
  classes deliberately: range-destroyed nodes are reached by the
  declared ranges themselves and never cascade through containment —
  the module's first draft cascaded them, re-derived round two's
  sibling capture through a device's own self-extent, and was caught
  by the committed regression before it ever compiled green — while
  cascade-destroyed consumers and products (whose substrate died with
  their evidence or producer) descend into their hosted content. The
  committed regressions hold: creating beside a pool member
  constructs while initializing the device refuses through the pool;
  the sibling ESP is never captured; and the round-three killer — the
  LUKS descent — reaches the pool below through production over
  destroyed substrate. Facts (extents, transports, member counts)
  arrive as evidence-contract inputs; carrying them in the snapshot
  body, and the plan-step constructor with its compile-fail proof,
  are later slices. Nothing here authorizes anything.

- **WP-010 increment 3d: SAFE-003's identity record (ADR-C3, ADR-C4's
  guard, ADR-0014's vocabulary, ADR-0015, ADR-0017).**
  `model::identity` lands the immutable target record: all available
  identifiers as contract-source-verbatim bytes, geometry, ADR-C3's
  three-valued table state — with ADR-C4's guard held in bytes
  (present, absent, and indeterminate are three pairwise distinct body
  values, by test) — and ADR-0017's continuity witness. Strength is
  derived, never stored: no field carries it, `strength()` computes
  SAFE-003's rule (a device whose table failed to parse cannot be
  Strong even with a serial; a positively absent table supports
  Strong), and a forged `strength` key in body bytes refuses as an
  undeclared field. The witness comparison follows the measured
  semantics exactly: comparable only within an unchanged epoch token,
  a decrease is a reset the token failed to witness and is
  incomparable, movement is exchange-observed, and the closed outcome
  vocabulary contains no word stronger than the liveness ceiling's
  own. The helper-authored enforcement for the table state lands at
  the plan boundary in a later slice; this module defines the shared
  vocabulary. Seven tests, including the ADR-C4 verification row and
  every witness arm.

- **WP-010 increment 3c: the snapshot body, envelope, and typed
  boundary (MODEL-003, MODEL-004, MODEL-005, MODEL-006, ADR-C2,
  ADR-C4, CONC-004).** `model::snapshot` gives the topology snapshot
  its hashed body — two schema identifiers so captured and simulated
  topologies are domain-separated and identical content never hashes
  equal across the two worlds; the CONC-004 transitional marking in
  the body so a transitional snapshot can never masquerade as stable;
  nodes and edges as MODEL-006 sorted sets — and its unhashed
  envelope, whose capture timestamp and MODEL-004 provenance move no
  body hash, which is what keeps PLAN-006 satisfiable.
  `from_canonical_body` is the typed decode/validate boundary the
  codec-remediation section mandated: strict `pce/1` decode, a
  schema-validation pass with its own error type (unknown fields,
  schemas, versions, malformed entries, forged collision counts all
  typed refusals), MODEL-006 order validation that refuses rather
  than repairs, and the decode-recompute equality — the parsed
  content is rebuilt through the same absorption and edge validation
  the encoder ran, and the rebuilt body must reproduce the input
  bytes exactly, so a forged forbidden edge refuses at decode.
  `model::provenance` lands ADR-C4's observation set with confidence
  derived and never stored (no constructor exists to store one), and
  observed absence as a value that conflicts with a presence rather
  than collapsing into unavailability. Eleven tests, including the
  ADR-C4 verification rows and both boundary-forgery refusals.
  Identity records, verdicts, and plan types remain later slices;
  nothing here authorizes anything.

- **WP-010 increment 3b: edges and topology construction (ADR-0019,
  ADR-0018's semantics-class handover).** `model::topology` adds the
  five MODEL-002 edge kinds, each carrying its semantics class —
  containment, backing, production, and host-backing are
  bytes-within-or-derive and bind-traversed; platform-membership is
  platform-asserted, detection-only, and bind-inert until the spec
  change ADR-0011 names — and `Topology::build`, the fail-closed
  construction boundary: nodes absorb per 3a's collision rule, and
  edges are refused as typed values (never a panic, never an encoder
  failure) for unknown referents, self-edges, duplicates, and any
  endpoint pair outside the edge kind's pair table. The
  no-sibling-capture theorem's premise — no backing, production, or
  host-backing edge targets a physical device — is a property of that
  table, enforced at construction and proved by exhaustive enumeration
  over every (kind, source, target) triple rather than sampled. The
  assembled-multipath shape is a committed regression: two
  equal-identity paths group, and the membership edge targets the
  grouped member entry. Snapshots, provenance, schema versioning, and
  the typed decode/validate/hash boundary remain later slices; nothing
  here hashes an artifact or authorizes anything.

- **WP-010 increment 3a: node naming lands as code (ADR-0019).**
  `crates/domain` gains the `model::naming` module: `NodeId` as a
  derived, document-local positional address — the SHA-256 of a
  domain-separated canonical preimage (`partman.node-id`, version 1)
  over the node's kind tag and ADR-0019's per-kind naming fields, with
  parent addresses embedded as digest bytes — plus collision-group
  absorption: same-kind nodes deriving equal addresses collapse into
  one counted entry whose construction is a deterministic,
  order-independent function of the observed multiset, flagged
  `duplicate_designator` for the cloned-aggregate case, total over
  every well-formed multiset so no observed content makes a node set
  unrepresentable. Identifier bytes are contract-source-verbatim (no
  case folding, no prefix stripping); the exclusion list stays out of
  every naming map. Twelve tests land the committed regressions:
  determinism, the ancestor-only address property, the L9
  byte-identical group with count, order-independence, count
  correctness, the duplicate-designator flag with
  nothing-re-designates, the stale-pair two-address case, the hybrid
  aliased-extent two-view case, unrecognized-discriminant raw-byte
  distinctness, designator-absent aggregate grouping, and distinct
  backing files as distinct addresses. Node payloads, edges,
  snapshots, provenance, and the typed decode/validate/hash boundary
  are later slices; nothing here hashes an artifact or authorizes
  anything.

### Changed

- **spec-change 12.9.1: SI-26 is resolved by ADR-0032 — Section 16's
  "Stable" is CAP-003's `supported`.** A 2.0.0-era stale synonym; the
  evidence rule already lives at `supported` ("backed by matrix
  evidence (CAP-006)"), so the prohibition and the definition are one
  rule seen from two sections. No maturity axis — the vocabulary-
  doubling shape ADR-C3 removed, for a qualified-but-immature state no
  requirement recognizes. WP-050's delivered engine already enforces
  the reading structurally (the CAP-006 token has no constructor until
  a qualifying row exists). Landed as a patch: the ADR-0020
  reading-selection shape with a one-phrase editorial parenthetical
  instead of a bare banner, the major counter-argument recorded in the
  ADR. Accepted by Nate McBride 2026-08-12 by directive ("finish SI-25
  and SI-26"), recorded as the ADR's acceptance basis. CAP-003's four
  values stand verbatim.

- **spec-change 12.9.0: SI-25 is resolved by ADR-0031 — CAP-002 is a
  required minimum over a closed-and-versioned operation vocabulary.**
  The floor keeps DIA-005 implementable and PART-007/010/011
  representable; the instant-closure keeps CAP-005's one-engine promise
  stable (no surface carries an operation the versioned vocabulary does
  not). `wipe` is a family whose six DIA-005 kinds become separate
  operations when erase surfaces are built — capability differs per
  kind, never-equivalent made structural; the kind-discriminant was
  rejected as equivalence at the modeling layer. The delivered
  `crates/capability` `Operation` enum stands until WP-050's next
  reviewed increment extends it under this discipline. Rejected and
  recorded: closed enumeration, unversioned minimum. Minor under §0.1:
  CAP-002's sentence verbatim, the rules additions. Accepted by Nate
  McBride 2026-08-12 by directive ("finish SI-25 and SI-26"), recorded
  as the ADR's acceptance basis.

- **spec-change 12.8.0: SI-23 is resolved by ADR-0030 — the REC-011
  backup is a first-class protection artifact.** Four rules for the
  object the spec mandated and then said nothing about. Home: a
  dedicated helper-owned store inheriting JRN-004's admin-protected
  location clause, sibling to and never inside the journal (JRN-005's
  bounds stand; ADR-0029's budget unbloated; ADR-0029's named fork
  answered — the lifecycle does not route through the journal).
  Reference by identity: journal, plan, and every SAFE-006 surface
  carry the content hash and store identity only — SAFE-006's list
  verbatim, a hash is not the material, helper-only reads (SAFE-008),
  restores as identity-validated plans (REC-001) at their own tier.
  Retention: ADR-0029's liveness rule adopted in REC-011's own text —
  exempt while the creating apply or its referencing closure is
  non-terminal, "RecoveryAction must reach it" made structural. End of
  life: explicit user-controlled retention in SEC-009's shape, never
  silent in either direction — retention preserves revoked-passphrase
  slots, deletion forfeits the only disaster-recovery copy, both
  stated at the deciding surface, defaults displayed and changeable.
  Rejected and recorded: journal embedding, arbitrary location,
  auto-delete with its silent-retention mirror. ADR-0024's
  corrupt-source discharge stays WP-R100's. Minor under §0.1:
  REC-011's two sentences verbatim, the rules additions. Accepted by
  Nate McBride 2026-08-12 by delegation, recorded as the ADR's
  acceptance basis. Store layout and encoding land with
  WP-R100/WP-070, jointly sequenced; no re-attribution follows —
  neither assignment exists, and the ADR records the verification
  obligations so their creation cannot omit them.

- **spec-change 12.7.0: SI-22 is resolved by ADR-0029 —
  liveness-scoped retention.** Bounded and unbounded stop colliding
  when they stop sharing a population: retention MAY reclaim only
  records of terminal applies, a non-terminal apply's records (the
  authorization act's included — ADR-0028's fed-forward fact absorbed,
  its revisit condition discharged) are exempt until their apply
  terminates, and the exemption closes over ADR-0027's linkage graph.
  JRN-004's bound stays true universally: terminal history under
  SEC-009's retention controls, the live segment under a per-apply
  journal budget whose exhaustion is a journaled failure through
  Section 8's existing edges — fail-closed toward the writer, never
  the recoverer, the round's sharpest finding turned into an enforced
  property. Reclamation writes a durable compaction record; replay
  classifies every gap as policy, torn tail, or corruption; sequence
  numbers are never reused or reset across rotation or compaction.
  Rejected and recorded: retention-wins (the filed trap ratified),
  recovery-wins-forever (unbounded journal), time-capped exemption
  (the hazard re-created on the unbounded state). Minor under §0.1:
  JRN-004's sentence verbatim, the rule additions; JRN-001, JRN-003,
  SEC-009, Section 8, SAFE-005 untouched. Accepted by Nate McBride
  2026-08-12 by delegation, recorded as the ADR's acceptance basis.
  The budget's magnitude and compaction encoding land with JRN-006
  under WP-070, jointly sequenced; no re-attribution follows — no
  WP-070 assignment exists, and the ADR records the verification
  obligations so its creation cannot omit them.

- **spec-change 12.6.0: SI-21 is resolved by ADR-0028 — an
  authorization act authorizes one apply, a journal-continuous
  lifecycle that interruption suspends and only terminals end.** No
  reuse occurs on Section 8's three re-entry edges because nothing is
  used twice: resume and roll-forward continue the same apply under
  the same journaled, hash-bound, single-use act, consumed once at the
  apply's start; "unbroken" is what JRN-001 already guarantees. The
  authorization is a journal fact, never process state — the
  helper-exit worry dissolves into JRN-003 and HLP-005 — and the
  caching prohibition forbids approvals outliving their apply, not
  applies outliving interruptions. Freshness is bounded by PLAN-007's
  existing machinery: re-entry past the window is rejected per HLP-004
  and readmitted only through re-approval against a fresh snapshot, a
  fresh act for the same continuing apply. Each edge keeps its named
  verification; WIN-009 reads as continuity, not a grant. Rejected and
  recorded: re-prompt-every-resume (rubber-stamp economics, new table
  edges), retained helper state, severity-scaled resume prompting.
  Fed forward to SI-22, undecided: the authorization record is
  recovery-critical. Minor under §0.1: additions defining a term the
  ladder used; every pre-existing sentence verbatim. Accepted by Nate
  McBride 2026-08-12 by delegation, recorded as the ADR's acceptance
  basis. No re-attribution follows — no WP-070 assignment exists, and
  the ADR records the verification obligations so its creation cannot
  omit them.

- **spec-change 12.5.0: SI-20 is resolved by ADR-0027 — the two
  RecoveryRequired exits are the two arms.** A roll-forward action
  continues the original plan (same hash, same journal, resuming from
  the last durable checkpoint through the existing → Executing edge,
  re-verification inherited from JRN-003) and is the one recovery act
  that is not its own plan — the prose sentence scoped, every other
  instance true. Any distinct recovery action is its own
  `OperationPlan`, and selecting it is the acceptance the → Failed
  trigger names: honest effect summary, full report, journaled linkage
  naming the recovery plan, one user act driving two records, with the
  disposal durable before the recovery plan may apply (JRN-002's shape,
  HLP-005-structural on shared device sets — the filed torn state
  unreachable). No state, edge, or trigger added; the rows, terminal
  list, and "No other transitions exist" stand verbatim; no
  → Cancelled edge, since unwind semantics belong to the Executing
  era. Rejected and recorded: recovery-as-the-original (breaks hash
  binding), new exits or `Superseded` (couples lifecycles or renames a
  fact), rewording the Failed row (major for what prose does at
  minor). SI-21 untouched on both edges. Minor under §0.1:
  closing-prose additions only. Accepted by Nate McBride 2026-08-12 by
  delegation, recorded as the ADR's acceptance basis. No
  re-attribution follows — no WP-070 assignment exists, and the ADR
  records the verification obligations so its creation cannot omit
  them.

- **WP-060: the assignment stops citing SI-24 as an open gate — its
  register-gate list is now empty.** The established re-attribution
  shape (#261/#264/#267/#270/#273): citing the retired question would
  be the drift the register's sole-authority rule forbids. The
  register-gates row records SI-24 as resolved (ADR-0026, spec 12.4.0:
  a dry run is an apply rehearsal, not CAP-003's simulation — it runs
  and refuses at the helper's own recomputed capability gate with a
  typed pending-qualification reason, never successful, PLAN-009's
  guarantee absolute); the PLAN-009 consumed-not-claimed line and the
  no-register-answer boundary bullet move from open gate to recorded
  decision, with the principle preserved for any newly filed gate.
  README's WP-060 row moves with the bytes it describes; the spec pin
  moves to 12.4.0. Every gate the assignment ever named — SI-15, SI-16,
  SI-17, SI-19, SI-24 — is now resolved through its own recorded
  decision, and four increments are startable (reversal, solver unlock,
  backup family, combination unlock), with the planner's gate-list
  comment and refusal debts riding the first Rust change among them.
  No code, test, type, or normative text changes.

- **spec-change 12.4.0: SI-24 is resolved by ADR-0026 — a dry run is an
  apply rehearsal, not CAP-003's simulation.** The conflict turned on
  one undefined word the spec's own vocabulary had already split:
  `preview` licenses the pure planner surface (PLAN-001 planning,
  PLAN-002's simulated final topology), while a PLAN-009 dry run
  belongs to the apply surface `preview` refuses. A dry run of a
  preview-backed plan runs — not refused upfront from the client's
  advisory view (CAP-007's inversion) — and terminates at the helper's
  own recomputed capability gate with a typed refusal naming the
  qualification gap and its CAP-006 remediation, distinguishable by
  type from every validation-failure class. Such a dry run is never
  successful, so PLAN-009's guarantee stands absolute with no
  success-with-caveat outcome representable; the pipeline's internal
  gate order is deliberately not decided (parity is the property,
  sameness of the refusal pair the tested fact, the order WP-070's).
  Rejected and recorded: success-with-carried-caveat, the partial
  pipeline, narrowing `preview`, upfront client-side refusal. Minor
  under §0.1: both existing texts stand verbatim, the additions define
  an undefined word and an unaddressed case. Accepted by Nate McBride
  2026-08-12 by delegation on the previous day's round, recorded as
  the ADR's acceptance basis. WP-060's last register gate clears;
  decided before the pipeline exists deliberately — the decision
  constrains the implementation rather than reading it, the ADR-0022
  class.

- **WP-060: the assignment stops citing SI-17 as an open gate — SI-24
  is the one that remains.** The established re-attribution shape
  (#261/#264/#267/#270): citing the retired question would be the drift
  the register's sole-authority rule forbids. The register-gates row
  records SI-17 as resolved (ADR-0025, spec 12.3.0:
  `irreversible-after-start` defined temporally — the flag claims the
  mid-execution window, severity claims endpoints, the combination
  legal, a flagged step's cancellation claiming `no-writes` only before
  its first write); the boundary bullet and the post-increments
  paragraph move from open gate to recorded decision, with the
  combination-unlock increment startable and its fixtures named.
  README's WP-060 row moves with the bytes it describes; the spec pin
  moves to 12.3.0. The planner's contested-combination refusal and its
  gate-list comments are Rust and stand as delivered — behavior for a
  reviewed increment, not a comment sweep — riding the same future
  Rust change as the SI-19/SI-15/SI-16 debts (the `39b59f5`
  stopping-condition economics). SI-24 remains open, refused
  conservatively as before. No code, test, type, or normative text
  changes.

- **spec-change 12.3.0: SI-17 is resolved by ADR-0025 —
  `irreversible-after-start` claims the mid-execution window; severity
  claims endpoints.** The flag is defined for the first time: a step
  carries it when a reachable interrupted state exists from which the
  pre-step state cannot be restored by unwinding — recovery past the
  first write is roll-forward per the journal, never unwind — with the
  criterion a reachable unrestorable intermediate, not the existence of
  a write. Severity 1's "fully undoable before or after apply"
  quantifies over endpoints (ADR-0022's completed-apply boundary), so
  the combination is legal and PLAN-004's declared orthogonality
  becomes true. One coupling rule: a flagged step's cancellation claims
  `no-writes` only before its first write — Section 8's existing effect
  values, selected, not extended; cannot-stop and cannot-unwind stay
  independent in both directions. No new guard needed: any flag binds
  the ADR-0021 ceremony, the reversal-draft obligation stands, UI-005
  displays both facts. Rejected and recorded: permanent illegality
  (severity inflation or flag suppression — the 2.0.0 conflation in
  reverse), endpoint-irreversibility as the definition, dropping the
  flag. Minor under §0.1: the flag had no prior definition; severity
  1's text, PLAN-005, and Section 8 stand verbatim. Accepted by Nate
  McBride 2026-08-11 by delegation, recorded as the ADR's acceptance
  basis. The planner's combination refusal unlocks, riding the crate's
  next Rust increment; SI-24 is WP-060's one remaining register gate.

- **WP-060: the assignment stops citing SI-16 as an open gate.** The
  established re-attribution shape (#261/#264/#267): citing the retired
  question would be the drift the register's sole-authority rule
  forbids. The register-gates row records SI-16 as resolved (ADR-0024,
  spec 12.2.0: PART-013 discharges by the helper's authored table
  state — parse-level backup on `Present`, a journaled positive
  determination on `Absent` with no user acknowledgement, a verified
  raw capture of the write-target regions for the typed REC-001 repair
  family on `Indeterminate`, capture-impossible refusing except under a
  plan-creation acknowledgement naming the regions); the boundary
  bullet and the post-increments paragraph move from open gate to
  recorded decision, with the backup-family increment startable and its
  fixtures named. README's WP-060 row moves with the bytes it
  describes; the spec pin moves to 12.2.0. The `crates/planner`
  gate-list comment naming SI-16 is Rust and rides the same future
  increment as the SI-19 and SI-15 debts (the `39b59f5` stopping-
  condition economics, recorded in #264/#267). SI-17 and SI-24 remain
  open, refused conservatively as before. No code, test, type, or
  normative text changes.

- **spec-change 12.2.0: SI-16 is resolved by ADR-0024 — PART-013
  discharges by the helper's authored table state.** Each of the
  filing's three options is right somewhere, and the error was choosing
  one for all cases. `Present`: parse-level backup untouched, verified,
  failure → Failed. `Absent`: the obligation discharges as a journaled
  determination — the backup record is the positively determined
  absence, a value not a skip (ADR-C4 reaching the journal), the same
  fresh determination PART-001 requires, with no user acknowledgement.
  `Indeterminate`: ordinary operations stay SAFE-005-disabled before
  PART-013 is reached, while the typed REC-001 repair family — a step
  class, never an intent flag — backs up a verified raw capture of
  exactly the regions it will write; capture-impossible refuses per
  Section 8's existing row, with Section 12's own
  separately-supported-recovery-strategy exit formalized as a
  plan-creation journaled acknowledgement naming the uncapturable
  regions. A blank device and an unreadable one never take the same
  arm, and no arm is silent. Rejected and recorded: uniform vacuous
  satisfaction (fail-open on corrupt media), uniform acknowledgement
  (ceremony where it cannot inform), uniform block (the filing's own
  reductio). Minor under §0.1: PART-013's sentence stands verbatim and
  the arms are additions; SAFE-005, Section 8, REC-011, and the
  MUST-NOT clause untouched. Accepted by Nate McBride 2026-08-11 by
  delegation, recorded as the ADR's acceptance basis. The protection
  record's journal encoding lands with JRN-006 under WP-070, jointly
  sequenced; the backup step family stays unbuilt until its own
  increment; SI-17 and SI-24 stay open.

- **WP-060: the assignment stops citing SI-15 as an open gate.** The
  #261/#264 re-attribution shape: citing the retired question would be
  the drift the register's sole-authority rule forbids. The
  register-gates row records SI-15 as resolved (ADR-0023, spec 12.1.0:
  a PART-009 deviation is authored, not inherited — the filed
  grow-at-tail case proceeds, authoring only the aligned new end, the
  untouched start an inherited fact in consequence text, coincident-
  edge placement conformant); the boundary bullet, the increment-3
  delivery note, and the post-increments paragraph move from open gate
  to recorded decision, with the solver unlock startable as a
  separately reviewed increment implementing the ADR's fixtures.
  README's WP-060 row moves with the bytes it describes; the spec pin
  moves to 12.1.0. **The solver's refusal itself stands in code as
  delivered** — `crates/planner`'s named SI-15 refusal, its test, and
  its doc comments are Rust, any non-Markdown merge re-opens the three
  VM acceptances pinned at `39b59f5`, and the refusal is behavior a
  reviewed increment should change, not a comment sweep — so the
  unlock increment (with the SI-19 comment debt riding the same
  change) is where the code catches up, and until then the refusal
  cites a decision rather than an open question, which the assignment
  now says in place. SI-16, SI-17, SI-24 remain open, refused
  conservatively as before. No code, test, type, or normative text
  changes.

- **spec-change 12.1.0: SI-15 is resolved by ADR-0023 — a PART-009
  deviation is authored, not inherited.** An authored boundary (byte
  offset set by the plan) meets the 1 MiB default, is coincident with a
  pre-existing structural edge (conformant, recorded — the round's
  sharpest finding: without this the same issue re-files about the
  grown end), or carries one of the two existing deviation causes; no
  fourth state. A boundary byte-identical before and after is an
  inherited fact — no override, no block, recorded in consequence text
  as a fact about the device. The filed case proceeds: growing a legacy
  misaligned MBR partition at its tail authors only the aligned new
  end; realignment stays an explicit PART-005 move at severity 3, so a
  grow is never silently a move in either direction. Rejected and
  recorded: the strict reading (safety theater), auto-realign (severity
  laundering), permanent refusal (fail-closed posture spent where no
  failure exists), typed alignment-fact carriage (revisit condition).
  Minor under §0.1: PART-009's two pre-existing sentences stand
  verbatim and the scoping is additions, with the major
  counter-argument recorded in the ADR. Accepted by Nate McBride
  2026-08-11 by delegation, recorded as the ADR's acceptance basis.
  WP-060's named solver refusal unlocks, the code change riding the
  crate's next Rust increment with the SI-19 comment debt; the
  deviation-override vocabulary stays deliberately inexpressible;
  SI-16, SI-17, SI-24 stay open.

- **WP-060: the assignment stops citing SI-19 as an open gate.** The
  #261 re-attribution shape applied to the package SI-19's resolution
  unblocks: citing the retired question would be the drift the
  register's sole-authority rule forbids. The register-gates row now
  records SI-19 as resolved (ADR-0022, spec 12.0.0: the reversal is an
  ordinary `OperationPlan` draft bound at its own validation after the
  forward apply, linked by reference — `OperationPlan` is not
  recursive — with step-output references for created-node targets and
  two-time truthfulness as draft preconditions); the PLAN-008
  requirement line, the no-register-answer boundary bullet, and the
  post-increments paragraph move from gate to recorded decision, with
  the reversal increment startable as the next separately reviewed
  increment and the linkage byte encoding still a jointly-sequenced
  WP-010 schema change; README's WP-060 row moves with the bytes it
  describes; the assignment's spec-version pin moves to 12.0.0. The
  delivered increments' rows keep their withheld-Reversible and
  refused-gate phrasing as history of what shipped while the gate
  held. SI-15, SI-16, SI-17, SI-24 remain open gates, refused
  conservatively as before. **Deliberately not touched: the two
  `crates/planner` doc-comment references to SI-19** (`lib.rs`'s gate
  list, the withheld-Reversible comment) — any non-Markdown merge
  re-opens the three VM acceptances pinned at `39b59f5` (the #261
  precedent), so the comment re-attribution rides the next Rust change
  to the crate, which the now-startable reversal increment will be. No
  code, test, type, or normative text changes.

- **spec-change 12.0.0: SI-19 is resolved by ADR-0022 — the reversal is
  an ordinary draft, linked by reference, and `OperationPlan` is not
  recursive.** The filing predated 8.0.0, which dissolved its core:
  binding is a validation act for every plan, so a reversal emitted at
  planning time is exactly as unbound as every other draft. The draft's
  proposal is the forward plan's simulated final topology; its binding
  is its own validate-plan after the forward apply, so nobody ever
  applies a prediction and the delivered Simulated-never-binds rule
  stands untouched. Section 6's body item becomes reversal linkage —
  the draft's plan ID and body hash, acyclic by construction
  (forward→hash, reversal→ID, mutual hash references being
  unconstructible). Round three's created-node residue gets its only
  possible spelling: typed step-output references resolved to derived
  addresses at the reversal's validation per ADR-0019, refusing when
  unresolvable. Truthfulness is a two-time property re-checked as
  body-content preconditions — the volume-that-gained-data fixture
  refuses rather than silently becoming destructive — and a reversal
  apply takes its own ADR-0021 authorization at its own severity and
  flags. Rejected and recorded: binding the simulated topology
  (collides with a delivered mutation-tested rule), exemption (the
  fail-open arm), lazy re-planning with no emission (kills REC-010's
  advertisement and severity 1's definition; survives as the staleness
  fallback), recursive embedding (regress, depth budgets, frozen-draft
  agreement obligation). Major under §0.1: PLAN-008's and Section 6's
  existing texts change meaning. Accepted by Nate McBride 2026-08-11 by
  delegation, recorded as the ADR's acceptance basis. WP-060's PLAN-008
  increment unlocks; the linkage byte encoding lands as the
  jointly-sequenced WP-060/WP-010 schema change when implemented;
  SI-15/16/17/20/24 and REC-* stay open.

- **WP-040: the assignment and skeleton documents stop citing SI-18 as
  an open gate.** The re-attribution shape of #197/#215, applied to the
  package the resolution unblocks: citing the retired question would be
  the drift the register's sole-authority rule forbids. The assignment's
  register-gates row now reads none open and records the answer it was
  waiting on (ADR-0021, spec 11.2.0: two-tier ladder, tier
  helper-derived from recomputed severity and flags, no plan-carried
  authorization-requirement field — so the jointly-sequenced WP-010
  schema change the assignment anticipated does not arise); the
  Boundary's no-authorization-semantics bullet and the delivery
  status's closing sentence move from gate to standing decision;
  `schemas/rpc/authentication.md` §3 does the same, with the closure
  test's exhaustive-match pin now guarding a standing rule rather than
  a wait; README's WP-040 row closing sentence moves with the bytes it
  describes; both documents' spec-version pins move to 11.2.0. The
  increments' delivery rows keep their "while SI-18 holds" phrasing as
  history of what was delivered under the gate, said so in the closing
  sentence rather than silently reinterpreted. **Deliberately not
  touched: the three `crates/rpc` doc-comment references**
  (`lib.rs`, `identity.rs`, `identity_tests.rs`) — any non-Markdown
  merge re-opens the three VM acceptances pinned at `39b59f5`, a
  sitting no comment edit justifies, so the comment re-attribution
  rides the next Rust change to the crate and is recorded here rather
  than left to be discovered. No type, test, schema byte, or normative
  text changes; the vocabulary stays pinned to exactly the three
  identity claims.

- **spec-change 11.2.0: SI-18 is resolved by ADR-0021 — authorization is
  a two-tier ladder, and SAFE-002 is untouched.** Every apply of every
  plan, at every severity including 0, requires a floor authorization: a
  fresh, explicit act by the RPC-001-authenticated user naming the exact
  plan hash, single-use, valid only inside the plan's PLAN-007 window,
  journaled, never cached, session-wide, or remembered, and satisfiable
  programmatically — which keeps SAFE-003's unattended/scripted-apply
  population a live surface. The interactive OS-mediated ceremony
  HLP-003 already required at severity ≥ Disruptive stands verbatim and
  additionally binds any plan carrying a step flag — the
  severity-plus-flags participation PLAN-004 promised and HLP-003 never
  stated; the concrete gap was a LUKS keyslot addition, fully reversible
  (severity 1) yet `security-sensitive`. A flagged plan can never be
  applied unattended. The enforced tier derives from the helper's own
  recomputed severity and flags (HLP-002), never from client claims, and
  no authorization-requirement field enters the plan — the register's
  named question answered: it is a total function of body content
  already present, so WP-040's authorization vocabulary unlocks with no
  jointly-sequenced WP-010 schema change and the authentication skeleton
  stays identity-only. Rejected and recorded: reading SAFE-002 through
  HLP-003's silence (inverts §0.2, the SI-38 shape), the ceremony
  everywhere (rubber-stamps the ceremony where it carries load and
  forecloses a population SAFE-003 contemplates), and a helper-authored
  plan-carried field. Minor under §0.1: both pre-existing HLP-003
  sentences stand verbatim and nothing narrows. Accepted by Nate McBride
  2026-08-11 by delegation, recorded as the ADR's acceptance basis; the
  verification obligations land with WP-070 and are enumerated in the
  ADR so none is discovered late.

- **SI-40 is resolved by ADR-0020, with no spec change — deliberately.**
  Reading (a) of the filing's options, decided by the decision owner
  the same day the filing landed: FS-007's "blocked reasons" is the
  generic noun phrase for the capability reason vocabulary, and an
  immutable technology limit's status follows CAP-003's definitions —
  `unsupported`, carrying `Reason::TechnologyLimit` as its explicit
  reason and `Remediation::NoneExists` as an exact statement rather
  than a lazy one. The ADR records the deciding safety property:
  `blocked` keeps meaning remediable, so a permanent impossibility
  never invites remediation of the unremediable — and records why the
  absent spec change is deliberate: the decision selects between two
  readings of existing text and amends neither. Readings (b) (the
  literal `blocked` status, retexting CAP-003's definitions) and (c)
  (a widened status vocabulary, SI-26's territory) are recorded with
  their costs. WP-050 increment 2's technology-limit composition is
  unblocked; nothing else waited on the decision.

- **WP-035: the chassis's in-band gate references stop citing resolved
  register items as open (#215).** The product-byte half of the #197
  re-attribution, discharged on the principle the `GATED` list's own
  ADR-0011 precedent states: citing the retired question would be the
  drift the register's sole-authority rule forbids. The standing gated
  list's `partition-table-state` entry moves to `helper-authored`
  (ADR-0014) beside `never-inferred` (ADR-0011) — the client never
  computes table state, a standing rule now rather than an open
  question — while `identity-strength` keeps `not-established`
  (SI-28), the one citation still open. The `inventory` and `topology`
  refusals move from `not-established` over "SI-27, SI-28, SI-35" and
  "SI-27, SI-28, SI-34, SI-35 … remain open" to `not-implemented`
  naming what actually holds each surface out: ADR-0019's landed
  naming types unconsumed by this chassis, ADR-0014's helper-sole-
  author rule, ADR-0016's verdict placement, and SI-28's open
  attribution question. Every pinned literal moved with its surface
  (human fragments, the ordered JSON gate contract, the typed-refusal
  cases), the module doc comments carry the same re-attribution, and
  README's in-band-list sentence and WP-035 status-row sentences move
  with the bytes they describe. **MODEL-003 assessment, as #215
  requested:** every changed value rides `partman.cli.envelope/0`,
  documented in `apps/cli/src/lib.rs` as provisional and free to
  change until CLI-001's stable schema regime exists — the change is
  value-level within that regime, there is no per-payload version to
  bump, and this entry is the documentation the provisional regime
  requires. The refusal posture, exit codes, envelope schema string,
  and state vocabulary (both states already existed) are unchanged.

- **Register: the Part 1 framing block that still described SI-34's
  placement as open is marked as dated history (#198).** The bounded
  integrity pass the 2026-08-10 grant authorizes, executed exactly to
  its reach: a dated-history banner now prefaces the five paragraphs
  from "Half the approach is settled; the other half is reopened."
  through the measured-second-instance paragraph, recording that SI-35
  resolved in spec 8.0.0 by ADR-0014 and SI-34 in spec 9.0.0 by
  ADR-0016 — the placement question closed by the architecture the
  SI-35 resolution built, so the block's closing advice is addressed
  to a decision that no longer exists — and directing current status
  to the authoritative table and the issues' own entries. The
  paragraphs stay verbatim as history; no state, class, dependency,
  option, evidence record, or normative text moves. The issue's
  requested sweep of Part 1's remaining framing prose found nothing
  else superseded by the 7.0.0–11.1.0 resolutions: the status prose
  and table are current through 11.1.0, the SI-31 and SI-28 paragraphs
  state their resolved and Mitigated-open postures accurately, and
  "Read SI-28 first" stands — consistent with the reclassification
  record, which moved SI-28's class, not its openness.

- **WP-035: the gated-surface list stops citing resolved register
  items as open gates (#197).** The SI-12 re-attribution shape applied
  to five Boundary entries: the protection-verdict entry moves from
  "SI-11 (inputs SI-29, SI-30, SI-37)" to ADR-0018 (SI-29/SI-30
  resolved within it, SI-37 reclassified open at Later); the
  node/snapshot/hash/plan entry from SI-27/SI-34/SI-35 to ADRs
  0019/0016/0014 plus the unconsumed `crates/domain` increment-3
  types; the stable-handle entry from SI-27 to ADR-0019; the
  table-state entry from SI-35 to ADR-0014's standing helper-sole-
  author rule; and the `IdentityStrength` entry narrows to SI-28
  alone, the one citation that is still an open register item. Every
  prohibition is unchanged — each entry's authority moved from an open
  question to an accepted decision plus types this chassis does not
  consume. Docs-only: the chassis's own in-band gate references and
  the README rows that describe them still carry the old citations and
  are deliberately untouched here — they are product bytes pinned by
  tests, filed as their own follow-up rather than swept silently into
  a docs pass.

- **Register: SI-28 reclassified off the increment-3 gate; nothing
  gates increment 3.** Decided by the decision owner 2026-08-09, the
  SI-37 pattern applied to the register's last direct blocker, under
  the WP-010 grant landed the same day. SI-28 stays Mitigated-open —
  not Resolved, Part 7's warning in full force — its interim
  conservative floor unchanged and its relaxation route staying
  ADR-0017's named revisit condition. Only the class moves: the floor
  is computable from decided, contract-readable facts (transport
  class, removability, identifier presence), so no undecided hashed
  field feeds it, and the refused population can hold no issued
  authorization for a later discriminating mechanism to invalidate.
  The priced cost is stated in the entry's banner: a future mechanism
  adding an identity-record field pays a MODEL-003 schema major after
  implementation exists, accepted because the alternative was gating
  the domain model on a mechanism nobody can measure. WP-010's stage
  line now reads unblocked; increment 3 may start.

- **spec-change 11.1.0: SI-27 is resolved by ADR-0019, on its round
  four.** Node identifiers are derived, kind-discriminated positional
  addresses — the surviving decomposition kept: an address, never a
  device identity — computed from fields ADR-0018's evidence contract
  reads, canonicalized by the contract's one named source per platform
  verbatim, recomputed at every decode by the schema-validation pass,
  which rejects unknown referents. Equal derived addresses collapse
  before encoding into counted, flagged, indeterminate collision
  groups whose operands are blocked pairwise — the representation of
  the ambiguity ADR-0011/SAFE-005 already declare, preserving two-ness
  and never silent, with the whole-host unencodability failure the
  register's governing finding condemns impossible by construction.
  The ancestor-only address property is a committed property test;
  nothing re-designates on a duplicate-designator clone. The four
  collision families each get a mechanism: the platform-membership
  edge (typed; path-set encoding untouched, deferred per ADR-0011),
  BackingExtent with the host-backing edge (closing CONC-001's empty
  loop-device bind set and round three's own-fixtures-collide defect),
  offset-qualified signature addresses (the stale pair is the
  committed two-address regression), and role-discriminated table
  views with partitions re-parented onto the table and verbatim
  conflicting-entry evidence scoped by ADR-0018's closure. The
  preserved-unknown budgets are fixed and the redaction rule
  versioned. Minor: Section 5 and MODEL-002 gain additions, LIN-006's
  deferred-edge-kind clause gains its promised pointer, no existing
  claim narrows. The gate on increment 3 drops to one item: SI-28.

- **spec-change 11.0.0: SI-11 is resolved by ADR-0018, on the fourth
  round.** The protection closure is computed, total, and fail-closed.
  Per-node verdicts are three-valued with an `Indeterminate` residual —
  never `Permitted` by default, round three's fail-open arm inverted
  and property-tested — computed from a named two-layer helper evidence
  contract: the helper's own bounded, enumerating, fuzz-obligated
  parsers over raw device bytes (ADR-0014's architecture generalized
  from the table to every on-disk verdict input), named per-platform
  state APIs for the rest, and a protective join. That discharges the
  named-contract hard input ADR-0016 transferred to this round. A
  mutating step's affected set closes over destroyed substrate —
  downward containment bounded by the destroyed ranges, upward backing,
  downward production — with release counted as destruction, so the
  recorded root-on-ZFS-over-LUKS destruction path refuses while the
  no-sibling-capture theorem is a committed property test and creating
  a partition beside a pool member constructs. Device scope inverts to
  a closed positive local-transport list; capability status is computed
  from canonical steps by the same closure, so CAP-005 agreement holds
  by construction; source classes are never suppressed; PART-014
  classification is exhaustive, Regime B, and outside the body. A
  closed three-entry acknowledgment vocabulary — release,
  opaque-destruction, identity-bound-restore — replaces both silent
  permission and forever-refusal, the consumed-member case deliberately
  unrepresentable. SI-29 resolves within the decision (the narrow
  boundary: file systems inside a Storage Space are ordinary targets
  within the provisioned block interface, health-gated — the narrowing
  that makes this major); SI-30 resolves within it
  (deletion-by-containing-erase severed from sealed-object
  modification, routed via MAC-009 and the documented-paths clause, an
  empty-in-v1 step family); SI-37 is reclassified — open, off the
  increment-3 gate, its dual-path matrix now relaxation evidence. The
  locked-container residual is stated in the resolution banner rather
  than rounded away, and the write-path demonstrations join the
  SI-33/SI-34/SI-35 obligations on the first write-capable increment.
  The gate on increment 3 drops to two items, both direct: SI-27,
  SI-28.

- **spec-change 10.0.0: SI-33 is resolved by ADR-0017.** The continuity
  witness exists and is a refusal input, never an assurance: an
  epoch-token/counter field of SAFE-003's identity record —
  client-readable and helper-verified like a serial, deliberately not a
  MODEL-005 authoring-set entry, the set staying closed at two — scoped
  to exchange-capable targets on qualified apparatus, one qualified
  today. The semantics are the measurements': comparable only within an
  unchanged epoch token and never on a decrease (a reset the token
  failed to witness, the adversarial round's finding), movement or
  incomparability rejecting covered targets under the existing
  identity-change rule, and `no-exchange-observed` — the liveness
  ceiling's own words, the vocabulary's strongest — relaxing nothing
  anywhere, so staleness on unmeasured hardware costs exactly the
  assurance that was never claimed. The S4-measured undetectable vector
  — swap between plan and apply on media whose every identifier is
  identical — becomes a refusal where the apparatus is qualified.
  SI-28's floor and Mitigated-open state are untouched; the relaxation
  route is ADR-0017's named revisit condition. SI-33 becomes
  hash-visible through the placement this resolution decided, and its
  row is corrected. Major, because an existing requirement's record
  contents change. The gate on increment 3 drops to six items, three
  direct: SI-11, SI-27, SI-28.

- **spec-change 9.0.0: SI-34 is resolved by ADR-0016.** The derived
  protection verdict is hashed-body content, helper-authored at
  validation — ADR-0014's architecture applied to the second and last
  field only the helper derives, in the adversarial round the register
  recorded that option (c) never had. Major, because it changes what
  8.0.0's closed authoring-set sentence claims: the set holds exactly
  two named entries and stays closed to creep. The filed options all
  bridged a two-observer world 8.0.0 removed — (a)'s clamp blinded the
  helper for an agreement no longer needed, (b) un-authenticated the
  value the user most needs bound, and (c)'s
  freshness-projection-plus-floor machinery dissolves with the second
  author, its two open dependencies (projection membership, the
  monotonicity proof) having been costs of bridging authors. What
  survives of (c) is its point, by construction: a client cannot weaken
  the safety decision, because no client claim is representable in a
  bindable artifact. Within-target divergence between stamp and
  recomputation rejects under existing SAFE-003/PLAN-006 rules; the
  journaled-continue relaxation is foreseen, not foreclosed. The round's
  sharpest finding transfers to SI-11 as a hard input — the verdict
  binds to a named, deterministic helper evidence contract with measured
  re-probe stability, the intra-helper wipefs/blkid asymmetry making an
  unnamed set round two's refuted premise returned. Write-path
  demonstrations are named obligations on the first write-capable
  increment, in the resolution banner beside SI-35's. The entry's stale
  M10-not-taken currency is corrected in the same change. The register's
  gate on increment 3 drops to seven items, four direct: SI-11, SI-27,
  SI-28, SI-33.

- **spec-change 8.0.0: SI-35 is resolved.** The chain the 2026-08-09
  resolution round accepted closes with the normative instrument: the
  four amendments ADR-0014's Consequences enumerated before any was
  drafted (PART-001's categorical helper invariant — the major;
  MODEL-005's named authoring-at-validation verb, closed to the one
  field only the helper derives; Section 6 binding the
  validation-produced snapshot hash; INV-003 stating the
  client-emits-no-table-state consequence in terms), plus the
  `Present {checksum}` basis open since round one, fixed over
  copy-invariant content in `schemas/table-checksum.md`. The refusal
  demonstration is discharged at its honest scope — classification of
  both decisive fixtures, mutation-verified, with claimed-never-`Absent`
  a searched fuzz property — and the end-to-end write-path
  re-demonstration is a named obligation on the first write-capable
  increment, recorded in SI-35's resolution banner rather than a review
  memory. The register's gate on increment 3 drops to eight items, five
  direct: SI-11, SI-27, SI-28, SI-33, SI-34. Chain: #185 governance,
  #186 fixture, #187 parser, #188 fuzz target, #189 registration, this.

- **The table parser lands: ADR-0014's contract becomes code, and the
  SI-35 refusal demonstration's classification half runs at Tier 1.**
  `crates/table-parser` (reserved by the 2026-08-09 governance change) is
  the pure, bounded, `unsafe`-free classifier of caller-supplied head and
  tail windows — the exact shape M10 measured as separating — into
  ADR-C3's three states: GPT both-copies parsing with header and
  entry-array CRC validation, MBR protective/hybrid/standalone reading,
  APM recognition, no I/O, no process, no Section 5 type. The accepted
  classification table holds against every catalogue fixture, in tests
  that build the images in memory from source: the decisive
  `gpt-conflicting-tables-512` classifies `Indeterminate` on the
  ambiguous arm (mutation-verified: a parser that crowns the primary
  fails the test by name), `gpt-both-copies-invalid-512` on the
  unreadable arm (mutation-verified against the absent-collapse),
  one-valid-authority shapes are `Present` with their condition per the
  fixtures' own recorded claims, and every signature-only medium is
  `Absent` — which says nothing about data. The `Present` checksum is
  SHA-256 over copy-invariant content, proven copy-invariant by the
  fixture pair that shares one table across different carrying copies;
  probing a 4Kn medium under a 512-byte contract answers
  `Indeterminate`, reproducing the measured libblkid trap honestly. The
  state type carries no proceed-enabling reading, pinned by source scan.
  Its Section 11.4 fuzz target follows in this chain's next change,
  searching the load-bearing line: a claimed table never classifies as
  `Absent`.

- **ADR-0014 fixes SI-35's axis: the helper is the sole author of
  partition-table state.** Drafted and accepted 2026-08-08 from two
  adversarially reviewed rounds the same day — the ADR-C4 guard fork
  (decided separately: the guard is a priced permission, four conditions,
  recorded inside the ADR) and the axis round itself. Two measured facts
  drove it: nothing separates the decisive GPT pair except raw sector
  bytes — every client projection on three platforms failed, and so did
  the privileged `blkid`/`wipefs` probes — and ADR-C3's `Present` means
  "read and hashed," which no denied-raw-read client can produce. So the
  helper computes all three states from its own raw-sector parser (a
  Section 11.4 fuzz-obligated parser, landing later), the client emits no
  table state on any platform — which also resolves INV-003's `Present`
  face for the client, parked at SI-35 by SI-39's filing — and the state
  lives in the hashed body of helper-produced artifacts, stamped at
  validation, where the flow already puts the helper before the user's
  hash-bound authorization. ADR-C4's guard is satisfied unamended; the
  fork's priced permission goes unused. The sweep the axis round demanded
  found the structural confirmation: PLAN-006's body-hash equality plus
  Section 6's bound source-snapshot hash require the authorized plan to
  bind a validation-produced snapshot, so client views are proposals and
  the user authorizes what the helper established. **SI-35 stays Open,
  axis decided** (the ADR-0012 shape): resolution waits on the parser and
  its refusal demonstration on `gpt-conflicting-tables-512`. No spec text
  changes with this ADR — the amendments it necessitates (PART-001's
  categorical invariant, ADR-C2's authoring verb, Section 6's
  bound-at-validation wording, the client prohibition) are enumerated in
  its Consequences and land with the resolution round. The register's
  direct-blocker count is unchanged.

- **spec-change 7.0.0: SI-39 is resolved by ADR-0015.** SAFE-003's
  blank-can-be-Strong derivation is scoped to the observing contract.
  Drafted 2026-08-08 from the same day's adversarially reviewed
  recommendation round; **acceptance is recorded in the ADR itself and
  this entry lands only with it** — an unaccepted draft of this entry on a
  branch is a proposal, not a change.

  The conflict was measured: INV-003 (6.0.0) forbids the client reporting
  a medium as positively without a table where its contract does not
  separate that case; the macOS matrix measured `blank-512` as
  byte-identical to four occupied media; so no macOS client-derived blank
  record is positively determined, while SAFE-003 said such a device can
  be Strong. The repair is scoped to the one false sentence: **the
  strength rule is untouched**, Strong keeps one invariant meaning
  everywhere, and only the attainable population varies by contract —
  which is what rejected reach-relative strength (option (a)). Rejected
  with it, and recorded in the ADR: reportable-`Absent` under caveat (the
  recorded data-loss path — PART-001 initializes blank media), a split
  client/helper strength vocabulary, and a hoped-for separating interface
  (kept as a self-executing revisit condition, since the contract-relative
  wording restores client-side Strong without amendment if one is ever
  measured).

  The accepted consequence, stated rather than smoothed: on macOS, blank
  media carry Weak identity at plan time — PART-001 initialization takes
  typed device-name confirmation, an immediate pre-apply re-probe (M10
  measured the re-probe's observer as the one that separates), and no
  unattended apply without the recorded override. The plan's claim on
  such media is "initialize this device, which the client could not
  distinguish from occupied," never "this medium is blank." **Major under
  §0.1** — it narrows an existing requirement's claim, the class 3.1.0
  mis-numbered. SI-39 moves to Resolved, the register's gate on
  increment 3 drops to nine items (six direct), and SI-35's `Present`
  face stays deliberately untouched.

- **WP-035 increment 9 delivers the macOS enumeration adapter** (2026-08-08),
  on the decided bounded-reader route: no dependency taken, the
  empty-dependency-closure guard intact, and a hand-written XML plist reader
  whose grammar is exactly what the measured `diskutil` captures use — data,
  date, real, comments, CDATA, numeric character references, undefined
  entities, DOCTYPE internal subsets, duplicate keys, non-UTF-8 bytes,
  over-depth, and oversize values all refuse the whole input with typed
  errors rather than substituting or truncating. The adapter launches
  `diskutil list -plist` once and `info -plist` once per whole device, at
  the compiled absolute path through the launcher seam, which gains an
  argument-bearing method with caller-stated per-stream output bounds (the
  doctor's version probe keeps its own 4096-byte bound; the enumeration
  states 4 MiB for the list and 64 KiB per info); a source-pinned guard
  holds the macOS adapter as that channel's only shipped caller. Twelve
  identity keys report as raw interface-labelled strings — `Content`, UUID,
  and APFS fields deliberately unread, because the scheme name is SI-35's
  material and increment 7's adversarial round already refused cells built
  on it — with a missing key a positively determined absence, a
  present-but-container value a typed failure never flattened, a nonzero
  diskutil exit a failure whose output is never parsed, and a `WholeDisks`
  name outside disk-then-digits refused before it reaches argv. The macOS
  reach declaration moves to implemented-reaches-no-table-state with every
  cell still negative, the same shape Linux took in increment 8. The
  enumeration is threaded through the injected launcher from dispatch, so
  no Tier-1 test can launch a real diskutil, and the adapter module
  compiles on every platform so its tests run on all three CI legs. The
  reader is a parser of externally supplied bytes: its Section 11.4 fuzz
  target lands as its own chain — the fuzz crate and `fuzzing.md` are
  WP-010's and the xtask target list is WP-000's — and is **in flight, not
  silently absent**; this entry is the record that says so.

- **WP-035 increment 10 is closed as deferred, by the recorded route
  decision its grant required** (2026-08-08). The three named routes each
  carried a recorded cost — WMI/CIM needs FFI the crate cannot host and a
  separate crate would break the empty-closure guard; PowerShell adds a
  shell to the SAFE-004 roster and still needs a JSON reader; deferral is
  the shape spec 6.1.0 built the M0.5 gate to accept — and deferral is
  chosen, per the prior analysis and the 2026-08-08 briefs, on Nate
  McBride's direction to proceed with the briefs' recommendations. The
  Windows `inspect` answer now names the recorded decision in-band beside
  WP-W100 (a `deferral` field in JSON, a line in the human answer), the
  Windows reach reference names the decision instead of a pending
  increment, and a Tier-1 test holds the two surfaces to the same story on
  every platform shape. WP-W100's Section 14 row is untouched; M0.5's exit
  is unaffected. The decision record, its costs, and its revisit conditions
  are in `docs/work-packages/WP-035.md`.

- **WP-020's increment 2f row catches up to 2026-08-03.** The delivery-status
  row still said "not yet exercised on a real kernel" five days after the
  SI-35 instrument's three sittings ran `run_probed_session` thirty times in
  the disposable VMs — the exact first exercise the row itself predicted. The
  row now records that fact at its correct strength: the two void sittings'
  defects all lived in the instrument's unprivileged projection half, none in
  the session protocol, whose capture half completed cleanly in all three
  sittings; and thirty clean sessions under the SI-35 protocol's gates are
  that record's evidence, not a registered 2f acceptance, of which none
  exists and none is claimed. The "weaker than 2e by construction" boundary
  is untouched. The historical 2f delivery entry below correctly said "no
  real loop device ran" at its writing and stays as written.

- **The macOS second-reader readback is discharged** (2026-08-08). An
  independent reader session — not the session that produced any of the three
  records — retrieved both `partman-macos-sitting-2` transcripts and the M10
  transcript through their evidence-store locators and rehashed each to its
  recorded digest, all three matching; sitting 2's byte lengths matched as
  recorded, and the M10 capture's full 172-entry SHA-256 inventory was
  additionally rehashed with every entry matching. Each record's custody
  caveat travels into the discharge rather than being erased by it: sitting
  2's digests were computed after retention from an unmodified copy, so its
  matching rehash confirms the copy unchanged since 2026-08-05 and nothing
  stronger, while M10's digest was recorded at retention and its rehash
  carries the full property. The readback also surfaced and recorded an
  omission of sitting 2's class in the M10 record — a digest with no byte
  length, where the custody rule requires both; 23 516 bytes is now recorded
  as a readback-time measurement, stated as such. This discharge removes the
  custody caveat the SI-34 currency note and SI-39's dependency paragraph
  carry; it resolves neither issue and decides nothing on the register.

- **WP-035 increments 7 and 8 delivered** (2026-08-05; recorded here 2026-08-08
  — these entries should have landed with their pull requests, and their
  absence was found by the same record sweep that fixed the README's stale
  "observes no real device yet" sentence). Increment 7 publishes the INV-003
  reach declaration on all three platforms, for the contract this package
  itself reads and nothing wider: one answer per state INV-003 lists, derived
  from the contract rather than from any device, every negative present
  rather than omitted. An earlier draft that shipped measured cells for
  interfaces the increment does not read was refused by adversarial review on
  five recorded grounds; the measured tables move to the increments that make
  them true. The table is built by one const fn with exactly one place a
  positive could be written, and a mutation-verified test fails if it is.
  Increment 8 delivers the Linux enumeration adapter and wires it into
  `inspect`: whole devices through sysfs file reads with no subprocess,
  reporting size, block sizes, vendor, model, serial, and WWN as raw
  identifier strings labelled by the interface that reported them, udev
  values carrying an in-band caveat that they are what root's `udevd` cached
  at device-add time. Adversarial review fixed three doc-versus-code
  disagreements before merge — a documented refusal implemented as silent
  truncation, a trim that manufactured positively-determined absences from
  padding, and a partition filter that failed open — and the wiring commit
  repaired its own predecessor's overclaim, `enumerate` having been called
  from nothing but tests when the commit title said the adapter read real
  devices. macOS and Windows still answer with typed not-implemented
  statements; increment 9 is macOS, and increment 10 opens only after a
  recorded choice among its three named routes.

- WP-010 files **SI-39**, the SAFE-003 / INV-003 conflict, under Section 0.2's
  requirement to file rather than silently pick a side. SAFE-003 says "A blank
  device can therefore be Strong"; INV-003, as ADR-0013 amended it in spec
  6.0.0, forbids the unprivileged layer reporting "a medium as positively
  without a table" where its contract does not separate that case; and the
  macOS increment 6 matrix measured that non-separation directly — `blank-512`
  and media carrying ext4, an mdraid member, LUKS2 and LVM2 all project
  byte-identically. So on macOS a client may not report `Absent`, the state is
  not positively determined, and the device is Weak where SAFE-003 says it can
  be Strong.

  **The filing records that this repository created the conflict**, hours
  before finding it: INV-003's governing sentence is ADR-0013's, and that ADR's
  adversarial round did not reach SAFE-003. A register that files a conflict as
  discovered when it was introduced misleads the next round about where to look.

  Classified a direct blocker; the authoritative count moves to ten, seven of
  them direct. Four resolution options are recorded and none is recommended,
  including the one with a recorded data-loss path (amending INV-003 so a
  medium indistinguishable from blank is reportable as `Absent`, when PART-001
  initializes blank media). The `Present` face of the same INV-003 sentence
  reaches all three platforms and overlaps SI-35's open axis question; it is
  recorded as adjacent and **deliberately not decided**. The macOS rows carry an
  outstanding second-reader readback, and the filing says so. No requirement is
  amended and no specification version changes.

- **spec-change 6.1.0: WP-035 gains unprivileged whole-device enumeration and
  the INV-003 reach declaration.** The read-only CLI may now report real
  attached devices — raw identifier strings labelled by the interface that
  reported them, under session-local selectors — so the project has a working
  read-only alpha while the register decisions proceed in parallel.

  **Minor, an addition.** No requirement in Sections 2, 3, 5, 6 or 7 is
  retexted. INV-003 is *implemented in part*, not amended: 6.0.0 created the
  reach obligation hours earlier and nothing implemented it. WP-035's charter
  sentence survives verbatim and governs the new scope in full, and the change
  adds prohibitions rather than relaxing any — no strength, no ADR-C3 state or
  checksum, no typed Section 5 node, no artifact hash, no stable handle, no
  same-device claim, no protection or CAP-003 verdict, and the standing gated
  list still travels in every answer.

  **WP-W100/WP-L100/WP-M100 are untouched.** Narrowing them would remove scope
  from existing text and would be a major bump; this enumerator is interim and
  defers to them.

  M0.5's gate is **extended, not rewritten**, in the shape 4.2.0 used when it
  created M0.5. It deliberately does not require three live adapters: a
  platform whose access route is an open structural question ships its reach
  declaration and a typed `not-implemented` answer naming the recorded
  decision that defers it. Coupling M0.5's exit to that question would gate
  every sequential milestone after it on a structural argument this change
  exists to avoid importing.

- **SI-33's liveness precondition is recorded as discharged** (decided
  2026-08-05). Its filing said "until that passes on real hardware, this is a
  hypothesis"; the 2026-08-04 sittings passed it — immediate re-read and
  sixty-second idle gap both moved, close-before-event/reopen survived 3/3
  across true no-handle windows. **SI-33 stays Open**: what the pass
  discharges is the precondition, not the issue.

  The entry records three limits the protocol declared before any data
  existed, because the headline sentence would otherwise be read as removing
  them: the positive cannot be attributed to exchange-synchronous detection (a
  background poll explains it equally well, so the ceiling is "no staleness
  observed under these conditions"); it is bounded to the slot-exchange family
  on one reader, one bridge, one build; and the exposed reading is **not
  globally monotone**, a measured decrease across a PnP-arrival boundary making
  an equality-only witness unsafe, so a design must characterize the counter's
  epoch or use another witness. No axis, design, or placement is decided, and
  **SI-28's interim conservative floor is not relaxed** — that route is the
  design, not the liveness pass.

  Two stale statements are cleared in the same pass: Part 6's precondition 1
  still described M10 as untaken hours after it was taken, and SI-38's
  Dependencies paragraph still asserted a gate on SI-35 directly beneath its
  own Resolved banner. Both are now historical rather than contradictory, and
  the M10 sentence carries the outstanding second-reader readback.

- **spec-change 6.0.0: SI-38 is resolved by ADR-0013.** INV-003's detection
  duty is scoped by privilege, and the unprivileged discovery layer must
  publish the reach of its platform contract. Accepted by Nate McBride on
  2026-08-05 after an adversarial round that changed the recommendation.

  The conflict was real and measured: INV-003 required the discovery layer to
  detect hybrid and inconsistent partition tables, SAFE-002 places that layer
  at no elevation, and the enumerated client projections on Linux
  (2026-08-03), Windows (2026-08-04) and macOS (2026-08-05) all fail to
  separate a healthy GPT from one whose two tables describe different
  partitions. M10 (2026-08-05) located the separating fact in the backup
  table, behind a read the unprivileged client is denied on the same
  attachment.

  **Major under §0.1** because it narrows an existing MUST rather than adding
  one. The full detection set survives on the privileged path; the
  unprivileged layer is no longer required to do what it measurably cannot,
  and is instead required to say so.

  Two rejections are recorded rather than omitted. Reporting the remainder as
  undetermined is **unimplementable** — the client cannot identify the
  remainder, a conflicting table presenting as an ordinary valid GPT, so the
  rule would either never fire or mark every GPT undetermined; this was the
  first recommendation and review killed it. Qualifying **SAFE-002** was
  rejected on precedence: a Section 3 constraint may not be bent to satisfy a
  Section 7 functional requirement without inverting Section 0.2's ordering.

  Where the reach does not cover a state, the privileged re-discovery HLP-002
  already requires before the first write determines it; the unprivileged
  layer neither refuses on the ground of its own blindness nor represents that
  blindness as a determination. SAFE-005 is unchanged. SI-38 moves to
  Resolved, the register's count returns to nine, the transitive-blocker class
  returns to empty, and **SI-35 is unblocked and remains a direct blocker,
  undecided**.

- **M10, the privileged comparison leg, is taken** (2026-08-05), and with it
  **no preregistered cell on any platform is `not yet taken`**. It ran in a
  GitHub-hosted `macos-15` runner — `RELEASE_ARM64_VMAPPLE`, macOS 15.7.7, an
  ephemeral Apple Virtualization Framework guest destroyed at job end, which
  the cell's environment rule admits as a hosted macOS test environment. The
  harness captures both halves of every attachment itself, because this is a
  different machine from the M1–M8 sitting and M10 asks about the *same*
  attachment.

  On all seven fixtures the unprivileged client's raw read was denied `EACCES`
  while root read the device, and every helper byte-range digest equals the
  source image's. **The decisive pair separates for the helper**: identical
  first-64-KiB digests and differing last-64-KiB digests, placing the two
  tables' disagreement in the backup, which no client interface on any
  measured platform reports — and this sitting's own client half reproduces
  that blindness. The four signatures the client called byte-identical to
  blank each carry a distinct helper head digest.

  Two incidental observations are recorded because they refine the earlier
  sitting: the denial is `EACCES` here where the physical host gave the
  unexplained `EPERM`, making that a contrast rather than a lone oddity
  without attributing a cause; and the node modes differ. The client half also
  reproduced the decisive-pair byte-identity on a different macOS major
  version, weakening the earlier record's "one host, one build" limitation for
  that finding only. Limitations recorded: shared-infrastructure runner, no
  checksum-pinned image, first and last 64 KiB only, `dd` through
  `/dev/rdiskN` and no other privileged interface. The second-reader readback
  is outstanding. No register disposition changes, no SI-34 or SI-35 option is
  decided, and SI-34's freshness-projection element is **not** satisfied — it
  names a projection that does not yet exist.

- WP-010's third evidence-currency pass records the macOS observability record
  on the two register surfaces that understated it (issue #155). Precondition
  1 now says all three platforms have the non-elevated record it asks for, and
  SI-34's evidence clause gains a bounded status note: its **observability
  element is satisfied; every other element remains unsatisfied**. The note
  states the reading it rests on — precondition 1 defines the record as
  "established empirically and non-elevated", so macOS's untaken privileged
  leg does not hold that element open — and records the narrower reading as
  **rejected rather than ignored**, with what changes if it is preferred. It
  also states what is missing on macOS under either reading: M10 is untaken,
  so there is no privileged comparison leg, and SI-34's separate both-views
  freshness requirement is unmeasured there and unmeasurable until M10 exists.
  SI-34 is not resolved, its evidence clause is not discharged, no option is
  decided or ranked, and no state, class, dependency, or metadata changes.

- The increment 6 **macOS matrix is executed** (2026-08-05, Apple Silicon,
  console session, macOS 26.3.2 build 25D2140, SIP enabled), valid on its
  second sitting. Sitting 1 is **void on two harness defects** and is retained
  with the amendments it produced: tool versions were captured through
  `diskutil version` and `hdiutil version`, neither of which is a real verb,
  so no version was recorded — now replaced by a SHA-256 over every declared
  binary; and the post phase ran without the reboot M5 depends on, detected
  only afterwards by disk-numbering inference — now a hard `kern.boottime`
  gate that voids M5 in-transcript, tested by forcing each outcome before the
  amended harness shipped. The favourable reading of defect 1, that OS-bundled
  tools have no version beyond the recorded OS build, is recorded as
  considered and rejected.

  Substantive results: **macOS is the third platform whose enumerated
  unprivileged projection does not separate the decisive SI-35 pair** —
  `diskutil`'s structured output is byte-identical and unnormalized between a
  healthy GPT and one whose two tables describe different partitions. **Every
  non-native signature projects byte-identically to a blank disk** (live ext4
  with a stale mdraid superblock, an mdraid member, a LUKS2 container, an LVM2
  orphan), while GPT, MBR and APM are each named distinctly. APFS container
  membership and its UUID are client-readable, the UUID is carried by both
  interfaces and is stable across a verified reboot, and the unprivileged raw
  device read is denied `EPERM` — recorded as observed and unexplained, since
  `EACCES` was the expected errno for a user outside the owning group. M9 is
  `not established` (Apple Silicon has no Fusion Drive) and **M10, the
  privileged comparison leg, stays `not yet taken`** for want of a disposable
  macOS VM. No separation claim is made from the `ioreg` interface, whose
  capture is a whole-registry dump with no normalizer declared before it. The
  second-reader readback is outstanding. No register disposition changes, no
  SI-35 option is decided, and no existential hypothesis is refuted.

- WP-010 files **SI-38**, the INV-003 / SAFE-002 conflict, under Section 0.2's
  requirement to file rather than silently pick a side. INV-003 requires the
  discovery layer to detect hybrid and inconsistent partition tables; SAFE-002
  places that layer at no elevation; and the 2026-08-03 Linux and 2026-08-04
  Windows runs establish that the unprivileged projection distinguishes
  neither. HLP-002's independent re-discovery does not dissolve it, being
  scoped "before the first write" — plan time, not inventory time — so at
  inventory the unprivileged layer is the sole observer. Classified a
  transitive blocker to SI-35 rather than an input, because an ADR may not
  amend a MUST and every available resolution is a normative amendment; the
  register's count moves to ten and the previously empty transitive class is
  repopulated. Four resolution options are recorded and none is recommended.
  No requirement is amended, no specification version changes, and SI-35 is
  neither decided nor pre-empted.
- WP-010's bounded evidence-currency pass stops SI-35's register row
  understating evidence WP-035 has already established (issue #142). Two of
  its three acceptance-evidence categories are now recorded as discharged —
  the loop category by the descriptor-bound non-WSL run of 2026-08-03, valid
  on its third sitting, and the Windows category by the completion rerun of
  2026-08-04 — and the third, the chosen-option refusal demonstration, is
  recorded as blocked on a decision nobody has taken rather than on a
  measurement anyone can schedule. The round-four observability-status
  sentence now names macOS as the only platform still unestablished. SI-35
  stays **Open**; no option is decided or ranked, no existential hypothesis
  becomes refuted, no finding extends past the projection its run covered,
  and no class, dependency, hash-visible value, requirement, or evidence
  record changes.
- The increment 6 real-partitioned-Linux matrix is executed in full
  (sitting of 2026-08-04): disposable Proxmox VM from the digest-verified
  jammy image, two explicitly authorized SanDisk fixture media passed
  through, six declared layouts provisioned by the separated setup actor
  with per-layout digest brackets, and rows L1–L10 all `observed`. The
  substantive results: the ordinary-client baseline truly lacks raw block
  access while `disk`-group membership alone flips it; the client/helper
  signature asymmetry is measured in both directions — an empty cached
  client projection over live ZFS labels (event-time-cache mechanism
  recorded), and the live-plus-stale L-F pair answered with exactly the
  stale signature by both single-answer interfaces while only the
  enumerating probe reveals both; LVM2's member-independent designator
  (VG id) is helper-only; identity facts and designators survive replug
  and reboot; and two byte-identical media collide as silent
  last-writer-wins on `by-uuid`/`by-partuuid`/`by-label` with no
  duplicate signal, only the bus-serial `by-id` staying distinct. The
  sitting's six instrument corrections and incidents are recorded, each
  caught by a declared gate before any cell was derived, including a USB2
  passthrough wedge recovered by an XHCI reattach. The matrix's section
  heading, deleted accidentally by the 2026-08-03 mechanism-amendment
  commit, is restored, and the Method section's setup-write sentence is
  widened exactly as the protocol pre-filed. Custody is complete with an
  independent second-reader rehash; the VM is destroyed with post-destroy
  verification; the status header, README M0.5 prose, and WP-035 row are
  trued.

- The SI-33/SI-28 successor protocol's S1, S2, S2b, and S3 arms are
  executed on the reattached parent-record reader (sitting of
  2026-08-04), completing every arm of the protocol. S1: `moved` in all
  three trials — `count Δ=+1` per genuine exchange, read from a fresh
  handle after a true process-local no-handle window, every empty-slot
  assertion and arrival bracket clean; the fail-open `unchanged` outcome
  never appeared, establishing the register's close-before-event/reopen
  survival sequence on this reader at three trials. S3: L4's trials 2
  and 3 both `count Δ=+1`, final at first re-read and stable through the
  idle re-reads, bringing the leg to its requested three trials. S2: the
  boundary-1 counter reset is measured (five events above the epoch
  floor before a reader re-arrival, at the floor after), the storage-node
  PDO name qualifies as an unprivileged epoch signal — changed across
  every induced boundary and the S2b reboot, stable between, counter-
  independent — and ContainerId and the USB-node PDO name are refuted;
  the qualifying token is recorded with its limit (an allocation name,
  not a globally unique epoch id). Turn-based operation and its sample
  latency are declared in the record; custody is complete with an
  independent second-reader rehash; the status header, README M0.5
  prose, and WP-035 row are trued.

- The S4 card-move rider is executed (sitting 3, 2026-08-04), completing
  every S4 arm. The operator exchanged the cards between the attached
  units — the swap declared as the executed form of the preregistered
  one-card move — and the storage-layer record is invariant under the
  exchange: the exchange is visible only as media facts (sizes and
  volumes travelling with the cards). On a pair sharing one constant at
  every layer, follows-card versus follows-reader is undecidable by
  value, with the reader attribution resting on the empty-slot form,
  which held again on both units. Recorded at full strength: both units
  re-arrived inside the exchange window despite the operator reporting
  the readers stayed attached (cause not attributable unprivileged), and
  across those re-arrivals the serial-derived instance identity migrated
  ports — a first-arrival artifact, not a unit identity — so physical-
  unit continuity across the exchange is unverifiable from any read
  surface. Custody is complete with an independent second-reader rehash;
  the status header, README M0.5 prose, and WP-035 row are trued.

- The SI-33/SI-28 successor protocol's S4 collision test is executed on a
  same-model pair (sitting 2, 2026-08-04): both NS1081-model units report
  one identical 15-character placeholder serial at the USB-descriptor and
  storage layers on all four LUNs — the preregistered collision — with
  Windows re-keying the second-arrived unit by port and clearing its
  `CM_DEVCAP_UNIQUEID` bit; the empty-slot rider is observed on both
  units; the card-move rider stays `not yet taken`. The named distinctness
  instrument (distinct USB serials) was unavailable for the reason under
  test; distinctness is established by simultaneous distinct-port
  enumeration and declared as a substitution. Custody is complete with an
  independent second-reader rehash, and the sitting's three instrument
  failures — a mis-dated transcript header, a recalled constant, and a
  decode contradicting its own raw value — are recorded as corrections
  caught before any cell moved. Additionally: the SI-35 loop and
  Windows-rerun records' outstanding second-reader obligations are
  discharged by an independent 2026-08-04 retrieve-and-rehash of all six
  named artifacts (every digest matching), recorded beside the designated
  readbacks on their pull requests with archive locators and custodian now
  named in both records; the rerun subsection's status line still read
  `not yet taken` under its taken-and-valid heading and now states its
  executed status; and the observability status header, README's M0.5
  prose, and the WP-035 row are trued to all of the above.

- **The SI-35 Windows completion rerun was taken on 2026-08-04 and is valid**,
  closing the second of SI-35's three acceptance-evidence categories. All
  three added gates are satisfied: R1 total retention (a 48 KB property-bag
  record, not a summary — the 2026-08-02 query-and-discard defect does not
  recur), R2's restored digest bracket (all seven fixtures' post-detach VHD
  digests equal their pre-attach pair, so read-only is measured rather than
  documented), and R3's mandatory index fallback (both `MSFT_Disk`-absent
  fixtures probed at every `Win32_DiskDrive`-supplied index). Two consoles
  with opposite, separately recorded privilege assertions, the measurement one
  launched ordinarily and asserting both non-elevation and absence of the
  Administrators group.
  **W-H1, W-H2, and W-H3 are all refuted.** The refutation was evaluated
  mechanically: 76 retained `MSFT_Disk`/`MSFT_Partition` fields compared
  field-by-field between the healthy control and each of the conflicting,
  damaged-primary, and missing-backup fixtures, excluding a named list of
  session-local addressing fields — **exactly one field differs in each, and
  it is `Location`, the backing file's path**, which is by-name provenance
  rather than disk state. Every named status surface is equal, the partition
  rows are identical (so under W-H1's own wording the primary table was
  parsed and presented without complaint), and the layout IOCTL agrees.
  **W-Q4 is answered `other (verbatim)`**: neither `hybrid-mbr-gpt-512` nor
  its `mbr-basic-512` control reports any scheme — both are absent from
  `MSFT_Disk` and their layout IOCTL fails `ERROR_IO_DEVICE` through a
  succeeding zero-access open, with nothing flagging the aliasing.
  A surface the parent run discarded is now retained and carries a new fact:
  `MSFT_PhysicalDisk` reports a row for **all seven** fixtures including the
  two `MSFT_Disk` omits, so the enumeration gap is specific to `MSFT_Disk`.
  Recorded rather than smoothed over: R1 and this file's no-operator-paths
  rule genuinely conflict at `MSFT_Disk.Location`, resolved by elision as the
  protocol treats embedded paths elsewhere — a redacted citation copy is
  cited and the raw copy stays local — and the measurement script now elides
  at capture time. The sitting decides no SI-35 option, supplies no
  chosen-option refusal proof, and claims nothing about interfaces it did not
  enumerate. Second-reader readback is required before reliance.
- **The SI-35 hardened non-WSL confirmation protocol was taken and passed as
  valid** on its third 2026-08-03 sitting, in fresh disposable VM 9423 at
  `b231e0f`, after two void sittings whose instrument amendments are
  recorded. Every validity gate passed: all ten sessions' descriptor-bound
  bindings and byte-continuity hashes, the distinct-inode and repeat-attach
  controls, projection stability, trial coherence, the preallocated-node
  event shape, and the unprivileged reader's negative assertions (uid 1001,
  no `disk` group, zero `CapEff`, denied direct loop reads). The result: the
  named candidate client projection is **`non-separating`** for the decisive
  healthy/conflicting GPT pair — byte-identical in every valid trial — and
  the labelled privileged `blkid -p`/`wipefs -n` comparison was likewise
  non-differing. The WSL2 promotion hold is lifted and **M0.5's loop
  criterion is satisfied**, pending the protocol's second-reader rehash of
  the retained transcript (SHA-256 `76bbd9e1…52b994`, raw capture
  `8af58b26…c7700a`). The run chooses no SI-35 option, supplies no
  chosen-option refusal demonstration, does not substitute for SI-34's macOS
  and real-partitioned-Linux rows, and refutes no existential H-separation
  hypothesis. VM 9423 was destroyed with post-destroy verification; the two
  host-attached USB devices were never referenced and are unchanged.
  A post-result review of the instrument against gate 7 found and closed a
  latent false-pass path: two *successfully captured but empty* udev entries
  would have compared equal and printed `non-separating`, where gate 7
  requires `inconclusive (udev coverage gate)`. The instrument now counts
  retained properties per subject and reports `observed(absent)`, failing the
  run. Adding a gate after seeing output is the move the protocol distrusts,
  so the claim is bounded: the gate is strictly stricter — it can only turn a
  pass into inconclusive — and the verdict was **re-derived**, not assumed, by
  re-running the amended projection over the digest-verified retained capture
  off the sitting host on a different unprivileged Linux machine, with an
  identical verdict and every gate passing. Coverage was 12 retained
  properties for the disk and 23 per partition in every session.

- The SI-35 instrument's second sitting (fresh VM 9422 at `bc922f0`) is
  recorded as **void (gate 7)** and the instrument is amended a second time,
  recorded before any subsequent run's output. The first amendment held:
  every control and trial-coherence comparison passed with `DISKSEQ`
  dropped. Two narrower evaluation defects voided the run: the stability
  comparison covered udevadm's whole rendering, and the `S:` symlink block
  renders its set in varying order — the same nondeterminism `DEVLINKS`
  shows, in a section that is not part of gate 6's projection; and the event
  requirement demanded a disk `add` that a preallocated loop node never
  emits — this kernel pre-creates `loop0`–`loop7`, so attach produces disk
  `change` plus one `add` per partition, which the captured streams show
  exactly. Amendments: stability now compares the gate 6 projection (the
  `E:` sequence with the declared `DEVLINKS` canonicalization), and the
  event gate requires adds ≥ partitions plus at least one disk `change`.
  The void run's decisive-pair line remains unusable and unquoted. Gate
  texts, schedule, trials, hypotheses, and outcome rules untouched;
  transcript retained externally, SHA-256 `f435cf5b…be5974`.

- The SI-35 instrument's first sitting is recorded as **void (gates 4 and
  7)** and the instrument is amended, with the amendment recorded before any
  subsequent run's output. The capture half completed all ten sessions
  cleanly in fresh disposable VM 9421 at `491d10f`; the unprivileged
  projection half then voided the run on three instrument defects the first
  real kernel exposed: `DISKSEQ` — a kernel-assigned monotone attach counter
  not among the six preregistered droppable keys — varied per attach and
  failed every control comparison; `udevadm info` rendered `DEVLINKS`'s
  symlink set in varying order between back-to-back captures, failing
  byte-stability; and the passive monitor's netlink subscription raced the
  attach's first event burst, capturing 2 of 3 required add events. The
  decisive-pair output of the void run is deliberately unused and unquoted.
  Amendments, justified by the validity failures alone: `DISKSEQ` joins the
  droppable keys (session plumbing, same class as `USEC_INITIALIZED`); one
  declared token-order canonicalization for `DEVLINKS` that preserves every
  token; and the monitor now waits for its readiness banner and refuses
  rather than racing. Gates, schedule, trials, hypotheses, and outcome rules
  are untouched. Raw capture and transcript retained externally, transcript
  SHA-256 `2c2528d7…a71d067`; no hypothesis row moves.

- WP-035 delivers the SI-35 hardened-protocol instrument as the second and
  only other registered higher-tier selector,
  `cargo xtask test --tier 2 --profile destructive --acceptance
  si35-loop-capture`, plus its unprivileged counterpart
  `cargo xtask si35-project --raw <file>`. The capture half runs, under the
  2e acceptance's exact native-Linux/no-WSL/explicit-elevation gate, the
  **compiled preregistered schedule** — gate 5's negative controls (two
  generation roots for distinct inodes, plus a repeat attach), six
  order-balanced alternating trials of the healthy and conflicting fixtures,
  and a closing healthy control — each entry one crate-owned
  `run_probed_session` inside a fresh 0700 scratch that is created-or-refused
  and cleaned exactly, with passive `udevadm monitor` block-event capture
  around each session and an environment record digesting the instrument
  binary, kernel config, udev ruleset, tool versions, and fixture manifest
  before the first attach. It emits raw JSON-line records and registers no
  destructive suite. The projection half **refuses elevation**, records the
  measurement user's negative assertions (no disk group, empty capabilities,
  denied direct loop reads), applies the frozen normalizer — exactly the six
  preregistered plumbing keys dropped, everything else retained — and
  evaluates the stability, control, trial-coherence, event, and decisive-pair
  gates, exiting nonzero if any gate voids the run. Monitor output is
  evidence only and is never parsed for addressing. No measurement is taken
  by this change and every observability result cell stays `not yet taken`;
  the instrument exists to be run in a fresh disposable VM sitting.

- WP-020 increment 2f's session takes the in-process verification reads its
  amended boundary names for the SI-35 instrument's gates 3, 6, and 7: the
  attached device's complete logical contents are hashed through the held
  loop descriptor before the first and after the last external launch and
  must equal the compiled catalogue digest; the named sysfs projection facts
  (disk size, read-only flag, logical block size; partition index, start,
  size, read-only flag) are read from the retained-rdev root and released
  with the report; the mount table must contain no session device number and
  every session node must report read-only, both checked during the window;
  and the predeclared udev query now runs twice per node for the
  byte-stability gate. Each check refuses with a typed value
  (`LoopDeviceHashMismatch`, `SessionNodeMounted`, `SessionNodeWritable`)
  and still reaches the unconditional detach. These reads launch nothing and
  accept no caller input; the public surface grows by integer-only facts
  types and one pinned slice getter. Tier-1 evidence: 62 Linux / 39 Windows
  crate tests, clippy clean on both platforms; no real loop device ran.

- The SI-35 hardened non-WSL protocol gains its **mechanism amendment,
  recorded before any output exists**, as WP-020's increment 2f boundary
  requires: the preregistered setup and privileged comparison actors merge
  into the one crate-owned session (`run_probed_session`), and the live
  client-projection capture moves into that session's predeclared `udevadm`
  launches, because the 2f boundary forbids disclosing the live device
  identity to any caller while the device is bound — a lent descriptor or
  name cannot be confined, per the recorded `try_clone_to_owned`
  measurement. The udev database and named sysfs attributes are
  world-readable state whose content does not depend on reader privilege,
  so the capture's privilege changes who reads, not what exists to be read;
  the unprivileged measurement shell keeps its negative environment
  assertions and all post-release analysis over the quarantine-released
  records. One preregistered mitigation is substituted and the substitute
  is recorded as weaker: gate 2's authority-drop is replaced by the
  crate-enforced bracket around every launch, which detects a rebind that
  happened rather than preventing one. No measurement is taken, no gate,
  normalizer, trial, or outcome rule changes, and every result cell stays
  `not yet taken`.
- WP-020 increment 2f is implemented: `crates/ffi-linux-loop` gains
  `run_probed_session`, the hold-open loop session its merged authorization
  boundary permits. The session consumes a SAFE-007 `Authorization` selecting
  exactly one registered fixture, configures the same read-only
  autoclear/partscan mapping 2e uses from the held verified descriptor, and
  then — the genuinely new capability — launches the predeclared probers
  itself: `udevadm settle`, then `udevadm info --query=all`,
  `blkid -p -o udev`, and `wipefs -n` against the disk and each enumerated
  partition, every launch under compiled absolute paths, structured argv, no
  shell, no `PATH` search, a cleared environment with one fixed locale pin,
  bounded capture that refuses overflow rather than truncating evidence, and
  a kill deadline. Node identity is re-statted by `lstat` (a planted symlink
  is seen, not followed) and the full `LOOP_GET_STATUS64` binding re-verified
  immediately before and after every launch, as protocol control flow a
  caller cannot skip. Captured output is quarantined in the session gate and
  released only after `ENXIO`-confirmed detach and partition teardown; every
  refusal path drops the bytes unpublished, and no public signature returns a
  descriptor, `File`, name, path, or device number — the design that lent the
  caller a descriptor was rejected in the authorization on the
  `try_clone_to_owned` escape measurement, and the structural tests now pin
  the five-function public surface. Partition enumeration is exact-or-refuse:
  a child under the session's own retained-rdev sysfs root whose name,
  `partition`, or `dev` attribute disagrees refuses the session rather than
  being skipped or guessed at. Tier-1 evidence lands on both platforms (58
  Linux / 37 Windows crate tests, two compile-fail doctests); no real loop
  device ran, Tier 1 still opens no block device, and the session's first
  real-kernel exercise is intentionally the WP-035 SI-35 instrument. The 2e
  entry point, its protocol, and its proven behavior are unchanged; shared
  configure/verify/detach logic was extracted verbatim rather than
  duplicated or altered. Increment 2f remains **weaker than 2e by
  construction** — across the open window the bracketing detects a rebind
  that happened rather than preventing one — and nothing here registers a
  destructive suite.

- WP-035's status surfaces are corrected for the closure of repository issue
  #94, and the correction is deliberately narrow: **no measurement changes and
  no register disposition moves.** The loop-backed half of SI-35 was blocked
  until #94 closed; it closed on 2026-08-03 when WP-020 increment 2e's
  descriptor-bound mechanism landed and passed a real acceptance, so the
  preregistered hardened non-WSL protocol is now **runnable rather than
  blocked**. What lifted is stated exactly rather than generally: the gate was
  the absence of a descriptor-bound attach, so a loop-backed measurement that
  configures the device *from a verified descriptor* no longer carries the gap,
  while one reaching the device by pathname — anything built on plain
  `losetup <file>` — still does, and the recording rule still applies to it.
  Nothing already measured improves. The 2026-08-02 WSL2 run stays
  non-qualifying, its decisive-pair negative stays unavailable to a register
  decision, and **M0.5's loop criterion stays unsatisfied** — takeable is not
  met, and a closed gate is not a result. Historical statements that a
  measurement was taken while #94 was open are left exactly as written,
  including the run's `Binding status | open at run time` row and the quoted
  increment-5 rule, because they were true when recorded. Touched surfaces are
  WP-035's own: the README M0.5 section and WP-035 row, the observability
  status header and its two loop-gate paragraphs, and WP-035's increment-5
  scope. No new authorization was needed — WP-035's existing share already
  names its own status rows and the M0.5 roadmap section.

- WP-020 increment 2e is **Delivered** and repository issue #94 is **closed**:
  it introduces the sole runnable
  higher-tier acceptance,
  `cargo xtask test --tier 2 --profile destructive --acceptance
  linux-loop-read-only`, while every generic destructive Tier-2 request and
  every Tier-3 request continues to refuse. The Linux-only acceptance runs with
  explicit privilege in a disposable non-WSL VM and applies SAFE-001, SAFE-002,
  and all SAFE-007 factors even though it is non-destructive and
  logical-content-read-only. Its verified backing descriptor remains held;
  backing, loop-control, and loop-device descriptors are `O_RDWR`-capable for
  mapping control; the mapping carries `LO_FLAGS_READ_ONLY`; the probe is
  in-process through the held loop descriptor; and no external storage tool or
  logical write, discard, or zero operation occurs. Linux's configure and
  rebind paths may `fsync` and write back already-dirty data or metadata, so the
  acceptance deliberately makes no zero-physical-write claim. Both authorized
  fixture objects are hashed before any attach; each initial digest must match
  the compiled fixture catalogue before any loop configuration, and
  both are hashed again after the ordinary and adversarial legs confirm detach
  and partition teardown. No observation is accepted until the sampled backing,
  loop configuration, and held-node identities match and those hashes are
  unchanged. External run evidence must exclude every other actor able to
  modify either fixture and every other actor able to administer or rebind loop
  devices. Ordinary kernel/udev read/open discovery is allowed and handled by
  bounded cleanup, but a loop-configuration `EBUSY` refuses immediately because
  isolated loop state was not established. Hash and status sampling cannot
  defeat an ABA change between samples; VM isolation bounds consequences but
  does not prove the exclusions, and this acceptance is not a continuous-binding
  guarantee. This exception grants SAFE-007
  coverage only to this named acceptance: it does not change Tier 1 or the
  product inspector's read-path boundary and it does not register a destructive
  suite.

  The acceptance **passed** on 2026-08-03 in a disposable
  Proxmox VE 9.2.4 guest — stock Ubuntu 22.04.5, kernel 5.15.0-186-generic,
  base image verified against Canonical's published `SHA256SUMS`, no USB or PCI
  passthrough, a `pre-acceptance` snapshot as the revert boundary. Two legs were
  configured and detached, the adversarial `LOOP_CHANGE_FD` rebind was detected
  and its observation discarded, partition teardown was confirmed, both
  fixtures' initial digests matched the compiled catalogue, and both were
  unchanged afterwards with `losetup -a` empty and no loop device holding a
  backing file. Four negative controls refused in the same session, including a
  generic destructive Tier 2 that authorized 13 targets and still refused
  because no suite is registered. It was run four times in that guest with
  identical harness results and identical fixture digests: three on the
  implementation commit `2dbf601`, and once more on the merged commit
  `c75b340` after main's `apps/cli` changes arrived. That last run was taken
  rather than argued: none of the changed files is on the acceptance's code
  path, which is exactly the reasoning a proof against a superseded tree
  invites and this package declines.
  The exclusions were established rather than asserted: `snapd` — which held
  four squashfs loop devices at first boot — and `udisks2` were purged, **a
  deliberate deviation from a stock image** without which the no-other-loop-
  administrator condition cannot hold; `/root` and the fixture directory are
  `drwx------` root-owned; and every non-root process's `CapEff` was read and
  none holds `CAP_SYS_ADMIN`. Two limits are recorded rather than smoothed
  over: **the guest was not network-isolated** — it held a DHCP address and a
  default route throughout, which the transcript records as a fact — and the
  digest and status checks remain discrete samples that cannot defeat an ABA
  change. Closing #94 registers no destructive suite; increment 2's own scope
  is unblocked and still unbuilt. The full record, including what the run does
  not establish, is in `docs/work-packages/WP-020.md`.

  Recorded because it will recur: this acceptance must run as root over a
  direct login with no `sudo` in the chain and no injected environment
  variables. WP-035's redaction sweep compares every environment value of six
  characters or more against CLI output, so `SUDO_USER=partman` — or any name
  colliding with product text — fails a Tier-1 gate before the acceptance is
  reached. That is the tripwire working correctly; the fix belongs in the
  environment, never in an exemption to the commit under proof.
- The SI-33/SI-28 successor protocol's two S4 rows move from `not yet
  taken` to `not established` on a custody-complete 2026-08-03 sitting:
  with both readers attached simultaneously, the delivered second unit
  measured as a different bridge model than the parent-record reader
  (VID:PID 2537:1081, NORELSYS 1081CSx, against 0BDA:0306), and S4's own
  rule forbids approximating the comparison with a different model. No
  collision hypothesis is evaluated and no register disposition changes;
  the arm remains takeable as preregistered once a same-model second unit
  exists. The sitting's capture script and complete transcript are
  archived outside the repository with digests recorded before the first
  device query and an independent second-reader rehash; the observability
  record carries the custody fields. Recorded as context only: the second
  unit reports one ascending-hexadecimal placeholder serial at both the
  USB-descriptor and storage layers, identical across its LUNs — the
  SI-28 bridge-constant form on a second bridge family.
- The SI-35 Windows completion rerun is preregistered in the observability
  record, all cells `not yet taken`: three hardened gates — total retention
  (query-and-discard, the recorded defect that made W-H1/H2/H3 unevaluable,
  now voids the sitting), the restored before/after digest bracket
  (wrapper prose is not a digest), and a mandatory layout-IOCTL
  index-fallback probe for `MSFT_Disk`-invisible fixtures — plus the W-Q4
  hybrid answer against its MBR control. The parent protocol's hypotheses,
  mechanics, and scope limits are inherited unchanged; W-H2's
  which-GPT-copy question is recorded as needing a WP-020 discriminating
  fixture and is not preregistered across that package boundary. No
  measurement is taken and no disposition changes.
- The SI-33/SI-28 successor protocol is preregistered in the observability
  record, all cells `not yet taken`: the close-before-event/reopen survival
  arm with a true process-local no-handle window and epoch-boundary
  bracketing, epoch-signal characterization over a fixed candidate list with
  the counter itself excluded as circular, completion of L4's originally
  requested three trials, and the second-reader storage-layer serial
  collision test — previously pre-registered only in conversation, which
  confers no standing — with its hypothesis, refutation condition,
  live-comparison requirement, and enumeration-failure-is-data rule now in
  the repository. README's M0.5 section and WP-035 row note the
  preregistered instruments. No measurement is taken and no disposition
  changes.
- ADR-0010 is accepted: Section 4.1's required UI layer is Svelte and
  TypeScript, SvelteKit excluded, Vite as the build tool — spec **5.0.0**, a
  major bump because §0.1 makes a semantic change to an existing requirement
  major regardless of implementation state. No code changes: `main` carries
  no UI and nothing React-specific, re-verified at acceptance. No desktop
  shell is approved — Tauri 2 stays the named shell with no new authority,
  PR #91's retirement stands, and the ten `G-AX-*` accessibility gates
  remain inconclusive. The ADR's verification obligations are deferred to
  shell authorization; until then the stack is intended, not validated.
  Current-version pointers (README, CONTRIBUTING, PR template, test tiers)
  move to 5.0.0. Substance is in the spec's own §0.3 changelog, which
  controls specification changes.
- WP-035 increment 6's measurement matrices are preregistered in the
  observability record with every cell `not yet taken`: a macOS matrix (raw
  read policy, IOMedia/diskutil projections, APFS membership and container
  UUID, foreign-signature fixtures through read-only no-mount attach, the
  SI-34 stale-signature freshness row, and a hardware-conditional Fusion
  cell) and a real-partitioned-Linux matrix (six declared layouts on
  disposable passthrough fixture media, baseline/`disk`-group/helper
  projections kept separately labelled, Part 6 precondition 2's native
  designator, stability, and collision rows, and the SI-34 freshness row on
  a real device tree). Both define closed result vocabularies, validity
  gates whose failure voids rather than refutes, and transcript custody
  with second-reader readback. The Linux provisioning-versus-Method wording
  conflict is filed inside the protocol rather than left to surface later.
  No measurement is taken, no register disposition changes, and no cell
  asserts platform behavior.
- WP-035's portability and fidelity pass gives the Linux replay open flags
  their per-ABI-family values: generic, MIPS, and SPARC targets each pin the
  reviewed `O_NONBLOCK | O_NOCTTY` encoding (`0x900`, `0x880`, `0xc000`), an
  unreviewed Linux ABI now refuses to compile rather than inheriting generic
  values, and Tier-1 asserts every family constant plus the selected target's
  wiring into the one replay open call. The dependency doctor's output bound
  is stated as it is enforced — 4096 bytes per child stream, at most 8192
  aggregate retained — in the constant names, the over-limit diagnostic, and
  a new Tier-1 boundary test; the enforced limits themselves are unchanged.
  The CLI manifest comment names the Tier-1 non-`Hash` ambiguity assertion
  rather than the removed compile-fail doctest. No limit value, requirement,
  schema version, or exit code changes.
- WP-010's authorized metadata and historical-fidelity repair marks SI-11 as
  hash-visible and ties its retained plan/snapshot obligations to MODEL-005 and
  Section 6; narrows SI-12 and spec 4.3.0 to the evidence actually retained for
  unequal multipath identifiers; and limits SI-34's stale-signature result to
  the named finite projection it measured. No option, issue state, class,
  dependency, normative requirement, specification version, or evidence status
  changes.
- WP-010's post-acceptance integrity pass reconciles four accepted ADRs with
  the authoritative register without changing a normative decision. ADR-0002
  and ADR-0005 now label their old blocker lists as acceptance-time snapshots;
  ADR-0011 distinguishes its absolute multipath policy from the cases its
  accepted detection rules currently cover and records its completed WP-035
  follow-up; ADR-0012 states the client-visible boundary of type-level
  protection and the helper's operative role for invisible facts that actually
  tighten or refuse. The register now classifies withdrawn
  SI-36 instead of leaving it outside the supposedly exhaustive class table,
  records all three SI-35 acceptance-evidence categories as unsatisfied, and
  removes claims of backup recovery and universal interface absence that the
  retained measurements cannot prove. The unequal-identifier, unassembled
  multipath residual is filed as hash-visible SI-37, an input resolved through
  SI-11 rather than an independent direct blocker; SI-11 and SI-27 retain their
  separate closure and naming responsibilities. No open option is selected, no
  evidence is promoted, and no requirement text is changed by this correction.
- The WP-035 audit closes the chassis honesty and evidence gaps it found.
  `inventory`, `topology`,
  and `capabilities` are now recognized reserved commands that return exact
  schema-versioned typed refusals at exit 3, naming the spec issues or CAP-005
  requirement that prevents each domain payload; they are no longer parser
  usage errors. The dependency doctor now preserves a real tool's nonzero exit
  as `nonzero-exit` failure rather than allowing its output to become version
  evidence, with Git supplying both deterministic success and failure launcher
  proofs at reviewed absolute paths. Human output uses an injective visible
  encoding for caller controls and backslashes, JSON escapes every Unicode
  control, and tests pin record boundaries against C0, DEL, and C1 injection.
  Replay opens now use the reviewed target ABI values — Linux
  `O_NONBLOCK|O_NOCTTY=0x900`, Darwin `0x20004` — with a source-use guard that
  keeps the constant wired into the actual open. The advertised non-`Hash`
  boundary is now a regular Tier-1 compile-time assertion rather than a doctest
  the tier did not execute, and generated traceability now carries direct
  INV-006 evidence without claiming WP-050's CAP-005 capability engine.
- The SI-33/SI-35 evidence record is corrected to the retained evidence's
  actual reach. SI-33's close-before-event/reopen arm remains unmeasured; its H
  matrix and L6a have no retained transcript, L4 reached only one of three
  trials, and the lower later reading does not characterize a counter epoch or
  prove an actual fail-open comparison. Windows SI-35 discarded queried
  PhysicalDisk property values, so equal retained fields do not satisfy the
  pre-registered existential refutation conditions. The historical WSL2 loop
  output is non-qualifying under issue #94 and a post-hoc normalizer. A new
  all-`not yet taken` protocol specifies descriptor-bound, replicated,
  order-balanced confirmation in a disposable non-WSL VM; it records no new
  measurement.
- The inspect chassis's standing gated list re-attributes its
  `same-device-claims` entry from SI-12 to ADR-0011, completing the follow-up
  the ADR's Consequences named: SI-12 resolved in spec 4.3.0, and a live
  surface citing a resolved issue as an open gate is the count-drift the
  register's sole-authority rule exists to prevent. The prohibition itself is
  unchanged — the inspector still never claims two paths are or are not the
  same device — but its state renders as `never-inferred` under the deciding
  ADR rather than `not-established` under a question that no longer exists,
  in both the JSON and human renderings, with the pinned tests moved to the
  new strings and the JSON state assertion now per-surface. README's two
  restatements of the gated list and WP-035's two boundary citations move
  with it; the `partman.cli.envelope/0` schema is provisional within major
  version 0, and this is the kind of deliberate change that provisionality
  exists for.

### Fixed

- **WP-040: the resume token's standalone decode now shares RPC-004's
  size bound.** Increment 1's discipline — the 1 MiB bound binding the
  wire before any parsing touches the bytes — held at the envelope's
  and the handshake's decode entries but not at `ResumeToken::decode`,
  which parsed first. Inside an envelope body the token was bounded
  transitively; standalone it was not, and the token travels standalone
  by design. The same pre-parse refusal now guards the token path,
  naming both numbers, with `the_resume_token_shares_the_size_bound`
  as its evidence row.

### Added

- The SI-35 **loop-device measurement** was taken read-only on 2026-08-02,
  six fixtures plus the 4Kn annex attached `--read-only --partscan` one at a
  time, read-only read back from `/sys` rather than trusted from the flag, and
  the measuring identity asserted unprivileged — uid non-zero, not in `disk`,
  and a direct read of the loop device **denied**, where the denial is the
  pass. **Repository issue #94 was open, so this measurement was taken across
  a block that had not lifted.** WP-035 says loop-backed work is *blocked*
  until #94 closes and adds a rule for what to record if a read-only
  measurement is taken anyway — a contingency, not a permission — and #94
  itself disclaims proposing "a manual, out-of-tier loop attach". The run was
  performed at the operator's explicit instruction after the gate was raised,
  though it was raised with an over-favourable reading since withdrawn.
  **M0.5's loop-backed exit criterion is not satisfied by this run.** The
  contingency was honoured: the binding-gap line travels beneath every table
  filled, WP-020's increment-2 row stays Blocked, and test-tiers.md's
  sentences remain true because nothing in the repository opened a device,
  though the environment was not §11.3's either — T2 is defined as disposable
  VMs and this ran on the operator's working WSL2 instance. The two
  authorities characterize the activity differently — WP-035 calling these
  operator-run experiments and not tier work, #94 calling the same probe
  Tier-2 work that cannot yet be made — and §11.3, cited by neither, places
  `loop` under T2 while restricting only *destructive suites* to T2 and T3.
  That is a documentation inconsistency, not a conflict between requirements,
  so it is **not** a §1.11 filing; an earlier version of this entry said it
  was, and filing it would have inflated the register's counts with a
  miscategorization. It is recorded on issue #94 instead. A post-attach
  run-time content check retained `matched` for all seven fixtures, but it was
  not part of the pre-registered phases and occurred after attach, partition
  scanning, and udev settlement; it cannot bind pathname resolution, those
  earlier events, or later rebinding. On this WSL2 run, the post-hoc-normalized
  client projections of conflict and healthy were identical, and the retained
  `blkid` properties and `wipefs` offsets did not separate them. This is
  historical, non-qualifying evidence while #94 remains open. The negative is
  also withheld pending a descriptor-bound non-WSL rerun. Every named retained
  reading taken so far — Windows CIM/layout IOCTL and Linux file/loop
  projections — failed to separate the decisive pair, but the Linux readings
  share libblkid 2.41 and are not independent. This gives option (b) no
  support; it does not decide viability. None of the register's three evidence
  categories is decision-complete: the Windows procedure ran but retained an
  incomplete surface, the loop run is non-qualifying, and the chosen option's
  refusal demonstration cannot exist before an option is implemented.
  Separately, this record requires the non-WSL confirmation, and M0.5 requires
  #94's closure. The historical WSL2 run recorded H-4Kn
  supported: a loop configured with a 4096-byte logical sector size exposed
  the GPT where file probing reported `PMBR`, consistent with the file result
  being a probing artifact. Because #94 was open, it does not qualify as the
  confirmation. The damaged-primary client projection differed from healthy
  by materializing no partitions while its udev entry still said `gpt`; that
  is a content difference, not an explicit damaged-state marker. Retained
  `wipefs` offsets exposed the missing backup while the client projection did
  not. The client projection carried no hybrid trace; helper-side hybrid
  classification was unmeasured because only offsets, not signature types,
  were retained. The initial diffs exposed backing filename,
  encoded-filename, inode, and device plumbing. The recomputation dropped all
  four `ID_LOOP_BACKING_*` keys post hoc; the retained summary does not show
  that every key differed in every pair. The first computation is void, and
  post-hoc normalizer extension is not a qualifying method; the future
  protocol freezes normalization and voids on undeclared plumbing. Equality of
  this finite projection is not a refutation of the existential claim that
  some other client-readable fact may separate the pair.

- The SI-35 **Windows partition-list measurement** was taken on 2026-08-02,
  seven WP-020 fixtures attached one at a time as read-only fixed VHDs by an
  elevated console while an ordinary non-elevated console did the measuring,
  with the executed wrapper reporting every post-detach digest `UNCHANGED`.
  The underlying pairs were not retained, so that verdict is not independently
  auditable and does not prove the bytes never changed transiently. No retained
  field from `MSFT_Disk`/`MSFT_Partition`, `MSFT_PhysicalDisk` row presence, or
  the layout IOCTL separated conflicting, damaged-primary, or missing-backup
  from healthy GPT. The wrapper retained `IsReadOnly=True` and PhysicalDisk
  `rows=1` for all five CIM-visible fixtures, but discarded queried
  PhysicalDisk property values; the declared W-H1/W-H2/W-H3 refutation
  conditions are therefore unevaluable and their broader existential
  hypotheses remain inconclusive. The damaged-primary rows cannot distinguish
  backup recovery from parsing the primary without CRC validation. The run
  gives option (b) no support on the retained Windows projection; it does not
  establish that Windows has no separating client-readable surface.
  `blank-512` remains distinct. Two fixtures — precisely those whose MBR has a
  non-protective entry — produced no `MSFT_Disk` row while `Win32_DiskDrive`
  and `Get-DiskImage` enumerated the attachment. That absence reproduced for
  each fixture in two runs about three minutes apart, but the host device set
  changed between runs. Both 800 ms and 3 s waits produced no row; longer
  settling remains untested. Because the executed wrapper returned on zero
  `MSFT_Disk` candidates, it never queried `MSFT_PhysicalDisk` or
  `MSFT_Partition` for those fixtures, and the reachable layout IOCTL was not
  run. The MBR-entry relationship is a correlation, not a mechanism, and the
  hybrid question is **not attempted**, not unanswerable.

- The SI-33 media-change-counter liveness experiment was **taken** on
  2026-08-02, on a card reader and two identical flash drives, entirely
  unprivileged and read-only, and its results are recorded in
  `docs/quality/observability.md`. L1 immediate exchanges and L2
  sixty-second-idle exchanges moved in 3/3 trials, but the full register
  sequence did not pass: L5b kept the original handle open across the event
  before opening a fresh handle, and retained L5a was floor-to-floor and
  uninterpretable. The close-before-event/reopen arm is unmeasured. L4 moved in
  its sole retained trial (1/1 taken of 3 requested), so its replication
  requirement is unmet. Prompt movement still cannot be attributed to
  exchange-synchronous detection. A later reading was lower across an interval
  containing a timestamped PnP arrival, so global monotonicity cannot be
  assumed; LastArrival is an event marker, not a driver-incarnation token, and
  reset causation was not established. The fail-open example remains explicitly
  constructed: the run establishes a decrease, not a repeated plan/apply floor
  equality or an actual witness failure. No incarnation signal was
  characterized: the run-2 same-instance Boolean was derived from the counter,
  PNPDeviceID persisted across L7 replug, and LastArrival identifies an arrival
  only. The H matrix and L6a survive only as operator reports without a retained
  transcript or group-membership record; durable evidence is limited to the
  narrower retained legs. L3's V1 step followed filesystem I/O in the same
  round instead of replacing it, so that arm also needs the specified rerun.
  An earlier L4 attempt whose physical action never happened was discarded and
  occupies no cell. The later SI-35 run records and their current limits are
  recorded in the newer entries above.
- At the time WP-035 increment 5 first landed — before the later operator runs
  recorded above — it recorded the register measurement instruments:
  protocols and recording formats for the SI-33 media-change-counter
  liveness experiment and the SI-35 loop-device and Windows partition-list
  measurements, spliced into `docs/quality/observability.md` beside the
  established rows they extend. The experiments are operator-run and
  read-only — not tests, not repository commands — and no measurement had yet
  been taken: every cell a run could fill initially read `not yet taken`. Each
  section extends the file's rule of use so such a cell cannot be relied on,
  cited, or paraphrased as a finding, and each hypothesis carried a
  preregistered outcome rule — with the degraded-state questions named as
  questions, not hypotheses — so no conclusion could be fitted to data
  afterward. The later review withdrew H-separation's finite-projection
  equality rule because it could not refute an existential hypothesis. The
  SI-33 protocol probes both CHECK_VERIFY variants across an
  access matrix from zero-access up, quarantines the device-reaching variant
  from the liveness legs, and records counter movement only as deltas and signs
  because an absolute count is session history of real hardware (SEC-006);
  the register's staleness worry — an answer the class driver already
  holds — is named as the hypothesis under test, not asserted in either
  direction. The SI-35 Windows protocol reaches the storage stack through
  a byte-deterministic fixed-VHD conversion (payload untouched, every free
  footer field pinned, with an intended digest bracket on both sides of a
  read-only attach), splits the privileged setup from the unprivileged
  measurement
  with both elevation states asserted, and makes the content-versus-state
  distinction load-bearing: the conflicting fixture's primary table is
  row-identical to the healthy one's, so no partition-list shape can be
  recorded as state separation. The loop protocol is blocked on issue #94
  and carries the gap structurally — the binding-status line must travel
  beneath every table a run fills. Every embedded script that can run
  without an attach — the VHD conversion, the CIM measurement query, the
  layout probe, and the SI-33 block — was executed on 2026-08-02 as
  validation only (a signed-hex-literal defect that killed the SI-33 block
  on paste was found this way, by running, not reading); the privileged
  attach and detach blocks and the loop-protocol shell phases were not
  executed, the latter because running the attach is itself the #94-gated
  act, and nothing from any validation run fills any cell. The README's
  M0.5 heading now reports the milestone in progress rather than not
  started, its WP-035 status row counts increments 1–4 delivered and
  increment 5's instruments recorded while stating in the same sentence
  that the package's objective — the measurements themselves — remains
  open, and WP-035's README share names the M0.5 roadmap section
  explicitly, per the WP-030 precedent that a milestone surface is
  distinct from a status row.
- WP-035 increment 4 delivers the observation records, and `partman
  inspect` answers for the first time. Every observation carries its
  attribution — adapter name, version, method — the precursor toward
  MODEL-004, whose hashed envelope remains WP-010's to deliver. The outcome
  vocabulary is ADR-C4's as the ADR wrote it, after adversarial review
  refused a paraphrase that folded read errors into unavailability:
  `observed` carries bytes or a positively determined absence — absence is
  a value, known from the opened handle's own length, and the state word
  says so; `unavailable` is the platform not exposing an answer; `failed`
  is the read itself erroring, kept distinct because collapsing the two is
  the conflation the ADR condemns. A probe straddling the object's end
  never claims absence of bytes that exist: the existing prefix is
  observed under an accurate subject and only the truly missing tail is an
  absence. `--replay` runs the fixture-replay adapter over one
  caller-named regular file: a pre-open look refuses devices and
  directories before any open in the common case, the authority is fstat
  through the opened handle, and on Unix the open is non-blocking so a
  FIFO is refused rather than hanging — a device named by path is refused
  unread, opened read-only under a rebinding race at most long enough for
  the handle to identify itself; no command reads a block device, and
  nothing opens one with write intent. A following flag is never swallowed
  as `--replay`'s value. Bytes are reported as lowercase hex at compiled
  probe offsets and classified by nobody: a mechanical test bans a named
  list of interpretation words from the observation renderings — the
  list's reach is exactly those words, with the residue held by review —
  and the standing gated list — identity-strength (SI-28),
  partition-table-state (SI-35), same-device-claims (SI-12) — travels
  in-band in every inspect answer, refusals included, because what the
  inspector will not say is stated, never inferred from silence. Without
  `--replay`, the answer is a typed no-adapter statement naming the
  platform adapter package rather than a plausible empty machine. The
  output echoes no path and no file name — a session-local selector
  stands in, and the tests pin that in both renderings. Replay over one
  of WP-020's deterministic images is pinned byte-reproducible through
  the fixtures crate as a dev-dependency, which the empty-shipped-closure
  guard keeps out of the binary.

- WP-035 increment 3 delivers the dependency doctor and the technology
  facts. `partman doctor` reports each roster tool as facts with provenance —
  present or absent with every candidate path checked, the sanitized version
  banner, the parsed version, and whether it falls inside the one recorded
  tested family (util-linux 2.41, the version the fixture prober measured) —
  under SAFE-004-derived controls: compiled absolute candidate paths as the
  executable allow-list, no PATH search, structured argument arrays, a cleared
  child environment, bounded output, and a time limit. Executable identity is
  not verified beyond the absolute path. Mapping an out-of-range version to a
  `blocked` capability is SAFE-004's own last clause and stays with WP-050's
  capability engine; a test refuses the
  CAP-003 status vocabulary in every doctor and facts rendering. The Windows
  and macOS rosters are deliberately empty and render as a typed
  not-implemented statement naming the adapter package that will populate
  them — never as "all dependencies satisfied". `partman facts` ships
  FS-007's inputs: five immutable technology limits (led by the
  specification's own example, XFS not shrinking), each carrying the basis a
  reader can check, pinned as a literal contract. The doctor's I/O goes
  through an injected launcher seam, so WP-035's Tier-1 tests launch no roster tool.
  WP-035's direct test subprocess classes are Git and the compile-time-selected
  Cargo; Git supplies the real launcher's success and nonzero-exit subjects.
  The shipped
  binary's I/O statement grew with the increment and says so: existence
  checks and version probes of roster tools, and nothing else. A banner that
  does not parse stays unrecognized with its raw line preserved, never
  guessed; the launcher's time-limit path is exercised by review and manual
  probe rather than by a Tier-1 test, a stated trade rather than a hidden
  one.

- WP-035 increment 2 delivers the deny-by-default redaction allowlist and the
  redacted `export-diagnostics` command. The allowlist is the bundle
  builder's type, not a filter: a closed field enum is the only route into
  the bundle, every variant renders compile-time data, and no API accepts a
  caller-supplied key or value — so SEC-006's field list (device serials,
  paths, labels, usernames, keys, file names) is a deny-floor the increment
  cannot fall below because no runtime value exists to leak. The bundle
  carries the build's identity, target, command surface states, exit-code
  contract, and the missing discovery evidence as an in-band typed refusal
  naming WP-035 increment 4 — never an omission a reader could mistake for a
  clean bill. Redaction tests gate the tier from four sides: the bundle's
  JSON key set is pinned as literals so widening the allowlist is a visible
  reviewed edit; the human rendering is pinned byte-for-byte so a human-only
  disclosure fails like a smuggled key; no output in any mode may carry the
  host's username, home path, computer name, or any environment value six
  bytes or longer that is not byte-equal to a compile-time constant the
  bundle renders by definition (WSL's HOSTTYPE=x86_64 equals the build
  target and forced that exemption to be named); and a source guard refuses
  any `env::var`, `env::vars`, or `var_os` spelling in the shipped sources,
  which is what makes environment-independence a tested property rather
  than a review promise — the tripwire only sees variables the test host
  sets. The bundle's command-surface states are additionally required to
  agree with dispatch behavior, so the diagnostics cannot claim an
  unimplemented surface answers. The command emits to stdout only; the
  shipped binary still opens no socket, reads no file, and reads no
  environment variable — the source guard and the empty dependency closure
  are what hold that sentence, with the glob-import bypass refused by the
  workspace's pedantic clippy gate.
- WP-035 increment 1 delivers the `partman` CLI chassis: structured argument
  parsing that owns the non-Unicode seam as a typed usage refusal instead of
  `std::env::args()`'s panic; a documented exit-code contract (0 answered, 2
  usage refusal, 3 typed refusal) whose literal values are pinned by test; an
  ANSI-free, schema-versioned JSON envelope (`partman.cli.envelope/0`,
  provisional within major version 0) wrapping every JSON emission including
  usage refusals; and the typed refusal vocabulary — `partman inspect`
  refuses with state, reference, and detail on stdout rather than printing a
  plausible empty inspection. Domain payloads are absent from every surface,
  not emitted unversioned. Two structural guards land with their reach
  stated exactly: the shipped dependency closure is empty (asserted through
  `cargo metadata`; dev-dependencies cannot reach the binary) and the output
  type carries a Tier-1 compile-time ambiguity proof that it does not implement
  `Hash`; `std`'s
  own hashers used deliberately in-crate remain a named review obligation.
  The increment was adversarially reviewed before push; the review found and
  forced closure of a self-referential exit-code test, the argv panic seam,
  and guard prose that claimed more than the mechanics establish. WP-035's
  traceability converts to generated in the same change. The register
  measurements, observation records, redaction allowlist, and dependency
  doctor are later increments and are not claimed.

- WP-030 publishes the evidence-only result of ADR-0009's bounded Slint 1.17.1
  evaluation without merging the rejected desktop runtime. A strict normalized
  manifest records immutable source/lock/artifact identities, structured
  commands, exact renderer-graph observations, and supply-chain findings but
  cannot carry a `pass` or `result` field. The generator parses the ADR's exact
  41-gate registry, rejects duplicate JSON keys and missing/duplicate/unknown
  gates, hashes integer-only input with PartMan's shared `pce/1` implementation,
  and emits byte-stable Markdown. It derives one pass, hard failures at
  `G-CFG-08` and `G-SC-01`, and 38 inconclusive rows. Windows unstripped
  executable measurements remain explicitly non-decisive; no Slint runtime,
  renderer dependency, licence exception, or supply-chain waiver lands.
- WP-000 closes issue #39 after all four current package traceability documents
  completed package-owned, zero-loss conversions. Every document is now
  generated from validated source annotations and typed evidence; declarations
  are checked in both directions, hand edits fail CI, and each predecessor is
  frozen by an exact revision/blob migration ledger. WP-000 remains in progress
  for its other documented gaps rather than being rounded up by this one
  completion.
- WP-020 increment 1g converts its traceability document to generated evidence.
  Source-local annotations cover deterministic fixture construction,
  purpose-binding mutations, external-prober expectations, and the
  disposable-target interlock, while typed package evidence covers the shared
  runner, Linux-only prober command, workspace lint boundary, and zero-loss
  migration. A frozen source-revision/blob ledger accounts for every former
  row, correction, limitation, and residual risk. The conversion also removes
  a stale operator-provenance claim: the public, build-derived token proves only
  exact value presentation and supplies accident friction; it does not prove
  that an operator ran the generator or intended an operation. Tier 2 and Tier
  3 remain unavailable.
- WP-000 scheduled maintenance: every Monday and on manual request, dependency
  policy is re-evaluated on Windows, Linux, and macOS even when the repository
  has not changed, while both current fuzz targets receive 15 minutes each and
  carry a GitHub-hosted corpus forward from earlier successful runs. The
  pull-request workflow and all eleven branch-protection check names remain
  intact; a structural test holds the schedule, corpus key, duration, OS
  matrix, and existing gate roster to those claims. Its first full run exposed
  libFuzzer's 2 GiB aggregate RSS default as too low for 15 minutes of the
  allocation-heavy structured-value target despite only about 26 MiB of live
  heap. Fuzzing now pins a 4 GiB process ceiling while retaining a separate
  256 MiB single-allocation ceiling, a 4,096-byte input limit, and the existing
  25-second per-input timeout.
- WP-010 increment 2a resolves SI-31 with schema-level canonical set semantics
  in spec 4.1.0 and ADR-C6. Rust and TypeScript now sort set elements by an
  unsigned lexicographic comparison of each element's full canonical bytes,
  reject duplicates without deduplicating, validate strict order at the schema
  boundary, and inherit the enclosing artifact's depth budget when producing
  element sort keys. One shared, deliberately unsorted fixture pins exact
  bytes, hashes, comparator disagreement, and the accepted/rejected depth
  boundary. Semantic arrays and the `pce/1` profile remain unchanged. Both
  existing fuzz targets exercise the new producer/validator paths, and WP-010's
  traceability is now generated with a zero-loss migration ledger. The
  TypeScript authorization boundary also snapshots generic arrays, schema sets,
  and raw hash bytes without consulting caller-controlled constructor species
  or mutable prototype methods; regression tests cover element substitution,
  dropping, reordering, and wider typed-array views.
- WP-030 increment 1c converts its traceability document to generated evidence
  without treating row-count similarity as proof of preservation. Thirty-one
  live Rust tests now carry requirement-bound claims, non-test evidence is
  typed in the work-package source, and an exact source-revision/blob ledger
  accounts for every former evidence row and narrative section. The conversion
  also corrects vocabulary findings so entity, severity, and progress roles
  report UI-003, PLAN-004, and UI-011 respectively, with a hostile progress-role
  deletion proving the UI-011 path can fail.
- WP-010 increment 1: the `pce/1` canonical encoding. `schemas/canonical-encoding.md`
  specifies it normatively, and `crates/domain` implements the encoder, a strict
  validating decoder, and SHA-256 hashing over canonical bytes (MODEL-005).
  Golden vectors pin the encoding byte-for-byte, including the `2^53` and
  `2^64 - 1` boundaries that RFC 8785 could not have carried as JSON numbers.
  The decoder rejects rather than repairs: non-shortest arguments, floats, tags,
  indefinite lengths, non-text or misordered map keys, duplicate keys,
  ill-formed UTF-8, lengths beyond the input, nesting past a fixed depth limit,
  and trailing bytes.
- WP-010 increment 2: `packages/canonical`, the TypeScript half of MODEL-005.
  Both languages now read one shared fixture,
  `schemas/canonical-encoding-vectors.json`, so parity is proven against a
  single source rather than two per-language tables that could drift.
  `cargo xtask cross-language` runs the proof and gates CI as its own job. The
  package has no runtime dependencies: hashing uses Web Crypto and testing uses
  `node:test`.
- WP-020 increment 1: `crates/fixtures`, the deterministic disk-image generator,
  and the SAFE-007 disposable-target interlock. `cargo xtask fixtures` writes
  synthetic disk images — GPT, 4Kn GPT, MBR, blank, damaged-primary,
  conflicting-tables, hybrid MBR/GPT, APM, and on-disk signatures for LUKS2,
  LVM2 and mdraid — into the gitignored `tests/generated/`, each
  a pure function of the code that builds it, so two machines produce identical
  bytes and nothing binary is ever committed. The interlock requires all three of
  SAFE-007's proofs and computes disposability from a target's own bytes rather
  than accepting an assertion: a block device cannot pass, because its bytes will
  never equal a generated fixture. Tier 2 and Tier 3 still refuse, now for the
  honest reason that no destructive suite exists to run.
- WP-020 increment 1d: `crates/fixtures/src/evidence.rs`, which binds every
  fixture's bytes to the rationale recorded beside it. Until now nothing did:
  every layout and signature test rebuilt its own image from its own literals,
  so the catalogue was free to produce something else. Measured before it was
  fixed — with the LUKS2 builder emptied to a blank image and the
  multi-signature builder stripped of the stale mdraid superblock the Part 5
  asymmetry finding rests on, **all 64 tests passed**. Each catalogue entry now
  has a claim computed from its bytes; the set is exhaustive in both directions;
  each claim is paired with a mutation it must reject, so a check that cannot
  fail is caught; and `generate` refuses to write an image that no longer serves
  its purpose, naming what was lost. The oracles reimplement CRC-32, LVM2's CRC
  and mdraid's folded sum by different methods from the writers they check, each
  anchored outside the repository — to the published IEEE check value, and to
  three checksum fields `libblkid` 2.41 accepted, pinned with their provenance.

  The module was put through an adversarial pass before being proposed, and it
  found the first version repeating the defect it was written to end. The gate in
  `generate` was load-bearing on nothing — deleting it kept all 74 tests green,
  because every test fed it the real catalogue, which passes. Its
  "anchored outside this repository" claim held for one checksum of three:
  changing an initial constant in both writer and oracle kept everything green
  while making every fixture undetectable. And ten claims accepted mutations that
  destroyed a fixture's purpose while leaving its checksums valid — most sharply,
  "two tables that disagree" was proven by comparing entry-array CRCs, which one
  character of a partition *name* satisfies while both copies describe identical
  extents. All are closed, and the details are in
  `docs/work-packages/WP-020.md`.
- WP-020 increment 1e: `cargo xtask probe`, which re-runs `blkid` and `wipefs`
  over every generated fixture and compares against the expectations recorded in
  `crates/fixtures/src/prober.rs`. This closes the project review's open finding
  that real-prober acceptance was "manual, not regression-protected" — the last
  place in the package where an important property rested on someone having
  looked once. It needs Linux, so CI runs it as its own job; both tools are
  read-only and are handed regular files, never a device.

  A verbatim capture of `libblkid` 2.41's output is embedded in the tests and the
  recorded table is compared against it, so the table is checked on machines with
  no prober too — otherwise a transcription slip between the measurement and the
  table would look exactly like a passing test. The comparison is proved capable
  of failing in all four directions: a format no longer detected, a changed
  answer, a lost signature, and an added one.

  **Its first run falsified a claim increment 1 had recorded.** On util-linux
  2.39.3, which stock `ubuntu-24.04` ships, `blkid -p` reports nothing at all for
  `mdraid-1.2-member-512.img`, while `wipefs` still lists the superblock and both
  tools agree about every other fixture — including the 0.90 superblock in the
  stale-pair image. Increment 1 checked the signature writers by hand against
  2.41 on one machine and recorded the result as unconditional, so FS-004 Linux
  RAID and LIN-005 are **not** established on that platform. The expectation is
  now version-keyed rather than relaxed: below 2.41 the recorded answer is
  silence, so a prober that starts naming the fixture fails just as one that
  stops does. Which condition 2.39.3 rejects is unestablished — both versions'
  checksum routines are arithmetically identical, and the fixture satisfies the
  magic, `major_version` and `super_offset` checks — and it is recorded as
  unestablished rather than guessed at.
- WP-030 increment 1: `schemas/design-tokens.json` and `crates/tokens`, the
  design tokens and the accessibility harness that computes UI-001, UI-007 and
  UI-008 from them. The token file is the single source of truth and lives in
  `schemas/` for the reason `AGENTS.md` already records for the canonical
  vectors: when the front end arrives it must read *this* file, because an
  implementation checked against a table it also owns proves only
  self-consistency. `cargo xtask tokens` runs the audit and is part of
  `cargo xtask ci`.

  **The first palette failed its own harness on ten counts.** Chosen by eye and
  entirely reasonable-looking, it put `severity.reversible` — PLAN-004's "fully
  undoable" — at delta-E 10.1 from `severity.destructive` — "data is
  intentionally destroyed" — under deuteranopia, against a floor of 12. In the
  high-contrast theme, the one a low-vision user is most likely to choose, the
  same pair measured 4.8. Three further risk pairs collapsed the same way and
  three borders sat below WCAG's 3:1 floor for interface components. The floor
  was not lowered: the severity ramp now varies in lightness as well as hue,
  because lightness survives every colour-vision deficiency and the red-green
  axis does not, and the light theme's `reversible` is teal-leaning so it keeps
  a blue component deuteranopia preserves. Closest surviving pair is 21.9.

  Every check is paired with a mutation it must reject, and each was confirmed
  by deleting the check it targets and watching the table go red — the deletion
  sweep WP-020 established after finding a gate that was load-bearing on
  nothing. The colour maths is anchored outside the repository (black on white
  is WCAG's published 21:1; black against white is delta-E 100 because CIELAB
  lightness runs 0..=100), and the colour-vision matrices are checked by their
  defining property — red and green converge under protanopia and deuteranopia
  but not tritanopia, and greys are untouched by all three — rather than by
  trusting transcribed digits.

  What it does **not** establish is recorded in `docs/work-packages/WP-030.md`
  and repeated in the harness output on every run: it renders nothing, so the
  keyboard, screen-reader, zoom and reduced-motion halves of UI-008 are
  untouched; only declared pairings are checked, so a combination the front end
  invents is invisible to it; and the colour-vision check is a model, not a
  proof — UI-007's redundant channels are the guarantee. M0's "accessibility
  harness runs" criterion is therefore **partially** met.
- `cargo xtask verify-ownership`, closing the mechanically decidable half of
  Section 1.10. Every `docs/work-packages/WP-*.md` now carries an `owned-paths`
  block, which is the same text a reviewer reads, so the prose and the enforced
  data are one thing rather than two that drift. The check refuses a tracked
  file no package claims and a claim matching no file — both mutations were run
  — and reports overlaps rather than forbidding them, because `tools/xtask/**`
  is genuinely shared by three packages and forbidding that would push the
  sharing into prose where nothing can see it. All 100 tracked files are
  claimed. It runs inside `cargo xtask ci`.

  Only exact paths and `directory/**` are understood, and anything else is an
  error rather than a pattern silently matching nothing — the failure mode the
  action scanner was audited for twice.

  WP-030's increment-2 assignment is reserved *ahead of the work*, in an
  `owned-paths-reserved` block the checker reports rather than requires to
  match. Both audits observed that WP-030's assignment did not authorize
  creating a Tauri shell and that widening scope in a pull-request description
  afterwards is the pattern ownership exists to prevent. The reservation also
  records the audits' design constraint: the front end consumes a generated
  typed accessor, never a copy of the palette.

  What this does **not** do is decide whether a given change came from the
  package owning the path — that needs a pull-request-to-package mapping this
  repository does not carry, and it is the remaining half of issue #39.
- ADR-0007, accepted, deciding what SAFE-007's disposable-test token proves.
  WP-020 carried "decide a genuinely independent token factor" as an open
  precondition since increment 1, and both audits repeated it. The queued
  answer — add an entropy source — turned out to be wrong in an instructive
  way: `authorize` trusts nothing inside the directory it verifies, because
  accepting a caller-supplied manifest was a defect that let a hand-written one
  authorize an arbitrary target, and a per-generation random token cannot be
  compiled in. The interlock would have to read it from the fixture root,
  re-creating that exact trust dependency, so randomness would have added a
  dependency and a writable-file trust while defeating nobody.

  Read exactly, SAFE-007 requires the three factors to be *present* and forbids
  one environment variable from standing in for all of them; both hold, and it
  does not require independence. The token is an operator-intent proof, the
  documents already said so, and the precondition is closed by decision rather
  than by code. A real third factor needs state outside both the source tree and
  the fixture root, which is a T2/T3 lab-architecture question recorded as the
  ADR's revisit condition.
- SI-36 filed: SAFE-009 neither permits nor forbids reviewed `unsafe` in a
  test-fixture crate. WP-020's Windows other-name check needs link count by
  handle, `MetadataExt::number_of_links` is unstable behind `windows_by_handle`
  on the pinned 1.96.0 toolchain, and the FFI alternative runs into SAFE-009's
  two lists naming `crates/fixtures` in neither. An enumeration is not a rule,
  so per Section 0.2 it is filed rather than guessed. The residual is recorded
  and narrow: while an authorization is held the Windows share mode refuses
  writes through any name for the object.
- ADR-C1, accepted, fixing the canonical encoding and hash strategy.
- ADR-C5, accepted, fixing the aggregation vocabulary: one `Aggregate` node in
  place of three undefined Section 5 names, on-disk signatures as their own
  nodes, and `StorageSnapshot`. Landed as spec 4.0.0. It resolves four of the
  conflicts blocking WP-010 increment 3 and does not unblock it.

- WP-000 repository foundation: pinned Rust workspace, Tier-1 task runner,
  cross-platform CI, formatting/lint policy, dependency policy, and ADR
  template.
- `cargo xtask verify-actions` enforces SEC-010 digest pinning for GitHub
  Actions. It runs inside `cargo xtask ci` and as a Tier-1 test, and fails
  closed when no workflow can be read.
- `.gitattributes` normalizes line endings to LF in every working tree.
- `SECURITY.md` defines a private disclosure channel and reporting scope.
- Job timeouts on both CI jobs.

### Changed

- `cargo xtask verify-actions` enforces the rule it reports. SEC-010 and
  `AGENTS.md` require every action pinned to a full commit SHA **with the release
  tag in a trailing comment**, and the error message said exactly that — but the
  scanner stripped the comment before checking and `is_pinned` validated only the
  SHA, so a bare 40-character digest passed a gate that claimed to require a tag.
  The comment is now carried through and must name a version. Without one a
  reviewer cannot tell which release a digest corresponds to, so a bump becomes
  40 hex characters to resolve by hand. The repository's own workflows already
  complied, so nothing had to change to pass it — which is why the gap survived.
- `xtask` separates command parsing from execution, so every documented task,
  rejected task, and tier decision is unit-tested without launching a
  subprocess.
- Removed `[build] rustflags = ["-Dwarnings"]` from `.cargo/config.toml`. Cargo
  discovers that file from the working directory, so the flag applied to every
  crate compiled from anywhere inside the repository: third-party dependencies,
  out-of-workspace manifests, and the supply-chain job's
  `cargo install cargo-deny`, which built its entire dependency tree under
  `-D warnings`. That tree compiles warning-free today, so nothing was failing
  yet, but the exposure grows with every dependency added and every rustc
  release that introduces a lint. Workspace lint scope now comes from
  `[workspace.lints]`, and `cargo xtask ci` still fails on any warning in
  workspace code through `cargo clippy -- -D warnings`.

- ~~The project is deliberately unlicensed until it is complete.~~ Superseded
  below. PartMan is now `MIT OR Apache-2.0`.

- The project is licensed `MIT OR Apache-2.0` at the recipient's choice
  (ADR-0006), the Rust and Tauri ecosystem standard. `LICENSE-MIT` and
  `LICENSE-APACHE` carry the texts; every workspace member, the out-of-workspace
  `fuzz` crate, and `packages/canonical/package.json` declare the expression.
  Apache-2.0 supplies the explicit patent grant that MIT lacks — worth having
  for code that drives NTFS, exFAT, and APFS paths — while the MIT arm keeps the
  result usable by GPL-2.0-only projects, which Apache-2.0 alone would not.
  Both arms were already on `deny.toml`'s allow-list, so no supply-chain rule
  was relaxed to accommodate the choice.

  `[licenses.private]` is now `ignore = false`. That exemption existed only to
  stop `cargo deny` reporting `error[unlicensed]` against the unlicensed
  workspace; with the cause gone it is removed rather than left dormant, so the
  project's own crates are checked by the gate that checks every dependency.
  This closes the WP-000 known gap that recorded SEC-005's license inventory as
  unsatisfiable. Two manifests remain outside the gate and are recorded as gaps
  rather than counted: `fuzz/Cargo.toml` is outside cargo-deny's graph, and no
  license gate reads `packages/canonical/package.json`.

  ADR-0006 also makes the GPL boundary binding, which the unlicensed state had
  made moot: PartMan invokes GPL storage tools as separate processes under
  SAFE-004 and reaches UDisks2 over D-Bus, may link LGPL libraries such as
  `libblkid` and `libblockdev` dynamically, and MUST NOT link a GPL library.
  `libparted` is named specifically — it is the obvious dependency for a
  partition editor, and `cargo deny` cannot catch it, because a `-sys` crate
  declares its own license and not that of the C library it links.

- Contributions are accepted, inbound=outbound under Apache-2.0 §5, with no CLA.
  `CONTRIBUTING.md` previously barred outside contributions because the rights
  in one were undefined for both sides; that reason no longer holds.

- `cross-language` and `supply-chain` run on all three operating systems instead
  of Linux alone. Both were narrowed to save metered private-repository runner
  minutes, where Windows bills 2x and macOS 10x, and that constraint is gone.
  The widening is not symmetric bookkeeping: the MODEL-005 parity proof can fail
  on CRLF translation of the shared vector file or on a platform-specific Node
  build, and cargo-deny resolves a per-target graph, so once the platform
  helpers add `windows-sys` or `core-foundation` a Linux-only run would be blind
  to advisories reachable only from Windows or macOS. `prober-acceptance` and
  `fuzz-smoke` stay Linux-only for reasons that were never cost — `blkid` and
  `wipefs` have no Windows or macOS counterpart, and cargo-fuzz has no supported
  Windows target while the decoder under test is byte-oriented and
  endian-independent. Both reasons are now comments in the workflow, so neither
  job gets widened later for the appearance of consistency.

### Fixed

- `cargo xtask verify-change-ownership` closes the half of Section 1.10 that the
  inventory check deliberately left open, and that an audit then caught in
  practice: PR #47 was a nominal WP-000 change that also edited WP-010, WP-020
  and WP-030 documents, and `verify-ownership` passed it because every path was
  claimed by *someone*.

  Every commit now carries a `Work-Package: WP-0NN` trailer, and every changed
  path must fall inside that assignment. A trailer rather than a branch name or a
  label because this repository's branch names are inconsistent — keying on them
  would have been a guess dressed as a rule — and because trailers are already
  used here, need no API call, and stay in the log permanently.

  The assignment is read from the **base revision**, never the working tree, so
  widening your own `owned-paths` block in the same change buys nothing. That was
  the audit's specific criticism, and a deletion sweep confirms it: switching the
  read back to the working tree fails the self-widening test by name.

  `Governance: <reason>` permits editing the assignments themselves, and then
  **only** `docs/work-packages/WP-*.md` may change — otherwise the trailer would
  become a universal bypass for the check it sits beside. That restriction is
  swept too.

  Wired into CI as a *step* in the existing Tier-1 job rather than a new job, on
  Linux and pull requests only. A new job would need a new required-status-check
  name, and adding one without updating branch protection in the same change
  leaves every pull request waiting forever on a check that never reports.

- **The npm advisory check audits every package, not one named directory.** It
  ran in `packages/canonical` by name, because that was the only npm package
  there was. WP-030 reserves `packages/ui/`, `packages/design-tokens/` and
  `apps/desktop/`, and a Tauri front end normally brings its own `package.json` —
  each of which would have been audited by nobody while `cargo xtask
  cross-language` went on reporting success. Discovery is a tree walk now, for
  the same reason the action scanner's is: a gate that checks a hard-coded path
  stops covering the repository the moment the repository grows, and says
  nothing when it does. A package without a committed `package-lock.json` is a
  violation rather than a skip — `npm audit` without one reports a verdict about
  a tree that install time decides. Coverage is unchanged today (one package,
  the same one) and correct when the shell lands.

- **The Dockerfile half of `verify-actions` fails closed.** Structural YAML
  parsing closed the workflow half and left this separate line scanner alone; a
  project audit found three fail-open paths in it and an adversarial pass found
  six more. All nine are permanent regressions, and each was confirmed against
  the old scanner first — the gate exited successfully on every one of them.

  Four needed no unusual syntax at all. **A tab after `FROM`** hid the
  instruction completely, because the matcher was `strip_prefix("FROM ")` and
  BuildKit splits on `[\t\v\f\r ]+` — the cheapest bypass in the file. **A UTF-8
  BOM** on the first line hid it too, which is what a Windows editor produces by
  accident. **`COPY --from=<image>`** and **`RUN --mount=…,from=<image>`** pull
  images that never appear in any `FROM` and were never looked at. And
  **`FROM alpine AS alpine`** shadowed itself, because the stage was registered
  from the same line before the base was tested against it.

  The audit's three: a `$`-prefixed base was skipped outright, so
  `ARG BASE=alpine:3.20` + `FROM ${BASE}` passed while pulling a mutable tag —
  it is refused now rather than resolved, because resolving would have to prove
  no `--build-arg` overrides it; `FROM` matched case-sensitively though
  Dockerfile instructions are not; and `# syntax=docker/dockerfile:1`, which
  makes BuildKit fetch a frontend image and **run it as the builder**, was
  discarded as a comment.

  Two things are deliberately not violations, and are tested so they stay that
  way: `FROM scratch` is not a pull, and `# check=`/`# escape=` name no image,
  so only `syntax=` is treated as a dependency.

- **`unsafe_code = "deny"` was opt-in.** `[workspace.lints]` reaches a crate only
  if that crate's manifest says `[lints] workspace = true`, and nothing checked
  that it did. Measured: a workspace member without the stanza, containing an
  `unsafe fn`, produced **zero** diagnostics and `cargo xtask ci` stayed green.
  A safety property resting on a line somebody has to remember is the shape this
  repository rejects everywhere else. `cargo xtask ci` now asks `cargo metadata`
  for the member list and reads each manifest's text — metadata resolves the
  inheritance away, so by the time it reports a package, a crate that opted in
  and one that did not look identical. Deleting `[lints]` from `crates/tokens`
  now fails the gate by name.

- `verify-change-ownership` enforces the rule it claimed to. A project audit and
  an adversarial pass over the gate found five ways a change could travel without
  belonging to anything, and all five are closed with regressions and a deletion
  sweep each:

  - **One trailered commit laundered every untrailered commit beside it.** The
    trailers of a whole range were folded into one set, and the set had to hold
    exactly one package — so a two-commit pull request passed with a trailer only
    on the second. Each non-merge commit is now asked for its own.
  - **The parse was a line scan, not a trailer parse.** Any line beginning with
    the key after trimming counted, including a fenced example inside a commit
    body, while a genuine lowercase `work-package:` trailer — which git accepts —
    was refused. Git's own parser answers now, through `%(trailers:…)` in the
    same `git log` call: no house dialect to keep in step with git's.
  - **Merge commits are exempt deliberately, and that is now written down.** The
    documents said "every commit", which could never have been enforced: `strict:
    true` branch protection makes `gh pr update-branch` write untrailered merges,
    CI judges GitHub's generated `refs/pull/N/merge`, and `main` carries 51 merge
    commits of which none has a trailer. A literal rule would have failed every
    pull request the day it landed. The prose was corrected rather than the code
    tightened to match a sentence nobody could satisfy.
  - **An empty `Governance:` reason was accepted** and printed as an empty
    parenthesis — an audit record of nothing. And a commit declaring both modes
    was silently judged as governance, so the work package beside it was never
    checked against anything. Both are refusals.
  - **A rename was judged only at its destination.** Detection is on by default
    and `--name-only` prints only where a file landed, so `git mv` carried a file
    out of another package's territory unseen — and a `Governance:` change could
    delete *any* file in the repository by renaming it to a
    `docs/work-packages/WP-*.md` name, because every path the check could see was
    then an assignment document. `--no-renames` makes the source a deletion.

  Two more defects lived in that same expression. `-z`, because `--name-only`
  C-quotes a non-ASCII path, so `crates/tokens/src/café.rs` inside owned
  territory was refused as a stray — a gate rejecting work it should permit costs
  trust as fast as a bypass does. And no `trim`, because git does not quote a
  leading space, so ` crates/tokens/src/lib.rs` was silently normalised onto the
  owned path. A path is a byte string.

- The inventory and the change gate **agree about a reservation**, which they did
  not, and the disagreement deadlocked WP-030. A package may write inside its own
  `owned-paths-reserved` block — `verify-change-ownership` always allowed it — but
  `verify-ownership` did not count a matching reservation as coverage, so the
  first commit to create those files passed the change gate and then failed
  `cargo xtask ci` with "claimed by no work package" about a path the package had
  claimed in advance, in the document, precisely so this could not happen. The
  promotion that would have resolved it has no route that is both green and
  permitted: a governance change moving the paths early leaves `main` red on a
  stale claim, and moving them alongside the files is an assignment edit under a
  `Work-Package:` trailer, which `AGENTS.md` forbids. A reservation counts once
  it matches something; one that matches nothing is still reported, not counted.

- `verify-change-ownership` understands a **generated** file, which it had to
  before any package could add a crate. The gate as first landed made the next
  scheduled piece of work impossible, and not only that piece: `Cargo.lock` is
  claimed by WP-000 alone, and every package that adds a crate or a dependency
  rewrites it.

  Measured against `02ec952` rather than reasoned about. A minimal
  `apps/desktop/src-tauri` plus its workspace member line, committed as
  `Work-Package: WP-030`, was refused for `Cargo.lock` and `Cargo.toml`; the
  identical tree committed as `Work-Package: WP-000` was refused for the crate it
  would have had to create, because `apps/desktop/**` is WP-030's reservation.
  Neither package could take the first step. Landing the member line *before* the
  crate is not a way out either — Cargo fails to load a workspace whose member
  has no manifest, and a glob does not help: `apps/*/src-tauri` matching nothing
  falls back to the literal path and fails the same way, so `cargo xtask ci`
  would have been red for everyone in between.

  A `derived-paths` block declares a path generated rather than authored. Any
  package may then carry it — **but only alongside a manifest that lockfile
  actually resolves.** A lockfile moving on its own is refused with its own
  explanation, because nothing in such a change asks the resolver for a different
  answer, and a transitive dependency quietly re-pinned to a different version
  with a valid checksum satisfies `--locked` perfectly well.

  **The predicate took three attempts, and the first two were lexical.** The
  first accepted any `Cargo.toml` anywhere: `fuzz/` is excluded from the
  workspace, so editing `fuzz/Cargo.toml` cannot change the root `Cargo.lock` —
  yet it unlocked it. The second matched a manifest to the nearest lockfile above
  it, which an adversarial pass broke twice over: a file merely *named*
  `Cargo.toml` — a note, a fixture, a symlink — anywhere a package already owned
  was accepted as a manifest, and deleting `fuzz/Cargo.lock` in one pull request
  let `fuzz/Cargo.toml` vouch for the root lock in the next while `fuzz` stayed
  excluded.

  A fourth lexical predicate standing in for a semantic fact was not worth
  writing. `cargo metadata` is asked which manifests belong to the workspace that
  lockfile locks, so membership is answered by the tool that defines it, and the
  virtual root manifest is included explicitly because adding a member to it is
  the most legitimate reason of all for the lockfile to move. Both earlier holes
  are permanent regressions.

  **The three-OS gate earned its keep on the first run.** The membership lookup
  relativized `cargo metadata`'s absolute manifest paths with
  `if let Ok(relative) = path.strip_prefix(root)`, silently discarding anything
  that did not strip. On macOS nothing did: `std::env::temp_dir()` is
  `/var/folders/…`, `/var` is a symlink to `/private/var`, and cargo answers with
  the resolved path. The set shrank to the workspace root alone and a legitimate
  change was refused — with a message helpfully listing the manifests that would
  have worked, none of which was the one the author had just edited. Linux and
  Windows were green. Both spellings of the root are tried now, and a path that
  matches neither is a refusal rather than a quiet omission.

  A document may also only declare a path generated if it **owns** that path.
  Generatedness is a property of the file rather than a privilege of one
  assignment, and that argument stands — but a document asserting it about a file
  it does not answer for was a unilateral grant to every package, made in a
  change that edits nothing but assignment documents.

  Declaring a path generated is not claiming it: the inventory check still
  requires an `owned-paths` claim, or "this is generated" would be a way to make
  a file belong to nobody while the inventory read as complete. And a derived
  path whose derivation this tool cannot check is refused rather than exempted —
  an exemption nobody can verify is a hole with a comment beside it.

  Four deletion sweeps confirm every part is load-bearing: dropping the manifest
  requirement, accepting any manifest anywhere, accepting any derived pattern,
  and letting a derived declaration count as inventory coverage each fail a
  named test.

  **What it does not establish:** a re-pin travelling alongside a genuine
  manifest change passes. Telling the two apart needs the resolver's answer at
  both revisions — the base tree and a full resolution on every pull request.
  That residual risk is the one the repository has always carried; this does not
  widen it, and it is recorded in `docs/quality/dependency-policy.md` rather than
  implied to be covered.

- WP-020 increment 2d: the Windows halves of issue #51, and one of them was a
  live defect rather than a missing check. A file outside the fixture root
  holding a fixture's exact bytes, hard-linked in at that fixture's name, passed
  name, location, type, length and digest, authorized, and was destroyed through
  the authorized handle — reproduced before it was fixed, with the link count
  reading 2 through the handle the whole time and read only under `cfg(unix)`.
  The comment claiming the Windows share mode closed that hole is deleted: the
  share mode refuses *other* openers, and the write that reaches every name is
  this interlock's own.

  **Neither half needed FFI, so the crate-placement question issue #51 asks to
  be decided is not decided — the premise did not survive measurement.**
  Containment needs a held *directory* handle, not a handle-relative open, and
  `std` opens directories given `FILE_FLAG_BACKUP_SEMANTICS`; the link count
  needs `GetFileInformationByHandle`, which a safe wrapper exposes. So
  `unsafe_code = "deny"` still holds across the whole workspace and no new crate
  exists. The rejected options, including the ntdll route that would have been
  stronger, are recorded in `docs/work-packages/WP-020.md` rather than dropped.

  The root is now held with a share mode excluding `FILE_SHARE_DELETE`, which
  makes the filesystem refuse to rename or delete it during the one window the
  target handles cannot cover — between the root open and the first child open.
  **Windows containment is therefore enforcement by the filesystem, not
  resolution from a handle, and it holds only as far as the driver does.**
  Measured: NTFS, `ReFS` and the Windows SMB server refuse the swap; the WSL 9p
  redirector does not, and a swap staged from the Linux side redirected the open
  to a decoy with the root handle held. Roots that are not locally served are
  refused outright, which over-refuses SMB to a Windows server and is the
  deliberate direction under SAFE-005.

  Six mutations, each caught by the test named for it — and two of the tests
  were decoration until the mutations said so. The namespace test exercised only
  its classifier, so deleting the call site left the suite green; it now goes
  through `RootDirectory::hold`. The reparse-point guard proved unreachable —
  `is_file()` through the handle already refuses a file symlink and a junction,
  measured both ways, contradicting a review finding — so it was removed rather
  than shipped as an untestable branch. A drafted test asserting that the root
  cannot be renamed while an authorization is alive was also rejected: that
  refusal comes from the *target* handle and predates this increment, so the
  test passed with the fix removed.

  What it does not close is listed in `docs/work-packages/WP-020.md`: a
  third-party filesystem presenting as a drive letter is indistinguishable from
  NTFS by a path-prefix check; the link count is a snapshot and a later alias is
  not prevented; `ReFS` file identity is unproven; and the symlink test is
  environment-gated. Tier 2 stays unavailable on every platform.

- WP-020 increment 2c: containment now starts from a held directory object, on
  Unix. Increment 2b bound every check to the target's handle and still opened
  that handle by absolute pathname, and `O_NOFOLLOW` constrains only the final
  path component — so renaming the fixture root aside and leaving a symlink at
  its name redirected the open to an out-of-root file whose length, digest, type
  and link count all matched. No check on the object could have caught it: a
  user's ordinary file may hold a fixture's exact bytes, which is what
  `object_verification_alone_cannot_prove_root_membership` already recorded.

  `Authorization` holds the fixture directory open and targets are opened
  relative to that handle by catalogue basename, via `rustix::fs::openat` with
  `NOFOLLOW`. There are no intermediate components left to redirect.

  *Corrected 2026-07-30: this entry also said the directory handle "outlives the
  target handles because one value owns both". It does not —
  `Authorization::into_targets(self)` moves the targets out and drops the root
  field before the caller uses them. The implementation is unaffected, because
  containment is established at `openat` time and is a property of the returned
  descriptor rather than something the directory handle maintains afterwards; the
  root is worth holding for a different reason, that it denies a consumer a root
  path to reopen by name. The false rationale is corrected here; the same
  sentence in `docs/work-packages/WP-020.md` and the comment in
  `crates/fixtures/src/interlock.rs` are WP-020's to correct, and are recorded in
  the progress notes rather than edited from this package.*
  `rustix` is a safe wrapper, so no `unsafe` appears in this crate and SAFE-009
  needs no exception — the adapter crate F-03 contemplated is not required for
  the Unix half. The regression stages the audit's exact attack through the
  pre-open seam and compares the authorized handle's **inode** to the real
  fixture's, because the decoy holds identical bytes and content cannot tell
  them apart.

  **Windows is not closed.** The standard library exposes no safe
  handle-relative open, and the `NtCreateFile` route needs FFI that SAFE-009
  permits only in an adapter/FFI/helper crate. That platform still opens by
  pathname and the full finding stands there; the code says so at
  `RootDirectory::open_child`, and Tier 2 must not be enabled on Windows until
  it is closed.

- Containers are executable dependencies, and the scanner now sees them. It
  collected only `uses` keys, so a job container
  (`jobs.<id>.container.image`), the documented `container: <image>` shorthand, a
  service container (`jobs.<id>.services.<name>.image`), and a Docker action's
  `runs.image` were all invisible — every one of them code GitHub pulls and
  runs. An `image:` value must now be pinned by content digest, because a tag can
  be repointed exactly like a mutable action tag. `image: Dockerfile` is followed
  to that file's `FROM` lines, with multi-stage builds understood so an internal
  stage reference is not mistaken for a pull.

- A release-tag comment can no longer be borrowed from another step. The check
  searched the whole file for the reference and returned the first comment it
  found, so two steps sharing one SHA — one tagged, one bare — both passed on the
  tagged one's comment, and a reviewer reading the bare step saw no version at
  all. Every occurrence must now carry its own tag.

- A symlinked `action.yml` can no longer escape the repository. Containment was
  checked on the local action's *directory* and then inferred for its contents,
  so metadata linked to a file outside the tree would have been read and treated
  as first-party code. The metadata file is canonicalized and re-checked. A
  deletion sweep found this fix had **no test** — the check could be removed with
  everything still green, which was the audit's criticism of the traversal
  coverage repeated — so a Unix regression now exercises a real symlink.

- **WP-020's status table said containment was closed while the prose beneath it
  said reopened.** The corrections had gone into the deep prose and not the table
  a reader actually consults, which could have authorized Tier-2 work on a
  reopened precondition — the most dangerous kind of documentation drift this
  project can have. The table now carries increment 2c (not started, precondition
  1 reopened) and states that Tier 2 stays unavailable per platform until it
  lands. *Superseded 2026-07-30: 2c has since landed and closed precondition 1 on
  Unix; Windows is still open.* The token's "proves the operator ran the
  generator" wording is corrected
  in all three places it survived, and precondition 3 no longer cites SI-36 as a
  live blocker.

- WP-000 traceability cited three tests that no longer exist, having named the
  text-scanner suite the YAML parse replaced. Traceability naming absent evidence
  is worse than naming none. The rows now cite the tests that exist, including
  the container, Dockerfile, comment-binding and local-resolution regressions.

- Recorded, not hidden: PR #47 was a nominal WP-000 change that also edited
  WP-010, WP-020 and WP-030 documents. The ownership *inventory* passed because
  every path was claimed by someone; only reading caught it. Audit-driven
  corrections to another package's records are a legitimate need with no route in
  the current model, and the fix is a governance route under issue #39 — **not**
  widening WP-000's claims, which is the move that would make the checker
  complicit.

- Action discovery is a structural YAML parse, reversing a decision this project
  defended twice. Three text-based attempts were each defeated by valid YAML,
  and every one of them reported *success with one fewer reference* — silence
  shaped like a pass. The third attempt, a sweep for `owner/repo@ref` tokens
  described here as "syntax-independent" and "unbypassable", fell three ways at
  once: `"actions/checkout@v7"` hides the `@` behind a YAML escape no text
  search decodes; `docker://alpine:3.20` is a documented, mutable step reference
  containing no `@` at all; and a local action outside `.github/actions/` was
  never recursed into, so its own remote references went unread.

  `yaml-rust2` now parses each workflow and every `uses` mapping key in the tree
  is a reference with its value decoded — context-free, so a position GitHub
  adds later cannot be missed. Containers must be pinned by `@sha256:` digest.
  Local references are resolved wherever they live, must carry action metadata
  if they name a directory, must stay inside the repository, and are recursed
  into with a visited set that survives cycles. Unparseable YAML is a violation
  rather than a skip, and the release-tag comment survives as a separate textual
  auditability layer rather than as discovery.

  All three bypasses are permanent regressions. A deletion sweep also caught one
  of the *new* tests not being load-bearing: removing the container-digest
  branch still refused `docker://alpine:3.20`, because `is_pinned` reports it as
  "not pinned to a full commit SHA" — true, but it tells a reader to look for a
  git SHA on a Docker image. The test now asserts the container-specific
  guidance the branch exists to produce.

- **WP-020 precondition 1 is reopened.** *Superseded 2026-07-30 by increment 2c,
  which closed this form of the attack on Unix by opening a direct child relative
  to a held root-directory object. The Windows residual stands, and Tier 2 stays
  unavailable on every platform because no destructive suite exists. The entry is
  kept rather than deleted: the reasoning below is why 2c had to exist.*
  `O_NOFOLLOW` constrains only the final
  path component, which `open(2)` documents plainly and increment 2b overlooked.
  Renaming the fixture root aside and putting a symlink in its place redirects
  the open to an out-of-root file, and matching length, digest, type and link
  count then all pass — the same lesson as
  `object_verification_alone_cannot_prove_root_membership`, one directory up.
  Closing it needs a held root-directory object and an `openat`-style
  direct-child open; more `canonicalize` calls cannot. Tier 2 stays unavailable
  on every platform until that lands.

- SI-36 is **withdrawn the day it was filed.** SAFE-009 permits `unsafe` *only*
  in adapter, FFI, and helper crates, which forbids it in `crates/fixtures` and
  names the route in the same clause. Reading the omission of that crate from
  both lists as ambiguity was using the §0.2 process to convert an
  implementation-location constraint into permission by omission. Precondition 3
  is ordinary work with a known route, not a blocked decision.

- ADR-0007's justification is corrected. It said the token proves the operator
  ran the generator; a pure function of public source cannot prove that history,
  since anyone with the repository can compute the value. It proves only that
  the invocation presented the exact build-derived value — accident friction,
  which is what the decision actually rests on. The decision stands.

- Stale documentation corrected across the review set: `HANDOFF`'s execution
  order, `DECISION_NOTES`' disproved claims, the progress report, the audit
  response, README's WP-000/WP-020/WP-030 rows, WP-000's "only filesystem reads"
  sentence, and four traceability headers that named fewer increments or
  requirements than their own evidence tables contained. The token-mutation
  count said 12 where the table holds 26, and the ownership count said 100 where
  the tree holds 101 — that one is now left to the tool to print rather than
  restated in prose that goes stale on the next file added.

- WP-020 increment 2b: the object binding now starts at the open. Increment 2a
  bound every check to the handle but opened the target by path a second time,
  and the 2026-07-29 follow-up audit showed what lives in that gap: replace
  `root/name` with a symlink to an out-of-root file holding the fixture's exact
  bytes, and the handle is outside the fixture tree while every handle-based
  check — regular file, link count, length, digest — accepts it. Increment 2a
  had claimed a raced symlink was harmless *because* the object is verified
  after opening; `object_verification_alone_cannot_prove_root_membership` now
  records why that was wrong, by demonstrating the object checks accepting an
  out-of-root file. They establish fixture shape; containment is not a property
  of content, and a user's ordinary file may hold those bytes.

  The open refuses to leave the directory: `O_NOFOLLOW` on Unix, taken from
  `libc` because the value differs across Linux, macOS and the BSDs, and
  `FILE_FLAG_OPEN_REPARSE_POINT` on Windows. A test seam fires between
  canonicalization and open, so the race is scheduled rather than sampled —
  `a_symlink_swapped_in_before_open_is_refused` performs the audit's exact
  substitution, and a portable companion covers the seam on Windows, where
  creating a symlink needs a privilege CI cannot be relied on to hold. Removing
  the seam fails both by name.

  Still not claimed closed: the Windows hard-link vector. A hard link is not a
  reparse point, and stable Rust exposes link counts on Windows only behind an
  unstable feature — that is WP-020 precondition 3, and the reason precondition
  1 is narrowed rather than finished.

- The verified handle is handed over rewound. Hashing the contents left the
  cursor at EOF, so a consumer assuming a fresh file would have appended;
  the replace-after-authorization test having to seek explicitly was the smell.

- The action-pin gate no longer depends on recognising the `uses` key. Two
  audits in a row defeated the key-shaped reader with valid YAML it could not
  parse — a quoted key, then an anchored one, `&pin uses: actions/checkout@v7` —
  and each time it reported success having counted one *fewer* reference. A
  mutable tag was invisible rather than rejected, which is the worst failure
  mode a gate has: silence that looks like a pass. Discovery is now
  syntax-independent. An action reference must contain `owner/repo@ref`
  verbatim, whatever surrounds it, so a sweep for that shape finds every
  reference and anything the reader could not attribute to a `uses:` key is a
  violation. Anchors, tags, flow mappings, and every future spelling are
  covered by the same property, rather than by extending a subset one
  demonstrated bypass at a time. Verified against four spellings including both
  the audit's bypass and the tag variant it named as the same class.

- `verify-licenses` is semantic rather than lexical. It matched trimmed lines,
  so the follow-up audit moved the JSON property under `metadata`: the line
  still read `"license": "MIT OR Apache-2.0"` while the document's root
  `license` was `undefined`, and nine artefacts passed. `package.json` is now
  parsed as JSON with the property required at the root, Cargo licences come
  from `cargo metadata --locked --no-deps` (which resolves
  `license.workspace` inheritance the way the toolchain does), and a Cargo
  manifest that neither workspace includes is a violation because no gate
  resolves it. The blanket skip for directories named `generated` is gone.

- `supply-chain` no longer repairs the fuzz lock before auditing it. The
  preflight lived only in `fuzz()`, but `cargo deny` resolves the manifest to
  build its graph, so `supply-chain` silently restored a deleted
  `fuzz/Cargo.lock` entry and audited the repaired copy — the policy tool
  committing the fail-open shape it exists to catch, and leaving a later `fuzz`
  preflight nothing to refuse. `verify_fuzz_lock` is now shared and runs first
  in both entry points; the same mutation now refuses and the lock stays stale.

- `tokenSetVersion` is validated instead of merely present. It was only
  required to be non-empty, so `"not-a-version"` passed while WP-030 and the
  audit response both described parsing as "versioned" — a field nothing
  compares against is documentation. It is now compared against
  `REQUIRED_TOKEN_SET_VERSION` in `policy.rs`, alongside `specVersion`.

- WP-020 increment 2a: authorization holds the object it verified, not the
  name it found it under. The 2026-07-29 audit ranked this the most important
  precondition before any Tier-2 write: `Authorization` carried a
  `Vec<PathBuf>`, and a name can be rebound between verification and
  destructive use. It now carries open `File` handles — `fstat`, the hard-link
  count, the length, and every content byte are read through the handle, and
  the handle itself is what a destructive consumer receives, so renaming or
  swapping the path afterwards changes what the *name* means, never what the
  authorization holds. On Windows the handle's share mode refuses concurrent
  writes, deletion, and renames — through any name, hard links included — for
  as long as the authorization lives; the replace-after-authorization test
  asserts those refusals there and asserts write-through-handle reaches the
  verified object on POSIX. The proof is non-cloneable (a `compile_fail`
  doctest pins it) and consumed once.

  The first version of this fix repeated the defect it was written to end,
  and only planting regressions found it: downgrading the handle `fstat` to a
  by-path `stat` kept every test green, because the difference only shows
  during a race no unit test can stage. `verify_object` now takes no usable
  path, and its test deletes the path before verifying — handle-purity proven
  deterministically. Both planted regressions (`stat`-by-path,
  `fs::read`-by-path) fail that test by name.

  Deliberately not done here: platform no-follow open flags (hardcoding
  `O_NOFOLLOW` values without `libc` is its own defect factory; the by-name
  symlink refusal stays as hygiene and post-open object verification makes a
  raced symlink harmless), and the independent-token decision, which needs an
  entropy source and is a dependency-policy change — still recorded open in
  `docs/work-packages/WP-020.md`.

- The gate can no longer repair the lockfile it claims to enforce. The
  2026-07-29 audit deleted a package entry from `Cargo.lock` and ran
  `cargo xtask ci`: Cargo silently regenerated the entry while building `xtask`
  itself, and all 160 tests passed against a lockfile the repository had never
  committed — the internal `--locked` flags bind only once the binary is
  built. `--locked` now sits in the `xtask` alias, the boundary that builds the
  gate; the same mutation now refuses with "cannot update the lock file". A
  Tier-1 test fails by name if the alias loses the flag.

- `cargo xtask verify-actions` no longer goes blind on valid YAML. The audit
  rewrote one pinned step as `"uses": actions/checkout@v7` — the same YAML key,
  and GitHub executes it — and the scanner reported *success with one fewer
  reference*: the mutable tag was invisible rather than rejected. The scanner
  now enforces a deliberately small YAML subset and **refuses what it cannot
  positively read**: quoted keys are recognized and checked, while flow
  mappings, block scalars, aliases, anchors, escaped quoted keys, explicit-key
  syntax, and values continuing on the next line are each a named violation.
  Local composite actions under `.github/actions/` are scanned too — exempting
  `./` references is safe only if their own metadata is read. What remains
  manual is recorded: nothing verifies a tag comment resolves to its pinned
  SHA, and that is a review obligation, not an automated check.

- The fuzz crate's dependency graph is no longer outside every gate.
  `fuzz/Cargo.lock` was gitignored and the crate is excluded from the
  workspace, so every fresh CI run resolved `libfuzzer-sys` and `arbitrary` to
  whatever the registry served that day and ran their build scripts — on the
  job that executes hostile-byte parser tests, checked by no advisory, licence,
  or source policy. The lock is committed; `cargo xtask fuzz` refuses a stale
  lock before the nightly toolchain is even involved; `cargo xtask
  supply-chain` checks the fuzz graph as a second graph under the same
  `deny.toml` (which required allowing NCSA — `libfuzzer-sys` is
  `(MIT OR Apache-2.0) AND NCSA`, so the permissive NCSA arm is mandatory, and
  the addition is commented in `deny.toml`); and a `/fuzz` Dependabot entry
  updates what nothing previously watched.

- `cargo xtask verify-licenses` closes the recorded WP-000 gap: it walks every
  `Cargo.toml` and `package.json`, fails unless each declares
  `MIT OR Apache-2.0` and both licence texts exist, and runs inside
  `cargo xtask ci`. Previously `fuzz/Cargo.toml` and
  `packages/canonical/package.json` could lose their licence keys with CI
  green.

- WP-000 is reclassified from Complete to in progress. Section 12 defines done
  as generated traceability showing a package's evidence, and
  `docs/traceability/` is hand-maintained; the audit also demonstrated two
  fail-open evidence paths in what Complete claimed to cover. The README row
  now says what is delivered and what is not, and the hosted-runner deviation
  from SEC-010's builder-image digest rule is documented with its residual risk
  in `docs/quality/dependency-policy.md` instead of being silently absorbed.

- WP-030 increment 1a: the accessibility harness no longer takes its standards
  from the file it audits. The 2026-07-29 project audit demonstrated two live
  bypasses through the whole Tier-1 gate: lowering the token file's own `text`
  threshold from 4.5 to 3.0 let normal-size text pass at **3.33:1**, and
  deleting `entity.container` from every theme, pairing and channel table at
  once passed with six fewer checks — a coordinated omission indistinguishable
  from a smaller product, while UI-003 requires containers to be represented.
  Both are the self-consistency failure `AGENTS.md` records for the canonical
  vectors, committed inside the harness written to enforce that rule on
  colours: increment 1's mutation table mutated colours thoroughly and never
  mutated the policy.

  The WCAG floors, the colour-separation floor, the required themes and the
  full UI-003/PLAN-004/UI-011 role vocabulary now live in
  `crates/tokens/src/policy.rs`, outside the audited file. The JSON restates
  the floors for a front end to read, and the audit requires the restatement to
  agree exactly — a lowered value is a finding, not a new setting. Twelve
  mutations were added (threshold lowering with and without a hidden colour,
  threshold removal, role deletion, pairing removal, risk-pair removal, role
  invention, version mismatches); re-running the audit's own reproductions now
  yields 3 and 2 findings where both yielded none. The reader is genuinely
  strict too: `deny_unknown_fields` throughout, so a misspelled
  `nonColorChannels` key can no longer silently disable the UI-007 check.

- `docs/quality/test-tiers.md` overstated the SAFE-007 token: it said the token
  "cannot be known without having generated that fixture set", but the token is
  a pure function of the source, identical on every machine building the same
  commit. The file now carries the honest account `docs/work-packages/WP-020.md`
  always had — the factor is weak, three factors are effectively two, and the
  interlock's strength rests on targets byte-equalling generated images.

- The TypeScript encoder had no `default` arm, so an unrecognized value kind fell
  through the switch, `encode` returned zero bytes, and `hash` published SHA-256
  of the empty string as a well-formed digest over an artifact with no encoding.
  Rust cannot reach this, because its `match` is exhaustive at compile time.
  Payload runtime types are now checked too: `TextEncoder` coerces a non-string
  rather than failing, and `Uint8Array.from` truncates modulo 256. `fromHex`
  refused nothing outside hex, where `Number.parseInt` yields NaN and stores as
  0, so two distinct textual digests decoded to identical bytes.

- Three signature fixtures wrote fields at the wrong offsets, found by an audit
  of the project review. The mdraid 0.90 set UUID occupied `utime`, `state` and
  `active_disks` rather than words 13 to 15, leaving the array identity three
  quarters zero — `blkid` reported it as `fb2871eb-0000-0000-0000-000000000000`.
  LUKS2 wrote its checksum algorithm and UUID inside the 48-byte label field,
  leaving the fixture with no UUID at all. ext4 declared 8 MiB of blocks on a
  4 MiB device, having reused a sector count as a block count. Each correction is
  confirmed against `libblkid` rather than against the struct definition.

- The TypeScript encoder authenticated a forged boolean as the **opposite**
  logical value. The `bool` arm used JavaScript truthiness, so a runtime-forged
  `{ kind: 'bool', value: 'false' }` encoded as `f5` — canonical `true` — and
  `hash` published a digest over the other value, on the MODEL-005 and SEC-001
  authorization boundary. TypeScript types do not protect an object that arrived
  as JSON, over RPC, from a plugin, or as `unknown`, which is why `text` and
  `bytes` already validated at runtime; the reasoning had simply not been carried
  to the rest. Every variant now validates, including map keys — where
  `requireWellFormed` iterated `.length`, `undefined` on a number, so its loop
  never ran and `utf8` then coerced `1` to `"1"`, silently turning a map keyed by
  a forged number into one keyed by text. Rust was unaffected: its `match` is
  exhaustive over a real enum.

  An adversarial pass on that fix found the guards checked fields while a payload
  can lie *between two reads*: `kind` was read twice, containers declared a count
  and then wrote a body from a second read, `bytes` trusted a `Symbol.iterator`
  that `Uint8Array.from` truncated modulo 256, and `instanceof` let a
  prototype-only fake through to a native `TypeError`. Each field is now read
  exactly once and containers are snapshotted before being measured. The tests
  were vacuous the same way — the `array` case used a string, which has a
  `.length`, so the guard never ran and deleting it left the suite green. Every
  case now names the phrase its refusal must contain.
- Raw-byte hashing is no longer a way around strict decode. Both languages
  exported a function that hashed whatever it was handed, documented as "use this
  only for bytes produced by `encode` or accepted by `decode`" — an instruction,
  not a guarantee. `hash_canonical_bytes` and `hashCanonicalBytes` are replaced by
  `hash_encoded` / `hashEncoded`, which decode first, so canonicality is proven
  rather than asserted. The TypeScript version validated a *prefix* and hashed a
  *buffer* until the same pass caught it: `decode` walks the array through its
  `length` property while `crypto.subtle.digest` reads the underlying buffer. No
  digest changed; both languages still reproduce every recorded `sha256`.
- Two fail-open edges and a flaky test harness, found by a progress review. The
  sharpest is one the prober increment introduced in the module the evidence
  increment wrote about: both prober parsers **discarded what they could not
  read**, so an unreadable row was not an "unexpected signature" the comparison
  would report — it was no observation at all, and on the fixture whose
  expectation is *nothing*, an entirely changed output shape parsed as empty and
  passed. The module documentation claimed the signature set was compared in both
  directions. Both parsers now return `Result` and refuse a missing `=`, an empty
  or repeated key, a bad offset, a typeless row, a misplaced header, and a
  repeated row; `probe_output` no longer uses `from_utf8_lossy`.
- Fixture-directory pruning inferred ownership from a filename:
  `root.join(MANIFEST_FILE).is_file()` establishes nothing about who wrote the
  file and follows a symlink besides, so any directory holding an unrelated file
  or link named `MANIFEST` was treated as ours and could lose its other regular
  files. Ownership is now computed — a regular file reached without following a
  link, parsing as one of our manifests, with the token recomputed from its own
  entries. Every failure is a refusal to prune.
- Test sandboxes used fixed paths and deleted them at setup and drop, so two
  concurrent `cargo test` runs of the fixtures crate erased each other's trees —
  in the suite that gates destructive execution. Names now carry the process id
  and a per-process counter.
- Two documented claims that had outrun their code, corrected in opposite
  directions. `corrupt_primary_header_crc` still described itself as producing
  ADR-C3's `Indeterminate` state, contradicting `write_conflicting_backup` in the
  same file, which had already been corrected; the layout test repeated it. Both
  now say *recoverable*. And the review response's note that
  `authorization_cannot_be_forged_outside_this_module` "reportedly stays green
  with `verify_target`'s body short-circuited" — read as evidence the interlock
  suite was blind to that mutation — is false. It was run: **ten tests fail**,
  covering traversal, subdirectory copies, modified bytes, wrong names, missing
  targets and mixed requests. The named test does stay green, because it asserts
  a compile-time property rather than target verification. "Reportedly" marked a
  claim that had never been executed, in a document written about that exact
  failure.
- `every_generated_fixture_authorizes` asserted `targets.len() >= 8`. A floor
  lets catalogue entries be deleted silently while the test still reads as
  coverage; it is now an equality against the catalogue's own length.
- `gpt-missing-backup-512.img` had a backup. It zeroed the last sector only,
  leaving 16 KiB of byte-identical backup entry array at LBAs 8159 to 8190 —
  which any recovery tool that scans rather than seeking to the last LBA would
  find, on a fixture named for having no backup. The whole backup copy is now
  erased.
- `gpt-basic-512.img` and `gpt-basic-4kn.img` shared a disk GUID, both deriving
  it from the literal `"gpt-basic"`. Two different media with one identity is a
  manufactured instance of the collision SI-27 is trying to reason about. Found
  by a new catalogue-wide identity check that no single-image claim could see.

- A subdirectory bypass in the SAFE-007 interlock, introduced by the fix for the
  forged-manifest defect and missed by its own new tests. Containment was a path
  prefix while the name came from `file_name()`, so a byte-identical copy at
  `<root>/sub/blank-512.img` passed the root, name, length and digest checks at
  once. The resolved path must now equal the exact location that fixture is
  generated at.

- The TypeScript encoder could emit bytes its own decoder rejects, violating
  `schemas/canonical-encoding.md` §6.1. `TextEncoder` substitutes U+FFFD for an
  unpaired surrogate rather than failing, so two distinct values encoded
  identically and `encode` was not injective; a map holding both keys emitted a
  declared size of two with byte-identical keys, which §3 makes invalid. The
  encoder now refuses an ill-formed string instead of repairing it, and validates
  map keys before sorting so the refusal cannot depend on insertion order. Rust
  needed no change — `String` is validated UTF-8 — which is the point: the two
  implementations had disagreed about what was *encodable*. Reachable without an
  attacker, since NTFS permits unpaired surrogates in volume labels and INV-008
  requires such structures be represented rather than discarded.

- Windows could not pass `cargo fmt --check`. Git for Windows sets
  `core.autocrlf=true` in its system configuration by default, and the
  GitHub-hosted `windows-*` runner images do not override it, so checkout
  produced CRLF working-tree files that `newline_style = "Unix"` rejects.
  `.gitattributes` now pins LF in every working tree.


- WP-010 increment 4: `cargo-fuzz` targets for the canonical codec (Section
  11.4), plus `crates/domain/tests/canonicality.rs`, which asserts the same
  canonicality property on stable over every single-bit flip, truncation, and
  boundary substitution of every known-good encoding, and every one- and
  two-byte input exhaustively. `cargo xtask fuzz` runs the smoke pass and gates
  CI as its own job. `fuzz/` is excluded from the workspace and is the only
  place a nightly toolchain is permitted; it is pinned by exact date.
