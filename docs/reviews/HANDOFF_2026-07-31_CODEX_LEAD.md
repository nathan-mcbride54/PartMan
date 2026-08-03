# Lead-agent handoff — 2026-07-31

This file is intentionally **uncommitted** at Nate's request. It is a handoff to
the next agent, not project authority, release evidence, or a substitute for the
normative specification and work-package assignments.

## Read this first

1. Read `AGENTS.md` and all of `AGENT_BUILD_SPEC.md` before changing code.
2. Treat `origin/main` at
   `a0e6ae47fd38e8ee31a5961afafa42a01b8c6e10` as the authoritative finished
   state of this handoff.
3. Do **not** start new work from the checkout that contains this file without
   first examining its branch. `D:\PartMan` is on the old off-main Tauri
   comparison branch, not on `main`.
4. Do not stage or modify `.claude/settings.local.json`. The `.claude/`
   directory was already untracked when this handoff was written and belongs to
   the user.
5. Never push directly to `main`. Use one dependency-ready work package per
   branch and pull request.
6. Every ordinary commit needs a real final
   `Work-Package: WP-0NN` trailer. A governance-only assignment change may edit
   only `docs/work-packages/WP-*.md` and needs the exact applicable
   `Governance: ...` trailer instead.
7. Keep attributing Codex on merge-bound commits with:

   ```text
   Co-authored-by: Codex <267193182+codex@users.noreply.github.com>
   ```

   GitHub recognized `codex` as a commit author on the merged evidence commit,
   not merely as unparsed message text.

## One-paragraph state of the project

PartMan is still a safety-first **pre-product foundation**, not a usable disk
partitioner. Main has no GUI, CLI, storage discovery, planner, mutation path,
privileged helper, or elevation path. It does have strong repository policy,
canonical cross-language encoding and hashing, deterministic synthetic disk
fixtures, a multi-factor disposable-target interlock, real-prober acceptance,
generated traceability, and audited design tokens. The bounded Slint 1.17.1
desktop experiment was implemented off-main, tested, measured, and rejected by
its own immutable gates. Main contains only the normalized rejection evidence
and its non-product report generator. All temporary desktop implementation
authority has been retired.

## Live Git and GitHub state

### Authoritative main

- Repository: `nathan-mcbride54/PartMan`
- Default branch: `main`
- Current main merge: `a0e6ae4` — PR #91, Slint authority retirement
- Prior main merge: `e78a930` — PR #90, evidence-only publication
- Final WP-030 ownership inventory on main: 125 tracked files, 19 shared paths,
  **0 reserved paths**

### Pull requests that matter

- **PR #91 — merged:**
  <https://github.com/nathan-mcbride54/PartMan/pull/91>
  - Governance-only retirement.
  - Merge commit:
    `a0e6ae47fd38e8ee31a5961afafa42a01b8c6e10`.
  - Removed temporary application, root-manifest, `deny.toml`, Dependabot, CI,
    dependency-policy, packaging, and web-package authority.
  - Retained the exact landed token/evidence/accessibility/report surfaces.

- **PR #90 — merged:**
  <https://github.com/nathan-mcbride54/PartMan/pull/90>
  - Evidence-only publication.
  - Merge commit:
    `e78a930beb841af85a4ee4d351c5e0b7e2a25716`.
  - Main gained no Slint, Winit, renderer, AOT compiler, or candidate desktop
    dependency.

- **PR #89 — closed without merge:**
  <https://github.com/nathan-mcbride54/PartMan/pull/89>
  - Preserves the complete rejected native Slint candidate.
  - Candidate implementation checkpoint:
    `359e33101b8fe6ad017d51d7c1fc0f9e5c501288`.
  - Final evidence/report checkpoint:
    `1ef0f0d47bbb6a981b9554b3b7e3691d6ecc43d5`.
  - Do not reopen or merge this branch under the retired assignment.

- **PR #85 — still open as a draft, but must not merge:**
  <https://github.com/nathan-mcbride54/PartMan/pull/85>
  - Immutable Tauri comparison baseline:
    `b0f11249903372d9b9cfba76128479ecfd3917f3`.
  - Current head:
    `4c1fb0301878794f227b26c1d11332668c8c5252`.
  - GitHub currently reports it non-mergeable and its former desktop path
    authority is retired.
  - Its body is stale: it says the toolkit decision remains open. Update the
    body to the final historical result and close the PR without merge, while
    retaining the branch as comparison evidence. Do not update it onto main as
    though it were an active implementation.

