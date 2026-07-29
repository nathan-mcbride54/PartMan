# Repository instructions

Read `AGENT_BUILD_SPEC.md` in full before changing code. Its safety constraints
and product requirements are normative.

## Repository mechanics

- Work on one dependency-ready work package at a time.
- Create a branch and pull request for each work package. Never push directly to
  the default branch.
- Keep edits inside the owned paths listed in the work-package assignment.
- Use `cargo xtask ci` as the local Tier-1 gate.
- Use `cargo xtask test --tier 1` for the unprivileged test suite.
- Use `cargo xtask cross-language` for the MODEL-005 Rust/TypeScript hash-parity
  proof. It needs Node and is therefore not part of `cargo xtask ci`; CI runs it
  as its own required job.
- Use `cargo xtask probe` to re-check every fixture against `libblkid` and
  `wipefs`. It needs Linux, so it is not part of `cargo xtask ci`; CI runs it as
  its own job. Expectations live in `crates/fixtures/src/prober.rs`. If it
  disagrees, decide whether a fixture regressed or the prober changed, and say
  which in the commit — do not edit the table until the output matches.
- Both implementations read `schemas/canonical-encoding-vectors.json`. Never
  give either language its own copy of the vectors: an implementation checked
  against a table it also owns proves only self-consistency.
- Tier 2 and Tier 3 are intentionally unavailable until WP-020 provides the
  multi-factor disposable-target interlock required by SAFE-007.
- Use `cargo xtask supply-chain` after installing the pinned versions documented
  in `docs/quality/dependency-policy.md`.
- Pin every GitHub Action to a full commit SHA with the release tag in a
  trailing comment. `cargo xtask verify-actions` enforces this and runs inside
  `cargo xtask ci`.
- Do not add `[build] rustflags` to `.cargo/config.toml`; it escapes the
  workspace. Lint levels belong in `[workspace.lints]`.
- Never commit generated binary disk images, secrets, signing material, or raw
  diagnostic output.
- Every behavior change needs automated evidence and requirement-ID
  traceability.

## Rust policy

- The workspace toolchain is pinned in `rust-toolchain.toml`.
- Formatting and lint warnings fail CI.
- `unsafe` is denied workspace-wide. A future adapter, FFI, or helper exception
  requires an explicitly reviewed module and must remain within SAFE-009.
- External processes are launched with structured argument arrays. Storage
  execution must also satisfy SAFE-004's allow-list, identity, timeout, output,
  and environment requirements.

- Use `cargo xtask fuzz` for the Section 11.4 smoke run. It needs the pinned
  nightly toolchain, so it is not part of `cargo xtask ci`; CI runs it as its
  own job. `fuzz/` is excluded from the workspace and is the only place nightly
  is permitted. See `docs/quality/fuzzing.md`.

## What CI runs, and what it does not

Actions minutes are billed on this private repository, and not uniformly:
**Linux is 1×, Windows 2×, macOS 10×**, with every job rounded up to the minute.
The 45-second macOS leg was therefore the most expensive job in the workflow.

So the jobs are split by what changes their answer, across three workflow files
— a `paths` filter applies to a whole file, which is why they are separate:

| Workflow | Runs on a pull request | Always runs on `main`, weekly, and on demand |
| --- | --- | --- |
| `ci.yml` | Tier 1 on ubuntu and windows, cross-language, prober | plus Tier 1 on macOS |
| `fuzz.yml` | only when `crates/domain`, `packages/canonical`, `fuzz` or `schemas` changed | yes |
| `supply-chain.yml` | only when a manifest, `deny.toml` or the toolchain changed | yes |

Two consequences to hold in mind rather than rediscover:

- **A green pull request has not been checked on macOS**, and has probably not
  been fuzzed or supply-chain scanned. Those run at merge. Do not read a green
  PR as a full gate.
- **There is no branch protection** — GitHub Free does not offer it on private
  repositories — so nothing mechanically prevents merging a red pull request.
  Merging on green is a discipline here, not an enforcement, and "not pending"
  is not the same as "passed".

`cargo xtask ci` remains the local gate and is unaffected by any of this. Prefer
batching commits before pushing: every push to an open pull request starts a
fresh run.
