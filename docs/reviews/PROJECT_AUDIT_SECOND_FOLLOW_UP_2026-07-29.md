# Project audit — second follow-up, 2026-07-29

Feedback for the next agent, based on the current `origin/main` at
`2e3a44eac7937f9640fec6d4f10dfb68a3dc9618` (merge of PR #46).

Read `AGENT_BUILD_SPEC.md` and `AGENTS.md` first. They remain normative. This
document is an evidence-backed review of the implementation and documentation,
not a replacement for either.

## Outcome

PartMan has a strong, unusually well-tested foundation, but it is still not a
usable partition manager and M0 is not met. The latest round genuinely closed
several earlier findings. It also declared two safety properties closed that the
implementation does not establish:

1. `verify-actions` can still silently miss executable mutable dependencies.
   Three planted workflow mutations passed the gate.
2. WP-020's no-follow open protects only the final path component. Rebinding the
   fixture-root directory can still redirect the authorized write to an
   out-of-root file with matching bytes.

Windows also still lacks the other-name/hard-link refusal required before a
destructive consumer is safe. The current share mode does not make an existing
external hard-link alias disposable: a write through the authorized handle
changes every alias to that file.

These are not reasons to discard the work. They are reasons to keep Tier 2
unavailable and reopen the affected claims. The WP-020 gaps block destructive
work, not an independent UI scaffold; the action gate is the universal repair
to land before either track adds more workflow dependencies.

## What this review covered

- `AGENT_BUILD_SPEC.md` 4.0.0 in full and the repository instructions.
- The current tracked tree, history, work-package assignments, traceability
  files, ADRs, quality documents, README, and all review/handoff documents.
- The code and tests in the workspace, with focused review of WP-000,
  WP-010, WP-020, and WP-030.
- Current GitHub state: merged PRs #43–#46, open issue #39, branch protection,
  and the latest required CI run.
- Adversarial mutations of the action scanner, restored after testing.
- Independent review passes over the foundation gates, WP-020, and
  documentation/roadmap state.

No production code was changed by this audit.

## Current project state

| Area | Evidence-backed state | Do not claim yet |
| --- | --- | --- |
| WP-000 | In progress. Licence and lockfile gates are materially stronger; all current gates pass; tracked-path ownership is machine-readable. | Action discovery is not unbypassable. Ownership is not enforced against a PR's work-package identity. Traceability is not generated. |
| WP-010 | In progress and correctly blocked at increment 3 by unresolved model/specification issues, including SI-31. Canonical Rust/TypeScript parity remains green. | No canonical topology, planner, discovery, or storage-operation model is ready for product use. |
| WP-020 | Fixture generation, manifest validation, exact final-component no-follow opening, handle-based verification, consuming file handles, and cursor rewind exist. Tier 2 still refuses. | Root containment is not race-safe; Windows other-name coverage is absent; no destructive suite or consumer exists. |
| WP-030 | The design-token source, policy, mutation suite, and static colour/accessibility checks are valuable and green. | There is no shell. UI-002 is not implemented, and the rendered/interactive parts of UI-008 are untested. Increment 2 still needs integration ownership decisions. |
| WP-040/WP-050 | Not started and dependency-blocked. | No inventory, capabilities, planner, validator, helper, journal, CLI, or real product flow exists. |
| M0 | Partial only. | M0 is not met, and the repository must not be represented as a partition manager. |

## Closures that are genuine

Preserve these rather than reopening the whole previous audit:

- Cargo and npm licence declarations are checked semantically, including
  out-of-workspace manifests. The nested-JSON licence bypass is closed.
- The fuzz lockfile is checked before dependency resolution, and the separate
  fuzz graph participates in supply-chain policy.
- `tokenSetVersion` is pinned outside the token file.
- The exact target-name symlink swap is refused by the Unix `O_NOFOLLOW` /
  Windows reparse-point flags.
- WP-020 verifies metadata, length, and bytes through the opened file object,
  not through the pathname.
- The authorized file cursor is rewound to offset zero before handoff.
- `VerifiedTarget` is non-cloneable and hands a consuming file object to a
  future caller.
- Every currently tracked file is claimed by at least one machine-readable
  work-package ownership block.
- Tier 2 and Tier 3 still refuse rather than reporting success for an empty
  destructive suite.
- The README's blunt product-level statement remains directionally correct.

## Findings, ordered by priority

### F-01 — High — action dependency discovery still fails open

**Requirement:** SEC-010.

`verify_action_pins` (`tools/xtask/src/main.rs:661-758`) scans workflows and
YAML files below the fixed `.github/actions/` directory. It uses a partial key
reader plus a literal `owner/repo@ref` token sweep. Local references beginning
with `./` are exempt at lines 1366-1370. The decision notes and dependency
policy call the sweep syntax-independent and unbypassable.

It is neither.

Three temporary mutations were planted against the current code:

#### Mutation A: decoded `@` hidden behind valid YAML syntax

```yaml
- &pin uses: "actions/checkout\u0040v7"
```

The anchored key is outside the key reader. YAML's double-quoted scalar rules
decode `\u0040` to `@`, so a conforming YAML loader produces
`actions/checkout@v7`. The literal sweep never sees an `@`. The gate exited
successfully and reported six pinned references instead of the baseline seven.

This is the same dangerous failure mode as the two earlier audits: one
executable dependency disappears from the count and silence is reported as
success. The YAML 1.2.2 specification explicitly defines escape sequences in
double-quoted scalars:
<https://yaml.org/spec/1.2.2/#57-escaped-characters>.
GitHub also documents YAML anchors and aliases as supported workflow syntax:
<https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations#yaml-anchors-and-aliases>.

The local audit did not submit this exact anchored-key spelling to GitHub.
GitHub execution is therefore a standards-and-documentation inference; the
gate's silent pass is directly observed. Mutations B and C below use reference
forms GitHub documents explicitly.

#### Mutation B: mutable Docker image without an `@`

```yaml
- &pin uses: docker://alpine:3.20
```

GitHub accepts `docker://image:tag` as a step-level `uses` reference. The
anchored key again avoids the key reader, while the literal sweep requires an
`@`. The gate exited successfully and again reported six references.

GitHub's workflow syntax documents `docker://alpine:3.8` directly:
<https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#example-using-a-docker-hub-action>.

#### Mutation C: local action outside the one scanned directory

The workflow referenced:

```yaml
- uses: ./.github/local-action
```

and `.github/local-action/action.yml` contained:

```yaml
runs:
  using: composite
  steps:
    - uses: actions/cache@v4
```

The workflow's local reference is automatically exempt, and recursive metadata
scanning is hard-coded to `.github/actions/`. The nested mutable remote
reference was not read. The gate exited successfully and reported eight
references, counting the local exemption as acceptable.

GitHub permits local actions at arbitrary repository-relative paths, provided
the directory contains action metadata:
<https://docs.github.com/en/actions/tutorials/create-actions/create-a-composite-action#creating-a-composite-action-within-the-same-repository>.

**Why this is High:** a mutable action or container executes third-party code
with workflow permissions and repository context. The gate's purpose is to
prevent exactly that supply-chain substitution, and a passing result currently
does not prove it inspected every executable reference.

**Required correction:**

1. Reverse the no-parser decision. Parse workflow YAML structurally and inspect
   decoded `uses` nodes rather than searching source spelling.
2. Interpret `uses` in context. A step-level or composite
   `runs.steps[*].uses` node names an action or `docker://` image; a
   `jobs.<id>.uses` node names a reusable workflow. Apply the appropriate
   immutable identifier policy to each remote form.
3. Resolve step-level local `./...` action references using GitHub's
   repository-relative semantics, canonicalize them beneath the repository
   root, require `action.yml` or `action.yaml`, and inspect composite
   `runs.steps[*].uses` nodes.
4. Resolve local `jobs.<id>.uses` references as reusable workflow files and
   structurally inspect them too. Recurse through each kind of local reference
   with a visited set and explicit cycle handling. Do not assume local actions
   live below `.github/actions/`.
5. Preserve the release-tag-comment rule as a source-level auditability layer;
   it need not remain responsible for discovery.
6. Add all three mutations above as permanent regressions. Assert the expected
   reference roster or count as well as individual violations, so disappearing
   references cannot look like a pass.
7. Correct the closure claims in `README.md`,
   `docs/quality/dependency-policy.md`,
   `docs/reviews/AUDIT_RESPONSE_2026-07-29.md`,
   `docs/reviews/DECISION_NOTES_2026-07-29.md`, the progress report,
   CHANGELOG, and `docs/traceability/WP-000.md`.

Putting a maintained, reviewed YAML parser in a dependency-policy tool is a
smaller risk than knowingly interpreting security-relevant YAML incorrectly for
the third time. Pin and audit that parser like every other dependency.

### F-02 — High — WP-020 can follow a replaced fixture-root directory

**Requirements:** SAFE-001, SAFE-005, and SAFE-007.

`authorize` canonicalizes the fixture root once
(`crates/fixtures/src/interlock.rs:286-288`). `verify_target` canonicalizes the
target, performs a string containment check, then opens the saved absolute
pathname at lines 317-382. Unix adds `O_NOFOLLOW`; Windows adds
`FILE_FLAG_OPEN_REPARSE_POINT`.

Those flags protect the trailing component. On Unix, `open(2)` explicitly says
`O_NOFOLLOW` still follows symbolic links in earlier path components:
<https://man7.org/linux/man-pages/man2/open.2.html>.

A deterministic attack sequence is:

1. Start with canonical root `R` and expected target `R/name`.
2. Let target canonicalization and the existing pre-open checks complete.
3. Rename `R` to `R-old`.
4. Put a symlink at the old `R` pathname pointing to an outside directory `E`.
5. Put `E/name` at the expected length and with the expected fixture bytes.
6. The current `open(R/name, O_NOFOLLOW)` follows the intermediate `R`
   symlink, because `name` itself is not a symlink.
7. The saved path strings still satisfy `resolved == root.join(name)`.
8. Handle metadata, single-link count, length, and digest all pass.
9. A future destructive write through that authorized handle changes
   `E/name`, outside the fixture root.

This is the same principle already recorded by
`object_verification_alone_cannot_prove_root_membership`: matching fixture bytes
prove shape, not location or disposability. Increment 2b closed only the
final-component form of the race.

**Required correction:**

1. Establish and retain the fixture-root directory as an object without a
   separate canonicalize-then-open race. Validate that held object's identity;
   do not treat the resulting pathname as the authority.
2. Open a direct child relative to that held directory object, using only the
   catalogue basename. Do not reopen an absolute pathname as the security
   boundary.
3. On Unix, use a reviewed `openat`-style implementation with final-component
   no-follow semantics. Linux may use `openat2` with appropriate
   `RESOLVE_BENEATH`/`RESOLVE_NO_SYMLINKS` restrictions; `openat2` documents
   the distinction:
   <https://man7.org/linux/man-pages/man2/openat2.2.html>.
4. Define and verify the Windows equivalent using stable, handle-based platform
   identity. Keep platform-specific unsafe code, if unavoidable, in a separate,
   narrowly scoped adapter/FFI/helper crate and a reviewed, documented module as
   SAFE-009 requires.
5. Hold the directory object for at least as long as the authorized target
   objects.
6. Add a scheduled root-directory swap regression, not only a final-basename
   swap regression.
7. Reopen WP-020 precondition 1 and correct ADR-0007, WP-020, the decision
   notes, progress report, and traceability claims that say the no-follow open
   cannot leave the root.

Do not implement this by adding more `canonicalize` calls. Path resolution and
the later open would still be separate operations.

### F-03 — High — Windows can authorize a hard link to an outside file

**Requirement:** SAFE-007 disposable-target identity.

`verify_object` refuses `metadata.nlink() > 1` only on Unix
(`crates/fixtures/src/interlock.rs:467-476`). On Windows:

1. An outside ordinary file on the same volume can contain exactly the public
   fixture bytes.
2. Its hard link can be placed at the expected `root/name` before
   authorization.
3. It is a regular file, is not a reparse point, resolves beneath the fixture
   pathname, and matches the compiled name/length/digest.
4. There is no Windows link-count refusal, so authorization succeeds.
5. `FILE_SHARE_READ` prevents competing write/delete opens while the
   authorization lives, but the destructive consumer writes through the
   already authorized writable handle.
6. That write changes the file object through every name, including the
   outside alias.

The share mode therefore narrows concurrent replacement; it does not make a
pre-existing alias safe to destroy.

SI-36 describes SAFE-009 as ambiguous because `crates/fixtures` is not named in
either list. The specification is clearer than that description:

> `unsafe` ... is permitted only in adapter, FFI, and helper crates inside
> reviewed, documented modules.

That forbids unsafe code in the fixture crate altogether and supplies the
intended route: a separate, narrow platform-query adapter/FFI/helper crate with
a reviewed module, or a vetted safe dependency. Treat the necessary ownership
and lint exception as an explicit work package. Do not use the spec-issue
process to turn an implementation-location constraint into permission by
omission.

**Required correction before Windows Tier 2:**

- Query link count and stable file identity through the held handle.
- Refuse any pre-existing other name.
- Add Windows tests for a hard link created before authorization and an
  attempted hard link created while authorization is held.
- State the concurrency/threat boundary if the platform cannot prevent a new
  alias after the check.
- Keep precondition 3 open until the behavior is implemented and tested.

Linux/macOS harness scaffolding need not wait for a Windows implementation, but
no platform's destructive suite should be enabled until F-02 is closed on that
platform.

#### Related Low design note — Unix single-name status is only a snapshot

The Unix `nlink()` guard establishes that the object has one name at the moment
of authorization. A same-user process can add a hard link after that check and
before the destructive write. This is not equivalent to F-03: linking the
already verified synthetic fixture creates another name for disposable fixture
content and cannot replace an existing valuable file.

It is still worth defining the integrity/threat boundary. Existing evidence
covers a hard link present before authorization, not one added afterward. The
Tier-2 design should state whether isolation prevents post-check aliases and add
an attempted-alias test if it claims that property; do not describe the current
snapshot check as durable locking.

### F-04 — Medium — the source-derived token does not prove generator use

ADR-0007, WP-020, and `docs/quality/test-tiers.md` say the token proves the
operator ran the generator and copied its output. A pure function of public
source cannot prove that history. A caller can compute or copy the expected
value without running the generator.

The code proves only that the invocation presented the exact build-derived
value. That is useful accident friction and satisfies a literal “factor is
present” reading of SAFE-007, but it is not evidence of operator provenance.

Correct the wording now. Retain ADR-0007's existing revisit condition:
unattended Tier-2/Tier-3 execution removes the operator whose intent the ADR
claims to witness. The real harness design must then decide whether to store a
nonce or authorization state outside both the source tree and fixture root.

### F-05 — Medium — the future destructive consumer can still reopen a path

The handle API is a strong improvement, but `VerifiedTarget::path()` remains
public for reporting (`crates/fixtures/src/interlock.rs:102`). Nothing currently
forces a future consumer to use `into_file()` rather than reopen the path.

There is no present exploit because there is no consumer. Treat this as an API
design gate for the first Tier-2 implementation:

- the mutation interface should accept the held object, not a pathname;
- the execution component should never receive a path or string; reporting
  should sit behind a separate capability/view boundary;
- external tools that require a path need an explicit safe descriptor/handle
  handoff design rather than `/path` reopening;
- the test must replace the name after authorization and prove the actual
  destructive operation still reaches only the held fixture object.

### F-06 — Medium — ownership inventory is not change ownership

`verify-ownership` usefully proves that all 101 currently tracked files are
claimed, flags stale claims, and reports shared/reserved paths. The code also
honestly states its limit at `tools/xtask/src/main.rs:799-804`: it does not map
a pull request to a work package.

As a result, a feature PR can widen its own `owned-paths` block and then pass
against the widened current tree. Issue #39 remains open on the important half.

Recommended implementation:

1. Add one machine-readable work-package identity to every PR.
2. Compute the changed-path set against the PR's merge base.
3. Evaluate those paths against the ownership catalogue from the base revision,
   not a catalogue widened in the same feature PR.
4. Require assignment/reservation changes to land in a separate governance PR.
5. Fail a PR that names multiple work packages; shared paths should still have
   one owning package for that change.
6. Generate the traceability artifacts from requirement annotations and fail on
   a dirty regeneration.

The present hand-maintained traceability already demonstrates why generation is
needed:

- `docs/traceability/WP-000.md` says all 100 tracked files are claimed; there
  are 101 at the reviewed commit.
- Its header names only SEC-010 even though the evidence table contains SEC-005
  claims.
- `docs/traceability/WP-010.md` says increments 1 and 2 while containing
  increment-4 fuzz evidence.
- `docs/traceability/WP-020.md` says increment 1 while containing increment
  2a/2b evidence.
- `docs/traceability/WP-030.md` says the token audit has 12 mutations; the
  current `mutations()` table contains 26.

### F-07 — Medium — WP-030 increment 2 needs an integration assignment first

The reserved paths are a good response to the earlier scope gap, but they do not
make the Tauri shell fully dependency-ready.

The proposed shell contains a Rust `src-tauri` crate, while the root workspace
has explicit members. Adding it normally changes root `Cargo.toml` and
`Cargo.lock`, both assigned to WP-000; that is the unavoidable current ownership
gap. A TypeScript/React application also needs a declared package manager,
lockfile, scripts/configuration, and Node-version policy. Those can remain under
the reserved `apps/shell/**` path if the design is app-local. Any chosen
root-level Node files would need their own assignment. The specification's
proposed layout says `apps/desktop/`; WP-030 reserves `apps/shell/`. Either can
be valid, but the deviation should be decided rather than inherited
accidentally.

Before shell feature work:

1. Land a small governance/integration-preparation work package that chooses
   `apps/desktop` versus `apps/shell`, the package manager, app-local versus
   root Node configuration, unavoidable root Cargo integration, and exact
   ownership.
2. Decide whether the shell has its own required CI job. If job names or matrix
   entries change, update branch protection in the same coordinated change.
3. Generate a typed accessor from `schemas/design-tokens.json`; do not copy a
   palette into CSS or TypeScript.
4. Add a check that every rendered token pairing is declared by policy.
5. Build honest empty/loading/refusal states. Do not invent topology or planner
   behavior while WP-010 is blocked.
6. Keep UI-008 partial until keyboard, focus, screen-reader, 200% zoom, reduced
   motion, and real rendered contrast are exercised against the shell.

### F-08 — Medium — current handoff documents need a supersession layer

The original `HANDOFF_2026-07-29.md` remains useful for licence rationale,
branch-protection behavior, CI history, project culture, and the warning that
the product does not yet exist. Its “what to do next” section predates the
latest audit/fix rounds and is no longer a safe execution order.

`DECISION_NOTES_2026-07-29.md` is more current, but now contains disproved
claims:

- the action reference sweep is unbypassable;
- WP-020 precondition 1 is closed;
- no-follow opening cannot leave the fixture root;
- the token proves generator use;
- increment 2 is no longer gated;
- SI-36 reflects a specification ambiguity;
- the WP-030 shell is ready to begin without integration preparation.

Do not silently rewrite historical reasoning. Add a prominent correction or
superseded-by link at the top of each affected review document, and maintain one
generated/current status source linked from the README.

Also correct stale primary documentation:

- WP-020's early sections still describe `Authorization` as cloneable and
  pathname-carrying, despite the later handle work.
- WP-020 and the tier guide imply that matching bytes establish disposability.
  Fixture content is public and copyable; safety comes from the conjunction of
  regular-file type, held-root membership, direct-child name, object identity,
  other-name refusal, catalogue name, length, and digest.
- README calls WP-000's action scanning remediated even though F-01 disproves
  that closure. Its WP-020 and WP-030 rows also stop at increment 1 and do not
  reflect later increments or reopened preconditions.
- `docs/traceability/WP-000.md` says WP-000's only filesystem reads are
  `.github/workflows/`. Current gates also read action metadata, manifests,
  ownership declarations/tracked paths, lockfiles, and policy inputs.

## Better way to structure the next work

Keep the repository's one-work-package/one-PR discipline. First, land the one
universal prerequisite:

1. **WP-000 action-pin remediation.** Structural YAML discovery, recursive
   action and reusable-workflow resolution, the three regressions above, and
   correction headers for the false closure claims.

After that repair, the following streams are logically independent. The order
shown is risk-prioritized, not a claim that WP-020 blocks WP-030. If only one
package is active at a time, land them sequentially as separate reviewed PRs:

1. **WP-020 containment and platform identity.** First direct-child open
   relative to a held directory object with intermediate-component swap tests;
   then the narrow Windows adapter/FFI/helper crate, link-count/identity tests,
   and explicit alias semantics. Keep Tier 2 unavailable.
2. **Issue #39 / WP-000 governance.** PR-to-work-package diff enforcement and
   generated traceability, followed by one reconciliation of stale documents.
3. **WP-030 integration and shell.** Decide exact app path, root Cargo edits,
   app-local versus root Node configuration, workflow/check names, and
   ownership; then build token-driven presentation with no fake storage model
   and add rendered accessibility increments.
4. **WP-010 specification/model resolution.** This is the longer
   product-critical path. A polished empty shell is not a partition manager,
   and WP-040/WP-050 remain blocked until the canonical model supports them.

Only then add the **WP-020 destructive harness**, after its platform's
containment and identity properties are proved and the unattended-token and
handle-to-tool designs are resolved.

## Verification record

Against `2e3a44e`:

| Check | Result |
| --- | --- |
| `cargo xtask ci` | Pass; 190 Rust tests across the workspace, formatting, linting, action pins, licences, ownership, and token policy all green |
| `cargo xtask cross-language` | Pass; 28 TypeScript parity tests |
| `cargo xtask supply-chain` | Pass; root graph (24 packages) and fuzz graph (19 packages) |
| Focused token mutation test | Pass; current table contains 26 mutations |
| [Latest GitHub CI for PR #46](https://github.com/nathan-mcbride54/PartMan/actions/runs/30499748594) | Pass; all 11 required project jobs plus GitGuardian green |
| Real prober | Passed in the current Linux GitHub job; not rerun locally because the prober is Linux-only |
| Fuzz smoke | Passed in the current GitHub job |
| Tier 2 / Tier 3 | Intentionally unavailable; no destructive run attempted |
| Action mutation A | Gate incorrectly passed; reported 6 references |
| Action mutation B | Gate incorrectly passed; reported 6 references |
| Action mutation C | Gate incorrectly passed; reported 8 references |

All adversarial workflow mutations and temporary action metadata were restored.
The review branch matched `origin/main` before this document was added.

## Handoff summary

The next agent should inherit this precise statement:

> The repository has strong evidence infrastructure and several genuine
> foundation closures. It has no product implementation. The current green gate
> can miss executable workflow dependencies, and the current fixture interlock
> can be redirected through an intermediate directory component; Windows also
> lacks other-name refusal. Keep destructive tiers disabled, repair those
> boundaries first, generate the ownership/traceability evidence, and only then
> build the shell and harness against explicitly assigned integration paths.
