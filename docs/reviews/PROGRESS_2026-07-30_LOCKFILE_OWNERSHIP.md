# Progress notes — 2026-07-30, lockfile ownership

For the reviewer. This covers one phase: the attempt to start WP-030 increment 2,
what the attempt found, and the changes proposed in response. It is not
normative and does not restate `AGENT_BUILD_SPEC.md` or `AGENTS.md`.

Base for everything below is `02ec952`, with `cargo xtask ci` green (190 tests).

> **Second pass, later the same day.** `PROJECT_AUDIT_CURRENT_PROGRESS_2026-07-30.md`
> arrived after §1–§5 were written, and independently reached this document's own
> finding (its F-02). Every one of its findings was then re-verified rather than
> taken on trust, which confirmed most of them, **refuted or narrowed five**, and
> found **twelve further defects the audit did not reach — three of them in the
> fix proposed here.** §9 and §10 record what changed as a result. Read them
> before §3, which describes only the first pass.

---

## 1. What this phase was meant to be

The second follow-up audit's order put WP-030 increment 2 — the Tauri dark UI
shell — after the WP-020 and governance repairs, all of which have landed.
WP-030's own integration decision 3, added yesterday, said the first step was a
WP-000 pull request adding the workspace member and the lockfile entry, with the
shell following in its own change.

That step cannot be taken. This phase is the finding and the repair, not the
shell.

## 2. What was measured

Three propositions, each run rather than reasoned about.

**The shell cannot be created by WP-030.** A minimal `apps/desktop/src-tauri`
crate plus its `members` entry, committed with `Work-Package: WP-030`:

```
error: this change declares `WP-030`, but 2 path(s) are outside that assignment
  Cargo.lock
  Cargo.toml
```

The lockfile churn is unavoidable and is not about Tauri — a workspace member
with no dependencies at all still adds `[[package]] name = "partman-desktop"`.

**The shell cannot be created by WP-000 either.** The identical tree with the
trailer amended:

```
error: this change declares `WP-000`, but 2 path(s) are outside that assignment
  apps/desktop/src-tauri/Cargo.toml
  apps/desktop/src-tauri/src/main.rs
```

`apps/desktop/**` is WP-030's reservation. Neither package could take the first
step.

**The member line cannot land first.** Cargo refuses to load a workspace whose
member has no manifest, and a glob does not rescue it — `apps/*/src-tauri` and
`apps/*` both fall back to the literal path when they match nothing:

```
failed to read `...\apps\*\Cargo.toml`
```

