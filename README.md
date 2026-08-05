# PartMan

PartMan is a safety-first, cross-platform disk partition manager defined by
`AGENT_BUILD_SPEC.md` 5.0.0. The intended product is a dark-first Tauri desktop
application plus a scriptable CLI, backed by a shared Rust domain, planner,
validator, journal, image engine, and per-platform privileged helpers.

## Current status

**Not a usable partition manager, and must not be represented as one.** Nothing
here discovers, plans, or mutates storage. There is no GUI, no planner, and no
privileged helper. A read-only CLI exists — argument parsing, a documented
exit-code contract, a schema-versioned JSON envelope, a typed refusal
vocabulary, a dependency doctor, technology facts, and adapter-attributed
observation records over replayed regular files — and it observes no real
device yet: `partman inspect` without `--replay` answers with a typed
no-adapter statement naming the platform package that changes that, because
printing a plausible empty machine would be a fake success path. ADR-0009's
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
- The `partman` CLI chassis (WP-035 increments 1–4): structured argv that
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
  the standing gated list (SI-28, SI-35, and — a standing decision since
  spec 4.3.0 rather than an open question — ADR-0011) carried in-band in every
  answer, refusals included.

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
destructive Tier 2 and all Tier 3 still refuse because no destructive suite is
registered; a pass over no suite remains forbidden.

