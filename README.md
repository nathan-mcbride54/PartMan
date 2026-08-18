# PartMan

PartMan is a safety-first, cross-platform disk partition manager defined by
`AGENT_BUILD_SPEC.md` 17.3.0. The intended product is a dark-first Tauri desktop
application plus a scriptable CLI, backed by a shared Rust domain, planner,
validator, journal, image engine, and per-platform privileged helpers.

## Current status

**Not a usable partition manager, and must not be represented as one.** Nothing
here discovers, plans, or mutates storage. There is no GUI, no planner, and no
privileged helper. A read-only CLI exists — argument parsing, a documented
exit-code contract, a schema-versioned JSON envelope, a typed refusal
vocabulary, a dependency doctor, technology facts, and adapter-attributed
observation records over replayed regular files — and, since WP-035
increments 8 and 9, two real platform surfaces: on Linux and macOS,
`partman inspect` without `--replay` enumerates the host's attached whole
devices as raw identifier strings labelled by the interface that reported
them, computing no strength, table state, hash, verdict, or plan from any
of them. On Windows the same invocation answers with a typed
not-implemented statement naming WP-W100 and the recorded decision that
defers its adapter, because printing a plausible empty machine would be a
fake success path. ADR-0009's
off-main Slint 1.17.1 candidate was rejected and closed without merge; only
its reproducible evidence record lands here.

What does exist:

- Repository foundation: pinned toolchain, three-OS CI, the `xtask` task
  runner, dependency and lint policy (WP-000).
- The `pce/1` canonical encoding: a byte-exact encoder, a strict validating
  decoder, and SHA-256 hashing, implemented in Rust (`crates/domain`) and
  TypeScript (`packages/canonical`) and proven to agree (WP-010, increments 1
  and 2). Schema-declared sets now have a separate cross-language producer and
  validator: full element encodings sort by unsigned byte order, duplicates
  fail closed, and element sort keys inherit their enclosing depth (increment
  2a, MODEL-006). Ordinary arrays and all existing `pce/1` hashes are unchanged.
- The design tokens and the accessibility harness that computes WCAG 2.2 AA
  contrast, redundant non-colour channels, and colour-vision separation from
  them (WP-030 increment 1). There is still no user interface for them to
  style.
- A strict 41-row ADR-0009 Slint feasibility report. It records one pass, hard
  failures at `G-CFG-08` and `G-SC-01`, and 38 inconclusive rows, together with
  immutable candidate/artifact identities and explicit accessibility/platform
  gaps. No Slint runtime or renderer dependency is present on main.
- The `partman` CLI chassis (WP-035 increments 1–4 and 7–9): structured argv that
  owns the non-Unicode seam as a typed refusal instead of a panic, exit codes
  pinned by test, an ANSI-free schema-versioned envelope
  (`partman.cli.envelope/0`, provisional within major version 0), and the
  typed refusal vocabulary. Its shipped dependency closure is empty and its
  output type carries a Tier-1 compile-time non-`Hash` proof — both guards' exact
  reach is stated in the crate, and domain payloads are absent from every
  surface rather than emitted unversioned. `export-diagnostics` emits this
  build's identity and surface states through a deny-by-default field
  allowlist that is the builder's type rather than a filter; every field is
  compile-time data, the missing discovery evidence is an in-band typed
  refusal, and redaction tests — an exact-field pin, a byte-pinned human
  rendering, an environment-value tripwire, and an env-read source guard —
  gate the tier. `doctor` reports roster tools at compiled absolute paths —
  present, version, in or out of the recorded tested range — as facts with
  provenance under SAFE-004-derived launch controls. Executable identity is
  not verified beyond the absolute path, and capability mapping is deferred to
  WP-050; an empty platform roster is carried as a typed statement. `facts`
  ships FS-007's inputs, five
  immutable technology limits each with its basis, mechanically refused the
  CAP-003 status vocabulary that belongs to WP-050. `inspect` answers with
  adapter-attributed observation records: `--replay` runs the fixture-replay
  adapter over one regular file — refused unread if it is anything else,
  with fstat through the opened handle as the authority — reporting bytes
  as hex that no code classifies, with ADR-C4's outcome vocabulary as the
  ADR wrote it (observed, with absence as a value; unavailable; failed) and
  the standing gated list (SI-28, and — standing decisions rather than open
  questions — ADR-0014 since spec 8.0.0 and ADR-0011 since 4.3.0) carried
  in-band in every answer, refusals included. Increment 7 publishes the INV-003 reach
  declaration on all three platforms — one answer per state INV-003 lists,
  derived from the contract this package itself reads rather than from any
  device, with every negative cell present rather than omitted. Increment 8
  delivers the Linux enumeration adapter behind `inspect`: whole devices
  through sysfs file reads with no subprocess, reporting size, block sizes,
  vendor, model, serial, and WWN as raw strings labelled by reporting
  interface, with udev values carrying an in-band caveat that they are what
  root's `udevd` cached at device-add time; over-limit and non-UTF-8
  attribute values refuse rather than truncate or substitute, padding is
  preserved rather than trimmed into a false absence, and a partition-filter
  read error fails closed rather than promoting a partition into the device
  list. Increment 9 delivers the macOS enumeration adapter: `diskutil list
  -plist` once and `info -plist` once per whole device, launched at the
  compiled absolute path through the SAFE-004 launcher seam with stated
  output bounds, parsed by a bounded in-crate XML plist reader that refuses
  every construct outside its stated grammar — no dependency taken, the
  empty-closure guard intact — reporting twelve identity keys as raw
  interface-labelled strings, with `Content`, UUID, and APFS fields
  deliberately unread because they are the register's material. A nonzero
  diskutil exit is a failure whose output is never parsed; a
  present-but-container value is a typed failure rather than a flattening;
  a missing key is a positively determined absence. The reader's Section
  11.4 fuzz target lands separately under `fuzz/`'s own ownership
  (WP-010, with the WP-000 xtask row), recorded here as in flight rather
  than silently absent.

The domain crate performs no I/O and launches no process. Tier 1 retains its
host-safe boundary. The two runnable higher-tier selectors are WP-020's named
non-destructive Linux-VM loop check and WP-035's SI-35 measurement instrument
built on it; every generic destructive Tier-2 request and every Tier-3 request
still fails closed, and neither selector registers a destructive suite.

## Safe local gate

```text
cargo xtask ci
```

Verifies the pinned toolchain, GitHub Action digest pinning, formatting,
linting, and Tier-1 Rust tests. Tier 1 never requires elevation and contains no
destructive storage operations.

```text
cargo xtask cross-language
```

Proves that Rust and TypeScript produce identical hashes for identical content
(MODEL-005). It needs a Node toolchain, so it is deliberately **not** part of
`cargo xtask ci`; CI runs it as its own required job so the proof cannot be
silently skipped.

```text
cargo xtask test --tier 1
```

Two higher-tier selectors are registered, and no others:

```text
cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only
cargo xtask test --tier 2 --profile destructive --acceptance si35-loop-capture
```

The second is WP-035's SI-35 measurement instrument. It runs the same
crate-owned hold-open sessions under the same environmental gate, launches
only predeclared read-only probers at compiled absolute paths, and emits raw
records for an unprivileged projection half that refuses elevation. It
registers no destructive suite either. The paragraph below describes the
first; both share its privilege, isolation, and fail-closed discipline.

The first runs only with explicit privilege in a disposable non-WSL Linux VM and only
after SAFE-001, SAFE-002, and every SAFE-007 factor authorize the generated
fixture backing objects. The acceptance opens the backing, loop-control, and
loop-device descriptors `O_RDWR` for its mapping-control operations,
sets `LO_FLAGS_READ_ONLY`, probes in-process through the held loop descriptor,
issues no logical write, discard, or zero operation, and accepts no observation
unless each initial held-file hash matches the compiled fixture catalogue, its
sampled identity and configuration checks pass, detach and partition teardown
are confirmed, and both fixture hashes remain unchanged. External run evidence
must exclude every other actor able to modify either fixture and every other
actor able to administer or rebind loop devices. Ordinary kernel/udev read/open
discovery is allowed and handled by bounded cleanup, but a loop-configuration
`EBUSY` refuses immediately because isolated loop state was not established.
Hash and status sampling cannot defeat ABA changes entirely between samples, so
the result is not a continuous-binding guarantee. The disposable VM bounds
consequences but does not prove those exclusions. Linux may `fsync` inside the
mapping ioctls and write back already-dirty data or metadata, so this is not a
zero-physical-write claim. No external storage tool is launched. Generic
destructive Tier 2 and all Tier 3 still refuse: since WP-020 increment 2g the
fact that backs the refusal is a compiled destructive-suite registry, and a
generic request selects no suite from it. A pass over no suite remains
forbidden.

Increment 2h registered the first destructive suite, and increment 2j the
second — the first to exercise increment 2i's general executor beyond one
range — each reachable only through its exact selector:

```text
cargo xtask test --tier 2 --profile destructive --suite gpt-basic-512-signature-erase
cargo xtask test --tier 2 --profile destructive --suite gpt-basic-512-both-signatures-erase
```

The second erases both of the image's GPT header signatures in one two-range
run: the primary at offset 512 and the backup at the last 512-byte LBA, each
eight bytes replaced with zeros, everything outside the two ranges pinned by
one digest bracket over the complement of their union.