### Worktrees present when this handoff was written

- `D:\PartMan`
  - `codex/wp-030-desktop-shell-inc2-v2`
  - Tauri comparison / PR #85
  - Contains pre-existing untracked `.claude/` plus this untracked handoff

- `%USERPROFILE%\AppData\Local\Temp\partman-wp030-evidence-only`
  - `main`
  - Clean and synchronized with `origin/main` at `a0e6ae4`

- `%USERPROFILE%\AppData\Local\Temp\partman-wp000-active-reservation`
  - `codex/wp-030-slint-feasibility`
  - Closed rejected candidate / PR #89

- Several older governance worktrees remain under the Codex visualization
  directory. They are historical branches, not starting points for new work.

Prefer a fresh worktree or branch from current `origin/main` for the next work
package. Do not delete old worktrees casually; they preserve branch context and
may contain user-owned untracked files.

## What was completed during this lead-agent run

The work was broader than the final Slint experiment. The important merged
sequence on main includes:

- Source-backed, generated traceability across all four current work packages,
  including zero-loss migration ledgers and validation against real
  requirements, tracked paths, runnable commands, and live tests.
- Stronger change ownership judged against the **base revision**, so a change
  cannot grant itself wider paths and then use them in the same pull request.
- Correct generated-lockfile ownership: a package may carry root `Cargo.lock`
  churn only with a manifest that actually asks the resolver to change it.
- Workspace-wide lint inheritance checks, including denied unsafe code for new
  members.
- Scheduled dependency maintenance and fuzz resource contracts.
- MODEL-006 / ADR-C6 canonical-set semantics and shared Rust/TypeScript vectors,
  including inherited depth, exact ordering, and duplicate refusal.
- WP-020 traceability remediation and preservation of the Unix/Windows
  containment distinctions and residual risks.
- Multiple governance corrections that made the desktop comparison work
  package dependency-ready before experimentation rather than granting scope
  after implementation.
- ADR-0009, the bounded Slint evaluation, its exact gate inventory, and the
  failure/adoption sequences.
- The complete off-main Slint candidate, the evidence-only main publication,
  and final retirement of the failed candidate authority.

Representative merge PRs, in order, are #65–#84 for traceability, canonical
sets, fuzz/maintenance, and desktop governance; #86–#88 for the Slint ADR and
authorization; #90 for evidence; and #91 for retirement. Read the first-parent
log rather than relying on this compressed list if exact provenance matters.

## Slint evaluation: what actually happened

The user asked for a serious production evaluation because Electron-class
desktop shells are undesirable for this product. Slint was not rejected by
taste or by a superficial prototype. The branch built a native Rust shell with:

- generated bindings from the canonical design-token source;
- Rust-owned catalogue strings and opaque, collision-safe identifier display;
- exact `u64` storage values with preformatted display strings;
- synthetic four-region topology, selection, inspector, and plan surfaces;
- no discovery, planner, mutation, helper, elevation, telemetry, or application
  network path;
- an owned AOT compiler adapter rather than `slint-build`'s broader unused
  build graph;
- exact FemtoVG-only and software-only shipping graphs, with a marked combined
  non-shipping control;
- source-derived environment rejection and feature/target graph validation;
- generated style wrappers plus AST/lowered-IR checks intended to keep the
  canonical token schema authoritative.

That engineering was useful evidence. It was not wasted, and the closed branch
should be preserved. But useful evidence is not the same as a passing product
candidate.

### Mechanical result

The generated ADR registry contains exactly 41 gates:

- 1 pass: `G-CFG-02`
- 2 hard failures: `G-CFG-08` and `G-SC-01`
- 38 inconclusive
- Decision: **rejected**

Raw evidence manifest identity:

```text
pce/1:e1f91437816510737b0ca219c6c246ddc932b6d295a139b8898ce6eb6cbc7d10
```

### Hard supply-chain findings

The required candidate `cargo xtask supply-chain` run failed, and no waiver was
added:

- `RUSTSEC-2026-0206`: reachable `rustybuzz 0.20.1` is unmaintained and had no
  safe upgrade.
- `RUSTSEC-2026-0192`: reachable `ttf-parser 0.25.1` is unmaintained and had no
  safe upgrade.
