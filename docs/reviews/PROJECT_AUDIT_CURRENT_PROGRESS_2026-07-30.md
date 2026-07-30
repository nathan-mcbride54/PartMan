# Project audit — current progress, 2026-07-30

Feedback for the next agent, based on `origin/main` at
`02ec95254375c9517b400c7ef45fe55ae1817672` (merge of PR #53).

Read `AGENT_BUILD_SPEC.md` and `AGENTS.md` first. They are normative. This
document reviews the implementation, evidence, handoff, and current roadmap; it
does not replace them.

## Outcome

The latest work materially improved the foundation:

- the action scanner now parses YAML structurally, follows local references,
  sees container images, binds release comments to each occurrence, and checks
  local action metadata containment;
- Unix target acquisition now opens a direct child relative to a held fixture
  directory object;
- changed paths are checked against the work-package assignment at the base
  revision, so a pull request cannot widen its own assignment and then rely on
  that widened copy;
- WP-030 now has reserved application paths and recorded integration decisions;
- the complete local Tier-1, cross-language, and supply-chain gates are green,
  as is the latest GitHub run on all supported operating systems.

The project is still a foundation rather than a partition manager. There is no
inventory, planner, helper, CLI, desktop shell, or real storage-operation flow,
WP-010 increment 3 remains blocked, WP-040 has not started, and no destructive
suite exists. M0 is not complete.

Four newly identified gaps should change the immediate plan:

1. the Dockerfile half of `verify-actions` still has valid-syntax fail-open
   paths;
2. the proposed WP-030/root-Cargo sequencing is circular under `--locked` and
   the new ownership gate;
3. `verify-change-ownership` does not actually require a declaration on every
   commit and can miss the source side of a rename;
4. current status, traceability, test-tier, changelog, and handoff prose have
   drifted from the code again.

Do not start the shell from the current five-step decision as though it were
dependency-ready. First make the integration choreography capable of producing
a green commit under the repository's own lock and ownership rules.

## Scope reviewed

- `AGENT_BUILD_SPEC.md` 4.0.0 and the current repository instructions.
- All changes from the previous reviewed merge (`8132d6c`) through PRs #48,
  #49, #50, #52, and #53.
- `verify-actions`, `verify-ownership`, and `verify-change-ownership`, including
  their regression suites and CI wiring.
- The WP-020 interlock implementation, Unix root-swap regression, Windows
  residual, work-package record, and traceability.
- WP-030's path reservation and shell integration decisions.
- README, CHANGELOG, test-tier documentation, work packages, traceability,
  previous audits, decision notes, progress report, and handoff.
- Current GitHub pull requests, issues #35, #39 and #51, and the latest merged
  CI run.

No production code was changed by this review. The only repository change is
this feedback file.

## Evidence-backed current state

| Area | Current state | Do not claim yet |
| --- | --- | --- |
| WP-000 | Strong three-OS foundation; current gates pass; YAML action discovery, manifest licensing, inventory ownership, and base-revision change ownership exist. | The action dependency scan is not yet fail-closed for Dockerfile syntax. Change ownership does not enforce a real trailer on every commit and misses one side of a detected rename. Generated traceability remains open. |
| WP-010 | Canonical Rust/TypeScript encoding and hashing remain green; fuzz and stable canonicality evidence exist. | Increment 3 domain types remain blocked by the authoritative specification-issue register. No canonical inventory, operation plan, or storage model exists. |
| WP-020 | Deterministic fixtures and a heavily tested SAFE-007 interlock exist. Unix direct-child acquisition is now anchored at an open directory object. Tier 2 and Tier 3 still fail closed. | Windows root containment and pre-existing other-name refusal remain open in issue #51. No destructive suite has consumed an authorization. |
| WP-030 | Tokens, independent policy, mutation evidence, static contrast checks, reserved shell paths, and recorded integration choices exist. | No shell exists. The proposed Cargo sequence is not executable as written, and the proposed colour-literal grep is not strong enough to become the sole enforcement boundary. Rendered and interactive UI-008 evidence is absent. |
| M0 | CI, encoding parity, fixtures, and part of the accessibility harness are real. | M0 is not met: WP-010 is incomplete, WP-030 has no shell, WP-040 has not started, traceability is not generated, and WP-020 has no destructive harness. |

## Findings

### F-01 — High — Dockerfile dependency discovery still fails open

**Requirement:** SEC-010.

YAML discovery is now structural, which genuinely closes the three workflow
mutations in the previous audit. The Dockerfile reached through
`runs.image: Dockerfile` is still interpreted by a small line scanner in
`unpinned_dockerfile_bases`.

That scanner has three unsafe omissions:

1. It explicitly skips any base beginning with `$`:

   ```dockerfile
   ARG BASE=alpine:3.20
   FROM ${BASE}
   ```

   This is valid Dockerfile syntax. `ARG` may precede `FROM` specifically so it
   can supply the base image. The helper reaches `base.starts_with('$')` and
   returns no violation, even though `alpine:3.20` is mutable.

2. It recognizes only `FROM ` and `from `. Dockerfile instructions are
   case-insensitive, so a valid `From alpine:3.20` or `FrOm alpine:3.20` is
   invisible.

3. It skips every comment before classification. A Dockerfile syntax directive
   such as:

   ```dockerfile
   # syntax=docker/dockerfile:1
   ```

   tells BuildKit to pull a Dockerfile frontend. A mutable frontend tag is a
   builder dependency under SEC-010, but the scanner treats the line as an
   ordinary comment and never checks it.

Docker's own reference confirms all three relevant semantics: instructions are
case-insensitive, globally scoped `ARG` instructions may precede and feed
`FROM`, and the syntax directive may cause BuildKit to pull a frontend:
<https://docs.docker.com/reference/dockerfile>.

This has the same dangerous shape as the prior action-scanner defects: an
executable mutable dependency disappears and the gate reports success.
Structural YAML parsing did not make the separate Dockerfile parser structural.

**Required correction:**

1. Fail closed on variable-based base images unless the scanner resolves the
   globally scoped `ARG` to one immutable digest and can prove the build cannot
   override it. The safer initial policy is to reject variable-based `FROM`.
2. Parse instruction names case-insensitively and cover continuation syntax.
3. Treat a `# syntax=` directive as a builder dependency and require an
   immutable digest, or explicitly forbid the directive.
4. Add permanent regressions for `$BASE`, `${BASE}`, mixed-case `FROM`, a
   continued `FROM`, and a mutable syntax frontend.
5. Run deletion sweeps against each branch. A test that merely gets some
   violation from another branch is not evidence that its intended check ran.
6. Narrow the closure wording in CHANGELOG and WP-000 traceability until these
   tests exist.

### F-02 — High — the WP-030 Cargo sequence cannot be green as written

**Requirements:** Section 1.10, SEC-010, and the repository's `--locked`
boundary.

WP-030 now says:

1. a WP-000 pull request adds `apps/desktop/src-tauri` as a workspace member and
   adds its root `Cargo.lock` entry;
2. a later WP-030 pull request builds the shell.

That ordering cannot produce the stated first change:

- before the WP-030 crate exists, Cargo cannot resolve its manifest or compute
  its lockfile entry;
- after the WP-030 crate manifest exists, creating the new workspace package or
  adding any new dependency changes root `Cargo.lock`;
- `Cargo.lock` belongs only to WP-000, while `apps/desktop/**` belongs to
  WP-030;
- the ownership gate rejects a change declaring both packages, and
  `cargo xtask ci` enters through `cargo --locked`, so a WP-030 manifest cannot
  land with a stale lockfile.

This is not limited to Tauri. Any feature-owned Rust manifest that introduces a
new dependency has the same cross-package lockfile edge. The new gate exposed a
repository-wide integration rule that the assignments do not yet model.

**Required correction:**

Before writing shell code, choose and test one complete choreography. Viable
options include:

- pre-register a non-matching workspace-member glob in a WP-000 change, then
  give feature work packages a narrowly documented shared claim on
  `Cargo.lock`, with a semantic/review rule that the lock change is only the
  consequence of their owned manifests;
- temporarily keep the desktop Rust project as an explicitly excluded,
  independently locked sub-workspace, then integrate it in a later WP-000
  change after it exists;
- define a narrow integration work package that owns the application manifest,
  root workspace membership, and lockfile for one atomic integration change.

Each option has costs. The first weakens file-granular ownership of the global
lock; the second creates a temporary second Rust dependency graph; the third
adds governance. Pick one explicitly.

Whichever route is chosen, prove it with a throwaway scaffold before calling the
shell dependency-ready:

- the governance change passes alone;
- the WP-000 preparation passes `cargo xtask ci`;
- the first WP-030 crate commit passes both `cargo xtask ci` and
  `verify-change-ownership`;
- no commit needs a mixed work-package declaration;
- supply-chain scanning covers every resulting lockfile.

### F-03 — Medium — one declared commit covers undeclared commits

**Requirement:** Section 1.10 and the explicit `AGENTS.md` rule that every
commit carries a `Work-Package: WP-0NN` trailer.

`verify_change_ownership` reads every commit body in `base..HEAD`, but
`read_declarations` unions all `Work-Package:` values into one set. The check
passes if that set contains exactly one package. It never checks that each
commit contributed a declaration.

A two-commit pull request therefore passes when:

- commit A has no trailer;
- commit B says `Work-Package: WP-000`;
- the aggregate diff is within WP-000.

The same aggregation applies to governance reasons.

The parser also scans any trimmed body line beginning `Work-Package:` or
`Governance:`. It does not verify Git trailer placement or require a non-empty
governance reason. The implementation and test names say “commit trailer,” but
the accepted syntax is “matching line anywhere in the message.”

This makes the following current claims false:

- AGENTS and CHANGELOG: “Every commit now carries” the trailer;
- WP-000 traceability: the command “requires every commit” to carry it;
- issue #39's closure comment: the ownership half enforces every commit.

**Required correction:**

1. Inspect commits individually.
2. Parse actual trailers with Git's trailer machinery, rather than matching
   arbitrary body lines.
3. Require exactly one non-empty declaration mode per commit: one valid
   `Work-Package` value, or one non-empty `Governance` reason.
4. Require every ordinary commit in a pull request to name the same package;
   require a governance pull request to contain only governance commits.
5. Decide whether branch merge commits are forbidden or must carry a trailer;
   do not silently exempt them while policy says “every commit.”
6. Add multi-commit regressions: missing first trailer, missing last trailer,
   prose lookalike, empty reason, mixed governance/ordinary commits, and two
   packages across separate commits.

### F-04 — Medium — a detected rename checks only its destination

**Requirement:** Section 1.10.

The gate obtains changed paths with:

```text
git diff --name-only <base>...HEAD
```

With rename detection, `--name-only` reports only the destination. A controlled
throwaway repository confirmed:

```text
NAME-ONLY
declared-package/file.txt

NAME-STATUS
R100    other-package/file.txt    declared-package/file.txt
```

A package can therefore rename a file out of another package into one of its
own paths and the gate checks only the destination. The inventory check does
not reliably save this: moving one file out of a broad `directory/**` claim
does not make that claim stale.

**Required correction:**

- simplest: diff with rename detection disabled so the source is a deletion and
  the destination an addition, and check both;
- or parse `--name-status -z` and check both paths for rename/copy records.

Add regressions for cross-package rename in both directions and a within-package
rename that must still pass.

### F-05 — Medium — generated traceability is still present-tense work

**Requirement:** Section 11.7 and Section 12.

Issue #39 is correctly still open for generated traceability. The latest
changes provide more evidence for its urgency:

- WP-000 traceability overstates what the change-ownership gate enforces;
- WP-020's header stops at increment 2b while its table cites increment 2c;
- WP-030's header says increment 1 while its evidence includes the 1a/1b
  remediation;
- README status rows lag three merged pull requests;
- test-tier filesystem claims no longer describe the current gate;
- the historical handoff now points readers to another historical audit.

No work package can honestly be called complete under Section 12 while its
traceability is hand-maintained. Do not defer issue #39 to M4–M5 and do not
close it because the ownership half landed.

The generator should:

- consume machine-readable requirement annotations from implementation and
  tests;
- fail on zero annotations and on an evidence reference that no longer exists;
- generate deterministic per-package documents;
- fail CI on a dirty regeneration;
- separate “establishes,” “supports,” and “known limitation” so a negative test
  cannot be presented as proof of the property it deliberately shows absent.

### F-06 — Medium — documentation currently tells several different stories

**Requirements:** DOC-001, Section 1.10, Section 11.7, and Section 12.

The following should be reconciled in one documentation-only work package after
the gate fixes above, so the corrected text describes the final behavior:

1. **README**
   - WP-000 says change ownership is still open, although a partial gate landed.
   - WP-020 stops at 2b and says the root-swap precondition is open
     universally; 2c closes that form on Unix only.
   - WP-030 says an integration assignment is still needed. The reservation and
     decisions exist; what remains is an executable Cargo/lock choreography.
2. **WP-000 traceability, CHANGELOG, and issue #39**
   - all say every commit is enforced, which F-03 disproves.
3. **WP-020 traceability**
   - the header says increments 1 through 2b while its own evidence table
     includes 2c.
4. **WP-030 traceability**
   - the header omits the 1a/1b remediation represented in its evidence.
5. **`docs/quality/test-tiers.md`**
   - says Tier 1 filesystem access is limited to workflows, two schema files,
     and temporary directories. Current gates also read action metadata,
     Dockerfiles, Cargo/npm manifests, licence files, ownership documents and
     tracked paths, and invoke Git/Cargo metadata. The statement is no longer a
     safe-boundary description.
6. **Historical handoff**
   - its top note says everything except §7 remains accurate, but §8 says the
     README is accurate and §7 includes licence gaps that are already closed.
     Keep the historical rationale, but point its status/next-step claims to
     this review and label the old execution plan archival.
7. **CHANGELOG and WP-020**
   - say the root directory handle “outlives the target handles because
     `Authorization` owns both.” `Authorization::into_targets(self)` returns the
     targets and drops the root field before the returned target handles are
     consumed. The implementation is still safe: after `openat`, the target
     file handles preserve object identity without the directory handle. The
     lifetime claim is simply false and should say the root is held through
     acquisition and verification.
   - the Unreleased section also contains both the old “precondition reopened”
     account and the later Unix closure without clear supersession. Preserve
     the history, but date or mark the older statement as superseded.

The lesson is not “write more status prose.” It is to reduce the number of
manually maintained current-state surfaces and generate the traceability one.

### F-07 — Existing High residual — Windows is not safe for Tier 2

**Requirements:** SAFE-001, SAFE-005, SAFE-007, and SAFE-009.

Issue #51 accurately carries the remaining work:

- Windows still opens `self.path.join(name)` rather than a child relative to a
  held root handle, so the fixture-root replacement attack remains;
- Windows has no stable link-count check, so a pre-existing external hard-link
  alias can be modified by the authorized handle;
- both likely require a small reviewed FFI/helper boundary.

This is not a regression introduced by the latest work. Unix 2c is a real
closure of the scheduled attack on Linux/macOS. Keep the platform distinction
precise:

- Unix: the recorded intermediate-component swap is closed;
- Windows: it is open;
- every platform: Tier 2 remains unavailable because no destructive suite
  exists;
- Windows: Tier 2 remains unavailable even after a suite exists until issue #51
  closes.

Do not let a cross-platform status row round “Unix closed, Windows open” to
either “closed” or “open everywhere.”

## WP-030 design feedback before implementation

The decisions in PR #53 are useful, but decision 5 should be strengthened before
it becomes code.

A grep for hex, `rgb(`, `hsl(`, `oklch(`, and named colours is another lexical
security/policy boundary. It will need to understand comments, test fixtures,
SVG attributes, CSS escapes, template strings, generated files, and new CSS
syntax. This repository already has enough evidence that deny-list text scans
age badly.

A better layered design:

1. Generate CSS custom properties and TypeScript role types from
   `schemas/design-tokens.json`; dirty generation fails.
2. Make components consume a small typed style API, not arbitrary colour
   strings.
3. Validate declared foreground/background role tuples against
   `contrastRules.pairings` at component or story definition time. This catches
   undeclared pairings before the full rendered harness exists.
4. Use CSS/TypeScript-aware lint rules for raw colour syntax as a defense in
   depth, with an explicit allow-list for generated output.
5. In increment 3, inspect computed styles in rendered states to prove the
   declared pair is the one actually displayed.

This preserves the single source of truth without pretending a grep proves
semantic pairing.

## What is genuinely closed from the previous audit

Do not reopen these without new contrary evidence:

- the three YAML action-discovery bypasses from the second follow-up audit;
- job/service/Docker action container collection for literal image values;
- per-occurrence release-tag comments;
- local action metadata symlink containment;
- base-revision ownership lookup and the self-widening rejection;
- Unix final-component and intermediate-component target redirection;
- handle-based target metadata/content verification and consuming handle
  handoff;
- semantic manifest licence checks and both Rust dependency graphs;
- the WP-030 application-path choice (`apps/desktop/`) and reserved UI/token
  package paths.

F-01 is specifically the Dockerfile parser behind otherwise-correct structural
YAML discovery. F-03/F-04 are specifically gaps in the new change-ownership
gate, not reasons to discard its base-revision design.

## Recommended next order

One dependency-ready work package at a time:

1. **WP-000 supply-chain repair:** close F-01 with fail-closed Dockerfile
   parsing and regressions.
2. **WP-000 ownership repair:** close F-03 and F-04, then correct the false
   closure claims about “every commit.”
3. **Governance/integration decision:** choose a Cargo workspace/lock ownership
   choreography and prove its commits can pass independently.
4. **WP-030 increment 2:** scaffold the shell only after step 3 is executable;
   use generated typed tokens and semantic role-pair declarations.
5. **WP-020 Windows interlock:** close issue #51 before any Windows Tier-2 work.
   This does not block independent UI work, but it blocks destructive work.
6. **WP-000 generated traceability:** implement issue #39 early enough that
   packages can eventually meet Section 12, then reconcile the current
   documentation from generated/current sources.
7. **WP-010:** keep increment 3 stopped until the authoritative register's
   blockers are resolved. Do not let visible UI progress imply the product
   model exists.

Issue #35's weekly supply-chain/fuzz scheduling remains worthwhile, but it is
not more urgent than the current fail-open Dockerfile path and does not make the
shell dependency-ready.

## Verification record

| Check | Result |
| --- | --- |
| `cargo xtask ci` | Pass: 196 Rust tests; action, licence, ownership-inventory, token, format, lint, and Tier-1 gates green |
| `cargo xtask cross-language` | Pass: 28 TypeScript tests; shared-vector encode/hash parity green |
| `cargo xtask supply-chain` | Pass: advisories, bans, licences, and sources green for root and fuzz Rust graphs |
| `git diff --check` | Pass before this feedback file |
| Rename experiment | Confirmed `git diff --name-only` reports only the destination of an `R100` rename while `--name-status` reports both |
| Latest GitHub checks | PR #53: all 11 project jobs plus GitGuardian passed: <https://github.com/nathan-mcbride54/PartMan/actions/runs/30506905554> |
| Current open work | Generated traceability #39, Windows interlock #51, scheduled CI #35 |

`cargo xtask cross-language` and `cargo xtask supply-chain` needed the existing
user-level npm/Cargo caches outside the workspace. The recurring Windows
incremental-cache finalization warning did not change any verdict; all commands
exited successfully.

The local machine cannot run the Linux-only real-prober acceptance or the
Unix-only root-swap regression. Both ran green in the latest GitHub matrix.
Fuzz smoke also ran green there.

## Handoff sentence

> Treat the foundation as strong but unfinished: fix the Dockerfile fail-open
> path and make the ownership gate enforce the commit/rename rules it claims,
> prove a non-circular Cargo/lock integration route, then build the shell through
> typed token roles; keep Windows destructive work and package completion
> blocked until issue #51 and generated traceability respectively are closed.
