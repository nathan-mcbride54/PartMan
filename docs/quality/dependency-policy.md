# Dependency and supply-chain policy

This policy implements the WP-000 foundation for SEC-010 against
`AGENT_BUILD_SPEC.md` 4.0.0.

## Pinned inputs

- Rust: 1.96.0, selected by `rust-toolchain.toml`.
- Cargo deny: 0.19.4, installed with its published lockfile.
- Cargo audit: 0.22.2, installed with its published lockfile.
- GitHub checkout action: v7.0.1 at immutable commit
  `3d3c42e5aac5ba805825da76410c181273ba90b1`.

GitHub-hosted operating-system labels are fixed to `ubuntu-24.04`,
`windows-2025`, and `macos-15`. No project-controlled builder container image is
used in WP-000. If builder containers are introduced, they must be referenced
by digest. Runner image provenance is recorded by GitHub in each job.

## Rules

- Commit `Cargo.lock` and use `--locked` in CI.
- Reject wildcard Rust dependency versions, except for path dependencies between
  crates in this workspace. Those carry no version requirement, which the check
  reports as `*`, but they resolve to a sibling directory in this repository and
  cannot float to whatever was published most recently — which is the risk the
  rule exists to prevent. A genuine `foo = "*"` from a registry stays denied.
- Deny unknown registries and Git sources unless a reviewed policy change adds
  an exact source.
- Fail CI for known advisories, yanked crates, or disallowed licenses. This
  includes the workspace's own crates: they declare `MIT OR Apache-2.0`
  (ADR-0006) and `[licenses] private` is `ignore = false`, so a workspace member
  that loses its license key fails the same gate a dependency would.
- Pin every GitHub Action to a full commit SHA and retain the release tag in a
  comment for auditability.
- Dependency updates arrive through pull requests and run the full Tier-1 and
  supply-chain gates.
- A future release pipeline must publish an SBOM and dependency/license
  inventory under SEC-005.
- Never link a GPL library. ADR-0006 makes this binding: `libparted`
  (GPL-3.0-or-later) is the named hazard, since it is the obvious dependency for
  a partition editor and linking it would relicense the product by operation of
  law. LGPL libraries such as `libblkid` and `libblockdev` may be linked
  dynamically; GPL *programs* are invoked as separate processes under SAFE-004,
  which carries no such obligation. `cargo deny` cannot enforce this — a `-sys`
  crate declares its own license, not that of the C library it links — so it is
  a review obligation at the integration commit, not an automated check.

## Enforced automatically

`cargo xtask verify-actions` scans `.github/workflows/` and fails if any
`uses:` reference resolves to anything other than a 40-character lowercase
commit SHA (or a `sha256` digest for `docker://` images). Actions committed
inside this repository, which carry no independent supply chain, are exempt.
The check runs inside `cargo xtask ci` on all three operating systems and again
as a Tier-1 unit test, and it fails closed when the workflow directory is
missing or empty, so a renamed directory cannot make it pass vacuously.

A mutable tag such as `@v6` is not a pin: the upstream account can move it onto
new code that would then execute with this repository's credentials.

## Deliberate absence of global `RUSTFLAGS`

`.cargo/config.toml` intentionally defines no `[build] rustflags`. Cargo
discovers that file from the current working directory, so the setting applies
to every crate compiled anywhere inside the repository -- including third-party
dependencies and the `cargo install` invocations above, which build their whole
dependency trees under it. A repository-wide `-D warnings` therefore converts
any new upstream warning into an unrelated job failure, and the exposure grows
with each dependency and each rustc release. Lint levels belong in
`[workspace.lints]`, which Cargo scopes to workspace members; `cargo xtask ci`
promotes the remaining warnings to errors with
`cargo clippy ... -- -D warnings`.

## Node and npm

`packages/canonical` exists because MODEL-005 requires TypeScript and Rust to
produce identical hashes. Its policy mirrors the Cargo one:

- `package-lock.json` is committed, and CI installs with `npm ci`, which fails
  rather than silently resolving a different tree.
- `devDependencies` are pinned to exact versions, not ranges.
- `npm audit --audit-level=moderate` gates CI. It runs inside
  `cargo xtask cross-language` because that is the only gate with a Node
  toolchain; `cargo xtask supply-chain` runs without Node.
- The package has **no runtime dependencies**. Hashing uses Web Crypto and
  testing uses `node:test`, both built in. This is deliberate: the codec is on
  the authorization path, and every dependency there would be one more thing
  whose upgrade could change a hash.
- Node itself is pinned by exact version in the workflow, and
  `actions/setup-node` is digest-pinned like every other action.

`cargo xtask cross-language` is intentionally **not** part of `cargo xtask ci`,
so a contributor working only on Rust need not install Node. CI runs it as its
own job, so the MODEL-005 proof is never merely skipped.

## Pins not covered by Dependabot

Dependabot updates `Cargo.toml`/`Cargo.lock` and workflow action SHAs. It does
not update these, which must be reviewed manually each release cycle:

- `channel` in `rust-toolchain.toml` and the matching `PINNED_RUST_VERSION` in
  `tools/xtask/src/main.rs`, `rust-version` in `Cargo.toml`, and `msrv` in
  `clippy.toml`, which must move together.
- The `cargo install --version` pins for `cargo-deny` and `cargo-audit`, which
  appear in `.github/workflows/ci.yml`, `CONTRIBUTING.md`, and this document.
- The `runs-on` operating-system labels.

## Local commands

```text
cargo install cargo-deny --version 0.19.4 --locked
cargo install cargo-audit --version 0.22.2 --locked
cargo xtask supply-chain
```


## Fuzzing toolchain

`cargo-fuzz` requires nightly, so `fuzz/` is a bounded exception to the single
pinned toolchain. The nightly is pinned by exact date and `cargo-fuzz` by exact
version; both appear in `tools/xtask/src/main.rs` and
`.github/workflows/ci.yml`, must move together, and are covered by neither
Dependabot nor `cargo deny` (the crate is outside the workspace graph). Full
rationale in `docs/quality/fuzzing.md`.
