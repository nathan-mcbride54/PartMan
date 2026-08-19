# The capability seam — WP-L100 increment 5, 2026-08-19

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Written before the first line of code, per house convention;
> its base is `c97adfb` (main), spec 17.4.0, both forges at that SHA.

**Directive:** Nate — "start increment 5, plan doc first".
**What it delivers:** WP-L100's increment 5 as its record states it
(`WP-L100.md`, "5. The capability seam and the record"): `RuntimeFacts`
produced in CAP-004's shape for Linux — tool presence and version state
under SAFE-004's structured-argv, trusted-absolute-path discipline, and
the Section 9 floor determination for the Debian/Ubuntu and Arch tiers —
handed to WP-050's engine; INV-006 held structurally with a test pinning
the absence of every mount, unlock and repair-tool call; INV-007's
discovery material offered through the field allowlist WP-035's bundle
applies; the package record sweep.

## 0. What is already decided, so this plan decides nothing

- **The consumer's shape is delivered and fixed** (`crates/capability/src/engine.rs:78-135`,
  WP-050 increment 2): `RuntimeFacts { tools: Vec<ToolFact { tool, state }>,
  platform: PlatformFact }`, judged per operation; `ToolState` is
  `PresentInRange | Missing | OutOfRange` (ACC-009's two failure classes);
  `PlatformFact` is `MeetsFloor | BelowFloor` (Section 9: "the capability
  engine may narrow further at runtime (CAP-004); it may never widen below
  these floors"). The engine "performs no I/O to gather these and cannot";
  "producing them is WP-035's doctor today and the platform adapters'
  work (WP-W100/WP-L100/WP-M100) tomorrow" (`WP-050.md:86`).
- **The launch discipline is delivered once, in WP-035's doctor**
  (`apps/cli/src/doctor.rs`): a compiled absolute-path roster as the
  SAFE-004 allow-list, `--version` probes through an injected
  `ToolLauncher` seam (cleared environment plus `LC_ALL=C`, bounded output,
  a time limit, no shell, no `PATH`), a hand parser for util-linux's
  unstructured banner, and two carve-outs stated rather than implied:
  executable identity is not verified beyond the trusted absolute path
  ("Identity verification arrives with the packages that execute tools
  against storage, where SAFE-004 demands it"), and the range→`blocked`
  mapping "belongs to WP-050's capability engine, not here". Its roster is
  `blkid` and `wipefs` against a *tested range* — the fixtures prober's
  util-linux 2.41 family — which is expressly the repository's own toolchain
  expectation, not a product floor.
- **The floors store is empty by decision** (`docs/capabilities/format.md`
  §2, `tool-version-floors.json`): "no storage tool is invoked anywhere in
  the product yet … a floor for a tool nobody calls would be an assertion
  nobody can test. A tool's floor arrives with the first package that
  invokes it, under review, with its basis stated." ACC-009 gates the
  **write** step ("the planner cannot create the affected write step").
- **This adapter launches nothing and opens no device** — a structural
  Tier-1 guard over every shipped module
  (`the_adapter_opens_no_device_node_and_launches_no_process`,
  `tests.rs:620-631`, needles `std::process`, `Command::new`, `std::env`,
  `/dev/`), which is what keeps the whole suite unprivileged; and its
  interfaces are closed at three (sysfs, the udev database, procfs), each
  entered by an observability row and never by documentation.
- **The evidence rule** (`WP-L100.md:210-224`): a representational claim
  about what a real Linux host exposes rests on a capture from a real host;
  where none exists, the increment says so and delivers the fail-closed
  answer.
- **Section 9's Linux rows** (`AGENT_BUILD_SPEC.md:779-791`): Debian/Ubuntu
  — "Debian 12 / Ubuntu 22.04 LTS; kernel ≥ 5.15; UDisks2 ≥ 2.9"; Arch —
  "Current rolling … tool-version-gated". Floors change only via ADR.
- **LIN-001's UDisks2 route decision is deferred by the package record**
  ("Beyond these five: LIN-001's UDisks2 route decision").

## 1. Measured at `c97adfb`

| Claim the increment text makes | What the tree and the record hold |
| --- | --- |
| "tool presence and version state" | Delivered in WP-035's doctor for two tools, through a launcher seam, against a *tested range* that is not a floor. No `RuntimeFacts` producer exists anywhere; the engine's tests hand-build them. |
| "under SAFE-004's discipline" | Delivered in the doctor in every clause but identity verification, recorded as a carve-out. Nothing in `crates/adapter-linux` launches, by structural test. |
| "the Section 9 floor determination for the Debian/Ubuntu and Arch tiers" | **No producer, and only part of the input is measured.** The record's environment blocks capture `uname -r` and `grep ^PRETTY_NAME= /etc/os-release` (Phase 0 preflight, `observability.md:3088-3089`; the acceptance guests: Ubuntu 22.04.5, kernel `5.15.0-186-generic`) — the *command* outputs, not the shape of `/etc/os-release`'s `ID`/`VERSION_ID`/`ID_LIKE` keys nor of `/proc/sys/kernel/osrelease` as a client file read. **No Arch host or guest appears anywhere in the record.** |
| the UDisks2 ≥ 2.9 conjunct | **Unmeasurable by this contract and unmet where measured.** No file under the three interfaces carries UDisks2's version; every measured Linux guest had `udisks2` purged, absent or inactive (`observability.md:3759`, `:3851`, `:4114`, `:4449`), which is where every WP-020 acceptance since 2026-08-03 has passed. |
| "handed to WP-050's engine" | The engine takes `RuntimeFacts` by value; no adapter→engine wiring exists and none is required by WP-050 (its consumer seams are documented, not linked). `apps/cli` does not depend on `partman-adapter-linux`; the doctor's report types are the CLI's. |
| "INV-006 held structurally, with a test pinning the absence of every mount, unlock, and repair-tool call" | Held today by the no-process/no-device guard; no test names mount, unlock or repair *tools* as such. |
| "INV-007's discovery material offered through the field allowlist WP-035's bundle applies" | WP-035's bundle applies a closed field enum (`WP-035.md:122`); the adapter's observation surface is already the domain's `PropertyObservations`; nothing here is offered to the bundle yet, and the bundle is WP-035's to extend. |
| the tools any read-only operation needs | **None.** Every operation the adapter serves today is a source-class read of sysfs/udev/procfs files; INV-006 forbids repair tools during discovery; ACC-009 gates write steps. |

## 2. Three findings, before the shape

**F1 — the tool half of increment 5 is a seam and a truthful empty roster,
not a launcher.** No read-only operation needs a tool, the floors store
says a floor arrives with the first package that *invokes* the tool, and
the doctor's roster is the repository's toolchain. So the adapter's honest
tool facts for M1 are `tools: []` for every operation it serves — stated
per operation, pinned by a test that fails the moment a roster entry
appears against a read-only operation — plus the *mapping* from a
doctor-shaped report to `ToolState`, so that when WP-L110 invokes storage
tools its facts arrive through the seam this increment fixes. Moving the
launcher itself into this crate would break its SAFE-002 structural claim
and its Tier-1 purity for nothing that any operation needs; the identity
clause SAFE-004 still owes is WP-L110's, "where SAFE-004 demands it".

**F2 — the floor determination has three conjuncts and this contract can
read two, on rows that do not yet exist.** Distribution and version are
in `/etc/os-release` (`ID`, `VERSION_ID`, `ID_LIKE`) — a **fourth
interface**, which under this package's own rule enters only by a row;
the kernel is `/proc/sys/kernel/osrelease` — a fourth *path* on the
already-entered procfs interface, likewise unmeasured as a client file
read. UDisks2's version is behind D-Bus or a `udisksctl --version` launch,
neither of which this contract has, on a route LIN-001 has not decided,
for a daemon absent from every measured guest. And `PlatformFact` has no
undetermined arm: mapping "cannot determine" to `BelowFloor` is
fail-closed but would report every operation on every measured host as
`blocked` for the UDisks2 conjunct alone — a determination this adapter
would be *authoring* about a component it never consulted.

**F3 — Arch is unmeasurable today.** "Current rolling, tool-version-gated"
makes the distribution conjunct trivially met once `ID=arch` is read, but
no Arch host or guest exists in the record, so even the `os-release` shape
on Arch is unmeasured. The Proxmox apparatus can host an Arch cloud image
(a `qcow2` from `geo.mirror.pkgbuild.com/images/`), which is a new
apparatus decision — the pinned-digest discipline the jammy image has, for
a second image — and the decision owner's.

## 3. The shape

Split on evidence, as increment 4 was:

**5a — the seam and the truthful roster (Rust; owes a sitting; needs no
new row).** `crates/adapter-linux/src/runtime.rs`:

- `required_tools(operation) -> &'static [ToolRequirement]` — empty for
  every operation this adapter serves; a test enumerates every
  `Operation` and asserts the source-class ones (`Detect`, `Read`) and
  every operation the adapter answers today carry no requirement, and a
  mutation that adds `blkid` to `Detect` is killed. INV-006's test: the
  requirement table names no mount, unlock or repair tool, and the
  no-process guard stands.
- `ToolProbe` — the adapter's own small input type for one tool's probe
  result (present at a trusted absolute path with a parsed version /
  absent / probe failed / version unparsed), and `tool_state(probe,
  floor) -> ToolState` — the ACC-009 mapping the doctor deliberately left
  to "the capability engine": present and ≥ floor → `PresentInRange`;
  absent → `Missing`; below floor, unparsed, or failed → `OutOfRange`,
  the fail-closed direction, with the reason kept for remediation text.
  A floor is a `docs/capabilities/tool-version-floors.json` row read by
  the caller, never authored here; with the store empty the function is
  exercised only by tests, and says so.
- `runtime_facts(operation, probes, floor) -> RuntimeFacts` — the
  assembly, pure. Nothing launches; the doctor (or WP-L110) supplies
  probes.

**5b — the floor determination (Rust; owes a sitting; waits on rows and
one WP-050 addition).**

- **Rows first (WP-035, filed by WP-L100 in the DR bracket): DR16** —
  `/etc/os-release` as a client file read on the jammy guest: readable,
  the `ID`/`VERSION_ID`/`ID_LIKE`/`PRETTY_NAME` keys and their byte
  shape (quoting, trailing newline), double-captured, byte-stable across
  a reboot; **DR17** — `/proc/sys/kernel/osrelease` as a client file
  read beside `uname -r`, same terms; **DR18 (Arch, conditional on the
  apparatus)** — the same two files on an Arch cloud-image guest, plus
  whether `udisks2` is present by default. If the decision owner declines
  the Arch apparatus, the Arch arm ships as **undetermined by construction**
  and says so.
- **WP-050 first, consumer-first (the cross-package rule):** add
  `PlatformFact::Undetermined { reason }` to the engine — the engine
  treats it exactly as `BelowFloor` for status (`blocked`) with a
  remediation that names the undetermined conjunct, so no client-side
  guess reaches an answer and no host is called below a floor it was
  never measured against. An addition, valid under both regimes; the
  adapter adopts it after it lands.
- Then `platform_floor(source, etc_root, procfs_root, tier_evidence)` in
  the adapter: distribution and version from `os-release` (a positively
  determined `ID`/`VERSION_ID` against the Section 9 row, verbatim
  compare on the measured byte shape, no version arithmetic beyond what
  the row states — Debian `12`, Ubuntu `22.04`, and for Arch `ID=arch`
  alone), kernel from `osrelease` (parsed `major.minor` ≥ `5.15`, the
  parse refusing rather than guessing), and the **UDisks2 conjunct
  `Undetermined`** with the reason stated — the contract has no source
  for it and LIN-001's route is undecided — so the composite is
  `Undetermined` on every host until that route lands and a source is
  measured. That is the honest answer, and it is the same answer the
  measured acceptance environments would get: they run without
  `udisks2`.

**Not in this increment:** the doctor's relocation into the adapter; any
launch from the adapter; a floor for `blkid`/`wipefs` (nobody invokes
them against storage); the UDisks2 route (LIN-001, its own decision);
INV-007's bundle field (WP-035's enum, extended when the bundle first
carries adapter material — named here as an obligation on WP-035, not
delivered by this package).

## 4. Sequencing

1. This plan (WP-000).
2. **5a** — one PR, Rust, r47 at its head, named in the body before merge.
   Independent of every row.
3. The DR16–DR18 filing (WP-L100) and WP-035 preregistration; the Arch
   apparatus decision (§7.1) before DR18 is preregistered or dropped.
4. The WP-050 addition (`PlatformFact::Undetermined`) — its own PR under
   WP-050's grant, consumer-first; Rust, owes a sitting (or rides 5b's
   arc head, the r11 precedent, if the two land back-to-back).
5. The DR16–DR18 sitting on the DR apparatus (VMID 9476 next); the record.
6. **5b** — the floor determination on the rows; r-next at its head.
7. Package record: WP-L100 delivery row 5, README, CHANGELOG, traceability;
   the WP-L100 assignment's "Beyond these five" list gains the INV-007
   bundle field as WP-035's obligation.

## 5. Pricing

No spec text: Section 9 is untouched, and reporting a conjunct as
undetermined narrows nothing (the engine may narrow at runtime and may not
widen — an undetermined floor is not a widening). No ADR: no floor is set,
no route decided. One WP-050 type addition (minor, additive). Two Rust
slices under WP-L100 (each owes a sitting) and one under WP-050. Two or
three DR cells and possibly one new guest image on the apparatus.

## 6. What would change this plan

- A decided text under which a Section 9 conjunct the client cannot read
  may be *assumed met* — none found; Section 9's own sentence runs the
  other way ("may never widen below these floors").
- A row showing `udisks2` present and its version client-readable through
  a file under one of the three interfaces — then the conjunct is
  determinable and 5b measures it rather than reporting it undetermined.
- LIN-001's route decision landing first — it may make UDisks2 a
  consulted interface, in which case its version enters through that
  route and this increment's Undetermined arm closes on the same day.
- The decision owner wanting the doctor's launcher moved into the adapter
  now — then 5a's structural claim changes, its Tier-1 tests need a fake
  launcher (the doctor already has one), and SAFE-004's identity clause
  should land with the move rather than stay carved out.

## 7. Open for the decision owner

1. **The Arch apparatus.** Add an Arch cloud image (pinned digest, its own
   `01-host-create-vm` variant) to the Proxmox apparatus for DR18 — or
   ship the Arch tier's floor arm as undetermined-by-construction and say
   so in `fields.md`? The plan prefers the image: it is a one-time cost,
   and "Arch: current rolling" is otherwise a floor row nobody has ever
   measured against.
2. **`PlatformFact::Undetermined` in WP-050** — an additive variant, or
   would you rather the adapter map undetermined to `BelowFloor` and carry
   the reason only in a remediation? The plan prefers the variant: a
   verdict the adapter authored about a daemon it never consulted is the
   thing this package's rules exist to prevent, and `BelowFloor` would be
   read as measured.
3. **Whether to file the UDisks2 conjunct as a §1.11 item.** Section 9's
   floor names UDisks2 ≥ 2.9; LIN-001 says "use UDisks2 for
   discovery/authorization"; every measured acceptance environment runs
   without it and the read-only product works. That is a floor-versus-
   route tension, not a requirement-versus-requirement conflict, so the
   plan does not file it — it records it here and in 5b's docs, and
   LIN-001's route decision is where it resolves.