It writes exactly one contracted range — eight bytes at offset 512, the
primary GPT header's signature field, replaced with zeros — through a
**read-write** loop attachment, which is what makes `LOOP_CHANGE_FD`
inapplicable rather than merely detected, and the run attempts that rebind
before writing and requires the kernel to refuse it. Every byte outside the
contracted range is pinned by a digest bracket taken through the held backing
descriptor before the write and re-checked after confirmed detach, and the
range itself is read before the write and required to differ, so a change is
established rather than an equality. The runner then regenerates the fixture
tree and re-reads it from disk against the compiled catalogue — on refusal as
well as on success — because every refusal after the write leaves the fixture
mutated.

**Its acceptance passed operator-run on 2026-08-11, three times** — first on
`4fbb2f9`; re-taken the same day on `68298f2` after the #248/#249/#250
review-finding fixes changed its own probe and write lines; and re-taken
again on `0625b07` after increment 2i's general executor replaced the
executor it runs through — each time in a disposable Proxmox-hosted non-WSL
VM, in the same sitting and on the same commit as a re-take of the increment
2e read-only acceptance. The kernel
refused `LOOP_CHANGE_FD` on the read-write attachment — the assumption the
design rests on, now measured on two kernel revisions and, since the re-take,
classified from an observed status re-read rather than from the errno alone —
the contracted eight bytes changed and nothing else did, the fixture tree
ended byte-identical to the catalogue, and nine negative controls refused in
each sitting. It registers no product write path: the mutation is
harness-owned bytes through an authorized handle, and the deliverable is that
a Tier-2 destructive *test* can exist.

Run `cargo xtask help` for the full command list.

## Open issues

Two are open, and one of them narrowed on 2026-08-18: ADR-0052 (spec
17.3.0) decided PART-005's destination vocabulary and WP-060 increment
11 delivered it, so #371's enumeration now exists in the planner and
what remains of the issue is the §6 consequence-text body carriage the
record already tracks. **#366 closed on 2026-08-17 without an ADR** — a
decision-owner call re-pointing a deferral clause's addressee, the first
closure in this record that changed no code, no requirement and no
decision, only which package a deferral was addressed to. Before it
ADR-0051 (2026-08-17) closed #319's authorization half,
the oldest live hole in the tree. Before it #365 closed with
ADR-0050, entire —
its Half B decided, its two wrong doc comments corrected, its coverage
delivered and its Part 3 recorded as discharged by ADR-0045. Before it
#409 closed with ADR-0049, and
before it #392 with ADR-0048, ADR-0044's
named limit with it, and #397 closed with ADR-0047 before it. #409 was
filed from the round preceding #397's act, and #319 stands reopened —
it had been auto-closed in error by PR #406's body, whose sentence saying
the act does *not* close it carried a closing keyword adjacent to the
number. This section reports
status only; each issue's own thread is the record, and the review
rounds under `docs/reviews/` carry the measurements. The current handoff,
`docs/reviews/HANDOFF_2026-08-16_FABLE_ISSUE_333_TO_NEXT.md`, states the
reasoning behind the ordering below.

**A sitting is owed.** The ADR-0052 arc — WP-010 PR #455 (`names_within` public, the `consumed` doc-comment corrected) and WP-060 PR #457 (increment 11, the move) — ships Rust after the r39 pin at `7b68927`, so the **r40 sitting (VMID 9465) is owed at the arc's head, `0519285`**, taken once for both merges per the one-sitting-per-arc practice and named in both PR bodies before the first merge. Its check is `git diff --name-only 7b68927 0519285 | grep -v '\.md$'`, which lists five paths: `crates/domain/src/model/protection.rs` and four under `crates/planner/src/`. What a green run there establishes is that the descriptor-bound loop chain over generated fixtures (2e, 2h, 2j) is unmoved by a change that touches neither the loop code nor the fixture generator; what it does **not** establish is that the move increment is correct — that evidence is its own suite and its ten killed mutants, recorded in the PR and in `docs/work-packages/WP-060.md`. Before it: WP-L100 PR #447 — increment 3a of the Linux read-only inventory adapter: devices addressed under ADR-0019 from ADR-0034's designated sources, and INV-004's presentable derivation — shipped five paths under `crates/adapter-linux/src/` after the r38 pin at `31e58be`; the r39 sitting (VMID 9464, 2026-08-18 UTC) on its merge commit discharged it, and WP-020 is re-pinned at `7b68927`. **It breaks a run of three infrastructure trips** — r36, r37 and r38 could not have moved a measured value even in principle, and this one could, so the identical value set is evidence rather than a foregone conclusion. Two invocations were void and are retained in the record: a relative-path launch that repeated custody run 26 verbatim, and the rollback taken to recover it, which returned the guest on the preinstalled -187 kernel. Before it: WP-000 PR #444 — `cargo xtask ci` refuses a documentation claim that has fallen behind the specification — shipped `tools/xtask/src/main.rs` after the r37 pin at `e96e46a`; the r38 sitting (VMID 9462, 2026-08-18 UTC) on its merge commit discharged it, and WP-020 is re-pinned at `31e58be`. That is the third consecutive infrastructure trip — r36 the workflow, r37 the workflow and the tool together, r38 the tool alone — and none touches the product under test. Before it: WP-000 PR #439 — an arc of two: the Section 11.4 fuzz targets run concurrently, and Tier 1 restores only its dependency build from cache — shipped `.github/workflows/ci.yml` and `tools/xtask/src/main.rs` after the r36 pin at `5cf868d`; the r37 sitting (VMID 9461, 2026-08-18 UTC) at the arc's head discharged it. r36 was the first trip that was neither Rust nor a Rust comment; this arc is the first to carry both kinds at once, and neither half touches the product under test. Before it: WP-000 PR #436 — the pinned supply-chain policy tools cached rather than rebuilt from source on every run — shipped `.github/workflows/ci.yml` after the r35 pin at `2428134`; the r36 sitting (VMID 9460, 2026-08-18 UTC) on its merge commit discharged it. That trip is the first in the record that is neither Rust nor a Rust comment — it changes the instrument that measures, not the product measured — and the exemption was declined on the same ground as the test-only trips before it. Before it: issue #371's structural pin (PR #433 — WP-060, test-only), whose r35 sitting (VMID 9459) discharged its own trip, and before that: issue #319's authorization half (PR #428 — ADR-0051, spec 17.2.0), whose r34 sitting (VMID 9458) discharged its own trip, and before that: issue #365's arc (PRs #424 and #425 — ADR-0050, spec 17.1.0), whose r33 sitting (VMID 9457) at the arc's head discharged it, and before that: ADR-0049's planner coverage (PR #421 — WP-060, test-only), whose r32 sitting (VMID 9456) discharged its own trip, and before that: issue #409's act (PR #419 — ADR-0049, spec 17.0.0), whose r31 sitting (VMID 9455) discharged its own trip, and before that: ADR-0048's planner coverage (PR #416 — WP-060, test-only), whose r30 sitting (VMID 9454) discharged its own trip, and before that issue #392's act (PR #414 — ADR-0048, spec 16.0.0), and before that issue #397's act
(PR #411 — ADR-0047, spec 15.3.0) shipped Rust after WP-020's r27 pin at
`ca2bc0f`; the r28 sitting (VMID 9452, 2026-08-16 UTC) on that act's
merge commit discharged it, and WP-020 is re-pinned at `63ee769`. The
check is
`git diff --name-only 7b68927 HEAD | grep -v '\.md$'`, and it must be run
against `HEAD` rather than against whichever pin a pull request body cited;
any non-Markdown path it lists re-opens the three acceptances.

Most of the domain issues were ordered by a dependency chain established by
measurement — **#347 → #360 → #354's kind half → #333's enforcement**.
#347 closed with ADR-0043, #360 with ADR-0044, #354 with ADR-0045, and
#333 with ADR-0046 (all 2026-08-16): the chain is closed. What remains
is not a chain. #397 closed with ADR-0047 and #392 with ADR-0048;
#409 closed with ADR-0049, which gave the closure a fourth arm rather
than answering #365's frame question. #365 then closed entire with
ADR-0050 and #319's authorization half with ADR-0051 (both 2026-08-17),
so no host-backed question is open. What remains is what ADR-0040 split
out of #348: #370 waits on a preservation proof that is not computable
today, and #371 on a producer for a duty the spec states and nothing
implements.

| Issue | Package | Standing |
| --- | --- | --- |
| **#370** | WP-010 | A byte-preserving relocation of a protected structure refuses, and relief needs a preservation proof that is not computable today. Split from #348 by ADR-0040. |
| **#371** | WP-010 | **Narrowed 2026-08-18.** Both parts the issue filed are delivered: WP-060 increment 11 (PR #457, under ADR-0052, spec 17.3.0) gives the model a representable relocation — `SizedRequest::Move { target, new_start }` — and a `Consequence::RelocationReleases` variant that enumerates what a move releases without carrying, kind-level, its negative space bounded (no partition type or role; silence is not a boot verdict); the tripwire PR #433 pinned came down in the same change, and PART-005 traces to six move tests. What remains is the rider the 2026-08-17 reading named: "in the plan" is the hashed body's §6 consequence-text item, and the enumeration is planner-layer carriage until that jointly-sequenced WP-010/WP-060 slice lands (`crates/domain/src/model/plan.rs:11-13`, `docs/work-packages/WP-060.md`). The issue stays open on that rider alone, and closing it by assertion would still be wrong. Split from #348 by ADR-0040. |

Issues **#318**, **#338**, **#348** and **#355** closed on or before
`6d743a3`; **#349** and **#356** closed with ADR-0041 (the body's facts
are validated against its topology at assembly), whose record names what
each leaves open — #356's absent-extent spelling is #319's class and
still constructs; **#353** closed with ADR-0042 (a frame root is never
written wholesale, and a target frame root reaches what it carries),
which leaves the per-kind truthful entry open as a cross-package act;
**#347** closed with ADR-0043 (a destroyed partition table releases the
partitions it describes) on its third round, spec 14.0.0; **#360** closed
with ADR-0044 (destruction carries through the cascade, and a volume
carries a partition table), spec 15.0.0; **#354** closed with ADR-0045
(names are admitted where edges are: the pair-table-derived naming kind
check, and the three multipath-node containment rows), spec 15.1.0;
**#333** and **#401** closed with ADR-0046 (the frame rule is enforced:
the extent's frame, the containment edge and the name agree, and
occupancy is read as bytes), spec 15.2.0.

