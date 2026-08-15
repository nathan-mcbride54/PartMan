# Handoff — 2026-08-11, end of round

**From:** Claude (Fable), working with Nate through 2026-08-11; continued
by Codex later the same day.
**To:** whoever picks this up next.
**Pick up here:** §5. The immediate dependency-ready action is to
refresh PR #237 and, with current user authorization, land it. PR #242
is the remaining tooling draft: all 11 repository CI jobs are green,
but GitGuardian was still in progress at the last refresh. Neither PR
was merged by the Codex continuation. Issue #175 still needs the named
Proxmox operator environment and must wait until both PRs land.

> **Untracked local handoff artifact.** `docs/reviews/**` belongs to
> WP-000. Do not stage this into a WP-040 commit. Earlier handoffs sit
> untracked beside it for the same reason.

Repository state at the end of the Fable round: `main` at the PR #240
merge (`de9b7b8`), spec 11.1.0. PRs #238, #239, #241, and #240 all
merged on green that round. Local `cargo xtask ci` passed in the main
checkout at that commit (exit 0, 474 live tests).

Repository state after the Codex continuation: the active checkout is
`work/wp000-worktree-boundary` at `d1c475b`, based on `de9b7b8`, with
no tracked or staged changes. The only local changes are the same 15
untracked `docs/reviews/**` artifacts, including this handoff. The only
open PRs are #242 and #237. Exact live states are in §5; refresh GitHub
before relying on them.

---

## 1. What this round did

The round was WP-040 end to end: merge the in-flight increment 2, then
build, gate, and merge increments 3 and 4, plus one strictness fix the
work surfaced.

**PR #238 (increment 2, streams and reattach vocabulary)** — built in
the prior sitting — merged on green at the round's start.

**PR #239 (increment 3, the redaction boundary)** — SEC-006's
deny-floor at the protocol edge, held the WP-035 way:

- `redaction::FIELD_RULES` classifies every field of every format the
  package owns. The allowlist — the positions that may carry
  identifier-class bytes *at all* — is exactly two entries, each with
  its governing authority named on the rule: the envelope `body` (the
  `schemas/`-defined type the bytes encode governs them) and the
  resume token's `execution` handle (opacity is WP-070's minting
  obligation, said so rather than pretended verified). Every other
  position is structurally identifier-incapable — pinned constant,
  unsigned number, closed tag — so the strict validator is the
  mechanism and the allowlist needs no knowledge of the denied
  classes.
- The handshake `build` — the protocol's one free-entry text position
  — is held to RPC-002's own word for it, a *version*:
  `digits.digits.digits`, optional `+`/`-` suffix over
  `[A-Za-z0-9._+-]`, ASCII, ≤ 64 bytes, enforced at encode **and**
  decode. The refusal (`NotABuildVersion`) names the rule and never
  echoes the value. `partman.rpc.handshake` moved to schema version 2
  for it — the envelope-v2 reviewed-bump-while-no-consumer posture.
- The gate test plants a serial, two path shapes, a spaced label, a
  username, an armored key, and a file name in every non-allowlisted
  position of all three formats — including as an unknown field's own
  key — and each refuses. `schemas/rpc/redaction.md` records the rule
  and, in §4, what a grammar cannot do (a token deliberately shaped
  like a version is the peer's schema violation, not a preventable
  accident).

**PR #241 (increment 4, the authentication skeleton and the record)** —
the closed per-transport claim vocabulary RPC-001 implies:

- `identity::IdentityClaim`: one claim per transport (Windows pipe
  SDDL restriction, Unix socket peer credentials, macOS code-signing
  requirement) as **types naming what a peer proves, verified by
  nobody here**. Each claim's `waits_on` names the route decision its
  verifier arrives with and states that none is recorded — the
  endpoint-less state said per claim rather than left to read as
  oversight.
- **No authorization vocabulary**, deliberately: SI-18 holds the
  severity-1 fresh-authorization question, so the vocabulary names
  identity facts only. The closure test pins the vocabulary by
  exhaustive match, so a new claim, a recorded route, or an
  authorization field fails the suite as a visible reviewed edit.
- `schemas/rpc/authentication.md` records the vocabulary and says
  plainly it is a type vocabulary, not a wire format.