With the parent directory present the glob *does* expand ("referenced via
`apps/*`") and then demands a manifest in each match. So the ordering decision 3
prescribed would have left `cargo xtask ci` red for everyone in between.
(`exclude = ["apps"]` of a nonexistent path *is* accepted — the one pre-landable
form, and it takes the shell out of `[workspace.lints]`, where `unsafe_code =
"deny"` lives. Rejected for that reason.)

**The deadlock is not WP-030's.** `Cargo.lock` is claimed by WP-000 alone, and
every package that adds a crate or a dependency rewrites it. WP-010 adding a
dependency to `crates/domain` meets the identical wall. The change-ownership gate
landed on 2026-07-30 and no dependency-changing change has been attempted since,
which is the only reason this had not surfaced.

## 3. What is proposed

Two branches, deliberately separate because one is governance and one is code.

### `work/wp-000-derived-lockfile` — `Work-Package: WP-000`

A `derived-paths` block in a work-package document declares a path **generated
rather than authored**. `verify-change-ownership` then lets any package carry it,
**only alongside a manifest that lockfile actually resolves**. A lockfile moving
on its own is refused with its own explanation, because nothing in such a change
asks the resolver for a different answer.

Three properties are worth checking rather than taking on trust:

- **Declaring a path generated is not claiming it.** The inventory check still
  demands an `owned-paths` claim, or "this is generated" becomes a way to make a
  file belong to nobody while the inventory reads as complete.
- **A derivation this tool cannot check is refused, not exempted.** Only
  `Cargo.lock` has a defined derivation. `package-lock.json` is rejected today —
  npm lockfiles are a real future question and adding them means writing their
  rule first.
- **The manifest must be one the lockfile resolves.** The first version of the
  rule accepted any `Cargo.toml` anywhere. Attacking it found the hole
  immediately: `fuzz/` is excluded from the workspace and carries its own
  lockfile, so `fuzz/Cargo.toml` cannot change the root lock — and would have
  unlocked it. A manifest is now matched to the nearest lockfile above it, and
  the candidates are the lockfiles that **exist**, read from the base tree,
  not the ones someone declared.

Four deletion sweeps, each confirmed by breaking the check and watching a named
test go red:

| Check removed | Test that failed |
| --- | --- |
| the manifest requirement (any change may carry the lock) | `a_generated_lockfile_is_regenerated_by_whoever_changes_a_manifest`, `a_derived_path_needs_a_derivation_this_tool_can_check` |
| nearest-lockfile matching (any manifest anywhere) | the same two |
| `validate_derived_pattern` (any path may be declared derived) | `a_derived_path_needs_a_derivation_this_tool_can_check` |
| the inventory's `Owned`-only coverage rule | `declaring_a_path_generated_is_not_claiming_it` |

xtask tests go 39 → 42. `cargo xtask ci` green.

### `work/governance-lockfile-and-shell-manifest` — `Governance:`

Assignment documents only, as the trailer requires.

- WP-000 declares `Cargo.lock` generated. `fuzz/Cargo.lock` is deliberately left
  undeclared: nobody needs to carry it, and an exemption granted before it is
  needed is an exemption nobody is checking.
- WP-030 gains a **shared claim on root `Cargo.toml`, for its own
  `apps/desktop/src-tauri` entry in `members` and nothing else.** A sub-file
  grant, and therefore a review obligation the checker cannot express — the same
  shape as `tools/xtask/**` and WP-030's status rows in `README.md`.
- WP-030's decision 3 is corrected in place, with the measurements above and an
  explicit note that the original was written from the shape of the rules rather
  than from an attempt. The original wording is not silently rewritten; the
  correction says what it replaces.

## 4. The composed proof

The two branches are independent, so they were merged and re-run together.
`cargo xtask ci` green, 42 xtask tests, inventory `102 tracked file(s) claimed
across 4 package(s); 6 shared, 5 reserved`.

Then the probe from §2 was replayed against the composed state:

```
verify-change-ownership: 4 path(s) all belong to WP-030 as assigned at <base>
  regenerated, not authored: Cargo.lock
```

And the counter-case still refuses — a WP-030 change touching only `Cargo.lock`:

```
1 of these are generated files, and a generated file moving on its own is not
regeneration — nothing in this change asks the generator for a different answer.
```

All probe branches were deleted; `main` is untouched.

## 5. What this does not establish

Stated here rather than left to be assumed.

- **A re-pin travelling alongside a genuine manifest change passes.** Moving a
  transitive dependency to a different version with a valid checksum satisfies
  every manifest, so `--locked` accepts it too. Telling that apart from honest
  regeneration needs the resolver's answer at *both* revisions — the base tree
  and a full resolution on every pull request. This is the residual risk the
  repository has always carried; the derived declaration does not widen it, and
  `cargo deny`, `cargo audit` and owner review are what stand against it. It is
  in `docs/quality/dependency-policy.md`, not only here.
- **Permitting lockfile churn is not permitting a Tauri dependency tree.**
  Whether several hundred crates should enter this supply chain is `deny.toml`'s
  question and gets its own reviewed step. Recorded in WP-030's decision 3 so it
  cannot arrive as a side effect of "the shell needs it".
- **The shell still does not exist.** UI-002 is unimplemented, the rendered half
  of UI-008 is untested, and M0's accessibility criterion remains partial. This
  phase removed a blocker; it built no product.

## 6. Suggested next phase

1. **WP-030 increment 2**, now executable as one change: the crate, the member
   line, and the lockfile churn together. Decisions 1–5 in `WP-030.md` stand —
   npm, app-local Node configuration, no new required CI check, and no colour
   literal in the front end. Build the token-accessor package and the
   hex-literal check *with* the first component, while the surface is small
   enough for it to be cheap. Honest empty and refusal states only: WP-010 is
   blocked, so there is no topology to render and inventing one would be the
   fake-success path Section 12 forbids.
2. **WP-030's own README row** still says increment 2 "needs an integration
   assignment first". It is WP-030's row to change and was left alone here on
   purpose; increment 2 should correct it.
3. **Generated traceability (Section 11.7)** remains the oldest open WP-000 gap
   and is still assigned to nobody. Every file under `docs/traceability/` is
   hand-maintained, including the row this change added.
4. **F-03 (Windows other-name refusal)** is untouched by this phase and still
   blocks Windows Tier 2.

## 7. Where to attack this

If the reviewer wants the shortest path to a real finding:

- The plausibility rule keys on a file *named* `Cargo.toml`. A change that edits
  a manifest for an unrelated reason — a description, a lint table — unlocks the
  lockfile just as well as adding a dependency does. That is deliberate (the
  rule cannot know why a manifest moved) but it is the widest part of the door.
- `governing_lockfile` reads the base tree plus the change's own paths. A change
  that *deletes* a nested lockfile in the same commit that edits its manifest is
  a case nothing tests.
- The derived claim is matched with `claim_matches`, so `crates/**` in a
  `derived-paths` block is refused by basename — but a path literally named
  `something/Cargo.lock` anywhere in the tree would be accepted as a declaration.
  Only WP-000's document declares one today.

*All three of these were taken up in the second pass below, and the first two
turned out to be real. §7 is kept as written so the record shows what was known
when.*

---

## 9. Second pass — verifying the audit rather than believing it

Every finding in `PROJECT_AUDIT_CURRENT_PROGRESS_2026-07-30.md` was reproduced
before being acted on. That was worth doing in both directions.

### Where the audit was right, and understated

**F-01, the Dockerfile scanner.** Its three fail-open paths are real and were
reproduced. Attacking the same function found **six more**, four of which need no
unusual syntax at all — a tab after `FROM` (the matcher demanded one literal
space, while BuildKit splits on `[\t\v\f\r ]+`), a UTF-8 BOM, `COPY --from=`, and
`RUN --mount=…,from=`. All nine are regressions now, each confirmed against the
old scanner first, with seven deletion sweeps.

**F-03, the trailer rule.** Its (a) and (c) are exact. But the implied remedy —
enforce the documented "every commit" — **must not be implemented literally**,
and this is the most important correction in this pass. `main` carries 51 merge
commits and **none** has a trailer, because `strict: true` branch protection
makes `gh pr update-branch` write them and GitHub authors the `refs/pull/N/merge`
commit that CI actually judges. A literal rule would have failed every pull
request the day it landed. The rule is per **non-merge** commit, the exemption is
deliberate and documented, and the prose was corrected instead.

**F-04, renames.** Confirmed, and understated: the same line also let a
`Governance:` change **delete any file in the repository** by renaming it to a
`docs/work-packages/WP-*.md` name, since every path the check could then see was
an assignment document. Two further defects sat in that one expression — a
non-ASCII path was C-quoted into a false refusal, and `.map(str::trim)`
normalised a leading space onto an owned path.

### Where the audit was wrong or overstated

- **F-03(b)** — "prose counts" is not right. Mid-sentence prose and markdown
  blockquotes were already refused; what counted was an *indented or fenced*
  example. The mechanism was still wrong, so the fix stands.
- **F-03(d)** — "merge commits are silently exempt or unconsidered" is **false**.
  They were in the range and their bodies were parsed; a merge commit's own
  trailer could satisfy the gate. They are exempt *now*, which is a deliberate
  behaviour change, not the removal of an oversight.
- **F-06 claim 2, as applied to `AGENTS.md`** — that line is an imperative to
  authors, not a claim about enforcement. Clarified, not corrected.
- **F-06 claim 6** — largely overstated. HANDOFF §8 vouches for the README's
  *"Current status"* section, which is still accurate; the rows that drifted are
  in a different section, and the top note already declares §7 superseded.
- **F-06 claim 4** — substantively true but loose: the label "1b" never appears
  in `docs/traceability/WP-030.md`.

### What the audit did not reach — in the fix it was reviewing

Three defects in the branches proposed in §3 of this document, found by attacking
them:

1. **The manifest predicate was lexical.** Any file *named* `Cargo.toml` — a
   note, a fixture, a symlink — anywhere a package already owned would unlock the
   root lockfile. The third lexical predicate standing in for a semantic fact in
   two days. `cargo metadata` is now asked which manifests are members.
2. **The nesting guard was a writable proxy.** It matched a manifest to the
   nearest lockfile *file*, so deleting `fuzz/Cargo.lock` in one pull request let
   `fuzz/Cargo.toml` vouch for the root lock in the next while `fuzz` stayed
   excluded. Membership now comes from `exclude`.
3. **Any document could declare any file generated**, granting every package the
   exemption in a change touching nothing but assignment documents. A document
   may now only declare a path generated if it also owns it.

And the one that mattered most: **the shell scaffold was still red under
`cargo xtask ci`.** `verify_path_ownership` did not count a matching
`owned-paths-reserved` claim as coverage, so the first commit inside WP-030's own
reservation passed the change gate and failed the inventory. §3's claim that
"`cargo xtask ci` is never red in between" was false when written. It is true
now, and was checked by building the scaffold rather than asserted again:

```text
verify-change-ownership: 3 path(s) belong to WP-030 as assigned at <base>; 1 regenerated, not authored
  regenerated: Cargo.lock
cargo xtask ci -> GREEN
```

Two of my own tests were caught by deletion sweeps as unable to fail: a merge
fixture that merged an ancestor (which creates no commit at all), and an
assertion about a commented-out `[lints]` stanza that passes with or without the
comment stripping it was meant to exercise. Both are fixed, and the second is
labelled honestly rather than left looking like evidence.

## 10. What is deliberately not fixed, and why

**Not this package's to touch.** The ownership gate refuses these, working as
intended:

| Item | Owner | What is needed |
| --- | --- | --- |
| `docs/traceability/WP-020.md` header stops at 2b while its table cites 2c | WP-020 | one-line header fix |
| `docs/traceability/WP-030.md` header omits 1a/1b | WP-030 | one-line header fix |
| "the root handle outlives the target handles because `Authorization` owns both" — `crates/fixtures/src/interlock.rs:245`, `docs/work-packages/WP-020.md:467` | WP-020 | **The claim is false.** `into_targets(self)` moves the targets out and drops the root first. The *code* is fine: containment is established at `openat` time and is a property of the returned descriptor, not something the directory handle maintains afterwards. Replace the rationale — the root is still worth holding, but for the narrower reason already in the second half of that comment: it denies a consumer a root path to reopen by name. The CHANGELOG half is WP-000's and **is** corrected here |
| WP-020 holds no share of `README.md` or `CHANGELOG.md` | governance | which is *why* its rows drift and why WP-000 keeps repairing them. Consider granting it the sub-file share WP-030 has |

**Deferred, with reasons:**

- **Generated traceability (issue #39, Section 11.7)** — still unassigned, still
  the oldest open gap, and this change added two more hand-maintained rows to the
  very file that proves the point.
- **Windows interlock (issue #51)** — untouched, still blocking Windows Tier 2.
  Keep "Unix closed, Windows open" precise; do not let a status row round it.
- **npm lockfile discovery** — `cross_language` audits `packages/canonical` by
  name, so a frontend lockfile under `apps/desktop/` would be scanned by nobody.
  Recorded in WP-030's increment-2 checklist so it is decided *before* a frontend
  lands.
- **The Tauri dependency tree** — permitting lockfile churn is not permitting
  several hundred crates through `deny.toml`'s allow-list. Its own step.

## 11. Where to attack the second pass

- `derivation_is_plausible` trusts `cargo metadata` on the **working tree**, not
  the base. A change editing root `Cargo.toml` could make its own file a
  "manifest" — but only a package allowed to edit root `Cargo.toml` can, and
  today that is WP-000 and WP-030 alone. I decided against reading the base,
  because the legitimate case (WP-030 adding the shell member) *needs* the
  post-change answer. That is a decision, not an oversight.
- `inherits_workspace_lints` reads manifest text. It fails closed — an
  unrecognised spelling refuses a manifest rather than passing it — but it is the
  fifth lexical predicate here and deserves the same suspicion as the other four.
- The lockfile rule is per **pull request**, not per commit: a re-pin committed
  separately from a manifest change passes if both are in the range. Tightening
  it needs a decision about what a legitimate "manifest, then lock" split does.
- Merge commits are now exempt from declaring anything. Correct for the commits
  GitHub authors, but nothing checks that a merge is empty of its own changes, so
  a human-authored merge could carry content undeclared.