**#366** closed on 2026-08-17 with no ADR and no version change. WP-035's
observability share had deferred the fabric-versus-local
transport-discrimination protocol to "whichever package first records a
transport route decision" — a phrase that denotes WP-040's per-OS **IPC**
transport everywhere else in this repository, so the deferral was
addressed to a package that never consumes the row and would never pick
it up. The sponsor is now **WP-010**, named rather than described by
role: it owns `TransportClass` and the closure that reads it, and the
ADR-0034-pattern designation extension the deferring round named in the
same sentence is sponsored from its assignment. Nothing became owed —
every adapter's transport answer stays `Unrecognized`, which is
`Indeterminate` at the closure and never `Permitted`.

## Roadmap

Milestones and their exit criteria are normative in Section 13 of the build
specification; work-package order is normative in Section 14. This section
reports status only and never redefines either.

### M0 — Foundations (in progress)

| Exit criterion | Status |
| --- | --- |
| CI green on Windows, macOS, and Linux | Met |
| `xtask` single entry point works locally and in CI | Met |
| Schemas versioned, with cross-language hash golden tests (MODEL-005) | Partial — the golden tests exist and gate CI; MODEL-003 schema versioning is not implemented |
| CODEOWNERS enforces ownership | Partial — `cargo xtask verify-change-ownership` now rejects a diff touching paths outside the assignment its commits declare, judged against the base revision. CODEOWNERS itself still only requires an owner's review, and sub-file grants ("this package's own status rows") stay a review obligation no path checker can express |
| T1 fixture generator produces images | Met — `cargo xtask fixtures` produces 14 images deterministically (WP-020 increment 1; the fourteenth, `gpt-both-copies-invalid-512`, added 2026-08-09 for the SI-35 resolution's unreadable arm) |
| Accessibility harness runs | Partial — `cargo xtask tokens` computes UI-001/007/008 from `schemas/design-tokens.json` and gates CI (WP-030 increment 1). The rejected Slint report keeps all ten `G-AX-*` rows inconclusive; it confirms rather than closes the missing keyboard, screen-reader, zoom, text-spacing, high-contrast, rendered-state, and reduced-motion evidence |

The three partial rows are tracked as known gaps in
`docs/work-packages/WP-000.md`, `docs/work-packages/WP-010.md`, and
`docs/work-packages/WP-030.md`. They are recorded rather than rounded up: a
milestone that exits on a criterion nobody verified is worse than one that exits
late.

### M0.5 — Evidence (in progress)

WP-035, M0.5's package, is in progress: increments 1–4 delivered the chassis
and its surfaces, and increment 5 recorded the SI-33/SI-35 measurement
protocols in `docs/quality/observability.md`. The **SI-33 experiment was run**
on 2026-08-02, but it did not establish the register's full liveness sequence:
L1 and L2 moved in 3/3 trials, while the close-before-event/reopen arm remained
unmeasured because L5b kept the original handle open across the event. A lower
later reading across an interval containing a PnP arrival shows that global
monotonicity cannot be assumed; the counter epoch and reset cause were not
characterized, so an equality-only witness is unsafe across that boundary.
**The 2026-08-04 successor sitting closed those gaps on the same reader**:
the close-before-event/reopen arm `moved` in 3/3 trials across true
no-handle windows, L4 reached its three trials, a reader re-arrival was
measured resetting the counter to its epoch floor, and the storage-node
PDO name qualified as an unprivileged epoch signal (ContainerId and the
USB-node name refuted) — cross-epoch readings stay incomparable by
construction, with a boundary-detection token available on this apparatus.

The **SI-35 Windows and loop instruments also have 2026-08-02 run records**,
but neither closes the issue. On Windows, no retained `MSFT_Disk`/
`MSFT_Partition` field, `MSFT_PhysicalDisk` row count, or layout-IOCTL value
separated conflicting, damaged-primary, or missing-backup from healthy GPT;
the wrapper discarded queried `MSFT_PhysicalDisk` property values, so the
broader existential hypotheses remain inconclusive. The WSL2 loop run was taken
while issue #94 was open; its normalized client projection and retained `blkid`
properties/`wipefs` offsets did not separate conflicting from healthy, but
that negative is non-qualifying and withheld pending a descriptor-bound
non-WSL rerun. The historical run did record positive 4Kn observability on
that environment. **Issue #94 closed on 2026-08-03, and the descriptor-bound
non-WSL rerun was taken the same day**: after two void sittings whose
instrument defects were diagnosed, amended, and recorded before any
subsequent output, the third sitting passed every validity gate. The named
candidate client projection is **non-separating** for the decisive
healthy/conflicting pair on real non-WSL Linux, confirming the historical
WSL2 negative on qualifying ground and lifting its promotion hold; the
labelled privileged `blkid`/`wipefs` comparison was likewise non-differing.
**M0.5's loop criterion is satisfied**; the run chooses no SI-35 option
and supplies no chosen-option refusal demonstration. **The SI-35 Windows
completion rerun was taken on 2026-08-04 and is valid**: all three gates
held (total retention, per-fixture digest brackets, the mandatory
index-fallback probe), W-H1, W-H2, and W-H3 are refuted — across 76
retained fields only the backing file's by-name `Location` differs
between healthy and each degraded GPT fixture — and W-Q4 is answered:
neither the hybrid fixture nor its MBR control reports any scheme, both
absent from `MSFT_Disk` while `MSFT_PhysicalDisk` sees every fixture.
The loop and rerun second-reader obligations are both discharged:
designated readbacks recorded on their pull requests, then an
independent 2026-08-04 retrieve-and-rehash of all six artifacts, every
digest matching. The SI-33/SI-28 successor protocol's S4 collision test
is fully executed: 2026-08-03 found the then-attached pair cross-model
(`not established`); 2026-08-04, on a same-model pair, observed the
preregistered collision — one identical placeholder serial from both
units at every serial-bearing layer — and the same day's card-move
sitting completed the final rider: the exchange is invisible at every
serial surface, follows-card versus follows-reader is undecidable by
value on a shared-constant pair, and unit continuity across the exchange
is unverifiable unprivileged. Every arm of the SI-33/SI-28 successor
protocol is now executed. **The increment 6 real-partitioned-Linux matrix
was taken on 2026-08-04** in a disposable Proxmox VM with explicitly
authorized passthrough fixture media, every row executed: the baseline
truly lacks raw access while `disk`-group membership alone flips it; the
client/helper signature asymmetry is measured in both directions (an
empty cached client view over live ZFS labels; both single-answer
interfaces reporting exactly the stale signature of the live-plus-stale
pair that only the enumerating probe reveals); the LVM2
member-independent designator is helper-only; identity facts survive
replug and reboot; and two byte-identical media collide as silent
last-writer-wins on every UUID-keyed symlink surface with no duplicate
signal anywhere. **Increment 6's macOS matrix was taken on 2026-08-05** and
is valid on its second sitting; the first is void on two recorded harness
defects — tool versions captured through verbs that do not exist, and a post
phase run without the reboot M5 depends on — and is retained with the two
amendments it produced, a SHA-256 tool-identity record and a hard
`kern.boottime` reboot gate. The valid sitting makes **macOS the third
platform on which the enumerated unprivileged projection does not separate
the decisive SI-35 pair**: `diskutil`'s structured output is byte-identical,
unnormalized, between a healthy GPT and one whose two tables describe
different partitions. It also finds that **every non-native signature
projects byte-identically to a blank disk** — live ext4 with a stale mdraid
superblock, an mdraid member, a LUKS2 container, an LVM2 orphan — so a macOS
client cannot tell a disk holding a file system from one holding nothing,
while the three partition schemes it does own (GPT, MBR, APM) are each named
distinctly. APFS container membership and its UUID are client-readable, the
UUID is stable across a verified reboot, and the unprivileged raw device read
is denied as on the other two platforms. **M10, the privileged comparison leg, was taken the same day** in an
ephemeral hosted `macos-15` runner, at no cost: the unprivileged client's raw
read is denied while root reads the true bytes of every fixture, and **the
decisive pair separates for the helper** — identical first-64-KiB digests,
differing last-64-KiB digests, so the two tables' disagreement lives in the
backup that no client interface on any measured platform reports. The four
signatures the client called byte-identical to blank each carry a distinct
helper digest. Only M9's Fusion shape stays `not established`, Apple Silicon
having no such hardware. **The macOS second-reader readback was discharged
on 2026-08-08** by an independent reader session: all three transcripts
retrieved through their locators and rehashed to their recorded digests,
every digest matching, with each record's custody caveat carried rather than
erased. Exit criteria are in Section 13 and are not
restated here; M0.5 remains in progress.

### M1 through M5

Not started. Their themes and exit criteria are in Section 13 of the
specification and are not restated here, because a second copy of a normative
table is a second copy to drift.

## Work-package order

Section 14 of the specification is normative. Current state:

| Package | Stage | Scope | Status |
| --- | --- | --- | --- |
| WP-000 | Foundations (M0) | Repository, CI, `xtask`, CODEOWNERS, dependency policy | In progress. Lock boundary, licence, fuzz-graph and ownership-inventory gates delivered; action discovery is now a structural YAML parse after three text-based attempts were each defeated, and the Dockerfile scanner behind it fails closed after nine further bypasses were found and made regressions. Ownership is enforced against a change, not only inventoried, and a generated lockfile may travel with the manifest it follows. `unsafe_code = "deny"` is no longer opt-in per crate. All four current `docs/traceability/WP-*.md` files are generated from source-local test annotations and typed evidence declarations, anchored to real requirement definitions and numeric specification sections, and cross-checked against live tests, tracked/owned paths, and xtask's parser. Each hand-written predecessor has an exact zero-loss migration ledger, and a hand edit fails CI. Issue #39 is closed; WP-000 remains in progress for its other documented gaps rather than being rounded up by this one completion. |
| ADR-C1 | — | Canonical encoding and hash strategy | Accepted |
| ADR-C2 … ADR-C6 | — | Hashed-artifact body/envelope split, identity strength, provenance shape, aggregation vocabulary, canonical set ordering and depth | Accepted |
| WP-010 | Foundations (M0) | Canonical domain model, schema versioning, encoding and hashing | In progress — increments 1, 2, 2a, 4, and increment 3's twelve slices (3a–3k) are delivered: the full Section 5 domain model, protection closure, sole-constructor step and plan boundaries, snapshot and plan body schemas with cross-language vectors, and generated traceability with a zero-loss migration ledger. Slice 3l (2026-08-12) landed the ADR-0022 reversal-linkage schema change jointly sequenced with WP-060's reversal increment: plan body version 2 with the required reversal item (draft by ID+hash / per-step impossibility statements / reapply-forward by ID — acyclic by construction), required step preconditions re-checked at every boundary, the `ReversalDraft` artifact with step-output target spellings and its own binding boundary, the no-draft-no-Reversible rule structural, and prediction-never-binds guards on every binding surface; version 1 stays accepted until the planner migrates, its retirement its own reviewed change. Slice 3m (2026-08-12) landed the ADR-0024 protection-family change jointly sequenced with WP-060's backup-family increment: plan body version 3 with the typed per-step class (the REC-001 repair family as a class, never an intent flag), the capture-impossible `uncapturable-regions` acknowledgment naming exact well-formed regions, the class-conditioned acknowledgment law (both table-state kinds lawful exactly on a table-repair step over an Indeterminate-state device, unconstructible outside the family, re-run at the boundary), and the `pre-state-preserved-for-recovery` reversal statement; the one-window version 2 is refused at decode with its retirement recorded. Slice 3n (2026-08-12) landed the PLAN-005 cancellation carriage jointly sequenced with WP-060's cancellation-vocabulary increment, under the WP-060 recorded cancellation-class decision: plan body version 4 with a required per-step `cancellation` field closed at PLAN-005's own three words, typed with the fail-closed `non-cancellable` floor as its default, the draft step pinned to the floor exactly as its class is pinned to `ordinary`, and no coupling law to `irreversible-after-start` invented (spec 12.3.0 records the independence both ways); the one-window version 3 is refused at decode with its retirement recorded and its vectors regenerated as v4. Slice 3o (2026-08-12) then retired version 1 — the reviewed change §0 had promised since 3l: the crate's own tests and vectors were its last emitters and are migrated (the identity-bound vector's SAFE-003 coverage surviving as `plan-v4-bound-identity-wipe`), `OperationPlan::assemble` is removed so a plan without a reversal linkage is unconstructible rather than refused, and version 4 is the sole version the boundary emits or accepts, every retired version refusing at decode. See `docs/work-packages/WP-010.md` |
| WP-020 | Foundations (M0) | Disk-image fixture generator and destructive-test interlocks | In progress — increments 1–1g, 2a–2d, 2e, 2g, 2h, 2i, 2j, and increment 2 itself (as scoped) delivered — 2g gave "no destructive suite is registered" a compiled type, a suite registry whose admission consumes the SAFE-007 `Authorization`, and 2h registered the first suite behind its own recorded boundary: one contracted eight-byte range written through a read-write loop attachment that makes `LOOP_CHANGE_FD` inapplicable, with a digest bracket over every other byte. Its operator-run acceptance passed 2026-08-11 alongside a second re-take of the 2e acceptance, whose stopping condition the 2g and 2h code changes had tripped; both were re-taken again the same day on `68298f2` after the #248/#249/#250 review-finding fixes (the three open findings from 2h's adversarial review) tripped it a third time, and once more on `0625b07` after increment 2i — the executor generalized to the registry's full contract shape (N fixtures, N non-overlapping ranges, a pre-flight over every fixture before any is attached), registering nothing new — tripped it a fourth; 2i is delivered. Increment 2j then registered the second suite — the first two-range one, both GPT header signatures of `gpt-basic-512.img` erased in one run through the general executor — and its acceptance passed on its first take 2026-08-11 (`fixtures_executed=1`, `ranges_written=2`, eleven negative controls refused, both ranges restored), tripping and re-pinning the stopping condition a fifth time at `39b59f5`; WP-070 increment 1 then tripped it from outside this package (2026-08-12), and one further sitting (VMID 9429) re-took all three acceptances on `a2e6db2` with identical value sets, re-pinning there; WP-070 increment 2 tripped it again the same day, and the r7 sitting (VMID 9430) re-took all three on `15e6469` with identical value sets, re-pinning there; WP-070 increment 3 tripped it a third time, and the r8 sitting (VMID 9431) re-took all three on `94bfeba`, re-pinning there; WP-070 increment 4 tripped it a fourth time, and the r9 sitting (VMID 9432) re-took all three on `d4f61ed`, re-pinning there; WP-070 increment 5 tripped it a fifth time, and the r10 sitting (VMID 9433) re-took all three on `59ba1f6`, re-pinning there and closing the WP-070 arc; the WP-060 unlock arc (PRs #299–#304, six Rust merges — WP-060 increments 5–8 with the jointly-sequenced WP-010 plan-body slices 3l/3m) then tripped it a sixth time from outside, and the r11 sitting (VMID 9434, 2026-08-13 UTC) re-took all three on `667f6aa` with identical value sets, re-pinning there and closing that arc per its recorded one-sitting plan; the PLAN-005 cancellation arc (PRs #307–#309, three Rust merges — WP-010 plan-body slices 3n/3o with the jointly-sequenced WP-060 increment 9) then tripped it a seventh time, and the r12 sitting (VMID 9435, 2026-08-13 UTC) re-took all three on `77b0dd7` with identical value sets, re-pinning there and closing that arc per the same recorded one-sitting economics (its void sudo-invoked first invocation refused at 2e's own Tier-1 redaction sweep and is retained as custody run 18); the WP-L100 arc (PRs #314/#316/#317, three Rust merges — the Linux adapter's contract-and-seam increment, its devices-and-identity increment, and its recorded corrections) then tripped it an eighth time, and the r13 sitting (VMID 9436, 2026-08-13 UTC) re-took all three on `b50dd19` with identical value sets, re-pinning there and closing that arc — its one-sitting price decided at re-take time on the r11/r12 precedent, the arc's plan having recorded no sitting economics; the ADR-0036 planner-half arc (PR #336, one Rust merge — WP-060 increment 10, the scheme reserved-region and located-occupancy rules on issue #319, behind its two Markdown-only governance merges #334/#335) then tripped it a ninth time, and the r14 sitting (VMID 9437, 2026-08-13 UTC) re-took all three on `1f9f2c7` with identical value sets, re-pinning there and closing that arc — its sitting recorded in the arc's plan **before the first merge**, restoring the practice the r13 record found lapsed; the issue-341 panic fix (PR #342, one Rust merge — `plan_set` settling simulatability before it builds statements, so an unsized create refuses instead of aborting on an `unreachable!`) then tripped it a tenth time, and the r15 sitting (VMID 9438, 2026-08-14 UTC) re-took all three on `f463d58` with identical value sets, re-pinning there — its sitting named in that PR's own body before the merge; ADR-0038 (PR #345, one Rust merge — the release operations seeding the protection closure on issue #338) then tripped it an eleventh time, and the r16 sitting (VMID 9439, 2026-08-14 UTC) re-took all three on `901c7d2` with identical value sets, re-pinning there — the first sitting whose merged change alters protection behaviour, and the acceptances measure byte-for-byte what they measured before it. ADR-0039 (PR #351, one Rust merge — carried-content reach and a bounded descent at spec 13.0.0, closing issue #338’s held half) then tripped it a twelfth time, and the r17 sitting (VMID 9440, 2026-08-14 UTC) re-took all three on `b9d1ba2`, again with identical value sets, re-pinning there — the second protection-behaviour change in a row to leave every measured acceptance value unmoved. The verdict-multiplicity fix (PR #357, one Rust merge — `node_verdict` folding `worst` over every matching edge on issue #355) then tripped it a thirteenth time, and the r18 sitting (VMID 9441, 2026-08-14 UTC) re-took all three on `c9cd4bb` with identical value sets, re-pinning there — the third such change in a row, and the first sitting since r14 not named in its PR's body before the merge, recorded as a lapse. The issue-354 referent-sweep arc (PRs #361, #362 and #363, three Rust merges — a fixtures test-determinism fix, the naming-referent resolve sweep on issue #354, and the shared referent roster) then tripped it a fourteenth time, and the r19 sitting (VMID 9442, 2026-08-14 UTC) re-took all three on `86db930` with identical value sets, re-pinning there — taken once at the arc's head with the sitting named in all three PR bodies before their merges, the practice restored after r18's lapse; it is also the first arc spanning both this package's own paths and paths outside it, and the first in which a **test-only** merge re-opened all three acceptances. The issue-318 record sweep (PRs #368 and #367) then tripped it a fifteenth time on comment-only `.rs` paths — a crate doc comment, the `transport_class` doc comment, and a test's requirement comment, none changing behavior — and the r20 sitting (VMID 9444, 2026-08-15 UTC) re-took all three on `6d4a8fc` with identical value sets, re-pinning there; the sitting was named in #367's body before the merge. ADR-0040 (PR #372, the issue-348 relocation-exemption retirement) then landed 48 lines of `crates/domain/src/model/protection_tests.rs` *after* that pin, tripping the condition a sixteenth time. That PR named its sitting before merging, per the r15–r19 practice, but named it against the r19 pin `86db930`; the r20 sitting was taken on `6d4a8fc` — an earlier merge — so the promise was discharged at a commit that does not cover this change, and the debt was found by checking the stopping condition against `HEAD` rather than against the pin the PR body cited. The r21 sitting (VMID 9445, 2026-08-15 UTC, no void invocation) re-took all three on `b8d6a90` with identical value sets, re-pinning there; this was the third consecutive invitation (after r19's test-only merge and r20's comment-only paths) to exempt a change that touches no behaviour, and it was declined again. **The stopping condition is checked against `HEAD`, never against the pin a pull-request body cites.** The body-validity arc (PRs #377 and #379 — ADR-0041, spec 13.1.0) then tripped it a seventeenth time, and the r22 sitting (VMID 9446, 2026-08-15 UTC, no void invocation) re-took all three on `b002ac3`, the arc's head, with identical value sets, re-pinning there — the one sitting named in both PR bodies before the first merge and the condition checked against `HEAD` at re-pin. The issue-353 arc (PRs #382 and #384 — ADR-0042) then tripped it an eighteenth time, and the r23 sitting (VMID 9447, 2026-08-16 UTC, no void invocation) re-took all three on `53c90f1`, the arc's head, with identical value sets, re-pinning there under the same practice. Issue #347's round-3 act (PR #388 — ADR-0043, spec 14.0.0) then tripped it a nineteenth time, and the r24 sitting (VMID 9448, 2026-08-16 UTC, no void invocation) re-took all three on `c83d9f1` with identical value sets, re-pinning there. Issue #360's act (PR #393 — ADR-0044, spec 15.0.0) then tripped it a twentieth time, and the r25 sitting (VMID 9449, 2026-08-16 UTC, no void invocation) re-took all three on `91cd1c9` with identical value sets, re-pinning there. Issue #354's kind-half act (PR #398 — ADR-0045, spec 15.1.0) then tripped it a twenty-first time, and the r26 sitting (VMID 9450, 2026-08-16 UTC, no void invocation) re-took all three on `ee12af2` with identical value sets, re-pinning there. Issue #333's enforcement arc (PRs #403, #404 and #406 — ADR-0046, spec 15.2.0) then tripped it a twenty-second time, and the r27 sitting (VMID 9451, 2026-08-16 UTC, no void invocation, taken once at the arc's head) re-took all three on `ca2bc0f` with identical value sets, re-pinning there. Issue #397's act (PR #411 — ADR-0047, spec 15.3.0) then tripped it a twenty-third time, and the r28 sitting (VMID 9452, 2026-08-16 UTC, no void invocation) re-took all three on `63ee769` with identical value sets, re-pinning there. Issue #392's act (PR #414 — ADR-0048, spec 16.0.0) then tripped it a twenty-fourth time, and the r29 sitting (VMID 9453, 2026-08-16 UTC, no void invocation) re-took all three on `be6c3f0` with identical value sets, re-pinning there. ADR-0048's named obligation — WP-060's planner coverage, PR #416, whose one non-Markdown path is a test file — then tripped it a twenty-fifth time, and the r30 sitting (VMID 9454, 2026-08-16 UTC, no void invocation) re-took all three on `55e5601` with identical value sets, re-pinning there: the fourth test-only or comment-only invitation to exempt a change that touches no behaviour, declined again. Issue #409's act (PR #419 — ADR-0049, spec 17.0.0) then tripped it a twenty-sixth time, and the r31 sitting (VMID 9455, 2026-08-17 UTC, no void invocation) re-took all three on `8791a56` with identical value sets, re-pinning there. ADR-0049's named obligation — WP-060's planner coverage over a host-backed body, PR #421, whose one non-Markdown path is a test file — then tripped it a twenty-seventh time, and the r32 sitting (VMID 9456, 2026-08-17 UTC, no void invocation) re-took all three on `6168b47` with identical value sets, re-pinning there: the fifth test-only or comment-only invitation to exempt a change that touches no behaviour, declined again. Issue #365's arc (PRs #424 and #425 — ADR-0050, spec 17.1.0) then tripped it a twenty-eighth time with six non-Markdown paths, and the r33 sitting (VMID 9457, 2026-08-17 UTC, no void invocation) re-took all three on `7fad56d` with identical value sets, re-pinning there — one sitting at the arc's head, the intent recorded before the first merge. Issue #319's authorization half (PR #428 — ADR-0051, spec 17.2.0) then tripped it a twenty-ninth time, and the r34 sitting (VMID 9458, 2026-08-17 UTC, no void invocation) re-took all three on `fed2d76` with identical value sets, re-pinning there. Issue #371's structural pin (PR #433 — WP-060, one test file and no production line) then tripped it a thirtieth time, and the r35 sitting (VMID 9459, 2026-08-17 UTC, no void invocation) re-took all three on `2428134` with identical value sets, re-pinning there: the sixth test-only invitation to exempt a change that touches no behaviour, declined again. Void invocations are retained in the record rather than smoothed over in both sittings: r19's single one (a relative-path launch) with the kernel change its rollback caused, and r20's two — a dpkg lock held by cloud-init, which made the hardened provisioning script abort exactly as designed, and then the recovery's own side effect of leaving that script's unmount step with nothing to unmount. **Increment 2 is thereby delivered as scoped**: a Tier-2 destructive suite can exist, two do, each writing exactly its declared ranges of one generated fixture under every SAFE-007 factor in a disposable VM — and no product write path exists or is authorized by any of it. Increment 2e's descriptor-bound loop acceptance **passed on 2026-08-03** in a disposable Proxmox-hosted non-WSL Linux VM — on the implementation commit `2dbf601` and again on the merged commit `c75b340` that lands on main — and was **re-taken on 2026-08-11** on current main `582e6d1` in a fresh disposable VM after the record's stopping condition tripped (issue #175), reporting the identical harness values and byte-identical fixture digests — closing issue #94: the adversarial `LOOP_CHANGE_FD` rebind was detected and discarded, detach and partition teardown were confirmed, and both fixture digests were unchanged. Closing it registered no destructive suite by itself; the suites came later, each behind its own recorded boundary. The guest was deliberately not network-isolated and the run's exclusions, deviations (a required `snapd` purge), and limits are recorded in `docs/work-packages/WP-020.md` rather than summarized away. Preconditions 1 and 3 are closed on both platforms (issue #51): Unix opens a direct child relative to a held root object, Windows holds the root with a share mode the filesystem enforces, and the other-name refusal — which was a **live defect**, not a missing check — now reads the link count through the authorized handle everywhere. Windows containment is enforcement by the filesystem rather than resolution from a handle, so it is **unproven for roots that are not on a local volume**, and non-local roots are refused. WP-020 traceability is generated from validated source-local claims and typed evidence, with a source-revision/blob ledger preserving every former evidence row, correction, limitation, and residual risk. The runnable higher-tier selectors are the two registered read-only acceptances (`linux-loop-read-only`, `si35-loop-capture`) and the two compiled destructive suites through `--suite`; every generic destructive Tier-2 request and every Tier-3 request still refuses. See `docs/work-packages/WP-020.md` |
| WP-030 | Foundations (M0); desktop shell deferred, no authority on main | Design tokens, dark UI shell, accessibility harness | In progress — increments 1 through 1c delivered (tokens, the static accessibility harness, and zero-loss generated traceability). Increment 2S's bounded Slint 1.17.1 branch was implemented, measured, mechanically rejected on two hard gates, and closed without merge. Main now retains only normalized evidence, the byte-reproducible 41-row report, and accessibility limitations; no shell exists, UI-002 remains unimplemented, and the rendered half of UI-008 remains untested. The temporary implementation authority was retired by PR #91, so no desktop-shell path is authorized on main and reviving either off-main branch needs fresh governance rather than inertia |
| WP-035 | Evidence (M0.5) | Read-only CLI chassis and evidence instrument | In progress — increments 1–4 delivered the unprivileged CLI chassis, typed schema-versioned refusals, redacted diagnostics, bounded absolute-path dependency doctor, technology facts, and fixture-backed replay observations; increment 5 recorded the operator-run SI-33/SI-35 instruments. The audit reserves `inventory`, `topology`, and `capabilities` as exact typed refusals rather than accepting them as unknown commands, treats a dependency's nonzero exit as failure rather than version evidence, escapes terminal controls, and uses each Unix target's actual replay flags. The SI-33 run moved in L1/L2 but did not measure the required close-before-event/reopen arm; a lower reading across a PnP-arrival interval makes global monotonicity unsafe to assume without characterizing the counter epoch. The Windows SI-35 run found no difference in its retained CIM fields, PhysicalDisk row count, or layout IOCTL, but discarded queried PhysicalDisk property values, so its broader existential hypotheses are inconclusive. The historical WSL2 loop record did not separate healthy from conflicting in its post-hoc-normalized retained projections and did observe 4Kn through an explicit-sector-size loop, but it was taken while issue #94 was open and is non-qualifying pending a descriptor-bound non-WSL rerun. **Issue #94 closed on 2026-08-03** when WP-020 increment 2e's descriptor-bound mechanism landed and passed a real acceptance, and **the descriptor-bound non-WSL rerun was taken the same day and passed as valid on its third sitting** (two void sittings and their recorded instrument amendments precede it, retained rather than discarded): the named candidate client projection is **non-separating** for the decisive healthy/conflicting pair, the historical WSL2 negative is confirmed on qualifying ground, the WSL2 promotion hold is lifted, and **M0.5's loop criterion is satisfied**. SI-33's remaining measurement gaps closed on 2026-08-04: the close-before-event/reopen arm `moved` in 3/3 trials across true no-handle windows on the measured reader, L4 reached its three trials, the counter's re-arrival reset was measured, and the storage-node PDO name qualified as an unprivileged epoch signal while ContainerId and the USB-node name were refuted. The SI-35 Windows completion rerun was taken 2026-08-04 and is valid: W-H1/H2/H3 refuted over completely retained surfaces and W-Q4 answered; the 2026-08-02 wrapper's retention defect did not recur. (SI-35 has since resolved in spec 8.0.0 by ADR-0014, its refusal demonstration landing with the resolution; the measurement record above is retained as taken.) The loop and rerun second-reader obligations are discharged: designated readbacks recorded on their pull requests, then an independent 2026-08-04 retrieve-and-rehash of all six named artifacts, every digest matching. The SI-33/SI-28 successor protocol's S4 collision test is fully executed: on 2026-08-04 a same-model pair produced the predicted outcome — one identical placeholder serial from both units at every serial-bearing layer, after the 2026-08-03 sitting found the earlier pair cross-model — and the card-move sitting completed the final rider, the exchange invisible at every serial surface and unit continuity unverifiable on the shared-constant pair. The successor protocol is fully executed (S1, S2, S2b, S3, and S4). The increment 6 real-partitioned-Linux matrix is executed in full (2026-08-04, disposable Proxmox VM, authorized passthrough fixture media): client/helper signature asymmetry measured both ways, LVM2's member-independent designator helper-only, the SI-34 stale-signature finding established on a real device tree, and UUID-keyed addressing collapsing by silent last-writer-wins under byte-identical media. The increment 6 macOS matrix was taken 2026-08-05 and is valid on its second sitting, the first being void on two recorded harness defects and retained with the amendments it produced: the decisive SI-35 pair is non-separating on macOS too, making it the third platform to answer that way; every non-native signature projects byte-identically to a blank disk while GPT, MBR and APM are each named distinctly; APFS membership and its container UUID are client-readable and the UUID is stable across a verified reboot; and the unprivileged raw read is denied. M10, the privileged comparison leg, was taken the same day in an ephemeral hosted `macos-15` runner at no cost: the client's raw read is denied while root reads the true bytes, and the decisive pair separates for the helper — identical head digests, differing tail digests, placing the disagreement in the backup table no client interface reports — while the four signatures the client called byte-identical to blank each carry a distinct helper digest. Only M9 stays `not established` on Apple Silicon, no preregistered cell on any platform remains `not yet taken`, and the macOS second-reader readback was discharged on 2026-08-08 by an independent reader session — all three transcript digests matching on retrieve-and-rehash through their locators, each record's custody caveat carried in the discharge rather than erased. Under spec 6.1.0's grant, increment 7 published the INV-003 reach declaration on all three platforms (every cell derived from the contract this package reads, negatives present rather than omitted, guarded by a single-constructor table whose one possible positive is pinned by a mutation-verified test) and increment 8 delivered the Linux enumeration adapter behind `inspect` — whole devices through sysfs file reads with no subprocess, raw identifier strings labelled by reporting interface, refusing over-limit and non-UTF-8 values rather than truncating or substituting, preserving padding rather than manufacturing false absences, and failing closed on partition-filter read errors — so the CLI now reads real hardware on Linux. **Increment 9 delivered the macOS adapter (2026-08-08)** on the recorded bounded-reader route: `diskutil` launched twice per enumeration shape at its compiled absolute path through the widened launcher seam, parsed by a bounded in-crate XML plist reader that refuses every construct outside its stated grammar — no dependency, the empty-closure guard intact — reporting twelve identity keys as raw interface-labelled strings with `Content`, UUID, and APFS fields deliberately unread; a nonzero exit is never parsed, a container value is a typed failure, a missing key is a positively determined absence, and the macOS reach declaration moves to implemented-reaches-no-table-state with every cell still negative. Its Section 11.4 fuzz target lands separately under `fuzz/`'s ownership (WP-010 + WP-000 xtask row) and is in flight, not silently absent. **Increment 10 is closed as deferred by its recorded route decision (2026-08-08)**: no Windows route is simultaneously dependency-free, `unsafe`-free, and clean against the tool-invocation rules, so the interim Windows adapter is not built, the Windows answer and reach reference name the recorded decision in-band rather than a pending increment, and WP-W100's row is untouched. The package remains forbidden every domain surface its Boundary lists — the gates now cited to the accepted decisions where the register resolved them, to SI-28 where it stays open; see `docs/work-packages/WP-035.md` |
| WP-050 | Trustworthy read-only product (M1) | Capability engine interfaces and fixtures | In progress — increment 1 delivered the CAP-003 vocabulary as `crates/capability`: the four statuses with `supported` constructible only through CAP-006 qualification evidence, whose type has **no constructor yet** (compile-fail-proven unreachable, the correct answer while no apply path exists anywhere), the closed MODEL-003-versioned reason enum (`partman.capability.reason` v1) re-enumerating the domain's protection grounds through exhaustive `From` impls so a domain addition fails compilation here, the decided couplings carried by `from_protection_gate` (3g's refusals→`unsupported`, indeterminacies→`blocked`; `Clear` is no answer), and caller-stated remediation with the no-remedy case explicit. The `TechnologyLimit` reason's status coupling is deliberately unasserted: FS-007's "as explicit blocked reasons" and CAP-003's `blocked` definition ("implemented, but a runtime precondition fails") assign that case different statuses — the Section 1.11 shape, to be filed on the register rather than decided silently in a constructor. Increment 2 delivered the engine core: `capability()` composing the decided arms in refusal-precedence order — the domain's protection gate (the same closure the plan constructor runs), technology limits statused per ADR-0020's SI-40 resolution (`unsupported`, explicit reason, no-remedy-exists exactly), Section 9 floor and ACC-009 tool preconditions as `blocked` with tool-naming remediations, `preview` pending CAP-006 evidence otherwise — with the CAP-005 agreement enumerated over every operation/target pair of a six-target fixture rather than asserted, source classes never suppressed, and unknown targets a typed error rather than an answer. Increment 3 delivered the CAP-006 store structured and truthfully empty: `docs/capabilities/` with its format document, an advertised set that is empty with the vacuity named (advertising and qualifying are each reviewed acts), an empty floors file for the same stated reason, and a Tier-1 store test pinning the qualified-row count at zero — the evidence token gains no constructor at all until a consumer and a qualifying row both exist, so the increment-1 compile-fail proof holds verbatim. Increment 4 delivered the consumer seams (the API documented for the CLI, planner, and adapter classes, each under its own grant, none with authority over answers), the multipath detection-only arm the all-reasons coverage requirement caught missing — it precedes protection because LIN-006 names the reason that population reports, while the closure refuses the same population anyway, so precedence moves reporting and never permission — and the coverage test over every reachable status and reason with the two unreachable members asserted unreachable by proof. **Increments 1–4 delivered**; remaining obligations are consumer-driven: a first qualification row, the evidence loader for its first consumer, floors as tools join the roster. See `docs/work-packages/WP-050.md` |
| WP-060 | Planning and dry run (M2) | Pure planner, extent solver, risk model, simulated topology, reversal plans | In progress — increment 1 delivered the request vocabulary and pure chassis as `crates/planner`: `plan()` computing PLAN-001's deterministic side-effect-free result (byte-equal bodies held by test), conditioning on the WP-050 engine's answers with refusals carried verbatim (`unsupported`/`blocked` refuse, `preview` plans, `supported` is not a distinct planning state), every step through `PlanStep::mutating` and every plan through `OperationPlan::assemble`, source-class requests refused as not plan material, severities conservative-up with the Reversible claim explicitly withheld until PLAN-008 can emit a reversal (SI-19). The assignment names its register gates — SI-15/16/17/19/24 — each conservative-refused, never silently answered. Increment 2 delivered the step graph: explicit dependency edges, cycles refused with every unorderable member named, duplicates refused before ranges are compared, and the committed conflict rule — dependency-unordered steps touching the same bytes refuse naming both steps and the host, while ordered overlap is a legitimate chain explained by its dependency; Kahn's smallest-ready-first ordering keeps PLAN-001's byte-equal determinism over multi-step sets. Increment 3 delivered the alignment-conservative extent solver: free space from the authenticated extents alone, first-fit at PART-009's 1 MiB default with no-fit refusals naming the largest aligned fit, deviation inexpressible rather than half-supported until its vocabularies arrive, SI-15's misaligned-growth case refusing by name with the gate string carried, and `plan_sized` carrying solved geometry into deterministic revalidating bodies. Increment 4 delivered the simulated final topology: every entry point returns the plan and its prediction together because PLAN-002 makes simulation mandatory — unrepresentable effects produce no valid plan rather than a prediction that lies; wipes empty the container without removing it and drop the table-state stamp, sized creates mint under a single table view or refuse, and the simulated snapshot can never revalidate a plan (a prediction is not a capture, structurally, held by test). **Increments 1–4 delivered**; SI-19 resolved 2026-08-11 in spec 12.0.0 by ADR-0022 (the reversal is an ordinary draft bound at its own validation, linked by reference — `OperationPlan` is not recursive) and SI-15 resolved the same day in spec 12.1.0 by ADR-0023 (a PART-009 deviation is authored, not inherited — the misaligned grow-at-tail case proceeds), and SI-16 resolved the same day in spec 12.2.0 by ADR-0024 (PART-013 discharges by the helper's authored table state, so the backup family is buildable without answering a register question in code), and SI-17 resolved the same day in spec 12.3.0 by ADR-0025 (`irreversible-after-start` defined temporally — the flag claims the mid-execution window, severity claims endpoints, the combination legal), and SI-24 resolved 2026-08-12 in spec 12.4.0 by ADR-0026 (a dry run is an apply rehearsal, not CAP-003's simulation — the preview-arm semantics WP-070 builds under, decided before the pipeline exists), so **the assignment's register-gate list is empty** and the four unlock increments opened under recorded decisions. **Increment 5 delivered the SI-15 solver unlock (2026-08-12)**: the misaligned-growth refusal replaced by ADR-0023's decided behavior — the 63-sector grow-at-tail case authors only the aligned end with the untouched start carried as a typed inherited fact in the planner's consequence material, grow-to-fill is conformant and recorded as coincident naming the edge, the §11.2 authored/inherited split holds by test, and the no-fourth-state property is swept with authored-end policy now enforced on every solver path (`UnalignedAuthoredBoundary` naming the nearest conforming values); **Increment 6 delivered the reversal (2026-08-12)** on WP-010 slice 3l's jointly-sequenced plan-body v2 change: every plan emits PLAN-008's output — the sized create's truthful draft (a step-output-referenced delete of the empty created structure, the Reversible claim made exactly because it exists), the grow's shrink-back draft with the reclaimed tail judged in the target's own address space, and per-step machine-readable impossibility statements everywhere else — with the draft byte-deterministic, binding only against a real post-apply capture, refusing the pre-apply world and the prediction itself, and refusing by precondition once data lands (the truth-decay fixture end to end); **Increment 7 delivered the backup family (2026-08-12)** on WP-010 slice 3m's jointly-sequenced plan-body v3 change — PART-013's planning half, state-selected per ADR-0024: parse-backup on Present, the journaled determination on Absent with no acknowledgement demanded, ordinary operations on Indeterminate media refusing typed before any obligation is computed, and the typed table-repair family planning over exactly that media with write targets exactly the located table regions, the raw-capture obligation naming exactly them, and the capture-impossible arm proceeding only under the plan-creation acknowledgement riding the hashed body; **Increment 8 delivered the combination unlock (2026-08-12)**: the `irreversible-after-start` flag derived from ADR-0025's typed criterion (wipe/shrink/repair flagged, entry writes not, the journaled chunk copy stated unflagged for its family), severity 1 plus the flag constructing exactly on its truthful draft and refusing without one, the post-first-write `no-writes` claim unconstructible, and PLAN-004's plan-level flag union making the ceremony's inputs derivable with enforcement recorded as the helper packages' boundary — **all four unlock increments the resolved register gates opened are delivered**; **Increment 9 delivered the cancellation vocabulary (2026-08-12)** on WP-010 slice 3n's jointly-sequenced plan-body v4 change, opened by the recorded cancellation-class decision in the WP-035 route-decision shape: every step declares exactly one of PLAN-005's three words in the hashed body, the class a per-family stated declaration over the fail-closed `non-cancellable` floor (never a derivation from the interruption profile — cannot-stop and cannot-unwind are independent both ways per spec 12.3.0), the journaled-chunk-copy family stated `checkpoint-cancellable` before the planner emits it, every family the planner emits today on the floor with named revisit conditions, and the derivation wired explicitly into every step construction — every item the assignment's beyond-list named is now delivered. **Increment 10 delivered the scheme's own regions and located occupancy (2026-08-13)** under ADR-0036, spec 12.13.0, on issue #319 — a measured fail-open in the delivered free-extent derivation, where a create of exactly the 1 MiB default landed at offset 0 over the protective MBR and the GPT header and was recorded as conforming, because the fixture's partition-table node carries a containment edge and no extent and the subtraction never saw it: free extents now also withhold the regions a host's declared table schemes claim at each end, derived from the table node's own hashed `TableRole` and stated as a bound rather than a measurement (no sector size reaches the module), with the head uniform across recognized roles, a tail only for `Gpt`/`HybridMbr`, a conflicting entry's `view_role` widening its table's reservation, and an unnameable scheme refusing rather than guessing a layout whose metadata may sit anywhere; and every partition the authenticated names place on the host must be one the subtraction actually removes at the offset its own hashed name declares — located-ness deliberately **not** key presence, which is the measured fatal that killed two rival designs, with occupancy read from the naming fields and never from containment edges (held as a property: the free list is byte-identical with every edge removed). Two further §11.2 defects close with it — a host extent exceeding the size its own name declares, and a child extent leaving its host. Issue #319's authorization half stays open — recorded at the time as blocked on issue #333's anchoring question, and **corrected by ADR-0037 (spec 12.14.0) to issue #338**, where operations that seed no destroyed range leave the closure with nothing to propagate from — **narrowed to six by ADR-0038 (PR #345), which gave `Shrink` and `Move` the conservative entry ADR-0018 already licenses, and closed by ADR-0039 (spec 13.0.0), which gave the closure carried-content reach and a per-edge-target geometric bound** — so the six that destroy nothing refuse through the content their target carries, and a partial destruction reaches what it truncates; #333 itself is decided, its rule accepted and its enforcement held. **Increment 11 delivered the move (2026-08-18)** under ADR-0052, spec 17.3.0, on issue #371: `SizedRequest::Move { target, new_start }` with the solver's destination rule (start on the default, end judged as every authored end, the destination inside one run of free space plus the source, intersecting no node neither the target nor named within it — the scoped clause, whose literal form refused a downward move over the partition's own file system), the copy mode derived from the two ranges, the conservative `destroyed = S`, `consumed = D` declaration ADR-0052 amended ADR-0018's consumed class to admit (measured: an orphan signature inside the overlap is reached and refuses under it and is reached by no arm under the precise one), the simulation renaming the moved partition and everything named within it at the destination with carried extents translated, the `MoveDraft` reversal resolved by `consumed = [D]` under the unchanged step-output contract with the released end of the source as its precondition, and the hosted-signature duty discharged by naming position — carried content preserved, uncarried content refused inside the overlap and enumerated as `Consequence::RelocationReleases` in the released source, its negative space bounded (no partition type or role; silence is not a boot verdict). The `no_representable_request_relocates_bytes` tripwire came down in the same change, replaced by `only_a_move_relocates_a_pre_existing_start`; ten mutations, each proven applied, all killed. Named in the ADR and not here: WP-050's post-move spelling, WP-070's §11.2 interrupted-move invariant, the consequence-text body slice, and `Copy`. See `docs/work-packages/WP-060.md` |
| WP-040 | Foundations (M0) | RPC schemas, transport per OS, handshake, helper authentication skeleton, redaction | In progress — increment 1 delivered the message layer as `crates/rpc`: the versioned envelope over `pce/1` (both RPC ends already encode and hash identically under MODEL-005's cross-language proof), RPC-004's 1 MiB bound binding the wire before any parsing, RPC-002's handshake with refuse-never-degrade as a total function carrying a remediation naming the older side, and RPC-003's strict validation with one validator for both ends so laxness has nowhere to live; `schemas/rpc/envelope.md` and `handshake.md` record the formats. The assignment gates every OS transport behind its own recorded route decision (the WP-035 increment-10 triangle, three times over) and names SI-18 as holding all authorization vocabulary out of the authentication skeleton. Increment 2 delivered the streams and reattach vocabulary: envelope v2 with the event `sequence` field under per-channel presence rules held strictly both ways, monotone gap-free producer sequencing with a total consumer classification (in-order processes, replays discard, gaps name their missing range and recover from the journal — WP-070's, said so), the strict resume token, and clock-less timeout vocabulary. Increment 3 delivered the redaction boundary: SEC-006's deny-floor as a schema-level classification of every owned field position, whose allowlist — exactly the envelope `body` and the resume token's `execution` handle, each with its governing authority named — needs no knowledge of the denied classes because every other position is structurally identifier-incapable, with the handshake `build` constrained to a build-version grammar in both directions (handshake schema v2, the envelope-v2 reviewed-bump posture) and a gate test planting every SEC-006 class raw in every non-allowlisted position; `schemas/rpc/redaction.md` records the rule and what a grammar cannot do. Increment 4 delivered the authentication skeleton: the closed per-transport claim vocabulary RPC-001 implies — the Windows pipe's SDDL restriction, the Unix socket's peer credentials, the macOS code-signing requirement — as types naming what a peer proves, verified by nobody here, each verifier waiting on its transport's unrecorded route decision by name, with no authorization vocabulary while SI-18 holds that question and a closure test that fails the suite on widening. **The four ungated increments are delivered**; SI-18 — the register gate the assignment named — resolved 2026-08-11 in spec 11.2.0 by ADR-0021 with no plan-carried authorization-requirement field, so it unlocks no schema work here and the skeleton's identity-only vocabulary is a standing decision rather than an interim posture (the mid-row "while SI-18 holds" phrasings describe increments as delivered, which is history); what remains is one transport increment per OS behind its own recorded route decision. See `docs/work-packages/WP-040.md` |
| WP-070 | Planning and dry run era (M2) | Journal (JRN-001…006) and execution state machine (Section 8) | In progress — the assignment was created with an empty register-gate list (SI-20/21/22/23 all resolved first, ADR-0027/0028/0029/0030) and thirteen imported verification obligations enumerated per increment. Increment 1 delivered the pure state machine as `crates/statemachine`: Section 8's thirteen states and twenty-three-row transition table as `Transition` variants — an undeclared pair has no variant, proven unrepresentable over all 169 ordered pairs against an independent transcription of the specification's rows — with terminal records structurally carrying their effect summaries, the published per-row effect constraints (`no-writes` alone on the three no-writes rows, `no-writes`-or-`partial` on the honored cancel, none where the row constrains nothing), ADR-0027's two arms asserted as the exact `RecoveryRequired` exit set, and the machine-readable table Section 8 requires rendered into `schemas/state-machine.md` from the same variants the property tests check, held byte-fresh by test with a documented regeneration example. The crate has no dependencies and the journal (increment 2) builds on it, never the reverse. Increment 2 delivered the journal core as `crates/journal`, pure and dependency-free: JRN-001's append-only checksummed frames (CRC-32/IEEE pinned against an independent transcription) with strictly monotonic sequence numbers, torn-tail detection and safe truncation swept over every byte cut, interior damage refusing rather than truncating, the JRN-002 durability rule as a typed injected seam (`WriteClearance` constructible only behind the seam-advanced watermark, platform fsync truth staying the M3 helper packages' acceptance obligation), JRN-003's idempotent replay with recovery a fixpoint and the sequence never resetting, and obligation 11's core — the three-way gap classification over typed covered ranges, with the mid-chain gap the named corruption case refusing by name; Increment 3 delivered the record vocabulary under JRN-006: `partman.journal.record` v1 encoded through WP-010's `pce/1` codec, carrying every imported record class — the authorization act with helper-computed tier (ADR-0021/0028), transition records enforcing Section 8's per-row effect constraints at record-write time with the ADR-0027 disposal linkage on the `failure-accepted` row alone, checkpoints, the three-variant protection record with hash-only artifact references and bytes structurally absent (ADR-0024/0030), the compaction record with the per-apply budget constant (ADR-0029), and the dry-run refusal class (ADR-0026) — the WP-010 joint sequencing discharged hash-only, JRN-005 held structurally with the SEC-006 exemplar gate proving no position carries free text, strict MODEL-003 rejection, golden vectors pinned in `schemas/journal/records.md`, and the disposal chain reconstructed from journal bytes alone; Increment 4 delivered retention and compaction under ADR-0029's liveness rule: the exemption computed from decoded records with the linkage closure (an unstarted disposal-named recovery counts as non-terminal, fail-closed toward retention), `compact()` as the sole reclamation entry point computing its own reclaimable set so no code path can name a live record, budget exhaustion resolving only to the published `Executing → RecoveryRequired` row, `CoveredRanges` derived from the journal's own durable compaction records by two-pass replay, monotonicity across compaction with compaction records never reclaimed, and the ADR-0028 chain trace reading identically across a compaction around the live apply; Increment 5 delivered the apply lifecycle enforced at the library layer (ADR-0028): one act admits one apply of exactly its plan — a second grant against a consumed act refuses at the grant — a linkage-named recovery is inadmissible until the original's Failed record is durable, the three re-entry edges trace to the original act through an unbroken chain with broken chains refusing by name, past-window re-entry rejects with a post-suspension fresh act readmitting as two-acts-one-apply, roll-forward demands the fresh-re-discovery attestation by type, and every admission derives from journal bytes alone with no process state and no public constructor for a forged grant. **All five assigned increments are delivered**; what remains is consumer-driven — the platform helper packages re-take the seam-level proofs against real transport, privilege, and durability under their own assignments. See `docs/work-packages/WP-070.md` |
| WP-L100 | Trustworthy read-only product (M1) | Linux read-only inventory and capability adapter | In progress — increment 1 delivered the contract, its seam, and its published reach as `crates/adapter-linux`: the injected read seam returns **bytes**, so every rule this crate declares about a value is decided above the seam where a Tier-1 fake drives it — the entry bound and the byte bound each refusing with the count seen rather than truncating, non-UTF-8 refusing rather than converting lossily, and exactly one trailing newline stripped so a padded vendor is not turned into an absence. ADR-C4's separation is structural rather than documented: reading an attribute requires an `InterfaceAnswered` token that only a successful listing produces (compile-fail-proven unassertable), so a missing attribute cannot be read as a positively determined absence by a caller who cannot show the interface answered, and an interface that did not answer is `unavailable`, never an empty listing. The interface decides MODEL-004's method — `sysfs` direct and so `authoritative`, the `udev` database heuristic and so `inferred`, because a cached third-party computation is not this client's observation — and the INV-003 reach declaration publishes one cell per state in INV-003's order, fixed-size so a missing cell is a compile error, every cell negative on a **deliberately** not-measured basis with the decision recorded (`schemas/adapter-linux/reach.md`). No identity record is emitted and no strength derived, and no test is platform-gated. Devices and identity material, the topology with INV-004's derivations, LIN-006's detection layer, and the capability seam are increments 3–5. Increment 2 delivered devices and their identity material: whole-device enumeration admitted **only** on a positively determined absence of the `partition` attribute, so an unreadable attribute admits nothing rather than promoting a partition into the device list; nine sysfs fields and six database keys, each an attributed observation under its own `interface:native-property` key, electing nothing — the attribute layer’s serial and the database’s serial-shaped key are two properties because they are two interfaces’ different answers, and merging them would manufacture a `conflicting` confidence from values never in conflict; ADR-C4’s separation carried across the database half by a second token producer, so a record that does not exist makes every key `unavailable` while a key missing from a record that does exist is a positively determined absence; and the ADR-0018 transport answer is **`Unrecognized` for every device**, which is that imported obligation’s own terms rather than a shortfall — its fabric-versus-local discrimination rows are outstanding on every platform, so a positive class could come only from vendor documentation — classifying values are recorded on Linux since the 2026-08-13 readback, but a value names no class until those rows say which classes are local. The published reach moved to `implemented-reaches-no-table-state` with every cell still negative on a re-decided not-measured basis, and `schemas/adapter-linux/fields.md` states per field the observability row that supports reading it or an explicit none. **Increment 3a** addressed devices and delivered INV-004's presentable derivation. ADR-0034's first delivery obligation is discharged: a bytes-preserving naming seam beside the delivered text path, which that ADR records as not a lawful naming-input path because it validates UTF-8, refuses non-text, and strips one trailing newline — and the divergence is measured rather than asserted, since the same fixture file holding a lone newline is a positively determined absence through the text path and a one-byte name through this one. The designated serial resolves **structurally** rather than at the measured depth: the walk climbs the ancestor chain and takes the first node answering as a USB device node, with two decoy `serial` attributes — one nearer, one farther — killing the fixed-depth and first-readable readings together. ADR-0034's two outcome rules land with it: a measured absence leaves an operand with a weaker name, a failed read leaves an indeterminate non-operand, and an undesignated cell is read **not at all**, held by a source that records its reads. `total_bytes` comes from FR5's measured 512-byte unit, with every non-decimal, signed, empty and overflowing sector count refusing the address rather than guessing one, and an unaddressable device reported rather than dropped; collision grouping is the domain's own `absorb`. ADR-0033's imported obligation is discharged with a fixture for each arm: alignment presented over authoritative and inferred inputs, withheld with the input's own state surfaced over `unavailable` and `conflicting`, and withheld over an input fit by confidence that carries no usable value. Free extents are **not presented at all**, on INV-004's own ground — this contract builds no partition-table node, so it declares no scheme — which is ADR-0036's second branch, taken and recorded. The first branch was declined on measured grounds rather than for want of a value: `ID_PART_TABLE_TYPE` is carried in the client-readable udev database for loop-attached fixtures and the record measures it wrong on exactly the cases the model exists to represent, reporting `gpt` on an indeterminate table, `gpt` on a damaged one the kernel materialized nothing from, and `PMBR` on a 4Kn GPT disk. The layered topology is **increment 3b**, blocked on that route. Fourteen mutants killed by named tests before proposal; see `docs/work-packages/WP-L100.md` |

Stage labels name Section 13's milestone themes where one exists; the deferred
desktop shell has none while its authority stays retired. Each label is carried
as a `- Stage:` bullet in its assignment document, and Section 14's milestone
column is normative — this column never overrides it.

WP-020 and WP-030 depend only on WP-000 and could begin in parallel. WP-040 is
the first package gated on WP-010. WP-035 depends on WP-000 and WP-020 only,
which is the point: it is the instrument intended to take the outstanding
measurements the register's blockers name as prerequisite evidence, so it
precedes rather than waits on the blocked Section 5 model (WP-010
increment 3).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. `MIT OR Apache-2.0` is the Rust and Tauri ecosystem's standard
dual license; the reasoning, the rejected alternatives, and the rule that
PartMan invokes GPL storage tools as separate processes and never links GPL
libraries are recorded in
[ADR-0006](docs/adr/0006-project-license-and-gpl-tool-boundary.md).

PartMan is free software and every capability it gains is free, including ones
comparable tools sell. It requires no account and no network: SEC-007 mandates
that core functionality work fully offline, and Section 2.1 lists accounts and
cloud services as explicit non-goals.

Contributions are welcome, and are inbound=outbound: unless you state otherwise,
work you submit is offered under the same dual terms, per Apache-2.0 §5. No CLA.
Read [CONTRIBUTING.md](CONTRIBUTING.md) first — the safety constraints in
`AGENT_BUILD_SPEC.md` override everything else, and no pull request may run a
destructive operation against a real disk.