- The record sweep landed in the same PR: README row, CHANGELOG,
  regenerated traceability, and — at Nate's request — a
  `## Delivery status` table in `docs/work-packages/WP-040.md`
  (WP-010's table shape) marking increments 1–4 Delivered and the
  per-OS transports row "Not started — no route recorded, and the
  endpoint-less state is truthful".

**PR #240 (the resume-token size bound)** — increment 3's review pass
noticed `ResumeToken::decode` parsed before bounding, unlike the
envelope's and handshake's decode entries (RPC-004's bound binds
before any parsing). The token travels standalone by design, so it
cannot borrow the envelope's gate. This was spun off as a background
task; Nate ran it in a separate session, which delivered the fix and
PR #240. This round rebased that branch onto post-#241 main —
the only conflict was the generated traceability map, resolved by
taking main's and regenerating (`the_resume_token_shares_the_size_bound`
row present alongside the increment 3–4 rows); the CHANGELOG `### Fixed`
entry merged cleanly into the current Unreleased cycle — re-ran both
gates cold in a clean worktree (exit 0, 474 live tests; ownership 4
paths WP-040), force-pushed, and **merged on green** (`de9b7b8`).

## 2. Decisions taken in-flight, so review can challenge them

1. **Handshake schema v2 for the build grammar.** Tightening what
   decode accepts is a format change; the version says so. Precedent:
   the envelope's v2 bump in increment 2.
2. **`execution` is allowlisted with its obligation named**, not
   constrained: nothing at the protocol layer can verify what
   helper-chosen bytes were derived from, and pretending otherwise
   would be a check that does not exist. The alternative — a length
   bound dressed up as redaction — was rejected as not a redaction
   property.
3. **The unknown-field refusal echoes the key it refuses.** RPC-003's
   strictness refuses *by name* and the name is the violation. The
   tension with never-echo is recorded in `redaction.md` §4 rather
   than silently accepted; surfaces rendering refusals keep their own
   SEC-006 obligations.
