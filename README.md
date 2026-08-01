# PartMan

PartMan is a safety-first, cross-platform disk partition manager defined by
`AGENT_BUILD_SPEC.md` 4.1.0. The intended product is a dark-first Tauri desktop
application plus a scriptable CLI, backed by a shared Rust domain, planner,
validator, journal, image engine, and per-platform privileged helpers.

## Current status

**Not a usable partition manager, and must not be represented as one.** Main
still has no GUI, CLI, planner, or privileged helper, and nothing on any branch
discovers, plans, or mutates storage. Draft PR #89 contains only an off-main,
synthetic Slint shell for ADR-0009's bounded production-feasibility evaluation.

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
- The versioned design tokens and static accessibility harness. In addition to
  WCAG 2.2 AA contrast, redundant non-colour channels, and colour-vision
  separation, the off-main Slint feasibility work defines canonical label IDs,
  theme signals, typography, layout, and cursor vocabularies; generates a
  committed typed `.slint` contract; and supplies a dependency-free Rust
  catalogue, lossless hostile-identifier presentation, and opaque selection
  primitives (WP-030 increments 1 and 2S). Draft PR #89 now AOT-compiles and
  includes generated Rust for a native Winit shell, provides exact single-
  renderer FemtoVG and software configurations, maps the operating-system
  light/dark signal through the generated token adapter, and binds a Rust-owned
  synthetic view model with fail-closed opaque selections and exact byte
  facts. Source, licence, feature-graph, and ambient-input replay remain strict.
  The generated 41-row ADR report rejects it on `G-CFG-08` and `G-SC-01`: the
  current supply-chain result is a hard failure on upstream unmaintained
  dependencies and unresolved licence-policy findings. It is not adopted and
  has no storage or helper boundary. Windows-only unstripped executable sizes
  are retained as non-decisive observations; no package, runtime, accessibility,
  or cross-platform threshold is claimed from them.

The domain crate performs no I/O and launches no process. Tier 2 and Tier 3
test suites fail closed and cannot run at all yet.

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

Tier 2 and Tier 3 still refuse. The SAFE-007 interlock now exists and is
exercised (WP-020 increment 1), but no destructive suite is registered yet, and
reporting a pass for a run of nothing would be a fake success path.

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
| Accessibility harness runs | Partial — `cargo xtask tokens` checks the complete static UI-001/003/007/008/011/013 and PLAN-004 token policy plus the byte-exact generated `.slint` ABI. Desktop and AOT tests compile the native shell with its generated wrappers, exact catalogue, Rust-owned synthetic view model, system light/dark mapping, hostile-identifier display, and opaque selection boundary under exact source, licence, feature, and ambient-input controls (WP-030 increment 2S). No committed platform accessibility-tree capture, assistive-technology transcript, rendered-pixel contrast result, text-spacing/zoom proof, reduced-motion proof, or high-contrast operating-system signal exists, and the candidate currently fails its supply-chain gate |

The three partial rows are tracked as known gaps in
`docs/work-packages/WP-000.md`, `docs/work-packages/WP-010.md`, and
`docs/work-packages/WP-030.md`. They are recorded rather than rounded up: a
milestone that exits on a criterion nobody verified is worse than one that exits
late.

### M1 through M5

Not started. Their themes and exit criteria are in Section 13 of the
specification and are not restated here, because a second copy of a normative
table is a second copy to drift.

## Work-package order

Section 14 of the specification is normative. Current state:

| Package | Scope | Status |
| --- | --- | --- |
| WP-000 | Repository, CI, `xtask`, CODEOWNERS, dependency policy | In progress. Lock boundary, licence, fuzz-graph and ownership-inventory gates delivered; action discovery is now a structural YAML parse after three text-based attempts were each defeated, and the Dockerfile scanner behind it fails closed after nine further bypasses were found and made regressions. Ownership is enforced against a change, not only inventoried, and a generated lockfile may travel with the manifest it follows. `unsafe_code = "deny"` is no longer opt-in per crate. All four current `docs/traceability/WP-*.md` files are generated from source-local test annotations and typed evidence declarations, anchored to real requirement definitions and numeric specification sections, and cross-checked against live tests, tracked/owned paths, and xtask's parser. Each hand-written predecessor has an exact zero-loss migration ledger, and a hand edit fails CI. Issue #39 is closed; WP-000 remains in progress for its other documented gaps rather than being rounded up by this one completion. |
| ADR-C1 | Canonical encoding and hash strategy | Accepted |
| ADR-C2 … ADR-C6 | Hashed-artifact body/envelope split, identity strength, provenance shape, aggregation vocabulary, canonical set ordering and depth | Accepted |
| WP-010 | Canonical domain model, schema versioning, encoding and hashing | In progress. SI-31 is resolved by the delivered schema-set boundary, and traceability is generated with a zero-loss migration ledger; increment 3 remains blocked by the authoritative issue register. See `docs/work-packages/WP-010.md` |
| WP-020 | Disk-image fixture generator and destructive-test interlocks | In progress — increments 1–1g and 2a–2d delivered. Preconditions 1 and 3 are now closed on both platforms (issue #51): Unix opens a direct child relative to a held root object, Windows holds the root with a share mode the filesystem enforces, and the other-name refusal — which was a **live defect**, not a missing check — now reads the link count through the authorized handle everywhere. Windows containment is enforcement by the filesystem rather than resolution from a handle, so it is **unproven for roots that are not on a local volume**, and non-local roots are refused. WP-020 traceability is generated from validated source-local claims and typed evidence, with a source-revision/blob ledger preserving every former evidence row, correction, limitation, and residual risk. Tier 2 stays unavailable on every platform because no destructive suite exists; see `docs/work-packages/WP-020.md` |
| WP-030 | Design tokens, dark UI shell, accessibility harness | In progress — increments 1 through 1c are delivered on main. Draft PR #89 carries the bounded Slint 1.17.1 candidate: strict version-2 tokens, deterministic AOT generation and inclusion, generated style wrappers, a closed catalogue, collision-safe identifier presentation, exact byte formatting, a Rust-owned synthetic four-region shell view model, fail-closed opaque selections, Winit, and exact FemtoVG/software runtime graphs. UI-002 therefore has executable off-main implementation evidence, but no production adoption or rendered cross-platform qualification. Its generated exact-registry report records one pass, two hard failures, and 38 inconclusive rows; ADR-0009 therefore rejects the candidate. The preserved Tauri comparison also remains off-main |

WP-020 and WP-030 depend only on WP-000 and could begin in parallel. WP-040 is
the first package gated on WP-010.

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
