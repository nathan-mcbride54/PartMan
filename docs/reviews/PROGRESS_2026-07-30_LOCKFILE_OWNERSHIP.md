# Progress notes — 2026-07-30, lockfile ownership

For the reviewer. This covers one phase: the attempt to start WP-030 increment 2,
what the attempt found, and the two changes proposed in response. It is not
normative and does not restate `AGENT_BUILD_SPEC.md` or `AGENTS.md`.

Base for everything below is `02ec952`, with `cargo xtask ci` green (190 tests).

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
