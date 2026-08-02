# Repository instructions

Read `AGENT_BUILD_SPEC.md` in full before changing code. Its safety constraints
and product requirements are normative.

## Repository mechanics

- Work on one dependency-ready work package at a time.
- Create a branch and pull request for each work package. Never push directly to
  the default branch.
- **Put a `Work-Package: WP-<id>` trailer on every commit you write**, naming
  the assignment the change belongs to. The id names an assignment the
  catalogue holds and is not only three digits — Section 14 already lists ids
  like `WP-W100` and `WP-DOC100`, and the checker matches the `WP-` prefix and
  then requires the assignment document to exist at the base revision, so those
  become legal the moment a `Governance:` change lands their assignment. Prose
  narrower than the code here would refuse a legitimate platform package. `cargo xtask verify-change-ownership --base
  origin/main` refuses a change whose paths fall outside that assignment, and CI
  runs it on every pull request. The assignment is read from the **base**
  revision, so widening your own `owned-paths` block in the same change does not
  help — that was the hole an audit found in the inventory-only check.
- It must be a **real git trailer** — a `Key: value` line in the message's last
  paragraph, which is what `git interpret-trailers --parse` recognises. Git's
  parser is the one that answers, so a quoted example in the body does not
  count and a lowercase key does. **Every non-merge commit needs its own**; one
  trailered commit used to cover an untrailered one beside it. Merge commits are
  exempt because neither `gh pr update-branch` nor GitHub's generated
  `refs/pull/N/merge` can carry a trailer, and CI judges the latter.
- A change to the assignments themselves uses `Governance: <reason>` instead, and
  may then edit **only** `docs/work-packages/WP-*.md`. Land it as its own pull
  request before the work that needs the new paths.
- **A lockfile is generated, so any package may carry it — alongside a manifest
  it resolves.** `Cargo.lock` is declared in a `derived-paths` block; a change
  that edits `crates/foo/Cargo.toml` may carry the lockfile churn that follows.
  A lockfile moving *by itself* is refused for every package but its owner,
  because nothing in such a change asks the resolver for a different answer. The
  manifest is matched to the nearest lockfile above it, so `fuzz/Cargo.toml`
  cannot vouch for the root lock.
- Keep edits inside the owned paths listed in the work-package assignment. The
  path checker is file-granular, so a sub-file grant — "this package's own status
  rows in `README.md`" — is still a review obligation, not something the tool
  can see.
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
- Use `cargo xtask tokens` to audit `schemas/design-tokens.json` against
  UI-001, UI-007 and UI-008. It runs inside `cargo xtask ci`. That file is the
  single source of truth for the visual language: when a front end exists it
  must read it rather than keep its own palette, for the same reason the
  canonical vectors are shared. Never weaken a threshold to make a colour pass —
  the first palette failed ten checks and the colours were changed, not the
  floors.
- Use `cargo xtask supply-chain` after installing the pinned versions documented
  in `docs/quality/dependency-policy.md`.
- Pin every GitHub Action to a full commit SHA with the release tag in a
  trailing comment. `cargo xtask verify-actions` enforces this and runs inside
  `cargo xtask ci`. It follows a Docker action into its Dockerfile, where
  **every image the build pulls** must be digest-pinned — `FROM` bases, a
  `# syntax=` BuildKit frontend, `COPY --from=`, and `RUN --mount=…,from=`. A
  base built from a variable is refused rather than resolved.
- Do not add `[build] rustflags` to `.cargo/config.toml`; it escapes the
  workspace. Lint levels belong in `[workspace.lints]`.
- **Every new workspace member needs `[lints] workspace = true` in its
  manifest.** Without it the crate inherits none of `[workspace.lints]`,
  `unsafe_code = "deny"` included, and it compiles clean. `cargo xtask ci`
  refuses a member that omits it. New crates also need a `//!` doc comment: CI
  runs clippy with `-D warnings`, so `missing_docs` is an error there.
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