- `clipboard-win 5.4.1` and `error-code 3.3.2` use BSL-1.0, outside PartMan's
  elected allow-list.
- Cargo-deny's mandatory all-features analysis reached inactive
  `i-slint-renderer-skia 1.17.1`. It deliberately received no exact Slint
  licence exception because every shipping graph prohibited Skia.

No advisory ignore, global licence expansion, Skia exception, royalty-free
Slint exception, or warning downgrade landed.

### Footprint observations, not product claims

One Windows development host produced these exact **unstripped executable**
sizes:

- Tauri baseline: 7,745,536 bytes
- Slint FemtoVG: 11,057,664 bytes (`1.4276x` Tauri)
- Slint software: 11,591,168 bytes (`1.4965x`)
- Slint combined control: 12,225,536 bytes (`1.5784x`)

These are not installers, stripped artifacts, clean-system runtime dependency
bytes, memory measurements, paired latency trials, or cross-platform evidence.
They decide no performance gate. Do not turn them into a slogan that Slint is
always larger; they only disprove any claim that this bounded Windows build had
already demonstrated a smaller executable.

### Evidence that landed on main

- `docs/quality/slint-feasibility-data/evidence.json`
- `docs/quality/slint-feasibility.md`
- `docs/quality/accessibility.md`
- `tools/slint-feasibility/**`
- the fixed `cargo xtask slint-report` check and explicit `--write` mode
- generated WP-030 traceability entries
- historical dependency-policy explanation

The normalized JSON cannot carry a pass/result field, rejects duplicate keys
and unknown fields, and is hashed through PartMan's shared canonical encoding.
The report generator owns the decision algorithm and exact ADR gate registry.
Ordinary CI checks byte freshness; only explicit `--write` regenerates the
Markdown.

## Verification record

The evidence-only branch passed locally:

```text
cargo fmt --all
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --locked -p partman-slint-feasibility --all-targets
cargo xtask slint-report
cargo xtask traceability
cargo xtask ci
cargo xtask test --tier 1
cargo xtask cross-language
cargo xtask supply-chain
cargo xtask verify-change-ownership --base origin/main
```

The governance retirement also passed `cargo xtask ci`, traceability, inventory,
and base-revision change ownership.

Both PR #90 and PR #91 passed the complete GitHub matrix:

- Tier 1 on macOS, Ubuntu, and Windows
- cross-language hash parity on all three platforms
- supply-chain policy on all three platforms
- fuzz smoke
- real-prober acceptance
- GitGuardian

## Current work-package posture

### WP-000

Still in progress. The repository, CI, ownership, lint, action pinning,
dependency, lockfile, fuzz, and generated-traceability foundations are strong.
Do not call the package complete merely because issue #39's traceability work
is complete.

### WP-010

Increments 1, 2, 2a, and 4 are delivered. Increment 3—the Section 5 domain
types with MODEL-003 versioning and MODEL-004 provenance—is blocked by the
authoritative table in `docs/spec-issues/README.md`. Do not copy its blocker
count elsewhere. At this handoff the register says nine items gate increment 3:
six direct decisions, one transitive blocker, and two required inputs.

The next agent may prepare decision material, but must not silently choose
hash-visible schema answers. Several choices would invalidate every future plan
hash if guessed incorrectly.

### WP-020

Fixture generation, evidence, real-prober acceptance, and the SAFE-007
interlock are delivered through increment 2d. Unix uses a held root plus
handle-relative `openat`; Windows uses a held local-volume root whose share mode
prevents replacement. Windows non-local roots are refused because that
containment proof is not established there. Tier 2 and Tier 3 remain unavailable
because no destructive suite exists.

### WP-030

Design tokens, the static accessibility audit, and generated traceability are
delivered. Slint increment 2S is rejected and retired. Tauri comparison 2T is
historical off-main evidence, not a deliverable. No shell exists on main.
Rendered keyboard, screen-reader, reflow, text spacing, high-contrast,
reduced-motion, and real-flow accessibility remain unproven.

## Documentation drift to fix first

This is the one loose end I would fix before substantial new development:

1. `README.md` on main still ends its WP-030 status row with:
   “Temporary implementation authority is pending its governance-only
   retirement.” PR #91 already completed that retirement. Change the row to say
   the authority is retired.
2. Draft PR #85 still says the toolkit decision is open. The decision is closed
   and Slint 1.17.1 was rejected. Update the PR body to historical wording and
   close it without merge, retaining its branch/commit as comparison evidence.

