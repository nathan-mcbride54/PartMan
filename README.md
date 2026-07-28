# PartMan

PartMan is a safety-first, cross-platform disk partition manager defined by
`AGENT_BUILD_SPEC.md` 4.0.0. The intended product is a dark-first Tauri desktop
application plus a scriptable CLI, backed by a shared Rust domain, planner,
validator, journal, image engine, and per-platform privileged helpers.

## Current status

**Not a usable partition manager, and must not be represented as one.** Nothing
here discovers, plans, or mutates storage. There is no GUI, no CLI, no planner,
and no privileged helper.

What does exist:

- Repository foundation: pinned toolchain, three-OS CI, the `xtask` task
  runner, dependency and lint policy (WP-000).
- The `pce/1` canonical encoding: a byte-exact encoder, a strict validating
  decoder, and SHA-256 hashing, implemented in Rust (`crates/domain`) and
  TypeScript (`packages/canonical`) and proven to agree (WP-010, increments 1
  and 2).

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
| CODEOWNERS enforces ownership | Partial — CODEOWNERS requires owner review, but does not reject a diff touching paths outside a work package's assignment |
| T1 fixture generator produces images | Met — `cargo xtask fixtures` produces 13 images deterministically (WP-020 increment 1) |
| Accessibility harness runs | Not started (WP-030) |

The two partial rows are tracked as known gaps in `docs/traceability/WP-000.md`
and `docs/traceability/WP-010.md`. They are recorded rather than rounded up:
a milestone that exits on a criterion nobody verified is worse than one that
exits late.

### M1 through M5

Not started. Their themes and exit criteria are in Section 13 of the
specification and are not restated here, because a second copy of a normative
table is a second copy to drift.

## Work-package order

Section 14 of the specification is normative. Current state:

| Package | Scope | Status |
| --- | --- | --- |
| WP-000 | Repository, CI, `xtask`, CODEOWNERS, dependency policy | Complete |
| ADR-C1 | Canonical encoding and hash strategy | Accepted |
| ADR-C2 … ADR-C5 | Hashed-artifact body/envelope split, identity strength, provenance shape, aggregation vocabulary | Accepted |
| WP-010 | Canonical domain model, schema versioning, encoding and hashing | In progress, blocked at increment 3; see `docs/work-packages/WP-010.md` |
| WP-020 | Disk-image fixture generator and destructive-test interlocks | In progress — increment 1 delivered; see `docs/work-packages/WP-020.md` |
| WP-030 | Design tokens, dark UI shell, accessibility harness | Not started |

WP-020 and WP-030 depend only on WP-000 and could begin in parallel. WP-040 is
the first package gated on WP-010.

## License

None yet. This project is deliberately unlicensed while it is incomplete, so
default copyright applies and no rights to use, copy, modify, or distribute it
are granted. A license will be chosen before the first release.

Until then, do not submit outside contributions: without license terms, the
rights in a contribution are undefined for both sides.
