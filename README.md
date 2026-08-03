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
host-safe boundary. The only runnable higher-tier acceptance is WP-020's named
non-destructive Linux-VM loop check; every generic destructive Tier-2 request
and every Tier-3 request still fails closed.

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

The sole higher-tier exception is:

```text
cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only
```

It runs only with explicit privilege in a disposable non-WSL Linux VM and only
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

The **SI-35 Windows and loop instruments also have 2026-08-02 run records**,
but neither closes the issue. On Windows, no retained `MSFT_Disk`/
`MSFT_Partition` field, `MSFT_PhysicalDisk` row count, or layout-IOCTL value
separated conflicting, damaged-primary, or missing-backup from healthy GPT;
the wrapper discarded queried `MSFT_PhysicalDisk` property values, so the
broader existential hypotheses remain inconclusive. The WSL2 loop run crossed
open issue #94; its normalized client projection and retained `blkid`
properties/`wipefs` offsets did not separate conflicting from healthy, but
that negative is non-qualifying and withheld pending a descriptor-bound
non-WSL rerun. The historical run did record positive 4Kn observability on
that environment. Increment 6's macOS and real-partitioned-Linux matrices,
the SI-33/SI-28 successor protocol, and the SI-35 Windows completion rerun
are preregistered in the
observability record — instruments awaiting hardware and operator sittings,
not evidence. Every cell is `not yet taken` except the successor protocol's
two S4 rows, `not established` after a 2026-08-03 sitting measured the
kit's second reader as a different bridge model than the parent-record
unit; S4 forbids a cross-model substitute. Exit criteria are in Section 13
and are not restated here; M0.5 remains in progress.

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
| WP-020 | Foundations (M0) | Disk-image fixture generator and destructive-test interlocks | In progress — increments 1–1g, 2a–2d, and 2e delivered. Increment 2e's descriptor-bound loop acceptance **passed on 2026-08-03** in a disposable Proxmox-hosted non-WSL Linux VM on commit `2dbf601`, closing issue #94: the adversarial `LOOP_CHANGE_FD` rebind was detected and discarded, detach and partition teardown were confirmed, and both fixture digests were unchanged. Closing it registered no destructive suite, so increment 2 is unblocked and still unbuilt. The guest was deliberately not network-isolated and the run's exclusions, deviations (a required `snapd` purge), and limits are recorded in `docs/work-packages/WP-020.md` rather than summarized away. Preconditions 1 and 3 are closed on both platforms (issue #51): Unix opens a direct child relative to a held root object, Windows holds the root with a share mode the filesystem enforces, and the other-name refusal — which was a **live defect**, not a missing check — now reads the link count through the authorized handle everywhere. Windows containment is enforcement by the filesystem rather than resolution from a handle, so it is **unproven for roots that are not on a local volume**, and non-local roots are refused. WP-020 traceability is generated from validated source-local claims and typed evidence, with a source-revision/blob ledger preserving every former evidence row, correction, limitation, and residual risk. The sole runnable higher-tier acceptance is `cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only`; it is non-destructive and logical-content-read-only, while every generic destructive Tier-2 request and every Tier-3 request still refuses. See `docs/work-packages/WP-020.md` |
| WP-030 | Foundations (M0); desktop shell deferred, no authority on main | Design tokens, dark UI shell, accessibility harness | In progress — increments 1 through 1c delivered (tokens, the static accessibility harness, and zero-loss generated traceability). Increment 2S's bounded Slint 1.17.1 branch was implemented, measured, mechanically rejected on two hard gates, and closed without merge. Main now retains only normalized evidence, the byte-reproducible 41-row report, and accessibility limitations; no shell exists, UI-002 remains unimplemented, and the rendered half of UI-008 remains untested. The temporary implementation authority was retired by PR #91, so no desktop-shell path is authorized on main and reviving either off-main branch needs fresh governance rather than inertia |

| WP-035 | Evidence (M0.5) | Read-only CLI chassis and evidence instrument | In progress — increments 1–4 delivered the unprivileged CLI chassis, typed schema-versioned refusals, redacted diagnostics, bounded absolute-path dependency doctor, technology facts, and fixture-backed replay observations; increment 5 recorded the operator-run SI-33/SI-35 instruments. The audit reserves `inventory`, `topology`, and `capabilities` as exact typed refusals rather than accepting them as unknown commands, treats a dependency's nonzero exit as failure rather than version evidence, escapes terminal controls, and uses each Unix target's actual replay flags. The SI-33 run moved in L1/L2 but did not measure the required close-before-event/reopen arm; a lower reading across a PnP-arrival interval makes global monotonicity unsafe to assume without characterizing the counter epoch. The Windows SI-35 run found no difference in its retained CIM fields, PhysicalDisk row count, or layout IOCTL, but discarded queried PhysicalDisk property values, so its broader existential hypotheses are inconclusive. The historical WSL2 loop record did not separate healthy from conflicting in its post-hoc-normalized retained projections and did observe 4Kn through an explicit-sector-size loop, but it crossed open issue #94 and is non-qualifying pending a descriptor-bound non-WSL rerun. All three instruments therefore have run records, while SI-33's full sequence remains unestablished, the Windows hypotheses remain incomplete, and M0.5's #94-gated loop criterion remains unsatisfied. Separately, SI-35 still requires the chosen option's refusal demonstration. Increment 6's macOS and real-partitioned-Linux measurement matrices, the SI-33/SI-28 successor protocol, and the SI-35 Windows completion rerun are preregistered with every cell `not yet taken` except the successor protocol's two S4 rows, `not established` on a custody-complete 2026-08-03 sitting: the kit's second reader measured as a different bridge model than the parent-record unit, and S4 forbids approximating the comparison with a different model. The package remains forbidden every domain surface gated by the open register; see `docs/work-packages/WP-035.md` |

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
