# Dependency and supply-chain policy

This policy implements the WP-000 foundation for SEC-010 against
`AGENT_BUILD_SPEC.md` 4.1.0.

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

- Commit `Cargo.lock` and use `--locked` in CI — **including at the gate's own
  boundary**. The `xtask` alias in `.cargo/config.toml` carries `--locked`,
  because the flags inside the binary bind only once the binary is built. The
  2026-07-29 audit deleted a lockfile entry and watched `cargo xtask ci`
  silently regenerate it while building `xtask`, then pass every test against a
  lockfile the repository had never committed. A Tier-1 test fails if the alias
  loses the flag.
- Commit `fuzz/Cargo.lock` too. The fuzz crate is excluded from the workspace,
  so the root lockfile never covers it; until 2026-07-29 its lock was
  gitignored, and every fresh CI checkout resolved the fuzzer dependencies to
  whatever the registry served that day — outside every gate, on the job that
  executes hostile-byte parser tests. `cargo xtask fuzz` now verifies the lock
  with `--locked` before fuzzing, `cargo xtask supply-chain` checks the fuzz
  graph against this same policy, and Dependabot updates `/fuzz` weekly.
- **A lockfile is generated, so ownership of it is ownership of its inputs.**
  `Cargo.lock` is WP-000's, but every package that adds a crate or a dependency
  rewrites it, and until 2026-07-30 that made the ownership gate refuse both
  halves of any such change. A `derived-paths` block in a work-package document
  declares a path generated; `verify-change-ownership` then lets any package
  carry it, **but only alongside a manifest the lockfile actually resolves**. A
  lockfile moving on its own is not regeneration — nothing in such a change asks
  the resolver for a different answer — and stays the owner's to make.

  The manifest is matched to the nearest lockfile above it, so editing
  `fuzz/Cargo.toml` cannot vouch for the root `Cargo.lock`: `fuzz/` is excluded
  from the workspace and carries its own lock. That was a hole in the first
  version of the rule, found by attacking it rather than by review.

  **What this does not establish:** a re-pin travelling alongside a genuine
  manifest change passes. `--locked` cannot see it either — a transitive
  dependency moved to a different version with a valid checksum still satisfies
  every manifest. Telling the two apart needs the resolver's answer at both
  revisions, which means the base tree and a full resolution on every pull
  request. This is the residual risk the repository has always carried; the
  derived declaration does not widen it, and `cargo deny`, `cargo audit` and
  owner review are what stand against it.
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
- The `Maintenance` workflow re-runs `cargo xtask supply-chain` every Monday at
  06:00 UTC on Windows, Linux, and macOS. This catches a newly published
  advisory or upstream yank even when neither lockfile changed. The scheduled
  workflow is separate from `CI`; all existing per-pull-request jobs and their
  branch-protection names remain unchanged.
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

`cargo xtask verify-actions` scans `.github/workflows/` — and, when present,
every YAML file under `.github/actions/`, because a local composite action's
own `uses:` references are remote supply chain like any other — and fails if
any `uses:` reference resolves to anything other than a 40-character lowercase
commit SHA (or a `sha256` digest for `docker://` images) with the release tag
in a trailing comment. Actions committed inside this repository, which carry no
independent supply chain, are exempt from pinning; their metadata files are
scanned regardless. The check runs inside `cargo xtask ci` on all three
operating systems and again as a Tier-1 unit test, and it fails closed when the
workflow directory is missing or empty, so a renamed directory cannot make it
pass vacuously.

**Discovery is a structural YAML parse.** It took three failed attempts at
reading source text to get here, and the history is the argument for the
dependency:

1. A line reader keyed on `uses:` — defeated by `"uses":`, a quoted key.
2. The same reader plus refusals for shapes it could not parse — defeated by
   `&pin uses:`, an anchored key, which it neither read nor refused.
3. A "syntax-independent" sweep for `owner/repo@ref` tokens — defeated three
   ways at once: `"actions/checkout@v7"` hides the `@` behind a YAML escape
   no text search decodes; `docker://alpine:3.20` is a documented, mutable
   step reference containing no `@` at all; and a local action outside
   `.github/actions/` was never recursed into.

Every attempt reported **success with one fewer reference** — silence shaped
like a pass, the worst failure mode a gate has. Deciding what a YAML document
*says* requires reading it as YAML. `yaml-rust2` is pinned and audited like
every other dependency; interpreting security-relevant YAML incorrectly a
fourth time is the larger risk, and that trade is now settled rather than
re-argued.

Two layers, answering different questions:

- **Discovery and pinning** come from the parsed document. Every `uses` mapping
  key anywhere in the tree is a reference, with its value decoded by the
  parser — context-free on purpose, so a position GitHub adds later cannot be
  missed. Actions and reusable workflows need a 40-character commit SHA;
  `docker://` images need `@sha256:` and 64 hex, because a container tag can be
  repointed. Local `./…` references are resolved wherever they live, required
  to carry `action.yml`/`action.yaml` if they name a directory, required to stay
  inside the repository, and recursed into with a visited set that survives
  cycles. A file that will not parse is a violation, not a skip.