Run `cargo xtask help` for the full command list.

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
| T1 fixture generator produces images | Met — `cargo xtask fixtures` produces 13 images deterministically (WP-020 increment 1) |
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
having no such hardware. The macOS second-reader readback is outstanding.
Exit criteria are in Section 13 and are not
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
| WP-010 | Foundations (M0); increment 3 blocked on the spec-issue register | Canonical domain model, schema versioning, encoding and hashing | In progress. SI-31 is resolved by the delivered schema-set boundary, and traceability is generated with a zero-loss migration ledger; increment 3 remains blocked by the authoritative issue register. See `docs/work-packages/WP-010.md` |
| WP-020 | Foundations (M0) | Disk-image fixture generator and destructive-test interlocks | In progress — increments 1–1g, 2a–2d, and 2e delivered. Increment 2e's descriptor-bound loop acceptance **passed on 2026-08-03** in a disposable Proxmox-hosted non-WSL Linux VM — on the implementation commit `2dbf601` and again on the merged commit `c75b340` that lands on main — closing issue #94: the adversarial `LOOP_CHANGE_FD` rebind was detected and discarded, detach and partition teardown were confirmed, and both fixture digests were unchanged. Closing it registered no destructive suite, so increment 2 is unblocked and still unbuilt. The guest was deliberately not network-isolated and the run's exclusions, deviations (a required `snapd` purge), and limits are recorded in `docs/work-packages/WP-020.md` rather than summarized away. Preconditions 1 and 3 are closed on both platforms (issue #51): Unix opens a direct child relative to a held root object, Windows holds the root with a share mode the filesystem enforces, and the other-name refusal — which was a **live defect**, not a missing check — now reads the link count through the authorized handle everywhere. Windows containment is enforcement by the filesystem rather than resolution from a handle, so it is **unproven for roots that are not on a local volume**, and non-local roots are refused. WP-020 traceability is generated from validated source-local claims and typed evidence, with a source-revision/blob ledger preserving every former evidence row, correction, limitation, and residual risk. The sole runnable higher-tier acceptance is `cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only`; it is non-destructive and logical-content-read-only, while every generic destructive Tier-2 request and every Tier-3 request still refuses. See `docs/work-packages/WP-020.md` |
| WP-030 | Foundations (M0); desktop shell deferred, no authority on main | Design tokens, dark UI shell, accessibility harness | In progress — increments 1 through 1c delivered (tokens, the static accessibility harness, and zero-loss generated traceability). Increment 2S's bounded Slint 1.17.1 branch was implemented, measured, mechanically rejected on two hard gates, and closed without merge. Main now retains only normalized evidence, the byte-reproducible 41-row report, and accessibility limitations; no shell exists, UI-002 remains unimplemented, and the rendered half of UI-008 remains untested. The temporary implementation authority was retired by PR #91, so no desktop-shell path is authorized on main and reviving either off-main branch needs fresh governance rather than inertia |

| WP-035 | Evidence (M0.5) | Read-only CLI chassis and evidence instrument | In progress — increments 1–4 delivered the unprivileged CLI chassis, typed schema-versioned refusals, redacted diagnostics, bounded absolute-path dependency doctor, technology facts, and fixture-backed replay observations; increment 5 recorded the operator-run SI-33/SI-35 instruments. The audit reserves `inventory`, `topology`, and `capabilities` as exact typed refusals rather than accepting them as unknown commands, treats a dependency's nonzero exit as failure rather than version evidence, escapes terminal controls, and uses each Unix target's actual replay flags. The SI-33 run moved in L1/L2 but did not measure the required close-before-event/reopen arm; a lower reading across a PnP-arrival interval makes global monotonicity unsafe to assume without characterizing the counter epoch. The Windows SI-35 run found no difference in its retained CIM fields, PhysicalDisk row count, or layout IOCTL, but discarded queried PhysicalDisk property values, so its broader existential hypotheses are inconclusive. The historical WSL2 loop record did not separate healthy from conflicting in its post-hoc-normalized retained projections and did observe 4Kn through an explicit-sector-size loop, but it was taken while issue #94 was open and is non-qualifying pending a descriptor-bound non-WSL rerun. **Issue #94 closed on 2026-08-03** when WP-020 increment 2e's descriptor-bound mechanism landed and passed a real acceptance, and **the descriptor-bound non-WSL rerun was taken the same day and passed as valid on its third sitting** (two void sittings and their recorded instrument amendments precede it, retained rather than discarded): the named candidate client projection is **non-separating** for the decisive healthy/conflicting pair, the historical WSL2 negative is confirmed on qualifying ground, the WSL2 promotion hold is lifted, and **M0.5's loop criterion is satisfied**. SI-33's remaining measurement gaps closed on 2026-08-04: the close-before-event/reopen arm `moved` in 3/3 trials across true no-handle windows on the measured reader, L4 reached its three trials, the counter's re-arrival reset was measured, and the storage-node PDO name qualified as an unprivileged epoch signal while ContainerId and the USB-node name were refuted. The SI-35 Windows completion rerun was taken 2026-08-04 and is valid: W-H1/H2/H3 refuted over completely retained surfaces and W-Q4 answered; the 2026-08-02 wrapper's retention defect did not recur. Separately, SI-35 still requires the chosen option's refusal demonstration. The loop and rerun second-reader obligations are discharged: designated readbacks recorded on their pull requests, then an independent 2026-08-04 retrieve-and-rehash of all six named artifacts, every digest matching. The SI-33/SI-28 successor protocol's S4 collision test is fully executed: on 2026-08-04 a same-model pair produced the predicted outcome — one identical placeholder serial from both units at every serial-bearing layer, after the 2026-08-03 sitting found the earlier pair cross-model — and the card-move sitting completed the final rider, the exchange invisible at every serial surface and unit continuity unverifiable on the shared-constant pair. The successor protocol is fully executed (S1, S2, S2b, S3, and S4). The increment 6 real-partitioned-Linux matrix is executed in full (2026-08-04, disposable Proxmox VM, authorized passthrough fixture media): client/helper signature asymmetry measured both ways, LVM2's member-independent designator helper-only, the SI-34 stale-signature finding established on a real device tree, and UUID-keyed addressing collapsing by silent last-writer-wins under byte-identical media. The increment 6 macOS matrix was taken 2026-08-05 and is valid on its second sitting, the first being void on two recorded harness defects and retained with the amendments it produced: the decisive SI-35 pair is non-separating on macOS too, making it the third platform to answer that way; every non-native signature projects byte-identically to a blank disk while GPT, MBR and APM are each named distinctly; APFS membership and its container UUID are client-readable and the UUID is stable across a verified reboot; and the unprivileged raw read is denied. M10, the privileged comparison leg, was taken the same day in an ephemeral hosted `macos-15` runner at no cost: the client's raw read is denied while root reads the true bytes, and the decisive pair separates for the helper — identical head digests, differing tail digests, placing the disagreement in the backup table no client interface reports — while the four signatures the client called byte-identical to blank each carry a distinct helper digest. Only M9 stays `not established` on Apple Silicon, no preregistered cell on any platform remains `not yet taken`, and the macOS second-reader readback is outstanding. The package remains forbidden every domain surface gated by the open register; see `docs/work-packages/WP-035.md` |

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
