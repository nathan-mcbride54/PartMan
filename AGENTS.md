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