- **Auditability** stays textual. A remote reference must also appear plainly in
  the source with its release tag in a trailing comment, so a reviewer can tell
  which release a digest is. A reference spelled so obscurely that the text
  layer cannot find it fails this check — deliberately, which is what makes
  writing one that way a build failure rather than a way to disappear.

A mutable tag such as `@v6` is not a pin: the upstream account can move it onto
new code that would then execute with this repository's credentials.

The trailing release-tag comment is required and format-checked, but **nothing
verifies that the named tag actually resolves to the pinned SHA** — that would
need the network at gate time. Checking the correspondence is a review
obligation on every action bump, recorded here rather than implied to be
automated.

`cargo xtask verify-licenses` requires every manifest in the repository to
declare `MIT OR Apache-2.0`, and both licence texts to exist. It runs inside
`cargo xtask ci`, and closes the WP-000 gap where `fuzz/Cargo.toml` (outside
cargo-deny's graph) and `packages/canonical/package.json` (outside any Cargo
tooling) could lose their declarations with CI green.

**The checks are semantic, not lexical.** Cargo licences come from
`cargo metadata --locked --no-deps`, which resolves `license.workspace`
inheritance and is the same view the toolchain has; a Cargo manifest that
neither the root workspace nor `fuzz/` includes is a violation, because no
licence gate resolves it. `package.json` is parsed as JSON and the property
must be a string at the document root. The first version matched trimmed lines
and the follow-up audit defeated it by nesting the property under `metadata`:
the line still read `"license": "MIT OR Apache-2.0"` while the document's root
`license` was `undefined`. A line cannot tell you where in a document it sits.

## Known gap: a duplicate major version will not fail CI

`[bans] multiple-versions` is `"warn"`, and `cargo xtask supply-chain` does not
pass `--deny warnings` to `cargo deny`. So two majors of one crate in the graph
produce a message nobody is required to read.

There is a concrete case waiting to happen. `winapi-util 0.1.11` — the safe
`GetFileInformationByHandle` wrapper `crates/fixtures` reads its SAFE-007 link
count through — requires `windows-sys >=0.48.0, <=0.61.*`. `rustix` depends on
`windows-sys` too. The day `windows-sys` 0.62 ships and `rustix` bumps to it,
the graph carries two majors, Dependabot raises the bump, and **nothing flags
the divergence**: the warning prints and the job stays green.

Recorded rather than fixed because the fix is a judgement this note is not
entitled to make on its own. Promoting `multiple-versions` to `"deny"` would
refuse the graph the moment any transitive dependency lags, which is a common
and usually harmless state; leaving it as a warning nobody surfaces is the
status quo, which is worse than it looks because the warning reads as covered.
Whoever next touches the supply-chain gate should decide between denying it,
surfacing the warning count in the job summary, or pinning the pair explicitly.

## Documented deviation: hosted runner images are not digest-pinned

SEC-010 requires CI actions **and builder images** to be pinned by digest. The
actions are; the builder images are not, and cannot be on GitHub-hosted
runners: `ubuntu-24.04`, `windows-2025`, and `macos-15` are labels whose image
contents GitHub updates in place, and GitHub offers no digest-addressed way to
select them. This is a deviation from the normative requirement, not
compliance with it.

Residual risk: a runner-image update can change toolchain-adjacent behaviour
under CI without a commit — mitigated, not eliminated, by the pinned Rust
toolchain, pinned actions, and locked dependency graphs, and by GitHub
recording each run's resolved image version in the job log. Revisit condition:
when release builds exist (ADR-S1, Section 19), decide whether they require
digest-pinned self-hosted or container-based builders; a release artefact built
on a mutable image weakens SEC-010's reproducibility goal in a way a CI test
run does not.

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
- `npm audit --audit-level=moderate` gates CI, for **every** npm package in the
  repository. It runs inside `cargo xtask cross-language` because that is the
  only gate with a Node toolchain; `cargo xtask supply-chain` runs without Node.
  The packages are discovered by walking the tree rather than named, because a
  gate pointed at one directory stops covering the repository as soon as the
  repository grows — WP-030 has `packages/ui/`, `packages/design-tokens/` and
  `apps/desktop/` reserved, and would have brought unaudited manifests into a
  green gate. A `package.json` with no committed `package-lock.json` is a
  violation: `npm audit` would otherwise report on a tree that install time
  decides.
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
`.github/workflows/ci.yml` and must move together. Full rationale in
`docs/quality/fuzzing.md`.

Pinning the runner never pinned the code it resolves and builds, and until
2026-07-29 nothing did: `fuzz/Cargo.lock` was gitignored and the crate sits
outside the workspace graph, so its dependencies were advisory-, licence- and
source-checked by nobody. The lock is now committed, verified with `--locked`
before every fuzz run, checked by `cargo xtask supply-chain` as a second graph
under this same `deny.toml`, and updated by a dedicated `/fuzz` Dependabot
entry.