4. **SEC-006 traceability rows live in WP-040's map** while the
   assignment lists SEC-006 as consumed-not-claimed ("this package
   applies them at its boundary"). The rows record what the tests
   establish at this boundary; the generator and CI accept them. If
   the next reviewer reads consumed-not-claimed more strictly, the
   annotations are the place to argue.

## 3. Tooling traps this round hit — read before running local gates

1. **`xtask` bakes its repository root at compile time**
   (`repository_root()` uses `env!("CARGO_MANIFEST_DIR")`,
   `tools/xtask/src/main.rs` ~1229). If you build xtask in a
   temporary gate worktree while sharing `CARGO_TARGET_DIR`, the
   binary keeps the worktree's path after the worktree is deleted, and
   every subsequent xtask git launch dies with "The directory name is
   invalid (os error 267)". Both this session and the resume-token
   session hit it independently. Recovery: `cargo clean -p xtask`
   (heavy — it removed ~1.1 GiB of deps too) and rebuild from the real
   checkout. Prevention: give gate worktrees their own target dir.
2. **Local `cargo xtask ci` cannot pass while an agent worktree exists
   under `.claude/worktrees/`**, for two independent reasons: the
   licence walk recurses into the nested checkout and flags its
   manifests as unresolved, and cargo resolves the nested worktree's
   `fuzz/` package against the *outer* repository's workspace and
   errors. Workaround used throughout this round: gate in a clean
   `git worktree add --detach` checkout outside the repository (cold
   build, ~10 min), remove it after.
3. **The manifest-walk fix is now draft PR #242.** The original
   `14ab88b` was reviewed, rebuilt on current `main`, and expanded into
   `d1c475b` (`Work-Package: WP-000`). It stops both the licence and npm
   advisory manifest walkers at an untracked nested checkout boundary,
   but fails closed on tracked gitlinks and nested `.git` markers that
   would hide outer-index source. It also documents the policy and
   regenerates SEC-005/SEC-010 traceability. This closes the two
   project-owned walkers; it does **not** claim to alter Cargo's own
   nested-workspace discovery, so keep disposable gate worktrees
   outside the repository unless a separate reviewed Cargo-boundary
   change lands. §5 has the exact paths, gates, and live PR state.

## 4. Open threads

1. **PR #242 / `work/wp000-worktree-boundary`** — implementation,
   review, local gates, and the 11 repository CI jobs are complete.
   It remains a draft; GitGuardian was still in progress at the last
   refresh. See §5 before changing its state.
2. **PR #237** (dependabot, `@types/node` 26.2.0) — repaired in place,
   fully green, clean, and mergeable, but not merged. See §5.
3. **WP-040's remainder is gated exactly as the assignment sequences
   it**: one transport increment per OS, each behind its own recorded
   route decision with costs stated (the WP-035 increment-10 shape),
   and whatever authorization-requirement field SI-18's resolution
   unlocks (a jointly-sequenced schema change with WP-010 if it enters
   a hashed body). There is no ungated WP-040 work left to pick up.
4. **The 2026-08-08 handoff's decision threads** (SI-39 option (c)
   recommendation, the decision briefs, the WP-020 increment 2 audit
   and plan, issue #175's acceptance re-take) were not advanced by
   this round and stand wherever that document left them.

The stoic-driscoll worktree (the deleted resume-token session's
leftover) was verified clean and removed at round end, which is why
local `xtask ci` passes in the main checkout again.

## 5. Codex continuation — exact pickup state

### 5.1 Next-agent order

1. Refresh PR #237 and get current authorization to merge it. It is
   the only open PR that is non-draft, clean, mergeable, and fully
   green. There is no further implementation work on it.
2. Refresh PR #242. If GitGuardian has completed successfully, get
   current user direction before marking the draft ready or merging
   it. Never weaken or bypass the external check.
3. Do not retake issue #175 before both open PRs land. Each changes
   non-Markdown source after the old acceptance baseline, so an
   earlier retake would immediately become stale by the issue's own
   stopping condition.
4. After #237 and #242 land, issue #175's named Proxmox/non-WSL
   current-HEAD retake is the next evidence slice and remains the gate
   before WP-020 increment 2.

### 5.2 PR #242: the WP-000 checkout-boundary fix

- PR: [#242](https://github.com/nathan-mcbride54/PartMan/pull/242).
- Branch/head: `work/wp000-worktree-boundary` at
  `d1c475b6ffa91372bb326afa74fb8d0f8f400d3f`; base `de9b7b8`.
  The active checkout matches the remote and is three commits ahead
  of live `main`, with no tracked or staged working-tree change.
- Commit sequence, each with a real `Work-Package: WP-000` trailer:
  `a2e6f02` (stop the licence walk at another checkout), `9912c85`
  (trace the boundary into policy), and `d1c475b` (fail closed on
  tracked checkout boundaries).
- Exact changed paths: `tools/xtask/src/main.rs`,
  `docs/quality/dependency-policy.md`, and generated
  `docs/traceability/WP-000.md` (251 insertions, 19 deletions).
- The boundary is shared by licence inventory and npm advisory
  discovery. An index-empty nested `.git` checkout is outside this
  source set; a mode-160000 gitlink or any outer-index descendant
  under a nested marker is committed or ambiguous and refuses closed.
- This does not change Cargo's own nested-workspace discovery. Keep
  disposable gate worktrees outside the repository until a separate
  reviewed Cargo-boundary change says otherwise.
- Local evidence recorded in the PR: `cargo xtask ci`,
  `cargo xtask test --tier 1`, `cargo xtask cross-language`,
  `cargo test --locked -p xtask` (91 tests), ownership against
  `origin/main`, and a real nested linked-worktree proof. Independent
  final review found no P0/P1/P2 issue.
- GitHub at handoff refresh: all 11 repository jobs succeeded across
  Tier 1, cross-language, prober, fuzz, and supply-chain matrices.
  The PR is open and draft. GitHub reports its commit mergeable, but
  overall state is `UNSTABLE` because GitGuardian still reports
  `IN_PROGRESS`. Refresh that external state rather than assuming it.

### 5.3 PR #237: repaired Dependabot update

- PR: [#237](https://github.com/nathan-mcbride54/PartMan/pull/237).
- Root cause of its sole failure: Dependabot's original commit
  `7458645` had no work-package declaration, so Ubuntu's
  `verify-change-ownership` gate correctly refused it.
- With Nate's explicit approval, Codex rebuilt the single commit on
  current `origin/main`, preserved `dependabot[bot]` authorship, added
  the real `Work-Package: WP-010` trailer, and updated the existing
  branch with an exact-SHA force-with-lease. New head:
  `9d2cf1822efa8332485b44b6f127a4c343632661`.
- Exact diff remains the intended two files:
  `packages/canonical/package.json` and
  `packages/canonical/package-lock.json` (6 insertions, 5 deletions).
  `@types/node` is consistently pinned at 26.2.0; the registry
  tarball, SHA-512 SRI, license, and `undici-types ~8.3.0` metadata
  were independently checked. No P0/P1/P2 issue was found.
- Local gates passed: `cargo xtask ci`,
  `cargo xtask test --tier 1`, `cargo xtask cross-language`,
  `cargo xtask supply-chain`, `git diff --check`, and
  `cargo xtask verify-change-ownership --base origin/main`.
- GitHub at handoff refresh: all 11 jobs succeeded; the PR is open,
  non-draft, `CLEAN`, and `MERGEABLE`. It was deliberately not merged
  without a separate instruction.

### 5.4 Cleanup and preservation boundary

- The disposable external repair worktree for #237 was removed after
  the push and green CI result.
- The active checkout stayed on `work/wp000-worktree-boundary` at
  `d1c475b`. No branch switch, merge, or new commit was made while
  updating this handoff.
- All 15 `docs/reviews/**` artifacts remain untracked. This requested
  handoff update is the only artifact changed in this continuation;
  do not stage the review set into PR #242 or #237.
- Issue #175 remains open with no labels, assignees, or comments. Do
  not substitute a local or WSL run for its named Proxmox acceptance
  environment. WP-040's remainder remains decision-gated as §4 says.