Why this remains: PR #91 was required to be a governance-only commit that
edited only `docs/work-packages/WP-*.md`; mixing the README correction into it
would have violated the repository's own change-ownership rule. The cleanup
should be a small normal WP-030 pull request with a proper trailer, not an
amendment to the governance commit.

Search again for stale future tense after that correction. In particular, do
not let “authorized,” “pending,” “will merge,” or “decision remains open” survive
outside clearly labeled historical sections.

## Recommended next sequence

1. **Close the two documentation-state gaps above.** Keep the PR small and
   reviewable.
2. **Do not immediately start another desktop framework spike.** The project
   has enough UI comparison evidence for now and still lacks its core domain
   model, planner, discovery, and safe execution path.
3. **Ask the decision owner to work through the authoritative WP-010 blocker
   register.** Prepare concise, source-backed decision briefs for SI-11, SI-12,
   SI-27, SI-28, SI-33, SI-34, and SI-35 and the SI-29/SI-30 inputs, but do not
   collapse “mitigated-open” into “resolved.”
4. **Audit WP-020 increment 2 for dependency readiness.** Its prerequisites are
   closed, but Tier 2 must remain fail-closed until a real disposable-target
   suite exists. If the assignment is ready, the loopback/VM harness is useful
   work that can proceed while specification decisions block WP-010.
5. **After WP-010 increment 3 is genuinely unblocked, return to the normative
   work-package order.** WP-040 is the first package gated on WP-010. Do not
   leap ahead to a polished shell over synthetic data while the canonical
   topology/plan boundary is unresolved.
6. **Revisit desktop technology only through fresh governance.** Specification
   4.1.0 still names Tauri 2/React/TypeScript, but the preserved Tauri baseline
   is not authorized to merge and carries its own Linux GTK/glib maintenance
   and unsoundness concerns. A future UI decision should evaluate current
   releases and the then-current supply chain; it must not resurrect either old
   branch as production by inertia.

## My candid view

The project is strongest where it refuses to confuse a plausible demo with a
safety claim. The fixture interlock, base-revision ownership gate, canonical
encoding boundaries, generated traceability, and the decision to reject our own
substantial Slint implementation are all signs of healthy engineering.

The corresponding risk is governance becoming the product. There is now a lot
of machinery proving that work is assigned and documented, while the utility
still cannot enumerate a disk. Do not weaken the safety gates; use them to move
forward in smaller vertical slices. A bounded, read-only capability that shows
one canonical topology from one well-specified adapter will teach the project
more than another broad architecture essay or another UI rewrite—once the
hash-visible model decisions are settled.

Slint itself is not “bad,” and the native prototype had real merits. Slint
1.17.1 simply did not meet PartMan's production bar under the exact graph and
evidence available. The correct lesson is not “never Slint” or “use Tauri at any
cost.” It is that desktop framework selection is subordinate to maintainable
supply chain, accessibility, deterministic configuration, platform support,
and honest measurement. Re-evaluate a future release only when there is new
evidence capable of changing the failed gates.

Finally, keep the documentation brutally literal. “Not implemented,”
“inconclusive,” “comparison-only,” and “rejected” are useful states. They protect
the user from a dangerous false sense of completion and give the next engineer a
clean place to begin.

## Handoff checklist for the next agent

- [ ] Confirm `origin/main` is at or beyond `a0e6ae4`.
- [ ] Confirm the checkout branch before editing anything.
- [ ] Preserve `.claude/` and this uncommitted handoff unless Nate directs
      otherwise.
- [ ] Read the normative spec and the target work package in full.
- [ ] Correct README retirement wording in a normal WP-030 PR.
- [ ] Update and close draft PR #85 without merging it.
- [ ] Run `cargo xtask ci` and the package-specific gates.
- [ ] Run `cargo xtask cross-language` when canonical code/vectors are in scope.
- [ ] Run `cargo xtask supply-chain` for dependency changes.
- [ ] Run `cargo xtask probe` on Linux when fixture/prober behavior is in scope.
- [ ] Verify every commit trailer with `git interpret-trailers --parse`.
- [ ] Run `cargo xtask verify-change-ownership --base origin/main` before push.
- [ ] Add the Codex co-author trailer on future merge-bound Codex work.
- [ ] Never represent PartMan as a usable partition manager yet.

