# Project audit and handoff feedback — 2026-07-29

This review is feedback for the next agent. It audits the repository at
`89aa5deeb69d708135e2726b4cf13e6aed793b83` (`main` after PR #37), including
the handoff in `docs/reviews/HANDOFF_2026-07-29.md`, the normative
`AGENT_BUILD_SPEC.md` 4.0.0, delivered code and tests, work-package records,
Git history, the current GitHub configuration, and the required gates.

No production code was changed. The adversarial mutations described below were
made one at a time, exercised, and restored. The only persistent change from
this audit is this document.

## Executive verdict

The repository is a strong, unusually candid foundation, but it is not as
closed-loop as its green checks suggest. The canonical codec and fixture
evidence are the most mature parts. Current `main` passes the local Tier-1,
cross-language, and supply-chain gates, and the latest GitHub run passed all
eleven protected checks.

There are four newly demonstrated fail-open evidence paths:

1. `cargo xtask` can repair a missing lockfile entry before its internal
   `--locked` commands run.
2. The accessibility file supplies the WCAG thresholds against which that same
   file is judged; a below-AA text colour passed the entire Tier-1 gate after
   the file lowered its own threshold.
3. A semantic token can be deleted consistently from every table and theme
   while the entire Tier-1 gate stays green.
4. The action-pin scanner misses a valid YAML `uses` key when the key is
   quoted.

The fuzz crate is also resolved from an ignored lockfile and lies outside the
advisory, licence, and source-policy graph. These issues should be corrected
before treating WP-000 or WP-030 increment 1 as complete evidence, and before
starting another feature increment.

The handoff is accurate about the repository's public/licensing state, recent
pull requests, current CI, branch protection, WP-010's block, and WP-020's
known interlock limitations. Its main weakness is omission: it inherits the
green gates as proof without testing whether their policy inputs and discovery
mechanisms can themselves be removed or weakened.

## Findings

### High — the committed Cargo lockfile is not enforced at the gate boundary

**Evidence**

- `.cargo/config.toml:1-2` expands `cargo xtask` to
  `cargo run --package xtask --`; the build that loads the gate has no
  `--locked`.
- `tools/xtask/src/main.rs:94-104` and `:437` add `--locked` only after the
  `xtask` binary is already built and running.
- `docs/quality/dependency-policy.md:21` says to commit `Cargo.lock` and use
  `--locked` in CI.
- SEC-010 at `AGENT_BUILD_SPEC.md:600` requires committed lockfiles.

**Reproduction**

The complete `partman-tokens` package entry was removed from `Cargo.lock`, then
`cargo xtask ci` was run. Cargo silently regenerated the missing entry while
building `xtask`; all 160 Rust tests and every internal check passed. The
regenerated file was byte-identical to the committed file, so the command even
left a clean diff.

This means a manifest change without a matching committed lockfile can resolve
new registry state in CI, update the checkout in memory/on disk, and pass the
very command documented as enforcing the lock.

**Required correction**

- Put `--locked` in the alias itself:
  `xtask = "run --locked --package xtask --"`, or invoke
  `cargo run --locked --package xtask -- <task>` directly in CI.
- Add an integration check that starts with an intentionally stale copied
  manifest/lock pair and proves the entry point refuses before running a task.
- Consider a final `git diff --exit-code -- Cargo.lock` defence in CI. It is
  useful evidence, but it should supplement rather than replace a fail-closed
  initial invocation.
- Apply the same boundary rule to every CI entry point that first builds
  `xtask`, not only `ci`.

### High — the accessibility input can lower the standard used to audit itself

**Evidence**

- `crates/tokens/src/audit.rs:152-165` reads each contrast floor from
  `set.contrast_rules.thresholds`.
- `crates/tokens/src/audit.rs:295-297` reads the colour-separation floor from
  `set.color_vision_separation.minimum_delta_e`.
- Both values come from `schemas/design-tokens.json:124` and `:198`.
- `crates/tokens/src/audit/tests.rs:247-262` has only a coarse hard floor of
  3.0 for the tightest *overall* contrast and greater-than-zero for colour
  separation. It does not preserve the 4.5 text floor or the chosen delta-E 12
  floor.
- `docs/traceability/WP-030.md:14-17` claims the gate establishes 4.5:1 text,
  3:1 UI, and mutation-proved checks.

**Reproduction**

The JSON text threshold was changed from 4.5 to 3.0 and light-theme
`text.secondary` was changed to `#7F8899`. `cargo xtask ci` passed all 160
tests while reporting:

```text
tightest contrast: 3.33:1 (light: text.secondary on surface.raised)
```

Normal-size text at 3.33:1 does not meet WCAG 2.2 SC 1.4.3's 4.5:1 floor.
The official W3C sources are
[SC 1.4.3 Contrast (Minimum)](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum)
and
[SC 1.4.11 Non-text Contrast](https://www.w3.org/WAI/WCAG22/understanding/non-text-contrast.html).

This is the exact self-consistency failure the repository warns about for
canonical vectors: the implementation is being checked against policy the
implementation's input also owns. It also contradicts `AGENTS.md:27-31` and
the handoff's `:220` rule never to lower a threshold to make a colour pass.

**Required correction**

- Move normative floors outside the palette being audited. At minimum, define
  versioned policy constants in the audit crate: normal text 4.5, UI/meaningful
  graphics 3.0, and the project's explicitly chosen colour-separation floor.
- If the JSON continues to carry those numbers for front-end consumption,
  require exact agreement with the external policy; do not treat the JSON
  values as authority.
- Add mutations that lower each threshold and keep a non-compliant colour just
  above the lowered value. The complete `cargo xtask ci` command must reject
  them.
- Treat a change to the project-specific delta-E floor as a policy/ADR change
  with explicit evidence, not as an ordinary palette edit.

### High — the semantic token vocabulary is deletable because the palette owns its roster

**Evidence**

- `crates/tokens/src/audit.rs:126-147` uses the current dark theme as the
  reference roster. It checks equality between themes, not equality with the
  product vocabulary.
- `crates/tokens/src/audit.rs:221-270` discovers meaning-bearing roles by
  filtering the same dark-theme keys for three prefixes.
- Contrast and colour-vision coverage iterate only the pairings present in the
  same JSON (`audit.rs:153-156` and `:295-297`).
- `docs/traceability/WP-030.md:15` says every meaningful role is covered, and
  `:50-53` says the file carries every UI-003 and UI-011 role.

**Reproduction**

`entity.container` was removed from all three themes, from
`contrastRules.pairings`, and from `nonColorChannels.roles`. The complete
`cargo xtask ci` gate passed all 160 tests; the token report simply fell from
234 to 228 checks.

A coordinated omission is therefore indistinguishable from a valid smaller
product. The same structure permits an empty or reduced pairing list and a
reduced `mustRemainDistinct` list, subject only to the weak summary assertions.

**Required correction**

- Establish a versioned semantic-token contract independent of the palette:
  required theme names, required role names, allowed threshold kinds, required
  risk/state distinctions, and rules for exhaustive pair coverage.
- Prefer deriving entity and progress roles from the canonical product state
  definitions once WP-010/WP-040 provide them. Until then, keep an explicit
  contract in code or a separately reviewed schema and test exact membership.
- Add deletion mutations for a complete role, a pairing, and a
  `mustRemainDistinct` pair.
- Validate `tokenSetVersion` and `specVersion`. They are currently deserialized
  at `crates/tokens/src/tokens.rs:18-36` but never checked.
- Make parsing genuinely strict. The structs lack
  `#[serde(deny_unknown_fields)]`, even though `tokens.rs:1-7` describes the
  reader as strict. Model intentional `note` fields explicitly before denying
  unknown fields.

### High — a valid YAML spelling bypasses the GitHub Action pin gate

**Evidence**

- `tools/xtask/src/main.rs:744-776` is a line-oriented scanner that recognizes
  only an optional `- ` followed by the exact unquoted prefix `uses:`.
- The tests at `tools/xtask/src/main.rs:1089-1120` cover quoted *values*, not
  quoted mapping keys, alternate whitespace, or flow mappings.
- `docs/quality/dependency-policy.md:50-56` says every workflow `uses:`
  reference is scanned and that the check fails closed.

**Reproduction**

One pinned checkout step was changed from:

```yaml
uses: actions/checkout@<40-character SHA> # v7.0.1
```

to the valid YAML form:

```yaml
"uses": actions/checkout@v7
```

`cargo xtask verify-actions` succeeded and reported six pinned references
instead of seven. The mutable action was invisible rather than rejected.

**Required correction**

- Parse workflow YAML structurally with a strict parser, then walk every
  mapping entry whose decoded key is `uses`.
- Scan local composite-action metadata under `.github/actions/**/action.yml`
  and `action.yaml` too; exempting a local action is safe only if its own remote
  references are inspected.
- Fail on duplicate keys and unsupported structures rather than interpreting
  them leniently.
- Add quoted-key, whitespace, flow-style, multiline/anchor, and composite-action
  regression cases.
- The trailing tag comment is useful for review, but
  `names_a_release` at `tools/xtask/src/main.rs:726-741` only recognizes
  version-shaped prose. It does not prove that the comment's tag resolves to
  the pinned SHA. Either document that as a review obligation or add a
  networked scheduled verification.

### High — the fuzz crate has an unlocked and ungated dependency graph

**Evidence**

- `fuzz/.gitignore:8` ignores `fuzz/Cargo.lock`; a local ignored lockfile exists,
  but a fresh CI checkout has none.
- `fuzz/Cargo.toml:20-23` uses registry ranges for `libfuzzer-sys` and
  `arbitrary`.
- The root `Cargo.lock` contains neither package because `fuzz/` is excluded
  from the workspace at `Cargo.toml:1-6`.
- `tools/xtask/src/main.rs:562-599` invokes `cargo fuzz run` without a locked
  preflight.
- `docs/quality/dependency-policy.md:119-124` acknowledges that cargo-deny does
  not cover the crate, but discusses only the pinned nightly and `cargo-fuzz`
  executable. Pinning the runner does not pin the code it resolves and builds.
- Dependabot covers Cargo only at `/` (`.github/dependabot.yml:3-7`).

On every fresh fuzz CI run, Cargo can resolve a newer compatible fuzzer
dependency and execute its build scripts without a repository change. Root
`cargo audit`/`cargo deny` do not examine that graph. This violates SEC-010's
committed-lockfile and policy-gate requirements on a CI job designed to execute
hostile-byte parser tests.

**Required correction**

- Commit `fuzz/Cargo.lock` and stop ignoring it.
- Before fuzzing, run a locked metadata/build operation against
  `fuzz/Cargo.toml`; use `--locked` directly with `cargo fuzz` as well if the
  pinned version supports it.
- Run advisory, licence, ban, and source checks against the fuzz manifest/lock
  graph explicitly.
- Add a `/fuzz` Cargo Dependabot entry or document and automate an equivalent
  update process.
- Extend the repository manifest-consistency gate proposed in the handoff to
  cover both declarations *and dependency graphs*. Checking only that
  `fuzz/Cargo.toml` has a `license` key leaves the larger gap open.

### High, known blocker — WP-020 authorization still proves a pathname, not the object used

This is not a newly introduced regression; the handoff and WP-020 record it
honestly. It remains the most important precondition before any Tier-2 write.

- `crates/fixtures/src/interlock.rs:54-63` makes `Authorization` cloneable and
  stores only `Vec<PathBuf>`.
- `interlock.rs:196-209` verifies a target and returns its canonical path.
  Nothing holds the verified file object through attachment and destructive
  use.
- `docs/work-packages/WP-020.md:426-443` already prescribes the correct platform
  work: no-follow open, descriptor/file-identity checks, Windows sharing flags,
  non-cloneable consuming authorization, and replace-after-check tests.

Do not start a loopback/VM destructive consumer until those preconditions are
part of the same bounded increment. Also resolve the known source-derived token:
`interlock.rs:174-188` compares against a manifest derived from the compiled
catalogue, so it is not independent evidence.

### Medium — hosted runner labels do not satisfy SEC-010's builder-image digest rule

- SEC-010 explicitly requires “CI actions and builder images” to be pinned by
  digest (`AGENT_BUILD_SPEC.md:600`).
- `.github/workflows/ci.yml:27-30`, `:53-56`, `:78`, `:103`, and `:142-145` use
  fixed hosted-runner labels. Those labels select images whose contents GitHub
  updates.
- `docs/quality/dependency-policy.md:14-17` records runner provenance and says
  future project-controlled containers will be digest-pinned, but it does not
  identify this as a deviation from the normative requirement.

Immutable hosted Windows/macOS images may be operationally unavailable. If so,
do not imply compliance: file a spec issue or documented deviation with the
residual risk, record the resolved image release/provenance in each run, and
decide whether self-hosted immutable builders are required for release builds.

### Medium — work-package ownership is neither enforceable nor sufficient for the proposed next work

- The normative rule at `AGENT_BUILD_SPEC.md:55` forbids edits outside assigned
  owned paths.
- WP-030 owns only the paths at
  `docs/work-packages/WP-030.md:14-20`. A Tauri shell needs new application,
  package, lockfile, workspace, and likely CI paths that are not assigned.
- WP-020's paths at `docs/work-packages/WP-020.md:13-18` likewise do not reserve
  VM harness, platform scripts, or workflow paths for increment 2.
- Commit `fb3dd06` (WP-030 increment 1) changed `AGENTS.md`, `CHANGELOG.md`,
  root `Cargo.toml`, root `Cargo.lock`, and `README.md` in addition to its
  assigned paths. The PR described the edits, but the assignment did not own
  them.
- `docs/traceability/WP-000.md:38-42` correctly admits that CODEOWNERS cannot
  enforce assignment boundaries.

Before either recommended increment, create an exact, machine-readable
assignment that includes its integration paths, or split shared plumbing into a
separate dependency-ready subtask/PR. Do not repeat the historical pattern of
expanding scope in the PR description after the assignment has already been
violated.

### Medium — WP-000 is reported complete despite failing the specification's definition of done

- `README.md:93` says WP-000 is complete.
- `AGENT_BUILD_SPEC.md:813-823` says a package is complete only when generated
  traceability shows its evidence, among other conditions.
- `docs/traceability/WP-000.md:30-42` says traceability is hand-maintained and
  path ownership is not mechanically enforced.
- This audit additionally demonstrates lock and action-pin fail-open paths, and
  the builder-image rule is not met.

Use “foundation increment delivered; remediation open” or “in progress” until
the normative evidence closes. This does not diminish the substantial work
already delivered; it keeps the status vocabulary faithful.

### Medium — several documentation statements have drifted from the implementation

Correct these with the relevant remediation PRs:

- `README.md:76` says “two partial rows,” but the M0 table at `:71-74` has three:
  schemas, CODEOWNERS, and accessibility.
- The handoff says the README status is accurate
  (`docs/reviews/HANDOFF_2026-07-29.md:272-277`), but WP-000's “Complete” label
  is not compatible with its own known gaps or Section 12.
- `docs/quality/test-tiers.md:47-50` says the token cannot be known without
  generating the fixtures and the target is checked against the generated
  manifest. The token is source-derived, and authorization uses the compiled
  catalogue. `docs/work-packages/WP-020.md:78-88` is the accurate account.
- `docs/quality/test-tiers.md:18-21` omits the design-token file from current
  Tier-1 filesystem reads.
- `docs/traceability/WP-030.md:14-17` and
  `docs/work-packages/WP-030.md:47-55` overstate what the current self-owned
  thresholds and role lists establish.
- `docs/quality/dependency-policy.md:50-56` and
  `docs/traceability/WP-000.md:13-14` overstate action-pin completeness until
  YAML is parsed structurally.
- The fuzzing documents should explicitly say that the fuzz dependency lock and
  policy graph are absent, not only that the nightly toolchain is an exception.

### Medium, already recorded — traceability and two licence declarations are not gated

The handoff accurately carries these gaps:

- Section 11.7 (`AGENT_BUILD_SPEC.md:805-807`) requires generated traceability,
  while `docs/traceability/` is hand-maintained.
- `fuzz/Cargo.toml` and `packages/canonical/package.json` can lose their licence
  declarations with current gates still green
  (`docs/traceability/WP-000.md:50-55`).

Do not wait until WP-DOC100 at M4–M5 to solve a rule that Section 12 already
uses to define completion. Assign an early foundation-remediation package.

## Handoff assessment

### What was verified and should be retained

- The repository is public and dual-licensed `MIT OR Apache-2.0`; ADR-0006 and
  the GPL-library boundary are present and internally consistent.
- The latest `main` run,
  [GitHub Actions run 30458194357](https://github.com/nathan-mcbride54/PartMan/actions/runs/30458194357),
  succeeded in all eleven jobs.
- `main` protection is strict, applies to administrators, requires the eleven
  named contexts, requires conversation resolution, and disables force pushes
  and deletion. Zero approving reviews is a documented solo-maintainer
  trade-off.
- PRs #31, #33, #34, #36, and the handoff PR are reflected in `main`; #32 is
  closed rather than merged.
- [Issue #35](https://github.com/nathan-mcbride54/PartMan/issues/35) is the only
  open issue and preserves per-PR coverage while proposing scheduled runs and
  workflow separation.
- WP-010 increment 3 remains correctly blocked on the unresolved specification
  issues, including SI-31.
- WP-020 accurately identifies pathname/object lifetime, the non-independent
  token, and missing Windows link coverage as increment-2 preconditions.
- The warning that the project has no usable partition-manager product yet is
  correct and important.

### What should change in the handoff's advice

1. Do not start WP-030 increment 2 first. Close the foundation gate failures
   above, or at least assign and sequence them as an explicit prerequisite.
2. Before a UI shell, expand WP-030's owned paths. The present assignment does
   not authorize creating that shell.
3. Replace “the harness will tell you” at handoff `:175` with the narrower
   truth: it will detect violations only while external floors and a required
   roster prevent the input from weakening or deleting its own policy.
4. Keep the “front end uses no undeclared pairing” goal, but implement it
   structurally. Generate typed semantic styles/component variants from the
   token contract and forbid raw colour use in application code. A text scan of
   CSS would repeat the line-oriented Actions-scanner mistake.
5. Expand the handoff's licence-manifest recommendation into a complete
   repository manifest/dependency policy: declarations, committed lockfiles,
   advisory/licence/source checks, and update automation for every independent
   graph.
6. Treat generated traceability as present-tense definition-of-done work, not a
   distant documentation package.

## Faithful progress map

| Area | Evidence-backed status |
| --- | --- |
| WP-000 | **Partial.** Three-OS CI, task runner, lint/tests, current dependency checks, licensing, and protection exist. Initial lock enforcement, structural action discovery, builder-image pinning, owned-path enforcement, and generated traceability do not. |
| ADR-C1…C5 | **Accepted.** Decisions are present and the delivered canonical/fixture work is aligned with them. |
| ADR-0006 | **Accepted.** Licence and GPL boundary are documented; two non-workspace manifest declarations remain ungated. |
| WP-010 | **In progress, blocked at increment 3.** Increments 1, 2, and 4 provide the Rust/TypeScript `pce/1` codec, shared vectors, strict canonical decode, hashing, and two fuzz targets. Cross-language parity passes. The canonical domain schema is not implemented. |
| WP-020 | **In progress.** Thirteen deterministic fixtures, evidence checks, prober acceptance, and a fail-closed pre-consumer interlock exist. No Tier-2/Tier-3 suite exists. Object-lifetime binding, an independent token decision, and Windows link/file identity are hard prerequisites. |
| WP-030 | **In progress; increment-1 implementation exists but its evidence gate needs remediation.** The palette and colour maths are useful. External policy floors and an independent required-role contract are missing. No shell or rendered accessibility evidence exists. |
| M0 | **Not met.** Schema versioning/domain types, enforceable ownership, a complete accessibility harness, and a product shell are absent; the foundation evidence gaps above also remain. |
| Product | **Not started in user-usable terms.** No discovery, planner, validator, GUI, CLI, helper, or storage mutation exists. |

## Recommended next sequence

1. Create one bounded foundation-remediation assignment/PR for the initial
   Cargo lock boundary, structural Actions parsing, the fuzz lock/policy graph,
   repository-wide manifest licence consistency, and the corresponding
   documentation corrections. If that is too broad for one dependency-ready
   package, split it into non-overlapping supply-chain and workflow-scanner
   subtasks with exact paths.
2. Create a bounded WP-030 evidence-remediation increment: external floors,
   required semantic contract, strict/versioned parsing, and the deletion/
   threshold mutations demonstrated here.
3. Add the early generated-traceability and machine-readable owned-path work
   that Section 12 already requires. Reclassify WP-000 honestly while it is
   open.
4. Amend WP-030 increment 2's assignment with exact Tauri/application,
   workspace, package-lock, test, documentation, and CI paths. Build generated,
   typed semantic style APIs; forbid raw palette copies and raw colours in the
   shell.
5. In parallel only where paths do not overlap, design WP-020 increment 2 around
   file-handle lifetime and independent-factor decisions before writing any
   destructive harness code.
6. Keep WP-010 increment 3 blocked until the direct specification blockers are
   resolved; do not infer domain shapes from accepted encoding machinery.

## Verification performed

| Check | Result |
| --- | --- |
| `cargo xtask ci` on the unmodified baseline | Passed: format, clippy, token audit, and 160 Rust tests |
| `cargo xtask cross-language` | Passed: npm audit, TypeScript typecheck, 28 TypeScript tests, Rust/TypeScript shared-vector parity |
| `cargo xtask supply-chain` | Passed: advisories, bans, licences, and sources for the **root workspace graph** |
| `cargo xtask verify-actions` | Passed on the current spelling: seven references reported |
| `cargo xtask tokens` | Passed on the current palette: 234 checks, 3.57:1 tightest contrast, delta-E 21.9 closest declared pair |
| Latest GitHub `main` run | Passed all eleven protected jobs, including Linux prober acceptance and fuzz smoke |
| Branch protection/read-only GitHub audit | Strict/up-to-date requirement and exact eleven contexts confirmed |
| Stale root-lock mutation | **Gate incorrectly passed** and regenerated the missing entry |
| Below-4.5 text plus lowered-file-threshold mutation | **Full Tier-1 gate incorrectly passed** at 3.33:1 |
| Complete `entity.container` deletion mutation | **Full Tier-1 gate incorrectly passed**, evaluating six fewer token checks |
| Quoted mutable YAML `uses` key mutation | **Action gate incorrectly passed**, reporting one fewer reference |

`cargo xtask probe` was not run locally because this audit host is Windows and
the task correctly requires Linux `blkid`/`wipefs`; the protected Linux job
passed. The fuzz targets were not rerun locally because the pinned nightly and
`cargo-fuzz` are separate prerequisites; the protected Linux smoke job passed.
Those green jobs establish the current code's behavior, not the missing fuzz
dependency lock/policy guarantees described above.

All mutations were restored. The pre-existing untracked `.claude/` directory
was not read, edited, or included.
